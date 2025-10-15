use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;
use tauri::State;
use rusqlite::Connection;
use tokio::sync::Mutex;
use rodio::{Sink, OutputStream, OutputStreamHandle, Decoder, Source};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    pub is_playing: bool,
    pub current_audio_id: Option<i64>,
    pub current_audio_name: Option<String>,
    pub volume: f32,
    pub speed: f32,
    pub playlist_queue: Vec<i64>,
    pub current_index: usize,
    pub is_auto_play: bool,
}

pub struct AudioPlayer {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    sink: Option<Sink>,
    current_audio_id: Option<i64>,
    current_audio_name: Option<String>,
    playlist_queue: Vec<i64>,
    current_index: usize,
    volume: f32,
    speed: f32,
    is_auto_play: bool,
}

// 手动实现Send，因为我们确保只在单线程中访问
unsafe impl Send for AudioPlayer {}
unsafe impl Sync for AudioPlayer {}

impl AudioPlayer {
    pub fn new() -> Self {
        Self {
            _stream: None,
            stream_handle: None,
            sink: None,
            current_audio_id: None,
            current_audio_name: None,
            playlist_queue: Vec::new(),
            current_index: 0,
            volume: 0.5,
            speed: 1.0,
            is_auto_play: false,
        }
    }

    pub fn init_stream(&mut self) {
        if self.stream_handle.is_none() {
            if let Ok((stream, handle)) = OutputStream::try_default() {
                self._stream = Some(stream);
                self.stream_handle = Some(handle);
            }
        }
    }

    pub fn play(&mut self, file_path: &str) -> Result<(), String> {
        self.init_stream();

        let stream_handle = self.stream_handle.as_ref()
            .ok_or("音频流未初始化")?;

        // 停止当前播放
        if let Some(sink) = &self.sink {
            sink.stop();
        }

        // 创建新的Sink
        let sink = Sink::try_new(stream_handle).map_err(|e| e.to_string())?;

        // 打开音频文件
        let file = File::open(file_path).map_err(|e| e.to_string())?;
        let source = Decoder::new(BufReader::new(file)).map_err(|e| e.to_string())?;

        // 应用倍速
        let source = source.speed(self.speed);

        sink.append(source);
        sink.set_volume(self.volume);
        sink.play();

        self.sink = Some(sink);

        Ok(())
    }

    pub fn play_with_info(&mut self, file_path: &str, audio_id: i64, audio_name: String) -> Result<(), String> {
        self.current_audio_id = Some(audio_id);
        self.current_audio_name = Some(audio_name);
        self.play(file_path)
    }

    pub fn set_playlist_queue(&mut self, queue: Vec<i64>, is_auto_play: bool) {
        self.playlist_queue = queue;
        self.current_index = 0;
        self.is_auto_play = is_auto_play;
    }

    pub fn play_next(&mut self) -> Option<i64> {
        if self.playlist_queue.is_empty() {
            return None;
        }

        if self.current_index + 1 < self.playlist_queue.len() {
            self.current_index += 1;
            Some(self.playlist_queue[self.current_index])
        } else {
            None
        }
    }

    pub fn play_previous(&mut self) -> Option<i64> {
        if self.playlist_queue.is_empty() {
            return None;
        }

        if self.current_index > 0 {
            self.current_index -= 1;
            Some(self.playlist_queue[self.current_index])
        } else {
            None
        }
    }

    pub fn pause(&self) {
        if let Some(sink) = &self.sink {
            sink.pause();
        }
    }

