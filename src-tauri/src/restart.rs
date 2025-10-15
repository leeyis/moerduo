use tauri::AppHandle;
use std::process::Command;
use std::env;
use std::thread;
use std::time::Duration;
use std::fs::OpenOptions;
use std::io::Write;

/// 写入日志文件
fn write_log(message: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("restart_log.txt")
    {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(file, "[{}] {}", timestamp, message);
    }
}

/// 重启应用
#[tauri::command]
pub async fn restart_app(_app: AppHandle) -> Result<(), String> {
    write_log("开始重启应用");

    // 获取当前可执行文件路径
    let current_exe = env::current_exe()
        .map_err(|e| {
            write_log(&format!("获取当前可执行文件路径失败: {}", e));
            format!("获取当前可执行文件路径失败: {}", e)
        })?;

    // 获取当前工作目录
    let current_dir = env::current_dir()
        .map_err(|e| {
            write_log(&format!("获取当前工作目录失败: {}", e));
            format!("获取当前工作目录失败: {}", e)
        })?;

    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    let exe_path = current_exe.to_string_lossy().to_string();

    write_log(&format!("可执行文件路径: {}", exe_path));
    write_log(&format!("工作目录: {}", current_dir.display()));
    write_log(&format!("命令行参数: {:?}", args));

    // 在新进程中启动应用
    #[cfg(target_os = "windows")]
    {
        write_log("Windows系统，开始启动新进程");

        // 方案1：使用 explorer.exe (最简单可靠)
        let mut cmd = Command::new("explorer.exe");
        cmd.arg(&exe_path)
            .current_dir(&current_dir);

        match cmd.spawn() {
            Ok(_) => {
                write_log("使用explorer.exe成功启动新进程");
            }
            Err(e) => {
                write_log(&format!("explorer.exe启动失败: {}", e));

                // 方案2：使用批处理文件
                write_log("尝试使用批处理文件重启");
                let batch_content = format!(
                    "@echo off\r\ntitle 重启应用\r\necho 正在重启...\r\n\"{}\" {}\r\n",
                    exe_path,
                    if args.len() > 1 {
                        args[1..].join(" ")
                    } else {
                        String::new()
                    }
                );

                if let Ok(mut file) = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open("restart.bat")
                {
                    let _ = file.write_all(batch_content.as_bytes());

                    write_log("创建批处理文件成功，尝试执行");

                    match Command::new("cmd")
                        .args(&["/C", "restart.bat"])
                        .current_dir(&current_dir)
                        .spawn()
                    {
                        Ok(_) => {
                            write_log("批处理文件执行成功");
                        }
                        Err(_) => {
                            let e2 = "批处理文件执行失败";
                            write_log(e2);
                            return Err(format!("重启失败: {} 和 {}", e, e2));
                        }
                    }
                } else {
                    let e2 = "创建批处理文件失败";
                    write_log(e2);
                    return Err(format!("重启失败: {} 和 {}", e, e2));
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        write_log("非Windows系统，直接启动");
        Command::new(&current_exe)
            .args(&args[1..]) // 跳过第一个参数（可执行文件本身）
            .current_dir(&current_dir)
            .spawn()
            .map_err(|e| {
                write_log(&format!("启动新进程失败: {}", e));
                format!("启动新进程失败: {}", e)
            })?;
    }

    write_log("重启命令已发送，准备退出当前进程");

    // 延迟退出当前应用，给新进程启动时间
    thread::sleep(Duration::from_millis(1000));

    write_log("退出当前应用");

    // 退出当前应用
    std::process::exit(0);
}