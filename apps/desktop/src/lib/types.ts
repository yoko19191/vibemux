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

export type SessionProfileKind = "local_shell" | "wsl" | "ssh" | "command";

export interface SessionProfile {
  id: string;
  name: string;
  kind: SessionProfileKind;
  cwd?: string | null;
  shell?: string | null;
  program?: string | null;
  args: string[];
  distro?: string | null;
  host?: string | null;
  ssh_config_host?: string | null;
  user?: string | null;
  port?: number | null;
  identity_file?: string | null;
  remote_cwd?: string | null;
}

export interface ProfilesConfig {
  default_profile_id?: string | null;
  last_used_profile_id?: string | null;
  items: SessionProfile[];
}

export interface SessionCapabilities {
  platform: "macos" | "windows" | "linux" | "unknown";
  shells: string[];
  wslDistros: string[];
  sshAvailable: boolean;
  sshConfigHosts: SshConfigHost[];
}

export interface SshConfigHost {
  alias: string;
  hostname?: string | null;
  user?: string | null;
  port?: number | null;
  identityFile?: string | null;
}

export interface TerminalConfig {
  font_family: string;
  font_size: number;
  line_height: number;
  scrollback_lines?: number;
  replay_buffer_lines?: number;
  replay_buffer_mb?: number;
  alternate_scroll_mode?: "off" | "arrows";
}

export interface ThemeConfig {
  background: string;
  foreground: string;
  cursor: string;
  selection: string;
  black: string;
  red: string;
  green: string;
  yellow: string;
  blue: string;
  magenta: string;
  cyan: string;
  white: string;
  bright_black: string;
  bright_red: string;
  bright_green: string;
  bright_yellow: string;
  bright_blue: string;
  bright_magenta: string;
  bright_cyan: string;
  bright_white: string;
}

export interface LayoutConfig {
  focused_pane_width: number;
  animation_ms: number;
  max_hot_sessions: number;
}

export interface KeysConfig {
  prefix: string;
}

export interface UserConfig {
  terminal: TerminalConfig;
  theme: ThemeConfig;
  layout: LayoutConfig;
  shell: { default: string };
  keys: KeysConfig;
  ai: AiConfig;
  profiles: ProfilesConfig;
  onboarding_completed?: boolean;
}

export type AiEvent =
  | { type: "started"; requestId: string; threadId: string; assistantMessageId: string }
  | { type: "delta"; requestId: string; threadId: string; assistantMessageId: string; content: string }
  | { type: "done"; requestId: string; threadId: string; assistantMessageId: string }
  | { type: "error"; requestId: string; threadId: string; assistantMessageId: string; message: string };
