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
use std::fs::File;
use std::io::Write;
use zip::ZipArchive;
use dirs::home_dir;
use rodio::{Decoder, Source};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::formats::FormatOptions;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Windows平台的CREATE_NO_WINDOW标志
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 创建一个隐藏窗口的Command
fn create_command(program: &str) -> Command {
    let mut cmd = Command::new(program);

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd
}

/// 创建一个隐藏窗口的Command (PathBuf版本)
fn create_command_from_path(program: &PathBuf) -> Command {
    let mut cmd = Command::new(program);

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd
}

#[derive(Debug, Serialize)]
pub struct FFmpegStatus {
    pub available: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

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

/// 获取FFmpeg可执行文件路径
async fn get_ffmpeg_executable_path(app: Option<&AppHandle>) -> Option<PathBuf> {
    // 首先尝试使用tools目录中的ffmpeg（优先级最高）
    if let Some(app_handle) = app {
        // 开发环境：使用项目根目录下的tools
        #[cfg(debug_assertions)]
        {
            if let Some(exe_dir) = app_handle.path_resolver().app_data_dir() {
                if let Some(project_root) = exe_dir.parent().and_then(|p| p.parent()) {
                    let tools_ffmpeg = project_root.join("tools").join("ffmpeg.exe");
                    if tools_ffmpeg.exists() {
                        if let Ok(output) = create_command_from_path(&tools_ffmpeg).arg("-version").output() {
                            if output.status.success() {
                                return Some(tools_ffmpeg);
                            }
                        }
                    }
                }
            }
        }

        // 生产环境：尝试多个可能的tools目录位置
        #[cfg(not(debug_assertions))]
        {
            // 尝试1: exe所在目录的tools子目录（NSIS安装程序）
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let tools_ffmpeg = exe_dir.join("tools").join("ffmpeg.exe");
                    if tools_ffmpeg.exists() {
                        if let Ok(output) = create_command_from_path(&tools_ffmpeg).arg("-version").output() {
                            if output.status.success() {
                                return Some(tools_ffmpeg);
                            }
                        }
                    }
                }
            }

            // 尝试2: 应用数据目录的tools子目录
            if let Some(app_dir) = app_handle.path_resolver().app_data_dir() {
                let tools_ffmpeg = app_dir.join("tools").join("ffmpeg.exe");
                if tools_ffmpeg.exists() {
                    if let Ok(output) = create_command_from_path(&tools_ffmpeg).arg("-version").output() {
                        if output.status.success() {
                            return Some(tools_ffmpeg);
                        }
                    }
                }
            }

            // 尝试3: 资源目录的tools子目录
            if let Some(resource_dir) = app_handle.path_resolver().resource_dir() {
                let tools_ffmpeg = resource_dir.join("tools").join("ffmpeg.exe");
                if tools_ffmpeg.exists() {
                    if let Ok(output) = create_command_from_path(&tools_ffmpeg).arg("-version").output() {
                        if output.status.success() {
                            return Some(tools_ffmpeg);
                        }
                    }
                }
            }
        }
    }

    // 其次尝试使用PATH中的ffmpeg
    if let Ok(output) = create_command("ffmpeg").arg("-version").output() {
        if output.status.success() {
            return Some(PathBuf::from("ffmpeg"));
        }
    }

    // 最后尝试使用用户目录中安装的ffmpeg
    #[cfg(target_os = "windows")]
    {
        if let Some(home_dir) = home_dir() {
            let local_ffmpeg = home_dir.join("ffmpeg").join("bin").join("ffmpeg.exe");
            if local_ffmpeg.exists() {
                if let Ok(output) = create_command_from_path(&local_ffmpeg).arg("-version").output() {
                    if output.status.success() {
                        return Some(local_ffmpeg);
                    }
                }
            }
        }
    }

    None
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
    // 获取FFmpeg可执行文件路径
    let ffmpeg_path = get_ffmpeg_executable_path(Some(&app)).await
        .ok_or("FFmpeg未安装。请将ffmpeg.exe放入tools目录，或点击\"一键安装FFmpeg\"按钮进行安装".to_string())?;

    let input_path = PathBuf::from(&video_path);
    if !input_path.exists() {
        return Err("视频文件不存在".to_string());
    }

    // 获取视频文件的原始名称（不含扩展名）
    let video_original_name = input_path
        .file_stem()
        .and_then(|n| n.to_str())
        .ok_or("无法获取视频文件名")?
        .to_string();

    // 决定使用的 original_name：用户指定的名称 或 视频原始名称
    let original_name = if output_filename.is_empty() {
        video_original_name.clone()
    } else {
        output_filename.clone()
    };

    // 生成唯一的文件名（用于实际存储）
    let filename = format!(
        "{}_{}.mp3",
        chrono::Local::now().format("%Y%m%d_%H%M%S"),
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
    );

    let output_path = audio_dir.join(&filename);

    // 发送进度开始事件
    app.emit_all("extract-progress", 0u8).map_err(|e| e.to_string())?;

    // 构建FFmpeg命令
    let mut cmd = create_command_from_path(&ffmpeg_path);
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
            &original_name,  // 使用视频文件的原始名称或用户指定的名称
            output_path.to_str().unwrap(),
            file_size,
            duration,
            "mp3",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        ),
    )
    .map_err(|e| format!("保存到数据库失败: {}", e))?;

    Ok(original_name)  // 返回 original_name 而不是 filename
}

