/**
 * Scrcpy Launcher - 前端主逻辑
 * 使用 Tauri API 与 Rust 后端通信
 */

const { invoke, convertFileSrc } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;
const { open } = window.__TAURI__.dialog;

// 全局状态
// 全局状态
let config = null;
let isSortingMode = false;
let editingAppPackage = null;
let editingPresetName = null;
let deletingAppPackage = null;
let customIconsDir = null;
let deviceApps = [];

let appDatabase = [];
let installedPackages = new Set();
let isInstalledAppsSynced = false;

async function loadAppDatabase() {
    try {
        const response = await fetch('app_database.json?t=' + Date.now());
        appDatabase = await response.json();
        console.log('App database loaded:', appDatabase.length, 'entries');
        // showMessage('数据库已更新: ' + appDatabase.length + ' 个应用');
    } catch (e) {
        console.error('Failed to load app database:', e);
        showMessage('数据库加载失败');
    }
}

// 辅助：安全绑定点击事件
function bindClick(id, handler) {
    const el = $(id);
    if (el) {
        el.onclick = handler;
    } else {
        console.warn(`Element #${id} not found, skipping event binding.`);
    }
}

// ==================== 局域网扫描 (提前定义) ====================
async function scanDevices() {
    console.log('点击扫描按钮');
    showMessage('正在启动扫描...');
    try {
        showModal('scan-results-modal');
    } catch (e) {
        console.error('显示弹窗失败:', e);
        showMessage('显示弹窗失败: ' + e);
        return;
    }

    const list = $('scan-results-list');
    const statusText = $('scan-status-text');
    const spinner = $('scan-spinner');

    if (list) list.innerHTML = '';
    if (statusText) statusText.textContent = '正在全速扫描局域网 (无需 Root)...';
    if (spinner) spinner.style.display = 'block';

    try {
        console.log('调用后端 invoke scan_tcp_devices');
        const devices = await invoke('scan_tcp_devices');
        console.log('扫描完成，结果:', devices);
        renderScanResults(devices);

        if (devices.length === 0) {
            if (statusText) statusText.textContent = '未发现开启 5555 端口的设备';
        } else {
            if (statusText) statusText.textContent = `发现 ${devices.length} 个设备`;
        }
    } catch (e) {
        console.error('扫描出错:', e);
        if (statusText) statusText.textContent = `扫描出错: ${e}`;
        showMessage(`扫描失败: ${e}`);
    } finally {
        if (spinner) spinner.style.display = 'none';
    }
}

function renderScanResults(devices) {
    const list = $('scan-results-list');
    if (!list) return;

    if (devices.length === 0) {
        list.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--text-secondary);">暂无发现<br><small>请确保手机已连接同一 Wi-Fi 并开启了"无线调试"或已通过 USB 执行过 `adb tcpip 5555`</small></div>';
        return;
    }

    list.innerHTML = devices.map(ip => `
        <div class="device-item" onclick="selectScanDevice('${ip}')">
            <div>
                <div class="device-ip">${ip}</div>
                <div class="device-hint">端口: 5555</div>
            </div>
            <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
                <path d="M5 12h14M12 5l7 7-7 7"/>
            </svg>
        </div>
    `).join('');
}

async function selectScanDevice(ip) {
    hideModal('scan-results-modal');
    const input = $('ip-input');
    if (input) input.value = ip + ":5555";
    showMessage(`已选择设备: ${ip}`);

    // 自动连接
    await connectWireless();
}

// 暴露给全局 (HTML onclick)
window.scanDevices = scanDevices;
window.selectScanDevice = selectScanDevice;


// DOM 元素缓存
const $ = (id) => document.getElementById(id);

// 初始化
document.addEventListener('DOMContentLoaded', async () => {
    console.log('App DOMContentLoaded');
    try {
        await loadConfig();
        loadAppDatabase(); // Load silently in background
        setupEventListeners();
        initUI();
        console.log('App Initialized Successfully');
    } catch (e) {
        console.error('App Initialization Failed:', e);
        showMessage('初始化失败: ' + e);
    }
});

// 加载配置
async function loadConfig() {
    try {
        config = await invoke('get_config');
        applyTheme(config.global_settings.theme);

        // 应用全局图标显示设置
        const showIcons = config.global_settings.show_app_icons !== false; // 默认为 true
        const globalShowIcons = $('global-show-icons');
        if (globalShowIcons) globalShowIcons.checked = showIcons;

        // 获取自定义图标目录
        try {
            customIconsDir = await invoke('get_custom_icons_dir');
            console.log('Custom icons dir:', customIconsDir);
        } catch (e) {
            console.warn('Failed to get custom icons dir:', e);
        }

    } catch (e) {

        console.error('加载配置失败:', e);
        showMessage('加载配置失败');
    }
}

// 初始化 UI
function initUI() {
    if (config.first_run) {
        showModal('first-run-modal');
    } else {
        checkAdbStatus();
        renderApps();
        loadAppDatabase(); // 加载数据库
        loadHistoryDropdown();
        if (config.adb_history.length > 0) {
            $('ip-input').value = config.adb_history[0];
        }
    }
}

