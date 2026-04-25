<h1 align="center">
  <img src="assets/icon.png" alt="Vibemux icon" width="48" height="48"><br>
  Vibemux
</h1>

<p align="center">
  <video src="assets/demo.mp4" controls muted loop playsinline width="100%"></video>
</p>

<p align="center">
  A keyboard-first, cross-platform terminal multiplexer for ADHD-friendly multitasking and Vibe Coding workflows. Built with Tauri, Rust, Svelte, and xterm.js.
</p>

<p align="center">
  <img alt="Platform" src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-blue">
  <img alt="Version" src="https://img.shields.io/badge/version-0.1.0-orange">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green">
</p>

---

[\[中文\]](README_zh.md)

Vibemux is not a plain terminal emulator, and not a tmux wrapper. It is designed for ADHD-friendly terminal workflows and Vibe Coding sessions where multiple shells, dev servers, tests, logs, and coding agents are running at the same time. It treats each terminal as a **session task** with a name, color identity, working directory, process state, and lifecycle state, so you can see what is active, what is detached, and what needs attention.

## Features

- **Horizontal Deck** with one focused terminal and overlapping peripheral terminals.
- **Detached sessions** keep their PTY running while the full xterm renderer is unloaded.
- **Detach / Attach** moves sessions between the Deck and Detached bar without killing the process.
- **Screen snapshot + replay** restores detached sessions with their static screen content, scrollback buffer, and live output stream.
- **Keyboard-first Navigation Mode** with a configurable prefix key; the default is `Ctrl+B`.
- **Global command palette** with `Cmd+K` on macOS or `Ctrl+K` on Linux/Windows for session search, quick actions, and Ask AI.
- **Ask AI** supports OpenAI-compatible endpoints, model discovery, streamed responses, saved threads, and optional focused-terminal context.
- **GUI session creation** for shell or command sessions, including name, working directory, shell/program, arguments, and color.
- **Titlebar actions** for new session, search, and settings.
- **Pane and Detached-card actions** for detach/attach, rename, color changes, close, and force kill.
- **Drag-to-reorder Deck** so hot sessions can be rearranged visually.
- **Busy indicators and attention badges** show running, active, done, failed, or input-needed sessions at a glance.
- **Onboarding flow** for choosing a navigation key, default shell, terminal theme/font, and optional AI setup.
- **Settings panel** for terminal fonts, themes, layout, max hot sessions, prefix key, and AI configuration.
- **Config persistence** with TOML, atomic writes, and a corruption-safe fallback banner.
- **Embedded terminal fonts** to keep build output and terminal glyphs rendering consistently.
- **Cross-platform desktop app** for macOS, Linux, and Windows.

## Keyboard Shortcuts

The default Navigation Mode prefix is `Ctrl+B`. You can change it in **Settings -> Keys** or during onboarding. Presets include `Ctrl+B`, `Ctrl+Space`, <code>Ctrl+&#96;</code>, `Ctrl+A`, `Cmd+Space`, and custom key combos.

| Key | Action |
|-----|--------|
| `Ctrl+B` by default | Enter / exit Navigation Mode |
| `Cmd+K` / `Ctrl+K` | Open global search and Ask AI |
| `h` / `←` | Focus previous hot session |
| `l` / `→` | Focus next hot session |
| `n` | New session |
| `b` | Detach current session |
| `j` / `↓` | Select next detached session |
| `k` / `↑` | Select previous detached session |
| `Enter` | Attach selected detached session |
| `r` | Rename current session |
| `/` | Search sessions |
| `?` | Show keyboard help |
| `x` | Close current session gracefully |
| `X` | Kill current session forcefully |
| `Esc` | Exit Navigation Mode |

In the search panel, type normally to filter sessions and saved AI threads. Start with `#` or choose **Ask AI** to enter chat mode.

## Key Concepts

### Thermal Model

Every session has a thermal state:

