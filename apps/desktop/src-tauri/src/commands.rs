use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config::{save_config, ConfigErrorState, ConfigState, UserConfig};
use crate::models::*;
use crate::session_manager::SessionManager;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionPayload {
    pub name: String,
    pub cwd: String,
    pub command_type: String, // "shell" or "command"
    pub shell: Option<String>,
    pub program: Option<String>,
    pub args: Option<Vec<String>>,
    pub color: Option<ColorToken>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub id: String,
    pub name: String,
    pub cwd: String,
    pub color: ColorToken,
    pub thermal_state: ThermalState,
    pub process_state: ProcessState,
    pub attention_state: AttentionState,
    pub terminal_title: String,
    pub last_activity_at: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSnapshot {
    pub id: String,
    pub name: String,
    pub hot_session_ids: Vec<String>,
    pub warm_session_ids: Vec<String>,
    pub focused_session_id: Option<String>,
    pub layout: String,
    pub sessions: Vec<SessionSnapshot>,
}

pub type AppState = Arc<Mutex<SessionManager>>;

fn session_to_snapshot(session: &crate::models::Session) -> SessionSnapshot {
    SessionSnapshot {
        id: session.id.to_string(),
        name: session.name.clone(),
        cwd: session.cwd.clone(),
        color: session.color.clone(),
        thermal_state: session.thermal_state.clone(),
        process_state: session.process_state.clone(),
        attention_state: session.attention_state.clone(),
        terminal_title: session.terminal_title.clone(),
        last_activity_at: session.last_activity_at.to_rfc3339(),
    }
}

#[tauri::command]
pub async fn session_create(
    state: State<'_, AppState>,
    payload: CreateSessionPayload,
) -> Result<SessionSnapshot, String> {
    eprintln!("[vibemux] session_create called: name={}, cwd={}, type={}", payload.name, payload.cwd, payload.command_type);
    let command = match payload.command_type.as_str() {
        "shell" => {
            let shell = payload
                .shell
                .unwrap_or_else(|| std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into()));
            eprintln!("[vibemux] shell={}", shell);
            SessionCommand::Shell { shell }
        }
        "command" => {
            let program = payload
                .program
                .ok_or("program is required for command type")?;
            let args = payload.args.unwrap_or_default();
            SessionCommand::Command { program, args }
        }
        other => return Err(format!("unknown command type: '{}'. Use 'shell' or 'command'", other)),
    };

    eprintln!("[vibemux] acquiring lock...");
    let mut manager = state.lock().await;
    eprintln!("[vibemux] lock acquired, creating session...");
    let session_id = manager.create_session(payload.name, payload.cwd, command, 80, 24)?;
    eprintln!("[vibemux] session created: {}", session_id);

    let session = manager
        .get_session(session_id)
        .ok_or_else(|| format!("session {} was created but not found", session_id))?;

    let snap = session_to_snapshot(session);
    eprintln!("[vibemux] returning snapshot: {:?}", snap);
    Ok(snap)
}

#[tauri::command]
pub async fn session_write(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let manager = state.lock().await;
    manager.write_to_session(uuid, data.as_bytes())
}

#[tauri::command]
pub async fn session_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let manager = state.lock().await;
    manager.resize_session(uuid, cols, rows)
}

#[tauri::command]
pub async fn workspace_get_snapshot(
    state: State<'_, AppState>,
) -> Result<WorkspaceSnapshot, String> {
    let manager = state.lock().await;
    let ws = manager.get_workspace();

    let mut sessions = Vec::new();
    for sid in ws.hot_session_ids.iter().chain(ws.warm_session_ids.iter()) {
        if let Some(session) = manager.get_session(*sid) {
            sessions.push(session_to_snapshot(session));
        }
    }

    Ok(WorkspaceSnapshot {
        id: ws.id.to_string(),
        name: ws.name.clone(),
        hot_session_ids: ws.hot_session_ids.iter().map(|id| id.to_string()).collect(),
        warm_session_ids: ws.warm_session_ids.iter().map(|id| id.to_string()).collect(),
        focused_session_id: ws.focused_session_id.map(|id| id.to_string()),
        layout: ws.layout.clone(),
        sessions,
    })
}

#[tauri::command]
pub async fn session_focus(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let mut manager = state.lock().await;
    manager.focus_session(uuid)
}

#[tauri::command]
pub async fn session_close(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let mut manager = state.lock().await;
    manager.close_session(uuid)
}

#[tauri::command]
pub async fn session_kill(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let mut manager = state.lock().await;
    manager.kill_session(uuid)
}

#[tauri::command]
pub async fn session_rename(
    state: State<'_, AppState>,
    session_id: String,
    name: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let mut manager = state.lock().await;
    manager.rename_session(uuid, name)
}

#[tauri::command]
pub async fn session_set_color(
    state: State<'_, AppState>,
    session_id: String,
    color: ColorToken,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let mut manager = state.lock().await;
    manager.set_session_color(uuid, color)
}

#[tauri::command]
pub async fn session_recall(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let mut manager = state.lock().await;
    manager.recall_session(uuid)
}

#[tauri::command]
pub async fn session_park(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let mut manager = state.lock().await;
    manager.park_session(uuid)
}

#[tauri::command]
pub async fn session_reorder(
    state: State<'_, AppState>,
    session_ids: Vec<String>,
) -> Result<(), String> {
    let uuids: Vec<Uuid> = session_ids
        .iter()
        .map(|id| Uuid::parse_str(id).map_err(|_| format!("invalid session id: '{}'", id)))
        .collect::<Result<Vec<_>, _>>()?;
    let mut manager = state.lock().await;
    manager.reorder_hot_sessions(uuids)
}

#[tauri::command]
pub fn config_get(config_state: State<'_, ConfigState>) -> Result<UserConfig, String> {
    let cfg = config_state.lock().map_err(|e| e.to_string())?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn config_update(
    config_state: State<'_, ConfigState>,
    update: serde_json::Value,
) -> Result<UserConfig, String> {
    let mut cfg = config_state.lock().map_err(|e| e.to_string())?;
    // Merge: serialize current, merge JSON, deserialize back
    let mut current_json = serde_json::to_value(&*cfg)
        .map_err(|e| format!("serialize error: {}", e))?;
    merge_json(&mut current_json, &update);
    let new_cfg: UserConfig = serde_json::from_value(current_json)
        .map_err(|e| format!("deserialize error: {}", e))?;
    *cfg = new_cfg.clone();
    drop(cfg);
    save_config(&new_cfg)?;
    Ok(new_cfg)
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("failed to open url: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("failed to open url: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", &url])
            .spawn()
            .map_err(|e| format!("failed to open url: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn session_set_title(
    state: State<'_, AppState>,
    session_id: String,
    title: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let mut manager = state.lock().await;
    manager.set_session_title(uuid, title)
}

#[tauri::command]
pub fn config_get_error(error_state: State<'_, ConfigErrorState>) -> Option<String> {
    error_state.lock().ok().and_then(|e| e.clone())
}

fn merge_json(base: &mut serde_json::Value, update: &serde_json::Value) {
    if let (serde_json::Value::Object(base_map), serde_json::Value::Object(update_map)) =
        (base, update)
    {
        for (k, v) in update_map {
            let entry = base_map.entry(k).or_insert(serde_json::Value::Null);
            if v.is_object() && entry.is_object() {
                merge_json(entry, v);
            } else {
                *entry = v.clone();
            }
        }
    }
}
