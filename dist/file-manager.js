/* 独立窗口中的双栏文件管理，不依赖主界面的状态。 */
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = id => document.getElementById(id);
let busy = false;
const panes = {};
for (const side of ['computer', 'phone']) {
    const element = document.querySelector(`[data-side="${side}"]`);
    const pane = panes[side] = { element, directory: null, selected: new Set(), loading: false, request: 0 };
    element.querySelector('form').onsubmit = event => {
        event.preventDefault();
        loadDirectory(side, element.querySelector('.path-bar input').value);
    };
    element.querySelector('.up').onclick = () => loadDirectory(side, pane.directory.parent);
    element.querySelector('.root').onclick = () => loadDirectory(side, side === 'computer' ? '' : '/');
    element.querySelector('.refresh').onclick = () => loadDirectory(side, pane.directory?.path);
    element.querySelector('.select-all').onchange = event => {
        pane.selected.clear();
        if (event.target.checked) pane.directory?.entries.filter(entry => !entry.is_link).forEach(entry => pane.selected.add(entry.path));
        render(side);
    };
    element.querySelector('.transfer').onclick = () => transfer(side);
}

function updateControls() {
    for (const [side, pane] of Object.entries(panes)) {
        pane.element.querySelectorAll('button, input').forEach(control => control.disabled = busy || pane.loading);
        pane.element.querySelector('.up').disabled = busy || pane.loading || pane.directory?.parent == null;
        pane.element.querySelector('.transfer').disabled = busy || !pane.selected.size || Object.values(panes).some(p => p.loading || !p.directory) || !$('device').value || !panes.computer.directory?.path;
        pane.element.querySelector('.selection').textContent = `已选择 ${pane.selected.size} 项`;
        pane.element.querySelectorAll('.file-row input').forEach((control, index) => {
            if (pane.directory.entries[index].is_link) control.disabled = true;
        });
    }
    $('device').disabled = busy || panes.phone.loading;
    $('devices-refresh').disabled = busy || panes.phone.loading;
}

function sizeText(size) {
    if (size < 1024) return `${size} B`;
    if (size < 1024 ** 2) return `${(size / 1024).toFixed(1)} KB`;
    if (size < 1024 ** 3) return `${(size / 1024 ** 2).toFixed(1)} MB`;
    return `${(size / 1024 ** 3).toFixed(1)} GB`;
}

// 使用约一秒的滑动窗口，停传时速率也会回落到零。
class TransferSpeed {
    constructor() { this.reset(); }
    reset(now = performance.now(), bytes = 0) { this.samples = [{ bytes, time: now }]; }
    sample(bytes, now = performance.now()) {
        if (bytes < this.samples[this.samples.length - 1].bytes) this.reset(now);
        this.samples.push({ bytes, time: now });
        while (this.samples.length > 2 && this.samples[1].time <= now - 1000) this.samples.shift();
        const first = this.samples[0];
        return now > first.time ? (bytes - first.bytes) * 1000 / (now - first.time) : 0;
    }
}
const fileSpeed = new TransferSpeed();
const totalSpeed = new TransferSpeed();
let speedTimer = null;
let speedBytes = { file: 0, total: 0, path: '' };

function renderTransferSpeed() {
    const now = performance.now();
    $('file-speed').textContent = `${sizeText(fileSpeed.sample(speedBytes.file, now))}/s`;
    $('total-speed').textContent = `${sizeText(totalSpeed.sample(speedBytes.total, now))}/s`;
}

function render(side) {
    const pane = panes[side];
    const list = pane.element.querySelector('.file-list');
    list.replaceChildren();
    for (const entry of pane.directory?.entries || []) {
        const row = document.createElement('div');
        row.className = 'file-row';
        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.checked = pane.selected.has(entry.path);
        checkbox.setAttribute('aria-label', `选择 ${entry.name}`);
        checkbox.onchange = () => {
            if (checkbox.checked) pane.selected.add(entry.path); else pane.selected.delete(entry.path);
            updateSelection(side);
        };
        const name = document.createElement('button');
        name.className = 'file-name';
        name.textContent = `${entry.is_link ? '↗' : entry.is_dir ? '📁' : '📄'} ${entry.name}`;
        name.title = entry.is_link ? '符号链接不支持传输；目录链接可通过路径栏浏览' : entry.path;
        name.onclick = () => { if (!entry.is_link) { checkbox.checked = !checkbox.checked; checkbox.onchange(); } };
        name.ondblclick = () => { if (entry.is_dir || entry.is_link) loadDirectory(side, entry.path); };
        name.onkeydown = event => { if (event.key === 'Enter' && (entry.is_dir || entry.is_link)) { event.preventDefault(); loadDirectory(side, entry.path); } };
        const size = document.createElement('span');
        size.className = 'file-size';
        size.textContent = entry.is_link ? '链接' : entry.is_dir ? '文件夹' : sizeText(entry.size);
        row.append(checkbox, name, size);
        list.append(row);
    }
    updateSelection(side);
}

function updateSelection(side) {
    const pane = panes[side];
    const count = pane.directory?.entries.filter(entry => !entry.is_link).length || 0;
    const all = pane.element.querySelector('.select-all');
    all.checked = count > 0 && pane.selected.size === count;
    all.indeterminate = pane.selected.size > 0 && pane.selected.size < count;
    updateControls();
}

