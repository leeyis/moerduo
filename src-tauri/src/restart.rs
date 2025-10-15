use tauri::AppHandle;
use std::process::Command;
use std::env;
use std::thread;
use std::time::Duration;

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

    let exe_path = current_exe.to_string_lossy().to_string();

    // 在新进程中启动应用
    #[cfg(target_os = "windows")]
    {
        // 使用PowerShell启动新进程，更可靠
        let mut cmd = Command::new("powershell.exe");
        cmd.args(&[
            "-Command",
            &format!("Start-Process -FilePath \"{}\" -ArgumentList @({})",
                exe_path,
                if args.len() > 1 {
                    format!("'{}'", args[1..].join("' '"))
                } else {
                    String::new()
                }
            )
        ])
        .current_dir(&current_dir);

        match cmd.spawn() {
            Ok(_) => {
                println!("使用PowerShell成功启动新进程");
            }
            Err(e) => {
                println!("PowerShell启动失败: {}, 尝试直接启动", e);

                // 后备方案：直接启动
                match Command::new(&exe_path)
                    .args(&args[1..])
                    .current_dir(&current_dir)
                    .spawn()
                {
                    Ok(_) => {
                        println!("直接启动成功");
                    }
                    Err(e2) => {
                        println!("直接启动也失败: {}, 尝试使用cmd", e2);

                        // 最后的备选方案：使用cmd
                        let args_str = if args.len() > 1 {
                            args[1..].join(" ")
                        } else {
                            String::new()
                        };

                        let mut cmd = Command::new("cmd");
                        cmd.args(&["/C", &format!("\"{}\" {}", exe_path, args_str)])
                            .current_dir(&current_dir);

                        match cmd.spawn() {
                            Ok(_) => {
                                println!("使用cmd启动成功");
                            }
                            Err(e3) => {
                                println!("所有启动方式都失败了: {}, {}, {}", e, e2, e3);
                                return Err(format!("启动新进程失败: 尝试了多种方法都失败了"));
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new(&current_exe)
            .args(&args[1..]) // 跳过第一个参数（可执行文件本身）
            .current_dir(&current_dir)
            .spawn()
            .map_err(|e| format!("启动新进程失败: {}", e))?;
    }

    // 延迟退出当前应用，给新进程启动时间
    thread::sleep(Duration::from_millis(500));

    // 退出当前应用
    std::process::exit(0);
}