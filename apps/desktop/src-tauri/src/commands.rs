use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config::{
    save_config, ConfigErrorState, ConfigState, SessionProfile, SessionProfileKind, UserConfig,
};
use crate::models::*;
use crate::session_manager::SessionManager;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionPayload {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub command_type: Option<String>, // "shell" or "command"
    pub shell: Option<String>,
    pub program: Option<String>,
    pub args: Option<Vec<String>>,
    pub color: Option<ColorToken>,
    pub profile_id: Option<String>,
    pub profile: Option<SessionProfile>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SessionCapabilities {
    pub platform: String,
    pub shells: Vec<String>,
    pub wsl_distros: Vec<String>,
    pub ssh_available: bool,
    pub ssh_config_hosts: Vec<SshConfigHost>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SshConfigHost {
    pub alias: String,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity_file: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub id: String,
    pub name: String,
    pub custom_name: Option<String>,
    pub cwd: String,
    pub color: ColorToken,
    pub thermal_state: ThermalState,
    pub process_state: ProcessState,
    pub attention_state: AttentionState,
    pub identity: SessionIdentity,
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
        custom_name: session.custom_name.clone(),
        cwd: session.cwd.clone(),
        color: session.color.clone(),
        thermal_state: session.thermal_state.clone(),
        process_state: session.process_state.clone(),
        attention_state: session.attention_state.clone(),
        identity: session.identity.clone(),
        terminal_title: session.terminal_title.clone(),
        last_activity_at: session.last_activity_at.to_rfc3339(),
    }
}

#[tauri::command]
pub async fn session_create(
    state: State<'_, AppState>,
    config_state: State<'_, ConfigState>,
    payload: CreateSessionPayload,
) -> Result<SessionSnapshot, String> {
    eprintln!(
        "[vibemux] session_create called: name={:?}, cwd={:?}, type={:?}, profile_id={:?}",
        payload.name, payload.cwd, payload.command_type, payload.profile_id
    );
    let cfg = config_state.lock().map_err(|e| e.to_string())?.clone();
    let profile = resolve_payload_profile(&payload, &cfg)?;
    let command = if let Some(profile) = &profile {
        resolve_profile_command(profile)?
    } else {
        resolve_legacy_command(&payload, &cfg)?
    };
    let identity = build_session_identity(profile.as_ref(), &command);
    let session_name = payload
        .name
        .clone()
        .or_else(|| profile.as_ref().map(|profile| profile.name.clone()))
        .unwrap_or_else(|| "shell".to_string());
    let session_cwd = payload
        .cwd
        .clone()
        .or_else(|| profile.as_ref().and_then(|profile| profile.cwd.clone()))
        .unwrap_or_else(default_cwd);

    eprintln!("[vibemux] acquiring lock...");
    let max_hot_sessions = cfg.layout.max_hot_sessions as usize;
    let replay_buffer_lines = cfg.terminal.replay_buffer_lines as usize;
    let replay_buffer_bytes = (cfg.terminal.replay_buffer_mb as usize)
        .saturating_mul(1024)
        .saturating_mul(1024);
    let mut manager = state.lock().await;
    eprintln!("[vibemux] lock acquired, creating session...");
    let session_id = manager.create_session(
        session_name,
        session_cwd,
        command,
        identity,
        80,
        24,
        max_hot_sessions,
        replay_buffer_lines,
        replay_buffer_bytes,
    )?;
    eprintln!("[vibemux] session created: {}", session_id);

    let session = manager
        .get_session(session_id)
        .ok_or_else(|| format!("session {} was created but not found", session_id))?;

    let snap = session_to_snapshot(session);
    eprintln!("[vibemux] returning snapshot: {:?}", snap);
    Ok(snap)
}

fn resolve_payload_profile(
    payload: &CreateSessionPayload,
    cfg: &UserConfig,
) -> Result<Option<SessionProfile>, String> {
    if let Some(profile) = &payload.profile {
        return Ok(Some(profile.clone()));
    }

    if let Some(profile_id) = payload
        .profile_id
        .as_deref()
        .filter(|id| !id.trim().is_empty())
    {
        return cfg
            .profiles
            .items
            .iter()
            .find(|profile| profile.id == profile_id)
            .cloned()
            .map(Some)
            .ok_or_else(|| format!("profile not found: {}", profile_id));
    }

    Ok(None)
}

fn resolve_legacy_command(
    payload: &CreateSessionPayload,
    cfg: &UserConfig,
) -> Result<SessionCommand, String> {
    match payload.command_type.as_deref().unwrap_or("shell") {
        "shell" => {
            let shell = payload
                .shell
                .clone()
                .filter(|shell| !shell.trim().is_empty())
                .unwrap_or_else(|| cfg.shell.default.clone());
            eprintln!("[vibemux] shell={}", shell);
            Ok(SessionCommand::Shell { shell })
        }
        "command" => {
            let program = payload
                .program
                .clone()
                .ok_or("program is required for command type")?;
            let args = payload.args.clone().unwrap_or_default();
            Ok(SessionCommand::Command { program, args })
        }
        other => Err(format!(
            "unknown command type: '{}'. Use 'shell' or 'command'",
            other
        )),
    }
}

fn build_session_identity(
    profile: Option<&SessionProfile>,
    command: &SessionCommand,
) -> SessionIdentity {
    SessionIdentity {
        origin: build_session_origin(profile, command),
    }
}

fn build_session_origin(
    profile: Option<&SessionProfile>,
    command: &SessionCommand,
) -> SessionOrigin {
    if let Some(profile) = profile {
        return match profile.kind {
            SessionProfileKind::Ssh => SessionOrigin {
                kind: SessionOriginKind::Ssh,
                profile_id: Some(profile.id.clone()),
                profile_name: Some(profile.name.clone()),
                remote_label: ssh_profile_remote_label(profile),
                badge: Some("SSH".to_string()),
            },
            SessionProfileKind::Wsl => SessionOrigin {
                kind: SessionOriginKind::Wsl,
                profile_id: Some(profile.id.clone()),
                profile_name: Some(profile.name.clone()),
                remote_label: profile.distro.clone(),
                badge: None,
            },
            SessionProfileKind::Command => {
                if is_ssh_command(command) {
                    SessionOrigin {
                        kind: SessionOriginKind::Ssh,
                        profile_id: Some(profile.id.clone()),
                        profile_name: Some(profile.name.clone()),
                        remote_label: ssh_command_remote_label(command),
                        badge: Some("SSH".to_string()),
                    }
                } else {
                    SessionOrigin {
                        kind: SessionOriginKind::Command,
                        profile_id: Some(profile.id.clone()),
                        profile_name: Some(profile.name.clone()),
                        remote_label: None,
                        badge: None,
                    }
                }
            }
            SessionProfileKind::LocalShell => SessionOrigin {
                kind: SessionOriginKind::Local,
                profile_id: Some(profile.id.clone()),
                profile_name: Some(profile.name.clone()),
                remote_label: None,
                badge: None,
            },
        };
    }

    if is_ssh_command(command) {
        return SessionOrigin {
            kind: SessionOriginKind::Ssh,
            profile_id: None,
            profile_name: None,
            remote_label: ssh_command_remote_label(command),
            badge: Some("SSH".to_string()),
        };
    }

    match command {
        SessionCommand::Shell { .. } => SessionOrigin::default(),
        SessionCommand::Command { .. } => SessionOrigin {
            kind: SessionOriginKind::Command,
            profile_id: None,
            profile_name: None,
            remote_label: None,
            badge: None,
        },
    }
}

fn ssh_profile_remote_label(profile: &SessionProfile) -> Option<String> {
    if let Some(alias) = profile
        .ssh_config_host
        .as_deref()
        .filter(|host| !host.trim().is_empty())
    {
        return Some(alias.to_string());
    }

    let host = profile
        .host
        .as_deref()
        .filter(|host| !host.trim().is_empty())?;
    let label = match profile
        .user
        .as_deref()
        .filter(|user| !user.trim().is_empty())
    {
        Some(user) => format!("{}@{}", user, host),
        None => host.to_string(),
    };
    Some(match profile.port {
        Some(port) => format!("{}:{}", label, port),
        None => label,
    })
}

fn is_ssh_command(command: &SessionCommand) -> bool {
    let SessionCommand::Command { program, .. } = command else {
        return false;
    };
    command_basename(program) == "ssh"
}

fn ssh_command_remote_label(command: &SessionCommand) -> Option<String> {
    let SessionCommand::Command { args, .. } = command else {
        return None;
    };

    let mut skip_next = false;
    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }

        if arg == "-p" || arg == "-i" || arg == "-F" || arg == "-J" || arg == "-l" {
            skip_next = true;
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        if arg.contains('=') {
            continue;
        }

        return Some(arg.clone());
    }

    None
}

