use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::Connection;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: i64,
    pub name: String,
    pub hour: i64,
    pub minute: i64,
    pub repeat_mode: String,
    pub custom_days: Option<String>,
    pub playlist_id: i64,
    pub playlist_name: String,
    pub volume: i64,
    pub fade_in_duration: i64,
    pub duration_minutes: Option<i64>,
    pub is_enabled: bool,
    pub priority: i64,
    pub created_date: String,
}

#[tauri::command]
pub async fn get_scheduled_tasks(
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<Vec<ScheduledTask>, String> {
    let conn = conn.lock().await;
    let mut stmt = conn
        .prepare(
            "SELECT st.id, st.name, st.hour, st.minute, st.repeat_mode, st.custom_days,
                    st.playlist_id, p.name as playlist_name, st.volume, st.fade_in_duration,
                    st.duration_minutes, st.is_enabled, st.priority, st.created_date
             FROM scheduled_tasks st
             JOIN playlists p ON st.playlist_id = p.id
             ORDER BY st.hour, st.minute"
        )
        .map_err(|e| e.to_string())?;

    let tasks = stmt
        .query_map([], |row| {
            Ok(ScheduledTask {
                id: row.get(0)?,
                name: row.get(1)?,
                hour: row.get(2)?,
                minute: row.get(3)?,
                repeat_mode: row.get(4)?,
                custom_days: row.get(5)?,
                playlist_id: row.get(6)?,
                playlist_name: row.get(7)?,
                volume: row.get(8)?,
                fade_in_duration: row.get(9)?,
                duration_minutes: row.get(10)?,
                is_enabled: row.get(11)?,
                priority: row.get(12)?,
                created_date: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(tasks)
}

#[tauri::command]
pub async fn create_scheduled_task(
    name: String,
    hour: i64,
    minute: i64,
    repeat_mode: String,
    custom_days: Option<String>,
    playlist_id: i64,
    volume: i64,
    fade_in_duration: i64,
    duration_minutes: Option<i64>,
    priority: i64,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<i64, String> {
    let conn = conn.lock().await;
    conn.execute(
        "INSERT INTO scheduled_tasks (name, hour, minute, repeat_mode, custom_days, playlist_id, volume, fade_in_duration, duration_minutes, priority)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (
            &name,
            hour,
            minute,
            &repeat_mode,
            &custom_days,
            playlist_id,
            volume,
            fade_in_duration,
            duration_minutes,
            priority,
        ),
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    Ok(id)
}

#[tauri::command]
pub async fn update_scheduled_task(
    id: i64,
    name: String,
    hour: i64,
    minute: i64,
    repeat_mode: String,
    custom_days: Option<String>,
    playlist_id: i64,
    volume: i64,
    fade_in_duration: i64,
    duration_minutes: Option<i64>,
    priority: i64,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    let conn = conn.lock().await;
    conn.execute(
        "UPDATE scheduled_tasks SET name = ?1, hour = ?2, minute = ?3, repeat_mode = ?4,
         custom_days = ?5, playlist_id = ?6, volume = ?7, fade_in_duration = ?8, duration_minutes = ?9, priority = ?10
         WHERE id = ?11",
        (
            &name,
            hour,
            minute,
            &repeat_mode,
            &custom_days,
            playlist_id,
            volume,
            fade_in_duration,
            duration_minutes,
            priority,
            id,
        ),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_scheduled_task(
    id: i64,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    let conn = conn.lock().await;
    conn.execute("DELETE FROM scheduled_tasks WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn toggle_scheduled_task(
    id: i64,
    enabled: bool,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    let conn = conn.lock().await;
    conn.execute(
        "UPDATE scheduled_tasks SET is_enabled = ?1 WHERE id = ?2",
        (enabled, id),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct TaskConflict {
    pub task_id: i64,
    pub task_name: String,
    pub hour: i64,
    pub minute: i64,
}

// 检查两个任务的重复模式是否可能在同一天执行
fn check_repeat_conflict(mode1: &str, days1: &Option<String>, mode2: &str, days2: &Option<String>) -> bool {
    // 如果任一任务是 "once"，则不考虑重复冲突（同一天的时间冲突仍需检查）
    if mode1 == "once" || mode2 == "once" {
        return true;
    }

    // daily 与任何模式都冲突
    if mode1 == "daily" || mode2 == "daily" {
        return true;
    }

    // weekday 与 weekday 或 custom(包含工作日) 冲突
    if mode1 == "weekday" {
        if mode2 == "weekday" {
            return true;
        }
        if mode2 == "custom" {
            if let Some(days) = days2 {
                if let Ok(custom_days) = serde_json::from_str::<Vec<i32>>(days) {
                    // 检查是否包含1-5（周一到周五）
                    if custom_days.iter().any(|d| *d >= 1 && *d <= 5) {
                        return true;
                    }
                }
            }
        }
        return false;
    }

    // weekend 与 weekend 或 custom(包含周末) 冲突
    if mode1 == "weekend" {
        if mode2 == "weekend" {
            return true;
        }
        if mode2 == "custom" {
            if let Some(days) = days2 {
                if let Ok(custom_days) = serde_json::from_str::<Vec<i32>>(days) {
                    // 检查是否包含0或6（周日或周六）
                    if custom_days.iter().any(|d| *d == 0 || *d == 6) {
                        return true;
                    }
                }
            }
        }
        return false;
    }

    // custom 模式的冲突检查
    if mode1 == "custom" && mode2 == "custom" {
        if let (Some(days1_str), Some(days2_str)) = (days1, days2) {
            if let (Ok(days1_vec), Ok(days2_vec)) = (
                serde_json::from_str::<Vec<i32>>(days1_str),
                serde_json::from_str::<Vec<i32>>(days2_str),
            ) {
                // 检查是否有交集
                return days1_vec.iter().any(|d| days2_vec.contains(d));
            }
        }
    }

    if mode1 == "custom" {
        if let Some(days1_str) = days1 {
            if let Ok(days1_vec) = serde_json::from_str::<Vec<i32>>(days1_str) {
                if mode2 == "weekday" {
                    return days1_vec.iter().any(|d| *d >= 1 && *d <= 5);
                }
                if mode2 == "weekend" {
                    return days1_vec.iter().any(|d| *d == 0 || *d == 6);
                }
            }
        }
    }

    false
}

// 检查任务时间冲突
#[tauri::command]
pub async fn check_task_conflicts(
    task_id: Option<i64>, // 如果是更新任务，传入任务ID；如果是新建任务，传入None
    hour: i64,
    minute: i64,
    repeat_mode: String,
    custom_days: Option<String>,
    duration_minutes: Option<i64>,
    playlist_id: i64,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<Vec<TaskConflict>, String> {
    let conn = conn.lock().await;

    // 获取播放列表的总时长（如果没有设置 duration_minutes）
    let estimated_duration = if let Some(dur) = duration_minutes {
        dur
    } else {
        // 计算播放列表的总时长（秒转分钟）
        let total_seconds: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(af.duration), 0) FROM playlist_items pi
                 JOIN audio_files af ON pi.audio_id = af.id
                 WHERE pi.playlist_id = ?1",
                [playlist_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        (total_seconds + 59) / 60 // 向上取整到分钟
    };

    // 计算任务的开始和结束时间（分钟）
    let start_time = hour * 60 + minute;
    let end_time = start_time + estimated_duration;

    // 查询所有启用的任务
    let mut stmt = conn
        .prepare(
            "SELECT st.id, st.name, st.hour, st.minute, st.repeat_mode, st.custom_days,
                    st.duration_minutes, st.playlist_id
             FROM scheduled_tasks st
             WHERE st.is_enabled = 1"
        )
        .map_err(|e| e.to_string())?;

    let existing_tasks: Vec<(i64, String, i64, i64, String, Option<String>, Option<i64>, i64)> = stmt
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
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut conflicts = Vec::new();

    for (id, name, h, m, mode, days, dur_min, pl_id) in existing_tasks {
        // 跳过自己（更新任务时）
        if let Some(current_id) = task_id {
            if id == current_id {
                continue;
            }
        }

        // 检查重复模式是否可能冲突
        if !check_repeat_conflict(&repeat_mode, &custom_days, &mode, &days) {
            continue;
        }

        // 计算现有任务的时长
        let existing_duration = if let Some(dur) = dur_min {
            dur
        } else {
            let total_seconds: i64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(af.duration), 0) FROM playlist_items pi
                     JOIN audio_files af ON pi.audio_id = af.id
                     WHERE pi.playlist_id = ?1",
                    [pl_id],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            (total_seconds + 59) / 60
        };

        let existing_start = h * 60 + m;
        let existing_end = existing_start + existing_duration;

        // 检查时间段是否重叠
        // 两个时间段重叠的条件：start1 < end2 && start2 < end1
        if start_time < existing_end && existing_start < end_time {
            conflicts.push(TaskConflict {
                task_id: id,
                task_name: name,
                hour: h,
                minute: m,
            });
        }
    }

    Ok(conflicts)
}
