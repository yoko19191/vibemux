pub mod ai;
pub mod commands;
pub mod config;
pub mod events;
pub mod models;
pub mod pty_host;
pub mod ring_buffer;
pub mod session_manager;

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use ai::{AiState, AiStore};
use commands::AppState;
use config::{load_config_with_error, ConfigErrorState, ConfigState};
use session_manager::SessionManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let manager = SessionManager::new(event_tx);
    let state: AppState = Arc::new(Mutex::new(manager));
    let event_state = state.clone();
    let (cfg, cfg_error) = load_config_with_error();
    let config_state: ConfigState = Arc::new(std::sync::Mutex::new(cfg));
    let config_error_state: ConfigErrorState = Arc::new(std::sync::Mutex::new(cfg_error));
    let ai_state: AiState = Arc::new(Mutex::new(AiStore::load()));

    tauri::Builder::default()
        .manage(state)
        .manage(config_state)
        .manage(config_error_state)
        .manage(ai_state)
        .invoke_handler(tauri::generate_handler![
            commands::session_create,
            commands::session_get,
            commands::session_write,
            commands::session_resize,
            commands::session_focus,
            commands::session_close,
            commands::session_kill,
            commands::session_park,
            commands::session_recall,
            commands::session_save_snapshot,
            commands::session_rename,
            commands::session_set_color,
            commands::session_reorder,
            commands::session_reorder_warm,
            commands::workspace_get_snapshot,
            commands::config_get,
            commands::config_update,
            commands::config_get_error,
            commands::open_url,
            commands::session_set_title,
            commands::detect_shells,
            commands::list_monospace_fonts,
            ai::ai_list_models,
            ai::ai_list_threads,
            ai::ai_get_thread,
            ai::ai_delete_thread,
            ai::ai_send_message,
            ai::ai_get_focused_context,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            events::start_event_bridge(handle, event_state.clone(), event_rx);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
