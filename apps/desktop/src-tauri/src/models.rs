use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SessionCommand {
    Shell { shell: String },
    Command { program: String, args: Vec<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ThermalState {
    Hot,
    Warm,
    Cold,
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
