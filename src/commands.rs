//! Scrcpy Launcher - Tauri 命令模块
//!
//! 定义所有可以从前端调用的 Tauri 命令

use crate::config::{Config, AppConfig, AppSettings, ScrcpyOptions};
use crate::scrcpy::{self, AdbStatus, CommandResult, LaunchMode};
use crate::adb_sync::AdbPusher;
use tauri::Emitter;


/// 获取配置
#[tauri::command]
pub fn get_config() -> Config {
    Config::load()
}

/// 保存配置
#[tauri::command]
pub fn save_config(config: Config) {
    config.save();
}

/// 检查 ADB 状态
#[tauri::command]
pub fn check_adb_status() -> AdbStatus {
    scrcpy::check_adb_status()
}

/// 无线连接
/// 无线连接
#[tauri::command]
pub async fn connect_wireless(ip: String) -> CommandResult {
    scrcpy::connect_wireless(&ip).await
}

/// 无线配对
#[tauri::command]
pub fn pair_device(addr: String, code: String) -> CommandResult {
    scrcpy::pair_device(&addr, &code)
}

/// 启用 TCP/IP 模式
#[tauri::command]
pub fn enable_tcpip() -> CommandResult {
    scrcpy::enable_tcpip_mode()
}

/// 断开所有连接
#[tauri::command]
pub fn disconnect_all() -> CommandResult {
    scrcpy::disconnect_all()
}

/// 获取已安装应用
#[tauri::command]
pub fn get_installed_apps() -> Result<Vec<String>, String> {
    scrcpy::get_installed_apps()
}


/// 关闭所有 scrcpy
#[tauri::command]
pub fn kill_scrcpy() -> CommandResult {
    scrcpy::kill_scrcpy_processes()
}

/// 扫描局域网设备
#[tauri::command]
pub async fn scan_tcp_devices() -> Vec<String> {
    scrcpy::scan_local_network().await
}

/// 启动屏幕镜像
#[tauri::command]
pub fn launch_mirror() -> CommandResult {
    scrcpy::launch_scrcpy(LaunchMode::Mirror)
}

/// 启动纯音频
#[tauri::command]
pub fn launch_audio() -> CommandResult {
    scrcpy::launch_scrcpy(LaunchMode::Audio)
}

/// 启动应用
#[tauri::command]
pub fn launch_app(package: String, settings: Option<AppSettings>, scrcpy_args: Option<String>) -> CommandResult {
    scrcpy::launch_scrcpy(LaunchMode::App { package, settings, scrcpy_args })
}

/// 添加 ADB 历史记录
#[tauri::command]
pub fn add_adb_history(ip: String) {
    let mut config = Config::load();
    config.add_adb_history(&ip);
}

/// 保存应用配置
#[tauri::command]
pub fn save_app(package: String, app_config: AppConfig) {
    let mut config = Config::load();
    config.apps.insert(package, app_config);
    config.save();
}

/// 删除应用
#[tauri::command]
pub fn delete_app(package: String) {
    let mut config = Config::load();
    config.apps.shift_remove(&package);
    config.save();
}

/// 重新排序应用
#[tauri::command]
pub fn reorder_apps(new_order: Vec<String>) {
    let mut config = Config::load();
    config.reorder_apps(new_order);
}

/// 保存预设
#[tauri::command]
pub fn save_preset(name: String, long: u32, short: u32, dpi: u32, is_landscape: bool) {
    let mut config = Config::load();
    config.add_preset(&name, long, short, dpi, is_landscape);
}

/// 删除预设
#[tauri::command]
pub fn delete_preset(name: String) {
    let mut config = Config::load();
    config.delete_preset(&name);
}

/// 更新预设
#[tauri::command]
pub fn update_preset(old_name: String, new_name: String, long: u32, short: u32, dpi: u32, is_landscape: bool) {
    let mut config = Config::load();
    config.update_preset(&old_name, &new_name, long, short, dpi, is_landscape);
}

/// 设置首次运行完成
#[tauri::command]
pub fn set_first_run_complete() {
    let mut config = Config::load();
    config.set_first_run_complete();
}

/// 保存全局设置
#[tauri::command]
pub fn save_global_settings(
    dpi: u32, 
    full_res: String, 
    show_app_icons: bool,
    filter_installed_apps: bool
) {
    let mut config = Config::load();
    config.global_settings.dpi = dpi;
    config.global_settings.full_res = full_res;
    config.global_settings.show_app_icons = show_app_icons;
    config.global_settings.filter_installed_apps = filter_installed_apps;
    config.save();
}

/// 保存应用图标
#[tauri::command]
pub fn save_app_icon(package: String, source_path: String) -> Result<String, String> {
    // 验证源文件存在
    let src = std::path::Path::new(&source_path);
    if !src.exists() {
        return Err("源文件不存在".to_string());
    }

    // 目标目录: dist/custom_icons/
    let mut dest = std::env::current_dir().map_err(|e| e.to_string())?;
    dest.push("dist");
    dest.push("custom_icons");

    // 确保目录存在
    if !dest.exists() {
        std::fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    }

    // 目标文件名: {package}.png (简单起见统一转为 png 后缀，实际可能是 ico/jpg)
    // 为了更好的兼容性，我们保留原扩展名或者统一转存？
    // 为了前端方便加载，我们尽量保持原扩展名，或者前端尝试加载多种。
    // 这里简单处理：直接拷贝，文件名设为 package.png (假设用户选的是图片)
    // 如果用户选的是 ico，强行命名 png 也可以显示，但最好是保留扩展名。
    // 既然前端逻辑可以固定 `<img src="...png">`，那我们也可以强制用户选 png，
    // 或者不管扩展名，统一存为 png 文件 (内容其实是 jpg/ico 也不影响浏览器显示，大部分现代浏览器靠内容嗅探)
    // 让我们稍微严谨一点：获取源扩展名，如果是支持的图片格式，就拷贝。
    // 但为了前端简单 `src="${pkg}.png"`，我们直接把目标文件命名为 `.png` 结尾。
    
    dest.push(format!("{}.png", package));

    std::fs::copy(src, &dest).map_err(|e| format!("复制文件失败: {}", e))?;

    Ok(format!("图标已保存: {}", package))
}

