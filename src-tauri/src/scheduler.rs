use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, interval};
use rusqlite::Connection;
use chrono::{Local, Timelike, Datelike};
use crate::player::AudioPlayer;

pub struct Scheduler {
    db: Arc<Mutex<Connection>>,
    player: Arc<Mutex<AudioPlayer>>,
}

impl Scheduler {
    pub fn new(db: Arc<Mutex<Connection>>, player: Arc<Mutex<AudioPlayer>>) -> Self {
        Self { db, player }
    }

    pub async fn start(&self) {
        let db = self.db.clone();
        let player = self.player.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10)); // 每10秒检查一次，避免漏掉任务

            loop {
                interval.tick().await;

                if let Err(e) = Self::check_and_execute_tasks(db.clone(), player.clone()).await {
                    eprintln!("检查任务失败: {}", e);
                }
            }
        });
    }

    async fn check_and_execute_tasks(
        db: Arc<Mutex<Connection>>,
        player: Arc<Mutex<AudioPlayer>>,
    ) -> Result<(), String> {
        let now = Local::now();
        let current_hour = now.hour() as i64;
        let current_minute = now.minute() as i64;
        let current_weekday = now.weekday().number_from_sunday() as i64; // 0=周日, 1-6=周一到周六

        println!("[Scheduler] 检查时间: {}:{:02}, 星期: {}", current_hour, current_minute, current_weekday);

        // 查询所有启用的任务
        let tasks = {
            let conn = db.lock().await;
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, hour, minute, repeat_mode, custom_days, playlist_id,
                            volume, fade_in_duration, priority
                     FROM scheduled_tasks
                     WHERE is_enabled = 1
                     ORDER BY priority DESC, hour, minute"
                )
                .map_err(|e| e.to_string())?;

            let tasks: Vec<(i64, String, i64, i64, String, Option<String>, i64, i64, i64, i64)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                    ))
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            tasks
        };

        for (task_id, name, hour, minute, repeat_mode, custom_days, playlist_id, volume, fade_in_duration, _priority) in tasks {
            // 检查时间是否匹配（允许当前分钟或前一分钟内执行，避免因检查间隔导致错过）
            let time_matches = if current_minute == 0 {
                // 如果当前是整点，需要检查上一小时的59分
                (hour == current_hour && minute == 0) ||
                (hour == if current_hour == 0 { 23 } else { current_hour - 1 } && minute == 59)
            } else {
                (hour == current_hour && minute == current_minute) ||
                (hour == current_hour && minute == current_minute - 1)
            };

            if !time_matches {
                continue;
            }

            println!("[Scheduler] 发现匹配任务: {} ({}:{:02})", name, hour, minute);

            // 检查是否应该在今天执行
            let should_execute = match repeat_mode.as_str() {
                "daily" => true,
                "weekday" => current_weekday >= 1 && current_weekday <= 5, // 周一到周五
                "weekend" => current_weekday == 0 || current_weekday == 6, // 周六周日
                "custom" => {
                    if let Some(days_str) = custom_days {
                        if let Ok(days) = serde_json::from_str::<Vec<i64>>(&days_str) {
                            days.contains(&current_weekday)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                "once" => {
                    // 仅一次，检查是否已经执行过
                    let conn = db.lock().await;
                    let executed = conn
                        .query_row(
                            "SELECT COUNT(*) FROM execution_history WHERE task_id = ?1",
                            [task_id],
                            |row| row.get::<_, i64>(0),
                        )
                        .unwrap_or(0);
                    executed == 0
                }
                _ => false,
            };

            if !should_execute {
                println!("[Scheduler] 任务 {} 今天不应该执行 (repeat_mode: {})", name, repeat_mode);
                continue;
            }

            // 检查今天是否已经执行过（避免重复执行）
            let already_executed_today = {
                let conn = db.lock().await;
                let today_start = format!("{} 00:00:00", now.format("%Y-%m-%d"));
                let count: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM execution_history
                         WHERE task_id = ?1 AND execution_time >= ?2",
                        (&task_id, &today_start),
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                count > 0
            };

            if already_executed_today {
                println!("[Scheduler] 任务 {} 今天已经执行过了", name);
                continue;
            }

            // 执行任务
            println!("✅ [Scheduler] 执行定时任务: {} (ID: {})", name, task_id);

            // 记录开始执行
            {
                let conn = db.lock().await;
                let _ = conn.execute(
                    "INSERT INTO execution_history (task_id, status, execution_time)
                     VALUES (?1, 'started', datetime('now'))",
                    [task_id],
                );
            }

            // 播放播放列表
            if let Err(e) = Self::play_playlist(
                db.clone(),
                player.clone(),
                playlist_id,
                volume,
                fade_in_duration,
            )
            .await
            {
                eprintln!("播放失败: {}", e);

                // 记录失败
                let conn = db.lock().await;
                let _ = conn.execute(
                    "UPDATE execution_history SET status = 'failed'
                     WHERE task_id = ?1 AND execution_time = (
                         SELECT MAX(execution_time) FROM execution_history WHERE task_id = ?1
                     )",
                    [task_id],
                );
            }
        }

        Ok(())
    }

    async fn play_playlist(
        db: Arc<Mutex<Connection>>,
        player: Arc<Mutex<AudioPlayer>>,
        playlist_id: i64,
        volume: i64,
        fade_in_duration: i64,
    ) -> Result<(), String> {
        // 获取播放列表中的所有音频
        let audio_files = {
            let conn = db.lock().await;
            let mut stmt = conn
                .prepare(
                    "SELECT af.id, af.file_path, af.duration, af.original_name
                     FROM playlist_items pi
                     JOIN audio_files af ON pi.audio_id = af.id
                     WHERE pi.playlist_id = ?1
                     ORDER BY pi.sort_order"
                )
                .map_err(|e| e.to_string())?;

            let files: Vec<(i64, String, i64, String)> = stmt
                .query_map([playlist_id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            files
        };

        if audio_files.is_empty() {
            return Err("播放列表为空".to_string());
        }

        // 设置播放队列
        let audio_ids: Vec<i64> = audio_files.iter().map(|(id, _, _, _)| *id).collect();
        let mut player_guard = player.lock().await;
        player_guard.set_playlist_queue(audio_ids, true); // 标记为自动播放
        drop(player_guard);

        // 播放每个音频文件
        for (audio_id, file_path, duration, audio_name) in audio_files {
            let mut player_guard = player.lock().await;

            // 如果配置了渐强，先设置较低音量
            if fade_in_duration > 0 {
                player_guard.set_volume(0.0);
            } else {
                player_guard.set_volume(volume as f32 / 100.0);
            }

            // 开始播放
            player_guard.play_with_info(&file_path, audio_id, audio_name)?;

            // 实现渐强效果
            if fade_in_duration > 0 {
                let target_volume = volume as f32 / 100.0;
                let steps = fade_in_duration as u64;
                let volume_step = target_volume / steps as f32;

                drop(player_guard); // 释放锁，以便渐强过程中不阻塞

                for i in 0..=steps {
                    let current_volume = volume_step * i as f32;
                    let mut player_guard = player.lock().await;
                    player_guard.set_volume(current_volume.min(target_volume));
                    drop(player_guard);
                    sleep(Duration::from_secs(1)).await;
                }
            } else {
                drop(player_guard);
            }

            // 等待播放完成
            sleep(Duration::from_secs(duration as u64)).await;

            // 更新播放计数
            let conn = db.lock().await;
            let _ = conn.execute(
                "UPDATE audio_files SET play_count = play_count + 1, last_played = datetime('now') WHERE id = ?1",
                [audio_id],
            );
        }

        // 记录完成
        let conn = db.lock().await;
        let _ = conn.execute(
            "UPDATE execution_history SET status = 'completed'
             WHERE execution_time = (
                 SELECT MAX(execution_time) FROM execution_history
             )",
            [],
        );

        Ok(())
    }
}
