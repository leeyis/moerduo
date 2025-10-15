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
                    st.is_enabled, st.priority, st.created_date
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
                is_enabled: row.get(10)?,
                priority: row.get(11)?,
                created_date: row.get(12)?,
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
    priority: i64,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<i64, String> {
    let conn = conn.lock().await;
    conn.execute(
        "INSERT INTO scheduled_tasks (name, hour, minute, repeat_mode, custom_days, playlist_id, volume, fade_in_duration, priority)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        (
            &name,
            hour,
            minute,
            &repeat_mode,
            &custom_days,
            playlist_id,
            volume,
            fade_in_duration,
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
    priority: i64,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    let conn = conn.lock().await;
    conn.execute(
        "UPDATE scheduled_tasks SET name = ?1, hour = ?2, minute = ?3, repeat_mode = ?4,
         custom_days = ?5, playlist_id = ?6, volume = ?7, fade_in_duration = ?8, priority = ?9
         WHERE id = ?10",
        (
            &name,
            hour,
            minute,
            &repeat_mode,
            &custom_days,
            playlist_id,
            volume,
            fade_in_duration,
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
