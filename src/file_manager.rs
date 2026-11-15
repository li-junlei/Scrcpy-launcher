//! 双栏文件管理：目录浏览与绑定设备的双向批量传输。
use crate::{adb_sync::AdbPusher, scrcpy::get_adb_path};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};
use tokio::process::Command;

/// 文件列表中的一个项目。
#[derive(Serialize)]
pub struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    is_link: bool,
    size: u64,
}

/// 当前目录及可浏览的上级目录。
#[derive(Serialize)]
pub struct Directory {
    path: String,
    parent: Option<String>,
    entries: Vec<FileEntry>,
}

fn adb(serial: Option<&str>) -> Command {
    let mut cmd = Command::new(get_adb_path());
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    if let Some(serial) = serial {
        cmd.args(["-s", serial]);
    }
    cmd
}

async fn run_adb(serial: Option<&str>, args: &[&str]) -> Result<String, String> {
    let output = adb(serial)
        .args(args)
        .output()
        .await
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn quote(path: &str) -> String {
    format!("'{}'", path.replace('\'', "'\\''"))
}
fn remote_path(path: &str) -> Result<String, String> {
    if !path.starts_with('/') || path.contains('\0') || path.split('/').any(|p| p == "..") {
        return Err("请输入不含 .. 的手机绝对路径".into());
    }
    Ok(format!(
        "/{}",
        path.split('/')
            .filter(|p| !p.is_empty() && *p != ".")
            .collect::<Vec<_>>()
            .join("/")
    ))
}
fn child(parent: &str, name: &str) -> Result<String, String> {
    if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\', '\0']) {
        return Err("无法安全传输此文件名".into());
    }
    Ok(format!("{}/{}", parent.trim_end_matches('/'), name))
}
fn sorted(mut dir: Directory) -> Directory {
    dir.entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    dir
}

/// 打开或聚焦独立文件管理窗口。
#[tauri::command]
pub async fn open_file_manager(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("file-manager") {
        window.show().map_err(|e| e.to_string())?;
        return window.set_focus().map_err(|e| e.to_string());
    }
    tauri::WebviewWindowBuilder::new(
        &app,
        "file-manager",
        tauri::WebviewUrl::App("file-manager.html".into()),
    )
    .title("文件互传 · 电脑与手机")
    .inner_size(1100.0, 720.0)
    .min_inner_size(760.0, 500.0)
    .build()
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 返回已授权设备，由文件管理窗口明确选择。
#[tauri::command]
pub async fn file_manager_devices() -> Result<Vec<String>, String> {
    Ok(run_adb(None, &["devices"])
        .await?
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let serial = parts.next()?;
            (parts.next()? == "device").then(|| serial.to_owned())
        })
        .collect())
}

