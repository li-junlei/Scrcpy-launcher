[根目录](../CLAUDE.md) > **src**

# src/ - Rust 后端模块

> 最后更新：2026-01-27 00:02:27
> 语言：Rust (Edition 2021)
> 状态：✅ 完整扫描

---

## 变更记录

### 2026-01-27
- 初始化模块文档
- 完成所有源文件扫描与注释抽取

---

## 模块职责

**src/** 是 Scrcpy Launcher 的核心后端模块，负责：

1. **应用生命周期管理**：Tauri 应用初始化、窗口事件、单实例限制
2. **IPC 接口层**：定义 25+ 个 Tauri 命令，供前端调用
3. **配置管理**：配置序列化/反序列化、持久化存储
4. **Scrcpy/ADB 交互**：进程管理、网络扫描、无线连接
5. **系统托盘**：托盘图标、菜单、快捷操作
6. **文件传输**：原生 ADB SYNC 协议实现

---

## 入口与启动

### main.rs - 应用入口

**文件路径**：`src/main.rs` (67 行)

**核心职责**：
- Tauri 应用初始化与插件注册
- 系统托盘设置
- 窗口事件处理（关闭时最小化到托盘）
- IPC 命令注册

**关键代码片段**：

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 单实例限制：显示已存在的窗口
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            // 设置系统托盘
            if let Err(e) = tray::setup_tray(app) {
                eprintln!("Failed to setup tray: {}", e);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // 窗口关闭时最小化到托盘，不退出应用
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            // 注册 25+ 个 IPC 命令
            commands::get_config,
            commands::save_config,
            commands::check_adb_status,
            // ... 更多命令
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**启动流程**：
1. 加载 Tauri 插件（shell、dialog、fs、single-instance）
2. 初始化系统托盘
3. 注册窗口事件监听器
4. 注册所有 IPC 命令处理器
5. 启动应用主窗口

---

## 对外接口

### commands.rs - IPC 命令层

**文件路径**：`src/commands.rs` (295 行)

**职责**：定义所有可从前端调用的 Tauri 命令

**命令分类**：

#### 配置管理
- `get_config()` - 获取配置
- `save_config(config)` - 保存配置
- `save_global_settings(dpi, full_res)` - 保存全局设置
- `set_theme(theme)` - 设置主题

#### ADB 操作
- `check_adb_status()` - 检查 ADB 连接状态
- `connect_wireless(ip)` - 无线连接
- `pair_device(addr, code)` - 无线配对（Android 11+）
- `enable_tcpip()` - 启用 TCP/IP 模式（有线转无线）
- `disconnect_all()` - 断开所有连接
- `scan_tcp_devices()` - 扫描局域网设备

#### Scrcpy 启动
- `launch_mirror()` - 启动屏幕镜像
- `launch_audio()` - 启动纯音频
- `launch_app(package, settings, scrcpy_args)` - 启动应用流转
- `kill_scrcpy()` - 关闭所有 Scrcpy 窗口

#### 应用管理
- `get_installed_apps()` - 获取设备已安装应用
- `save_app(package, app_config)` - 保存应用配置
- `delete_app(package)` - 删除应用
- `reorder_apps(new_order)` - 重新排序应用

#### 分辨率预设
- `save_preset(name, long, short, dpi, is_landscape)` - 保存预设
- `update_preset(old_name, new_name, ...)` - 更新预设
- `delete_preset(name)` - 删除预设

#### 其他
- `adb_push_file(window, local_path, remote_path)` - 文件传输（带进度）
- `save_first_run_config(...)` - 保存首次运行配置
- `save_tray_settings(app_count, show_mirror, show_audio)` - 保存托盘设置

**接口示例**：

```rust
#[tauri::command]
pub async fn connect_wireless(ip: String) -> CommandResult {
    scrcpy::connect_wireless(&ip).await
}

#[tauri::command]
pub async fn adb_push_file(
    window: tauri::Window,
    local_path: String,
    remote_path: Option<String>
) -> Result<CommandResult, String> {
    let target = remote_path.unwrap_or_else(|| "/sdcard/Download/".to_string());
    let pusher = AdbPusher::new(None);

    // 进度回调
    let window_clone = window.clone();
    pusher.push(&local_path, &target, Some(Box::new(move |current, total| {
        let percent = if total > 0 { (current as f64 / total as f64) * 100.0 } else { 0.0 };
        let _ = window_clone.emit("adb-push-progress", serde_json::json!({
            "progress": percent as u32,
            "message": format!("传输中: {:.1}%", percent)
        }));
    }))).await.map_err(|e| e.to_string())?;

    Ok(CommandResult {
        success: true,
        message: format!("发送成功: {}", local_path),
    })
}
```

---

## 关键依赖与配置

### Cargo.toml - 依赖配置

**核心依赖**：

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dirs = "5"
indexmap = { version = "2", features = ["serde"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
byteorder = "1.4"

[target.'cfg(windows)'.dependencies]
tauri-plugin-single-instance = "2"
```

**依赖说明**：
- **tauri 2.0**：应用框架，提供窗口、IPC、托盘等功能
- **serde**：配置序列化/反序列化（JSON）
- **tokio**：异步运行时（用于网络扫描、文件传输）
- **anyhow**：错误处理
- **dirs**：获取系统目录（配置文件路径）
- **indexmap**：有序 HashMap（保持应用顺序）

**编译配置**：

```toml
[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "z"
strip = true
```

优化策略：体积优先（`opt-level = "z"`）、LTO 链接优化、剥离调试符号。

---

## 数据模型

### config.rs - 配置数据结构

**文件路径**：`src/config.rs` (268 行)

**核心数据结构**：

```rust
/// 主配置结构
pub struct Config {
    pub adb_history: Vec<String>,              // ADB 连接历史
    pub global_settings: GlobalSettings,        // 全局设置
    pub apps: IndexMap<String, AppConfig>,      // 应用列表（有序）
    pub presets: HashMap<String, PresetConfig>, // 分辨率预设
    pub first_run: bool,                        // 首次运行标记
    pub use_custom_args: bool,                  // 使用自定义参数
    pub custom_args: String,                    // 自定义参数字符串
    pub scrcpy_options: ScrcpyOptions,          // Scrcpy 选项（镜像）
    pub use_app_stream_args: bool,              // 应用流转使用独立选项
    pub use_app_custom_args: bool,              // 应用流转使用自定义参数
    pub app_custom_args: String,                // 应用流转自定义参数
    pub app_stream_options: ScrcpyOptions,      // Scrcpy 选项（应用流转）
    pub tray_app_count: u32,                    // 托盘显示应用数量
    pub tray_show_mirror: bool,                 // 托盘显示镜像选项
    pub tray_show_audio: bool,                  // 托盘显示音频选项
}

/// 全局显示设置
pub struct GlobalSettings {
    pub use_full_res_switch: bool,
    pub full_res: String,   // 分辨率（如 "1200x2670"）
    pub dpi: u32,
    pub is_landscape: bool,
    pub theme: String,      // "light" 或 "dark"
}

/// 应用配置
pub struct AppConfig {
    pub name: String,
    pub settings: Option<AppSettings>,     // 分辨率设置
    pub scrcpy_args: Option<String>,       // 自定义 Scrcpy 参数
}

/// 应用分辨率设置
pub struct AppSettings {
    pub use_full_res_switch: bool,
    pub full_res: String,
    pub dpi: u32,
    pub is_landscape: bool,
}

/// 分辨率预设
pub struct PresetConfig {
    pub long: u32,
    pub short: u32,
    pub dpi: u32,
    pub is_landscape: bool,
}

/// Scrcpy 启动选项
pub struct ScrcpyOptions {
    pub hid_keyboard: bool,        // HID 键盘
    pub stay_awake: bool,          // 保持唤醒
    pub turn_screen_off: bool,     // 关闭屏幕
    pub show_touches: bool,        // 显示触摸
    pub always_on_top: bool,       // 窗口置顶
    pub borderless: bool,          // 无边框
    pub power_off_on_close: bool,  // 关闭时断电
    pub local_ime: bool,           // 本地输入法
    pub max_size: u32,             // 最大尺寸
    pub max_fps: u32,              // 最大帧率
}
```

**配置持久化**：

- 文件路径：可执行文件同目录的 `config.json`
- 加载策略：不存在则创建默认配置
- 保存时机：每次修改后立即保存
- 编码：UTF-8，JSON 格式，美化输出（`serde_json::to_string_pretty`）

---

## 核心功能实现

### scrcpy.rs - Scrcpy/ADB 交互

**文件路径**：`src/scrcpy.rs` (772 行)

**核心功能**：

#### 1. ADB 状态检查

```rust
pub fn check_adb_status() -> AdbStatus {
    let adb_path = get_adb_path();
    let output = create_command(&adb_path)
        .arg("devices")
        .output();

    // 解析输出，查找 "device" 标记
    // 返回连接状态和设备 ID
}
```

#### 2. 无线连接（优化版）

```rust
pub async fn connect_wireless(ip: &str) -> CommandResult {
    // 1. TCP 预检查（2 秒超时）
    // 避免连接无效 IP 时长时间卡顿
    if let Ok(addr) = target_addr_str.parse::<std::net::SocketAddr>() {
        match tokio::time::timeout(
            Duration::from_secs(2),
            tokio::net::TcpStream::connect(addr)
        ).await {
            Ok(Ok(_)) => { /* 继续连接 */ },
            Ok(Err(e)) => return Err("无法连接到设备"),
            Err(_) => return Err("连接超时"),
        }
    }

    // 2. 执行真正的 adb connect
    let output = create_command(&adb_path)
        .args(["connect", ip])
        .output();

    // 解析结果
}
```

#### 3. 局域网扫描（智能多网卡）

```rust
pub async fn scan_local_network() -> Vec<String> {
    let local_ips = get_all_local_ips();  // 获取所有本地 IP
    let mut tasks = Vec::new();

    // 针对每个本地 IP 的网段进行 /24 扫描
    for local_ip in local_ips {
        let prefix = format!("{}.{}.{}.", parts[0], parts[1], parts[2]);

        for i in 1..255 {
            let ip = format!("{}{}", prefix, i);
            tasks.push(tokio::spawn(async move {
                // 绑定本地 IP，绕过 VPN/TUN 劫持
                let socket = tokio::net::TcpSocket::new_v4()?;
                socket.bind(bind_addr)?;
                tokio::time::timeout(
                    Duration::from_millis(150),
                    socket.connect(target_addr)
                ).await
            }));
        }
    }

    // 收集结果并去重
}
```

**关键特性**：
- **多网卡支持**：自动识别所有物理网卡
- **VPN 兼容**：绑定本地 IP，绕过虚拟网卡劫持
- **虚拟网卡过滤**：排除 VMware、VirtualBox、WSL、TUN 等
- **快速扫描**：150ms 超时，并行扫描

#### 4. Scrcpy 启动参数构建

```rust
pub fn launch_scrcpy(mode: LaunchMode) -> CommandResult {
    let config = Config::load();
    let scrcpy_path = get_scrcpy_path();
    let mut args = Vec::new();

    // 根据模式选择参数源
    let scrcpy_args_list = match &mode {
        LaunchMode::App { scrcpy_args: Some(custom_args), .. } => {
            // 应用专属自定义参数
            custom_args.split_whitespace().map(String::from).collect()
        }
        LaunchMode::App { .. } if config.use_app_stream_args => {
            // 应用流转默认参数
            build_args_from_options(&config.app_stream_options)
        }
        _ => {
            // 全局参数
            build_args_from_options(&config.scrcpy_options)
        }
    };

    args.extend(scrcpy_args_list);

    // 根据模式添加特定参数
    match &mode {
        LaunchMode::Audio => {
            args.push("--no-video".to_string());
        }
        LaunchMode::Mirror => {
            args.push(format!("--max-size={}", max_dim));
        }
        LaunchMode::App { package, settings, .. } => {
            args.push(format!("--new-display={}/{}", full_res, dpi));
            args.push(format!("--start-app={}", package));
        }
    }

    // 启动进程（分离模式）
    create_command(&scrcpy_path)
        .args(&args)
        .spawn()?;
}
```

---

### adb_sync.rs - 原生 ADB SYNC 协议

**文件路径**：`src/adb_sync.rs` (117 行)

**职责**：实现 ADB SYNC 协议，提供精准的文件传输进度反馈

**核心实现**：

```rust
pub struct AdbPusher {
    device_serial: Option<String>,
}