// 设置事件监听器
function setupEventListeners() {
    console.log('Setting up event listeners...');

    // ADB 区域
    bindClick('refresh-btn', checkAdbStatus);
    bindClick('connect-btn', connectWireless);
    bindClick('history-btn', toggleHistoryDropdown);
    bindClick('tcpip-btn', enableTcpip);
    bindClick('disconnect-btn', disconnectAll);
    bindClick('kill-btn', killScrcpy);
    bindClick('push-file-btn', pushFile);
    bindClick('pair-btn', openPairModal);
    bindClick('pair-cancel-btn', () => hideModal('pair-modal'));
    bindClick('pair-confirm-btn', pairDevice);

    // 新增：扫描功能
    bindClick('scan-btn', () => {
        console.log('Scan button clicked via safe bind');
        scanDevices();
    });
    bindClick('scan-cancel-btn', () => hideModal('scan-results-modal'));

    // 标签页
    document.querySelectorAll('.tab').forEach(tab => {
        tab.onclick = () => switchTab(tab.dataset.tab);
    });

    // 直接投屏
    bindClick('settings-btn', () => openSettingsModal());
    bindClick('advanced-btn', openAdvancedModal);
    bindClick('theme-toggle-btn', toggleTheme);
    bindClick('mirror-btn', launchMirror);
    bindClick('audio-btn', launchAudio);

    // 应用流转
    bindClick('add-app-btn', () => openAppConfigModal());
    bindClick('sort-btn', toggleSortMode);

    // 绑定应用名输入建议
    const appNameInput = $('app-name');
    if (appNameInput) {
        appNameInput.addEventListener('input', handleAppNameInput);
        document.addEventListener('click', (e) => {
            if (e.target !== appNameInput && e.target.id !== 'search-name-only') {
                $('app-name-suggestions').classList.add('hidden');
            }
        });

        // Toggle Search Name Only
        const searchNameOnly = $('search-name-only');
        if (searchNameOnly) {
            searchNameOnly.onchange = handleAppNameInput;
        }
    }

    // 首次运行
    bindClick('save-first-run-btn', saveFirstRunConfig);

    // 设置
    bindClick('settings-cancel-btn', () => hideModal('settings-modal'));
    bindClick('settings-save-btn', saveSettings);

    // Filter Installed Toggle (Global)
    const globalFilterInstalled = $('global-filter-installed');
    if (globalFilterInstalled) {
        globalFilterInstalled.onchange = () => {
            // Optional: Immediately sync if enabled and connected
            if (globalFilterInstalled.checked && config.adb_status && config.adb_status.connected && !isInstalledAppsSynced) {
                syncInstalledApps();
            }
        };
    }

    const useCustomArgs = $('use-custom-args');
    if (useCustomArgs) useCustomArgs.onchange = toggleCustomArgsMode;

    const useAppStreamArgs = $('use-app-stream-args');
    if (useAppStreamArgs) useAppStreamArgs.onchange = toggleAppStreamArgs;

    const useAppCustomArgs = $('use-app-custom-args');
    if (useAppCustomArgs) useAppCustomArgs.onchange = toggleAppCustomArgsMode;

    bindClick('advanced-defaults-btn', restoreDefaults);
    bindClick('advanced-cancel-btn', () => hideModal('advanced-modal'));
    bindClick('advanced-save-btn', saveAdvancedSettings);

    // 应用配置
    bindClick('browse-apps-btn', () => showModal('browse-apps-modal'));
    bindClick('manage-presets-btn', () => openPresetsModal());

    const presetSelect = $('preset-select');
    if (presetSelect) presetSelect.onchange = applyPreset;

    const useCustomRes = $('use-custom-res');
    if (useCustomRes) useCustomRes.onchange = toggleResolutionFields;

    const useAppScrcpyArgs = $('use-app-scrcpy-args');
    if (useAppScrcpyArgs) useAppScrcpyArgs.onchange = toggleAppScrcpyArgs;

    bindClick('app-cancel-btn', () => hideModal('app-config-modal'));
    bindClick('app-save-btn', saveApp);
    bindClick('select-icon-btn', selectAppIcon); // 选择图标事件
    bindClick('reset-icon-btn', deleteCustomIcon); // 绑定恢复默认按钮
    bindClick('open-icons-folder-btn', openCustomIconsDir); // 绑定打开文件夹按钮


    // App Package Input Listener for Icon Preview
    const appPackageInput = $('app-package');
    if (appPackageInput) {
        appPackageInput.addEventListener('input', function () {
            updateIconPreview(this.value.trim());
        });
    }

    // 浏览应用
    bindClick('browse-back-btn', () => hideModal('browse-apps-modal'));
    bindClick('load-apps-btn', loadDeviceApps);

    const appSearch = $('app-search');
    if (appSearch) appSearch.oninput = filterDeviceApps;

    // 预设管理
    bindClick('add-preset-btn', () => openPresetEditModal());
    bindClick('presets-close-btn', () => hideModal('presets-modal'));

    // 编辑预设
    bindClick('preset-cancel-btn', () => hideModal('preset-edit-modal'));
    bindClick('preset-save-btn', savePreset);

    // 删除确认
    bindClick('delete-cancel-btn', () => hideModal('delete-confirm-modal'));
    bindClick('delete-confirm-btn', performDeleteApp);

    // 点击外部关闭历史下拉
    document.addEventListener('click', (e) => {
        if (!e.target.closest('#history-btn') && !e.target.closest('#history-dropdown')) {
            const dropdown = $('history-dropdown');
            if (dropdown) dropdown.classList.remove('show');
        }
    });

    // 监听进度事件
    listen('adb-push-progress', (event) => {
        const { progress, message } = event.payload;
        updateProgressBar(progress, message);
    });

    // 监听文件拖拽
    setupFileDropListeners();
}

