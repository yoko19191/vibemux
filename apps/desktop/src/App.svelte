<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Deck from "./lib/Deck.svelte";
  import Shelf from "./lib/Shelf.svelte";
  import NewSessionPanel from "./lib/NewSessionPanel.svelte";
  import SettingsPanel from "./lib/SettingsPanel.svelte";
  import SessionSearch from "./lib/SessionSearch.svelte";
  import HelpOverlay from "./lib/HelpOverlay.svelte";
  import Titlebar from "./lib/Titlebar.svelte";
  import Onboarding from "./lib/Onboarding.svelte";
  import { onReplayStart, onReplayChunk, onReplayEnd, cancelReplay } from "./lib/terminalReplay";
  import type { MuxEvent, SessionSnapshot } from "./lib/types";
  import { parsePrefixKey, matchesPrefixKey, formatPrefixKey } from "./lib/keymap";
  import type { PrefixKeyMatcher } from "./lib/keymap";

  let sessions: SessionSnapshot[] = $state([]);
  let focusedSessionId: string | null = $state(null);
  let error: string | null = $state(null);
  let configError: string | null = $state(null);
  let navMode = $state(false);
  let showNewSession = $state(false);
  let showSettings = $state(false);
  let settingsInitialTab: "terminal" | "theme" | "layout" | "keys" | "ai" = $state("terminal");
  let showSearch = $state(false);
  let searchQuery = $state("");
  let showHelp = $state(false);
  let homeCwd = $state("/");
  let terminalApis: Map<string, { writeOutput: (data: string) => void; resetAndResize: () => void; focus: () => void; blur: () => void }> = new Map();
  let restoringSessionIds: Set<string> = $state(new Set());
  let pendingAttachSessionIds: Set<string> = new Set(); // Sessions waiting for TerminalPane mount to trigger reset+resize
  let unlisten: (() => void) | null = null;
  let selectedShelfIdx: number | null = $state(null);
  let renamingSessionId: string | null = $state(null);
  let prefixKeyConfig = $state("ctrl+b");
  let prefixKeyMatcher: PrefixKeyMatcher = $derived(parsePrefixKey(prefixKeyConfig));
  let prefixKeyDisplay = $derived(formatPrefixKey(prefixKeyConfig));
  let showOnboarding = $state(false);
  let maxHotSessions = $state(6);
  let hotSessionLimitWarning: { limit: number } | null = $state(null);

  // Terminal config derived from user config — passed to all TerminalPane instances
  let terminalConfig: { fontFamily?: string; fontSize?: number; lineHeight?: number; theme?: Record<string, string> } = $state({});

  let hotSessions = $derived(sessions.filter((s) => s.thermalState === "Hot"));
  let warmSessions = $derived(sessions.filter((s) => s.thermalState === "Warm"));

  function readMaxHotSessions(cfg: any): number {
    const value = Number(cfg?.layout?.max_hot_sessions);
    return Number.isFinite(value) && value > 0 ? Math.floor(value) : 6;
  }

  function buildTerminalConfig(cfg: any) {
    if (!cfg) return undefined;
    const t = cfg.theme ?? {};
    return {
      fontFamily: cfg.terminal?.font_family,
      fontSize: cfg.terminal?.font_size,
      lineHeight: cfg.terminal?.line_height,
      theme: {
        background: t.background,
        foreground: t.foreground,
        cursor: t.cursor,
        selectionBackground: t.selection,
        black: t.black,
        red: t.red,
        green: t.green,
        yellow: t.yellow,
        blue: t.blue,
        magenta: t.magenta,
        cyan: t.cyan,
        white: t.white,
        brightBlack: t.bright_black,
        brightRed: t.bright_red,
        brightGreen: t.bright_green,
        brightYellow: t.bright_yellow,
        brightBlue: t.bright_blue,
        brightMagenta: t.bright_magenta,
        brightCyan: t.bright_cyan,
        brightWhite: t.bright_white,
      },
    };
  }

  // Dynamic window title
  $effect(() => {
    const focused = sessions.find((s) => s.id === focusedSessionId && s.thermalState === "Hot");
    let title = "Vibemux";
    if (focused) {
      const name = focused.customName ?? focused.name;
      const busy = focused.processState.type === "Running";
      const dynamic = focused.terminalTitle && focused.terminalTitle !== name ? focused.terminalTitle : null;
      const shortCwd = focused.cwd.replace(/^.*\/([^/]+)\/?$/, "$1") || focused.cwd;
      const hotCount = hotSessions.length;
      const warmCount = warmSessions.length;
      const sessionInfo = warmCount > 0 ? `[${hotCount}+${warmCount}]` : `[${hotCount}]`;
      const label = dynamic ?? name;
      title = busy ? `⚙ ${label} · ${shortCwd} ${sessionInfo} — Vibemux` : `${label} · ${shortCwd} ${sessionInfo} — Vibemux`;
    }
    getCurrentWindow().setTitle(title).catch(() => {});
  });

  // Manage terminal focus based on nav mode:
  // blur xterm when entering nav mode so keys reach window handler,
  // refocus when leaving so typing goes back to the terminal.
  $effect(() => {
    if (!focusedSessionId) return;
    const api = terminalApis.get(focusedSessionId);
    if (navMode) {
      api?.blur();
    } else {
      api?.focus();
    }
  });

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
          focusedSessionId = sessions.find((s) => s.thermalState === "Hot")?.id ?? null;
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
        // replayStart is no longer emitted (we skip replay and send SIGWINCH instead)
        // so we update thermalState → Hot and focus here on replayEnd
        const isWarm = sessions.find((s) => s.id === muxEvent.sessionId)?.thermalState === "Warm";
        if (isWarm) {
          sessions = sessions.map((s) =>
            s.id === muxEvent.sessionId ? { ...s, thermalState: "Hot" as const } : s
          );
          focusedSessionId = muxEvent.sessionId;
        }
      } else if (muxEvent.type === "attentionChanged") {
        sessions = sessions.map((s) =>
          s.id === muxEvent.sessionId ? { ...s, attentionState: muxEvent.attentionState } : s
        );
      } else if (muxEvent.type === "sessionUpdated") {
        // Refresh session snapshot to pick up title/name/color changes
        invoke<import("./lib/types").SessionSnapshot>("session_get", { sessionId: muxEvent.sessionId })
          .then((snap) => {
            sessions = sessions.map((s) => s.id === snap.id ? snap : s);
          })
          .catch(() => {
            // session_get not available — do a full workspace refresh
            invoke<import("./lib/types").WorkspaceSnapshot>("workspace_get_snapshot")
              .then((ws) => { sessions = ws.sessions; })
              .catch(console.error);
          });
      }
    });

    try {
      homeCwd = await getHomeDir();
      console.log("[vibemux] homeCwd:", homeCwd);
      // Check for config load error
      const cfgErr = await invoke<string | null>("config_get_error");
      console.log("[vibemux] config_get_error done:", cfgErr);
      if (cfgErr) configError = cfgErr;

      // Load prefix key from config and check onboarding
      try {
        const cfg = await invoke<any>("config_get");
        if (cfg?.keys?.prefix) {
          prefixKeyConfig = cfg.keys.prefix;
        }
        maxHotSessions = readMaxHotSessions(cfg);
        terminalConfig = buildTerminalConfig(cfg) ?? {};
        if (!cfg?.onboarding_completed) {
          showOnboarding = true;
          return; // don't create session yet — onboarding will trigger it
        }
      } catch {
        // keep default
      }

      await createInitialSession();
    } catch (e) {
      console.error("[vibemux] session_create failed:", e);
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

  async function createInitialSession() {
    try {
      console.log("[vibemux] calling session_create...");
      const snapshot: SessionSnapshot = await invoke("session_create", {
        payload: {
          name: "shell",
          cwd: homeCwd,
          commandType: "shell",
        },
      });
      console.log("[vibemux] session_create returned:", JSON.stringify(snapshot));
      sessions = [snapshot];
      focusedSessionId = snapshot.id;
    } catch (e) {
      console.error("[vibemux] session_create failed:", e);
      error = `Failed to create session: ${e}`;
    }
  }

  async function handleOnboardingComplete(prefixKey: string) {
    showOnboarding = false;
    prefixKeyConfig = prefixKey;
    await createInitialSession();
  }

  function handleTerminalReady(sessionId: string, api: { writeOutput: (data: string) => void; resetAndResize: () => void; focus: () => void; blur: () => void }) {
    terminalApis.set(sessionId, api);
    // If this session was just recalled, trigger reset+resize then focus
    if (pendingAttachSessionIds.has(sessionId)) {
      pendingAttachSessionIds.delete(sessionId);
      api.resetAndResize();
      api.focus();
    }
  }

  function requestNewSession() {
    navMode = false;
    if (hotSessions.length >= maxHotSessions) {
      hotSessionLimitWarning = { limit: maxHotSessions };
      showNewSession = false;
      return;
    }
    showNewSession = true;
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
    if (matchesPrefixKey(e, prefixKeyMatcher)) {
      e.preventDefault();
      if (showNewSession) {
        showNewSession = false;
      }
      navMode = !navMode;
      return;
    }

    // CMD+K (Mac) / Ctrl+K global search — works without nav mode
    const isMac = navigator.platform.toUpperCase().includes("MAC");
    if ((isMac ? e.metaKey : e.ctrlKey) && e.key === "k") {
      e.preventDefault();
      showSearch = !showSearch;
      navMode = false;
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
        requestNewSession();
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
      case "/":
        showSearch = true;
        navMode = false;
        e.preventDefault();
        break;
      case "?":
        showHelp = true;
        navMode = false;
        e.preventDefault();
        break;
    }
  }

  function handleSearchSelect(sessionId: string, thermal: "Hot" | "Warm") {
    showSearch = false;
    if (thermal === "Hot") {
      handleFocusSession(sessionId);
    } else {
      recallSession(sessionId);
    }
  }

  function focusPrevSession() {
    if (hotSessions.length < 2 || !focusedSessionId) return;
    const idx = hotSessions.findIndex((s) => s.id === focusedSessionId);
    if (idx === -1) {
      handleFocusSession(hotSessions[0].id);
      return;
    }
    const prevIdx = (idx - 1 + hotSessions.length) % hotSessions.length;
    handleFocusSession(hotSessions[prevIdx].id);
  }

  function focusNextSession() {
    if (hotSessions.length < 2 || !focusedSessionId) return;
    const idx = hotSessions.findIndex((s) => s.id === focusedSessionId);
    if (idx === -1) {
      handleFocusSession(hotSessions[0].id);
      return;
    }
    const nextIdx = (idx + 1) % hotSessions.length;
    handleFocusSession(hotSessions[nextIdx].id);
  }

  async function closeCurrentSession() {
    if (!focusedSessionId) return;
    try {
      await invoke("session_close", { sessionId: focusedSessionId });
      sessions = sessions.filter((s) => s.id !== focusedSessionId);
      terminalApis.delete(focusedSessionId);
      focusedSessionId = sessions.find((s) => s.thermalState === "Hot")?.id ?? null;
    } catch (e) {
      console.error("Failed to close session:", e);
    }
  }

  async function killCurrentSession() {
    if (!focusedSessionId) return;
    try {
      await killSessionById(focusedSessionId);
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
      // TerminalPane hasn't mounted yet — mark for reset+resize on next onReady
      pendingAttachSessionIds.add(sessionId);
    } catch (e) {
      console.error("Failed to recall session:", e);
      if (String(e).includes("Hot Session limit reached")) {
        hotSessionLimitWarning = { limit: maxHotSessions };
      }
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

  async function closeSessionById(sessionId: string) {
    try {
      await invoke("session_close", { sessionId });
      sessions = sessions.filter((s) => s.id !== sessionId);
      terminalApis.delete(sessionId);
      if (focusedSessionId === sessionId) {
        focusedSessionId = sessions.find((s) => s.thermalState === "Hot")?.id ?? null;
      }
    } catch (e) {
      console.error("Failed to close session:", e);
    }
  }

  async function killSessionById(sessionId: string) {
    await invoke("session_kill", { sessionId });
    sessions = sessions.filter((s) => s.id !== sessionId);
    terminalApis.delete(sessionId);
    if (focusedSessionId === sessionId) {
      focusedSessionId = sessions.find((s) => s.thermalState === "Hot")?.id ?? null;
    }
  }

  async function handleShelfRename(sessionId: string, name: string) {
    try {
      await invoke("session_rename", { sessionId, name });
      sessions = sessions.map((s) => s.id === sessionId ? { ...s, name } : s);
    } catch (e) {
      console.error("Failed to rename session:", e);
    }
  }

  async function handleShelfSetColor(sessionId: string, color: string) {
    try {
      await invoke("session_set_color", { sessionId, color });
    } catch (e) {
      console.error("Failed to set color:", e);
    }
  }

  async function handleShelfKill(sessionId: string) {
    try {
      await killSessionById(sessionId);
    } catch (e) {
      console.error("Failed to kill session:", e);
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<main class:has-detached={warmSessions.length > 0}>
  <Titlebar
    prefixKey={prefixKeyDisplay}
    onNewSession={requestNewSession}
    onSearch={() => { showSearch = true; navMode = false; }}
    onSettings={() => {
      settingsInitialTab = "terminal";
      showSettings = true;
    }}
  />

  <div class="content-area" class:has-detached={warmSessions.length > 0}>
    {#if configError}
      <div class="config-error-banner">
        <span class="config-error-icon">⚠</span>
        <span class="config-error-msg">Config reset to defaults: {configError}</span>
        <button class="config-error-dismiss" onclick={() => (configError = null)}>✕</button>
      </div>
    {/if}

    {#if error}
      <div class="error">{error}</div>
    {:else if hotSessions.length > 0}
      <Deck
        sessions={hotSessions}
        {focusedSessionId}
        {renamingSessionId}
        {terminalConfig}
        {prefixKeyMatcher}
        onTerminalReady={handleTerminalReady}
        onFocusSession={handleFocusSession}
        onRenameConfirm={handleRenameConfirm}
        onRenameCancel={handleRenameCancel}
        onStartRename={(id) => { renamingSessionId = id; }}
        onPark={(sessionId) => invoke("session_park", { sessionId }).catch(console.error)}
        onClose={closeSessionById}
      />
    {:else if sessions.length > 0}
      <div class="loading">All sessions parked</div>
    {:else}
      <div class="loading">Starting session...</div>
    {/if}
  </div>

  <Shelf
    sessions={warmSessions}
    onRecall={recallSession}
    selectedIdx={selectedShelfIdx}
    onRename={handleShelfRename}
    onSetColor={handleShelfSetColor}
    onClose={closeSessionById}
    onKill={handleShelfKill}
  />

  {#if navMode}
    <div class="nav-indicator" class:detached-offset={warmSessions.length > 0}>
      <span class="nav-badge">NAV ({prefixKeyDisplay})</span>
      <span class="nav-hint">h/l: switch · n: new · b: detach · j/k: detached · Enter: attach · x: close · X: kill · esc: cancel</span>
    </div>
  {/if}

  {#if showNewSession}
    <NewSessionPanel
      defaultCwd={homeCwd}
      onCreated={handleSessionCreated}
      onCancel={() => (showNewSession = false)}
    />
  {/if}

  {#if showSettings}
    <SettingsPanel
      initialTab={settingsInitialTab}
      onClose={() => (showSettings = false)}
      onConfigChange={(cfg) => {
        if (cfg?.keys?.prefix) prefixKeyConfig = cfg.keys.prefix;
        maxHotSessions = readMaxHotSessions(cfg);
        terminalConfig = buildTerminalConfig(cfg) ?? {};
      }}
    />
  {/if}

  {#if hotSessionLimitWarning}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-overlay" onclick={() => (hotSessionLimitWarning = null)}>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="limit-dialog" onclick={(e) => e.stopPropagation()}>
        <div class="limit-title">Hot Session limit reached</div>
        <p>
          Current Hot Session limit is {hotSessionLimitWarning.limit}. Park or close a Hot Session first.
          To adjust the limit, open Settings and change Max Hot Sessions under Layout.
        </p>
        <div class="limit-actions">
          <button class="secondary-action" onclick={() => (hotSessionLimitWarning = null)}>OK</button>
          <button
            class="primary-action"
            onclick={() => {
              hotSessionLimitWarning = null;
              settingsInitialTab = "layout";
              showSettings = true;
            }}
          >
            Open Settings
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if showSearch}
    <SessionSearch
      sessions={sessions}
      query={searchQuery}
      onQueryChange={(value) => (searchQuery = value)}
      onSelect={handleSearchSelect}
      onNewSession={() => { showSearch = false; requestNewSession(); }}
      onKillSession={killSessionById}
      onClose={() => (showSearch = false)}
    />
  {/if}

  {#if showHelp}
    <HelpOverlay
      prefixKey={prefixKeyDisplay}
      onClose={() => (showHelp = false)}
    />
  {/if}

  {#if showOnboarding}
    <Onboarding onComplete={handleOnboardingComplete} />
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

  .content-area {
    position: absolute;
    top: 36px;
    left: 0;
    right: 0;
    bottom: 0;
    overflow: hidden;
  }

  .content-area.has-detached {
    bottom: 31px;
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

  .nav-indicator.detached-offset {
    bottom: 31px;
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

  .config-error-banner {
    position: fixed;
    top: 36px;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.75rem;
    background: rgba(234, 179, 8, 0.15);
    border-bottom: 1px solid #eab308;
    font-family: system-ui, -apple-system, sans-serif;
    z-index: 300;
  }

  .config-error-icon {
    color: #eab308;
    font-size: 0.85rem;
    flex-shrink: 0;
  }

  .config-error-msg {
    color: #d9d4c7;
    font-size: 0.75rem;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .config-error-dismiss {
    background: none;
    border: none;
    color: #888;
    cursor: pointer;
    font-size: 0.75rem;
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .config-error-dismiss:hover {
    color: #d9d4c7;
    background: #2a2a2a;
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    z-index: 250;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.6);
    font-family: system-ui, -apple-system, sans-serif;
  }

  .limit-dialog {
    width: 380px;
    max-width: calc(100vw - 2rem);
    background: #1a1a1a;
    border: 1px solid #3a3a3a;
    border-radius: 8px;
    color: #d9d4c7;
    padding: 1rem;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.45);
  }

  .limit-title {
    color: #f5f5f5;
    font-size: 0.95rem;
    font-weight: 600;
    margin-bottom: 0.55rem;
  }

  .limit-dialog p {
    color: #aaa;
    font-size: 0.82rem;
    line-height: 1.45;
    margin: 0;
  }

  .limit-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }

  .limit-actions button {
    border: 1px solid transparent;
    border-radius: 4px;
    cursor: pointer;
    font: inherit;
    font-size: 0.8rem;
    padding: 0.45rem 0.8rem;
  }

  .secondary-action {
    background: #2a2a2a;
    color: #d9d4c7;
  }

  .secondary-action:hover {
    background: #333;
  }

  .primary-action {
    background: #3b82f6;
    color: #fff;
  }

  .primary-action:hover {
    background: #2563eb;
  }
</style>