/// 删除自定义图标
#[tauri::command]
pub fn delete_custom_icon(package: String) -> Result<String, String> {
    // 目标目录: dist/custom_icons/
    let mut dest = std::env::current_dir().map_err(|e| e.to_string())?;
    dest.push("dist");
    dest.push("custom_icons");
    dest.push(format!("{}.png", package));

    if dest.exists() {
        std::fs::remove_file(dest).map_err(|e| format!("删除图标失败: {}", e))?;
        Ok("恢复默认图标成功".to_string())
    } else {
        Ok("未找到自定义图标，无需恢复".to_string())
    }
}

/// 设置主题
#[tauri::command]
pub fn set_theme(theme: String) {
    let mut config = Config::load();
    config.global_settings.theme = theme;
    config.save();
}

/// 保存 scrcpy 选项
#[tauri::command]
pub fn save_scrcpy_options(
    use_custom: bool,
    custom_args: String,
    options: ScrcpyOptions,
    use_app_stream_args: bool,
    use_app_custom_args: bool,
    app_custom_args: String,
    app_options: ScrcpyOptions,
) {
    let mut config = Config::load();
    config.use_custom_args = use_custom;
    config.custom_args = custom_args;
    config.scrcpy_options = options;
    config.use_app_stream_args = use_app_stream_args;
    config.use_app_custom_args = use_app_custom_args;
    config.app_custom_args = app_custom_args;
    config.app_stream_options = app_options;
    config.save();
}

/// 保存托盘设置
#[tauri::command]
pub fn save_tray_settings(app_count: u32, show_mirror: bool, show_audio: bool) {
    let mut config = Config::load();
    config.tray_app_count = app_count;
    config.tray_show_mirror = show_mirror;
    config.tray_show_audio = show_audio;
    config.save();
}

/// 保存首次运行配置
#[tauri::command]
pub fn save_first_run_config(
    phone_long: u32,
    phone_short: u32,
    phone_dpi: u32,
    phone_landscape: bool,
    pc_long: u32,
    pc_short: u32,
    pc_dpi: u32,
    pc_landscape: bool,
) {
    let mut config = Config::load();
    
    // 添加预设
    config.presets.insert(
        format!("我的手机 ({}x{})", phone_long, phone_short),
        crate::config::PresetConfig {
            long: phone_long,
            short: phone_short,
            dpi: phone_dpi,
            is_landscape: phone_landscape,
        },
    );
    config.presets.insert(
        format!("我的电脑 ({}x{})", pc_long, pc_short),
        crate::config::PresetConfig {
            long: pc_long,
            short: pc_short,
            dpi: pc_dpi,
            is_landscape: pc_landscape,
        },
    );
    
    // 设置全局分辨率
    let min_dim = phone_long.min(phone_short);
    let max_dim = phone_long.max(phone_short);
    config.global_settings.full_res = format!("{}x{}", min_dim, max_dim);
    config.global_settings.use_full_res_switch = true;
    config.first_run = false;
    
    config.save();
}

/// ADB 推送文件
#[tauri::command]
pub async fn adb_push_file(window: tauri::Window, local_path: String, remote_path: Option<String>) -> Result<CommandResult, String> {
    let target = remote_path.unwrap_or_else(|| "/sdcard/Download/".to_string());
    
    // 处理目标路径：如果是目录（以 / 结尾），则追加文件名
    let target_file = if target.ends_with('/') {
        let filename = std::path::Path::new(&local_path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("unknown_file");
        // ADB SYNC 协议通常需要 "路径,权限" 格式，或者至少完整路径
        // 我们尝试发送 "路径,0644" 以确保权限正确 (标准 adb push 行为)
        format!("{}{},0644", target, filename)
    } else {
        format!("{},0644", target)
    };

    // 使用原生 TCP 连接进行传输
    let pusher = AdbPusher::new(None); 
    
    let window_clone = window.clone();
    
    pusher.push(&local_path, &target_file, Some(Box::new(move |current, total| {
        let percent = if total > 0 {
            (current as f64 / total as f64) * 100.0
        } else { 
            0.0 
        };
        
        let _ = window_clone.emit("adb-push-progress", serde_json::json!({
            "progress": percent as u32,
            "message": format!("传输中: {:.1}%", percent)
        }));
    }))).await.map_err(|e| e.to_string())?;
    
    // 发送 100% 进度
    let _ = window.emit("adb-push-progress", serde_json::json!({
        "progress": 100,
        "message": "传输完成"
    }));

    Ok(CommandResult {
        success: true,
        message: format!("发送成功: {}", local_path),
    })
}






