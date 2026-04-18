use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::commands::AppState;
use crate::config::{ConfigState, UserConfig};
use crate::models::{AttentionState, ProcessState, Session, ThermalState};

const MAX_CONTEXT_BYTES: usize = 24 * 1024;
const MODEL_REQUEST_TIMEOUT_SECS: u64 = 20;
const AI_CONNECT_TIMEOUT_SECS: u64 = 10;
const AI_STREAM_READ_TIMEOUT_SECS: u64 = 60;

pub type AiState = Arc<Mutex<AiStore>>;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AiMessage {
    pub id: String,
    pub role: AiMessageRole,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AiMessageRole {
    User,
    Assistant,
    System,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AiThread {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<AiMessage>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AiThreadSummary {
    pub id: String,
    pub title: String,
    pub updated_at: DateTime<Utc>,
    pub last_message_preview: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AiSendResponse {
    pub request_id: String,
    pub thread_id: String,
    pub assistant_message_id: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AiSendPayload {
    pub thread_id: Option<String>,
    pub content: String,
    pub include_focused_context: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AiFocusedContext {
    pub session_id: String,
    pub name: String,
    pub cwd: String,
    pub thermal_state: ThermalState,
    pub process_state: ProcessState,
    pub attention_state: AttentionState,
    pub terminal_title: String,
    pub output: String,
    pub truncated: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AiEvent {
    #[serde(rename_all = "camelCase")]
    Started {
        request_id: String,
        thread_id: String,
        assistant_message_id: String,
    },
    #[serde(rename_all = "camelCase")]
    Delta {
        request_id: String,
        thread_id: String,
        assistant_message_id: String,
        content: String,
    },
    #[serde(rename_all = "camelCase")]
    Done {
        request_id: String,
        thread_id: String,
        assistant_message_id: String,
    },
    #[serde(rename_all = "camelCase")]
    Error {
        request_id: String,
        thread_id: String,
        assistant_message_id: String,
        message: String,
    },
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct AiThreadFile {
    threads: Vec<AiThread>,
}

pub struct AiStore {
    path: PathBuf,
    threads: Vec<AiThread>,
}

impl AiStore {
    pub fn load() -> Self {
        let path = ai_threads_path();
        let threads = std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str::<AiThreadFile>(&content).ok())
            .map(|file| file.threads)
            .unwrap_or_default();
        Self { path, threads }
    }

    fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create AI data dir: {}", e))?;
        }
        let content = serde_json::to_string_pretty(&AiThreadFile {
            threads: self.threads.clone(),
        })
        .map_err(|e| format!("failed to serialize AI threads: {}", e))?;
        let tmp_path = self.path.with_extension("json.tmp");
        std::fs::write(&tmp_path, content)
            .map_err(|e| format!("failed to write AI thread temp file: {}", e))?;
        std::fs::rename(&tmp_path, &self.path)
            .map_err(|e| format!("failed to save AI threads: {}", e))?;
        Ok(())
    }

    fn summaries(&self) -> Vec<AiThreadSummary> {
        let mut summaries: Vec<AiThreadSummary> = self
            .threads
            .iter()
            .map(|thread| AiThreadSummary {
                id: thread.id.clone(),
                title: thread.title.clone(),
                updated_at: thread.updated_at,
                last_message_preview: thread
                    .messages
                    .iter()
                    .rev()
                    .find(|message| !message.content.trim().is_empty())
                    .map(|message| preview(&message.content, 96))
                    .unwrap_or_default(),
            })
            .collect();
        summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        summaries
    }

    fn get_thread(&self, thread_id: &str) -> Option<AiThread> {
        self.threads
            .iter()
            .find(|thread| thread.id == thread_id)
            .cloned()
    }

    fn delete_thread(&mut self, thread_id: &str) -> Result<(), String> {
        let initial_len = self.threads.len();
        self.threads.retain(|thread| thread.id != thread_id);
        if self.threads.len() == initial_len {
            return Err(format!("AI thread {} not found", thread_id));
        }
        self.save()
    }

    fn create_or_append_user_message(
        &mut self,
        thread_id: Option<String>,
        content: String,
        include_focused_context: bool,
    ) -> Result<(String, String), String> {
        let now = Utc::now();
        let user_message = AiMessage {
            id: Uuid::new_v4().to_string(),
            role: AiMessageRole::User,
            content: content.clone(),
            created_at: now,
            metadata: Some(json!({ "includeFocusedContext": include_focused_context })),
        };
        let assistant_message_id = Uuid::new_v4().to_string();
        let assistant_message = AiMessage {
            id: assistant_message_id.clone(),
            role: AiMessageRole::Assistant,
            content: String::new(),
            created_at: now,
            metadata: None,
        };

        let id = if let Some(thread_id) = thread_id {
            if let Some(thread) = self
                .threads
                .iter_mut()
                .find(|thread| thread.id == thread_id)
            {
                thread.messages.push(user_message);
                thread.messages.push(assistant_message);
                thread.updated_at = now;
                thread_id
            } else {
                return Err(format!("AI thread {} not found", thread_id));
            }
        } else {
            let thread_id = Uuid::new_v4().to_string();
            self.threads.push(AiThread {
                id: thread_id.clone(),
                title: title_from_content(&content),
                created_at: now,
                updated_at: now,
                messages: vec![user_message, assistant_message],
            });
            thread_id
        };
        self.save()?;
        Ok((id, assistant_message_id))
    }

    fn update_assistant_message(
        &mut self,
        thread_id: &str,
        assistant_message_id: &str,
        content: String,
    ) -> Result<(), String> {
        let now = Utc::now();
        let thread = self
            .threads
            .iter_mut()
            .find(|thread| thread.id == thread_id)
            .ok_or_else(|| format!("AI thread {} not found", thread_id))?;
        let message = thread
            .messages
            .iter_mut()
            .find(|message| message.id == assistant_message_id)
            .ok_or_else(|| format!("assistant message {} not found", assistant_message_id))?;
        message.content = content;
        thread.updated_at = now;
        self.save()
    }
}

#[tauri::command]
pub async fn ai_list_models(config_state: State<'_, ConfigState>) -> Result<Vec<String>, String> {
    let cfg = config_state.lock().map_err(|e| e.to_string())?.clone();
    let ai = cfg.ai;
    if ai.base_url.trim().is_empty() {
        return Err("AI Base URL is required".to_string());
    }
    if ai.api_key.trim().is_empty() {
        return Err("AI API Key is required".to_string());
    }
    let url = v1_url(&ai.base_url, "models");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(MODEL_REQUEST_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("failed to create AI client: {}", e))?;
    let value: serde_json::Value = client
        .get(url)
        .bearer_auth(ai.api_key)
        .send()
        .await
        .map_err(|e| format!("failed to fetch models: {}", e))?
        .error_for_status()
        .map_err(|e| format!("model request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("failed to parse models: {}", e))?;

    let mut models = value
        .get("data")
        .and_then(|data| data.as_array())
        .map(|data| {
            data.iter()
                .filter_map(|item| {
                    item.get("id")
                        .and_then(|id| id.as_str())
                        .map(str::to_string)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    models.sort();
    models.dedup();
    Ok(models)
}

#[tauri::command]
pub async fn ai_list_threads(ai_state: State<'_, AiState>) -> Result<Vec<AiThreadSummary>, String> {
    let store = ai_state.lock().await;
    Ok(store.summaries())
}

#[tauri::command]
pub async fn ai_get_thread(
    ai_state: State<'_, AiState>,
    thread_id: String,
) -> Result<AiThread, String> {
    let store = ai_state.lock().await;
    store
        .get_thread(&thread_id)
        .ok_or_else(|| format!("AI thread {} not found", thread_id))
}

#[tauri::command]
pub async fn ai_delete_thread(
    ai_state: State<'_, AiState>,
    thread_id: String,
) -> Result<(), String> {
    let mut store = ai_state.lock().await;
    store.delete_thread(&thread_id)
}

#[tauri::command]
pub async fn ai_get_focused_context(
    state: State<'_, AppState>,
) -> Result<Option<AiFocusedContext>, String> {
    focused_context_from_state(state.inner().clone()).await
}

#[tauri::command]
pub async fn ai_send_message(
    app: AppHandle,
    ai_state: State<'_, AiState>,
    config_state: State<'_, ConfigState>,
    session_state: State<'_, AppState>,
    payload: AiSendPayload,
) -> Result<AiSendResponse, String> {
    let cfg = config_state.lock().map_err(|e| e.to_string())?.clone();
    validate_ai_config(&cfg)?;

    let content = payload.content.trim().to_string();
    if content.is_empty() {
        return Err("AI instruction is empty".to_string());
    }

    let focused_context = if payload.include_focused_context {
        focused_context_from_state(session_state.inner().clone()).await?
    } else {
        None
    };

    let request_id = Uuid::new_v4().to_string();
    let (thread_id, assistant_message_id) = {
        let mut store = ai_state.lock().await;
        store.create_or_append_user_message(
            payload.thread_id.clone(),
            content,
            payload.include_focused_context,
        )?
    };

    let response = AiSendResponse {
        request_id: request_id.clone(),
        thread_id: thread_id.clone(),
        assistant_message_id: assistant_message_id.clone(),
    };

    let ai_state = ai_state.inner().clone();
    tokio::spawn(async move {
        stream_chat_completion(
            app,
            ai_state,
            cfg,
            request_id,
            thread_id,
            assistant_message_id,
            focused_context,
        )
        .await;
    });

    Ok(response)
}

async fn stream_chat_completion(
    app: AppHandle,
    ai_state: AiState,
    cfg: UserConfig,
    request_id: String,
    thread_id: String,
    assistant_message_id: String,
    focused_context: Option<AiFocusedContext>,
) {
    emit_ai_event(
        &app,
        AiEvent::Started {
            request_id: request_id.clone(),
            thread_id: thread_id.clone(),
            assistant_message_id: assistant_message_id.clone(),
        },
    );

    let thread = {
        let store = ai_state.lock().await;
        store.get_thread(&thread_id)
    };
    let Some(thread) = thread else {
        emit_error(
            &app,
            &ai_state,
            &request_id,
            &thread_id,
            &assistant_message_id,
            "AI thread disappeared before streaming".to_string(),
            String::new(),
        )
        .await;
        return;
    };

    let body = build_chat_body(&cfg, &thread, focused_context);
    let client = match reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(AI_CONNECT_TIMEOUT_SECS))
        .read_timeout(Duration::from_secs(AI_STREAM_READ_TIMEOUT_SECS))
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            emit_error(
                &app,
                &ai_state,
                &request_id,
                &thread_id,
                &assistant_message_id,
                format!("failed to create AI client: {}", e),
                String::new(),
            )
            .await;
            return;
        }
    };
    let result = client
        .post(v1_url(&cfg.ai.base_url, "chat/completions"))
        .bearer_auth(cfg.ai.api_key)
        .json(&body)
        .send()
        .await;

    let response = match result {
        Ok(response) => match response.error_for_status() {
            Ok(response) => response,
            Err(e) => {
                emit_error(
                    &app,
                    &ai_state,
                    &request_id,
                    &thread_id,
                    &assistant_message_id,
                    format!("AI request failed: {}", e),
                    String::new(),
                )
                .await;
                return;
            }
        },
        Err(e) => {
            emit_error(
                &app,
                &ai_state,
                &request_id,
                &thread_id,
                &assistant_message_id,
                format!("failed to send AI request: {}", e),
                String::new(),
            )
            .await;
            return;
        }
    };

    let mut stream = response.bytes_stream();
    let mut pending = String::new();
    let mut assistant_content = String::new();
    let mut last_save_len = 0usize;
    let mut last_save_at = Instant::now();

    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(e) => {
                emit_error(
                    &app,
                    &ai_state,
                    &request_id,
                    &thread_id,
                    &assistant_message_id,
                    format!("AI stream failed: {}", e),
                    assistant_content,
                )
                .await;
                return;
            }
        };
        pending.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(index) = pending.find('\n') {
            let line = pending[..index].trim_end_matches('\r').to_string();
            pending = pending[index + 1..].to_string();
            let Some(data) = line.strip_prefix("data:") else {
                continue;
            };
            let data = data.trim();
            if data == "[DONE]" {
                finalize_stream(
                    &app,
                    &ai_state,
                    &request_id,
                    &thread_id,
                    &assistant_message_id,
                    assistant_content,
                )
                .await;
                return;
            }
            if let Some(delta) = parse_sse_delta(data) {
                assistant_content.push_str(&delta);
                emit_ai_event(
                    &app,
                    AiEvent::Delta {
                        request_id: request_id.clone(),
                        thread_id: thread_id.clone(),
                        assistant_message_id: assistant_message_id.clone(),
                        content: delta,
                    },
                );
                if assistant_content.len().saturating_sub(last_save_len) > 1500
                    || last_save_at.elapsed() > Duration::from_secs(2)
                {
                    let _ = ai_state.lock().await.update_assistant_message(
                        &thread_id,
                        &assistant_message_id,
                        assistant_content.clone(),
                    );
                    last_save_len = assistant_content.len();
                    last_save_at = Instant::now();
                }
            }
        }
    }

    finalize_stream(
        &app,
        &ai_state,
        &request_id,
        &thread_id,
        &assistant_message_id,
        assistant_content,
    )
    .await;
}

async fn finalize_stream(
    app: &AppHandle,
    ai_state: &AiState,
    request_id: &str,
    thread_id: &str,
    assistant_message_id: &str,
    assistant_content: String,
) {
    let _ = ai_state.lock().await.update_assistant_message(
        thread_id,
        assistant_message_id,
        assistant_content,
    );
    emit_ai_event(
        app,
        AiEvent::Done {
            request_id: request_id.to_string(),
            thread_id: thread_id.to_string(),
            assistant_message_id: assistant_message_id.to_string(),
        },
    );
}

async fn emit_error(
    app: &AppHandle,
    ai_state: &AiState,
    request_id: &str,
    thread_id: &str,
    assistant_message_id: &str,
    message: String,
    assistant_content: String,
) {
    let content = if assistant_content.trim().is_empty() {
        format!("Error: {}", message)
    } else {
        assistant_content
    };
    let _ =
        ai_state
            .lock()
            .await
            .update_assistant_message(thread_id, assistant_message_id, content);
    emit_ai_event(
        app,
        AiEvent::Error {
            request_id: request_id.to_string(),
            thread_id: thread_id.to_string(),
            assistant_message_id: assistant_message_id.to_string(),
            message,
        },
    );
}

fn emit_ai_event(app: &AppHandle, event: AiEvent) {
    let _ = app.emit("ai-event", event);
}

fn build_chat_body(
    cfg: &UserConfig,
    thread: &AiThread,
    focused_context: Option<AiFocusedContext>,
) -> serde_json::Value {
    let mut messages = Vec::new();
    let system_prompt = cfg.ai.system_prompt.trim();
    if !system_prompt.is_empty() {
        messages.push(json!({ "role": "system", "content": system_prompt }));
    }
    if let Some(context) = focused_context {
        messages.push(json!({
            "role": "system",
            "content": format_focused_context(&context),
        }));
    }
    for message in &thread.messages {
        if message.role == AiMessageRole::System {
            continue;
        }
        if message.content.trim().is_empty() && message.role == AiMessageRole::Assistant {
            continue;
        }
        messages.push(json!({
            "role": match message.role {
                AiMessageRole::User => "user",
                AiMessageRole::Assistant => "assistant",
                AiMessageRole::System => "system",
            },
            "content": message.content,
        }));
    }
    json!({
        "model": cfg.ai.model,
        "messages": messages,
        "stream": true,
    })
}

async fn focused_context_from_state(state: AppState) -> Result<Option<AiFocusedContext>, String> {
    let source = {
        let manager = state.lock().await;
        manager.get_focused_context_source()
    };
    let Some((session, buffer)) = source else {
        return Ok(None);
    };
    let entries = {
        let buffer = buffer.lock().await;
        buffer.get_all()
    };
    Ok(Some(context_from_entries(&session, entries)))
}

fn context_from_entries(
    session: &Session,
    entries: Vec<crate::ring_buffer::RingBufferEntry>,
) -> AiFocusedContext {
    let mut bytes = Vec::new();
    for entry in entries {
        bytes.extend_from_slice(&entry.data);
    }
    let truncated = bytes.len() > MAX_CONTEXT_BYTES;
    if truncated {
        bytes = bytes[bytes.len() - MAX_CONTEXT_BYTES..].to_vec();
    }
    let output = strip_ansi(&String::from_utf8_lossy(&bytes));
    AiFocusedContext {
        session_id: session.id.to_string(),
        name: session.name.clone(),
        cwd: session.cwd.clone(),
        thermal_state: session.thermal_state.clone(),
        process_state: session.process_state.clone(),
        attention_state: session.attention_state.clone(),
        terminal_title: session.terminal_title.clone(),
        output,
        truncated,
    }
}

fn format_focused_context(context: &AiFocusedContext) -> String {
    format!(
        "Focused terminal context:\nSession: {}\nCWD: {}\nThermal state: {:?}\nProcess state: {:?}\nAttention state: {:?}\nTitle: {}\nOutput{}:\n{}",
        context.name,
        context.cwd,
        context.thermal_state,
        context.process_state,
        context.attention_state,
        context.terminal_title,
        if context.truncated { " (truncated to recent output)" } else { "" },
        context.output
    )
}

fn validate_ai_config(cfg: &UserConfig) -> Result<(), String> {
    if !cfg.ai.enabled {
        return Err("AI is disabled in Settings".to_string());
    }
    if cfg.ai.base_url.trim().is_empty() {
        return Err("AI Base URL is required".to_string());
    }
    if cfg.ai.api_key.trim().is_empty() {
        return Err("AI API Key is required".to_string());
    }
    if cfg.ai.model.trim().is_empty() {
        return Err("AI model is required".to_string());
    }
    Ok(())
}

fn ai_threads_path() -> PathBuf {
    app_support_dir().join("ai_threads.json")
}

fn app_support_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("vibemux")
    }
    #[cfg(not(target_os = "macos"))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".config").join("vibemux")
    }
}

fn v1_url(base_url: &str, endpoint: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    let endpoint = endpoint.trim_start_matches('/');
    if base.ends_with("/v1") {
        format!("{}/{}", base, endpoint)
    } else {
        format!("{}/v1/{}", base, endpoint)
    }
}

fn parse_sse_delta(data: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(data).ok()?;
    value
        .get("choices")?
        .get(0)?
        .get("delta")?
        .get("content")?
        .as_str()
        .map(str::to_string)
}

fn title_from_content(content: &str) -> String {
    let title = content
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("New chat")
        .trim();
    preview(title, 48)
}

fn preview(content: &str, max_chars: usize) -> String {
    let normalized = content.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = String::new();
    for ch in normalized.chars().take(max_chars) {
        out.push(ch);
    }
    if normalized.chars().count() > max_chars {
        out.push_str("...");
    }
    out
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if matches!(chars.peek(), Some('[')) {
                chars.next();
                for next in chars.by_ref() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ColorToken, SessionCommand};

    #[test]
    fn test_v1_url_normalizes_base_url() {
        assert_eq!(
            v1_url("https://api.openai.com", "models"),
            "https://api.openai.com/v1/models"
        );
        assert_eq!(
            v1_url("https://api.openai.com/v1/", "/chat/completions"),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn test_parse_sse_delta() {
        let data = r#"{"choices":[{"delta":{"content":"hello"}}]}"#;
        assert_eq!(parse_sse_delta(data), Some("hello".to_string()));
    }

    #[test]
    fn test_focused_context_truncates_and_strips_ansi() {
        let session = Session {
            id: Uuid::new_v4(),
            name: "shell".to_string(),
            custom_name: None,
            cwd: "/tmp".to_string(),
            command: SessionCommand::Shell {
                shell: "/bin/zsh".to_string(),
            },
            color: ColorToken::Blue,
            workspace_id: Uuid::new_v4(),
            thermal_state: ThermalState::Hot,
            process_state: ProcessState::Running,
            attention_state: AttentionState::Normal,
            terminal_title: "title".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_activity_at: Utc::now(),
        };
        let entries = vec![crate::ring_buffer::RingBufferEntry {
            seq: 1,
            data: format!("{}\x1b[31mred\x1b[0m", "x".repeat(MAX_CONTEXT_BYTES + 16)).into_bytes(),
        }];
        let context = context_from_entries(&session, entries);
        assert!(context.truncated);
        assert!(context.output.contains("red"));
        assert!(!context.output.contains("\x1b[31m"));
    }
}