fn command_basename(program: &str) -> String {
    std::path::Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(program)
        .to_ascii_lowercase()
}

fn resolve_profile_command(profile: &SessionProfile) -> Result<SessionCommand, String> {
    match profile.kind {
        SessionProfileKind::LocalShell => {
            let shell = profile
                .shell
                .clone()
                .filter(|shell| !shell.trim().is_empty())
                .ok_or_else(|| "local shell profile requires a shell".to_string())?;
            Ok(SessionCommand::Shell { shell })
        }
        SessionProfileKind::Command => {
            let program = profile
                .program
                .clone()
                .filter(|program| !program.trim().is_empty())
                .ok_or_else(|| "command profile requires a program".to_string())?;
            Ok(SessionCommand::Command {
                program,
                args: profile.args.clone(),
            })
        }
        SessionProfileKind::Wsl => {
            let distro = profile
                .distro
                .clone()
                .filter(|distro| !distro.trim().is_empty())
                .ok_or_else(|| "wsl profile requires a distro".to_string())?;
            let mut args = vec!["-d".to_string(), distro];
            if let Some(remote_cwd) = profile
                .remote_cwd
                .as_deref()
                .filter(|cwd| !cwd.trim().is_empty())
            {
                args.push("--cd".to_string());
                args.push(remote_cwd.to_string());
            }
            if let Some(shell) = profile
                .shell
                .as_deref()
                .filter(|shell| !shell.trim().is_empty())
            {
                args.push("--exec".to_string());
                args.push(shell.to_string());
            }
            Ok(SessionCommand::Command {
                program: "wsl.exe".to_string(),
                args,
            })
        }
        SessionProfileKind::Ssh => {
            let ssh_config_host = profile
                .ssh_config_host
                .clone()
                .filter(|alias| !alias.trim().is_empty());
            let target = if let Some(alias) = ssh_config_host {
                alias
            } else {
                let host = profile
                    .host
                    .clone()
                    .filter(|host| !host.trim().is_empty())
                    .ok_or_else(|| "ssh profile requires a host".to_string())?;
                match profile
                    .user
                    .as_deref()
                    .filter(|user| !user.trim().is_empty())
                {
                    Some(user) => format!("{}@{}", user, host),
                    None => host,
                }
            };
            let mut args = vec![];
            if profile
                .ssh_config_host
                .as_deref()
                .is_none_or(|alias| alias.trim().is_empty())
            {
                if let Some(port) = profile.port {
                    args.push("-p".to_string());
                    args.push(port.to_string());
                }
                if let Some(identity_file) = profile
                    .identity_file
                    .as_deref()
                    .filter(|identity_file| !identity_file.trim().is_empty())
                {
                    args.push("-i".to_string());
                    args.push(identity_file.to_string());
                }
            }
            if profile
                .remote_cwd
                .as_deref()
                .is_some_and(|cwd| !cwd.trim().is_empty())
            {
                args.push("-t".to_string());
            }
            args.push(target);
            if let Some(remote_cwd) = profile
                .remote_cwd
                .as_deref()
                .filter(|cwd| !cwd.trim().is_empty())
            {
                args.push(format!("cd {} && exec $SHELL -l", shell_quote(remote_cwd)));
            }
            Ok(SessionCommand::Command {
                program: "ssh".to_string(),
                args,
            })
        }
    }
}

