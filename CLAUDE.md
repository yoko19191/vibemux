# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is Vibemux

A keyboard-first, cross-platform terminal multiplexer built with Tauri + Rust + xterm.js. Core concept: sessions have a thermal lifecycle (Hot → Warm → Cold). Hot sessions live in the Deck (full xterm instances). Warm sessions live in the Shelf (PTY keeps running, xterm destroyed, output buffered). Recall replays the ring buffer then attaches live output.

## Project Structure

```
apps/desktop/
  src/                    # Svelte 5 frontend
    App.svelte            # Root: session state, keyboard nav, event routing
    lib/
      types.ts            # Shared TS types (SessionSnapshot, MuxEvent, etc.)
      Deck.svelte         # Hot session layout
      DeckPane.svelte     # Individual pane with 2.5D colored border
      TerminalPane.svelte # xterm.js lifecycle per session
      Shelf.svelte        # Warm session cards
      ShelfCard.svelte
      terminalReplay.ts   # Chunked replay queue logic
      deckLayout.ts       # Pane sizing calculations
      NewSessionPanel.svelte
      SessionSearch.svelte
      SettingsPanel.svelte
      HelpOverlay.svelte
  src-tauri/src/          # Rust backend
    lib.rs                # Tauri setup, state wiring
    commands.rs           # All #[tauri::command] handlers + SessionSnapshot/WorkspaceSnapshot
    models.rs             # Core types: Session, Workspace, ThermalState, ProcessState, AttentionState, ColorToken
    session_manager.rs    # SessionManager: owns all sessions, PTY routing, park/recall logic
    pty_host.rs           # portable-pty wrapper: spawn, write, resize, kill
    ring_buffer.rs        # OutputRingBuffer: seq-numbered, evicts by line count or bytes
    events.rs             # Tauri event bridge: batches MuxEvents at 12ms intervals → "mux-event"
    config.rs             # UserConfig (TOML), atomic save, load-with-error fallback
```

## Commands

All commands run from `apps/desktop/`.

```bash
# Dev (run manually — long-running)
npm run tauri dev

# Type-check frontend
npm run check

# Build
npm run tauri build

# Rust tests (ring_buffer has unit tests)
cd src-tauri && cargo test

# Rust type/lint check
cd src-tauri && cargo check
cd src-tauri && cargo clippy
```

## Architecture: Data Flow

**Frontend → Backend**: `invoke("session_create" | "session_write" | "session_park" | "session_recall" | ...)` via Tauri commands.

**Backend → Frontend**: All Rust-side events go through `mpsc::UnboundedSender<MuxEvent>` → `events.rs` batches them at 12ms → emits as `"mux-event"` Tauri event → `App.svelte` dispatches to terminal APIs or updates session state.

**Session state in frontend**: `App.svelte` holds the `sessions: SessionSnapshot[]` array as the single source of truth. `hotSessions` and `warmSessions` are derived from `thermalState`.

**Terminal instances**: `terminalApis` is a `Map<sessionId, { writeOutput }>` — populated when `TerminalPane` mounts and calls `onTerminalReady`. Output events route through this map.

## Architecture: Key Invariants

- The Rust `SessionManager` is the truth source for session state. Frontend reflects it via snapshots.
- Hot sessions: xterm instance lives in DOM, output written directly via `writeOutput`.
- Warm sessions: xterm destroyed, PTY alive, output goes to `OutputRingBuffer`. On recall, `ReplayStart/ReplayChunk/ReplayEnd` events replay the buffer through `terminalReplay.ts` before live output resumes.
- Events are batched in Rust (12ms) to avoid flooding the bridge under high-output conditions.
- Config is stored at `~/Library/Application Support/vibemux/config.toml` (macOS) or `~/.config/vibemux/config.toml` (Linux/Windows). Writes are atomic (write temp → rename).

## Navigation Mode

Prefix key: `Cmd+Space` (Mac) / `Ctrl+Space` (other). Toggles `navMode` in `App.svelte`. While active: `h/l` switch hot sessions, `j/k` select shelf, `Enter` recalls, `b` parks, `n` new session, `r` rename, `x` close, `X` kill, `/` search, `?` help.

## Adding a New Tauri Command

1. Add handler in `commands.rs` with `#[tauri::command]`
2. Register it in `lib.rs` `invoke_handler!()`
3. Call from frontend with `invoke("command_name", { ...args })`

## Adding a New MuxEvent

1. Add variant to `MuxEvent` enum in `session_manager.rs`
2. Add corresponding `FrontendEvent` variant in `events.rs` and handle in `convert_event()`
3. Add to `MuxEvent` union type in `src/lib/types.ts`
4. Handle in `App.svelte`'s `listen("mux-event", ...)` callback

## Config

`UserConfig` has four sections: `terminal`, `theme`, `layout`, `shell`. All fields have `#[serde(default)]` so partial configs are safe. Frontend reads via `invoke("config_get")`, updates via `invoke("config_update", { update: partialJson })` which deep-merges and atomically saves.
