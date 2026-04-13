<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import Deck from "./lib/Deck.svelte";
  import type { MuxEvent, SessionSnapshot, WorkspaceSnapshot } from "./lib/types";

  let sessions: SessionSnapshot[] = $state([]);
  let focusedSessionId: string | null = $state(null);
  let error: string | null = $state(null);
  let terminalApis: Map<string, { writeOutput: (data: string) => void }> = new Map();
  let unlisten: (() => void) | null = null;

  onMount(async () => {
    // Listen for mux events
    unlisten = await listen<MuxEvent>("mux-event", (event) => {
      const muxEvent = event.payload;
      if (muxEvent.type === "sessionOutput") {
        terminalApis.get(muxEvent.sessionId)?.writeOutput(muxEvent.data);
      }
    });

    // Create first session
    try {
      const home = await getHomeDir();
      const snapshot: SessionSnapshot = await invoke("session_create", {
        payload: {
          name: "shell",
          cwd: home,
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
</script>

<main>
  {#if error}
    <div class="error">{error}</div>
  {:else if sessions.length > 0}
    <Deck
      {sessions}
      {focusedSessionId}
      onTerminalReady={handleTerminalReady}
      onFocusSession={handleFocusSession}
    />
  {:else}
    <div class="loading">Starting session...</div>
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
</style>
