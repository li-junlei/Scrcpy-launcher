//! Scrcpy Launcher - Scrcpy 核心模块
//!
//! 处理与 scrcpy 和 adb 的所有交互，包括：
//! - ADB 设备状态检查
//! - 无线连接
//! - 构建和执行 scrcpy 命令

use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::io::BufReader;
use crate::config::{Config, ScrcpyOptions};
use tauri::Emitter;
use std::time::Duration;


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
/// 无线连接到设备
pub async fn connect_wireless(ip: &str) -> CommandResult {
    let adb_path = get_adb_path();
    
    // 保存到历史记录
    let mut config = Config::load();
    config.add_adb_history(ip);

    // 1. TCP 预检查：快速检测目标是否可达，避免 adb connect 阻塞太久
    // 处理端口：如果用户输入没带端口，默认 5555
    let target_addr_str = if ip.contains(':') {
        ip.to_string()
    } else {
        format!("{}:5555", ip)
    };

    if let Ok(addr) = target_addr_str.parse::<std::net::SocketAddr>() {
        // 尝试建立 TCP 连接，超时设置为 2 秒
        // 使用 tokio::net::TcpStream
        let check_result = tokio::time::timeout(
            Duration::from_secs(2),
            tokio::net::TcpStream::connect(addr)
        ).await;

        match check_result {
            Ok(Ok(_)) => {
                // 连接成功，目标在线，继续执行 ADB 连接
            },
            Ok(Err(e)) => {
                return CommandResult {
                    success: false,
                    message: format!("无法连接到设备 (拒绝连接): {}", e),
                };
            },
            Err(_) => {
                // 超时
                return CommandResult {
                    success: false,
                    message: "连接超时：设备不可达或未开启无线调试".to_string(),
                };
            }
        }
    } else {
         // 解析 IP 失败，可能是域名，跳过预检查直接交给 ADB，或者直接报错
         // 这里选择继续交给 ADB 处理，也许 ADB 能处理某些特殊格式
    }

    // 2. 执行真正的 adb connect
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

/// 通过 ipconfig 获取所有本机 IPv4 地址，并过滤掉虚拟网卡
fn get_all_local_ips() -> Vec<String> {
    let output = std::process::Command::new("ipconfig")
        .output()
        .ok();

    let mut ips = Vec::new();

    if let Some(output) = output {
        // Windows 的 ipconfig 输出编码通常是 GBK (中文环境)，但 `String::from_utf8_lossy` 处理 GBK 会乱码。
        // 不过我们主要匹配 "IPv4" (ASCII) 和 ":", 以及 IP 数字。
        // 适配器名称如果是中文可能会乱码，导致过滤失效。
        // 这是一个潜在风险点。但通常 Virtual/VMware 等关键词是英文。
        // "vEthernet" 也是英文。
        // 如果能检测到 "Virtual", "Pseudo", "VMware", "Box", "VPN" 等关键词最好。
        
        // 稍微优化：尝试 decode GBK 最好，但引入依赖麻烦。
        // 这里假设关键的虚拟网卡标识通常包含英文部分，或者通过 IP 特征辅助过滤。
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut current_adapter = String::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // 适配器行通常以 ":" 结尾，且不包含 " . . ."
            // 例如 "Ethernet adapter Ethernet:" 或 "以太网适配器 vEthernet (WSL):"
            if line.ends_with(':') && !line.contains(". . .") {
                current_adapter = line.to_lowercase();
                continue;
            }

            // 匹配 IPv4
            if line.contains("IPv4") && line.contains(":") {
                // 检查当前适配器是否应该被忽略
                if current_adapter.contains("vmware") 
                    || current_adapter.contains("virtual") 
                    || current_adapter.contains("vethernet") // Hyper-V / WSL
                    || current_adapter.contains("pseudo")
                    || current_adapter.contains("tap-windows")
                    || current_adapter.contains("vpn")
                    || current_adapter.contains("tun")
                    || current_adapter.contains("singbox")
                    || current_adapter.contains("wsl")
                {
                    continue;
                }

                let parts: Vec<&str> = line.split(':').collect();
                if let Some(ip_part) = parts.last() {
                    let ip = ip_part.trim().to_string();
                    
                    // 过滤 IP 特征
                    if ip == "127.0.0.1" { continue; }
                    
                    // 过滤常见的 TUN/Fake IP 网段
                    // 198.18.0.0/15 是保留用于性能测试的，常被 TUN 模式用来做 Fake IP
                    if ip.starts_with("198.18.") { continue; }
                    
                    // 169.254.x.x (APIPA)
                    if ip.starts_with("169.254.") { continue; }

                    if ip.split('.').count() == 4 {
                        ips.push(ip);
                    }
                }
            }
        }
    }
    
    // 兜底：如果过滤太严格导致没 IP 了，尝试 UDP 方式（至少能拿到一个出网 IP）
    if ips.is_empty() {
         if let Some(ip) = get_local_ip_udp() {
             // 再次检查 UDP 拿到的 IP 是否也是 TUN IP
             if !ip.starts_with("198.18.") {
                 ips.push(ip);
             }
         }
    }

    ips
}

