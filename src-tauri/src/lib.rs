mod index;
mod scanner;
mod commands;
mod trash;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;

use crate::index::FileIndex;

pub struct AppState {
    pub index:            Arc<FileIndex>,
    pub cancelled_handle: Arc<Mutex<Arc<AtomicBool>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            index:            Arc::new(FileIndex::new()),
            cancelled_handle: Arc::new(Mutex::new(Arc::new(AtomicBool::new(false)))),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1"); }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::scan::start_scan,
            commands::scan::cancel_scan,
            commands::query::get_children,
            commands::query::get_node,
            commands::query::get_storage_info,
            commands::query::search,
            commands::devtools::get_dev_artifacts,
            commands::steam::get_steam_games,
            commands::delete::move_to_trash,
            commands::delete::delete_permanently,
            commands::delete::open_in_file_manager,
            commands::trash_list::list_trash,
            commands::trash_list::empty_trash,
            commands::trash_list::restore_from_trash,
            commands::duplicates::find_duplicates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
