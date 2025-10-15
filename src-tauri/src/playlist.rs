use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::Connection;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub play_mode: String,
    pub created_date: String,
    pub updated_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub id: i64,
    pub playlist_id: i64,
    pub audio_id: i64,
    pub sort_order: i64,
    pub audio_name: String,
    pub duration: i64,
}

#[tauri::command]
pub async fn get_playlists(
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<Vec<Playlist>, String> {
    let conn = conn.lock().await;
    let mut stmt = conn
        .prepare("SELECT id, name, play_mode, created_date, updated_date FROM playlists ORDER BY created_date DESC")
        .map_err(|e| e.to_string())?;

    let playlists = stmt
        .query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                play_mode: row.get(2)?,
                created_date: row.get(3)?,
                updated_date: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(playlists)
}

#[tauri::command]
pub async fn create_playlist(
    name: String,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<i64, String> {
    let conn = conn.lock().await;
    conn.execute(
        "INSERT INTO playlists (name) VALUES (?1)",
        [&name],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    Ok(id)
}

#[tauri::command]
pub async fn delete_playlist(
    id: i64,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    let conn = conn.lock().await;
    conn.execute("DELETE FROM playlists WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn set_playlist_mode(
    playlist_id: i64,
    mode: String,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    let conn = conn.lock().await;
    conn.execute(
        "UPDATE playlists SET play_mode = ?1, updated_date = datetime('now') WHERE id = ?2",
        (&mode, playlist_id),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_playlist_items(
    playlist_id: i64,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<Vec<PlaylistItem>, String> {
    let conn = conn.lock().await;
    let mut stmt = conn
        .prepare(
            "SELECT pi.id, pi.playlist_id, pi.audio_id, pi.sort_order, af.original_name, af.duration
             FROM playlist_items pi
             JOIN audio_files af ON pi.audio_id = af.id
             WHERE pi.playlist_id = ?1
             ORDER BY pi.sort_order"
        )
        .map_err(|e| e.to_string())?;

    let items = stmt
        .query_map([playlist_id], |row| {
            Ok(PlaylistItem {
                id: row.get(0)?,
                playlist_id: row.get(1)?,
                audio_id: row.get(2)?,
                sort_order: row.get(3)?,
                audio_name: row.get(4)?,
                duration: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(items)
}

#[tauri::command]
pub async fn add_to_playlist(
    playlist_id: i64,
    audio_id: i64,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    let conn = conn.lock().await;

    // 获取当前最大排序值
    let max_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM playlist_items WHERE playlist_id = ?1",
            [playlist_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO playlist_items (playlist_id, audio_id, sort_order) VALUES (?1, ?2, ?3)",
        (playlist_id, audio_id, max_order + 1),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn remove_from_playlist(
    id: i64,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    let conn = conn.lock().await;
    conn.execute("DELETE FROM playlist_items WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn check_playlist_tasks(
    playlist_id: i64,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<Vec<String>, String> {
    let conn = conn.lock().await;
    let mut stmt = conn
        .prepare(
            "SELECT name FROM scheduled_tasks WHERE playlist_id = ?1 AND is_enabled = 1"
        )
        .map_err(|e| e.to_string())?;

    let task_names: Vec<String> = stmt
        .query_map([playlist_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(task_names)
}
