use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use uuid::Uuid;

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
    let command = match payload.command_type.as_str() {
        "shell" => {
            let shell = payload
                .shell
                .unwrap_or_else(|| std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into()));
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

    let mut manager = state.lock().await;
    let session_id = manager.create_session(payload.name, payload.cwd, command, 80, 24)?;

    let session = manager
        .get_session(session_id)
        .ok_or_else(|| format!("session {} was created but not found", session_id))?;

    Ok(session_to_snapshot(session))
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