    #[allow(dead_code)]
    pub fn resume(&self) {
        if let Some(sink) = &self.sink {
            sink.play();
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
        self.sink = None;
        self.current_audio_id = None;
        self.current_audio_name = None;
        self.playlist_queue.clear();
        self.current_index = 0;
        self.is_auto_play = false;
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.max(0.0).min(1.0);
        if let Some(sink) = &self.sink {
            sink.set_volume(self.volume);
        }
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.max(0.5).min(3.0);
        // 需要重新播放才能应用新的倍速
        // 调用者需要重新调用 play
    }

    #[allow(dead_code)]
    pub fn is_playing(&self) -> bool {
        self.sink.as_ref().map_or(false, |s| !s.is_paused() && !s.empty())
    }

    pub fn get_state(&self) -> PlaybackState {
        PlaybackState {
            is_playing: self.is_playing(),
            current_audio_id: self.current_audio_id,
            current_audio_name: self.current_audio_name.clone(),
            volume: self.volume,
            speed: self.speed,
            playlist_queue: self.playlist_queue.clone(),
            current_index: self.current_index,
            is_auto_play: self.is_auto_play,
        }
    }
}

#[tauri::command]
pub async fn play_audio(
    id: i64,
    player: State<'_, Arc<Mutex<AudioPlayer>>>,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    // 从数据库获取文件路径和名称
    let (file_path, audio_name): (String, String) = {
        let conn = conn.lock().await;
        conn.query_row(
            "SELECT file_path, original_name FROM audio_files WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?
    };

    // 播放音频
    let mut player = player.lock().await;
    player.play_with_info(&file_path, id, audio_name)?;

    // 更新播放计数
    let conn = conn.lock().await;
    conn.execute(
        "UPDATE audio_files SET play_count = play_count + 1, last_played = datetime('now') WHERE id = ?1",
        [id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn pause_audio(
    player: State<'_, Arc<Mutex<AudioPlayer>>>,
) -> Result<(), String> {
    let player = player.lock().await;
    player.pause();
    Ok(())
}

#[tauri::command]
pub async fn stop_audio(
    player: State<'_, Arc<Mutex<AudioPlayer>>>,
) -> Result<(), String> {
    let mut player = player.lock().await;
    player.stop();
    Ok(())
}

#[tauri::command]
pub async fn set_volume(
    volume: f32,
    player: State<'_, Arc<Mutex<AudioPlayer>>>,
) -> Result<(), String> {
    let mut player = player.lock().await;
    player.set_volume(volume);
    Ok(())
}

#[tauri::command]
pub async fn set_speed(
    speed: f32,
    player: State<'_, Arc<Mutex<AudioPlayer>>>,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    let mut player = player.lock().await;
    player.set_speed(speed);

    // 如果正在播放，需要重新播放当前音频以应用新倍速
    if let Some(audio_id) = player.current_audio_id {
        let (file_path, audio_name): (String, String) = {
            let conn = conn.lock().await;
            conn.query_row(
                "SELECT file_path, original_name FROM audio_files WHERE id = ?1",
                [audio_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| e.to_string())?
        };

        player.play_with_info(&file_path, audio_id, audio_name)?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_playback_state(
    player: State<'_, Arc<Mutex<AudioPlayer>>>,
) -> Result<PlaybackState, String> {
    let player = player.lock().await;
    Ok(player.get_state())
}

#[tauri::command]
pub async fn play_next(
    player: State<'_, Arc<Mutex<AudioPlayer>>>,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    let mut player = player.lock().await;

    if let Some(next_audio_id) = player.play_next() {
        let (file_path, audio_name): (String, String) = {
            let conn = conn.lock().await;
            conn.query_row(
                "SELECT file_path, original_name FROM audio_files WHERE id = ?1",
                [next_audio_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| e.to_string())?
        };

        player.play_with_info(&file_path, next_audio_id, audio_name)?;

        // 更新播放计数
        let conn = conn.lock().await;
        conn.execute(
            "UPDATE audio_files SET play_count = play_count + 1, last_played = datetime('now') WHERE id = ?1",
            [next_audio_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn play_previous(
    player: State<'_, Arc<Mutex<AudioPlayer>>>,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    let mut player = player.lock().await;

    if let Some(prev_audio_id) = player.play_previous() {
        let (file_path, audio_name): (String, String) = {
            let conn = conn.lock().await;
            conn.query_row(
                "SELECT file_path, original_name FROM audio_files WHERE id = ?1",
                [prev_audio_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| e.to_string())?
        };

        player.play_with_info(&file_path, prev_audio_id, audio_name)?;

        // 更新播放计数
        let conn = conn.lock().await;
        conn.execute(
            "UPDATE audio_files SET play_count = play_count + 1, last_played = datetime('now') WHERE id = ?1",
            [prev_audio_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn play_playlist(
    playlist_id: i64,
    is_auto_play: bool,
    player: State<'_, Arc<Mutex<AudioPlayer>>>,
    conn: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    // 获取播放列表中的所有音频 ID
    let audio_ids: Vec<i64> = {
        let conn = conn.lock().await;
        let mut stmt = conn
            .prepare(
                "SELECT audio_id FROM playlist_items
                 WHERE playlist_id = ?1
                 ORDER BY sort_order"
            )
            .map_err(|e| e.to_string())?;

        let ids: Vec<i64> = stmt
            .query_map([playlist_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        ids
    };

    if audio_ids.is_empty() {
        return Err("播放列表为空".to_string());
    }

    let mut player = player.lock().await;
    player.set_playlist_queue(audio_ids.clone(), is_auto_play);

    // 播放第一首
    let first_audio_id = audio_ids[0];
    let (file_path, audio_name): (String, String) = {
        let conn = conn.lock().await;
        conn.query_row(
            "SELECT file_path, original_name FROM audio_files WHERE id = ?1",
            [first_audio_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?
    };

    player.play_with_info(&file_path, first_audio_id, audio_name)?;

    // 更新播放计数
    let conn = conn.lock().await;
    conn.execute(
        "UPDATE audio_files SET play_count = play_count + 1, last_played = datetime('now') WHERE id = ?1",
        [first_audio_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