| State | User-facing location | PTY | xterm instance |
|-------|----------------------|-----|----------------|
| **Hot** | Deck | Running | Alive in DOM |
| **Warm** | Detached bar | Running | Destroyed, output buffered |
| **Cold** | Archive / history | Stopped | None |

**Detach** moves a hot session out of the Deck. The process keeps running, Vibemux saves the screen snapshot, destroys the xterm instance, and continues buffering output.

**Attach** brings a warm session back. Vibemux restores the saved screen, replays buffered output, resizes the PTY, focuses the terminal, and then attaches the live output stream.

### Deck

The Deck is the horizontal strip of active hot sessions. One session is **focused** and fully interactive. Other sessions stay visible as compressed, overlapping peripheral panes, so you keep context without giving every process a full terminal viewport.

### Detached Bar

Detached sessions live in the compact bar at the bottom of the window. Each card shows the session name, color, short working directory, busy state, and attention badge. Click a card or use Navigation Mode to attach it back to the Deck.

### Attention States

Vibemux watches detached session output and process exits:

- `Active` means new output arrived.
- `NeedsInput` means output looks like a prompt, such as `y/n`, `press enter`, or `do you want`.
- `Failed` means output matched failure patterns such as `error`, `panic`, or `fatal`, or the process exited non-zero.
- `Done` means the process exited with code 0.

## Installation

Download the latest release for your platform from the [Releases](../../releases) page.

| Platform | File |
|----------|------|
| macOS (Apple Silicon / Intel) | `.dmg` |
| Linux | `.AppImage` or `.deb` |
| Windows | `.msi` or `.exe` |

### macOS unsigned build note

The first release is not signed or notarized yet. macOS may block it on first launch.

Try this first:

1. Move Vibemux to `/Applications`.
2. Control-click `Vibemux.app`.
3. Choose **Open**.
4. Confirm **Open** again.

If macOS still reports that the app is damaged or cannot be opened, advanced users can remove the quarantine attribute:

```bash
xattr -dr com.apple.quarantine /Applications/Vibemux.app
open /Applications/Vibemux.app
```

If the command reports a permission error, run the same `xattr` command with `sudo`. Only do this for builds downloaded from the official Vibemux release page.

## Building from Source

**Prerequisites:**

- [Rust](https://rustup.rs/) stable
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
font_family = "Menlo, Monaco, 'Courier New', monospace"
font_size = 14
line_height = 1.2
scrollback_lines = 10000
replay_buffer_lines = 10000
replay_buffer_mb = 20

[theme]
background = "#111111"
foreground = "#d9d4c7"
cursor = "#ff6b57"
selection = "#3b82f640"

[layout]
focused_pane_width = 0.6
preview_opacity = 0.8
animation_ms = 150
max_hot_sessions = 6
shelf_position = "bottom"

[keys]
prefix = "ctrl+b"

[shell]
default = "/bin/zsh"

[ai]
enabled = false
base_url = "https://api.openai.com"
api_key = ""
model = ""
system_prompt = "You are a helpful assistant inside Vibemux, a terminal multiplexer. Keep answers concise and practical."
```

All fields are optional. Missing values fall back to defaults. If the config file is corrupted, Vibemux starts with defaults and shows a warning banner in the UI.

## Tech Stack

- **Shell**: [Tauri](https://tauri.app/) v2
- **Backend**: Rust + [Tokio](https://tokio.rs/) + [portable-pty](https://github.com/wez/wezterm/tree/main/pty)
- **Frontend**: [Svelte](https://svelte.dev/) 5 + TypeScript
- **Terminal renderer**: [xterm.js](https://xtermjs.org/) v6
- **AI transport**: OpenAI-compatible `/v1/models` and `/v1/chat/completions` APIs

## Acknowledgements

Thanks to [Vibe99](https://github.com/NekoApocalypse/Vibe99) for inspiring the terminal multiplexing idea behind Vibemux.

## License

MIT