async function loadDirectory(side, path) {
    if (busy) return;
    const pane = panes[side];
    const request = ++pane.request;
    const status = pane.element.querySelector('.pane-status');
    pane.loading = true;
    status.classList.remove('error');
    status.textContent = '正在读取目录…';
    updateControls();
    try {
        if (side === 'phone' && !$('device').value) throw new Error('未发现已授权手机，请连接设备后刷新');
        const directory = await invoke(side === 'computer' ? 'list_computer_files' : 'list_phone_files', {
            path: path ?? (side === 'phone' ? '/sdcard' : null), serial: $('device').value,
        });
        if (request !== pane.request) return;
        pane.directory = directory;
        pane.selected.clear();
        pane.element.querySelector('.path-bar input').value = directory.path;
        status.textContent = directory.entries.length ? `${directory.entries.length} 项` : '此目录为空';
        render(side);
    } catch (error) {
        if (request !== pane.request) return;
        status.classList.add('error');
        status.textContent = `无法读取：${error}`;
        // 旧列表保留时恢复旧路径，防止把传输目标误认为失败的新路径。
        pane.element.querySelector('.path-bar input').value = pane.directory?.path || '';
    } finally {
        if (request === pane.request) { pane.loading = false; updateControls(); }
    }
}

async function refreshDevices() {
    try {
        const previous = $('device').value;
        const devices = await invoke('file_manager_devices');
        $('device').replaceChildren(...devices.map(serial => new Option(serial, serial)));
        if (devices.includes(previous)) $('device').value = previous;
        if (!devices.length) $('device').append(new Option('无已授权设备', ''));
        panes.phone.directory = null;
        panes.phone.selected.clear();
        render('phone');
        await loadDirectory('phone', '/sdcard');
    } catch (error) { $('transfer-message').textContent = `刷新设备失败：${error}`; }
}

async function transfer(side) {
    if (busy || Object.values(panes).some(p => p.loading || !p.directory)) return;
    const source = panes[side];
    const target = panes[side === 'computer' ? 'phone' : 'computer'];
    const paths = [...source.selected];
    if (!paths.length) return;
    busy = true;
    updateControls();
    $('transfer-results').textContent = '';
    $('transfer-message').textContent = '正在统计文件及总大小…';
    $('current-file').textContent = '当前文件：—';
    for (const id of ['file-progress', 'transfer-progress']) { $(id).value = 0; $(id).max = 100; }
    $('file-progress-text').textContent = '0%';
    $('total-progress-text').textContent = '正在统计…';
    speedBytes = { file: 0, total: 0, path: '' };
    fileSpeed.reset();
    totalSpeed.reset();
    renderTransferSpeed();
    speedTimer = setInterval(renderTransferSpeed, 250);
    try {
        const results = await invoke('transfer_files', { serial: $('device').value, upload: side === 'computer', paths, destination: target.directory.path });
        $('transfer-results').textContent = results.join('\n');
        $('transfer-message').textContent = `批量处理结束：${results.filter(result => result.startsWith('成功：')).length} 项成功，共 ${paths.length} 项。失败项可能留有不完整文件，请查看结果。`;
        if (results.every(result => result.startsWith('成功：'))) {
            $('transfer-progress').value = 100;
            if ($('total-progress-text').textContent.startsWith('0.0%')) $('total-progress-text').textContent = '100% · 0 B / 0 B';
        }
        source.selected.clear();
    } catch (error) {
        $('transfer-message').textContent = `传输失败：${error}`;
    } finally {
        clearInterval(speedTimer);
        speedTimer = null;
        $('file-speed').textContent = '0 B/s';
        $('total-speed').textContent = '0 B/s';
        busy = false;
        await Promise.all(Object.keys(panes).map(side => loadDirectory(side, panes[side].directory?.path)));
        updateControls();
    }
}

function renderTransferProgress(payload) {
        if (payload.file_path !== speedBytes.path) {
            fileSpeed.reset(performance.now(), payload.file_bytes);
            speedBytes.path = payload.file_path;
        }
        speedBytes.file = payload.file_bytes;
        speedBytes.total = payload.total_bytes;
        renderTransferSpeed();
        const filePercent = payload.file_size ? Math.min(100, payload.file_bytes / payload.file_size * 100) : (payload.file_complete ? 100 : 0);
        const totalPercent = payload.total_size ? Math.min(100, payload.total_bytes / payload.total_size * 100) : 0;
        $('file-progress').value = filePercent;
        $('transfer-progress').value = totalPercent;
        $('file-progress-text').textContent = `${filePercent.toFixed(1)}% · ${sizeText(payload.file_bytes)} / ${sizeText(payload.file_size)}`;
        $('total-progress-text').textContent = `${totalPercent.toFixed(1)}% · ${sizeText(payload.total_bytes)} / ${sizeText(payload.total_size)}`;
        $('current-file').textContent = `当前文件：${payload.file_name || '—'}`;
        $('current-file').title = payload.file_path;
}

$('device').onchange = () => {
    panes.phone.directory = null;
    panes.phone.selected.clear();
    render('phone');
    loadDirectory('phone', '/sdcard');
};
$('devices-refresh').onclick = refreshDevices;
function handleFileManagerClose(event) {
    if (busy) {
        event.preventDefault();
        $('transfer-message').textContent = '传输正在进行，请等待处理结束后关闭窗口。';
    }
}

(async () => {
    await window.__TAURI__.window.getCurrentWindow().onCloseRequested(handleFileManagerClose);
    const config = await invoke('get_config').catch(() => null);
    if (config?.theme === 'dark') document.documentElement.dataset.theme = 'dark';
    await listen('file-transfer-bytes', ({ payload }) => { if (busy) renderTransferProgress(payload); });
    await listen('file-transfer-progress', ({ payload }) => {
        if (!busy) return;
        $('transfer-message').textContent = `已处理 ${payload.done}/${payload.total} 项 · ${payload.current}`;
    });
    await Promise.all([loadDirectory('computer'), refreshDevices()]);
})();
