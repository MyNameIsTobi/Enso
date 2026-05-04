use tauri::{AppHandle, Emitter, State};
use serde::Serialize;

use crate::AppState;
use crate::index::NodeSummary;

#[derive(Debug, Serialize)]
pub struct TrashResult {
    pub trashed: Vec<String>,
    pub failed:  Vec<FailedItem>,
}

#[derive(Debug, Serialize)]
pub struct DeleteResult {
    pub deleted: Vec<String>,
    pub failed:  Vec<FailedItem>,
}

#[derive(Debug, Serialize)]
pub struct FailedItem {
    pub path:  String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanUpdateEvent {
    pub removed_ids:        Vec<u32>,
    pub updated_ancestors:  Vec<NodeSummary>,
}

#[tauri::command]
pub async fn move_to_trash(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<TrashResult, String> {
    let mut trashed = Vec::new();
    let mut failed  = Vec::new();
    let mut removed_ids: Vec<u32> = Vec::new();

    for path_str in &paths {
        let path = std::path::Path::new(path_str);
        match crate::trash::move_to_trash(path) {
            Ok(_) => {
                trashed.push(path_str.clone());
                // Remove from index
                if let Some(&id) = state.index.path_map.get(&std::sync::Arc::new(path.to_path_buf())).as_deref() {
                    let ids = state.index.remove_nodes(&[id]);
                    removed_ids.extend(ids.iter().map(|&id| id));
                }
            }
            Err(e) => {
                failed.push(FailedItem { path: path_str.clone(), error: e.to_string() });
            }
        }
    }

    // Emit update event
    if !removed_ids.is_empty() {
        let updated_ancestors = removed_ids.iter()
            .filter_map(|&id| state.index.ancestors_summaries(id).into_iter().next())
            .collect::<Vec<_>>();

        let _ = app.emit("scan://update", ScanUpdateEvent {
            removed_ids: removed_ids.clone(),
            updated_ancestors,
        });
    }

    Ok(TrashResult { trashed, failed })
}

#[tauri::command]
pub async fn delete_permanently(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
    confirmed: bool,
) -> Result<DeleteResult, String> {
    if !confirmed {
        return Err("Deletion not confirmed".to_string());
    }

    let mut deleted = Vec::new();
    let mut failed  = Vec::new();
    let mut removed_ids: Vec<u32> = Vec::new();

    for path_str in &paths {
        let path = std::path::Path::new(path_str);
        let result = if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        };

        match result {
            Ok(_) => {
                deleted.push(path_str.clone());
                if let Some(&id) = state.index.path_map.get(&std::sync::Arc::new(path.to_path_buf())).as_deref() {
                    let ids = state.index.remove_nodes(&[id]);
                    removed_ids.extend(ids.iter().map(|&id| id));
                }
            }
            Err(e) => {
                failed.push(FailedItem { path: path_str.clone(), error: e.to_string() });
            }
        }
    }

    if !removed_ids.is_empty() {
        let updated_ancestors = removed_ids.iter()
            .filter_map(|&id| state.index.ancestors_summaries(id).into_iter().next())
            .collect::<Vec<_>>();

        let _ = app.emit("scan://update", ScanUpdateEvent {
            removed_ids: removed_ids.clone(),
            updated_ancestors,
        });
    }

    Ok(DeleteResult { deleted, failed })
}

#[tauri::command]
pub async fn open_in_file_manager(path: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    let cmd = "xdg-open";
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(target_os = "windows")]
    let cmd = "explorer";

    std::process::Command::new(cmd)
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