/// 从在线视频提取音频（使用yt-dlp + FFmpeg）
#[tauri::command]
pub async fn extract_audio_from_online_video(
    video_url: String,
    output_filename: String,
    app: AppHandle,
    conn: State<'_, Arc<Mutex<Connection>>>,
    audio_dir: State<'_, PathBuf>,
) -> Result<String, String> {
    // 获取FFmpeg可执行文件路径
    let ffmpeg_path = get_ffmpeg_executable_path(Some(&app)).await
        .ok_or("FFmpeg未安装。请将ffmpeg.exe放入tools目录，或点击\"一键安装FFmpeg\"按钮进行安装".to_string())?;

    // 获取yt-dlp可执行文件路径
    let ytdlp_path = get_ytdlp_executable_path(Some(&app)).await
        .ok_or("yt-dlp未安装。请将yt-dlp.exe放入tools目录".to_string())?;

    // 决定使用的 original_name：用户指定的名称 或 视频标题
    let original_name = if output_filename.is_empty() {
        // 使用视频URL的hash作为临时名称（稍后会尝试获取真实标题）
        format!("online_{}", chrono::Local::now().format("%Y%m%d_%H%M%S"))
    } else {
        output_filename.clone()
    };

    // 生成唯一的文件名（用于实际存储）
    let filename = format!(
        "{}_{}.mp3",
        chrono::Local::now().format("%Y%m%d_%H%M%S"),
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
    );

    let output_path = audio_dir.join(&filename);

    // 发送进度开始事件
    app.emit_all("extract-progress", 0u8).map_err(|e| e.to_string())?;

    // 使用yt-dlp下载音频（直接提取最佳音频）
    let mut cmd = create_command_from_path(&ytdlp_path);
    cmd
        .arg("-x") // 提取音频
        .arg("--audio-format").arg("mp3") // 转换为mp3
        .arg("--audio-quality").arg("0") // 最佳音质
        .arg("--ffmpeg-location").arg(ffmpeg_path.to_str().unwrap()) // 指定ffmpeg位置
        .arg("-o").arg(output_path.to_str().unwrap()) // 输出路径
        .arg("--no-playlist") // 不下载播放列表
        .arg("--no-warnings") // 不显示警告
        .arg(&video_url);

    // 发送进度 20%
    app.emit_all("extract-progress", 20u8).map_err(|e| e.to_string())?;

    // 执行yt-dlp命令
    let output = cmd.output().map_err(|e| format!("执行yt-dlp命令失败: {}. 请确保已安装 yt-dlp", e))?;

    // 发送进度 90%
    app.emit_all("extract-progress", 90u8).map_err(|e| e.to_string())?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp执行失败: {}. 请检查视频URL是否正确", error_msg));
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

    // 尝试从yt-dlp输出获取真实标题（如果用户没有指定文件名）
    let final_name = if output_filename.is_empty() {
        // 尝试从stderr获取标题信息
        let _stdout_str = String::from_utf8_lossy(&output.stdout);
        let _stderr_str = String::from_utf8_lossy(&output.stderr);

        // 简单的标题提取逻辑（yt-dlp通常在输出中包含标题信息）
        // 这里使用原始名称，实际使用时可以改进
        original_name
    } else {
        output_filename
    };

    // 保存到数据库
    let conn = conn.lock().await;
    conn.execute(
        "INSERT INTO audio_files (filename, original_name, file_path, file_size, duration, format, upload_date)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            &filename,
            &final_name,
            output_path.to_str().unwrap(),
            file_size,
            duration,
            "mp3",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        ),
    )
    .map_err(|e| format!("保存到数据库失败: {}", e))?;

    Ok(final_name)
}

