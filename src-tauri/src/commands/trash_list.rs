use serde::Serialize;
use std::path::{Path, PathBuf};

use super::delete::{DeleteResult, FailedItem};

#[derive(Debug, Serialize)]
pub struct TrashEntry {
    pub name: String,
    pub original_path: String,
    pub deletion_date: String,
    pub size: u64,
    pub is_dir: bool,
}

// ────────────────────────────────────────────────────────────────────────────
// Platform-specific trash dir resolution
// ────────────────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn linux_trash_dirs() -> (PathBuf, PathBuf) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let trash = PathBuf::from(home).join(".local/share/Trash");
    (trash.join("files"), trash.join("info"))
}

#[cfg(target_os = "macos")]
fn macos_trash_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".Trash")
}

// ────────────────────────────────────────────────────────────────────────────
// list_trash
// ────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_trash() -> Result<Vec<TrashEntry>, String> {
    #[cfg(target_os = "linux")]
    { list_trash_linux() }
    #[cfg(target_os = "macos")]
    { list_trash_macos() }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    { Ok(Vec::new()) }
}

#[cfg(target_os = "linux")]
fn list_trash_linux() -> Result<Vec<TrashEntry>, String> {
    let (files_dir, info_dir) = linux_trash_dirs();
    let mut entries = Vec::new();

    let info_entries = match std::fs::read_dir(&info_dir) {
        Ok(rd) => rd,
        Err(_) => return Ok(entries),
    };

    for entry in info_entries.flatten() {
        let info_name = entry.file_name().to_string_lossy().into_owned();
        let Some(name) = info_name.strip_suffix(".trashinfo") else { continue };

        let content = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let Some((original_path, deletion_date)) = parse_trashinfo(&content) else { continue };

        let file_path = files_dir.join(name);
        let (size, is_dir) = match file_path.metadata() {
            Ok(m) => {
                if m.is_dir() {
                    (dir_size(&file_path), true)
                } else {
                    (m.len(), false)
                }
            }
            Err(_) => (0, false),
        };

        entries.push(TrashEntry {
            name: name.to_string(),
            original_path,
            deletion_date,
            size,
            is_dir,
        });
    }

    entries.sort_by(|a, b| b.deletion_date.cmp(&a.deletion_date));
    Ok(entries)
}

#[cfg(target_os = "linux")]
fn parse_trashinfo(content: &str) -> Option<(String, String)> {
    let mut path = None;
    let mut date = None;
    for line in content.lines() {
        if let Some(p) = line.strip_prefix("Path=") {
            path = Some(p.to_string());
        } else if let Some(d) = line.strip_prefix("DeletionDate=") {
            date = Some(d.to_string());
        }
    }
    Some((path?, date.unwrap_or_default()))
}

#[cfg(target_os = "macos")]
fn list_trash_macos() -> Result<Vec<TrashEntry>, String> {
    let trash = macos_trash_dir();
    let mut entries = Vec::new();

    let rd = match std::fs::read_dir(&trash) {
        Ok(rd) => rd,
        Err(_) => return Ok(entries),
    };

    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') { continue; } // .DS_Store etc.

        let path = e.path();
        let meta = match e.metadata() { Ok(m) => m, Err(_) => continue };
        let is_dir = meta.is_dir();
        let size = if is_dir { dir_size(&path) } else { meta.len() };
        let deletion_date = ctime_iso(&meta).unwrap_or_default();

        entries.push(TrashEntry {
            name,
            original_path: String::new(), // macOS keeps origin in Finder-internal plist
            deletion_date,
            size,
            is_dir,
        });
    }

    entries.sort_by(|a, b| b.deletion_date.cmp(&a.deletion_date));
    Ok(entries)
}