impl AdbPusher {
    /// 连接 ADB Server 并建立 Transport
    async fn connect(&self) -> Result<TcpStream> {
        let mut stream = TcpStream::connect("127.0.0.1:5037").await?;

        // 切换到指定设备
        let target = match &self.device_serial {
            Some(s) => format!("host:transport:{}", s),
            None => "host:transport-any".to_string(),
        };
        self.send_packet(&mut stream, &target).await?;
        self.read_status(&mut stream).await?;

        Ok(stream)
    }

    /// 发送 ADB 格式的数据包（4 字节长度 + 内容）
    async fn send_packet(&self, stream: &mut TcpStream, payload: &str) -> Result<()> {
        let len_str = format!("{:04x}", payload.len());
        stream.write_all(len_str.as_bytes()).await?;
        stream.write_all(payload.as_bytes()).await?;
        Ok(())
    }

    /// 带进度的文件传输
    pub async fn push(
        &self,
        local_path: &str,
        remote_path: &str,
        callback: Option<ProgressCallback>
    ) -> Result<()> {
        let mut file = File::open(local_path).await?;
        let file_size = file.metadata()?.len();
        let mut buffer = [0u8; 64 * 1024]; // 64KB 块大小

        // 1. 连接并进入 SYNC 模式
        let mut stream = self.connect().await?;
        self.send_packet(&mut stream, "sync:").await?;
        self.read_status(&mut stream).await?;

        // 2. 发送 SEND 请求
        stream.write_all(b"SEND").await?;
        stream.write_u32_le(remote_path_bytes.len() as u32).await?;
        stream.write_all(remote_path_bytes).await?;

        // 3. 循环发送 DATA 数据块
        let mut total_sent = 0u64;
        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 { break; }

            stream.write_all(b"DATA").await?;
            stream.write_u32_le(n as u32).await?;
            stream.write_all(&buffer[..n]).await?;

            total_sent += n as u64;
            if let Some(cb) = &callback {
                cb(total_sent, file_size);  // 触发进度回调
            }
        }

