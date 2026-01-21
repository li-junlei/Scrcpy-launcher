# Scrcpy Launcher (Rust)

一个基于 Tauri 2.0 的 Scrcpy 图形化启动器，支持 Android 设备屏幕镜像和应用流转。

## 功能特性

- ✅ ADB 无线连接管理
- ✅ 屏幕镜像和纯音频传输
- ✅ 应用流转（自定义分辨率和 DPI）
- ✅ 系统托盘集成
- ✅ 深色模式支持
- ✅ 完整的配置管理系统

## 快速开始

### 开发环境要求

- Rust (stable)
- Node.js 16+ (用于 Tauri CLI)
- Visual Studio C++ Build Tools (Windows)

### 开发

```bash
# 安装依赖
cargo install tauri-cli --version "^2.0.0"

# 运行开发服务器
cargo tauri dev
```

### 构建

```bash
# 构建生产版本
cargo tauri build

# 构建产物位置
# - NSIS 安装程序: target/release/bundle/nsis/
# - MSI 安装程序: target/release/bundle/msi/
```

## 项目结构

```
├── src/              # Rust 后端源码
│   ├── main.rs       # 应用入口
│   ├── commands.rs   # Tauri 命令
│   ├── config.rs     # 配置管理
│   ├── scrcpy.rs     # Scrcpy 核心逻辑
│   └── tray.rs       # 系统托盘
├── dist/             # 前端源码 (HTML/JS/CSS)
├── resources/        # 资源文件
│   ├── bin/          # ADB 和 Scrcpy 可执行文件
│   └── fonts/        # 字体文件
├── tauri.conf.json   # Tauri 配置
└── CLAUDE.md         # AI 辅助开发指南
```

## 配置说明

应用程序的配置文件 `config.json` 位于可执行文件同目录，首次运行时会自动创建。

**注意**: `config.json` 包含用户个人设置，已加入 `.gitignore` 不会被提交到仓库。

## 技术栈

- **后端**: Rust + Tauri 2.0
- **前端**: Vanilla JavaScript (无框架)
- **构建**: Cargo + Tauri CLI

## 许可证

本项目仅供学习和个人使用。

## 致谢

- [Genymobile/scrcpy](https://github.com/Genymobile/scrcpy) - 核心镜像引擎
- [tauri-apps/tauri](https://github.com/tauri-apps/tauri) - 应用框架
