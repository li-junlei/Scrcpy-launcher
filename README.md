# Scrcpy Launcher (Rust)

<div align="center">

**一个基于 Tauri 2.0 的 Scrcpy 图形化启动器**

支持 Android 设备屏幕镜像和应用流转

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-stable-orange)](https://www.rust-lang.org/)

</div>

## 功能特性

### 核心功能
- **ADB 无线连接管理** - 支持保存多个设备连接地址，一键连接
- **屏幕镜像** - 全屏镜像 Android 设备，支持自定义分辨率
- **纯音频传输** - 仅传输音频，不显示视频画面
- **应用流转** - 在独立窗口中运行 Android 应用，支持自定义分辨率和 DPI
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

### 1. 首次使用 - 连接 Android 设备

#### USB 连接（推荐）

**USB 连接是最简单、最稳定的连接方式，推荐首次使用时采用。**

1. **启用开发者选项**
   - 在 Android 设备上进入 `设置` → `关于手机`
   - 连续点击 `版本号` 7 次启用开发者选项

2. **启用 USB 调试**
   - 返回 `设置` → `系统` → `开发者选项`
   - 找到并启用 `USB 调试`

3. **连接设备**
   - 使用 USB 数据线将设备连接到电脑
   - 设备上会弹出 `允许 USB 调试吗？` 提示，勾选 `始终允许使用这台计算机`
   - 点击 `允许`
   - Scrcpy Launcher 会自动检测设备，状态栏显示 "已连接"

#### 无线连接（需先 USB 连接）

**无线连接需要先通过 USB 连接设备并开启 TCP/IP 模式。**

1. **完成上述 USB 连接步骤**
   - 确保设备已通过 USB 连接并成功被识别

2. **开启无线模式**
   - 在 Scrcpy Launcher 主界面点击 `有线转无线` 按钮
   - 等待提示 "成功开启无线模式，端口 5555"
   - 此时设备已开放 5555 端口

3. **获取设备 IP 地址**
   - 在 Android 设备上进入 `设置` → `WLAN`（或 `网络和互联网`）
   - 点击当前连接的 Wi-Fi 网络
   - 记录显示的 IP 地址（例如：`192.168.1.100`）

4. **拔掉 USB 数据线**

5. **无线连接**
   - 在 Scrcpy Launcher 的 "ADB IP 地址" 输入框中输入 IP 地址
   - 点击 `连接` 按钮
   - 等待连接成功提示

**提示**：首次使用无线连接后，IP 地址会被保存到历史记录，下次直接选择即可。

### 2. 屏幕镜像

1. 确保设备已连接（状态栏显示 "已连接"）
2. 点击 `启动镜像` 按钮
3. Scrcpy 窗口将打开，显示设备屏幕
4. 在窗口中可以使用鼠标和键盘控制设备

**配置选项：**
- 进入 `全局设置` → `镜像选项` 调整参数：
  - **HID 键盘** - 启用后键盘输入更流畅
  - **保持唤醒** - 防止设备自动锁屏
  - **关闭屏幕** - 镜像时关闭设备屏幕（省电）
  - **显示触摸** - 在屏幕上显示触摸点
  - **窗口置顶** - 窗口始终在最前
  - **无边框** - 移除窗口边框
  - **最大尺寸** - 限制镜像窗口大小
  - **最大帧率** - 限制传输帧率

### 3. 应用流转

应用流转功能可以在独立窗口中运行 Android 应用，就像原生 Windows 应用一样。

1. **添加应用**
   - 切换到 `应用流转` 标签
   - 点击 `添加应用` 按钮
   - 输入应用包名（例如：`com.android.chrome`）
   - 输入应用名称（例如：`Chrome 浏览器`）

2. **配置应用**
   - 点击应用卡片右侧的 `编辑` 按钮
   - 设置应用窗口的分辨率和 DPI
   - 可选：添加自定义 Scrcpy 参数

3. **启动应用**
   - 点击应用卡片上的 `启动` 按钮
   - 应用将在独立窗口中打开

**获取应用包名的方法：**
```bash
# 使用 ADB 命令列出所有已安装应用
adb shell pm list packages -3

# 或者使用：
adb shell pm list packages | grep 关键词
```

### 4. 分辨率预设管理

为了方便不同场景使用，可以保存多个分辨率预设：

1. 进入 `全局设置` → `分辨率预设`
2. 点击 `添加预设`：
   - **名称**：例如 "我的手机"、"4K 显示器" 等
   - **长边**：屏幕较长边的像素值
   - **短边**：屏幕较短边的像素值
   - **DPI**：每英寸点数（影响 UI 元素大小）
   - **横屏**：勾选后窗口默认为横向

3. 启动时从预设列表选择即可

### 5. 系统托盘使用

应用最小化到托盘后：

- **左键点击托盘图标**：恢复主窗口
- **右键点击托盘图标**：显示快捷菜单
  - 快速启动最近使用的应用（默认 4 个）
  - 启动镜像/音频
  - 退出应用

**配置托盘菜单：**
- 进入 `全局设置` → `托盘设置`
- 设置显示的应用数量
- 选择是否显示镜像和音频选项

### 6. 纯音频传输

如果只需要音频，不需要视频画面：

1. 确保设备已连接
2. 点击 `启动音频` 按钮
3. 后台传输音频，不显示视频窗口

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
