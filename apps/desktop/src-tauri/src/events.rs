use serde::Serialize;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::commands::SessionSnapshot;
use crate::models::ProcessState;
use crate::session_manager::MuxEvent;

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FrontendEvent {
    #[serde(rename_all = "camelCase")]
    SessionCreated {
        session: SessionSnapshot,
    },
    #[serde(rename_all = "camelCase")]
    SessionOutput {
        session_id: String,
        data: String,
        seq: u64,
    },
    #[serde(rename_all = "camelCase")]
    SessionExited {
        session_id: String,
        exit_code: Option<i32>,
    },
    #[serde(rename_all = "camelCase")]
    SessionUpdated {
        session_id: String,
    },
    #[serde(rename_all = "camelCase")]
    SessionParked {
        session_id: String,
    },
}

const BATCH_INTERVAL_MS: u64 = 12;

pub fn start_event_bridge(app_handle: AppHandle, mut event_rx: mpsc::UnboundedReceiver<MuxEvent>) {
    tokio::spawn(async move {
        let mut batch: Vec<FrontendEvent> = Vec::new();
        let mut interval = tokio::time::interval(Duration::from_millis(BATCH_INTERVAL_MS));

        loop {
            tokio::select! {
                event = event_rx.recv() => {
                    match event {
                        Some(mux_event) => {
                            let fe = convert_event(mux_event);
                            batch.push(fe);
                        }
                        None => {
                            // Channel closed, flush remaining
                            flush_batch(&app_handle, &mut batch);
                            break;
                        }
                    }
                }
                _ = interval.tick() => {
                    flush_batch(&app_handle, &mut batch);
                }
            }
        }
    });
}

fn convert_event(event: MuxEvent) -> FrontendEvent {
    match event {
        MuxEvent::SessionOutput {
            session_id,
            data,
            seq,
        } => FrontendEvent::SessionOutput {
            session_id: session_id.to_string(),
            data: String::from_utf8_lossy(&data).to_string(),
            seq,
        },
        MuxEvent::SessionExited {
            session_id,
            process_state,
        } => {
            let exit_code = match process_state {
                ProcessState::Exited { code } => code,
                _ => None,
            };
            FrontendEvent::SessionExited {
                session_id: session_id.to_string(),
                exit_code,
            }
        }
        MuxEvent::SessionParked { session_id } => FrontendEvent::SessionParked {
            session_id: session_id.to_string(),
        },
    }
}

fn flush_batch(app_handle: &AppHandle, batch: &mut Vec<FrontendEvent>) {
    if batch.is_empty() {
        return;
    }
    for event in batch.drain(..) {
        let _ = app_handle.emit("mux-event", &event);
    }
}

pub fn emit_session_created(app_handle: &AppHandle, snapshot: SessionSnapshot) {
    let event = FrontendEvent::SessionCreated { session: snapshot };
    let _ = app_handle.emit("mux-event", &event);
}

pub fn emit_session_updated(app_handle: &AppHandle, session_id: Uuid) {
    let event = FrontendEvent::SessionUpdated {
        session_id: session_id.to_string(),
    };
    let _ = app_handle.emit("mux-event", &event);
}
