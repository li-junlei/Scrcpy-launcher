/**
 * Scrcpy Launcher - 前端主逻辑
 * 使用 Tauri API 与 Rust 后端通信
 */

const { invoke, convertFileSrc } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;
const { open } = window.__TAURI__.dialog;

// 全局状态
let config = null;
let isSortingMode = false;
let editingAppPackage = null;
let editingPresetName = null;
let deviceApps = [];


// DOM 元素缓存
const $ = (id) => document.getElementById(id);

// 初始化
document.addEventListener('DOMContentLoaded', async () => {
    await loadConfig();
    setupEventListeners();
    initUI();
});

// 加载配置
async function loadConfig() {
    try {
        config = await invoke('get_config');
        applyTheme(config.global_settings.theme);
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
        loadHistoryDropdown();
        if (config.adb_history.length > 0) {
            $('ip-input').value = config.adb_history[0];
        }
    }
}

// 设置事件监听器
function setupEventListeners() {
    // ADB 区域
    $('refresh-btn').onclick = checkAdbStatus;
    $('connect-btn').onclick = connectWireless;
    $('history-btn').onclick = toggleHistoryDropdown;
    $('tcpip-btn').onclick = enableTcpip;
    $('disconnect-btn').onclick = disconnectAll;
    $('kill-btn').onclick = killScrcpy;
    $('push-file-btn').onclick = pushFile;

    // 标签页
    document.querySelectorAll('.tab').forEach(tab => {
        tab.onclick = () => switchTab(tab.dataset.tab);
    });

    // 直接投屏
    $('settings-btn').onclick = () => openSettingsModal();
    $('advanced-btn').onclick = openAdvancedModal;
    $('theme-toggle-btn').onclick = toggleTheme;
    $('mirror-btn').onclick = launchMirror;
    $('audio-btn').onclick = launchAudio;

    // 应用流转
    $('add-app-btn').onclick = () => openAppConfigModal();
    $('sort-btn').onclick = toggleSortMode;

    // 首次运行
    $('save-first-run-btn').onclick = saveFirstRunConfig;

    // 设置
    $('settings-cancel-btn').onclick = () => hideModal('settings-modal');
    $('settings-save-btn').onclick = saveSettings;

    // 高级设置
    $('use-custom-args').onchange = toggleCustomArgsMode;
    $('use-app-stream-args').onchange = toggleAppStreamArgs;
    $('use-app-custom-args').onchange = toggleAppCustomArgsMode;
    $('advanced-defaults-btn').onclick = restoreDefaults;
    $('advanced-cancel-btn').onclick = () => hideModal('advanced-modal');
    $('advanced-save-btn').onclick = saveAdvancedSettings;

    // 应用配置
    $('browse-apps-btn').onclick = () => showModal('browse-apps-modal');
    $('manage-presets-btn').onclick = () => openPresetsModal();
    $('preset-select').onchange = applyPreset;
    $('use-custom-res').onchange = toggleResolutionFields;
    $('use-app-scrcpy-args').onchange = toggleAppScrcpyArgs;
    $('app-cancel-btn').onclick = () => hideModal('app-config-modal');

    $('app-save-btn').onclick = saveApp;

    // 浏览应用
    $('browse-back-btn').onclick = () => hideModal('browse-apps-modal');
    $('load-apps-btn').onclick = loadDeviceApps;
    $('app-search').oninput = filterDeviceApps;

    // 预设管理
    $('add-preset-btn').onclick = () => openPresetEditModal();
    $('presets-close-btn').onclick = () => hideModal('presets-modal');

    // 编辑预设
    $('preset-cancel-btn').onclick = () => hideModal('preset-edit-modal');
    $('preset-save-btn').onclick = savePreset;

    // 点击外部关闭历史下拉
    document.addEventListener('click', (e) => {
        if (!e.target.closest('#history-btn') && !e.target.closest('#history-dropdown')) {
            $('history-dropdown').classList.remove('show');
        }
    });

    // 监听进度事件
    listen('adb-push-progress', (event) => {
        const { progress, message } = event.payload;
        updateProgressBar(progress, message);
    });
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
    } catch (e) {
        $('status-dot').classList.remove('connected');
        $('status-text').textContent = '检查失败';
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
            // 创建内容 - 仅文字
            card.innerHTML = `
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
                    deleteApp(pkg);
                };
            }
        }

        grid.appendChild(card);
    });
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
    document.querySelectorAll('.app-menu').forEach(m => m.classList.remove('show'));
    $(`menu-${pkg}`).classList.toggle('show');
}

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

async function deleteApp(pkg) {
    if (!confirm(`确定要删除 "${config.apps[pkg]?.name || pkg}" 吗？`)) return;

    try {
        await invoke('delete_app', { package: pkg });
        await loadConfig();
        renderApps();
        showMessage('已删除');
    } catch (e) {
        showMessage(`删除失败: ${e}`);
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

    // 自定义参数
    $('use-app-scrcpy-args').checked = app.scrcpy_args !== null && app.scrcpy_args !== undefined;
    $('app-scrcpy-args').value = app.scrcpy_args || config.custom_args || '';
    toggleAppScrcpyArgs();

    showModal('app-config-modal');
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
    $('tray-count').value = config.tray_app_count;
    $('tray-mirror').checked = config.tray_show_mirror;
    $('tray-audio').checked = config.tray_show_audio;
    showModal('settings-modal');
}

async function saveSettings() {
    const dpi = parseInt($('global-dpi').value) || 400;
    const fullRes = $('global-res').value || '1200x2670';
    const trayCount = parseInt($('tray-count').value) || 4;
    const showMirror = $('tray-mirror').checked;
    const showAudio = $('tray-audio').checked;

    try {
        await invoke('save_global_settings', { dpi, fullRes });
        await invoke('save_tray_settings', { appCount: trayCount, showMirror, showAudio });
        await loadConfig();
        hideModal('settings-modal');
        showMessage('设置已保存（托盘菜单将在重启后生效）');
    } catch (e) {
        showMessage(`保存失败: ${e}`);
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
window.deleteApp = deleteApp;
window.selectDeviceApp = selectDeviceApp;
window.openPresetEditModal = openPresetEditModal;
window.deletePreset = deletePreset;
window.selectHistory = selectHistory;