#[cfg(target_os = "macos")]
fn ctime_iso(meta: &std::fs::Metadata) -> Option<String> {
    use std::os::unix::fs::MetadataExt;
    use chrono::{Local, TimeZone};
    let dt = Local.timestamp_opt(meta.ctime(), 0).single()?;
    Some(dt.format("%Y-%m-%dT%H:%M:%S").to_string())
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(rd) = std::fs::read_dir(path) {
        for entry in rd.flatten() {
            let ft = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };
            if ft.is_dir() {
                total += dir_size(&entry.path());
            } else {
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    total
}

// ────────────────────────────────────────────────────────────────────────────
// empty_trash
// ────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn empty_trash(names: Vec<String>) -> Result<DeleteResult, String> {
    #[cfg(target_os = "linux")]
    { empty_trash_linux(names) }
    #[cfg(target_os = "macos")]
    { empty_trash_macos(names) }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    { Ok(DeleteResult { deleted: vec![], failed: vec![] }) }
}

#[cfg(target_os = "linux")]
fn empty_trash_linux(names: Vec<String>) -> Result<DeleteResult, String> {
    let (files_dir, info_dir) = linux_trash_dirs();
    let mut deleted = Vec::new();
    let mut failed = Vec::new();

    let targets: Vec<String> = if names.is_empty() {
        match std::fs::read_dir(&info_dir) {
            Ok(rd) => rd.flatten()
                .filter_map(|e| {
                    let n = e.file_name().to_string_lossy().into_owned();
                    n.strip_suffix(".trashinfo").map(|s| s.to_string())
                })
                .collect(),
            Err(e) => return Err(e.to_string()),
        }
    } else {
        names
    };

    for name in &targets {
        let file_path = files_dir.join(name);
        let info_path = info_dir.join(format!("{name}.trashinfo"));

        let file_result = if file_path.is_dir() {
            std::fs::remove_dir_all(&file_path)
        } else {
            std::fs::remove_file(&file_path)
        };

        match file_result {
            Ok(_) => {
                let _ = std::fs::remove_file(&info_path);
                deleted.push(name.clone());
            }
            Err(e) => {
                if !file_path.exists() {
                    let _ = std::fs::remove_file(&info_path);
                    deleted.push(name.clone());
                } else {
                    failed.push(FailedItem { path: name.clone(), error: e.to_string() });
                }
            }
        }
    }

    Ok(DeleteResult { deleted, failed })
}

#[cfg(target_os = "macos")]
fn empty_trash_macos(names: Vec<String>) -> Result<DeleteResult, String> {
    let trash = macos_trash_dir();
    let mut deleted = Vec::new();
    let mut failed = Vec::new();

    let targets: Vec<String> = if names.is_empty() {
        match std::fs::read_dir(&trash) {
            Ok(rd) => rd.flatten()
                .filter_map(|e| {
                    let n = e.file_name().to_string_lossy().into_owned();
                    if n.starts_with('.') { None } else { Some(n) }
                })
                .collect(),
            Err(e) => return Err(e.to_string()),
        }
    } else {
        names
    };

    for name in &targets {
        let path = trash.join(name);
        let result = if path.is_dir() {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };
        match result {
            Ok(_) => deleted.push(name.clone()),
            Err(e) => failed.push(FailedItem { path: name.clone(), error: e.to_string() }),
        }
    }

    Ok(DeleteResult { deleted, failed })
}

// ────────────────────────────────────────────────────────────────────────────
// restore_from_trash
// ────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn restore_from_trash(names: Vec<String>) -> Result<Vec<String>, String> {
    #[cfg(target_os = "linux")]
    { restore_from_trash_linux(names) }
    #[cfg(target_os = "macos")]
    { restore_from_trash_macos(names) }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    { Err("restore not supported on this platform".to_string()) }
}

#[cfg(target_os = "linux")]
fn restore_from_trash_linux(names: Vec<String>) -> Result<Vec<String>, String> {
    let (files_dir, info_dir) = linux_trash_dirs();
    let mut restored = Vec::new();

    for name in &names {
        let info_path = info_dir.join(format!("{name}.trashinfo"));
        let file_path = files_dir.join(name);

        let content = std::fs::read_to_string(&info_path)
            .map_err(|e| format!("Cannot read trashinfo for {name}: {e}"))?;

        let (original_path, _) = parse_trashinfo(&content)
            .ok_or_else(|| format!("Invalid trashinfo for {name}"))?;

        let dest = PathBuf::from(&original_path);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create parent dir for {original_path}: {e}"))?;
        }

        match std::fs::rename(&file_path, &dest) {
            Ok(_) => {}
            Err(e) if e.raw_os_error() == Some(libc::EXDEV) => {
                if file_path.is_dir() {
                    copy_dir_all(&file_path, &dest)
                        .map_err(|e| format!("Failed to copy {name} back: {e}"))?;
                    std::fs::remove_dir_all(&file_path)
                        .map_err(|e| format!("Failed to remove trash copy of {name}: {e}"))?;
                } else {
                    std::fs::copy(&file_path, &dest)
                        .map_err(|e| format!("Failed to copy {name} back: {e}"))?;
                    std::fs::remove_file(&file_path)
                        .map_err(|e| format!("Failed to remove trash copy of {name}: {e}"))?;
                }
            }
            Err(e) => return Err(format!("Failed to restore {name}: {e}")),
        }

        let _ = std::fs::remove_file(&info_path);
        restored.push(original_path);
    }

    Ok(restored)
}

#[cfg(target_os = "macos")]
fn restore_from_trash_macos(names: Vec<String>) -> Result<Vec<String>, String> {
    // macOS keeps the original location in Finder-internal metadata, accessible
    // via the Finder "put back" command. Shell out to osascript.
    let mut restored = Vec::new();

    for name in &names {
        let escaped = name.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            r#"tell application "Finder" to put back (every item of trash whose name is "{}")"#,
            escaped
        );
        let out = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| format!("osascript invocation failed: {e}"))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(format!("Restore failed for {name}: {}", stderr.trim()));
        }
        restored.push(name.clone());
    }

    Ok(restored)
}

#[cfg(target_os = "linux")]
fn copy_dir_all(src: &Path, dst: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dest = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        } else {
            std::fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}