/// 检查yt-dlp是否可用
async fn check_ytdlp_available(app: Option<&AppHandle>) -> bool {
    get_ytdlp_executable_path(app).await.is_some()
}

/// 获取yt-dlp可执行文件路径
async fn get_ytdlp_executable_path(app: Option<&AppHandle>) -> Option<PathBuf> {
    // 首先尝试使用tools目录中的yt-dlp（优先级最高）
    if let Some(app_handle) = app {
        // 开发环境：使用项目根目录下的tools
        #[cfg(debug_assertions)]
        {
            if let Some(exe_dir) = app_handle.path_resolver().app_data_dir() {
                if let Some(project_root) = exe_dir.parent().and_then(|p| p.parent()) {
                    let tools_ytdlp = project_root.join("tools").join("yt-dlp.exe");
                    if tools_ytdlp.exists() {
                        if let Ok(output) = create_command_from_path(&tools_ytdlp).arg("--version").output() {
                            if output.status.success() {
                                return Some(tools_ytdlp);
                            }
                        }
                    }
                }
            }
        }

        // 生产环境：尝试多个可能的tools目录位置
        #[cfg(not(debug_assertions))]
        {
            // 尝试1: exe所在目录的tools子目录（NSIS安装程序）
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let tools_ytdlp = exe_dir.join("tools").join("yt-dlp.exe");
                    if tools_ytdlp.exists() {
                        if let Ok(output) = create_command_from_path(&tools_ytdlp).arg("--version").output() {
                            if output.status.success() {
                                return Some(tools_ytdlp);
                            }
                        }
                    }
                }
            }

            // 尝试2: 应用数据目录的tools子目录
            if let Some(app_dir) = app_handle.path_resolver().app_data_dir() {
                let tools_ytdlp = app_dir.join("tools").join("yt-dlp.exe");
                if tools_ytdlp.exists() {
                    if let Ok(output) = create_command_from_path(&tools_ytdlp).arg("--version").output() {
                        if output.status.success() {
                            return Some(tools_ytdlp);
                        }
                    }
                }
            }

            // 尝试3: 资源目录的tools子目录
            if let Some(resource_dir) = app_handle.path_resolver().resource_dir() {
                let tools_ytdlp = resource_dir.join("tools").join("yt-dlp.exe");
                if tools_ytdlp.exists() {
                    if let Ok(output) = create_command_from_path(&tools_ytdlp).arg("--version").output() {
                        if output.status.success() {
                            return Some(tools_ytdlp);
                        }
                    }
                }
            }
        }
    }

    // 其次尝试使用PATH中的yt-dlp
    if let Ok(output) = create_command("yt-dlp").arg("--version").output() {
        if output.status.success() {
            return Some(PathBuf::from("yt-dlp"));
        }
    }

    None
}

