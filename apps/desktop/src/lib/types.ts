export type ThermalState = "Hot" | "Warm" | "Cold";

export type ProcessState =
  | { type: "Starting" }
  | { type: "Running" }
  | { type: "Exited"; code: number | null }
  | { type: "FailedToStart"; message: string }
  | { type: "Killed" };

export type AttentionState =
  | "Normal"
  | "Active"
  | "NeedsInput"
  | "Failed"
  | "Done";

export type ColorToken =
  | "Red"
  | "Orange"
  | "Yellow"
  | "Green"
  | "Cyan"
  | "Blue"
  | "Purple"
  | "Pink";

export interface SessionSnapshot {
  id: string;
  name: string;
  customName: string | null;
  cwd: string;
  color: ColorToken;
  thermalState: ThermalState;
  processState: ProcessState;
  attentionState: AttentionState;
  terminalTitle: string;
  lastActivityAt: string;
}

export interface WorkspaceSnapshot {
  id: string;
  name: string;
  hotSessionIds: string[];
  warmSessionIds: string[];
  focusedSessionId: string | null;
  layout: string;
  sessions: SessionSnapshot[];
}

export type MuxEvent =
  | { type: "sessionCreated"; session: SessionSnapshot }
  | { type: "sessionOutput"; sessionId: string; data: string; seq: number }
  | { type: "sessionExited"; sessionId: string; exitCode: number | null }
  | { type: "sessionUpdated"; sessionId: string }
  | { type: "sessionParked"; sessionId: string }
  | { type: "replayStart"; sessionId: string; fromSeq: number; toSeq: number }
  | { type: "replayChunk"; sessionId: string; data: string; seq: number }
  | { type: "replayEnd"; sessionId: string }
  | { type: "attentionChanged"; sessionId: string; attentionState: AttentionState };

export interface AiConfig {
  enabled: boolean;
  base_url: string;
  api_key: string;
  model: string;
  system_prompt: string;
}

export type AiMessageRole = "user" | "assistant" | "system";

export interface AiMessage {
  id: string;
  role: AiMessageRole;
  content: string;
  createdAt: string;
  metadata: Record<string, unknown> | null;
}

export interface AiThread {
  id: string;
  title: string;
  createdAt: string;
  updatedAt: string;
  messages: AiMessage[];
}

export interface AiThreadSummary {
  id: string;
  title: string;
  updatedAt: string;
  lastMessagePreview: string;
}

export interface AiFocusedContext {
  sessionId: string;
  name: string;
  cwd: string;
  thermalState: ThermalState;
  processState: ProcessState;
  attentionState: AttentionState;
  terminalTitle: string;
  output: string;
  truncated: boolean;
}

export type AiEvent =
  | { type: "started"; requestId: string; threadId: string; assistantMessageId: string }
  | { type: "delta"; requestId: string; threadId: string; assistantMessageId: string; content: string }
  | { type: "done"; requestId: string; threadId: string; assistantMessageId: string }
  | { type: "error"; requestId: string; threadId: string; assistantMessageId: string; message: string };
