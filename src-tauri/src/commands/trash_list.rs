use serde::Serialize;
use std::path::PathBuf;

use super::delete::{DeleteResult, FailedItem};

#[derive(Debug, Serialize)]
pub struct TrashEntry {
    pub name: String,
    pub original_path: String,
    pub deletion_date: String,
    pub size: u64,
    pub is_dir: bool,
}

fn trash_dirs() -> anyhow::Result<(PathBuf, PathBuf)> {
    let home = std::env::var("HOME")
        .unwrap_or_else(|_| "/tmp".to_string());
    let trash = PathBuf::from(home).join(".local/share/Trash");
    Ok((trash.join("files"), trash.join("info")))
}

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

#[tauri::command]
pub async fn list_trash() -> Result<Vec<TrashEntry>, String> {
    let (files_dir, info_dir) = trash_dirs().map_err(|e| e.to_string())?;
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

    // Sort by deletion_date descending (most recent first)
    entries.sort_by(|a, b| b.deletion_date.cmp(&a.deletion_date));
    Ok(entries)
}

fn dir_size(path: &std::path::Path) -> u64 {
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

#[tauri::command]
pub async fn empty_trash(names: Vec<String>) -> Result<DeleteResult, String> {
    let (files_dir, info_dir) = trash_dirs().map_err(|e| e.to_string())?;
    let mut deleted = Vec::new();
    let mut failed = Vec::new();

    let targets: Vec<String> = if names.is_empty() {
        // Empty everything
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
                // Try removing info anyway if file is already gone
                if !file_path.exists() {
                    let _ = std::fs::remove_file(&info_path);
                    deleted.push(name.clone());
                } else {
                    failed.push(FailedItem {
                        path: name.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }
    }

    Ok(DeleteResult { deleted, failed })
}

#[tauri::command]
pub async fn restore_from_trash(names: Vec<String>) -> Result<Vec<String>, String> {
    let (files_dir, info_dir) = trash_dirs().map_err(|e| e.to_string())?;
    let mut restored = Vec::new();

    for name in &names {
        let info_path = info_dir.join(format!("{name}.trashinfo"));
        let file_path = files_dir.join(name);

        let content = std::fs::read_to_string(&info_path)
            .map_err(|e| format!("Cannot read trashinfo for {name}: {e}"))?;

        let (original_path, _) = parse_trashinfo(&content)
            .ok_or_else(|| format!("Invalid trashinfo for {name}"))?;

        let dest = PathBuf::from(&original_path);

        // Ensure parent directory exists
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create parent dir for {original_path}: {e}"))?;
        }

        // Try rename first (same filesystem), fall back to copy+remove
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

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> anyhow::Result<()> {
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