async function setupFileDropListeners() {
    // Tauri v2 拖拽事件 (v1 是 file-drop-hover)
    await listen('tauri://drag-enter', () => {
        $('drop-overlay').classList.remove('hidden');
    });

    // 拖拽离开 (v1 是 file-drop-cancelled)
    await listen('tauri://drag-leave', () => {
        $('drop-overlay').classList.add('hidden');
    });

    // 拖拽放下 (v1 是 file-drop)
    await listen('tauri://drag-drop', async (event) => {
        $('drop-overlay').classList.add('hidden');

        // Tauri v2 payload 结构: { paths: string[], position: { x, y } }
        const payload = event.payload;
        // 兼容处理：检查 paths 字段，或者如果 payload 本身是数组 (v1/早期v2)
        const files = payload.paths || (Array.isArray(payload) ? payload : []);

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
    // 重置并显示进度条 (追加模式或者单文件模式)
    // 这里简单处理：每次都更新进度条
    updateProgressBar(0, `准备传输 ${path.split(/[\\/]/).pop()}...`);

    try {
        // 默认推送到 /sdcard/Download/
        const result = await invoke('adb_push_file', { localPath: path, remotePath: null });
        showMessage(result.message);
    } catch (e) {
        showMessage(`操作失败: ${e}`);
    } finally {
        setLoading('push-file-btn', false);
    }
}

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

// ==================== ADB 操作 ====================

async function checkAdbStatus() {
    try {
        const status = await invoke('check_adb_status');
        $('status-dot').classList.toggle('connected', status.connected);
        $('status-text').textContent = status.message;

        // 智能补全状态同步
        if (status.connected) {
            if (!isInstalledAppsSynced && (config.filter_installed_apps !== false)) {
                syncInstalledApps();
            }
        } else {
            // 断开连接，清空缓存，确保降级为显示所有
            installedPackages.clear();
            isInstalledAppsSynced = false;
        }

    } catch (e) {
        $('status-dot').classList.remove('connected');
        $('status-text').textContent = '检查失败';
        installedPackages.clear(); // 检查失败视为断开，清空缓存
    }
}

async function syncInstalledApps() {
    if (isInstalledAppsSynced) return; // 避免重复同步

    console.log('开始同步已安装应用...');
    try {
        const apps = await invoke('get_installed_apps');
        installedPackages = new Set(apps);
        isInstalledAppsSynced = true;
        console.log(`已同步 ${installedPackages.size} 个已安装应用。`);
    } catch (e) {
        console.error('同步已安装应用失败:', e);
        installedPackages.clear();
        isInstalledAppsSynced = false;
    }
}

async function syncInstalledApps() {
    if (isInstalledAppsSynced) return; // 避免重复同步

    console.log('开始同步已安装应用...');
    try {
        const apps = await invoke('get_installed_apps');
        installedPackages = new Set(apps);
        isInstalledAppsSynced = true;
        console.log(`已同步 ${installedPackages.size} 个已安装应用。`);
    } catch (e) {
        console.error('同步已安装应用失败:', e);
        installedPackages.clear();
        isInstalledAppsSynced = false;
    }
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

async function disconnectAll() {
    setLoading('disconnect-btn', true);
    try {
        const result = await invoke('disconnect_all');
        showMessage(result.message);
    } catch (e) {
        showMessage(`失败: ${e}`);
    } finally {
        setLoading('disconnect-btn', false);
        checkAdbStatus();
    }
}

async function killScrcpy() {
    setLoading('kill-btn', true);
    try {
        const result = await invoke('kill_scrcpy');
        showMessage(result.message);
    } catch (e) {
        showMessage(`失败: ${e}`);
    } finally {
        setLoading('kill-btn', false);
    }
}

async function pushFile() {
    try {
        const selected = await open({
            multiple: false,
            directory: false,
        });

        if (selected) {
            setLoading('push-file-btn', true);
            // 重置并显示进度条
            updateProgressBar(0, '准备开始...');

            // 默认推送到 /sdcard/Download/，后端已处理默认值
            const result = await invoke('adb_push_file', { localPath: selected, remotePath: null });
            showMessage(result.message);
        }
    } catch (e) {
        showMessage(`操作失败: ${e}`);
    } finally {
        setLoading('push-file-btn', false);
    }
}

// ==================== 无线配对 ====================

function openPairModal() {
    // 尝试预填 IP (从输入框获取 IP 部分)
    const currentInput = $('ip-input').value.trim();
    if (currentInput) {
        const ip = currentInput.split(':')[0];
        if (ip && ip.match(/^\d+\.\d+\.\d+\.\d+$/)) {
            $('pair-ip').value = `${ip}:`;
        }
    }
    showModal('pair-modal');
    $('pair-code').focus();
}

async function pairDevice() {
    const addr = $('pair-ip').value.trim();
    const code = $('pair-code').value.trim();

    if (!addr || !addr.includes(':')) {
        showMessage('请输入正确的 IP:端口');
        return;
    }
    if (!code) {
        showMessage('请输入配对码');
        return;
    }

    setLoading('pair-confirm-btn', true);
    try {
        const result = await invoke('pair_device', { addr, code });
        if (result.success) {
            hideModal('pair-modal');
            showMessage(result.message);
            // 提示用户连接
            const connectIp = addr.split(':')[0]; // 这里配对端口通常不是连接端口
            showMessage(`配对成功！请在主界面输入 "连接端口" 进行连接 (IP: ${connectIp})`);

            // 尝试将 IP 填入主界面 (保留方便用户修改端口)
            $('ip-input').value = connectIp + ":";
        } else {
            showMessage(result.message);
        }
    } catch (e) {
        showMessage(`配对请求失败: ${e}`);
    } finally {
        setLoading('pair-confirm-btn', false);
    }
}

// ==================== 启动 Scrcpy ====================

async function launchMirror() {
    setLoading('mirror-btn', true);
    try {
        const result = await invoke('launch_mirror');
        showMessage(result.success ? result.message : `错误: ${result.message}`);
    } catch (e) {
        showMessage(`启动失败: ${e}`);
    } finally {
        setLoading('mirror-btn', false);
    }
}

async function launchAudio() {
    setLoading('audio-btn', true);
    try {
        const result = await invoke('launch_audio');
        showMessage(result.success ? result.message : `错误: ${result.message}`);
    } catch (e) {
        showMessage(`启动失败: ${e}`);
    } finally {
        setLoading('audio-btn', false);
    }
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

// ==================== 标签页 ====================

function switchTab(tabName) {
    document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));

    document.querySelector(`.tab[data-tab="${tabName}"]`).classList.add('active');
    $(`tab-${tabName}`).classList.add('active');
}

// ==================== 应用管理 ====================

function renderApps() {
    const grid = $('apps-grid');
    grid.innerHTML = '';

    const showIcons = config.global_settings.show_app_icons !== false;
    grid.classList.toggle('show-icons', showIcons);

    const appEntries = Object.entries(config.apps || {});

    if (appEntries.length === 0) {
        grid.innerHTML = '<p style="color: #888; text-align: center; padding: 40px;">暂无应用，点击上方"添加新应用"</p>';
        return;
    }

    appEntries.forEach(([pkg, app], index) => {
        const card = document.createElement('div');
        card.className = 'app-card';
        card.dataset.pkg = pkg;

        // 排序模式
        if (isSortingMode) {
            card.classList.add('sorting');
            // 使用左右按钮排序
            card.innerHTML = `
                <div class="sort-controls">
                    <button class="sort-btn left" ${index === 0 ? 'disabled' : ''} title="左移">
                        <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"></polyline></svg>
                    </button>
                    <button class="sort-btn right" ${index === appEntries.length - 1 ? 'disabled' : ''} title="右移">
                        <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>
                    </button>
                </div>
                <!-- 图标 (双层加载: custom -> database -> hidden) -->
                ${showIcons ? `
                <div class="app-icon-wrapper">
                    <img id="app-icon-${pkg}" src="app_icons/${pkg}.png" 
                         onerror="this.onerror=null; this.style.visibility='hidden';" style="display: block;">
                </div>` : ''}
                <span class="app-name">${app.name || '未命名'}</span>
            `;


            // 左移按钮 (前移)
            const leftBtn = card.querySelector('.sort-btn.left');
            if (leftBtn && index > 0) {
                leftBtn.onclick = () => moveApp(pkg, index, index - 1);
            }

            // 右移按钮 (后移)
            const rightBtn = card.querySelector('.sort-btn.right');
            if (rightBtn && index < appEntries.length - 1) {
                rightBtn.onclick = () => moveApp(pkg, index, index + 1);
            }
        } else {
            // 创建内容
            card.innerHTML = `
                <!-- 图标 (双层加载: custom -> database -> hidden) -->
                ${showIcons ? `
                <div class="app-icon-wrapper">
                    <img id="app-icon-${pkg}" src="app_icons/${pkg}.png" 
                         onerror="this.onerror=null; this.style.visibility='hidden';" style="display: block;">
                </div>` : ''}
                <span class="app-name">${app.name || '未命名'}</span>
                <button class="menu-btn">⋮</button>
                <div class="app-menu" id="menu-${pkg}">
                    <button>编辑</button>
                    <button class="danger">删除</button>
                </div>
            `;

            // 非排序模式 - 点击启动应用
            card.onclick = function (e) {
                if (!e.target.closest('.menu-btn') && !e.target.closest('.app-menu')) {
                    launchApp(pkg);
                }
            };

            // 菜单按钮
            const menuBtn = card.querySelector('.menu-btn');
            if (menuBtn) {
                menuBtn.onclick = function (e) {
                    e.stopPropagation();
                    toggleAppMenu(pkg);
                };
            }

            // 菜单项
            const menu = card.querySelector('.app-menu');
            if (menu) {
                const buttons = menu.querySelectorAll('button');
                buttons[0].onclick = function (e) {
                    e.stopPropagation();
                    openAppConfigModal(pkg);
                };
                buttons[1].onclick = function (e) {
                    e.stopPropagation();
                    deleteAppConfirm(pkg);
                };
            }
        }

        grid.appendChild(card);
    });

    // 异步加载自定义图标 (Base64) - 解决 Release 版本权限导致的显示问题
    if (showIcons) {
        loadCustomIconsForList(appEntries.map(([pkg]) => pkg));
    }
}

async function loadCustomIconsForList(packages) {
    for (const pkg of packages) {
        try {
            const base64Data = await invoke('get_app_icon_data', { package: pkg });
            if (base64Data) {
                const img = $(`app-icon-${pkg}`);
                if (img) {
                    img.src = base64Data;
                    img.style.visibility = 'visible';
                }
            }
        } catch (e) {
            console.warn(`Failed to load icon for ${pkg}:`, e);
        }
    }
}


// 移动应用位置
async function moveApp(pkg, fromIndex, toIndex) {
    const keys = Object.keys(config.apps);
    const item = keys.splice(fromIndex, 1)[0];
    keys.splice(toIndex, 0, item);

    try {
        await invoke('reorder_apps', { newOrder: keys });
        await loadConfig();
        renderApps();
    } catch (e) {
        showMessage(`排序失败: ${e}`);
    }
}

function toggleAppMenu(pkg) {
    const targetMenu = $(`menu-${pkg}`);
    const isCurrentlyOpen = targetMenu.classList.contains('show');

    // Close all menus first
    document.querySelectorAll('.app-menu').forEach(m => m.classList.remove('show'));

    // If it wasn't open, open it. If it was open, we just closed it above.
    if (!isCurrentlyOpen) {
        targetMenu.classList.add('show');
    }
}

// Global click listener for closing app menus
document.addEventListener('click', (e) => {
    // If click is not on a menu button and not inside a menu, close all menus
    if (!e.target.closest('.menu-btn') && !e.target.closest('.app-menu')) {
        document.querySelectorAll('.app-menu').forEach(m => m.classList.remove('show'));
    }
});

function toggleSortMode() {
    isSortingMode = !isSortingMode;
    $('sort-btn').textContent = isSortingMode ? '完成' : '排序';
    renderApps();
}

async function reorderApps(fromPkg, toPkg) {
    if (fromPkg === toPkg) return;

    const keys = Object.keys(config.apps);
    const fromIdx = keys.indexOf(fromPkg);
    const toIdx = keys.indexOf(toPkg);

    keys.splice(fromIdx, 1);
    keys.splice(toIdx, 0, fromPkg);

    try {
        await invoke('reorder_apps', { newOrder: keys });
        await loadConfig();
        renderApps();
    } catch (e) {
        showMessage(`排序失败: ${e}`);
    }
}

async function deleteAppConfirm(pkg) {
    deletingAppPackage = pkg;
    const appName = config.apps[pkg]?.name || pkg;
    const textEl = $('delete-confirm-text');
    if (textEl) textEl.textContent = `确定要删除 "${appName}" 吗？`;

    showModal('delete-confirm-modal');
}

async function performDeleteApp() {
    if (!deletingAppPackage) return;

    // 立即关闭弹窗
    hideModal('delete-confirm-modal');

    try {
        await invoke('delete_app', { package: deletingAppPackage });
        await loadConfig();
        renderApps();
        showMessage('已删除');
    } catch (e) {
        showMessage(`删除失败: ${e}`);
    } finally {
        deletingAppPackage = null;
    }
}

// ==================== 应用配置对话框 ====================

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
    } else {
        $('app-long').value = '1920';
        $('app-short').value = '1080';
        $('app-dpi').value = '320';
        $('app-landscape').checked = false;
    }

    $('use-app-scrcpy-args').checked = app.scrcpy_args !== null && app.scrcpy_args !== undefined;
    $('app-scrcpy-args').value = app.scrcpy_args || config.custom_args || '';
    toggleAppScrcpyArgs();

    // 更新图标预览
    updateIconPreview(pkg || '');

    showModal('app-config-modal');
}

