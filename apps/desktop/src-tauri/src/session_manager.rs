use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::models::*;
use crate::pty_host::{PtyExitStatus, PtyHost};
use crate::ring_buffer::OutputRingBuffer;

const MAX_HOT_SESSIONS: usize = 6;

const COLOR_CYCLE: [ColorToken; 8] = [
    ColorToken::Red,
    ColorToken::Orange,
    ColorToken::Yellow,
    ColorToken::Green,
    ColorToken::Cyan,
    ColorToken::Blue,
    ColorToken::Purple,
    ColorToken::Pink,
];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MuxEvent {
    SessionOutput {
        session_id: Uuid,
        data: Vec<u8>,
        seq: u64,
    },
    SessionExited {
        session_id: Uuid,
        process_state: ProcessState,
    },
}

struct ManagedSession {
    session: Session,
    pty: PtyHost,
    output_buffer: Arc<Mutex<OutputRingBuffer>>,
}

pub struct SessionManager {
    sessions: HashMap<Uuid, ManagedSession>,
    workspace: Workspace,
    color_index: usize,
    event_tx: mpsc::UnboundedSender<MuxEvent>,
}

impl SessionManager {
    pub fn new(event_tx: mpsc::UnboundedSender<MuxEvent>) -> Self {
        let workspace = Workspace {
            id: Uuid::new_v4(),
            name: "Default".to_string(),
            hot_session_ids: Vec::new(),
            warm_session_ids: Vec::new(),
            focused_session_id: None,
            layout: "deck".to_string(),
        };

        Self {
            sessions: HashMap::new(),
            workspace,
            color_index: 0,
            event_tx,
        }
    }

    pub fn create_session(
        &mut self,
        name: String,
        cwd: String,
        command: SessionCommand,
        cols: u16,
        rows: u16,
    ) -> Result<Uuid, String> {
        let (cmd_str, args) = match &command {
            SessionCommand::Shell { shell } => (shell.clone(), vec![]),
            SessionCommand::Command { program, args } => (program.clone(), args.clone()),
        };

        let (output_tx, output_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (exit_tx, exit_rx) = mpsc::unbounded_channel::<PtyExitStatus>();

        let pty = PtyHost::spawn(&cmd_str, &args, &cwd, cols, rows, output_tx, exit_tx)
            .map_err(|e| e.to_string())?;

        let session_id = Uuid::new_v4();
        let now = Utc::now();
        let color = COLOR_CYCLE[self.color_index % COLOR_CYCLE.len()].clone();
        self.color_index += 1;

        let is_hot = self.workspace.hot_session_ids.len() < MAX_HOT_SESSIONS;
        let thermal_state = if is_hot {
            ThermalState::Hot
        } else {
            ThermalState::Warm
        };

        let session = Session {
            id: session_id,
            name,
            cwd,
            command,
            color,
            workspace_id: self.workspace.id,
            thermal_state,
            process_state: ProcessState::Running,
            attention_state: AttentionState::Normal,
            terminal_title: String::new(),
            created_at: now,
            updated_at: now,
            last_activity_at: now,
        };

        if is_hot {
            self.workspace.hot_session_ids.push(session_id);
        } else {
            self.workspace.warm_session_ids.push(session_id);
        }
        self.workspace.focused_session_id = Some(session_id);

        let output_buffer = Arc::new(Mutex::new(OutputRingBuffer::new()));

        // Spawn output routing task
        let buf_clone = Arc::clone(&output_buffer);
        let event_tx = self.event_tx.clone();
        let sid = session_id;
        tokio::spawn(async move {
            Self::route_output(sid, output_rx, buf_clone, event_tx).await;
        });

        // Spawn exit monitoring task
        let event_tx = self.event_tx.clone();
        let sid = session_id;
        tokio::spawn(async move {
            Self::monitor_exit(sid, exit_rx, event_tx).await;
        });

        self.sessions.insert(
            session_id,
            ManagedSession {
                session,
                pty,
                output_buffer,
            },
        );

        Ok(session_id)
    }

    pub fn write_to_session(&self, session_id: Uuid, data: &[u8]) -> Result<(), String> {
        let managed = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| format!("session {} not found", session_id))?;
        managed.pty.write(data).map_err(|e| e.to_string())
    }

    pub fn resize_session(&self, session_id: Uuid, cols: u16, rows: u16) -> Result<(), String> {
        let managed = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| format!("session {} not found", session_id))?;
        managed.pty.resize(cols, rows).map_err(|e| e.to_string())
    }

    pub fn get_session(&self, session_id: Uuid) -> Option<&Session> {
        self.sessions.get(&session_id).map(|m| &m.session)
    }

    pub fn get_workspace(&self) -> &Workspace {
        &self.workspace
    }

    pub fn update_process_state(&mut self, session_id: Uuid, state: ProcessState) {
        if let Some(managed) = self.sessions.get_mut(&session_id) {
            managed.session.process_state = state;
            managed.session.updated_at = Utc::now();
        }
    }

    pub fn focus_session(&mut self, session_id: Uuid) -> Result<(), String> {
        if !self.sessions.contains_key(&session_id) {
            return Err(format!("session {} not found", session_id));
        }
        self.workspace.focused_session_id = Some(session_id);
        Ok(())
    }

    pub fn close_session(&mut self, session_id: Uuid) -> Result<(), String> {
        let managed = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| format!("session {} not found", session_id))?;

        // Send SIGHUP for graceful close
        let _ = managed.pty.kill(); // Best effort — will be force-killed by timeout in caller
        self.remove_session_from_workspace(session_id);
        self.sessions.remove(&session_id);
        Ok(())
    }

    pub fn kill_session(&mut self, session_id: Uuid) -> Result<(), String> {
        let managed = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| format!("session {} not found", session_id))?;

        let _ = managed.pty.kill();
        self.remove_session_from_workspace(session_id);
        self.sessions.remove(&session_id);
        Ok(())
    }

    fn remove_session_from_workspace(&mut self, session_id: Uuid) {
        self.workspace.hot_session_ids.retain(|id| *id != session_id);
        self.workspace.warm_session_ids.retain(|id| *id != session_id);

        if self.workspace.focused_session_id == Some(session_id) {
            self.workspace.focused_session_id = self
                .workspace
                .hot_session_ids
                .first()
                .or(self.workspace.warm_session_ids.first())
                .copied();
        }
    }

    async fn route_output(
        session_id: Uuid,
        mut output_rx: mpsc::UnboundedReceiver<Vec<u8>>,
        buffer: Arc<Mutex<OutputRingBuffer>>,
        event_tx: mpsc::UnboundedSender<MuxEvent>,
    ) {
        while let Some(data) = output_rx.recv().await {
            let seq = {
                let mut buf = buffer.lock().await;
                buf.push(data.clone())
            };
            let _ = event_tx.send(MuxEvent::SessionOutput {
                session_id,
                data,
                seq,
            });
        }
    }

    async fn monitor_exit(
        session_id: Uuid,
        mut exit_rx: mpsc::UnboundedReceiver<PtyExitStatus>,
        event_tx: mpsc::UnboundedSender<MuxEvent>,
    ) {
        if let Some(status) = exit_rx.recv().await {
            let process_state = match status {
                PtyExitStatus::Exited(code) => ProcessState::Exited { code },
                PtyExitStatus::Killed => ProcessState::Killed,
            };
            let _ = event_tx.send(MuxEvent::SessionExited {
                session_id,
                process_state,
            });
        }
    }
}
