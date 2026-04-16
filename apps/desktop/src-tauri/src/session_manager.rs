use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::models::*;
use crate::pty_host::{PtyExitStatus, PtyHost};
use crate::ring_buffer::OutputRingBuffer;

const COLOR_CYCLE: [ColorToken; 8] = [
    ColorToken::Red,
    ColorToken::Cyan,
    ColorToken::Orange,
    ColorToken::Blue,
    ColorToken::Yellow,
    ColorToken::Purple,
    ColorToken::Green,
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
    SessionParked {
        session_id: Uuid,
    },
    ReplayStart {
        session_id: Uuid,
        from_seq: u64,
        to_seq: u64,
    },
    ReplayChunk {
        session_id: Uuid,
        data: Vec<u8>,
        seq: u64,
    },
    ReplayEnd {
        session_id: Uuid,
    },
    AttentionChanged {
        session_id: Uuid,
        attention_state: AttentionState,
    },
    SessionUpdated {
        session_id: Uuid,
    },
}

struct ManagedSession {
    session: Session,
    pty: PtyHost,
    output_buffer: Arc<Mutex<OutputRingBuffer>>,
    thermal_state: Arc<Mutex<ThermalState>>,
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
        max_hot_sessions: usize,
    ) -> Result<Uuid, String> {
        let max_hot_sessions = max_hot_sessions.max(1);
        if self.workspace.hot_session_ids.len() >= max_hot_sessions {
            return Err(hot_session_limit_message(max_hot_sessions));
        }

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

        let session = Session {
            id: session_id,
            name,
            custom_name: None,
            cwd,
            command,
            color,
            workspace_id: self.workspace.id,
            thermal_state: ThermalState::Hot,
            process_state: ProcessState::Running,
            attention_state: AttentionState::Normal,
            terminal_title: String::new(),
            created_at: now,
            updated_at: now,
            last_activity_at: now,
        };

        self.workspace.hot_session_ids.push(session_id);
        self.workspace.focused_session_id = Some(session_id);

        let output_buffer = Arc::new(Mutex::new(OutputRingBuffer::new()));
        let shared_thermal = Arc::new(Mutex::new(session.thermal_state.clone()));
        let shared_attention = Arc::new(Mutex::new(session.attention_state.clone()));

        // Spawn output routing task
        let buf_clone = Arc::clone(&output_buffer);
        let thermal_clone = Arc::clone(&shared_thermal);
        let attention_clone = Arc::clone(&shared_attention);
        let event_tx = self.event_tx.clone();
        let sid = session_id;
        tokio::spawn(async move {
            Self::route_output(
                sid,
                output_rx,
                buf_clone,
                event_tx,
                thermal_clone,
                attention_clone,
            )
            .await;
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
                thermal_state: shared_thermal,
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

    pub fn get_focused_context_source(&self) -> Option<(Session, Arc<Mutex<OutputRingBuffer>>)> {
        let session_id = self.workspace.focused_session_id?;
        self.sessions
            .get(&session_id)
            .map(|managed| (managed.session.clone(), Arc::clone(&managed.output_buffer)))
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

    pub fn rename_session(&mut self, session_id: Uuid, name: String) -> Result<(), String> {
        let managed = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| format!("session {} not found", session_id))?;
        managed.session.name = name.clone();
        managed.session.custom_name = Some(name);
        managed.session.updated_at = Utc::now();
        let _ = self.event_tx.send(MuxEvent::SessionUpdated { session_id });
        Ok(())
    }

    pub fn set_session_title(&mut self, session_id: Uuid, title: String) -> Result<(), String> {
        let managed = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| format!("session {} not found", session_id))?;
        managed.session.terminal_title = title.clone();
        // Only update name from terminal title if user hasn't set a custom name
        if managed.session.custom_name.is_none() {
            managed.session.name = title;
        }
        managed.session.updated_at = Utc::now();
        let _ = self.event_tx.send(MuxEvent::SessionUpdated { session_id });
        Ok(())
    }

    pub fn set_session_color(&mut self, session_id: Uuid, color: ColorToken) -> Result<(), String> {
        let managed = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| format!("session {} not found", session_id))?;
        managed.session.color = color;
        managed.session.updated_at = Utc::now();
        let _ = self.event_tx.send(MuxEvent::SessionUpdated { session_id });
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
        self.workspace
            .hot_session_ids
            .retain(|id| *id != session_id);
        self.workspace
            .warm_session_ids
            .retain(|id| *id != session_id);

        if self.workspace.focused_session_id == Some(session_id) {
            self.workspace.focused_session_id = self
                .workspace
                .hot_session_ids
                .first()
                .or(self.workspace.warm_session_ids.first())
                .copied();
        }
    }

    pub fn park_session(&mut self, session_id: Uuid) -> Result<(), String> {
        let managed = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| format!("session {} not found", session_id))?;

        if managed.session.thermal_state != ThermalState::Hot {
            return Err(format!("session {} is not a hot session", session_id));
        }

        managed.session.thermal_state = ThermalState::Warm;
        managed.session.updated_at = Utc::now();
        if let Ok(mut thermal) = managed.thermal_state.try_lock() {
            *thermal = ThermalState::Warm;
        }

        self.workspace
            .hot_session_ids
            .retain(|id| *id != session_id);
        self.workspace.warm_session_ids.push(session_id);

        if self.workspace.focused_session_id == Some(session_id) {
            self.workspace.focused_session_id = self.workspace.hot_session_ids.first().copied();
        }

        let _ = self.event_tx.send(MuxEvent::SessionParked { session_id });
        Ok(())
    }

    pub fn recall_session(
        &mut self,
        session_id: Uuid,
        max_hot_sessions: usize,
    ) -> Result<(), String> {
        let current_state = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| format!("session {} not found", session_id))?
            .session
            .thermal_state
            .clone();

        if current_state != ThermalState::Warm {
            return Err(format!("session {} is not a warm session", session_id));
        }

        let max_hot_sessions = max_hot_sessions.max(1);
        if self.workspace.hot_session_ids.len() >= max_hot_sessions {
            return Err(hot_session_limit_message(max_hot_sessions));
        }

        let managed = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| format!("session {} not found", session_id))?;

        managed.session.thermal_state = ThermalState::Hot;
        managed.session.updated_at = Utc::now();
        if let Ok(mut thermal) = managed.thermal_state.try_lock() {
            *thermal = ThermalState::Hot;
        }

        self.workspace
            .warm_session_ids
            .retain(|id| *id != session_id);
        self.workspace.hot_session_ids.push(session_id);
        self.workspace.focused_session_id = Some(session_id);

        // Emit replay events from the ring buffer
        let buffer = Arc::clone(&managed.output_buffer);
        let event_tx = self.event_tx.clone();
        let sid = session_id;
        tokio::spawn(async move {
            let entries = {
                let buf = buffer.lock().await;
                buf.get_all()
            };
            if entries.is_empty() {
                let _ = event_tx.send(MuxEvent::ReplayEnd { session_id: sid });
                return;
            }
            let from_seq = entries.first().map(|e| e.seq).unwrap_or(0);
            let to_seq = entries.last().map(|e| e.seq).unwrap_or(0);
            let _ = event_tx.send(MuxEvent::ReplayStart {
                session_id: sid,
                from_seq,
                to_seq,
            });
            for entry in entries {
                let _ = event_tx.send(MuxEvent::ReplayChunk {
                    session_id: sid,
                    data: entry.data,
                    seq: entry.seq,
                });
            }
            let _ = event_tx.send(MuxEvent::ReplayEnd { session_id: sid });
        });

        Ok(())
    }

    pub fn reorder_hot_sessions(&mut self, session_ids: Vec<Uuid>) -> Result<(), String> {
        // Validate all provided IDs are in hot_session_ids
        for id in &session_ids {
            if !self.workspace.hot_session_ids.contains(id) {
                return Err(format!("session {} is not a hot session", id));
            }
        }
        if session_ids.len() != self.workspace.hot_session_ids.len() {
            return Err("reorder must include all hot session IDs".to_string());
        }
        self.workspace.hot_session_ids = session_ids;
        Ok(())
    }

    async fn route_output(
        session_id: Uuid,
        mut output_rx: mpsc::UnboundedReceiver<Vec<u8>>,
        buffer: Arc<Mutex<OutputRingBuffer>>,
        event_tx: mpsc::UnboundedSender<MuxEvent>,
        thermal_state: Arc<Mutex<ThermalState>>,
        attention_state: Arc<Mutex<AttentionState>>,
    ) {
        while let Some(data) = output_rx.recv().await {
            let seq = {
                let mut buf = buffer.lock().await;
                buf.push(data.clone())
            };
            let _ = event_tx.send(MuxEvent::SessionOutput {
                session_id,
                data: data.clone(),
                seq,
            });

            // Attention state detection for warm sessions
            let thermal = thermal_state.lock().await.clone();
            if thermal == ThermalState::Warm {
                let text = String::from_utf8_lossy(&data).to_lowercase();
                let new_state = detect_attention_state(&text);
                if let Some(new_attention) = new_state {
                    let mut current = attention_state.lock().await;
                    if *current != new_attention {
                        *current = new_attention.clone();
                        let _ = event_tx.send(MuxEvent::AttentionChanged {
                            session_id,
                            attention_state: new_attention,
                        });
                    }
                } else {
                    // New output on warm session → Active (if not already Failed/NeedsInput/Done)
                    let mut current = attention_state.lock().await;
                    if *current == AttentionState::Normal {
                        *current = AttentionState::Active;
                        let _ = event_tx.send(MuxEvent::AttentionChanged {
                            session_id,
                            attention_state: AttentionState::Active,
                        });
                    }
                }
            }
        }
    }

    async fn monitor_exit(
        session_id: Uuid,
        mut exit_rx: mpsc::UnboundedReceiver<PtyExitStatus>,
        event_tx: mpsc::UnboundedSender<MuxEvent>,
    ) {
        if let Some(status) = exit_rx.recv().await {
            let process_state = match &status {
                PtyExitStatus::Exited(code) => ProcessState::Exited { code: *code },
                PtyExitStatus::Killed => ProcessState::Killed,
            };
            // Emit attention change based on exit status
            let attention = match &status {
                PtyExitStatus::Exited(Some(0)) => AttentionState::Done,
                PtyExitStatus::Exited(_) => AttentionState::Failed,
                PtyExitStatus::Killed => AttentionState::Failed,
            };
            let _ = event_tx.send(MuxEvent::AttentionChanged {
                session_id,
                attention_state: attention,
            });
            let _ = event_tx.send(MuxEvent::SessionExited {
                session_id,
                process_state,
            });
        }
    }
}

fn hot_session_limit_message(limit: usize) -> String {
    format!(
        "Hot Session limit reached. Current Hot Session limit is {}. Park or close a Hot Session, or change Max Hot Sessions in Settings > Layout.",
        limit
    )
}

fn detect_attention_state(text: &str) -> Option<AttentionState> {
    // NeedsInput patterns take priority
    let needs_input_patterns = [
        "continue?",
        "do you want",
        "press enter",
        "y/n",
        "[y/n]",
        "[y/n]",
        "(yes/no)",
    ];
    for pattern in &needs_input_patterns {
        if text.contains(pattern) {
            return Some(AttentionState::NeedsInput);
        }
    }

    // Failed patterns
    let failed_patterns = ["error", "failed", "panic", "fatal", "exception"];
    for pattern in &failed_patterns {
        if text.contains(pattern) {
            return Some(AttentionState::Failed);
        }
    }

    None
}
