use auto_launch::AutoLaunchBuilder;
use tauri::api::path::config_dir;
use std::path::PathBuf;

pub fn get_app_path() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| e.to_string())
}

pub fn is_auto_launch_enabled() -> Result<bool, String> {
    let app_path = get_app_path()?;
    let app_name = "磨耳朵";

    let auto = AutoLaunchBuilder::new()
        .set_app_name(app_name)
        .set_app_path(&app_path.to_string_lossy())
        .build()
        .map_err(|e| e.to_string())?;

    auto.is_enabled().map_err(|e| e.to_string())
}

pub fn enable_auto_launch() -> Result<(), String> {
    let app_path = get_app_path()?;
    let app_name = "磨耳朵";

    let auto = AutoLaunchBuilder::new()
        .set_app_name(app_name)
        .set_app_path(&app_path.to_string_lossy())
        .set_use_launch_agent(true)
        .build()
        .map_err(|e| e.to_string())?;

    auto.enable().map_err(|e| e.to_string())
}

pub fn disable_auto_launch() -> Result<(), String> {
    let app_path = get_app_path()?;
    let app_name = "磨耳朵";

    let auto = AutoLaunchBuilder::new()
        .set_app_name(app_name)
        .set_app_path(&app_path.to_string_lossy())
        .build()
        .map_err(|e| e.to_string())?;

    auto.disable().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_auto_launch_status() -> Result<bool, String> {
    is_auto_launch_enabled()
}

#[tauri::command]
pub async fn set_auto_launch(enable: bool) -> Result<(), String> {
    if enable {
        enable_auto_launch()
    } else {
        disable_auto_launch()
    }
}
