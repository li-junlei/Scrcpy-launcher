//! Scrcpy Launcher - Scrcpy 核心模块
//!
//! 处理与 scrcpy 和 adb 的所有交互，包括：
//! - ADB 设备状态检查
//! - 无线连接
//! - 构建和执行 scrcpy 命令

use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::io::{BufRead, BufReader};
use crate::config::{Config, ScrcpyOptions};
use tauri::Emitter;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Windows: CREATE_NO_WINDOW flag
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 获取资源目录路径
fn get_resources_path() -> PathBuf {
    // 在开发模式下使用相对路径，打包后使用资源目录
    if cfg!(debug_assertions) {
        PathBuf::from("resources")
    } else {
        // 获取可执行文件所在目录
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("resources"))
            .unwrap_or_else(|| PathBuf::from("resources"))
    }
}

/// 获取 ADB 可执行文件路径
fn get_adb_path() -> PathBuf {
    get_resources_path().join("bin").join("adb.exe")
}

/// 获取 Scrcpy 可执行文件路径
fn get_scrcpy_path() -> PathBuf {
    get_resources_path().join("bin").join("scrcpy.exe")
}

/// ADB 状态检查结果
#[derive(serde::Serialize, Clone)]
pub struct AdbStatus {
    pub connected: bool,
    pub message: String,
}

/// 创建命令并配置为无窗口模式（仅 Windows）
fn create_command(program: &PathBuf) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// 创建命令并配置为无窗口模式（程序名版本）
fn create_command_str(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// 检查 ADB 连接状态
pub fn check_adb_status() -> AdbStatus {
    let adb_path = get_adb_path();
    
    let output = create_command(&adb_path)
        .arg("devices")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().collect();
            
            // 跳过第一行 "List of devices attached"
            let devices: Vec<&str> = lines
                .iter()
                .skip(1)
                .filter(|line| line.contains("device") && !line.contains("offline"))
                .copied()
                .collect();

            if !devices.is_empty() {
                let device_id = devices[0].split_whitespace().next().unwrap_or("unknown");
                AdbStatus {
                    connected: true,
                    message: format!("已连接: {}", device_id),
                }
            } else {
                AdbStatus {
                    connected: false,
                    message: "未连接设备".to_string(),
                }
            }
        }
        Err(_) => AdbStatus {
            connected: false,
            message: "未找到ADB命令".to_string(),
        },
    }
}

/// 命令执行结果
#[derive(serde::Serialize, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
}

/// 无线连接到设备
pub fn connect_wireless(ip: &str) -> CommandResult {
    let adb_path = get_adb_path();
    
    // 保存到历史记录
    let mut config = Config::load();
    config.add_adb_history(ip);

    let output = create_command(&adb_path)
        .args(["connect", ip])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("connected to") {
                CommandResult {
                    success: true,
                    message: stdout.to_string(),
                }
            } else {
                CommandResult {
                    success: false,
                    message: stdout.to_string(),
                }
            }
        }
        Err(e) => CommandResult {
            success: false,
            message: format!("连接失败: {}", e),
        },
    }
}

/// 无线配对设备 (Android 11+)
pub fn pair_device(addr: &str, code: &str) -> CommandResult {
    let adb_path = get_adb_path();
    
    // adb pair <ip>:<port> <code>
    let output = create_command(&adb_path)
        .args(["pair", addr, code])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // 成功通常输出 "Successfully paired to ..."
            if stdout.contains("Successfully paired") || output.status.success() {
                CommandResult {
                    success: true,
                    message: format!("配对成功: {}\n{}", stdout, stderr),
                }
            } else {
                CommandResult {
                    success: false,
                    message: format!("配对失败: {}\n{}", stdout, stderr),
                }
            }
        }
        Err(e) => CommandResult {
            success: false,
            message: format!("执行失败: {}", e),
        },
    }
}

/// 启用 TCP/IP 模式
pub fn enable_tcpip_mode() -> CommandResult {
    let adb_path = get_adb_path();

    let output = create_command(&adb_path)
        .args(["tcpip", "5555"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if stdout.contains("restarting in TCP mode") || output.status.success() {
                CommandResult {
                    success: true,
                    message: "已开启无线调试 (端口 5555)，请断开USB线并输入IP连接。".to_string(),
                }
            } else {
                CommandResult {
                    success: false,
                    message: format!("失败: {}", if stderr.is_empty() { "未连接设备" } else { &stderr }),
                }
            }
        }
        Err(e) => CommandResult {
            success: false,
            message: format!("执行失败: {}", e),
        },
    }
}

