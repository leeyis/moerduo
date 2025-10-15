// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod audio;
mod player;
mod playlist;
mod task;
mod scheduler;
mod stats;
mod settings;
mod recorder;

use tauri::Manager;
use std::sync::Arc;
use tokio::sync::Mutex;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // 初始化数据库
            let app_handle = app.handle();
            let app_dir = app_handle.path_resolver()
                .app_data_dir()
                .expect("Failed to get app data dir");

            std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");

            let db_path = app_dir.join("moerduo.db");
            let conn = db::init_database(&db_path).expect("Failed to initialize database");

            // 创建音频存储目录
            let audio_dir = app_dir.join("audio");
            std::fs::create_dir_all(&audio_dir).expect("Failed to create audio dir");

            // 创建共享状态
            let db_conn = Arc::new(Mutex::new(conn));
            let audio_player = Arc::new(Mutex::new(player::AudioPlayer::new()));
            let audio_recorder = Arc::new(Mutex::new(recorder::AudioRecorder::new()));

            // 启动定时任务调度器
            let scheduler = scheduler::Scheduler::new(db_conn.clone(), audio_player.clone());
            tauri::async_runtime::spawn(async move {
                scheduler.start().await;
            });

            // 将状态放入管理
            app.manage(db_conn);
            app.manage(audio_dir.clone());
            app.manage(audio_player);
            app.manage(audio_recorder);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            audio::upload_audio_file,
            audio::get_audio_files,
            audio::delete_audio_file,
            audio::scan_audio_directory,
            player::play_audio,
            player::pause_audio,
            player::stop_audio,
            player::set_volume,
            player::set_speed,
            player::get_playback_state,
            player::play_next,
            player::play_previous,
            player::play_playlist,
            playlist::get_playlists,
            playlist::create_playlist,
            playlist::delete_playlist,
            playlist::set_playlist_mode,
            playlist::get_playlist_items,
            playlist::add_to_playlist,
            playlist::remove_from_playlist,
            task::get_scheduled_tasks,
            task::create_scheduled_task,
            task::update_scheduled_task,
            task::delete_scheduled_task,
            task::toggle_scheduled_task,
            stats::get_statistics,
            stats::get_top_audios,
            stats::get_daily_activity,
            stats::get_monthly_playback,
            settings::get_settings,
            settings::save_settings,
            settings::get_data_usage,
            settings::export_config,
            settings::import_config,
            recorder::start_recording,
            recorder::stop_recording,
            recorder::get_recording_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