/// 空路径表示电脑磁盘列表，初始路径默认用户目录。
#[tauri::command]
pub async fn list_computer_files(path: Option<String>) -> Result<Directory, String> {
    tokio::task::spawn_blocking(move || {
        let path = path.unwrap_or_else(|| {
            std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_else(|_| "/".into())
        });
        if path.is_empty() {
            #[cfg(windows)]
            let roots: Vec<PathBuf> = ('A'..='Z')
                .map(|letter| PathBuf::from(format!("{}:\\", letter)))
                .filter(|p| p.exists())
                .collect();
            #[cfg(not(windows))]
            let roots = vec![PathBuf::from("/")];
            return Ok(Directory {
                path,
                parent: None,
                entries: roots
                    .into_iter()
                    .map(|p| FileEntry {
                        name: p.display().to_string(),
                        path: p.display().to_string(),
                        is_dir: true,
                        is_link: false,
                        size: 0,
                    })
                    .collect(),
            });
        }
        let path = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            entries.push(FileEntry {
                name: entry.file_name().to_string_lossy().into_owned(),
                path: entry.path().display().to_string(),
                is_dir: kind.is_dir(),
                is_link: kind.is_symlink(),
                size: entry.metadata().map(|m| m.len()).unwrap_or(0),
            });
        }
        Ok(sorted(Directory {
            parent: Some(
                path.parent()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default(),
            ),
            path: path.display().to_string(),
            entries,
        }))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 列出手机目录；先检查可读性，避免把无权限目录误报为空目录。
#[tauri::command]
pub async fn list_phone_files(serial: String, path: String) -> Result<Directory, String> {
    let path = remote_path(&path)?;
    let q = quote(&path);
    run_adb(
        Some(&serial),
        &["shell", &format!("test -d {q} && ls -a {q} >/dev/null || {{ echo '目录不存在或无权读取' >&2; exit 1; }}")],
    )
    .await?;
    let entries = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        AdbPusher::new(Some(serial)).list(&path),
    )
    .await
    .map_err(|_| "读取手机目录超时".to_string())?
    .map_err(|e| e.to_string())?;
    let entries = entries
        .into_iter()
        .map(|(name, mode, size)| {
            let full = format!("{}/{}", path.trim_end_matches('/'), name);
            FileEntry {
                name,
                path: full,
                is_dir: mode & 0o170000 == 0o040000,
                is_link: mode & 0o170000 == 0o120000,
                size,
            }
        })
        .collect();
    Ok(sorted(Directory {
        parent: if path == "/" {
            None
        } else {
            Some(
                path.rsplit_once('/')
                    .map(|(p, _)| if p.is_empty() { "/" } else { p })
                    .unwrap()
                    .into(),
            )
        },
        path,
        entries,
    }))
}

/// 字节进度包含目录内当前文件；跳过/失败的字节不会伪装成已传输。
#[derive(Clone, Serialize, Default)]
struct TransferProgress {
    file_name: String,
    file_path: String,
    file_bytes: u64,
    file_size: u64,
    total_bytes: u64,
    total_size: u64,
    file_complete: bool,
}

struct ProgressReporter {
    progress: TransferProgress,
    on_progress: Option<Box<dyn Fn(&TransferProgress) + Send + Sync>>,
    last_emit: Instant,
}
impl ProgressReporter {
    fn emit(&mut self, force: bool) {
        if force || self.last_emit.elapsed() >= Duration::from_millis(100) {
            if let Some(callback) = &self.on_progress {
                callback(&self.progress);
            }
            self.last_emit = Instant::now();
        }
    }
    fn begin(&mut self, path: &str, size: u64) {
        self.progress.file_name = path.rsplit(['/', '\\']).next().unwrap_or(path).into();
        self.progress.file_path = path.into();
        self.progress.file_bytes = 0;
        self.progress.file_size = size;
        self.progress.file_complete = false;
        self.emit(true);
    }
    fn advance(&mut self, bytes: u64) {
        self.progress.total_bytes += bytes.saturating_sub(self.progress.file_bytes);
        self.progress.file_bytes = bytes;
        self.emit(false);
    }
    fn finish_file(&mut self) {
        self.progress.file_complete = true;
        self.emit(true);
    }
}
type SharedProgress = Arc<Mutex<ProgressReporter>>;

async fn phone_file_size(serial: &str, path: &str) -> Result<u64, String> {
    run_adb(
        Some(serial),
        &["shell", &format!("stat -c %s {}", quote(path))],
    )
    .await?
    .trim()
    .parse()
    .map_err(|_| "无法读取手机文件大小".into())
}

async fn measure_source(serial: &str, upload: bool, source: &str) -> Result<u64, String> {
    let mut total = 0u64;
    let mut pending = vec![source.to_owned()];
    while let Some(path) = pending.pop() {
        if upload {
            let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if meta.file_type().is_symlink() {
                return Err("不支持符号链接".into());
            }
            if meta.is_dir() {
                for entry in std::fs::read_dir(&path).map_err(|e| e.to_string())? {
                    pending.push(
                        entry
                            .map_err(|e| e.to_string())?
                            .path()
                            .to_str()
                            .ok_or("文件路径不是有效 Unicode")?
                            .into(),
                    );
                }
            } else if meta.is_file() {
                total += meta.len();
            } else {
                return Err("不支持此文件类型".into());
            }
        } else {
            let path = remote_path(&path)?;
            let q = quote(&path);
            let kind = run_adb(Some(serial), &["shell", &format!("if test -L {q}; then exit 1; elif test -d {q}; then echo directory; elif test -f {q}; then echo file; else exit 1; fi")]).await?;
            if kind.trim() == "directory" {
                for entry in list_phone_files(serial.into(), path).await?.entries {
                    if entry.is_link {
                        return Err("不支持符号链接".into());
                    }
                    validate_download_name(&entry.name)?;
                    pending.push(entry.path);
                }
            } else {
                total += phone_file_size(serial, &path).await?;
            }
        }
    }
    Ok(total)
}

fn file_progress(
    progress: &Option<SharedProgress>,
    path: &str,
    size: u64,
) -> Option<crate::adb_sync::ProgressCallback> {
    progress.as_ref().map(|reporter| {
        reporter.lock().unwrap().begin(path, size);
        let reporter = reporter.clone();
        Box::new(move |bytes, _| reporter.lock().unwrap().advance(bytes))
            as crate::adb_sync::ProgressCallback
    })
}

/// 每项传输前检查目标冲突，批量中单项失败继续并返回逐项结果。
#[tauri::command]
pub async fn transfer_files(
    window: tauri::Window,
    serial: String,
    upload: bool,
    paths: Vec<String>,
    destination: String,
) -> Result<Vec<String>, String> {
    if paths.is_empty() {
        return Err("请先选择文件或文件夹".into());
    }
    let destination = if upload {
        remote_path(&destination)?
    } else {
        let p = std::fs::canonicalize(&destination).map_err(|e| e.to_string())?;
        if !p.is_dir() {
            return Err("下载目标必须是目录".into());
        }
        p.display().to_string()
    };
    if upload {
        let q = quote(&destination);
        run_adb(
            Some(&serial),
            &["shell", &format!("test -d {q} && test -w {q} || {{ echo '目标目录不存在或无权写入' >&2; exit 1; }}")],
        )
        .await?;
    }
    let mut measured = Vec::new();
    let mut total_size = 0u64;
    for source in &paths {
        let size = measure_source(&serial, upload, source).await;
        if let Ok(size) = &size {
            total_size += size;
        }
        measured.push(size);
    }
    let progress_window = window.clone();
    let reporter = Arc::new(Mutex::new(ProgressReporter {
        progress: TransferProgress {
            total_size,
            ..Default::default()
        },
        on_progress: Some(Box::new(move |progress| {
            let _ = progress_window.emit("file-transfer-bytes", progress);
        })),
        last_emit: Instant::now(),
    }));
    reporter.lock().unwrap().emit(true);
    let total = paths.len();
    let mut results = Vec::new();
    for (index, source) in paths.iter().enumerate() {
        let _ = window.emit(
            "file-transfer-progress",
            serde_json::json!({"done": index, "total": total, "current": source}),
        );
        let result = match &measured[index] {
            Ok(_) => {
                transfer_with_progress(
                    &serial,
                    upload,
                    source,
                    &destination,
                    Some(reporter.clone()),
                )
                .await
            }
            Err(error) => Err(error.clone()),
        };
        results.push(match result {
            Ok(()) => format!("成功：{source}"),
            Err(e) => format!("失败/跳过：{source}\n{e}"),
        });
        let _ = window.emit(
            "file-transfer-progress",
            serde_json::json!({"done": index + 1, "total": total, "current": source}),
        );
    }
    reporter.lock().unwrap().emit(true);
    Ok(results)
}

#[cfg(test)]
async fn transfer_one(
    serial: &str,
    upload: bool,
    source: &str,
    destination: &str,
) -> Result<(), String> {
    transfer_with_progress(serial, upload, source, destination, None).await
}

async fn transfer_with_progress(
    serial: &str,
    upload: bool,
    source: &str,
    destination: &str,
    progress: Option<SharedProgress>,
) -> Result<(), String> {
    let name = if upload {
        let metadata = std::fs::symlink_metadata(source).map_err(|e| e.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("符号链接不支持传输".into());
        }
        Path::new(source)
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("文件名无效")?
            .to_owned()
    } else {
        let normalized = remote_path(source)?;
        normalized
            .rsplit('/')
            .next()
            .ok_or("文件名无效")?
            .to_owned()
    };
    child("/", &name)?;
    if upload {
        let target = child(&destination, &name)?;
        let q = quote(&target);
        run_adb(
            Some(&serial),
            &[
                "shell",
                &format!(
                    "if test -e {q} || test -L {q}; then echo '同名目标已存在，已跳过'; exit 1; fi"
                ),
            ],
        )
        .await?;
        let pusher = AdbPusher::new(Some(serial.to_owned()));
        let mut pending = vec![(PathBuf::from(source), target)];
        while let Some((local, remote)) = pending.pop() {
            let metadata = std::fs::symlink_metadata(&local).map_err(|e| e.to_string())?;
            if metadata.file_type().is_symlink() {
                return Err("文件夹内包含符号链接，不支持传输".into());
            }
            if metadata.is_dir() {
                run_adb(
                    Some(serial),
                    &["shell", &format!("mkdir -p {}", quote(&remote))],
                )
                .await?;
                for entry in std::fs::read_dir(&local).map_err(|e| e.to_string())? {
                    let entry = entry.map_err(|e| e.to_string())?;
                    let name = entry
                        .file_name()
                        .into_string()
                        .map_err(|_| "文件名不是有效 Unicode")?;
                    pending.push((entry.path(), child(&remote, &name)?));
                }
            } else if metadata.is_file() {
                pusher
                    .push(
                        local.to_str().ok_or("文件路径不是有效 Unicode")?,
                        &format!("{remote},33188"),
                        file_progress(
                            &progress,
                            local.to_str().ok_or("文件路径不是有效 Unicode")?,
                            metadata.len(),
                        ),
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                if let Some(reporter) = &progress {
                    reporter.lock().unwrap().finish_file();
                }
            } else {
                return Err("不支持此文件类型".into());
            }
        }
    } else {
        validate_download_name(&name)?;
        let target = Path::new(destination).join(&name);
        if target.symlink_metadata().is_ok() {
            return Err("同名目标已存在，已跳过".into());
        }
        let source = remote_path(source)?;
        let q = quote(&source);
        let kind = run_adb(Some(serial), &["shell", &format!("if test -L {q}; then exit 1; elif test -d {q}; then echo directory; elif test -f {q}; then echo file; else echo '文件不存在或无权读取' >&2; exit 1; fi")]).await?;
        let pusher = AdbPusher::new(Some(serial.to_owned()));
        let mut pending = vec![(source, target, kind.trim() == "directory")];
        while let Some((remote, local, is_dir)) = pending.pop() {
            if is_dir {
                std::fs::create_dir(&local).map_err(|e| e.to_string())?;
                let directory = list_phone_files(serial.to_owned(), remote).await?;
                for entry in directory.entries {
                    validate_download_name(&entry.name)?;
                    if entry.is_link {
                        return Err("文件夹内包含符号链接，不支持传输".into());
                    }
                    pending.push((entry.path, local.join(entry.name), entry.is_dir));
                }
            } else {
                let size = phone_file_size(serial, &remote).await?;
                pusher
                    .pull(&remote, &local, file_progress(&progress, &remote, size))
                    .await
                    .map_err(|e| e.to_string())?;
                if let Some(reporter) = &progress {
                    reporter.lock().unwrap().finish_file();
                }
            }
        }
    }
    Ok(())
}

fn validate_download_name(name: &str) -> Result<(), String> {
    child("/", name)?;
    let stem = name.split('.').next().unwrap_or("").to_uppercase();
    if name.contains([':', '*', '?', '"', '<', '>', '|'])
        || name.chars().any(|c| c < ' ')
        || name.ends_with(['.', ' '])
        || ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
    {
        return Err(format!("文件名不兼容电脑文件系统：{name}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn device_paths_and_shell_quotes() {
        assert_eq!(
            remote_path("/sdcard//Download/.").unwrap(),
            "/sdcard/Download"
        );
        assert!(remote_path("/sdcard/../data").is_err());
        assert!(remote_path("relative").is_err());
        assert_eq!(quote("a'b $x"), "'a'\\''b $x'");
        assert!(child("/sdcard", "../x").is_err());
        assert!(child("/sdcard", "a\\b").is_err());
        assert_eq!(child("/", "文件.txt").unwrap(), "/文件.txt");
        assert!(validate_download_name("CON.txt").is_err());
        assert!(validate_download_name("LPT1").is_err());
        assert!(validate_download_name("a:b").is_err());
        assert!(validate_download_name("a\\b").is_err());
        assert!(validate_download_name("引用'$文件.txt").is_ok());
    }
    #[test]
    fn byte_progress_accumulates_across_files() {
        let mut reporter = ProgressReporter {
            progress: TransferProgress {
                total_size: 300,
                ..Default::default()
            },
            on_progress: None,
            last_emit: Instant::now(),
        };
        reporter.begin("/folder/first.bin", 100);
        reporter.advance(40);
        reporter.advance(100);
        reporter.finish_file();
        assert_eq!(reporter.progress.total_bytes, 100);
        reporter.begin("/folder/第二个.bin", 200);
        reporter.advance(50);
        assert_eq!(reporter.progress.file_bytes, 50);
        assert_eq!(reporter.progress.total_bytes, 150);
        assert_eq!(reporter.progress.file_name, "第二个.bin");
        reporter.advance(200);
        reporter.finish_file();
        assert_eq!(reporter.progress.total_bytes, 300);
        assert!(reporter.progress.file_complete);
        reporter.begin("/empty", 0);
        reporter.finish_file();
        assert_eq!(reporter.progress.total_bytes, 300);
        assert!(reporter.progress.file_complete);
    }
    #[tokio::test]
    #[ignore = "requires SCRCPY_TEST_DEVICE and writes only a unique temporary test directory"]
    async fn device_roundtrip() {
        let serial = std::env::var("SCRCPY_TEST_DEVICE").expect("set SCRCPY_TEST_DEVICE");
        let id = format!(
            "scrcpy-transfer-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let temp = std::env::temp_dir().join(&id);
        let source = temp.join(&id);
        let nested = source.join("中文 空格");
        std::fs::create_dir_all(nested.join("empty")).unwrap();
        std::fs::write(nested.join("test.txt"), b"scrcpy transfer roundtrip").unwrap();
        let binary: Vec<u8> = (0..200_000).map(|i| (i % 251) as u8).collect();
        std::fs::write(nested.join("binary.bin"), &binary).unwrap();
        std::fs::write(source.join("引用'$文件.txt"), b"quoted filename").unwrap();
        std::fs::create_dir_all(temp.join("received")).unwrap();
        let remote = format!("/sdcard/Download/{id}");
        let result: Result<(), String> = async {
            let total_size = measure_source(&serial, true, source.to_str().unwrap()).await?;
            let progress = Arc::new(Mutex::new(ProgressReporter {
                progress: TransferProgress {
                    total_size,
                    ..Default::default()
                },
                on_progress: None,
                last_emit: Instant::now(),
            }));
            transfer_with_progress(
                &serial,
                true,
                source.to_str().unwrap(),
                "/sdcard/Download",
                Some(progress.clone()),
            )
            .await?;
            assert_eq!(progress.lock().unwrap().progress.total_bytes, total_size);
            assert_eq!(measure_source(&serial, false, &remote).await?, total_size);
            let listing = list_phone_files(serial.clone(), remote.clone()).await?;
            assert!(listing
                .entries
                .iter()
                .any(|e| e.name == "中文 空格" && e.is_dir));
            assert!(
                transfer_one(&serial, true, source.to_str().unwrap(), "/sdcard/Download")
                    .await
                    .is_err()
            );
            let received = std::fs::canonicalize(temp.join("received")).unwrap();
            let progress = Arc::new(Mutex::new(ProgressReporter {
                progress: TransferProgress {
                    total_size,
                    ..Default::default()
                },
                on_progress: None,
                last_emit: Instant::now(),
            }));
            transfer_with_progress(
                &serial,
                false,
                &remote,
                received.to_str().unwrap(),
                Some(progress.clone()),
            )
            .await?;
            assert_eq!(progress.lock().unwrap().progress.total_bytes, total_size);
            assert_eq!(
                std::fs::read(received.join(&id).join("中文 空格/test.txt")).unwrap(),
                b"scrcpy transfer roundtrip"
            );
            assert!(received.join(&id).join("中文 空格/empty").is_dir());
            assert_eq!(
                std::fs::read(received.join(&id).join("中文 空格/binary.bin")).unwrap(),
                binary
            );
            assert_eq!(
                std::fs::read(received.join(&id).join("引用'$文件.txt")).unwrap(),
                b"quoted filename"
            );
            let single = temp.join("single '文件.bin");
            std::fs::write(&single, &binary).unwrap();
            transfer_one(&serial, true, single.to_str().unwrap(), &remote).await?;
            transfer_one(
                &serial,
                false,
                &format!("{remote}/single '文件.bin"),
                received.to_str().unwrap(),
            )
            .await?;
            assert_eq!(
                std::fs::read(received.join("single '文件.bin")).unwrap(),
                binary
            );
            assert!(
                transfer_one(&serial, false, &remote, received.to_str().unwrap())
                    .await
                    .is_err()
            );
            Ok(())
        }
        .await;
        // Only remove the exact unique directories created by this test.
        assert!(remote.starts_with("/sdcard/Download/scrcpy-transfer-test-"));
        let cleanup = run_adb(
            Some(&serial),
            &["shell", &format!("rm -rf {}", quote(&remote))],
        )
        .await;
        assert_eq!(temp.parent(), Some(std::env::temp_dir().as_path()));
        std::fs::remove_dir_all(&temp).unwrap();
        result.unwrap();
        cleanup.unwrap();
    }
}
