//! Scrcpy Launcher - 配置管理模块
//! 
//! 管理应用程序的所有配置，包括：
//! - ADB 连接历史
//! - 全局显示设置
//! - 应用列表及其自定义配置
//! - Scrcpy 启动选项

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use indexmap::IndexMap;
use std::fs;
use std::path::PathBuf;

/// 配置文件名
const CONFIG_FILE: &str = "config.json";

/// 全局显示设置
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GlobalSettings {
    pub use_full_res_switch: bool,
    pub full_res: String,
    pub dpi: u32,
    pub is_landscape: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_theme() -> String {
    "light".to_string()
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            use_full_res_switch: true,
            full_res: "1200x2670".to_string(),
            dpi: 400,
            is_landscape: false,
            theme: default_theme(),
        }
    }
}

/// 应用的自定义设置
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    pub use_full_res_switch: bool,
    pub full_res: String,
    pub dpi: u32,
    pub is_landscape: bool,
}

/// 单个应用的配置
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub name: String,
    pub settings: Option<AppSettings>,
    pub scrcpy_args: Option<String>,
}

/// 分辨率预设
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PresetConfig {
    pub long: u32,
    pub short: u32,
    pub dpi: u32,
    pub is_landscape: bool,
}

/// Scrcpy 启动选项
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScrcpyOptions {
    pub hid_keyboard: bool,
    pub stay_awake: bool,
    pub turn_screen_off: bool,
    pub show_touches: bool,
    pub always_on_top: bool,
    pub borderless: bool,
    pub power_off_on_close: bool,
    pub local_ime: bool,
    pub max_size: u32,
    pub max_fps: u32,
}

impl Default for ScrcpyOptions {
    fn default() -> Self {
        Self {
            hid_keyboard: true,
            stay_awake: true,
            turn_screen_off: false,
            show_touches: false,
            always_on_top: false,
            borderless: false,
            power_off_on_close: false,
            local_ime: false,
            max_size: 0,
            max_fps: 0,
        }
    }
}

/// 主配置结构
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub adb_history: Vec<String>,
    pub global_settings: GlobalSettings,
    pub apps: IndexMap<String, AppConfig>,
    pub presets: HashMap<String, PresetConfig>,
    pub first_run: bool,
    pub use_custom_args: bool,
    pub custom_args: String,
    pub scrcpy_options: ScrcpyOptions,
    pub use_app_stream_args: bool,
    pub use_app_custom_args: bool,
    pub app_custom_args: String,
    pub app_stream_options: ScrcpyOptions,
    pub tray_app_count: u32,
    pub tray_show_mirror: bool,
    pub tray_show_audio: bool,
}

impl Default for Config {
    fn default() -> Self {
        let mut presets = HashMap::new();
        presets.insert(
            "我的手机 (2670x1200)".to_string(),
            PresetConfig {
                long: 2670,
                short: 1200,
                dpi: 400,
                is_landscape: false,
            },
        );
        presets.insert(
            "我的电脑 (2256x1504)".to_string(),
            PresetConfig {
                long: 2256,
                short: 1504,
                dpi: 260,
                is_landscape: true,
            },
        );

        let mut app_stream_options = ScrcpyOptions::default();
        app_stream_options.local_ime = true;

        Self {
            adb_history: Vec::new(),
            global_settings: GlobalSettings::default(),
            apps: IndexMap::new(),
            presets,
            first_run: true,
            use_custom_args: false,
            custom_args: "-K --stay-awake".to_string(),
            scrcpy_options: ScrcpyOptions::default(),
            use_app_stream_args: true,
            use_app_custom_args: false,
            app_custom_args: "-K --stay-awake --display-ime-policy=local".to_string(),
            app_stream_options,
            tray_app_count: 4,
            tray_show_mirror: true,
            tray_show_audio: true,
        }
    }
}

impl Config {
    /// 获取配置文件路径
    fn get_config_path() -> PathBuf {
        // 使用当前工作目录
        PathBuf::from(CONFIG_FILE)
    }

    /// 加载配置，如果不存在则创建默认配置
    pub fn load() -> Self {
        let path = Self::get_config_path();
        
        if !path.exists() {
            let config = Config::default();
            config.save();
            return config;
        }

        match fs::read_to_string(&path) {
            Ok(content) => {
                serde_json::from_str(&content).unwrap_or_else(|_| Config::default())
            }
            Err(_) => Config::default(),
        }
    }

    /// 保存配置到文件
    pub fn save(&self) {
        let path = Self::get_config_path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, content);
        }
    }

    /// 添加 ADB 连接历史
    pub fn add_adb_history(&mut self, ip: &str) {
        // 如果已存在，先移除
        self.adb_history.retain(|h| h != ip);
        // 插入到开头
        self.adb_history.insert(0, ip.to_string());
        // 只保留最近10条
        self.adb_history.truncate(10);
        self.save();
    }

    /// 标记首次运行完成
    pub fn set_first_run_complete(&mut self) {
        self.first_run = false;
        self.save();
    }

    /// 添加预设
    pub fn add_preset(&mut self, name: &str, long: u32, short: u32, dpi: u32, is_landscape: bool) {
        self.presets.insert(
            name.to_string(),
            PresetConfig {
                long,
                short,
                dpi,
                is_landscape,
            },
        );
        self.save();
    }

    /// 更新预设
    pub fn update_preset(
        &mut self,
        old_name: &str,
        new_name: &str,
        long: u32,
        short: u32,
        dpi: u32,
        is_landscape: bool,
    ) {
        self.presets.remove(old_name);
        self.add_preset(new_name, long, short, dpi, is_landscape);
    }

    /// 删除预设
    pub fn delete_preset(&mut self, name: &str) {
        self.presets.remove(name);
        self.save();
    }

    /// 重新排序应用
    pub fn reorder_apps(&mut self, new_order: Vec<String>) {
        let mut new_apps = IndexMap::new();
        for pkg in new_order {
            if let Some(app) = self.apps.shift_remove(&pkg) {
                new_apps.insert(pkg, app);
            }
        }
        // 添加任何剩余的应用
        for (pkg, app) in self.apps.drain(..) {
            new_apps.insert(pkg, app);
        }
        self.apps = new_apps;
        self.save();
    }
}
