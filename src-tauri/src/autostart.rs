use auto_launch::AutoLaunchBuilder;
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
    // 在开发模式下可能获取失败，返回false而不是错误
    match is_auto_launch_enabled() {
        Ok(status) => Ok(status),
        Err(e) => {
            eprintln!("获取自启动状态失败（开发模式下这是正常的）: {}", e);
            Ok(false) // 返回false而不是错误
        }
    }
}

#[tauri::command]
pub async fn set_auto_launch(enable: bool) -> Result<(), String> {
    // 在开发模式下，自启动功能可能无法正常工作（因为exe路径是临时的）
    // 我们捕获错误但不向用户抛出
    let result = if enable {
        enable_auto_launch()
    } else {
        disable_auto_launch()
    };

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            // 在开发模式下，这个错误是预期的，不应该阻止用户保存其他设置
            eprintln!("自启动设置失败（开发模式下这是正常的）: {}", e);
            // 不向用户返回错误，避免阻塞其他设置的保存
            Ok(())
        }
    }
}
