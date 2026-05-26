use serde::{Deserialize, Deserializer, Serialize};
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
    pub alternate_scroll_mode: String,
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
            alternate_scroll_mode: "off".to_string(),
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", default)]
pub struct AiConfig {
    pub enabled: bool,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub system_prompt: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "https://api.openai.com".to_string(),
            api_key: String::new(),
            model: String::new(),
            system_prompt: "You are a helpful assistant inside Vibemux, a terminal multiplexer. Keep answers concise and practical.".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionProfileKind {
    LocalShell,
    Wsl,
    Ssh,
    Command,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", default)]
pub struct SessionProfile {
    pub id: String,
    pub name: String,
    pub kind: SessionProfileKind,
    pub cwd: Option<String>,
    pub shell: Option<String>,
    pub program: Option<String>,
    pub args: Vec<String>,
    pub distro: Option<String>,
    pub host: Option<String>,
    pub ssh_config_host: Option<String>,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity_file: Option<String>,
    pub remote_cwd: Option<String>,
}

impl Default for SessionProfile {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            kind: SessionProfileKind::LocalShell,
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
        }
    }
}

impl SessionProfile {
    pub fn default_local_shell(shell: &ShellConfig) -> Self {
        Self {
            id: "default-local-shell".to_string(),
            name: "Default Shell".to_string(),
            kind: SessionProfileKind::LocalShell,
            cwd: None,
            shell: Some(shell.default.clone()),
            ..Self::default()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", default)]
pub struct ProfilesConfig {
    pub default_profile_id: Option<String>,
    pub last_used_profile_id: Option<String>,
    pub items: Vec<SessionProfile>,
}

impl Default for ProfilesConfig {
    fn default() -> Self {
        Self::from_shell(&ShellConfig::default())
    }
}

impl ProfilesConfig {
    pub fn from_shell(shell: &ShellConfig) -> Self {
        Self {
            default_profile_id: Some("default-local-shell".to_string()),
            last_used_profile_id: Some("default-local-shell".to_string()),
            items: vec![SessionProfile::default_local_shell(shell)],
        }
    }

    pub fn ensure_default(mut self, shell: &ShellConfig) -> Self {
        if self.items.is_empty() {
            return Self::from_shell(shell);
        }

        if self.default_profile_id.is_none()
            || !self
                .items
                .iter()
                .any(|p| Some(&p.id) == self.default_profile_id.as_ref())
        {
            self.default_profile_id = self.items.first().map(|p| p.id.clone());
        }

        if self
            .last_used_profile_id
            .as_ref()
            .is_some_and(|id| !self.items.iter().any(|p| &p.id == id))
        {
            self.last_used_profile_id = self.default_profile_id.clone();
        }

        self
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case", default)]
pub struct UserConfig {
    pub terminal: TerminalConfig,
    pub theme: ThemeConfig,
    pub layout: LayoutConfig,
    pub shell: ShellConfig,
    pub keys: KeysConfig,
    pub ai: AiConfig,
    pub profiles: ProfilesConfig,
    pub onboarding_completed: bool,
}

impl Default for UserConfig {
    fn default() -> Self {
        let shell = ShellConfig::default();
        Self {
            terminal: TerminalConfig::default(),
            theme: ThemeConfig::default(),
            layout: LayoutConfig::default(),
            profiles: ProfilesConfig::from_shell(&shell),
            shell,
            keys: KeysConfig::default(),
            ai: AiConfig::default(),
            onboarding_completed: false,
        }
    }
}

impl<'de> Deserialize<'de> for UserConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(rename_all = "snake_case", default)]
        struct RawUserConfig {
            terminal: Option<TerminalConfig>,
            theme: Option<ThemeConfig>,
            layout: Option<LayoutConfig>,
            shell: Option<ShellConfig>,
            keys: Option<KeysConfig>,
            ai: Option<AiConfig>,
            profiles: Option<ProfilesConfig>,
            onboarding_completed: Option<bool>,
        }

        let raw = RawUserConfig::deserialize(deserializer)?;
        let shell = raw.shell.unwrap_or_default();
        let profiles = raw
            .profiles
            .unwrap_or_else(|| ProfilesConfig::from_shell(&shell))
            .ensure_default(&shell);

        Ok(Self {
            terminal: raw.terminal.unwrap_or_default(),
            theme: raw.theme.unwrap_or_default(),
            layout: raw.layout.unwrap_or_default(),
            shell,
            keys: raw.keys.unwrap_or_default(),
            ai: raw.ai.unwrap_or_default(),
            profiles,
            onboarding_completed: raw.onboarding_completed.unwrap_or(false),
        })
    }
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
                let msg = format!(
                    "Config file at {:?} is corrupted: {}. Using defaults.",
                    path, e
                );
                eprintln!("Warning: {}", msg);
                (UserConfig::default(), Some(msg))
            }
        },
        Err(e) => {
            let msg = format!(
                "Could not read config at {:?}: {}. Using defaults.",
                path, e
            );
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
    let content =
        toml::to_string_pretty(config).map_err(|e| format!("failed to serialize config: {}", e))?;

    // Atomic write: write to temp file then rename
    let tmp_path = path.with_extension("toml.tmp");
    std::fs::write(&tmp_path, &content)
        .map_err(|e| format!("failed to write temp config: {}", e))?;
    std::fs::rename(&tmp_path, &path).map_err(|e| format!("failed to rename config: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_creates_a_local_shell_profile() {
        let cfg = UserConfig::default();

        assert_eq!(
            cfg.profiles.default_profile_id.as_deref(),
            Some("default-local-shell")
        );
        assert_eq!(
            cfg.profiles.last_used_profile_id.as_deref(),
            Some("default-local-shell")
        );
        assert_eq!(cfg.profiles.items.len(), 1);
        assert_eq!(cfg.profiles.items[0].id, "default-local-shell");
        assert!(matches!(
            cfg.profiles.items[0].kind,
            SessionProfileKind::LocalShell
        ));
    }

    #[test]
    fn legacy_config_without_profiles_gets_default_profiles() {
        let cfg: UserConfig = toml::from_str(
            r#"
            [shell]
            default = "/bin/bash"
            "#,
        )
        .expect("legacy config should still deserialize");

        assert_eq!(cfg.shell.default, "/bin/bash");
        assert_eq!(cfg.profiles.items.len(), 1);
        assert_eq!(cfg.profiles.items[0].shell.as_deref(), Some("/bin/bash"));
    }
}
