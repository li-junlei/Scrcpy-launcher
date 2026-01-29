[根目录](../CLAUDE.md) > **dist**

# dist/ - 前端模块

> 最后更新：2026-01-27 00:02:27
> 语言：Vanilla JavaScript (ES6+), HTML5, CSS3
> 状态：✅ 完整扫描

---

## 变更记录

### 2026-01-27
- 初始化模块文档
- 完成所有前端文件扫描与分析

---

## 模块职责

**dist/** 是 Scrcpy Launcher 的前端界面模块，负责：

1. **用户界面渲染**：HTML 结构与 CSS 样式
2. **用户交互**：按钮、表单、拖拽等事件处理
3. **IPC 通信**：通过 Tauri API 调用后端 Rust 命令
4. **状态管理**：配置缓存、UI 状态同步
5. **事件监听**：文件传输进度、拖拽事件等

**技术特点**：
- **无框架依赖**：纯原生 JavaScript，轻量高效
- **响应式设计**：适配不同窗口尺寸
- **主题切换**：支持浅色/深色主题
- **实时更新**：监听后端事件，动态更新 UI

---

## 入口与启动

### index.html - 主界面

**文件路径**：`dist/index.html` (300+ 行)

**DOM 结构**：

```html
<div id="app">
    <!-- ADB 连接区域 -->
    <section class="card adb-section">
        <div class="status-row">
            <span class="status-dot" id="status-dot"></span>
            <span class="status-text" id="status-text">正在检查 ADB...</span>
        </div>

        <div class="input-row">
            <input type="text" id="ip-input" placeholder="输入无线地址 (IP:5555)">
            <button id="history-btn">历史记录</button>
            <button id="scan-btn">扫描局域网设备</button>
            <button id="connect-btn">连接</button>
        </div>

        <div class="button-row">
            <button id="pair-btn">无线配对</button>
            <button id="tcpip-btn">有线转无线</button>
            <button id="disconnect-btn">断开连接</button>
        </div>

        <button id="kill-btn">关闭所有 Scrcpy 窗口</button>
    </section>

    <!-- 工具箱区域 -->
    <section class="card">
        <h3>工具箱</h3>
        <button id="push-file-btn">传文件到手机</button>
        <div class="progress-container hidden" id="push-progress">
            <div class="progress-bar">
                <div class="progress-fill" id="progress-fill"></div>
            </div>
            <div class="progress-text" id="progress-text">0%</div>
        </div>
    </section>

    <!-- 标签页 -->
    <div class="tabs">
        <button class="tab active" data-tab="mirror">直接投屏</button>
        <button class="tab" data-tab="apps">应用流转</button>
    </div>

    <!-- 直接投屏标签页 -->
    <section class="tab-content active" id="tab-mirror">
        <button id="settings-btn">设置</button>
        <button id="advanced-btn">高级设置</button>
        <button id="mirror-btn">启动屏幕镜像</button>
        <button id="audio-btn">启动纯音频</button>
    </section>

    <!-- 应用流转标签页 -->
    <section class="tab-content" id="tab-apps">
        <button id="add-app-btn">添加新应用</button>
        <button id="sort-btn">排序</button>
        <div id="apps-grid" class="apps-grid"></div>
    </section>
</div>

<!-- 模态框 -->
<div id="settings-modal" class="modal">
    <!-- 设置内容 -->
</div>

<div id="app-config-modal" class="modal">
    <!-- 应用配置内容 -->
</div>

<div id="scan-results-modal" class="modal">
    <!-- 扫描结果内容 -->
</div>

<!-- 其他模态框... -->
```

**UI 组件**：
- **ADB 区域**：连接状态、IP 输入、历史记录、扫描按钮
- **工具箱**：文件传输按钮、进度条
- **直接投屏**：镜像/音频启动按钮、设置入口
- **应用流转**：应用卡片网格、添加/排序按钮
- **模态框**：设置、应用配置、预设管理、扫描结果等

---

## 对外接口

### Tauri API 调用

**文件路径**：`dist/main.js` (1250+ 行)

**核心 API**：

```javascript
// 导入 Tauri API
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;
const { open } = window.__TAURI__.dialog;

// 调用后端命令
const config = await invoke('get_config');
const result = await invoke('connect_wireless', { ip: '192.168.1.100' });
const apps = await invoke('get_installed_apps');
```

**事件监听**：

```javascript
// 监听文件传输进度
listen('adb-push-progress', (event) => {
    const { progress, message } = event.payload;
    updateProgressBar(progress, message);
});

// 监听文件拖拽
await listen('tauri://drag-enter', () => {
    $('drop-overlay').classList.remove('hidden');
});

await listen('tauri://drag-drop', async (event) => {
    const files = event.payload.paths;
    for (const file of files) {
        await pushFileFromPath(file);
    }
});
```

---

## 核心功能实现

### main.js - 前端逻辑

**文件路径**：`dist/main.js` (1250+ 行)

**全局状态**：

```javascript
let config = null;              // 配置缓存
let isSortingMode = false;      // 排序模式标记
let editingAppPackage = null;   // 正在编辑的应用
let editingPresetName = null;   // 正在编辑的预设
let deviceApps = [];            // 设备已安装应用列表
```

#### 1. 初始化流程

```javascript
document.addEventListener('DOMContentLoaded', async () => {
    try {
        await loadConfig();         // 加载配置
        setupEventListeners();      // 设置事件监听器
        initUI();                   // 初始化 UI
    } catch (e) {
        console.error('App Initialization Failed:', e);
        showMessage('初始化失败: ' + e);
    }
});

async function loadConfig() {
    config = await invoke('get_config');
    applyTheme(config.global_settings.theme);
}

function initUI() {
    if (config.first_run) {
        showModal('first-run-modal');  // 首次运行向导
    } else {
        checkAdbStatus();              // 检查 ADB 状态
        renderApps();                  // 渲染应用列表
        loadHistoryDropdown();         // 加载历史记录
    }
}
```

#### 2. ADB 连接管理

```javascript
async function checkAdbStatus() {
    const status = await invoke('check_adb_status');
    $('status-dot').classList.toggle('connected', status.connected);
    $('status-text').textContent = status.message;
}

async function connectWireless() {
    const ip = $('ip-input').value.trim();
    if (!ip) {
        showMessage('请输入IP地址');
        return;
    }

    setLoading('connect-btn', true);
    $('status-text').textContent = `正在连接 ${ip}...`;

    try {
        const result = await invoke('connect_wireless', { ip });
        if (result.success) {
            showMessage('无线连接成功！');
            await loadConfig();
            loadHistoryDropdown();
        } else {
            showMessage(`连接失败: ${result.message}`);
        }
    } catch (e) {
        showMessage(`连接失败: ${e}`);
    } finally {
        setLoading('connect-btn', false);
        checkAdbStatus();
    }
}

async function enableTcpip() {
    setLoading('tcpip-btn', true);
    showMessage('正在开启无线模式...');

    try {
        const result = await invoke('enable_tcpip');
        showMessage(result.message);
    } catch (e) {
        showMessage(`失败: ${e}`);
    } finally {
        setLoading('tcpip-btn', false);
        checkAdbStatus();
    }
}
```

#### 3. 局域网扫描

```javascript
async function scanDevices() {
    console.log('点击扫描按钮');
    showMessage('正在启动扫描...');

    try {
        showModal('scan-results-modal');
    } catch (e) {
        console.error('显示弹窗失败:', e);
        return;
    }

    const list = $('scan-results-list');
    const statusText = $('scan-status-text');
    const spinner = $('scan-spinner');

    list.innerHTML = '';
    statusText.textContent = '正在全速扫描局域网 (无需 Root)...';
    spinner.style.display = 'block';

    try {
        const devices = await invoke('scan_tcp_devices');
        renderScanResults(devices);

        if (devices.length === 0) {
            statusText.textContent = '未发现开启 5555 端口的设备';
        } else {
            statusText.textContent = `发现 ${devices.length} 个设备`;
        }
    } catch (e) {
        console.error('扫描出错:', e);
        statusText.textContent = `扫描出错: ${e}`;
        showMessage(`扫描失败: ${e}`);
    } finally {
        spinner.style.display = 'none';
    }
}

function renderScanResults(devices) {
    const list = $('scan-results-list');

    if (devices.length === 0) {
        list.innerHTML = '<div style="padding: 20px; text-align: center;">暂无发现</div>';
        return;
    }

    list.innerHTML = devices.map(ip => `
        <div class="device-item" onclick="selectScanDevice('${ip}')">
            <div>
                <div class="device-ip">${ip}</div>
                <div class="device-hint">端口: 5555</div>
            </div>
            <svg><!-- 箭头图标 --></svg>
        </div>
    `).join('');
}

async function selectScanDevice(ip) {
    hideModal('scan-results-modal');
    $('ip-input').value = ip + ":5555";
    showMessage(`已选择设备: ${ip}`);
    await connectWireless();  // 自动连接
}
```

#### 4. 文件传输（拖拽 + 进度）

```javascript
async function setupFileDropListeners() {
    // 拖拽进入
    await listen('tauri://drag-enter', () => {
        $('drop-overlay').classList.remove('hidden');
    });

    // 拖拽离开
    await listen('tauri://drag-leave', () => {
        $('drop-overlay').classList.add('hidden');
    });

    // 拖拽放下
    await listen('tauri://drag-drop', async (event) => {
        $('drop-overlay').classList.add('hidden');

        const files = event.payload.paths;
        if (files && files.length > 0) {
            for (const file of files) {
                await pushFileFromPath(file);
            }
        }
    });
}

async function pushFileFromPath(path) {
    if (!path) return;

    setLoading('push-file-btn', true);
    updateProgressBar(0, `准备传输 ${path.split(/[\\/]/).pop()}...`);

    try {
        const result = await invoke('adb_push_file', {
            localPath: path,
            remotePath: null  // 默认推送到 /sdcard/Download/
        });
        showMessage(result.message);
    } catch (e) {
        showMessage(`操作失败: ${e}`);
    } finally {
        setLoading('push-file-btn', false);
    }
}

// 监听进度事件
listen('adb-push-progress', (event) => {
    const { progress, message } = event.payload;
    updateProgressBar(progress, message);
});

function updateProgressBar(progress, message) {
    const container = $('push-progress');
    const fill = $('progress-fill');
    const text = $('progress-text');

    if (container.classList.contains('hidden')) {
        container.classList.remove('hidden');
    }

    fill.style.width = `${progress}%`;
    text.textContent = message || `${progress}%`;

    if (progress >= 100) {
        setTimeout(() => {
            container.classList.add('hidden');
            fill.style.width = '0%';
        }, 3000);
    }
}
```

#### 5. 应用流转

```javascript
function renderApps() {
    const grid = $('apps-grid');
    grid.innerHTML = '';

    const appEntries = Object.entries(config.apps || {});

    if (appEntries.length === 0) {
        grid.innerHTML = '<p style="color: #888; text-align: center;">暂无应用</p>';
        return;
    }

    appEntries.forEach(([pkg, app], index) => {
        const card = document.createElement('div');
        card.className = 'app-card';
        card.dataset.pkg = pkg;

        // 排序模式
        if (isSortingMode) {
            card.classList.add('sorting');
            card.innerHTML = `
                <div class="sort-controls">
                    <button class="sort-btn left" ${index === 0 ? 'disabled' : ''}>左移</button>
                    <button class="sort-btn right" ${index === appEntries.length - 1 ? 'disabled' : ''}>右移</button>
                </div>
                <span class="app-name">${app.name || '未命名'}</span>
            `;
        } else {
            // 正常模式
            card.innerHTML = `
                <span class="app-name">${app.name || '未命名'}</span>
                <button class="menu-btn">⋮</button>
                <div class="app-menu" id="menu-${pkg}">
                    <button>编辑</button>
                    <button class="danger">删除</button>
                </div>
            `;
        }

        grid.appendChild(card);
    });
}

async function launchApp(pkg) {
    const appConfig = config.apps[pkg];
    if (!appConfig) return;

    try {
        const result = await invoke('launch_app', {
            package: pkg,
            settings: appConfig.settings || null,
            scrcpyArgs: appConfig.scrcpy_args || null
        });
        showMessage(result.success ? result.message : `错误: ${result.message}`);
    } catch (e) {
        showMessage(`启动失败: ${e}`);
    }
}
```

#### 6. 配置管理

```javascript
function openAppConfigModal(pkg = null) {
    editingAppPackage = pkg;
    const isEdit = pkg !== null;
    const app = isEdit ? config.apps[pkg] : {};

    $('app-config-title').textContent = isEdit ? '编辑应用' : '添加应用';
    $('app-package').value = pkg || '';
    $('app-package').readOnly = isEdit;
    $('app-name').value = app.name || '';

    // 预设下拉
    const presetSelect = $('preset-select');
    presetSelect.innerHTML = '<option value="">快速填充预设</option>';
    for (const name of Object.keys(config.presets)) {
        presetSelect.innerHTML += `<option value="${name}">${name}</option>`;
    }

    // 分辨率设置
    const hasSettings = app.settings && Object.keys(app.settings).length > 0;
    $('use-custom-res').checked = hasSettings;
    toggleResolutionFields();

    if (hasSettings) {
        const parts = (app.settings.full_res || '1920x1080').split('x').map(Number);
        $('app-long').value = Math.max(...parts);
        $('app-short').value = Math.min(...parts);
        $('app-dpi').value = app.settings.dpi || 320;
        $('app-landscape').checked = app.settings.is_landscape || false;
    }

    showModal('app-config-modal');
}

async function saveApp() {
    const pkg = $('app-package').value.trim();
    if (!pkg) {
        showMessage('包名不能为空');
        return;
    }

    let settings = null;
    if ($('use-custom-res').checked) {
        const long = parseInt($('app-long').value) || 1920;
        const short = parseInt($('app-short').value) || 1080;
        const dpi = parseInt($('app-dpi').value) || 320;
        const isLandscape = $('app-landscape').checked;
        const fullRes = isLandscape ? `${Math.max(long, short)}x${Math.min(long, short)}` : `${Math.min(long, short)}x${Math.max(long, short)}`;

        settings = {
            use_full_res_switch: true,
            full_res: fullRes,
            dpi: dpi,
            is_landscape: isLandscape
        };
    }

    const appConfig = {
        name: $('app-name').value || pkg,
        settings: settings,
        scrcpy_args: $('use-app-scrcpy-args').checked ? $('app-scrcpy-args').value : null
    };

    try {
        await invoke('save_app', { package: pkg, appConfig });
        await loadConfig();
        renderApps();
        hideModal('app-config-modal');
        showMessage('已保存');
    } catch (e) {
        showMessage(`保存失败: ${e}`);
    }
}
```

#### 7. 主题切换

```javascript
function applyTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
}

async function toggleTheme() {
    const currentTheme = document.documentElement.getAttribute('data-theme') || 'light';
    const newTheme = currentTheme === 'light' ? 'dark' : 'light';

    applyTheme(newTheme);

    // 更新本地配置缓存
    if (config && config.global_settings) {
        config.global_settings.theme = newTheme;
    }

    try {
        await invoke('set_theme', { theme: newTheme });
    } catch (e) {
        console.error('保存主题失败:', e);
    }
}
```

---

## 样式与主题

### styles.css - 主样式表

**文件路径**：`dist/styles.css` (行数未知)

**设计特点**：
- **CSS 变量**：主题色、间距、圆角等可配置
- **深色模式**：通过 `[data-theme="dark"]` 切换
- **卡片布局**：统一卡片样式，阴影和圆角
- **响应式**：适配不同窗口尺寸

**主题变量**（示例）：

```css
:root {
    --primary-color: #007bff;
    --danger-color: #dc3545;
    --success-color: #28a745;
    --warning-color: #ffc107;
    --bg-color: #f5f5f5;
    --card-bg: #ffffff;
    --text-color: #333333;
    --border-color: #e0e0e0;
    --shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    --radius: 8px;
}

[data-theme="dark"] {
    --bg-color: #1e1e1e;
    --card-bg: #2d2d2d;
    --text-color: #e0e0e0;
    --border-color: #404040;
}
```

### overrides.css - 样式覆盖

**文件路径**：`dist/overrides.css` (行数未知)

**职责**：覆盖 `styles.css` 中的特定样式，提供定制化。

---

## 关键依赖与配置

### Tauri API

**必需的全局对象**：

```javascript
window.__TAURI__.core        // invoke, convertFileSrc
window.__TAURI__.event       // listen, emit
window.__TAURI__.window      // getCurrentWindow
window.__TAURI__.dialog      // open, save
```

**注意**：这些对象仅在 Tauri 环境中可用，不能在浏览器中直接运行。

### 浏览器兼容性

- **目标平台**：Windows (WebView2)
- **ES 版本**：ES6+ (async/await、箭头函数、模板字符串)
- **API 要求**：
  - `classList` (IE10+)
  - `addEventListener` (所有现代浏览器)
  - `querySelector` (IE8+)

---

## 测试与质量

### 当前状态

⚠️ **无自动化测试**

- 无单元测试框架（Jest、Vitest 等）
- 无 E2E 测试（Playwright、Cypress 等）
- 无类型检查（TypeScript、Flow 等）

### 建议的测试改进

#### 单元测试（使用 Vitest）

```javascript
import { describe, it, expect, vi } from 'vitest';

describe('配置管理', () => {
    it('should parse resolution correctly', () => {
        const parts = '1920x1080'.split('x').map(Number);
        expect(parts).toEqual([1920, 1080]);
    });

    it('should calculate landscape resolution', () => {
        const long = 1920;
        const short = 1080;
        const isLandscape = true;
        const fullRes = isLandscape ? `${Math.max(long, short)}x${Math.min(long, short)}` : `${Math.min(long, short)}x${Math.max(long, short)}`;
        expect(fullRes).toBe('1920x1080');
    });
});
```

#### E2E 测试（使用 Playwright）

```javascript
import { test, expect } from '@playwright/test';

test('连接无线设备', async ({ page }) => {
    await page.goto('http://localhost:5173');
    await page.fill('#ip-input', '192.168.1.100');
    await page.click('#connect-btn');
    await expect(page.locator('#status-text')).toContainText('已连接');
});
```

---

## 常见问题 (FAQ)

### Q1: 如何添加新的 UI 组件？

**A**:

1. 在 `index.html` 中添加 HTML 结构
2. 在 `styles.css` 或 `overrides.css` 中添加样式
3. 在 `main.js` 中添加事件监听器

示例：
```html
<!-- HTML -->
<button id="my-new-button">点击我</button>

<!-- JavaScript -->
bindClick('my-new-button', () => {
    console.log('按钮被点击');
});
```

### Q2: 如何调用新的后端命令？

**A**:

使用 `invoke` 函数：
```javascript
const result = await invoke('my_new_command', {
    param1: '值1',
    param2: 42
});
```

**注意**：参数名必须与 Rust 命令定义一致（驼峰命名）。

### Q3: 如何监听后端事件？

**A**:

使用 `listen` 函数：
```javascript
await listen('my-event', (event) => {
    console.log('收到事件:', event.payload);
});
```

后端（Rust）发送事件：
```rust
window.emit("my-event", serde_json::json!({
    "data": "值"
}));
```

### Q4: 如何调试前端代码？

**A**:

1. **开发模式**：运行 `cargo tauri dev`，打开 DevTools（F12）
2. **日志输出**：使用 `console.log()` 或 `console.error()`
3. **断点调试**：在 DevTools 中设置断点
4. **网络监控**：查看 IPC 调用时序

### Q5: 如何优化前端性能？

**A**:

1. **减少 DOM 操作**：批量更新 UI，使用 `DocumentFragment`
2. **防抖/节流**：对频繁触发的事件（如输入、滚动）使用防抖
3. **虚拟滚动**：如果列表很长，考虑虚拟滚动
4. **懒加载**：延迟加载非关键资源

示例（防抖）：
```javascript
function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

const debouncedSearch = debounce((term) => {
    filterDeviceApps();
}, 300);

$('app-search').oninput = (e) => {
    debouncedSearch(e.target.value);
};
```

---

## 相关文件清单

### 核心文件

| 文件 | 行数 | 职责 |
|------|------|------|
| `index.html` | 300+ | 主界面结构 |
| `main.js` | 1250+ | 前端逻辑 |
| `styles.css` | 未知 | 主样式表 |
| `overrides.css` | 未知 | 样式覆盖 |

### 其他文件

| 文件 | 职责 |
|------|------|
| `app_icons/` | 应用图标示例（PNG 格式） |

---

## 下一步建议

### 功能扩展

- [ ] 添加多语言支持（i18n）
- [ ] 支持自定义主题色
- [ ] 添加快捷键支持
- [ ] 优化移动端适配（如果支持移动平台）

### 技术改进

- [ ] 引入 TypeScript（类型安全）
- [ ] 使用构建工具（Vite、Webpack）
- [ ] 添加单元测试（Vitest）
- [ ] 添加 E2E 测试（Playwright）
- [ ] 代码格式化（Prettier）

### UI/UX 改进

- [ ] 添加加载动画
- [ ] 优化错误提示（Toast、Snackbar）
- [ ] 添加操作确认对话框
- [ ] 支持拖拽排序（Drag & Drop）

---

## 代码片段库

### 模态框管理

```javascript
function showModal(id) {
    $(id).classList.add('show');
}

function hideModal(id) {
    $(id).classList.remove('show');
}
```

### 按钮加载状态

```javascript
function setLoading(btnId, loading) {
    const btn = $(btnId);
    btn.disabled = loading;
    btn.classList.toggle('loading', loading);
}
```

### 消息提示

```javascript
function showMessage(msg) {
    const snackbar = $('snackbar');
    snackbar.textContent = msg;
    snackbar.classList.add('show');
    setTimeout(() => snackbar.classList.remove('show'), 3000);
}
```

### DOM 元素快捷访问

```javascript
const $ = (id) => document.getElementById(id);
```

---

**最后扫描时间**：2026-01-27 00:02:27
**文档版本**：v1.0
**作者**：AI 初始化助手