/// UDP 方式获取 IP (原 get_local_ip)
fn get_local_ip_udp() -> Option<String> {
    // 这是一个同步调用，我们在 async 上下文中不能直接用?转换 async 结果
    // 这里简单起见，既然 scan_local_network 是 async 的，我们可以稍微从简
    // 但为了不引入 complex async block for now, use std::net if possible or just assume this is rare fallback
    // Use std::net::UdpSocket for synchronous check
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

/// 扫描局域网内开放 5555 端口的设备
pub async fn scan_local_network() -> Vec<String> {
    let local_ips = get_all_local_ips();
    let mut tasks = Vec::new();

    // 针对每个找到的本地 IP 所在的网段进行扫描
    for local_ip in local_ips {
        let parts: Vec<&str> = local_ip.split('.').collect();
        if parts.len() != 4 {
            continue;
        }

        // 假设是 /24 子网
        let prefix = format!("{}.{}.{}.", parts[0], parts[1], parts[2]);
        
        for i in 1..255 {
            let ip = format!("{}{}", prefix, i);
            if ip == local_ip {
                continue;
            }

            let bind_ip = local_ip.clone();

            tasks.push(tokio::spawn(async move {
                let target_addr: std::net::SocketAddr = format!("{}:5555", ip).parse().ok()?;
                let bind_addr: std::net::SocketAddr = format!("{}:0", bind_ip).parse().ok()?;
                
                // 使用 TcpSocket 绑定本地 IP，这可以强制流量走正确的物理网卡，
                // 从而绕过 V2Ray/Tun 模式的全局流量劫持 (因为 Tun 通常无法劫持绑定了特定物理 IP 的流量)
                let socket = tokio::net::TcpSocket::new_v4().ok()?;
                socket.bind(bind_addr).ok()?;

                // 缩短超时时间以加快多网段扫描速度
                match tokio::time::timeout(Duration::from_millis(150), socket.connect(target_addr)).await {
                    Ok(Ok(_)) => Some(ip),
                    _ => None,
                }
            }));
        }
    }

    let mut devices = Vec::new();
    for task in tasks {
        if let Ok(Some(ip)) = task.await {
            // 去重
            if !devices.contains(&ip) {
                devices.push(ip);
            }
        }
    }
    
    // 排序
    devices.sort_by(|a, b| {
        let a_parts: Vec<&str> = a.split('.').collect();
        let b_parts: Vec<&str> = b.split('.').collect();
        // 简单按最后一段排序，不够严谨但够用
        let a_last: u8 = a_parts.last().unwrap_or(&"0").parse().unwrap_or(0);
        let b_last: u8 = b_parts.last().unwrap_or(&"0").parse().unwrap_or(0);
        a_last.cmp(&b_last)
    });

    devices
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
    if let Some(stderr) = child.stderr.take() {
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
