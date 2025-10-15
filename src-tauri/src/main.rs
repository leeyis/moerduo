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
mod autostart;

use tauri::{Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, CustomMenuItem};
use std::sync::Arc;
use tokio::sync::Mutex;

fn main() {
    // 创建系统托盘菜单
    let show = CustomMenuItem::new("show".to_string(), "显示主窗口");
    let hide = CustomMenuItem::new("hide".to_string(), "隐藏窗口");
    let quit = CustomMenuItem::new("quit".to_string(), "退出应用");

    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(hide)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "show" => {
                        let window = app.get_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    "hide" => {
                        let window = app.get_window("main").unwrap();
                        window.hide().unwrap();
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
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
            playlist::check_playlist_tasks,
            task::get_scheduled_tasks,
            task::create_scheduled_task,
            task::update_scheduled_task,
            task::delete_scheduled_task,
            task::toggle_scheduled_task,
            task::check_task_conflicts,
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
            audio::extract_audio_from_video,
            audio::check_ffmpeg_status,
            audio::install_ffmpeg,
            autostart::get_auto_launch_status,
            autostart::set_auto_launch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