// 选择应用图标
async function selectAppIcon() {
    try {
        const selected = await open({
            multiple: false,
            filters: [{
                name: 'Image',
                extensions: ['png', 'jpg', 'jpeg', 'ico', 'webp']
            }]
        });

        if (selected) {
            // 获取当前包名 (如果尚未输入包名，提示用户先输入)
            const pkg = $('app-package').value.trim();
            if (!pkg) {
                showMessage('请先输入应用包名');
                return;
            }

            // 调用后端保存图标
            const result = await invoke('save_app_icon', {
                package: pkg,
                sourcePath: selected
            });
            showMessage(result);

            // 刷新预览
            updateIconPreview(pkg);
        }
    } catch (e) {
        showMessage('选择图标失败: ' + e);
        console.error(e);
    }
}

// 更新图标预览 (逻辑: 先尝试 custom，失败(onerror) 加载 database，再失败隐藏)
// 更新图标预览
async function updateIconPreview(pkg) {
    const img = $('app-icon-preview');
    if (!img) return;

    img.style.visibility = 'visible';
    img.onerror = null;

    if (pkg) {
        const timestamp = Date.now();

        try {
            // 优先尝试获取 Base64 数据 (解决权限问题)
            const base64Data = await invoke('get_app_icon_data', { package: pkg });
            if (base64Data) {
                console.log("Using Base64 icon for", pkg);
                img.src = base64Data;
                return;
            }
        } catch (e) {
            console.warn("Failed to get base64 icon:", e);
        }

        // 降级: 使用标准路径 (仅作备用)
        // img.src = customIconsDir
        //     ? convertFileSrc(customIconsDir + (customIconsDir.endsWith('\\') || customIconsDir.endsWith('/') ? '' : '\\') + pkg + '.png') + `?t=${timestamp}`
        //     : `custom_icons/${pkg}.png?t=${timestamp}`;

        // 如果没有自定义图标，或者加载失败，尝试加载默认数据库图标
        img.src = `app_icons/${pkg}.png?t=${timestamp}`;

        img.onerror = function () {
            this.onerror = null;
            this.style.visibility = 'hidden';
        };

    } else {
        img.src = '';
        img.style.visibility = 'hidden';
    }
}