        // 4. 发送 DONE（结束 + 修改时间）
        stream.write_all(b"DONE").await?;
        stream.write_u32_le(timestamp).await?;

        // 5. 等待服务器确认 OKAY
        stream.read_exact(&mut resp).await?;
        if &resp != b"OKAY" {
            bail!("传输未被确认");
        }

        Ok(())
    }
}
```

**协议特点**：
- **二进制协议**：4 字节长度前缀 + ASCII 命令 + 数据
- **块传输**：64KB 块大小（ADB 推荐）
- **进度回调**：实时反馈传输进度
- **错误处理**：检查 OKAY/FAIL 状态码

---

### tray.rs - 系统托盘

**文件路径**：`src/tray.rs` (123 行)

**职责**：创建系统托盘图标、菜单、事件处理

**核心实现**：

```rust
pub fn setup_tray<R: Runtime>(app: &tauri::App<R>) -> Result<()> {
    let config = Config::load();

    // 创建菜单项
    let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "隐藏到托盘", true, None::<&str>)?;
    let mirror_item = MenuItem::with_id(app, "mirror", "屏幕镜像", true, None::<&str>)?;
    let audio_item = MenuItem::with_id(app, "audio", "纯音频", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    // 动态创建应用快捷方式
    let app_menu_items: Vec<MenuItem<R>> = config.apps
        .iter()
        .take(config.tray_app_count as usize)
        .filter_map(|(pkg, app_config)| {
            MenuItem::with_id(app, &format!("app:{}", pkg), &app_config.name, true, None::<&str>).ok()
        })
        .collect();

    // 构建菜单
    let menu = Menu::with_items(app, &menu_items)?;

    // 创建托盘图标
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Scrcpy Launcher")
        .menu(&menu)
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "show" => { /* 显示窗口 */ }
                "hide" => { /* 隐藏窗口 */ }
                "mirror" => { /* 启动镜像 */ }
                "audio" => { /* 启动音频 */ }
                "quit" => {
                    cleanup_before_exit();
                    app.exit(0);
                }
                id if id.starts_with("app:") => {
                    // 启动应用
                    let package = id.strip_prefix("app:").unwrap();
                    // ...
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            // 左键点击显示窗口
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
```

**功能**：
- 左键点击：显示/隐藏主窗口
- 右键点击：显示菜单（应用快捷方式、镜像、音频、退出）
- 动态菜单：根据配置显示最近使用的应用

---

## 测试与质量

### 当前状态

⚠️ **无自动化测试**

- 无单元测试文件（`*_test.rs`）
- 无文档测试（`///` 中的示例）
- 无集成测试（`tests/` 目录）

### 建议的测试改进

#### 单元测试示例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.first_run, true);
        assert_eq!(config.global_settings.theme, "light");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config.first_run, deserialized.first_run);
    }

    #[test]
    fn test_add_adb_history() {
        let mut config = Config::default();
        config.add_adb_history("192.168.1.100");
        assert_eq!(config.adb_history[0], "192.168.1.100");
    }
}
```

#### 集成测试建议

- 测试 ADB 命令执行（使用模拟 ADB）
- 测试文件传输流程
- 测试网络扫描逻辑

---

## 常见问题 (FAQ)

### Q1: 如何添加新的 Tauri 命令？

**A**: 按以下步骤操作：

1. 在 `commands.rs` 中定义函数：
```rust
#[tauri::command]
pub fn my_new_command(param: String) -> CommandResult {
    CommandResult {
        success: true,
        message: "操作成功".to_string(),
    }
}
```

2. 在 `main.rs` 中注册：
```rust
.invoke_handler(tauri::generate_handler![
    // ... 其他命令
    commands::my_new_command,
])
```

3. 在前端调用（`dist/main.js`）：
```javascript
const result = await invoke('my_new_command', { param: '值' });
```

### Q2: 如何扩展配置结构？

**A**:

1. 在 `config.rs` 的 `Config` 结构中添加字段：
```rust
pub struct Config {
    // ... 现有字段
    #[serde(default = "default_my_field")]
    pub my_new_field: String,
}

