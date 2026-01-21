//! Scrcpy Launcher - 主程序入口
//!
//! 基于 Tauri 的 scrcpy 图形化启动器

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use scrcpy_launcher_lib::{commands, tray};
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 当尝试打开第二个实例时，显示主窗口
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
            // 处理窗口关闭事件 - 最小化到托盘而不是退出
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::check_adb_status,
            commands::connect_wireless,
            commands::enable_tcpip,
            commands::disconnect_all,
            commands::get_installed_apps,
            commands::kill_scrcpy,
            commands::launch_mirror,
            commands::launch_audio,
            commands::launch_app,
            commands::add_adb_history,
            commands::save_app,
            commands::delete_app,
            commands::reorder_apps,
            commands::save_preset,
            commands::delete_preset,
            commands::update_preset,
            commands::set_first_run_complete,
            commands::save_global_settings,
            commands::save_scrcpy_options,
            commands::save_tray_settings,
            commands::save_first_run_config,
            commands::set_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