fn shell_quote(value: &str) -> String {
    if value.starts_with('~')
        || value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "/._-".contains(ch))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn default_cwd() -> String {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string())
}

#[tauri::command]
pub async fn session_get(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<SessionSnapshot, String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let manager = state.lock().await;
    let session = manager
        .get_session(uuid)
        .ok_or_else(|| format!("session {} not found", uuid))?;
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
    let mut manager = state.lock().await;
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
        warm_session_ids: ws
            .warm_session_ids
            .iter()
            .map(|id| id.to_string())
            .collect(),
        focused_session_id: ws.focused_session_id.map(|id| id.to_string()),
        layout: ws.layout.clone(),
        sessions,
    })
}

#[tauri::command]
pub async fn session_focus(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let mut manager = state.lock().await;
    manager.focus_session(uuid)
}

#[tauri::command]
pub async fn session_close(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let mut manager = state.lock().await;
    manager.close_session(uuid)
}

#[tauri::command]
pub async fn session_kill(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
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
    config_state: State<'_, ConfigState>,
    session_id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let max_hot_sessions = config_state
        .lock()
        .map_err(|e| e.to_string())?
        .layout
        .max_hot_sessions as usize;
    let mut manager = state.lock().await;
    manager.recall_session(uuid, max_hot_sessions)
}

