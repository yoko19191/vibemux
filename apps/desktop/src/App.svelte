<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import Deck from "./lib/Deck.svelte";
  import Shelf from "./lib/Shelf.svelte";
  import NewSessionPanel from "./lib/NewSessionPanel.svelte";
  import { onReplayStart, onReplayChunk, onReplayEnd, cancelReplay } from "./lib/terminalReplay";
  import type { MuxEvent, SessionSnapshot } from "./lib/types";

  let sessions: SessionSnapshot[] = $state([]);
  let focusedSessionId: string | null = $state(null);
  let error: string | null = $state(null);
  let navMode = $state(false);
  let showNewSession = $state(false);
  let homeCwd = $state("/");
  let terminalApis: Map<string, { writeOutput: (data: string) => void }> = new Map();
  let restoringSessionIds: Set<string> = $state(new Set());
  let unlisten: (() => void) | null = null;
  let selectedShelfIdx: number | null = $state(null);
  let renamingSessionId: string | null = $state(null);

  const isMac = navigator.platform.toUpperCase().includes("MAC");

  let hotSessions = $derived(sessions.filter((s) => s.thermalState === "Hot"));
  let warmSessions = $derived(sessions.filter((s) => s.thermalState === "Warm"));

  onMount(async () => {
    unlisten = await listen<MuxEvent>("mux-event", (event) => {
      const muxEvent = event.payload;
      if (muxEvent.type === "sessionOutput") {
        terminalApis.get(muxEvent.sessionId)?.writeOutput(muxEvent.data);
      } else if (muxEvent.type === "sessionExited") {
        cancelReplay(muxEvent.sessionId);
        sessions = sessions.filter((s) => s.id !== muxEvent.sessionId);
        terminalApis.delete(muxEvent.sessionId);
        restoringSessionIds = new Set([...restoringSessionIds].filter((id) => id !== muxEvent.sessionId));
        if (focusedSessionId === muxEvent.sessionId) {
          focusedSessionId = sessions[0]?.id ?? null;
        }
      } else if (muxEvent.type === "sessionParked") {
        sessions = sessions.map((s) =>
          s.id === muxEvent.sessionId ? { ...s, thermalState: "Warm" as const } : s
        );
        if (focusedSessionId === muxEvent.sessionId) {
          const hotSessions = sessions.filter((s) => s.thermalState === "Hot");
          focusedSessionId = hotSessions[0]?.id ?? null;
        }
      } else if (muxEvent.type === "replayStart") {
        sessions = sessions.map((s) =>
          s.id === muxEvent.sessionId ? { ...s, thermalState: "Hot" as const } : s
        );
        focusedSessionId = muxEvent.sessionId;
        const api = terminalApis.get(muxEvent.sessionId);
        if (api) {
          onReplayStart(
            muxEvent.sessionId,
            api.writeOutput,
            (active) => {
              restoringSessionIds = new Set(
                active
                  ? [...restoringSessionIds, muxEvent.sessionId]
                  : [...restoringSessionIds].filter((id) => id !== muxEvent.sessionId)
              );
            }
          );
        }
      } else if (muxEvent.type === "replayChunk") {
        onReplayChunk(muxEvent.sessionId, muxEvent.data);
      } else if (muxEvent.type === "replayEnd") {
        onReplayEnd(muxEvent.sessionId);
      } else if (muxEvent.type === "attentionChanged") {
        sessions = sessions.map((s) =>
          s.id === muxEvent.sessionId ? { ...s, attentionState: muxEvent.attentionState } : s
        );
      }
    });

    try {
      homeCwd = await getHomeDir();
      const snapshot: SessionSnapshot = await invoke("session_create", {
        payload: {
          name: "shell",
          cwd: homeCwd,
          commandType: "shell",
        },
      });
      sessions = [snapshot];
      focusedSessionId = snapshot.id;
    } catch (e) {
      error = `Failed to create session: ${e}`;
    }
  });

  onDestroy(() => {
    unlisten?.();
  });

  async function getHomeDir(): Promise<string> {
    try {
      const { homeDir } = await import("@tauri-apps/api/path");
      return await homeDir();
    } catch {
      return "/";
    }
  }

  function handleTerminalReady(sessionId: string, api: { writeOutput: (data: string) => void }) {
    terminalApis.set(sessionId, api);
  }

  async function handleFocusSession(sessionId: string) {
    try {
      await invoke("session_focus", { sessionId });
      focusedSessionId = sessionId;
    } catch (e) {
      console.error("Failed to focus session:", e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    // Prefix key: Cmd+Space (Mac) or Ctrl+Space (other)
    const prefixKey = e.code === "Space" && (isMac ? e.metaKey : e.ctrlKey);

    if (prefixKey) {
      e.preventDefault();
      if (showNewSession) {
        showNewSession = false;
      }
      navMode = !navMode;
      return;
    }

    if (!navMode) return;

    // Navigation mode keybindings
    switch (e.key) {
      case "Escape":
        navMode = false;
        e.preventDefault();
        break;
      case "h":
      case "ArrowLeft":
        focusPrevSession();
        navMode = false;
        e.preventDefault();
        break;
      case "l":
      case "ArrowRight":
        focusNextSession();
        navMode = false;
        e.preventDefault();
        break;
      case "n":
      case "N":
        showNewSession = true;
        navMode = false;
        e.preventDefault();
        break;
      case "X":
        if (e.shiftKey) {
          killCurrentSession();
        }
        navMode = false;
        e.preventDefault();
        break;
      case "x":
        closeCurrentSession();
        navMode = false;
        e.preventDefault();
        break;
      case "b":
      case "B":
        parkCurrentSession();
        navMode = false;
        e.preventDefault();
        break;
      case "j":
      case "ArrowDown":
        if (warmSessions.length > 0) {
          selectedShelfIdx = selectedShelfIdx === null
            ? 0
            : Math.min(selectedShelfIdx + 1, warmSessions.length - 1);
          e.preventDefault();
        }
        break;
      case "k":
      case "ArrowUp":
        if (warmSessions.length > 0 && selectedShelfIdx !== null) {
          selectedShelfIdx = Math.max(selectedShelfIdx - 1, 0);
          e.preventDefault();
        }
        break;
      case "Enter":
        if (selectedShelfIdx !== null && warmSessions[selectedShelfIdx]) {
          recallSession(warmSessions[selectedShelfIdx].id);
          selectedShelfIdx = null;
          navMode = false;
          e.preventDefault();
        }
        break;
      case "r":
      case "R":
        if (focusedSessionId) {
          renamingSessionId = focusedSessionId;
          navMode = false;
          e.preventDefault();
        }
        break;
    }
  }

  function focusPrevSession() {
    if (sessions.length < 2 || !focusedSessionId) return;
    const idx = sessions.findIndex((s) => s.id === focusedSessionId);
    const prevIdx = (idx - 1 + sessions.length) % sessions.length;
    handleFocusSession(sessions[prevIdx].id);
  }

  function focusNextSession() {
    if (sessions.length < 2 || !focusedSessionId) return;
    const idx = sessions.findIndex((s) => s.id === focusedSessionId);
    const nextIdx = (idx + 1) % sessions.length;
    handleFocusSession(sessions[nextIdx].id);
  }

  async function closeCurrentSession() {
    if (!focusedSessionId) return;
    try {
      await invoke("session_close", { sessionId: focusedSessionId });
      sessions = sessions.filter((s) => s.id !== focusedSessionId);
      terminalApis.delete(focusedSessionId);
      focusedSessionId = sessions[0]?.id ?? null;
    } catch (e) {
      console.error("Failed to close session:", e);
    }
  }

  async function killCurrentSession() {
    if (!focusedSessionId) return;
    try {
      await invoke("session_kill", { sessionId: focusedSessionId });
      sessions = sessions.filter((s) => s.id !== focusedSessionId);
      terminalApis.delete(focusedSessionId);
      focusedSessionId = sessions[0]?.id ?? null;
    } catch (e) {
      console.error("Failed to kill session:", e);
    }
  }

  async function parkCurrentSession() {
    if (!focusedSessionId) return;
    try {
      await invoke("session_park", { sessionId: focusedSessionId });
    } catch (e) {
      console.error("Failed to park session:", e);
    }
  }

  async function recallSession(sessionId: string) {
    try {
      await invoke("session_recall", { sessionId });
    } catch (e) {
      console.error("Failed to recall session:", e);
    }
  }

  async function handleRenameConfirm(sessionId: string, name: string) {
    renamingSessionId = null;
    try {
      await invoke("session_rename", { sessionId, name });
      sessions = sessions.map((s) => s.id === sessionId ? { ...s, name } : s);
    } catch (e) {
      console.error("Failed to rename session:", e);
    }
  }

  function handleRenameCancel() {
    renamingSessionId = null;
  }

  function handleSessionCreated(snapshot: SessionSnapshot) {
    sessions = [...sessions, snapshot];
    focusedSessionId = snapshot.id;
    showNewSession = false;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<main class:has-shelf={warmSessions.length > 0}>
  {#if error}
    <div class="error">{error}</div>
  {:else if hotSessions.length > 0}
    <Deck
      sessions={hotSessions}
      {focusedSessionId}
      {renamingSessionId}
      onTerminalReady={handleTerminalReady}
      onFocusSession={handleFocusSession}
      onRenameConfirm={handleRenameConfirm}
      onRenameCancel={handleRenameCancel}
    />
  {:else if sessions.length > 0}
    <div class="loading">All sessions parked</div>
  {:else}
    <div class="loading">Starting session...</div>
  {/if}

  <Shelf sessions={warmSessions} onRecall={recallSession} selectedIdx={selectedShelfIdx} />

  {#if navMode}
    <div class="nav-indicator" class:shelf-offset={warmSessions.length > 0}>
      <span class="nav-badge">NAV</span>
      <span class="nav-hint">h/l: switch · n: new · b: park · j/k: shelf · Enter: recall · x: close · X: kill · esc: cancel</span>
    </div>
  {/if}

  {#if showNewSession}
    <NewSessionPanel
      defaultCwd={homeCwd}
      onCreated={handleSessionCreated}
      onCancel={() => (showNewSession = false)}
    />
  {/if}
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: #111111;
  }

  main {
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    position: relative;
  }

  .error {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #ff6b57;
    font-family: system-ui, -apple-system, sans-serif;
    font-size: 1rem;
    padding: 2rem;
    text-align: center;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #d9d4c7;
    font-family: system-ui, -apple-system, sans-serif;
    font-size: 1rem;
  }

  .nav-indicator {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.4rem 0.75rem;
    background: rgba(59, 130, 246, 0.15);
    border-top: 1px solid #3b82f6;
    font-family: system-ui, -apple-system, sans-serif;
    z-index: 50;
  }

  .nav-indicator.shelf-offset {
    bottom: 62px;
  }

  .nav-badge {
    background: #3b82f6;
    color: white;
    padding: 0.1rem 0.4rem;
    border-radius: 3px;
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.05em;
  }

  .nav-hint {
    color: #999;
    font-size: 0.75rem;
  }
</style>
