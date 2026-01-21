//! Scrcpy Launcher - 系统托盘模块

use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{Menu, MenuItem, PredefinedMenuItem},
    Manager, Runtime,
};
use crate::config::Config;

/// 设置系统托盘
pub fn setup_tray<R: Runtime>(app: &tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load();
    
    // 创建菜单项
    let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "隐藏到托盘", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    
    // 创建快捷操作
    let mirror_item = MenuItem::with_id(app, "mirror", "屏幕镜像", true, None::<&str>)?;
    let audio_item = MenuItem::with_id(app, "audio", "纯音频", true, None::<&str>)?;
    
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    
    // 构建菜单
    let mut menu_items: Vec<&dyn tauri::menu::IsMenuItem<R>> = Vec::new();
    
    // 添加应用快捷方式
    let apps = config.apps.clone();
    let app_count = config.tray_app_count as usize;
    
    // 注意: 由于 Rust 的所有权规则，我们需要在这里创建应用菜单项
    let app_menu_items: Vec<MenuItem<R>> = apps
        .iter()
        .take(app_count)
        .filter_map(|(pkg, app_config)| {
            MenuItem::with_id(app, &format!("app:{}", pkg), &app_config.name, true, None::<&str>).ok()
        })
        .collect();
    
    for item in &app_menu_items {
        menu_items.push(item);
    }
    
    if !app_menu_items.is_empty() {
        menu_items.push(&separator);
    }
    
    // 添加镜像和音频选项
    if config.tray_show_mirror {
        menu_items.push(&mirror_item);
    }
    if config.tray_show_audio {
        menu_items.push(&audio_item);
    }
    if config.tray_show_mirror || config.tray_show_audio {
        menu_items.push(&separator);
    }
    
    menu_items.push(&show_item);
    menu_items.push(&hide_item);
    menu_items.push(&separator);
    menu_items.push(&quit_item);
    
    let menu = Menu::with_items(app, &menu_items)?;
    
    // 创建托盘图标
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Scrcpy Launcher")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            let id = event.id.as_ref();
            
            if id == "show" {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            } else if id == "hide" {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            } else if id == "mirror" {
                let _ = crate::scrcpy::launch_scrcpy(crate::scrcpy::LaunchMode::Mirror);
            } else if id == "audio" {
                let _ = crate::scrcpy::launch_scrcpy(crate::scrcpy::LaunchMode::Audio);
            } else if id == "quit" {
                crate::scrcpy::cleanup_before_exit();
                app.exit(0);
            } else if id.starts_with("app:") {
                let package = id.strip_prefix("app:").unwrap_or("");
                let config = Config::load();
                if let Some(app_config) = config.apps.get(package) {
                    let _ = crate::scrcpy::launch_scrcpy(crate::scrcpy::LaunchMode::App {
                        package: package.to_string(),
                        settings: app_config.settings.clone(),
                        scrcpy_args: app_config.scrcpy_args.clone(),
                    });
                }
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                // 左键点击显示窗口
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