#[tauri::command]
pub async fn session_park(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let mut manager = state.lock().await;
    manager.park_session(uuid)
}

#[tauri::command]
pub async fn session_save_snapshot(
    state: State<'_, AppState>,
    session_id: String,
    snapshot: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|_| format!("invalid session id: '{}'", session_id))?;
    let mut manager = state.lock().await;
    manager.save_screen_snapshot(uuid, snapshot)
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
pub async fn session_reorder_warm(
    state: State<'_, AppState>,
    session_ids: Vec<String>,
) -> Result<(), String> {
    let uuids: Vec<Uuid> = session_ids
        .iter()
        .map(|id| Uuid::parse_str(id).map_err(|_| format!("invalid session id: '{}'", id)))
        .collect::<Result<Vec<_>, _>>()?;
    let mut manager = state.lock().await;
    manager.reorder_warm_sessions(uuids)
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
    let mut current_json =
        serde_json::to_value(&*cfg).map_err(|e| format!("serialize error: {}", e))?;
    merge_json(&mut current_json, &update);
    let new_cfg: UserConfig =
        serde_json::from_value(current_json).map_err(|e| format!("deserialize error: {}", e))?;
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

#[tauri::command]
pub fn detect_shells() -> Vec<String> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let shells_file = std::fs::read_to_string("/etc/shells").unwrap_or_default();
        shells_file
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .filter(|l| std::path::Path::new(l).exists())
            .map(|l| l.to_string())
            .collect()
    }
    #[cfg(target_os = "windows")]
    {
        let candidates = [
            r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
            r"C:\Program Files\PowerShell\7\pwsh.exe",
            r"C:\Windows\System32\cmd.exe",
        ];
        candidates
            .iter()
            .filter(|p| std::path::Path::new(p).exists())
            .map(|p| p.to_string())
            .collect()
    }
}

#[tauri::command]
pub fn detect_session_capabilities() -> SessionCapabilities {
    SessionCapabilities {
        platform: current_platform().to_string(),
        shells: detect_shells(),
        wsl_distros: detect_wsl_distros(),
        ssh_available: which::which("ssh").is_ok(),
        ssh_config_hosts: read_ssh_config_hosts(),
    }
}

fn read_ssh_config_hosts() -> Vec<SshConfigHost> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_default();
    if home.is_empty() {
        return vec![];
    }

    let path = std::path::Path::new(&home).join(".ssh").join("config");
    match std::fs::read_to_string(path) {
        Ok(content) => parse_ssh_config_hosts(&content),
        Err(_) => vec![],
    }
}

fn parse_ssh_config_hosts(content: &str) -> Vec<SshConfigHost> {
    #[derive(Clone, Default)]
    struct HostBlock {
        aliases: Vec<String>,
        hostname: Option<String>,
        user: Option<String>,
        port: Option<u16>,
        identity_file: Option<String>,
    }

    fn flush(block: &mut Option<HostBlock>, hosts: &mut Vec<SshConfigHost>) {
        let Some(current) = block.take() else {
            return;
        };

        for alias in current.aliases {
            if alias.contains('*') || alias.contains('?') || alias == "!" {
                continue;
            }
            hosts.push(SshConfigHost {
                alias,
                hostname: current.hostname.clone(),
                user: current.user.clone(),
                port: current.port,
                identity_file: current.identity_file.clone(),
            });
        }
    }

    let mut hosts = vec![];
    let mut current: Option<HostBlock> = None;

    for raw_line in content.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(key) = parts.next() else {
            continue;
        };
        let values: Vec<&str> = parts.collect();

        if key.eq_ignore_ascii_case("host") {
            flush(&mut current, &mut hosts);
            current = Some(HostBlock {
                aliases: values.iter().map(|value| value.to_string()).collect(),
                ..HostBlock::default()
            });
            continue;
        }

        let Some(block) = current.as_mut() else {
            continue;
        };
        let value = values.join(" ");
        if value.is_empty() {
            continue;
        }

        if key.eq_ignore_ascii_case("hostname") {
            block.hostname = Some(value);
        } else if key.eq_ignore_ascii_case("user") {
            block.user = Some(value);
        } else if key.eq_ignore_ascii_case("port") {
            block.port = value.parse::<u16>().ok();
        } else if key.eq_ignore_ascii_case("identityfile") {
            block.identity_file = Some(value);
        }
    }

    flush(&mut current, &mut hosts);
    hosts
}

