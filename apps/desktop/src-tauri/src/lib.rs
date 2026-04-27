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

use tauri::{
    image::Image,
    menu::{AboutMetadataBuilder, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    Manager, RunEvent, WindowEvent,
};

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

            // macOS app menu with custom About metadata
            let about_metadata = AboutMetadataBuilder::new()
                .name(Some("Vibemux"))
                .version(Some("1.2.1"))
                .copyright(Some("Copyright © 2025 CGH"))
                .credits(Some("https://github.com/yoko19191/vibemux"))
                .build();

            let app_menu = Submenu::with_items(
                app,
                "Vibemux",
                true,
                &[
                    &PredefinedMenuItem::about(app, None, Some(about_metadata))?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::services(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::hide(app, None)?,
                    &PredefinedMenuItem::hide_others(app, None)?,
                    &PredefinedMenuItem::show_all(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::quit(app, None)?,
                ],
            )?;

            let window_menu = Submenu::with_items(
                app,
                "Window",
                true,
                &[
                    &PredefinedMenuItem::minimize(app, None)?,
                    &PredefinedMenuItem::maximize(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::close_window(app, None)?,
                ],
            )?;

            let menu = tauri::menu::Menu::with_items(app, &[&app_menu, &window_menu])?;
            app.set_menu(menu)?;

            // System tray
            let show_item = MenuItemBuilder::with_id("show", "Show Vibemux").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit Vibemux").build(app)?;
            let tray_menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let tray_icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .expect("failed to load tray icon");

            TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&tray_menu)
                .tooltip("Vibemux")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if let RunEvent::Reopen { .. } = event {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            #[cfg(not(target_os = "macos"))]
            let _ = (app, event);
        });
}