fn default_my_field() -> String {
    "默认值".to_string()
}
```

2. 更新 `Default` 实现

3. 前端可通过 `get_config()` 获取新字段

### Q3: 局域网扫描如何工作？

**A**: 扫描流程如下：

1. 获取所有本地网卡 IP（排除虚拟网卡）
2. 针对每个 IP 的 /24 网段，并行扫描 1-254 端口
3. 绑定本地 IP 进行 TCP 连接（绕过 VPN 劫持）
4. 150ms 超时，快速检测 5555 端口
5. 收集结果并去重

**虚拟网卡过滤**：排除 VMware、VirtualBox、WSL、TUN、VPN 等关键词。

### Q4: 如何处理 ADB 连接超时？

**A**: 在 `scrcpy.rs` 的 `connect_wireless` 函数中：

1. 先进行 TCP 预检查（2 秒超时）
2. 如果预检查失败，立即返回错误
3. 如果预检查成功，再执行 `adb connect`

这样可避免连接无效 IP 时长时间卡顿。

### Q5: 文件传输进度如何实现？

**A**: 使用原生 ADB SYNC 协议：

1. 建立到 ADB Server 的 TCP 连接（127.0.0.1:5037）
2. 进入 SYNC 模式（`sync:` 命令）
3. 发送 SEND 请求（包含远程路径）
4. 循环发送 DATA 数据块（64KB）
5. 每发送一块，触发进度回调
6. 发送 DONE（结束标记）
7. 等待 OKAY 确认

进度回调通过 Tauri 事件系统发送到前端：
```rust
window.emit("adb-push-progress", serde_json::json!({
    "progress": percent,
    "message": format!("传输中: {:.1}%", percent)
}));
```

---

## 相关文件清单

### 核心文件

| 文件 | 行数 | 职责 |
|------|------|------|
| `main.rs` | 67 | 应用入口、窗口事件、命令注册 |
| `lib.rs` | 8 | 库导出声明 |
| `commands.rs` | 295 | IPC 命令接口（25+ 命令） |
| `config.rs` | 268 | 配置数据结构与管理 |
| `scrcpy.rs` | 772 | Scrcpy/ADB 核心逻辑 |
| `tray.rs` | 123 | 系统托盘实现 |
| `adb_sync.rs` | 117 | ADB SYNC 协议实现 |

**总行数**：~1650 行（不含注释和空行）

---

## 下一步建议

### 功能扩展

- [ ] 添加日志系统（替换 `eprintln!`）
- [ ] 支持多设备同时连接
- [ ] 优化大文件传输性能（分块并行）
- [ ] 添加设备信息获取（型号、Android 版本）

### 技术改进

- [ ] 引入错误类型定义（替换 `CommandResult`）
- [ ] 使用 `tracing` 替代 `println!` 日志
- [ ] 添加单元测试和集成测试
- [ ] 代码重构：提取 ADB 客户端为独立模块

### 文档补充

- [ ] 为所有公共函数添加文档注释
- [ ] 生成 API 文档（`cargo doc`）
- [ ] 编写架构设计文档

---

**最后扫描时间**：2026-01-27 00:02:27
**文档版本**：v1.0
**作者**：AI 初始化助手