// 删除自定义图标
async function deleteCustomIcon(e) {
    if (e) e.stopPropagation();

    const pkg = $('app-package').value.trim();
    if (!pkg) return;

    if (!confirm('确定要恢复默认图标吗？(自定义图标将被删除)')) return;

    try {
        const result = await invoke('delete_custom_icon', { package: pkg });
        showMessage(result);
        updateIconPreview(pkg);
        // 刷新列表中的图标
        const listIcon = $(`app-icon-${pkg}`);
        if (listIcon) {
            listIcon.src = `app_icons/${pkg}.png?t=` + Date.now();
            listIcon.style.visibility = 'visible';
        }
    } catch (e) {
        showMessage(e);
    }

}

function toggleResolutionFields() {
    $('resolution-fields').classList.toggle('hidden', !$('use-custom-res').checked);
}

function toggleAppScrcpyArgs() {
    $('app-scrcpy-args').classList.toggle('hidden', !$('use-app-scrcpy-args').checked);
}

function applyPreset() {
    const presetName = $('preset-select').value;
    if (!presetName || !config.presets[presetName]) return;

    const preset = config.presets[presetName];
    $('app-long').value = preset.long;
    $('app-short').value = preset.short;
    $('app-dpi').value = preset.dpi;
    $('app-landscape').checked = preset.is_landscape;
    $('use-custom-res').checked = true;
    toggleResolutionFields();
}

