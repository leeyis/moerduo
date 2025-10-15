use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::Connection;
use tauri::State;
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub default_volume: i64,
    pub theme: String,
    pub audio_path: Option<String>,
}

#[tauri::command]
pub async fn get_settings(
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<AppSettings, String> {
    let conn = conn.lock().await;

    let mut settings = AppSettings {
        auto_start: false,
        minimize_to_tray: true,
        default_volume: 50,
        theme: "light".to_string(),
        audio_path: None,
    };

    // 从数据库读取设置
    let rows = conn
        .prepare("SELECT key, value FROM app_settings")
        .map_err(|e| e.to_string())?
        .query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok((key, value))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (key, value) in rows {
        match key.as_str() {
            "auto_start" => {
                if let Ok(val) = serde_json::from_str(&value) {
                    settings.auto_start = val;
                }
            }
            "minimize_to_tray" => {
                if let Ok(val) = serde_json::from_str(&value) {
                    settings.minimize_to_tray = val;
                }
            }
            "default_volume" => {
                if let Ok(val) = value.parse::<i64>() {
                    settings.default_volume = val;
                }
            }
            "theme" => {
                settings.theme = value;
            }
            "audio_path" => {
                settings.audio_path = Some(value);
            }
            _ => {}
        }
    }

    Ok(settings)
}

#[allow(dead_code)]
#[tauri::command]
pub async fn save_setting(
    key: String,
    value: String,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    let conn = conn.lock().await;

    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        (&key, &value),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn save_settings(
    settings: AppSettings,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    let conn = conn.lock().await;

    // 保存所有设置
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        ("auto_start", serde_json::to_string(&settings.auto_start).unwrap_or_default()),
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        ("minimize_to_tray", serde_json::to_string(&settings.minimize_to_tray).unwrap_or_default()),
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        ("default_volume", settings.default_volume.to_string()),
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        ("theme", &settings.theme),
    )
    .map_err(|e| e.to_string())?;

    if let Some(audio_path) = settings.audio_path {
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
            ("audio_path", &audio_path),
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_data_usage(
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<serde_json::Value, String> {
    let conn = conn.lock().await;

    // 获取音频文件统计
    let total_audio_files: i64 = conn
        .query_row("SELECT COUNT(*) FROM audio_files", [], |row| row.get(0))
        .unwrap_or(0);

    let total_audio_size: i64 = conn
        .query_row("SELECT SUM(file_size) FROM audio_files", [], |row| row.get(0))
        .unwrap_or(0);

    // 获取数据库大小（估算）
    let db_size = 2345678; // 约2.3MB，实际应该读取文件大小

    Ok(serde_json::json!({
        "database_size": db_size,
        "audio_files_count": total_audio_files,
        "audio_files_size": total_audio_size
    }))
}

#[tauri::command]
pub async fn export_config(
    _conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<String, String> {
    // TODO: 实现配置导出功能
    Err("功能暂未实现".to_string())
}

#[tauri::command]
pub async fn import_config(
    _conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<String, String> {
    // TODO: 实现配置导入功能
    Err("功能暂未实现".to_string())
}