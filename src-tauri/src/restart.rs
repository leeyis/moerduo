use tauri::AppHandle;
use std::process::Command;
use std::env;

/// 重启应用
#[tauri::command]
pub async fn restart_app(_app: AppHandle) -> Result<(), String> {
    // 获取当前可执行文件路径
    let current_exe = env::current_exe()
        .map_err(|e| format!("获取当前可执行文件路径失败: {}", e))?;

    // 获取当前工作目录
    let current_dir = env::current_dir()
        .map_err(|e| format!("获取当前工作目录失败: {}", e))?;

    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    // 在新进程中启动应用
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("cmd");
        cmd.args(&["/C", "start", "\"\""])
            .arg(&current_exe)
            .args(&args[1..]) // 跳过第一个参数（可执行文件本身）
            .current_dir(&current_dir);

        cmd.spawn()
            .map_err(|e| format!("启动新进程失败: {}", e))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new(&current_exe)
            .args(&args[1..]) // 跳过第一个参数（可执行文件本身）
            .current_dir(&current_dir)
            .spawn()
            .map_err(|e| format!("启动新进程失败: {}", e))?;
    }

    // 退出当前应用
    std::process::exit(0);
}