async function openCustomIconsDir(e) {
    if (e) e.stopPropagation();
    try {
        await invoke('open_custom_icons_dir');
    } catch (e) {
        showMessage(`打开目录失败: ${e}`);
    }
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

// 应用名输入建议
function handleAppNameInput() {
    const input = $('app-name');
    const suggestionsDiv = $('app-name-suggestions');
    const searchTerm = input.value.toLowerCase();

    if (searchTerm.length < 1) {
        suggestionsDiv.classList.add('hidden');
        return;
    }

    const filteredSuggestions = appDatabase.filter(app =>
        app.name.toLowerCase().includes(searchTerm) || app.package.toLowerCase().includes(searchTerm)
    ).slice(0, 10); // Limit to 10 suggestions

    suggestionsDiv.innerHTML = '';
    if (filteredSuggestions.length > 0) {
        filteredSuggestions.forEach(app => {
            const item = document.createElement('div');
            item.classList.add('suggestion-item');
            item.textContent = `${app.name} (${app.package})`;
            item.onclick = () => {
                input.value = app.name;
                $('app-package').value = app.package;
                updateIconPreview(app.package);
                suggestionsDiv.classList.add('hidden');
            };
            suggestionsDiv.appendChild(item);
        });
        suggestionsDiv.classList.remove('hidden');
    } else {
        suggestionsDiv.classList.add('hidden');
    }
}


// ==================== 浏览设备应用 ====================

async function loadDeviceApps() {
    setLoading('load-apps-btn', true);
    try {
        deviceApps = await invoke('get_installed_apps');
        deviceApps.sort();
        renderDeviceApps(deviceApps);
        showMessage(`成功加载 ${deviceApps.length} 个应用`);
    } catch (e) {
        showMessage(`加载失败: ${e}`);
    } finally {
        setLoading('load-apps-btn', false);
    }
}

function renderDeviceApps(apps) {
    const list = $('device-apps-list');
    list.innerHTML = apps.map(pkg => `
        <div class="apps-list-item" onclick="selectDeviceApp('${pkg}')">
            <span>${pkg}</span>
        </div>
    `).join('');
}

function filterDeviceApps() {
    const term = $('app-search').value.toLowerCase();
    const filtered = deviceApps.filter(pkg => pkg.toLowerCase().includes(term));
    renderDeviceApps(filtered);
}

function selectDeviceApp(pkg) {
    $('app-package').value = pkg;
    if (!$('app-name').value) {
        $('app-name').value = pkg;
    }
    hideModal('browse-apps-modal');
}

// ==================== 预设管理 ====================

function openPresetsModal() {
    renderPresets();
    showModal('presets-modal');
}

function renderPresets() {
    const list = $('presets-list');
    list.innerHTML = '';

    for (const [name, preset] of Object.entries(config.presets)) {
        const item = document.createElement('div');
        item.className = 'preset-item';
        item.innerHTML = `
            <div class="preset-info">
                <div class="name">${name}</div>
                <div class="details">${preset.long}×${preset.short} · DPI ${preset.dpi} · ${preset.is_landscape ? '横屏' : '竖屏'}</div>
            </div>
            <div class="preset-actions">
                <button class="icon-btn" onclick="openPresetEditModal('${name}')" title="编辑">
                    <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg>
                </button>
                <button class="icon-btn" onclick="deletePreset('${name}')" title="删除">
                    <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
                </button>
            </div>
        `;
        list.appendChild(item);
    }
}

function openPresetEditModal(name = null) {
    editingPresetName = name;
    const isEdit = name !== null;
    const preset = isEdit ? config.presets[name] : { long: 1920, short: 1080, dpi: 320, is_landscape: false };

    $('preset-edit-title').textContent = isEdit ? '编辑预设' : '添加预设';
    $('preset-name').value = name || '';
    $('preset-long').value = preset.long;
    $('preset-short').value = preset.short;
    $('preset-dpi').value = preset.dpi;
    $('preset-landscape').checked = preset.is_landscape;

    showModal('preset-edit-modal');
}

async function savePreset() {
    const name = $('preset-name').value.trim();
    if (!name) {
        showMessage('预设名称不能为空');
        return;
    }

    const long = parseInt($('preset-long').value) || 1920;
    const short = parseInt($('preset-short').value) || 1080;
    const dpi = parseInt($('preset-dpi').value) || 320;
    const isLandscape = $('preset-landscape').checked;

    try {
        if (editingPresetName && editingPresetName !== name) {
            await invoke('update_preset', {
                oldName: editingPresetName,
                newName: name,
                long, short, dpi, isLandscape
            });
        } else {
            await invoke('save_preset', { name, long, short, dpi, isLandscape });
        }
        await loadConfig();
        renderPresets();
        hideModal('preset-edit-modal');
        showMessage('预设已保存');
    } catch (e) {
        showMessage(`保存失败: ${e}`);
    }
}

async function deletePreset(name) {
    if (!confirm(`确定要删除预设 "${name}" 吗？`)) return;

    try {
        await invoke('delete_preset', { name });
        await loadConfig();
        renderPresets();
        showMessage('已删除');
    } catch (e) {
        showMessage(`删除失败: ${e}`);
    }
}

// ==================== 设置 ====================

function openSettingsModal() {
    $('global-dpi').value = config.global_settings.dpi;
    $('global-res').value = config.global_settings.full_res;
    $('global-show-icons').checked = config.global_settings.show_app_icons !== false;
    $('global-filter-installed').checked = config.global_settings.filter_installed_apps !== false;

    $('tray-count').value = config.tray_app_count;
    $('tray-mirror').checked = config.tray_show_mirror;
    $('tray-audio').checked = config.tray_show_audio;
    showModal('settings-modal');
}

async function saveSettings() {
    const dpi = parseInt($('global-dpi').value) || 400;
    const fullRes = $('global-res').value || '1200x2670';
    const showIcons = $('global-show-icons').checked;
    const filterInstalled = $('global-filter-installed').checked;

    const trayCount = parseInt($('tray-count').value) || 4;
    const showMirror = $('tray-mirror').checked;
    const showAudio = $('tray-audio').checked;

    setLoading('settings-save-btn', true);
    try {
        await invoke('save_global_settings', { dpi, fullRes, showAppIcons: showIcons, filterInstalledApps: filterInstalled });
        await invoke('save_tray_settings', { appCount: trayCount, showMirror: showMirror, showAudio: showAudio });
        await loadConfig();
        renderApps(); // 重新渲染以应用图标设置
        hideModal('settings-modal');
        showMessage('设置已保存');
    } catch (e) {
        showMessage(`保存失败: ${e}`);
    } finally {
        setLoading('settings-save-btn', false);
    }
}

// ==================== 高级设置 ====================

function openAdvancedModal() {
    // 全局选项
    $('use-custom-args').checked = config.use_custom_args;
    $('custom-args').value = config.custom_args;
    toggleCustomArgsMode();

    const opts = config.scrcpy_options;
    $('opt-hid').checked = opts.hid_keyboard;
    $('opt-awake').checked = opts.stay_awake;
    $('opt-screen-off').checked = opts.turn_screen_off;
    $('opt-touches').checked = opts.show_touches;
    $('opt-top').checked = opts.always_on_top;
    $('opt-borderless').checked = opts.borderless;
    $('opt-power-off').checked = opts.power_off_on_close;
    $('opt-ime').checked = opts.local_ime;
    $('opt-max-size').value = opts.max_size || '';
    $('opt-max-fps').value = opts.max_fps || '';

    // 根据配置开启/关闭选项
    if (config.use_custom_args) {
        $('use-custom-args').checked = true;
        toggleCustomArgsMode();
    } else {
        $('use-custom-args').checked = false;
        toggleCustomArgsMode();
    }

    // Filter Installed (default true if undefined)
    const filterInstalled = $('opt-filter-installed');
    if (filterInstalled) {
        filterInstalled.checked = config.filter_installed_apps !== false;
    }

    // 应用流转选项
    $('use-app-stream-args').checked = config.use_app_stream_args;
    $('use-app-custom-args').checked = config.use_app_custom_args;
    $('app-custom-args').value = config.app_custom_args;
    toggleAppStreamArgs();
    toggleAppCustomArgsMode();

    const appOpts = config.app_stream_options;
    $('app-opt-hid').checked = appOpts.hid_keyboard;
    $('app-opt-awake').checked = appOpts.stay_awake;
    $('app-opt-screen-off').checked = appOpts.turn_screen_off;
    $('app-opt-touches').checked = appOpts.show_touches;
    $('app-opt-top').checked = appOpts.always_on_top;
    $('app-opt-borderless').checked = appOpts.borderless;
    $('app-opt-power-off').checked = appOpts.power_off_on_close;
    $('app-opt-ime').checked = appOpts.local_ime;
    $('app-opt-max-size').value = appOpts.max_size || '';
    $('app-opt-max-fps').value = appOpts.max_fps || '';

    showModal('advanced-modal');
}

function toggleCustomArgsMode() {
    const useCustom = $('use-custom-args').checked;
    $('global-options-grid').classList.toggle('hidden', useCustom);
    $('custom-args').classList.toggle('hidden', !useCustom);
}

function toggleAppStreamArgs() {
    $('app-options-container').classList.toggle('hidden', !$('use-app-stream-args').checked);
}

function toggleAppCustomArgsMode() {
    const useCustom = $('use-app-custom-args').checked;
    $('app-options-grid').classList.toggle('hidden', useCustom);
    $('app-custom-args').classList.toggle('hidden', !useCustom);
}

function restoreDefaults() {
    $('use-custom-args').checked = false;
    $('custom-args').value = '-K --stay-awake';
    toggleCustomArgsMode();

    $('opt-hid').checked = true;
    $('opt-awake').checked = true;
    $('opt-screen-off').checked = false;
    $('opt-touches').checked = false;
    $('opt-top').checked = false;
    $('opt-borderless').checked = false;
    $('opt-power-off').checked = false;
    $('opt-ime').checked = false;
    $('opt-max-size').value = '';
    $('opt-max-fps').value = '';
    $('opt-filter-installed').checked = true; // Default to true

    $('use-app-stream-args').checked = true;
    $('use-app-custom-args').checked = false;
    $('app-custom-args').value = '-K --stay-awake --display-ime-policy=local';
    toggleAppStreamArgs();
    toggleAppCustomArgsMode();

    $('app-opt-hid').checked = true;
    $('app-opt-awake').checked = true;
    $('app-opt-screen-off').checked = false;
    $('app-opt-touches').checked = false;
    $('app-opt-top').checked = false;
    $('app-opt-borderless').checked = false;
    $('app-opt-power-off').checked = false;
    $('app-opt-ime').checked = true;
    $('app-opt-max-size').value = '';
    $('app-opt-max-fps').value = '';
}

async function saveAdvancedSettings() {
    const options = {
        hid_keyboard: $('opt-hid').checked,
        stay_awake: $('opt-awake').checked,
        turn_screen_off: $('opt-screen-off').checked,
        show_touches: $('opt-touches').checked,
        always_on_top: $('opt-top').checked,
        borderless: $('opt-borderless').checked,
        power_off_on_close: $('opt-power-off').checked,
        local_ime: $('opt-ime').checked,
        max_size: parseInt($('opt-max-size').value) || 0,
        max_fps: parseInt($('opt-max-fps').value) || 0,
    };

    const appOptions = {
        hid_keyboard: $('app-opt-hid').checked,
        stay_awake: $('app-opt-awake').checked,
        turn_screen_off: $('app-opt-screen-off').checked,
        show_touches: $('app-opt-touches').checked,
        always_on_top: $('app-opt-top').checked,
        borderless: $('app-opt-borderless').checked,
        power_off_on_close: $('app-opt-power-off').checked,
        local_ime: $('app-opt-ime').checked,
        max_size: parseInt($('app-opt-max-size').value) || 0,
        max_fps: parseInt($('app-opt-max-fps').value) || 0,
    };

    try {
        await invoke('save_scrcpy_options', {
            useCustom: $('use-custom-args').checked,
            customArgs: $('custom-args').value,
            options: options,
            useAppStreamArgs: $('use-app-stream-args').checked,
            useAppCustomArgs: $('use-app-custom-args').checked,
            appCustomArgs: $('app-custom-args').value,
            appOptions: appOptions,
        });
        await loadConfig();
        hideModal('advanced-modal');
        showMessage('高级设置已保存！');
    } catch (e) {
        showMessage(`保存失败: ${e}`);
    }
}

// ==================== 首次运行 ====================

async function saveFirstRunConfig() {
    try {
        await invoke('save_first_run_config', {
            phoneLong: parseInt($('phone-long').value) || 2670,
            phoneShort: parseInt($('phone-short').value) || 1200,
            phoneDpi: parseInt($('phone-dpi').value) || 400,
            phoneLandscape: $('phone-landscape').checked,
            pcLong: parseInt($('pc-long').value) || 2256,
            pcShort: parseInt($('pc-short').value) || 1504,
            pcDpi: parseInt($('pc-dpi').value) || 260,
            pcLandscape: $('pc-landscape').checked,
        });
        await loadConfig();
        hideModal('first-run-modal');
        showMessage('配置已保存！');
        initUI();
    } catch (e) {
        showMessage(`保存失败: ${e}`);
    }
}

// ==================== 工具函数 ====================

function loadHistoryDropdown() {
    const dropdown = $('history-dropdown');
    dropdown.innerHTML = config.adb_history.map(ip => `
        <div class="history-item" onclick="selectHistory('${ip}')">${ip}</div>
    `).join('');
}

function toggleHistoryDropdown() {
    $('history-dropdown').classList.toggle('show');
}

function selectHistory(ip) {
    $('ip-input').value = ip;
    $('history-dropdown').classList.remove('show');
}



function showModal(id) {
    $(id).classList.add('show');
}

function hideModal(id) {
    $(id).classList.remove('show');
}

function setLoading(btnId, loading) {
    const btn = $(btnId);
    btn.disabled = loading;
    btn.classList.toggle('loading', loading);
}

function showMessage(msg) {
    const snackbar = $('snackbar');
    snackbar.textContent = msg;
    snackbar.classList.add('show');
    setTimeout(() => snackbar.classList.remove('show'), 3000);
}

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

// 暴露到全局供 HTML onclick 使用
window.toggleAppMenu = toggleAppMenu;
window.openAppConfigModal = openAppConfigModal;
window.deleteAppConfirm = deleteAppConfirm;
window.selectDeviceApp = selectDeviceApp;
window.openPresetEditModal = openPresetEditModal;
window.deletePreset = deletePreset;
window.selectHistory = selectHistory;

// ==================== 局域网扫描 ====================

async function scanDevices() {
    console.log('点击扫描按钮');
    showMessage('正在启动扫描...');
    try {
        showModal('scan-results-modal');
    } catch (e) {
        console.error('显示弹窗失败:', e);
        showMessage('显示弹窗失败: ' + e);
        return;
    }

    const list = $('scan-results-list');
    const statusText = $('scan-status-text');
    const spinner = $('scan-spinner');

    list.innerHTML = '';
    statusText.textContent = '正在全速扫描局域网 (无需 Root)...';
    spinner.style.display = 'block';

    try {
        console.log('调用后端 invoke scan_tcp_devices');
        const devices = await invoke('scan_tcp_devices');
        console.log('扫描完成，结果:', devices);
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
        list.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--text-secondary);">暂无发现<br><small>请确保手机已连接同一 Wi-Fi 并开启了"无线调试"或已通过 USB 执行过 `adb tcpip 5555`</small></div>';
        return;
    }

    list.innerHTML = devices.map(ip => `
        <div class="device-item" onclick="selectScanDevice('${ip}')">
            <div>
                <div class="device-ip">${ip}</div>
                <div class="device-hint">端口: 5555</div>
            </div>
            <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
                <path d="M5 12h14M12 5l7 7-7 7"/>
            </svg>
        </div>
    `).join('');
}

