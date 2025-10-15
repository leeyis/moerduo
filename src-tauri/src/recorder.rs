use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use tauri::State;
use tokio::sync::Mutex;
use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RecordingState {
    pub is_recording: bool,
    pub duration: f32,
}

pub struct AudioRecorder {
    writer: Option<Arc<StdMutex<WavWriter<std::io::BufWriter<std::fs::File>>>>>,
    stream: Option<cpal::Stream>,
    is_recording: bool,
    output_path: Option<PathBuf>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        AudioRecorder {
            writer: None,
            stream: None,
            is_recording: false,
            output_path: None,
        }
    }

    pub fn start_recording(&mut self, output_path: PathBuf) -> Result<(), String> {
        if self.is_recording {
            return Err("已经在录音中".to_string());
        }

        // 获取默认音频输入设备
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("没有找到音频输入设备".to_string())?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("获取输入配置失败: {}", e))?;

        // 创建WAV文件
        let spec = WavSpec {
            channels: config.channels(),
            sample_rate: config.sample_rate().0,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let writer = WavWriter::create(&output_path, spec)
            .map_err(|e| format!("创建WAV文件失败: {}", e))?;

        let writer = Arc::new(StdMutex::new(writer));
        let writer_clone = Arc::clone(&writer);

        // 创建音频流
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => self.build_stream::<f32>(&device, &config.into(), writer_clone),
            cpal::SampleFormat::I16 => self.build_stream::<i16>(&device, &config.into(), writer_clone),
            cpal::SampleFormat::U16 => self.build_stream::<u16>(&device, &config.into(), writer_clone),
            _ => return Err("不支持的采样格式".to_string()),
        }?;

        stream.play().map_err(|e| format!("启动录音失败: {}", e))?;

        self.writer = Some(writer);
        self.stream = Some(stream);
        self.is_recording = true;
        self.output_path = Some(output_path);

        Ok(())
    }

    fn build_stream<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        writer: Arc<StdMutex<WavWriter<std::io::BufWriter<std::fs::File>>>>,
    ) -> Result<cpal::Stream, String>
    where
        T: cpal::Sample + hound::Sample,
    {
        let err_fn = |err| eprintln!("录音流错误: {}", err);

        let stream = device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut writer_guard) = writer.lock() {
                        for &sample in data {
                            if let Err(e) = writer_guard.write_sample(sample) {
                                eprintln!("写入采样失败: {}", e);
                            }
                        }
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| format!("创建录音流失败: {}", e))?;

        Ok(stream)
    }

    pub fn stop_recording(&mut self) -> Result<PathBuf, String> {
        if !self.is_recording {
            return Err("未在录音中".to_string());
        }

        // 停止流
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }

        // 关闭writer
        if let Some(writer) = self.writer.take() {
            let writer_guard = writer.lock().unwrap();
            writer_guard.finalize().map_err(|e| format!("完成WAV文件失败: {}", e))?;
        }

        self.is_recording = false;

        let output_path = self.output_path.take().ok_or("录音文件路径丢失".to_string())?;
        Ok(output_path)
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording
    }
}

#[tauri::command]
pub async fn start_recording(
    filename: String,
    audio_dir: State<'_, PathBuf>,
    recorder: State<'_, Arc<Mutex<AudioRecorder>>>,
) -> Result<String, String> {
    let output_path = audio_dir.join(format!("{}.wav", filename));

    let mut recorder = recorder.lock().await;
    recorder.start_recording(output_path.clone())?;

    Ok(output_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn stop_recording(
    recorder: State<'_, Arc<Mutex<AudioRecorder>>>,
    conn: State<'_, Arc<Mutex<Connection>>>,
    audio_dir: State<'_, PathBuf>,
) -> Result<i64, String> {
    let output_path = {
        let mut recorder = recorder.lock().await;
        recorder.stop_recording()?
    };

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

    // 将文件重命名为标准格式
    let dest_path = audio_dir.join(&filename);
    std::fs::rename(&output_path, &dest_path)
        .map_err(|e| format!("重命名文件失败: {}", e))?;

    // 简化：默认时长3分钟
    let duration = 180;

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
        duration: 0.0, // 可以后续添加录音时长追踪
    })
}
