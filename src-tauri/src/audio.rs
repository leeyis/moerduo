use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::Connection;
use tauri::State;
use anyhow::Result;
use std::fs;
use std::io::BufReader;
use rodio::{Decoder, Source};

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioFile {
    pub id: i64,
    pub filename: String,
    pub original_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub duration: i64,
    pub format: String,
    pub upload_date: String,
    pub play_count: i64,
    pub last_played: Option<String>,
}

/// 获取音频文件的真实时长（秒）
fn get_audio_duration(file_path: &std::path::Path) -> i64 {
    match fs::File::open(file_path) {
        Ok(file) => {
            match Decoder::new(BufReader::new(file)) {
                Ok(source) => {
                    // 尝试获取总时长
                    if let Some(duration) = source.total_duration() {
                        duration.as_secs() as i64
                    } else {
                        // 如果无法获取，返回默认值
                        180
                    }
                }
                Err(_) => {
                    // 解码失败，返回默认值
                    180
                }
            }
        }
        Err(_) => {
            // 文件打开失败，返回默认值
            180
        }
    }
}

#[tauri::command]
pub async fn upload_audio_file(
    file_path: String,
    conn: State<'_, Arc<Mutex<Connection>>>,
    audio_dir: State<'_, PathBuf>,
) -> Result<i64, String> {
    let src_path = PathBuf::from(&file_path);

    if !src_path.exists() {
        return Err("文件不存在".to_string());
    }

    let original_name = src_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("无效的文件名")?
        .to_string();

    let extension = src_path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or("无法获取文件扩展名")?
        .to_lowercase();

    // 验证音频格式
    if !["mp3", "wav", "ogg", "flac", "m4a"].contains(&extension.as_str()) {
        return Err("不支持的音频格式".to_string());
    }

    // 获取文件大小
    let metadata = std::fs::metadata(&src_path).map_err(|e| e.to_string())?;
    let file_size = metadata.len() as i64;

    // 生成唯一文件名
    let filename = format!(
        "{}_{}.{}",
        chrono::Local::now().format("%Y%m%d_%H%M%S"),
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap(),
        extension
    );

    let dest_path = audio_dir.join(&filename);

    // 复制文件
    std::fs::copy(&src_path, &dest_path).map_err(|e| e.to_string())?;

    // 获取音频真实时长
    let duration = get_audio_duration(&dest_path);

    // 保存到数据库
    let conn = conn.lock().await;
    conn.execute(
        "INSERT INTO audio_files (filename, original_name, file_path, file_size, duration, format)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &filename,
            &original_name,
            dest_path.to_str().unwrap(),
            file_size,
            duration,
            &extension,
        ),
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    Ok(id)
}

#[tauri::command]
pub async fn get_audio_files(
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<Vec<AudioFile>, String> {
    let conn = conn.lock().await;
    let mut stmt = conn
        .prepare("SELECT id, filename, original_name, file_path, file_size, duration, format, upload_date, play_count, last_played FROM audio_files ORDER BY id DESC")
        .map_err(|e| e.to_string())?;

    let files = stmt
        .query_map([], |row| {
            Ok(AudioFile {
                id: row.get(0)?,
                filename: row.get(1)?,
                original_name: row.get(2)?,
                file_path: row.get(3)?,
                file_size: row.get(4)?,
                duration: row.get(5)?,
                format: row.get(6)?,
                upload_date: row.get(7)?,
                play_count: row.get(8)?,
                last_played: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(files)
}

#[tauri::command]
pub async fn delete_audio_file(
    id: i64,
    delete_physical_file: bool,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    let conn = conn.lock().await;

    // 获取文件路径
    let file_path: String = conn
        .query_row(
            "SELECT file_path FROM audio_files WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // 根据用户选择决定是否删除物理文件
    if delete_physical_file {
        if let Err(e) = std::fs::remove_file(&file_path) {
            eprintln!("删除物理文件失败: {}", e);
            // 注意：即使物理删除失败，仍然从数据库中删除记录
        }
    }

    // 从数据库删除
    conn.execute("DELETE FROM audio_files WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct ScanResult {
    pub found_files: i32,
    pub added_files: i32,
    pub skipped_files: i32,
    pub error_files: i32,
}

#[tauri::command]
pub async fn scan_audio_directory(
    conn: State<'_, Arc<Mutex<Connection>>>,
    audio_dir: State<'_, PathBuf>,
) -> Result<ScanResult, String> {
    // 从数据库读取用户配置的音频路径
    let scan_path = {
        let conn_guard = conn.lock().await;
        let custom_path: Option<String> = conn_guard
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'audio_path'",
                [],
                |row| row.get(0),
            )
            .ok();

        if let Some(path_str) = custom_path {
            // 移除可能的引号
            let path_str = path_str.trim_matches('"');
            PathBuf::from(path_str)
        } else {
            // 使用默认路径
            audio_dir.as_path().to_path_buf()
        }
    };

    if !scan_path.exists() {
        return Err(format!("音频目录不存在: {}", scan_path.display()));
    }

    let mut found_files = 0;
    let mut added_files = 0;
    let mut skipped_files = 0;
    let mut error_files = 0;

    // 支持的音频格式
    let supported_formats = ["mp3", "wav", "ogg", "flac", "m4a"];

    // 读取目录中的所有文件
    let entries = match fs::read_dir(&scan_path) {
        Ok(entries) => entries,
        Err(e) => return Err(format!("读取目录失败: {}", e)),
    };

    let conn_guard = conn.lock().await;

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                error_files += 1;
                continue;
            }
        };

        let path = entry.path();

        // 只处理文件，跳过目录
        if !path.is_file() {
            continue;
        }

        // 检查文件扩展名
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                if supported_formats.contains(&ext_str.to_lowercase().as_str()) {
                    found_files += 1;

                    // 获取文件信息
                    let original_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let file_size = match fs::metadata(&path) {
                        Ok(metadata) => metadata.len() as i64,
                        Err(_) => {
                            error_files += 1;
                            continue;
                        }
                    };

                    let file_path_str = path.to_string_lossy().to_string();

                    // 检查文件是否已存在于数据库中
                    let existing_count: i64 = conn_guard
                        .query_row(
                            "SELECT COUNT(*) FROM audio_files WHERE file_path = ?1",
                            [&file_path_str],
                            |row| row.get(0),
                        )
                        .unwrap_or(0);

                    if existing_count > 0 {
                        skipped_files += 1;
                        continue;
                    }

                    // 添加到数据库
                    let filename = format!(
                        "{}_{}.{}",
                        chrono::Local::now().format("%Y%m%d_%H%M%S"),
                        uuid::Uuid::new_v4().to_string().split('-').next().unwrap(),
                        ext_str.to_lowercase()
                    );

                    // 获取音频真实时长
                    let duration = get_audio_duration(&path);

                    match conn_guard.execute(
                        "INSERT INTO audio_files (filename, original_name, file_path, file_size, duration, format, upload_date)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        (
                            &filename,
                            &original_name,
                            &file_path_str,
                            file_size,
                            duration,
                            &ext_str.to_lowercase(),
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        ),
                    ) {
                        Ok(_) => added_files += 1,
                        Err(_) => error_files += 1,
                    }
                }
            }
        }
    }

    Ok(ScanResult {
        found_files,
        added_files,
        skipped_files,
        error_files,
    })
}
