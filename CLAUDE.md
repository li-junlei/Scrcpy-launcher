# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个基于 **Tauri 2.0** 的 Scrcpy 启动器，为 Android 设备的屏幕镜像和控制提供图形化界面。使用 Rust 后端和原生 Web 前端（无框架）构建。

## 构建与开发命令

### 开发模式
```bash
cargo tauri dev
```
启动开发服务器，前端运行在 `http://localhost:5173`（可在 `tauri.conf.json` 中配置）

### 生产构建
```bash
cargo tauri build
```
构建产物位于：
- `target/release/bundle/nsis/` - NSIS 安装程序
- `target/release/bundle/msi/` - MSI 安装程序

### 仅构建 Rust 代码
```bash
cargo build --release
```

## 代码架构

### 后端架构 (`src/`)

**模块职责划分：**

- **`main.rs`** - 应用入口
  - 注册 Tauri 插件：`shell`, `dialog`, `fs`, `single-instance`
  - 配置系统托盘
  - 窗口事件处理（关闭时最小化到托盘而非退出）
  - 注册所有 Tauri 命令处理程序

- **`commands.rs`** - IPC 命令层
  - 定义所有可从前端调用的 Tauri 命令
  - 轻量级包装，调用其他模块的业务逻辑
  - 处理配置序列化/反序列化

- **`config.rs`** - 配置管理
  - 定义所有配置数据结构（使用 serde 序列化）
  - 配置文件读写（`config.json` 位于可执行文件目录）
  - 默认值管理
  - 使用 `IndexMap` 保持应用配置的插入顺序

- **`scrcpy.rs`** - 核心业务逻辑
  - ADB 设备状态检查和连接管理
  - 构建复杂的 scrcpy 命令行参数
  - 启动和管理 scrcpy 进程
  - 路径解析（开发模式使用 `./resources`，生产模式使用可执行文件目录）

- **`tray.rs`** - 系统托盘
  - 动态生成托盘菜单（包括应用快捷方式）
  - 处理托盘事件（左键显示窗口、菜单项操作）

### 前端架构 (`dist/`)

**单页应用设计：**
- **`index.html`** - 完整的 HTML 结构（使用模态对话框而非路由）
- **`main.js`** - 核心 JavaScript 逻辑
  - 通过 `window.__TAURI__.core.invoke` 与 Rust 后端通信
  - 全局状态管理（`config`, `isSortingMode`, `editingAppPackage` 等）
  - 事件监听器设置在 `DOMContentLoaded` 时统一注册
- **`styles.css`** - 主样式表（包含 CSS 变量的深色模式支持）
- **`overrides.css`** - 样式覆盖（特定组件的自定义样式）

**UI 模式：**
- 使用模态对话框而非多页面导航
- 标签页切换（直接投屏 vs 应用流转）
- 内联消息提示（Snackbar）

### 资源管理

**开发 vs 生产环境路径：**
- 开发模式：`./resources/bin/`
- 生产模式：`可执行文件目录/resources/bin/`

**包含的资源：**
- `adb.exe` - Android Debug Bridge
- `scrcpy.exe` - Scrcpy 可执行文件
- 相关 DLL 文件
- 字体文件（`resources/fonts/`）

## 核心数据流

### 配置管理流程
```
用户操作 → 前端 invoke() → commands.rs 函数
  → config.rs 修改内存 → config.save() 写入 config.json
```

### Scrcpy 启动流程
```
前端触发 → launch_* 命令 → scrcpy.rs:launch_scrcpy()
  → 1. 加载配置
  → 2. 根据模式构建参数（LaunchMode 枚举）
  → 3. 从 ScrcpyOptions 或自定义字符串生成参数列表
  → 4. Command::spawn() 启动独立进程
```

### 启动模式（LaunchMode 枚举）
- **Mirror** - 屏幕镜像（使用 `global_settings.full_res`）
- **Audio** - 纯音频（添加 `--no-video` 参数）
- **App** - 应用流转（使用 `--new-display` 和 `--start-app`）

## 重要技术细节

### Windows 特定处理
- 使用 `CREATE_NO_WINDOW` 标志隐藏控制台窗口
- 单实例锁定（`tauri-plugin-single-instance`）
- 进程终止使用 `taskkill /F /IM scrcpy.exe`

### 序列化
- 所有配置结构使用 `#[derive(Serialize, Deserialize)]`
- 使用 `serde_json` 进行 JSON 序列化
- 前端自动处理 JavaScript 对象与 Rust 结构的转换

### 应用配置优先级
每个应用可以有三层配置覆盖：
1. 应用专属自定义参数（`scrcpy_args`）
2. 应用流转全局选项（`app_stream_options`）
3. 镜像全局选项（`scrcpy_options`）

### 窗口行为
- 默认大小：460x700（在 `tauri.conf.json` 中配置）
- 关闭按钮：最小化到托盘（`main.rs:31-34`）
- 托盘左键：恢复并聚焦窗口

## 开发注意事项

### 修改前端
- 编辑 `dist/*.html`, `dist/*.js`, `dist/*.css`
- 前端更改会自动热重载（开发模式下）
- Tauri API 调用使用：`invoke('command_name', { param: value })`

### 修改后端
- 编辑 `src/*.rs` 文件
- 添加新命令需要在 `main.rs` 的 `invoke_handler!` 宏中注册
- 所有返回给前端的结构必须实现 `Serialize`
- 所有从前端接收的结构必须实现 `Deserialize`

### 添加新配置项
1. 在 `config.rs` 中添加字段到相应结构体
2. 在 `Default` 实现中设置默认值
3. 在 `commands.rs` 添加保存/加载命令
4. 在前端添加 UI 和调用逻辑

### 路径处理
- 始终使用 `get_resources_path()` 函数获取资源目录
- 不要硬编码路径（考虑开发和生产环境的差异）
- 配置文件始终使用可执行文件所在目录

## Tauri 2.0 特性

- 权限系统：`capabilities/default.json` 定义前端可调用的命令
- 全局 Tauri 对象：`withGlobalTauri: true` 允许使用 `window.__TAURI__`
- 插件系统：shell, dialog, fs 等功能通过插件提供

## 常见任务

### 添加新的 Tauri 命令
1. 在 `commands.rs` 定义函数并添加 `#[tauri::command]` 宏
2. 在 `main.rs` 的 `invoke_handler!` 宏中添加命令名
3. 在 `capabilities/default.json` 中添加权限（如果需要）

### 调试技巧
- 后端：使用 `println!` 或 `eprintln!` 输出到控制台
- 前端：使用浏览器开发者工具（开发模式下）
- 检查 `config.json` 文件以验证配置保存

### 代码风格
- Rust 代码遵循标准 Rust 命名规范
- 中文注释和用户界面文本
- 前端使用 ES6+ 语法
- CSS 使用 BEM 风格类名
