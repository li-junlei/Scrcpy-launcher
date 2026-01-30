# Scrcpy Launcher (Rust)

<div align="center">

**一个基于 Tauri 2.0 的 Scrcpy 图形化启动器**

支持 Android 设备屏幕镜像和应用流转

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-stable-orange)](https://www.rust-lang.org/)

</div>

## v2.6.0 更新日志
- **应用宝搜索**：
  - 将"从手机浏览应用"替换为"从应用宝搜索"功能
  - 输入应用名称即可搜索获取包名和图标
  - 选择应用后自动下载图标到本地
- **自动更新检查**：
  - 启动时自动检查 GitHub Release 新版本
  - 设置页面新增"关于"区域，显示当前版本号
  - 发现新版本时弹窗提示，支持"此版本不再提醒"

## 功能特性

### 核心功能
- **ADB 无线连接管理** - 支持保存多个设备连接地址，一键连接
- **局域网扫描** - 自动扫描局域网内开放 5555 端口的设备，即使开启 VPN/TUN 模式也能精准发现
- **无线连接优化** - 异步连接处理，增加 TCP 快速检测，连接无效 IP 时不再卡顿无响应
- **ADB 无线配对** - 支持 Android 11+ 原生无线调试配对 (扫码或配对码)
- **屏幕镜像** - 全屏镜像 Android 设备，支持自定义分辨率
- **纯音频传输** - 仅传输音频，不显示视频画面
- **应用流转** - 在独立窗口中运行 Android 应用，支持自定义分辨率和 DPI
- **智能自动填充** - 添加应用时支持根据名称自动匹配包名
- **系统托盘集成** - 最小化到托盘，快速启动常用应用
- **文件传输** - 支持将文件直接拖拽至窗口发送到设备，实时显示传输进度

### 高级特性
- **完整的配置管理** - 为每个应用配置独立的分辨率、DPI 和 Scrcpy 参数
- **深色模式支持** - 护眼的深色主题
- **应用自定义配置** - 支持应用专属的自定义 Scrcpy 参数
- **分辨率预设** - 保存和管理常用的分辨率配置
- **Scrcpy 选项配置** - 可视化配置各种 Scrcpy 启动参数
- **原生 ADB 协议** - 内置 Rust 实现的 ADB 客户端，提供更稳定精准的文件传输进度反馈

---

## 安装


### 从 Release 安装

1. **下载安装包**
   - 访问 [Releases 页面](https://github.com/li-junlei/Scrcpy-launcher/releases/)
   - 下载最新版本的安装包（`.exe` 文件）

2. **运行安装程序**
   - 双击下载的 `.exe` 文件
   - 按照安装向导完成安装
   - 默认安装路径：`C:\Users\你的用户名\AppData\Local\Programs\scrcpy-launcher\`

3. **启动应用**
   - 安装完成后会在桌面创建快捷方式
   - 或从开始菜单找到 "Scrcpy Launcher" 启动


## 使用教程

### 1. 连接 Android 设备

Scrcpy Launcher 支持多种连接方式，请根据你的情况选择：

#### 方式一：USB 有线连接（推荐首次使用）
最简单、最稳定的方式。
1. **开启 USB 调试**：在手机 `设置` -> `开发者选项` 中开启 "USB 调试"。
2. **连接电脑**：使用数据线连接手机和电脑。
   - 手机上弹出 "允许 USB 调试吗？" 时，勾选 "始终允许" 并点击 "允许"。
3. **完成**：软件顶部状态栏显示 "已连接"，即可开始使用。

#### 方式二：Wi-Fi 无线连接

**情况 A：常规无线连接 (推荐)**
适用于大多数设备。此方式会将 ADB 端口固定为 `5555`。

1. **开启端口**：
   - 先使用 USB 连接手机。
   - 点击主界面右上角的 **"有线转无线"** 按钮。
   - 等待提示开启成功后，拔掉数据线。

2. **连接设备**：
   - **知道 IP**：直接在 "ADB IP 地址" 栏输入手机 IP，点击 "连接"。
   - **不知道 IP**：点击输入框右侧的 **"扫描"** 图标（雷达），软件会自动发现局域网内已开启 5555 端口的设备，点击即可连接。

**情况 B：Android 11+ 无线配对**
完全无需数据线，但端口是随机的（不支持自动扫描）。

1. 手机进入 `开发者选项` -> `无线调试` -> `使用配对码配对设备`。
2. 软件点击 **"无线配对"**，输入手机显示的 IP:端口 和 配对码。
3. 配对成功后，使用 *无线调试主界面* 显示的 IP:端口（注意端口通常不是 5555）进行连接。

### 2. 屏幕镜像 & 音频传输
连接成功后：
- **启动镜像**：点击 **"启动屏幕镜像"**，投屏手机画面。支持键鼠控制。
- **纯音频**：点击 **"启动音频"**，仅传输手机声音到电脑，不显示画面（适合听歌/听书）。

**常用快捷键**：
- `Alt + F`：全屏模式
- `Alt + H`：回到桌面 (Home)
- `Alt + B`：返回 (Back)
- `Alt + S`：切换应用 (App Switch)
- `Alt + ↑/↓`：调节音量

### 3. 应用流转 (App Streaming)
让 Android 应用像电脑原生软件一样在独立窗口运行。

1. **添加应用**：
   - 切换到 "应用流转"主要标签页。
   - 点击 **"添加新应用"**。
   - **智能自动填充**：在 "应用名称" 输入 "微信"、"抖音" 等常见应用名，会自动检测并填充包名。
   - 或者点击 "浏览" 图标，从已安装应用列表中选择。
2. **个性化配置**：
   - 想要手机版界面？设置 **DPI** 为 400 左右，**长边** 2400。
   - 想要平板/电脑版界面？设置 **DPI** 为 160 (或更低)，**不可勾选** "强制横屏"。
3. **一键启动**：点击应用卡片上的启动按钮。

### 4. 文件传输
- **拖拽发送**：直接将电脑文件拖拽到 Scrcpy 投屏窗口中。
  - APK 文件：自动安装。
  - 其他文件：默认保存到 `/sdcard/Download/` 目录。
- **进度查看**：主界面右下角会实时显示文件传输进度条。

### 5. 分辨率预设
在 "全局设置" -> "分辨率预设" 中，你可以保存常用的窗口尺寸配置（例如 "我的手机"、"4K 大屏模式"），方便随时切换。

### 6. 系统托盘
软件支持最小化到系统托盘（右下角小图标）。
- **右键菜单**：快速启动最近使用的 5 个应用，或启动镜像/音频，无需打开主界面。
- **静默运行**：点击窗口关闭按钮可选择最小化到托盘，保持后台待机。

---

## 配置文件说明

配置文件 `config.json` 位于可执行文件同目录，包含以下配置：

```json
{
  "adb_history": ["192.168.1.100:5555"],  // ADB 连接历史
  "global_settings": {
    "use_full_res_switch": true,          // 是否使用分辨率切换开关
    "full_res": "1200x2670",               // 全局分辨率
    "dpi": 400,                            // 全局 DPI
    "is_landscape": false,                 // 是否横屏
    "theme": "light"                       // 主题：light / dark
  },
  "apps": {                                // 应用列表
    "com.example.app": {
      "name": "示例应用",
      "settings": {
        "full_res": "1920x1080",
        "dpi": 320
      },
      "scrcpy_args": "--no-keyboard"      // 应用自定义参数
    }
  },
  "presets": {                             // 分辨率预设
    "我的手机": {
      "long": 2670,
      "short": 1200,
      "dpi": 400,
      "is_landscape": false
    }
  },
  "scrcpy_options": {                     // 镜像选项
    "hid_keyboard": true,
    "stay_awake": true,
    "turn_screen_off": false,
    "show_touches": false,
    "always_on_top": false,
    "borderless": false,
    "power_off_on_close": false,
    "local_ime": false,
    "max_size": 0,
    "max_fps": 0
  }
}
```

**注意**：
- `config.json` 包含个人设置，已加入 `.gitignore`
- 修改配置后会在下次启动时生效
- 建议通过界面修改配置，手动编辑可能导致错误

---

## 技术栈

- **后端**: Rust + Tauri 2.0
- **前端**: Vanilla JavaScript (无框架)
- **构建**: Cargo + Tauri CLI
- **核心依赖**:
  - [Genymobile/scrcpy](https://github.com/Genymobile/scrcpy) - Android 镜像引擎
  - [tauri-apps/tauri](https://github.com/tauri-apps/tauri) - 应用框架

---

## 项目结构

```
├── src/              # Rust 后端源码
│   ├── main.rs       # 应用入口、窗口事件、托盘
│   ├── commands.rs   # Tauri 命令（IPC 接口）
│   ├── config.rs     # 配置管理
│   ├── scrcpy.rs     # Scrcpy 核心逻辑（进程管理）
│   └── tray.rs       # 系统托盘实现
├── dist/             # 前端源码 (HTML/JS/CSS)
│   ├── index.html    # 主界面
│   ├── main.js       # 核心逻辑
│   ├── styles.css    # 主样式表
│   └── overrides.css # 样式覆盖
├── resources/        # 资源文件
│   ├── bin/          # ADB 和 Scrcpy 可执行文件
│   └── fonts/        # 字体文件
├── tauri.conf.json   # Tauri 配置
├── CLAUDE.md         # AI 辅助开发指南
└── README.md         # 本文件
```

---

## 常见问题

### Q: 连接失败怎么办？
**A**: 检查以下几点：
1. 确认设备和电脑在同一网络
2. 检查 IP 地址和端口是否正确
3. 确保设备的无线调试已启用
4. 尝试重新启用无线调试

### Q: 镜像画面卡顿？
**A**: 尝试以下优化：
1. 降低 `最大帧率`（设置为 30 或 60）
2. 限制 `最大尺寸`（设置为 1920 或更低）
3. 使用有线网络而非 Wi-Fi
4. 关闭其他占用网络的程序

### Q: 如何获取应用包名？
**A**: 使用 ADB 命令：
```bash
# 列出所有第三方应用
adb shell pm list packages -3

# 搜索特定应用
adb shell pm list packages | grep chrome
```

### Q: 应用流转和屏幕镜像有什么区别？
**A**:
- **屏幕镜像**：镜像整个设备屏幕，适合远程控制
- **应用流转**：在独立窗口中运行单个应用，适合多任务处理

### Q: 如何卸载？
**A**:
- Windows: 从 `设置` → `应用` → 找到 "Scrcpy Launcher" → `卸载`
- 或运行安装包，选择 `卸载`

### Q: 配置文件在哪里？
**A**:
配置文件 `config.json` 位于可执行文件同目录，默认路径：
```
C:\Users\你的用户名\AppData\Local\Programs\scrcpy-launcher-rust\config.json
```

---

## 贡献

欢迎提交 Issue 和 Pull Request！

### 开发指南

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

---

## 许可证

本项目仅供学习和个人使用。

## 致谢

- [Genymobile/scrcpy](https://github.com/Genymobile/scrcpy) - 核心镜像引擎
- [tauri-apps/tauri](https://github.com/tauri-apps/tauri) - 应用框架
- 所有贡献者

---

<div align="center">

**如果觉得有用，请 Star 支持一下！**

Made by Li-Junlei

</div>