fn current_platform() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "unknown"
    }
}

fn detect_wsl_distros() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("wsl.exe")
            .args(["-l", "-q"])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let raw = String::from_utf8_lossy(&out.stdout).replace('\0', "");
                return raw
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .filter(|line| {
                        !line.eq_ignore_ascii_case("windows subsystem for linux distributions:")
                    })
                    .map(|line| line.to_string())
                    .collect();
            }
        }
        vec![]
    }
    #[cfg(not(target_os = "windows"))]
    {
        vec![]
    }
}

#[tauri::command]
pub fn list_monospace_fonts() -> Vec<String> {
    // Fonts shipped inside the app bundle (registered via @font-face in
    // fonts.css). Surfaced first so the picker works even if the user has
    // no system fonts and no fc-list available — and so users always have
    // a known-good Nerd Font option without installing anything.
    let bundled: &[&str] = &[
        "JetBrains Mono Nerd Font",
        "Fira Code Nerd Font",
        "Hack Nerd Font",
        "JetBrains Mono",
    ];

    let mut result: Vec<String> = bundled.iter().map(|s| s.to_string()).collect();

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let output = std::process::Command::new("fc-list")
            .args([":spacing=mono", "--format=%{family}\n"])
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                let raw = String::from_utf8_lossy(&out.stdout);
                let mut sys: Vec<String> = raw
                    .lines()
                    .flat_map(|l| l.split(','))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                sys.sort();
                sys.dedup();
                for f in sys {
                    if !result.iter().any(|b| b.eq_ignore_ascii_case(&f)) {
                        result.push(f);
                    }
                }
                return result;
            }
        }
        // fc-list missing/failed: append a small preset list of likely-installed mono fonts.
        for f in [
            "monospace",
            "Menlo",
            "Monaco",
            "Courier New",
            "Fira Code",
            "SF Mono",
        ] {
            if !result.iter().any(|b| b.eq_ignore_ascii_case(f)) {
                result.push(f.to_string());
            }
        }
        result
    }
    #[cfg(target_os = "windows")]
    {
        for f in ["Consolas", "Courier New", "Lucida Console", "Fira Code"] {
            if !result.iter().any(|b| b.eq_ignore_ascii_case(f)) {
                result.push(f.to_string());
            }
        }
        result
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SessionProfile, SessionProfileKind};

    #[test]
    fn ssh_profile_resolves_to_system_ssh_command() {
        let profile = SessionProfile {
            id: "prod".to_string(),
            name: "prod".to_string(),
            kind: SessionProfileKind::Ssh,
            cwd: Some("/Users/me".to_string()),
            shell: None,
            program: None,
            args: vec![],
            distro: None,
            host: Some("example.com".to_string()),
            ssh_config_host: None,
            user: Some("deploy".to_string()),
            port: Some(2222),
            identity_file: Some("~/.ssh/id_ed25519".to_string()),
            remote_cwd: Some("~/app".to_string()),
        };

        let command = resolve_profile_command(&profile).expect("ssh profile should resolve");

        assert!(matches!(command, SessionCommand::Command { .. }));
        if let SessionCommand::Command { program, args } = command {
            assert_eq!(program, "ssh");
            assert_eq!(
                args,
                vec![
                    "-p",
                    "2222",
                    "-i",
                    "~/.ssh/id_ed25519",
                    "-t",
                    "deploy@example.com",
                    "cd ~/app && exec $SHELL -l",
                ]
            );
        }
    }

    #[test]
    fn wsl_profile_resolves_to_wsl_exe_command() {
        let profile = SessionProfile {
            id: "ubuntu".to_string(),
            name: "Ubuntu".to_string(),
            kind: SessionProfileKind::Wsl,
            cwd: Some("/Users/me".to_string()),
            shell: Some("bash".to_string()),
            program: None,
            args: vec![],
            distro: Some("Ubuntu".to_string()),
            host: None,
            ssh_config_host: None,
            user: None,
            port: None,
            identity_file: None,
            remote_cwd: Some("~/work".to_string()),
        };

        let command = resolve_profile_command(&profile).expect("wsl profile should resolve");

        if let SessionCommand::Command { program, args } = command {
            assert_eq!(program, "wsl.exe");
            assert_eq!(
                args,
                vec!["-d", "Ubuntu", "--cd", "~/work", "--exec", "bash"]
            );
        } else {
            panic!("expected command profile");
        }
    }

    #[test]
    fn ssh_profile_requires_host() {
        let profile = SessionProfile {
            id: "bad".to_string(),
            name: "bad".to_string(),
            kind: SessionProfileKind::Ssh,
            cwd: None,
            shell: None,
            program: None,
            args: vec![],
            distro: None,
            host: None,
            ssh_config_host: None,
            user: None,
            port: None,
            identity_file: None,
            remote_cwd: None,
        };

        assert!(resolve_profile_command(&profile)
            .unwrap_err()
            .contains("host"));
    }

    #[test]
    fn ssh_config_profile_resolves_to_alias_only() {
        let profile = SessionProfile {
            id: "prod".to_string(),
            name: "prod".to_string(),
            kind: SessionProfileKind::Ssh,
            cwd: None,
            shell: None,
            program: None,
            args: vec![],
            distro: None,
            host: Some("prod.example.com".to_string()),
            ssh_config_host: Some("prod".to_string()),
            user: Some("deploy".to_string()),
            port: Some(2202),
            identity_file: Some("~/.ssh/prod_ed25519".to_string()),
            remote_cwd: Some("~/app".to_string()),
        };

        let command = resolve_profile_command(&profile).expect("ssh config profile should resolve");

        if let SessionCommand::Command { program, args } = command {
            assert_eq!(program, "ssh");
            assert_eq!(args, vec!["-t", "prod", "cd ~/app && exec $SHELL -l"]);
        } else {
            panic!("expected command profile");
        }
    }

    #[test]
    fn ssh_profile_builds_ssh_origin_identity() {
        let profile = SessionProfile {
            id: "prod".to_string(),
            name: "Production".to_string(),
            kind: SessionProfileKind::Ssh,
            cwd: None,
            shell: None,
            program: None,
            args: vec![],
            distro: None,
            host: Some("example.com".to_string()),
            ssh_config_host: None,
            user: Some("deploy".to_string()),
            port: Some(2222),
            identity_file: None,
            remote_cwd: None,
        };
        let command = resolve_profile_command(&profile).expect("ssh command");
        let identity = build_session_identity(Some(&profile), &command);

        assert_eq!(identity.origin.kind, SessionOriginKind::Ssh);
        assert_eq!(identity.origin.badge.as_deref(), Some("SSH"));
        assert_eq!(identity.origin.profile_name.as_deref(), Some("Production"));
        assert_eq!(
            identity.origin.remote_label.as_deref(),
            Some("deploy@example.com:2222")
        );
    }

    #[test]
    fn direct_ssh_command_builds_ssh_origin_identity() {
        let command = SessionCommand::Command {
            program: "/usr/bin/ssh".to_string(),
            args: vec![
                "-p".to_string(),
                "2222".to_string(),
                "deploy@example.com".to_string(),
            ],
        };
        let identity = build_session_identity(None, &command);

        assert_eq!(identity.origin.kind, SessionOriginKind::Ssh);
        assert_eq!(identity.origin.badge.as_deref(), Some("SSH"));
        assert_eq!(
            identity.origin.remote_label.as_deref(),
            Some("deploy@example.com")
        );
    }

    #[test]
    fn parses_ssh_config_hosts_with_connection_defaults() {
        let hosts = parse_ssh_config_hosts(
            r#"
            Host *
              User ignored

            Host prod prod-short
              HostName prod.example.com
              User deploy
              Port 2202
              IdentityFile ~/.ssh/prod_ed25519

            Host staging
              HostName 10.0.0.5
              User ubuntu
            "#,
        );

        assert_eq!(hosts.len(), 3);
        assert_eq!(hosts[0].alias, "prod");
        assert_eq!(hosts[0].hostname.as_deref(), Some("prod.example.com"));
        assert_eq!(hosts[0].user.as_deref(), Some("deploy"));
        assert_eq!(hosts[0].port, Some(2202));
        assert_eq!(
            hosts[0].identity_file.as_deref(),
            Some("~/.ssh/prod_ed25519")
        );
        assert_eq!(hosts[1].alias, "prod-short");
        assert_eq!(hosts[1].hostname.as_deref(), Some("prod.example.com"));
        assert_eq!(hosts[2].alias, "staging");
    }
}