async function selectScanDevice(ip) {
    hideModal('scan-results-modal');
    $('ip-input').value = ip + ":5555";
    showMessage(`已选择设备: ${ip}`);

    // 自动连接
    await connectWireless();
}

window.selectScanDevice = selectScanDevice;
window.scanDevices = scanDevices;

// ==================== 应用名自动补全 ====================
// 应用名输入建议
function handleAppNameInput() {
    const input = $('app-name');
    const suggestionsDiv = $('app-name-suggestions');
    const searchTerm = input.value.trim().toLowerCase();
    const searchNameOnly = $('search-name-only') ? $('search-name-only').checked : false;

    // console.log(`Autocomplete search: "${searchTerm}"`);

    if (searchTerm.length < 1) {
        suggestionsDiv.classList.add('hidden');
        return;
    }

    // 智能过滤条件
    const filterEnabled = (config.global_settings && config.global_settings.filter_installed_apps !== false);
    const shouldFilter = filterEnabled && isInstalledAppsSynced;

    let matches = [];

    // 移除最大数量限制，确保能搜索到所有匹配项
    for (const app of appDatabase) {

        const name = app.name.toLowerCase();
        const pkg = (app.package_name || '').toLowerCase();

        const matchName = name.includes(searchTerm);
        let matchPkg = false;

        if (!searchNameOnly) {
            matchPkg = pkg.includes(searchTerm);
        }

        if (matchName || matchPkg) {
            // Filter logic
            if (shouldFilter) {
                if (pkg && !installedPackages.has(app.package_name)) {
                    continue;
                }
            }

            // Calculate Score for Sorting
            let score = 0;
            // 1. Exact Name Match (Highest)
            if (name === searchTerm) score += 100;
            // 2. Exact Package Match
            else if (!searchNameOnly && pkg === searchTerm) score += 90;
            // 3. Name Starts With
            else if (name.startsWith(searchTerm)) score += 80;
            // 4. Package Starts With
            else if (!searchNameOnly && pkg.startsWith(searchTerm)) score += 70;
            // 5. Shortest Name Bonus (closer length to query is better)
            else score += (20 - Math.min(name.length - searchTerm.length, 20));

            matches.push({ app, score });
        }
    }

    // Sort by score descending
    matches.sort((a, b) => b.score - a.score);

    // Keep top 10
    const topMatches = matches.slice(0, 10).map(m => m.app);

    // console.log(`Found ${matches.length} matches, showing top ${topMatches.length}.`);

    suggestionsDiv.innerHTML = '';
    if (topMatches.length === 0) {
        suggestionsDiv.classList.add('hidden');
        return;
    }

    suggestionsDiv.classList.remove('hidden');

    topMatches.forEach(app => {
        const item = document.createElement('div');
        item.className = 'suggestion-item';
        item.innerHTML = `
            <img src="${app.icon_url || 'assets/icon_placeholder.png'}" onerror="this.src='assets/icon_placeholder.png'" alt="icon">
            <span class="name">${app.name}</span>
            <span class="pkg">${app.package_name || ''}</span>
        `;
        item.onclick = () => {
            $('app-name').value = app.name;
            if (app.package_name) {
                $('app-package').value = app.package_name;
            }
            updateIconPreview(app.package_name);
            suggestionsDiv.classList.add('hidden');
        };
        suggestionsDiv.appendChild(item);
    });
}
