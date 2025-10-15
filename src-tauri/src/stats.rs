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
