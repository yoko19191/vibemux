use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", default)]
pub struct TerminalConfig {
    pub font_family: String,
    pub font_size: u32,
    pub line_height: f32,
    pub scrollback_lines: u32,
    pub replay_buffer_lines: u32,
    pub replay_buffer_mb: u32,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            font_family: "Menlo, Monaco, 'Courier New', monospace".to_string(),
            font_size: 14,
            line_height: 1.2,
            scrollback_lines: 10_000,
            replay_buffer_lines: 10_000,
            replay_buffer_mb: 20,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", default)]
pub struct ThemeConfig {
    pub background: String,
    pub foreground: String,
    pub cursor: String,
    pub selection: String,
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
    pub bright_black: String,
    pub bright_red: String,
    pub bright_green: String,
    pub bright_yellow: String,
    pub bright_blue: String,
    pub bright_magenta: String,
    pub bright_cyan: String,
    pub bright_white: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background: "#111111".to_string(),
            foreground: "#d9d4c7".to_string(),
            cursor: "#ff6b57".to_string(),
            selection: "#3b82f640".to_string(),
            black: "#1a1a1a".to_string(),
            red: "#ef4444".to_string(),
            green: "#22c55e".to_string(),
            yellow: "#eab308".to_string(),
            blue: "#3b82f6".to_string(),
            magenta: "#a855f7".to_string(),
            cyan: "#06b6d4".to_string(),
            white: "#d9d4c7".to_string(),
            bright_black: "#555555".to_string(),
            bright_red: "#f87171".to_string(),
            bright_green: "#4ade80".to_string(),
            bright_yellow: "#facc15".to_string(),
            bright_blue: "#60a5fa".to_string(),
            bright_magenta: "#c084fc".to_string(),
            bright_cyan: "#22d3ee".to_string(),
            bright_white: "#f5f5f5".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", default)]
pub struct LayoutConfig {
    pub default: String,
    pub focused_pane_width: f32,
    pub preview_opacity: f32,
    pub animation_ms: u32,
    pub max_hot_sessions: u32,
    pub shelf_position: String,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            default: "deck".to_string(),
            focused_pane_width: 0.6,
            preview_opacity: 0.8,
            animation_ms: 150,
            max_hot_sessions: 6,
            shelf_position: "bottom".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", default)]
pub struct ShellConfig {
    pub default: String,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            default: std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", default)]
pub struct KeysConfig {
    pub prefix: String,
}

impl Default for KeysConfig {
    fn default() -> Self {
        Self {
            prefix: "ctrl+b".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "snake_case", default)]
pub struct UserConfig {
    pub terminal: TerminalConfig,
    pub theme: ThemeConfig,
    pub layout: LayoutConfig,
    pub shell: ShellConfig,
    pub keys: KeysConfig,
}

pub type ConfigState = Arc<Mutex<UserConfig>>;

/// Holds the last config load error, if any (set once at startup).
pub type ConfigErrorState = Arc<Mutex<Option<String>>>;

fn config_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("vibemux")
            .join("config.toml")
    }
    #[cfg(not(target_os = "macos"))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("vibemux")
            .join("config.toml")
    }
}

/// Returns (config, optional_error_message)
pub fn load_config_with_error() -> (UserConfig, Option<String>) {
    let path = config_path();
    if !path.exists() {
        return (UserConfig::default(), None);
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => match toml::from_str::<UserConfig>(&content) {
            Ok(cfg) => (cfg, None),
            Err(e) => {
                let msg = format!("Config file at {:?} is corrupted: {}. Using defaults.", path, e);
                eprintln!("Warning: {}", msg);
                (UserConfig::default(), Some(msg))
            }
        },
        Err(e) => {
            let msg = format!("Could not read config at {:?}: {}. Using defaults.", path, e);
            eprintln!("Warning: {}", msg);
            (UserConfig::default(), Some(msg))
        }
    }
}

pub fn load_config() -> UserConfig {
    load_config_with_error().0
}

pub fn save_config(config: &UserConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create config dir: {}", e))?;
    }
    let content = toml::to_string_pretty(config)
        .map_err(|e| format!("failed to serialize config: {}", e))?;

    // Atomic write: write to temp file then rename
    let tmp_path = path.with_extension("toml.tmp");
    std::fs::write(&tmp_path, &content)
        .map_err(|e| format!("failed to write temp config: {}", e))?;
    std::fs::rename(&tmp_path, &path)
        .map_err(|e| format!("failed to rename config: {}", e))?;
    Ok(())
}
