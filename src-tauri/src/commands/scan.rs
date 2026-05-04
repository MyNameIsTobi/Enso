use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::PathBuf;

use tauri::{AppHandle, State};

use crate::AppState;

#[tauri::command]
pub async fn start_scan(
    app: AppHandle,
    state: State<'_, AppState>,
    root_path: String,
) -> Result<(), String> {
    // Cancel any ongoing scan and replace the handle atomically
    let cancelled = Arc::new(AtomicBool::new(false));
    {
        let mut handle = state.cancelled_handle.lock().await;
        handle.store(true, Ordering::Relaxed); // stop previous scan
        *handle = cancelled.clone();           // install new cancelled flag
    }

    let index = state.index.clone();
    let path  = PathBuf::from(&root_path);

    // Spawn the scan task (non-blocking: returns immediately)
    tokio::spawn(async move {
        if let Err(e) = crate::scanner::scan(app, path, index, cancelled).await {
            eprintln!("Scan error: {e}");
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn cancel_scan(state: State<'_, AppState>) -> Result<(), String> {
    let handle = state.cancelled_handle.lock().await;
    handle.store(true, Ordering::Relaxed);
    Ok(())
}
