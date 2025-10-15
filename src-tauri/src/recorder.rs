use hound::{WavSpec, WavWriter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use tauri::State;
use tokio::sync::Mutex;
use rusqlite::Connection;
use serde::Serialize;
use std::io::BufReader;
use rodio::{Decoder, Source};

#[derive(Debug, Serialize)]
pub struct RecordingState {
    pub is_recording: bool,
    pub duration: f32,
}

/// 获取音频文件的真实时长（秒）
fn get_audio_duration(file_path: &std::path::Path) -> i64 {
    match std::fs::File::open(file_path) {
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

// 简化的录音器，不存储Stream对象
pub struct AudioRecorder {
    is_recording: Arc<StdMutex<bool>>,
    output_path: Arc<StdMutex<Option<PathBuf>>>,
}

// 手动实现Send和Sync
unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

impl AudioRecorder {
    pub fn new() -> Self {
        AudioRecorder {
            is_recording: Arc::new(StdMutex::new(false)),
            output_path: Arc::new(StdMutex::new(None)),
        }
    }

    pub fn is_recording(&self) -> bool {
        *self.is_recording.lock().unwrap()
    }

    pub fn set_recording(&self, recording: bool) {
        *self.is_recording.lock().unwrap() = recording;
    }

    pub fn get_output_path(&self) -> Option<PathBuf> {
        self.output_path.lock().unwrap().clone()
    }

    pub fn set_output_path(&self, path: Option<PathBuf>) {
        *self.output_path.lock().unwrap() = path;
    }
}

#[tauri::command]
pub async fn start_recording(
    filename: String,
    audio_dir: State<'_, PathBuf>,
    recorder: State<'_, Arc<Mutex<AudioRecorder>>>,
) -> Result<String, String> {
    let recorder = recorder.lock().await;

    if recorder.is_recording() {
        return Err("已经在录音中".to_string());
    }

    // 创建rec子目录用于存放录音文件
    let rec_dir = audio_dir.join("rec");
    std::fs::create_dir_all(&rec_dir)
        .map_err(|e| format!("创建录音目录失败: {}", e))?;

    let output_path = rec_dir.join(format!("{}.wav", filename));
    recorder.set_output_path(Some(output_path.clone()));
    recorder.set_recording(true);

    // 在后台线程中进行录音
    let output_path_clone = output_path.clone();
    let is_recording = Arc::clone(&recorder.is_recording);

    std::thread::spawn(move || {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

        // 获取默认音频输入设备
        let host = match cpal::default_host() {
            host => host,
        };

        let device = match host.default_input_device() {
            Some(device) => device,
            None => {
                eprintln!("没有找到音频输入设备");
                return;
            }
        };

        let config = match device.default_input_config() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("获取输入配置失败: {}", e);
                return;
            }
        };

        // 创建WAV文件
        let spec = WavSpec {
            channels: config.channels(),
            sample_rate: config.sample_rate().0,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let writer = match WavWriter::create(&output_path_clone, spec) {
            Ok(writer) => Arc::new(StdMutex::new(writer)),
            Err(e) => {
                eprintln!("创建WAV文件失败: {}", e);
                return;
            }
        };

        let writer_clone = Arc::clone(&writer);
        let is_recording_clone = Arc::clone(&is_recording);

        let err_fn = |err| eprintln!("录音流错误: {}", err);

        // 构建录音流
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if !*is_recording_clone.lock().unwrap() {
                            return;
                        }
                        if let Ok(mut writer_guard) = writer_clone.lock() {
                            for &sample in data {
                                let sample_i16 = (sample * i16::MAX as f32) as i16;
                                let _ = writer_guard.write_sample(sample_i16);
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if !*is_recording_clone.lock().unwrap() {
                            return;
                        }
                        if let Ok(mut writer_guard) = writer_clone.lock() {
                            for &sample in data {
                                let _ = writer_guard.write_sample(sample);
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            }
            cpal::SampleFormat::U16 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        if !*is_recording_clone.lock().unwrap() {
                            return;
                        }
                        if let Ok(mut writer_guard) = writer_clone.lock() {
                            for &sample in data {
                                let sample_i16 = (sample as i32 - 32768) as i16;
                                let _ = writer_guard.write_sample(sample_i16);
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            }
            _ => {
                eprintln!("不支持的采样格式");
                return;
            }
        };

        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("创建录音流失败: {}", e);
                return;
            }
        };

        if let Err(e) = stream.play() {
            eprintln!("启动录音失败: {}", e);
            return;
        }

        // 保持流存活，直到停止录音
        while *is_recording.lock().unwrap() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        drop(stream);

        // 完成WAV文件写入
        if let Ok(writer_mutex) = Arc::try_unwrap(writer) {
            if let Ok(writer) = writer_mutex.into_inner() {
                let _ = writer.finalize();
            }
        }
    });

    Ok(output_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn stop_recording(
    recorder: State<'_, Arc<Mutex<AudioRecorder>>>,
    conn: State<'_, Arc<Mutex<Connection>>>,
    audio_dir: State<'_, PathBuf>,
) -> Result<i64, String> {
    let recorder = recorder.lock().await;

    if !recorder.is_recording() {
        return Err("未在录音中".to_string());
    }

    recorder.set_recording(false);

    // 等待录音线程完成
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output_path = recorder.get_output_path()
        .ok_or("录音文件路径丢失".to_string())?;

    recorder.set_output_path(None);

    // 获取文件信息
    let metadata = std::fs::metadata(&output_path)
        .map_err(|e| format!("获取文件信息失败: {}", e))?;
    let file_size = metadata.len() as i64;

    let original_name = output_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("无效的文件名")?
        .to_string();

    let filename = format!(
        "{}_{}.wav",
        chrono::Local::now().format("%Y%m%d_%H%M%S"),
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
    );

    // 将文件重命名为标准格式（移动到主audio目录）
    let dest_path = audio_dir.join(&filename);
    std::fs::rename(&output_path, &dest_path)
        .map_err(|e| format!("重命名文件失败: {}", e))?;

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
            "wav",
        ),
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    Ok(id)
}

#[tauri::command]
pub async fn get_recording_state(
    recorder: State<'_, Arc<Mutex<AudioRecorder>>>,
) -> Result<RecordingState, String> {
    let recorder = recorder.lock().await;
    Ok(RecordingState {
        is_recording: recorder.is_recording(),
        duration: 0.0,
    })
}