/// 检查FFmpeg状态
#[tauri::command]
pub async fn check_ffmpeg_status(app: AppHandle) -> Result<FFmpegStatus, String> {
    // 首先尝试使用tools目录中的ffmpeg（优先级最高）
    // 开发环境：使用项目根目录下的tools
    #[cfg(debug_assertions)]
    {
        if let Some(exe_dir) = app.path_resolver().app_data_dir() {
            if let Some(project_root) = exe_dir.parent().and_then(|p| p.parent()) {
                let tools_ffmpeg = project_root.join("tools").join("ffmpeg.exe");
                if tools_ffmpeg.exists() {
                    if let Ok(output) = create_command_from_path(&tools_ffmpeg).arg("-version").output() {
                        if output.status.success() {
                            let version_str = String::from_utf8_lossy(&output.stdout);
                            let version_line = version_str.lines().next().unwrap_or("").to_string();
                            return Ok(FFmpegStatus {
                                available: true,
                                version: Some(version_line),
                                path: Some(format!("内置FFmpeg (tools目录): {}", tools_ffmpeg.display())),
                            });
                        }
                    }
                }
            }
        }
    }

    // 生产环境：尝试多个可能的tools目录位置
    #[cfg(not(debug_assertions))]
    {
        // 尝试1: exe所在目录的tools子目录（NSIS安装程序）
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let tools_ffmpeg = exe_dir.join("tools").join("ffmpeg.exe");
                if tools_ffmpeg.exists() {
                    if let Ok(output) = create_command_from_path(&tools_ffmpeg).arg("-version").output() {
                        if output.status.success() {
                            let version_str = String::from_utf8_lossy(&output.stdout);
                            let version_line = version_str.lines().next().unwrap_or("").to_string();
                            return Ok(FFmpegStatus {
                                available: true,
                                version: Some(version_line),
                                path: Some("内置FFmpeg (tools目录)".to_string()),
                            });
                        }
                    }
                }
            }
        }

        // 尝试2: 应用数据目录的tools子目录
        if let Ok(app_dir) = app.path_resolver().app_dir() {
            let tools_ffmpeg = app_dir.join("tools").join("ffmpeg.exe");
            if tools_ffmpeg.exists() {
                if let Ok(output) = create_command_from_path(&tools_ffmpeg).arg("-version").output() {
                    if output.status.success() {
                        let version_str = String::from_utf8_lossy(&output.stdout);
                        let version_line = version_str.lines().next().unwrap_or("").to_string();
                        return Ok(FFmpegStatus {
                            available: true,
                            version: Some(version_line),
                            path: Some("内置FFmpeg (tools目录)".to_string()),
                        });
                    }
                }
            }
        }

        // 尝试3: 资源目录的tools子目录
        if let Some(resource_dir) = app.path_resolver().resource_dir() {
            let tools_ffmpeg = resource_dir.join("tools").join("ffmpeg.exe");
            if tools_ffmpeg.exists() {
                if let Ok(output) = create_command_from_path(&tools_ffmpeg).arg("-version").output() {
                    if output.status.success() {
                        let version_str = String::from_utf8_lossy(&output.stdout);
                        let version_line = version_str.lines().next().unwrap_or("").to_string();
                        return Ok(FFmpegStatus {
                            available: true,
                            version: Some(version_line),
                            path: Some("内置FFmpeg (tools目录)".to_string()),
                        });
                    }
                }
            }
        }
    }

    // 尝试使用PATH中的ffmpeg
    if let Ok(output) = create_command("ffmpeg").arg("-version").output() {
        if output.status.success() {
            let version_str = String::from_utf8_lossy(&output.stdout);
            let version_line = version_str.lines().next().unwrap_or("").to_string();
            return Ok(FFmpegStatus {
                available: true,
                version: Some(version_line),
                path: Some("ffmpeg (系统PATH)".to_string()),
            });
        }
    }

    // 如果PATH中的不可用，尝试使用本地安装的ffmpeg
    #[cfg(target_os = "windows")]
    {
        if let Some(home_dir) = home_dir() {
            let local_ffmpeg = home_dir.join("ffmpeg").join("bin").join("ffmpeg.exe");
            if local_ffmpeg.exists() {
                if let Ok(output) = create_command_from_path(&local_ffmpeg).arg("-version").output() {
                    if output.status.success() {
                        let version_str = String::from_utf8_lossy(&output.stdout);
                        let version_line = version_str.lines().next().unwrap_or("").to_string();
                        return Ok(FFmpegStatus {
                            available: true,
                            version: Some(version_line),
                            path: Some(local_ffmpeg.to_string_lossy().to_string()),
                        });
                    }
                }
            }
        }
    }

    Ok(FFmpegStatus {
        available: false,
        version: None,
        path: None,
    })
}

/// 一键下载安装FFmpeg
#[tauri::command]
pub async fn install_ffmpeg(app: AppHandle) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        install_ffmpeg_windows(app).await
    }

    #[cfg(target_os = "macos")]
    {
        install_ffmpeg_macos().await
    }

    #[cfg(target_os = "linux")]
    {
        install_ffmpeg_linux().await
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("不支持的操作系统".to_string())
    }
}

