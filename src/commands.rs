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
#[tauri::command]
pub fn connect_wireless(ip: String) -> CommandResult {
    scrcpy::connect_wireless(&ip)
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
pub fn save_global_settings(dpi: u32, full_res: String) {
    let mut config = Config::load();
    config.global_settings.dpi = dpi;
    config.global_settings.full_res = full_res;
    config.save();
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






