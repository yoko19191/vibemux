use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SessionCommand {
    Shell { shell: String },
    Command { program: String, args: Vec<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SessionOriginKind {
    Local,
    Ssh,
    Wsl,
    Command,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionOrigin {
    pub kind: SessionOriginKind,
    pub profile_id: Option<String>,
    pub profile_name: Option<String>,
    pub remote_label: Option<String>,
    pub badge: Option<String>,
}

impl Default for SessionOrigin {
    fn default() -> Self {
        Self {
            kind: SessionOriginKind::Local,
            profile_id: None,
            profile_name: None,
            remote_label: None,
            badge: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionIdentity {
    pub origin: SessionOrigin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ThermalState {
    Hot,
    Warm,
    Cold,
}

impl ThermalState {
    pub fn to_u8(&self) -> u8 {
        match self {
            ThermalState::Hot => 0,
            ThermalState::Warm => 1,
            ThermalState::Cold => 2,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => ThermalState::Hot,
            1 => ThermalState::Warm,
            _ => ThermalState::Cold,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ProcessState {
    Starting,
    Running,
    Exited { code: Option<i32> },
    FailedToStart { message: String },
    Killed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AttentionState {
    Normal,
    Active,
    NeedsInput,
    Failed,
    Done,
}

impl AttentionState {
    pub fn to_u8(&self) -> u8 {
        match self {
            AttentionState::Normal => 0,
            AttentionState::Active => 1,
            AttentionState::NeedsInput => 2,
            AttentionState::Failed => 3,
            AttentionState::Done => 4,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => AttentionState::Normal,
            1 => AttentionState::Active,
            2 => AttentionState::NeedsInput,
            3 => AttentionState::Failed,
            _ => AttentionState::Done,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ColorToken {
    Red,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Purple,
    Pink,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub id: Uuid,
    pub name: String,
    pub custom_name: Option<String>,
    pub cwd: String,
    pub command: SessionCommand,
    pub color: ColorToken,
    pub workspace_id: Uuid,
    pub thermal_state: ThermalState,
    pub process_state: ProcessState,
    pub attention_state: AttentionState,
    pub identity: SessionIdentity,
    pub terminal_title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub hot_session_ids: Vec<Uuid>,
    pub warm_session_ids: Vec<Uuid>,
    pub focused_session_id: Option<Uuid>,
    pub layout: String,
}
