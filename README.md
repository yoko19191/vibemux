# Vibemux

A keyboard-first, cross-platform terminal multiplexer with a GUI. Built with Tauri, Rust, and xterm.js.

![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-blue)
![Version](https://img.shields.io/badge/version-0.1.0-orange)
![License](https://img.shields.io/badge/license-MIT-green)

---

Vibemux is not a plain terminal emulator, and not a tmux wrapper. It treats each terminal as a **session task** with a name, color identity, working directory, and lifecycle state — so you always know what's running, what's waiting, and what needs your attention.

## Key Concepts

### Thermal Model

Every session has a thermal state:

| State | Location | PTY | xterm instance |
|-------|----------|-----|----------------|
| **Hot** | Deck | Running | Alive in DOM |
| **Warm** | Shelf | Running | Destroyed, output buffered |
| **Cold** | Archive | Stopped | None |

**Park** moves a hot session to the Shelf — the process keeps running, xterm is destroyed, output goes into a ring buffer.
**Recall** brings a warm session back — replays the buffer into a fresh xterm, then attaches live output.

### Deck

The horizontal strip of active (hot) sessions. One session is **focused** (full size, interactive). Others are **peripheral** (compressed, visible). Switch between them instantly — no PTY respawn, no replay, no lost scrollback.

### Shelf

The background task bar. Each warm session shows as a card with name, color, status badge, working directory, last output, and attention state. The PTY keeps running the whole time.

### Attention States

Vibemux watches warm session output and automatically flags sessions:

- `Active` — new output arrived
- `NeedsInput` — output contains prompts like `y/n`, `press enter`, `do you want`
- `Failed` — output contains `error`, `panic`, `fatal`
- `Done` — process exited with code 0
- `Failed` — process exited non-zero

## Features

- **Horizontal Deck** with focused + peripheral panes and 2.5D colored borders
- **Shelf** for background sessions — PTY stays alive, cards show live status
- **Park / Recall** — move sessions in and out of background without killing them
- **Replay** — recall replays buffered output before attaching live stream
- **Keyboard-first navigation** — prefix key activates Navigation Mode
- **GUI session creation** — name, working directory, command type, color
- **Session rename** — inline rename from keyboard
- **Session search** — fuzzy search across name, cwd, status
- **Drag-to-reorder** hot sessions in the Deck
- **Theme + font settings** — full ANSI 16-color palette, font family, size, line height
- **Config persistence** — TOML config, atomic writes, corruption-safe fallback
- **Cross-platform** — macOS, Linux, Windows

## Keyboard Shortcuts

Activate Navigation Mode with `Cmd+Space` (macOS) or `Ctrl+Space` (Linux/Windows).

| Key | Action |
|-----|--------|
| `h` / `←` | Focus previous hot session |
| `l` / `→` | Focus next hot session |
| `j` / `↓` | Select next Shelf card |
| `k` / `↑` | Select previous Shelf card |
| `Enter` | Recall selected Shelf session |
| `n` | New session |
| `b` | Park current session to Shelf |
| `r` | Rename current session |
| `x` | Close current session |
| `X` (Shift+x) | Kill current session (force) |
| `/` | Search sessions |
| `?` | Show keyboard help |
| `Esc` | Exit Navigation Mode |

## Installation

Download the latest release for your platform from the [Releases](../../releases) page.

| Platform | File |
|----------|------|
| macOS (Apple Silicon / Intel) | `.dmg` |
| Linux | `.AppImage` or `.deb` |
| Windows | `.msi` or `.exe` |

## Building from Source

**Prerequisites:**
- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+
- [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform

```bash
git clone https://github.com/yoko19191/vibemux
cd vibemux/apps/desktop
npm install
npm run tauri build
```

The built app will be in `apps/desktop/src-tauri/target/release/bundle/`.

For development:

```bash
npm run tauri dev
```

## Configuration

Config file location:

- **macOS**: `~/Library/Application Support/vibemux/config.toml`
- **Linux / Windows**: `~/.config/vibemux/config.toml`

Example:

```toml
[terminal]
font_family = "JetBrains Mono"
font_size = 14
line_height = 1.2
scrollback_lines = 10000
replay_buffer_mb = 20

[theme]
background = "#111111"
foreground = "#d9d4c7"
cursor = "#ff6b57"

[layout]
focused_pane_width = 0.6
preview_opacity = 0.8
animation_ms = 150
max_hot_sessions = 6
shelf_position = "bottom"

[shell]
default = "/bin/zsh"
```

All fields are optional — missing values fall back to defaults. A corrupted config file is detected at startup and replaced with defaults (a warning banner appears in the UI).

## Tech Stack

- **Shell**: [Tauri](https://tauri.app/) v2
- **Backend**: Rust + [Tokio](https://tokio.rs/) + [portable-pty](https://github.com/wez/wezterm/tree/main/pty)
- **Frontend**: [Svelte](https://svelte.dev/) 5 + TypeScript
- **Terminal renderer**: [xterm.js](https://xtermjs.org/) v6

## License

MIT
