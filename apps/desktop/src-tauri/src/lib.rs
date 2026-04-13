pub mod commands;
pub mod events;
pub mod models;
pub mod pty_host;
pub mod ring_buffer;
pub mod session_manager;

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use commands::AppState;
use session_manager::SessionManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let manager = SessionManager::new(event_tx);
    let state: AppState = Arc::new(Mutex::new(manager));

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::session_create,
            commands::session_write,
            commands::session_resize,
            commands::session_focus,
            commands::session_close,
            commands::session_kill,
            commands::session_park,
            commands::session_recall,
            commands::session_rename,
            commands::session_set_color,
            commands::session_reorder,
            commands::workspace_get_snapshot,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            events::start_event_bridge(handle, event_rx);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