#[cfg(target_os = "windows")]
async fn install_ffmpeg_windows(app: AppHandle) -> Result<String, String> {
    let home_dir = home_dir().ok_or("无法获取用户目录")?;
    let ffmpeg_dir = home_dir.join("ffmpeg");
    let ffmpeg_exe = ffmpeg_dir.join("bin").join("ffmpeg.exe");

    // 检查是否已经安装
    if ffmpeg_exe.exists() {
        // 检查PATH环境变量
        if let Ok(output) = create_command("ffmpeg").arg("-version").output() {
            if output.status.success() {
                return Ok("FFmpeg已安装并配置完成".to_string());
            }
        }

        // 添加到PATH环境变量
        add_to_path_windows(ffmpeg_dir.join("bin").to_str().unwrap())?;
        return Ok("FFmpeg已安装，已配置环境变量".to_string());
    }

    // 发送进度开始事件
    app.emit_all("ffmpeg-install-progress", 0u8).map_err(|e| e.to_string())?;

    // 创建安装目录
    fs::create_dir_all(&ffmpeg_dir)
        .map_err(|e| format!("创建安装目录失败: {}", e))?;

    // 发送进度 10%
    app.emit_all("ffmpeg-install-progress", 10u8).map_err(|e| e.to_string())?;

    // 下载FFmpeg
    let download_url = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip";
    let client = reqwest::Client::new();

    let response = client.get(download_url)
        .send()
        .await
        .map_err(|e| format!("下载FFmpeg失败: {}", e))?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded = 0u64;

    // 发送进度 20%
    app.emit_all("ffmpeg-install-progress", 20u8).map_err(|e| e.to_string())?;

    // 下载文件
    let temp_zip_path = ffmpeg_dir.join("ffmpeg.zip");
    let mut file = File::create(&temp_zip_path)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("下载中断: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        downloaded += chunk.len() as u64;

        // 更新进度 (20% - 80%)
        if total_size > 0 {
            let progress = 20 + (downloaded * 60 / total_size) as u8;
            app.emit_all("ffmpeg-install-progress", progress).map_err(|e| e.to_string())?;
        }
    }

    drop(file);

    // 发送进度 80%
    app.emit_all("ffmpeg-install-progress", 80u8).map_err(|e| e.to_string())?;

    // 解压文件
    let zip_file = File::open(&temp_zip_path)
        .map_err(|e| format!("打开压缩文件失败: {}", e))?;
    let mut archive = ZipArchive::new(zip_file)
        .map_err(|e| format!("读取压缩文件失败: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("解压失败: {}", e))?;
        let outpath = ffmpeg_dir.join(file.mangled_name());

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)
                        .map_err(|e| format!("创建父目录失败: {}", e))?;
                }
            }
            let mut outfile = File::create(&outpath)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }
    }

    // 删除压缩文件
    fs::remove_file(&temp_zip_path)
        .map_err(|e| format!("删除临时文件失败: {}", e))?;

    // 发送进度 90%
    app.emit_all("ffmpeg-install-progress", 90u8).map_err(|e| e.to_string())?;

    // 添加到PATH环境变量
    add_to_path_windows(ffmpeg_dir.join("bin").to_str().unwrap())?;

    // 发送完成进度
    app.emit_all("ffmpeg-install-progress", 100u8).map_err(|e| e.to_string())?;

    Ok("FFmpeg安装完成！应用即将重启以使更改生效".to_string())
}

#[cfg(target_os = "windows")]
fn add_to_path_windows(ffmpeg_path: &str) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let environment = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .map_err(|e| format!("打开注册表失败: {}", e))?;

    let path_value: String = environment.get_value("Path")
        .unwrap_or_default();

    if !path_value.contains(ffmpeg_path) {
        let new_path = if path_value.is_empty() {
            ffmpeg_path.to_string()
        } else {
            format!("{};{}", path_value, ffmpeg_path)
        };

        environment.set_value("Path", &new_path)
            .map_err(|e| format!("设置PATH失败: {}", e))?;

        // 通知系统环境变量已更改
        unsafe {
            let env_str = "Environment\0".encode_utf16().collect::<Vec<u16>>();
            winapi::um::winuser::SendMessageW(
                winapi::um::winuser::HWND_BROADCAST,
                winapi::um::winuser::WM_SETTINGCHANGE,
                0,
                env_str.as_ptr() as isize,
            );
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
async fn install_ffmpeg_macos() -> Result<String, String> {
    let output = Command::new("brew")
        .args(&["install", "ffmpeg"])
        .output()
        .await
        .map_err(|e| format!("执行brew命令失败: {}", e))?;

    if output.status.success() {
        Ok("FFmpeg通过Homebrew安装完成".to_string())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("Homebrew安装FFmpeg失败: {}", error))
    }
}

#[cfg(target_os = "linux")]
async fn install_ffmpeg_linux() -> Result<String, String> {
    // 尝试apt
    let output = Command::new("apt")
        .args(&["update"])
        .output()
        .await;

    if let Ok(result) = output {
        if result.status.success() {
            let output = Command::new("apt")
                .args(&["install", "-y", "ffmpeg"])
                .output()
                .await
                .map_err(|e| format!("执行apt命令失败: {}", e))?;

            if output.status.success() {
                return Ok("FFmpeg通过apt安装完成".to_string());
            }
        }
    }

    // 尝试yum
    let output = Command::new("yum")
        .args(&["install", "-y", "ffmpeg"])
        .output()
        .await
        .map_err(|e| format!("执行yum命令失败: {}", e))?;

    if output.status.success() {
        Ok("FFmpeg通过yum安装完成".to_string())
    } else {
        Err("无法安装FFmpeg，请手动安装".to_string())
    }
}
