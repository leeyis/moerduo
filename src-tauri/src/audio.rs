use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::Connection;
use tauri::{State, AppHandle, Manager};
use anyhow::Result;
use std::fs;
use std::io::BufReader;
use std::process::Command;
use rodio::{Decoder, Source};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::formats::FormatOptions;

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
    // 使用 symphonia 获取准确的音频时长
    match fs::File::open(file_path) {
        Ok(file) => {
            let mss = MediaSourceStream::new(Box::new(file), Default::default());

            let mut hint = Hint::new();
            if let Some(extension) = file_path.extension() {
                if let Some(ext_str) = extension.to_str() {
                    hint.with_extension(ext_str);
                }
            }

            let format_opts = FormatOptions::default();
            let metadata_opts = MetadataOptions::default();

            match symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
                Ok(probed) => {
                    let format = probed.format;

                    // 尝试从默认音轨获取时长
                    if let Some(track) = format.default_track() {
                        if let Some(timebase) = track.codec_params.time_base {
                            if let Some(n_frames) = track.codec_params.n_frames {
                                // 使用时间基数和帧数计算准确时长
                                let duration_secs = (n_frames as f64 * timebase.numer as f64)
                                    / timebase.denom as f64;
                                return duration_secs.ceil() as i64;
                            }
                        }

                        // 如果无法从帧数计算，尝试从采样率和样本数计算
                        if let Some(sample_rate) = track.codec_params.sample_rate {
                            if let Some(n_frames) = track.codec_params.n_frames {
                                let duration_secs = n_frames as f64 / sample_rate as f64;
                                return duration_secs.ceil() as i64;
                            }
                        }
                    }

                    // 如果上述方法都失败，尝试使用 rodio 作为备选
                    if let Ok(file) = fs::File::open(file_path) {
                        if let Ok(source) = Decoder::new(BufReader::new(file)) {
                            if let Some(duration) = source.total_duration() {
                                return duration.as_secs() as i64;
                            }
                        }
                    }

                    180 // 默认值
                }
                Err(_) => {
                    // symphonia 失败，尝试使用 rodio 作为备选
                    if let Ok(file) = fs::File::open(file_path) {
                        if let Ok(source) = Decoder::new(BufReader::new(file)) {
                            if let Some(duration) = source.total_duration() {
                                return duration.as_secs() as i64;
                            }
                        }
                    }
                    180 // 默认值
                }
            }
        }
        Err(_) => 180 // 文件打开失败，返回默认值
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

/// 检查FFmpeg是否可用
fn check_ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// 从视频文件提取音频（使用FFmpeg命令行）
#[tauri::command]
pub async fn extract_audio_from_video(
    video_path: String,
    output_filename: String,
    app: AppHandle,
    conn: State<'_, Arc<Mutex<Connection>>>,
    audio_dir: State<'_, PathBuf>,
) -> Result<String, String> {
    // 检查FFmpeg是否可用
    if !check_ffmpeg_available() {
        return Err("FFmpeg未安装或不在PATH中。请安装FFmpeg后重试。\n安装方法：\n1. Windows: 从 https://ffmpeg.org/download.html 下载并添加到PATH\n2. macOS: brew install ffmpeg\n3. Linux: sudo apt install ffmpeg".to_string());
    }

    let input_path = PathBuf::from(&video_path);
    if !input_path.exists() {
        return Err("视频文件不存在".to_string());
    }

    // 生成输出文件名
    let filename = if output_filename.is_empty() {
        let now = chrono::Local::now();
        format!(
            "{}_{}.mp3",
            now.format("%Y%m%d_%H%M%S"),
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
        )
    } else {
        format!("{}.mp3", output_filename)
    };

    let output_path = audio_dir.join(&filename);

    // 发送进度开始事件
    app.emit_all("extract-progress", 0u8).map_err(|e| e.to_string())?;

    // 构建FFmpeg命令
    let mut cmd = Command::new("ffmpeg");
    cmd
        .arg("-i") // 输入文件
        .arg(&video_path)
        .arg("-vn") // 不要视频
        .arg("-acodec") // 音频编码器
        .arg("libmp3lame") // MP3编码器
        .arg("-ab") // 音频比特率
        .arg("128k") // 128kbps
        .arg("-ar") // 音频采样率
        .arg("44100") // 44.1kHz
        .arg("-ac") // 音频声道数
        .arg("2") // 立体声
        .arg("-y") // 覆盖输出文件
        .arg(output_path.to_str().unwrap());

    // 发送进度 10%
    app.emit_all("extract-progress", 10u8).map_err(|e| e.to_string())?;

    // 执行FFmpeg命令
    let output = cmd.output().map_err(|e| format!("执行FFmpeg命令失败: {}", e))?;

    // 发送进度 90%
    app.emit_all("extract-progress", 90u8).map_err(|e| e.to_string())?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg执行失败: {}", error_msg));
    }

    // 检查输出文件是否存在
    if !output_path.exists() {
        return Err("音频提取失败：输出文件不存在".to_string());
    }

    // 发送完成进度
    app.emit_all("extract-progress", 100u8).map_err(|e| e.to_string())?;

    // 获取输出文件信息
    let metadata = std::fs::metadata(&output_path)
        .map_err(|e| format!("无法获取输出文件信息: {}", e))?;
    let file_size = metadata.len() as i64;

    // 获取音频时长
    let duration = get_audio_duration(&output_path);

    // 保存到数据库
    let conn = conn.lock().await;
    conn.execute(
        "INSERT INTO audio_files (filename, original_name, file_path, file_size, duration, format, upload_date)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            &filename,
            &filename,
            output_path.to_str().unwrap(),
            file_size,
            duration,
            "mp3",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        ),
    )
    .map_err(|e| format!("保存到数据库失败: {}", e))?;

    Ok(filename)
}