/// 断开所有设备连接
pub fn disconnect_all() -> CommandResult {
    let adb_path = get_adb_path();

    let output = create_command(&adb_path)
        .arg("disconnect")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("disconnected everything") || output.status.success() {
                CommandResult {
                    success: true,
                    message: "已断开所有连接。".to_string(),
                }
            } else {
                CommandResult {
                    success: false,
                    message: "断开失败".to_string(),
                }
            }
        }
        Err(e) => CommandResult {
            success: false,
            message: format!("执行失败: {}", e),
        },
    }
}

/// 获取已安装的第三方应用列表
pub fn get_installed_apps() -> Result<Vec<String>, String> {
    let adb_path = get_adb_path();

    let output = create_command(&adb_path)
        .args(["shell", "pm", "list", "packages", "-3"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("执行失败: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let packages: Vec<String> = stdout
        .lines()
        .map(|line| line.replace("package:", "").trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(packages)
}

/// 关闭所有 scrcpy 进程
pub fn kill_scrcpy_processes() -> CommandResult {
    let output = create_command_str("taskkill")
        .args(["/F", "/IM", "scrcpy.exe"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Windows 中文版输出
            if stdout.contains("成功") || stdout.contains("SUCCESS") {
                CommandResult {
                    success: true,
                    message: "已关闭所有 Scrcpy 窗口。".to_string(),
                }
            } else if stdout.contains("没有找到") || stdout.contains("not found") {
                CommandResult {
                    success: true,
                    message: "当前没有正在运行的 Scrcpy 窗口。".to_string(),
                }
            } else {
                CommandResult {
                    success: true,
                    message: "关闭指令已发送。".to_string(),
                }
            }
        }
        Err(e) => CommandResult {
            success: false,
            message: format!("执行失败: {}", e),
        },
    }
}

/// 关闭 ADB 服务器
pub fn kill_adb_server() -> CommandResult {
    let adb_path = get_adb_path();
    
    let _ = create_command(&adb_path)
        .arg("kill-server")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();

    let _ = create_command_str("taskkill")
        .args(["/F", "/IM", "adb.exe"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();

    CommandResult {
        success: true,
        message: "ADB 服务已停止".to_string(),
    }
}

/// 退出前清理
pub fn cleanup_before_exit() {
    kill_scrcpy_processes();
    kill_adb_server();
}

/// 发送文件到设备 (带进度)
pub fn push_file<R: tauri::Runtime>(window: &tauri::Window<R>, local_path: &str, remote_path: &str) -> CommandResult {
    let adb_path = get_adb_path();

    // 尝试启动 adb push 进程
    // -p: 显示进度 (即使重定向输出也能强制显示)
    let mut child = match create_command(&adb_path)
        .args(["push", "-p", local_path, remote_path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
            Ok(c) => c,
            Err(e) => return CommandResult {
                success: false,
                message: format!("启动失败: {}", e),
            },
        };

    // 收集 stderr 用于最终结果
    let mut collected_stderr = Vec::new();

    // 处理标准错误 (ADB 进度信息通常在 stderr)
    if let Some(mut stderr) = child.stderr.take() {
        use std::io::Read;
        let mut reader = BufReader::new(stderr);
        let mut buffer = Vec::new();
        let mut byte = [0u8; 1];

        while let Ok(n) = reader.read(&mut byte) {
            if n == 0 { break; }
            
            collected_stderr.push(byte[0]);

            // 遇到回车或换行符时处理缓冲区
            if byte[0] == b'\r' || byte[0] == b'\n' {
                if !buffer.is_empty() {
                    let line = String::from_utf8_lossy(&buffer);
                    if line.contains("%]") {
                        if let Some(start) = line.find('[') {
                            if let Some(end) = line.find("%]") {
                                let percent_str = &line[start+1..end].trim();
                                if let Ok(percent) = percent_str.parse::<u32>() {
                                    let _ = window.emit("adb-push-progress", serde_json::json!({
                                        "progress": percent,
                                        "message": format!("正在传输: {}%", percent)
                                    }));
                                }
                            }
                        }
                    }
                    buffer.clear();
                }
            } else {
                buffer.push(byte[0]);
            }
        }
    }

    // 等待进程结束
    match child.wait_with_output() {
        Ok(output) => {
            // output.stderr 已经被我们 take() 走了，所以使用收集到的 collected_stderr
            let stderr = String::from_utf8_lossy(&collected_stderr);
            
            if output.status.success() {
                // 发送 100% 进度
                let _ = window.emit("adb-push-progress", serde_json::json!({
                    "progress": 100,
                    "message": "传输完成"
                }));
                
                CommandResult {
                    success: true,
                    message: format!("发送成功: {}", local_path),
                }
            } else {
                 CommandResult {
                    success: false,
                    message: format!("发送失败: {}", stderr),
                }
            }
        }
        Err(e) => CommandResult {
            success: false,
            message: format!("执行中断: {}", e),
        },
    }
}

/// 从选项构建参数列表
fn build_args_from_options(opts: &ScrcpyOptions) -> Vec<String> {
    let mut args = Vec::new();
    
    if opts.hid_keyboard {
        args.push("-K".to_string());
    }
    if opts.stay_awake {
        args.push("--stay-awake".to_string());
    }
    if opts.turn_screen_off {
        args.push("-S".to_string());
    }
    if opts.show_touches {
        args.push("--show-touches".to_string());
    }
    if opts.always_on_top {
        args.push("--always-on-top".to_string());
    }
    if opts.borderless {
        args.push("--window-borderless".to_string());
    }
    if opts.power_off_on_close {
        args.push("--power-off-on-close".to_string());
    }
    if opts.local_ime {
        args.push("--display-ime-policy=local".to_string());
    }
    if opts.max_size > 0 {
        args.push(format!("-m {}", opts.max_size));
    }
    if opts.max_fps > 0 {
        args.push(format!("--max-fps={}", opts.max_fps));
    }
    
    args
}

/// 启动模式
pub enum LaunchMode {
    Mirror,
    Audio,
    App { package: String, settings: Option<crate::config::AppSettings>, scrcpy_args: Option<String> },
}

/// 构建并运行 scrcpy 命令
pub fn launch_scrcpy(mode: LaunchMode) -> CommandResult {
    let config = Config::load();
    let scrcpy_path = get_scrcpy_path();
    
    // 构建参数列表
    let mut args: Vec<String> = Vec::new();
    
    // 根据模式确定使用哪些 scrcpy 选项
    let scrcpy_args_list = match &mode {
        LaunchMode::App { scrcpy_args: Some(custom_args), .. } => {
            // 应用专属自定义参数
            custom_args.split_whitespace().map(String::from).collect()
        }
        LaunchMode::App { .. } if config.use_app_stream_args => {
            if config.use_app_custom_args {
                config.app_custom_args.split_whitespace().map(String::from).collect()
            } else {
                build_args_from_options(&config.app_stream_options)
            }
        }
        _ => {
            if config.use_custom_args {
                config.custom_args.split_whitespace().map(String::from).collect()
            } else {
                build_args_from_options(&config.scrcpy_options)
            }
        }
    };
    
    args.extend(scrcpy_args_list);
    
    // 根据模式添加特定参数
    match &mode {
        LaunchMode::Audio => {
            args.push("--no-video".to_string());
        }
        LaunchMode::Mirror => {
            let res_str = &config.global_settings.full_res;
            if let Some(max_dim) = res_str
                .split('x')
                .filter_map(|s| s.parse::<u32>().ok())
                .max()
            {
                args.push(format!("--max-size={}", max_dim));
            } else {
                args.push("--max-size=1920".to_string());
            }
        }
        LaunchMode::App { package, settings, .. } => {
            let (dpi, full_res) = if let Some(s) = settings {
                (s.dpi, s.full_res.clone())
            } else {
                (config.global_settings.dpi, config.global_settings.full_res.clone())
            };
            
            args.push(format!("--new-display={}/{}", full_res, dpi));
            args.push(format!("--start-app={}", package));
            args.push("--capture-orientation=0".to_string());
        }
    }
    
    // 启动 scrcpy
    let result = create_command(&scrcpy_path)
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    
    match result {
        Ok(_) => {
            let mode_str = match &mode {
                LaunchMode::Mirror => "屏幕镜像",
                LaunchMode::Audio => "纯音频",
                LaunchMode::App { package, .. } => package,
            };
            CommandResult {
                success: true,
                message: format!("已启动 ({})", mode_str),
            }
        }
        Err(e) => CommandResult {
            success: false,
            message: format!("启动失败: {}", e),
        },
    }
}
