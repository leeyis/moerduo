use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use rusqlite::Connection;
use serde::Serialize;

#[derive(Serialize)]
pub struct Statistics {
    pub total_audio_count: i64,
    pub total_play_count: i64,
    pub total_play_duration: i64,
    pub this_week_play_count: i64,
    pub this_month_play_count: i64,
}

#[derive(Serialize)]
pub struct TopAudio {
    pub id: i64,
    pub name: String,
    pub play_count: i64,
    pub duration: i64,
}

#[tauri::command]
pub async fn get_statistics(
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<Statistics, String> {
    let conn = conn.lock().await;

    // 获取音频总数
    let total_audio_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM audio_files", [], |row| row.get(0))
        .unwrap_or(0);

    // 获取总播放次数
    let total_play_count: i64 = conn
        .query_row("SELECT SUM(play_count) FROM audio_files", [], |row| row.get(0))
        .unwrap_or(0);

    // 估算总播放时长（播放次数 × 平均时长）
    let total_play_duration: i64 = conn
        .query_row(
            "SELECT SUM(play_count * duration) FROM audio_files",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // 获取本周播放次数（从execution_history表）
    let this_week_play_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM execution_history
             WHERE execution_time >= datetime('now', '-7 days')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // 获取本月播放次数
    let this_month_play_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM execution_history
             WHERE execution_time >= datetime('now', '-30 days')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(Statistics {
        total_audio_count,
        total_play_count,
        total_play_duration,
        this_week_play_count,
        this_month_play_count,
    })
}

#[tauri::command]
pub async fn get_top_audios(
    limit: i64,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<Vec<TopAudio>, String> {
    let conn = conn.lock().await;

    let mut stmt = conn
        .prepare(
            "SELECT id, original_name, play_count, duration
             FROM audio_files
             WHERE play_count > 0
             ORDER BY play_count DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let audios = stmt
        .query_map([limit], |row| {
            Ok(TopAudio {
                id: row.get(0)?,
                name: row.get(1)?,
                play_count: row.get(2)?,
                duration: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(audios)
}

#[derive(Serialize)]
pub struct DailyActivity {
    pub date: String,
    pub play_count: i64,
}

#[tauri::command]
pub async fn get_daily_activity(
    days: i64,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<Vec<DailyActivity>, String> {
    let conn = conn.lock().await;

    let mut stmt = conn
        .prepare(
            "SELECT DATE(execution_time) as date, COUNT(*) as count
             FROM execution_history
             WHERE execution_time >= datetime('now', ?1)
             GROUP BY DATE(execution_time)
             ORDER BY date DESC",
        )
        .map_err(|e| e.to_string())?;

    let param = format!("-{} days", days);
    let activities = stmt
        .query_map([param], |row| {
            Ok(DailyActivity {
                date: row.get(0)?,
                play_count: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(activities)
}

#[derive(Serialize)]
pub struct MonthlyPlayback {
    pub date: String,
    pub play_count: i64,
    pub playlists: Vec<PlaylistPlayInfo>,
}

#[derive(Serialize)]
pub struct PlaylistPlayInfo {
    pub playlist_name: Option<String>,
    pub audio_count: i64,
}

#[tauri::command]
pub async fn get_monthly_playback(
    year: i32,
    month: i32,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<Vec<MonthlyPlayback>, String> {
    let conn = conn.lock().await;

    // 构建日期范围
    let start_date = format!("{:04}-{:02}-01", year, month);
    let end_date = if month == 12 {
        format!("{:04}-01-01", year + 1)
    } else {
        format!("{:04}-{:02}-01", year, month + 1)
    };

    // 获取该月的所有日期及其播放记录
    let mut stmt = conn
        .prepare(
            "SELECT DATE(play_time) as date,
                    COALESCE(playlist_name, '单独播放') as playlist_name,
                    COUNT(*) as audio_count
             FROM playback_history
             WHERE DATE(play_time) >= ?1 AND DATE(play_time) < ?2
             GROUP BY DATE(play_time), playlist_name
             ORDER BY date DESC, audio_count DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([&start_date, &end_date], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 按日期分组
    let mut date_map: std::collections::HashMap<String, Vec<PlaylistPlayInfo>> = std::collections::HashMap::new();

    for (date, playlist_name, audio_count) in rows {
        let playlist_name = if playlist_name == "单独播放" {
            None
        } else {
            Some(playlist_name)
        };

        date_map
            .entry(date)
            .or_insert_with(Vec::new)
            .push(PlaylistPlayInfo {
                playlist_name,
                audio_count,
            });
    }

    // 转换为结果格式
    let mut result: Vec<MonthlyPlayback> = date_map
        .into_iter()
        .map(|(date, playlists)| {
            let play_count = playlists.iter().map(|p| p.audio_count).sum();
            MonthlyPlayback {
                date,
                play_count,
                playlists,
            }
        })
        .collect();

    result.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(result)
}
