<script lang="ts">
  import { onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import MarkdownMessage from "./MarkdownMessage.svelte";
  import type { AiConfig, AiEvent, AiMessage, AiThread, AiThreadSummary, SessionSnapshot } from "./types";

  interface UserConfig {
    ai: AiConfig;
  }

  interface Props {
    sessions: SessionSnapshot[];
    query: string;
    onQueryChange: (query: string) => void;
    onSelect: (sessionId: string, thermal: "Hot" | "Warm") => void;
    onNewSession: () => void;
    onKillSession: (sessionId: string) => Promise<void> | void;
    onClose: () => void;
  }

  type PaletteItem =
    | { type: "ask-ai"; disabled: boolean; reason: string }
    | { type: "new-session" }
    | { type: "session"; session: SessionSnapshot }
    | { type: "thread"; thread: AiThreadSummary };

  let { sessions, query: initialQuery, onQueryChange, onSelect, onNewSession, onKillSession, onClose }: Props = $props();

  let query = $state("");
  let selectedIdx = $state(0);
  let inputEl: HTMLInputElement | null = $state(null);
  let mode: "search" | "chat" = $state("search");
  let aiConfig = $state<AiConfig | null>(null);
  let threads: AiThreadSummary[] = $state([]);
  let activeThread: AiThread | null = $state(null);
  let includeFocusedContext = $state(false);
  let aiError: string | null = $state(null);
  let sending = $state(false);
  let activeRequestId: string | null = $state(null);
  let pendingKillSession: SessionSnapshot | null = $state(null);
  let killConfirmButton: HTMLButtonElement | null = $state(null);
  let unlistenAi: (() => void) | null = null;
  let pendingDeltas = new Map<string, string>();
  let pendingDone = new Set<string>();
  let pendingErrors = new Map<string, string>();

  function getInitialQuery() {
    return initialQuery;
  }

  function isChatTrigger(value: string): boolean {
    return value.startsWith("#");
  }

  function promptFromTrigger(value: string): string {
    return isChatTrigger(value) ? value.slice(1).trimStart() : value;
  }

  query = promptFromTrigger(getInitialQuery());
  mode = isChatTrigger(getInitialQuery()) ? "chat" : "search";

  import { colorMap } from "./colors";

  const thermalLabel: Record<string, string> = {
    Hot: "hot",
    Warm: "warm",
    Cold: "cold",
  };

  let aiReady = $derived.by(() => {
    const cfg = aiConfig;
    return Boolean(cfg && cfg.enabled && cfg.base_url && cfg.api_key && cfg.model);
  });
  let aiUnavailableReason = $derived.by(() => {
    if (!aiConfig) return "Loading AI settings";
    if (!aiConfig.enabled) return "AI is disabled in Settings";
    if (!aiConfig.base_url) return "Add an AI Base URL in Settings";
    if (!aiConfig.api_key) return "Add an AI API Key in Settings";
    if (!aiConfig.model) return "Select an AI model in Settings";
    return "";
  });

  let filteredSessions = $derived.by(() => {
    const q = normalQuery();
    if (!q) return sessions;
    return sessions.filter(
      (s) =>
        s.name.toLowerCase().includes(q) ||
        s.cwd.toLowerCase().includes(q) ||
        (s.terminalTitle && s.terminalTitle.toLowerCase().includes(q))
    );
  });

  let filteredThreads = $derived.by(() => {
    const q = normalQuery();
    if (!q) return threads;
    return threads.filter(
      (thread) =>
        thread.title.toLowerCase().includes(q) ||
        thread.lastMessagePreview.toLowerCase().includes(q)
    );
  });

  let paletteItems = $derived.by(() => [
    { type: "ask-ai", disabled: !aiReady, reason: aiUnavailableReason } as PaletteItem,
    { type: "new-session" } as PaletteItem,
    ...filteredSessions.map((session) => ({ type: "session", session }) as PaletteItem),
    ...filteredThreads.map((thread) => ({ type: "thread", thread }) as PaletteItem),
  ]);

  $effect(() => {
    if (isChatTrigger(query)) {
      query = promptFromTrigger(query);
      mode = "chat";
    }
    onQueryChange(query);
  });

  $effect(() => {
    query;
    filteredSessions.length;
    filteredThreads.length;
    selectedIdx = 0;
  });

  $effect(() => {
    if (pendingKillSession) {
      setTimeout(() => killConfirmButton?.focus(), 0);
    }
  });

  setTimeout(() => inputEl?.focus(), 0);
  loadInitialData();
  listen<AiEvent>("ai-event", (event) => handleAiEvent(event.payload)).then((unlisten) => {
    unlistenAi = unlisten;
  });

  onDestroy(() => {
    unlistenAi?.();
  });

  async function loadInitialData() {
    await Promise.all([loadConfig(), loadThreads()]);
  }

  async function loadConfig() {
    try {
      const cfg = await invoke<UserConfig>("config_get");
      aiConfig = cfg.ai;
    } catch (e) {
      aiError = String(e);
    }
  }

  async function loadThreads() {
    try {
      threads = await invoke<AiThreadSummary[]>("ai_list_threads");
    } catch (e) {
      aiError = String(e);
    }
  }

  function normalQuery(): string {
    return mode === "chat" ? "" : query.toLowerCase().trim();
  }

  function aiInstruction(): string {
    return query.trim();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (pendingKillSession) {
      return;
    }

    if (e.key === "Escape") {
      e.preventDefault();
      if (mode === "chat") {
        mode = "search";
        activeThread = null;
        query = "";
        aiError = null;
      } else {
        onClose();
      }
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIdx = Math.min(selectedIdx + 1, Math.max(paletteItems.length - 1, 0));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIdx = Math.max(selectedIdx - 1, 0);
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (mode === "chat" || isChatTrigger(query)) {
        sendAiMessage();
      } else {
        activatePaletteItem(paletteItems[selectedIdx]);
      }
    }
  }

  function handleKillDialogKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      pendingKillSession = null;
    }
  }

  function activatePaletteItem(item: PaletteItem | undefined) {
    if (!item) return;
    if (item.type === "ask-ai") {
      if (item.disabled) {
        aiError = item.reason;
        return;
      }
      activeThread = null;
      query = "";
      mode = "chat";
      setTimeout(() => inputEl?.focus(), 0);
    } else if (item.type === "new-session") {
      onNewSession();
    } else if (item.type === "session") {
      onSelect(item.session.id, item.session.thermalState as "Hot" | "Warm");
    } else if (item.type === "thread") {
      openThread(item.thread.id);
    }
  }

  function requestKillSession(e: MouseEvent, session: SessionSnapshot) {
    e.stopPropagation();
    aiError = null;
    pendingKillSession = session;
  }

  async function confirmKillSession() {
    if (!pendingKillSession) return;
    const sessionId = pendingKillSession.id;
    try {
      await onKillSession(sessionId);
      pendingKillSession = null;
    } catch (e) {
      aiError = String(e);
    }
  }

  async function deleteThread(e: MouseEvent, thread: AiThreadSummary) {
    e.stopPropagation();
    aiError = null;
    try {
      await invoke("ai_delete_thread", { threadId: thread.id });
      threads = threads.filter((t) => t.id !== thread.id);
      if (activeThread?.id === thread.id) {
        activeThread = null;
      }
    } catch (err) {
      aiError = String(err);
    }
  }

  async function openThread(threadId: string) {
    try {
      activeThread = await invoke<AiThread>("ai_get_thread", { threadId });
      query = "";
      mode = "chat";
      aiError = null;
      setTimeout(() => inputEl?.focus(), 0);
    } catch (e) {
      aiError = String(e);
    }
  }

  async function sendAiMessage() {
    aiError = null;
    if (!aiReady) {
      aiError = aiUnavailableReason;
      return;
    }
    const content = aiInstruction();
    if (!content || sending) return;
    sending = true;
    try {
      const result = await invoke<{ requestId: string; threadId: string; assistantMessageId: string }>("ai_send_message", {
        payload: {
          threadId: activeThread?.id ?? null,
          content,
          includeFocusedContext,
        },
      });
      activeRequestId = result.requestId;
      appendOptimisticMessages(result.threadId, result.assistantMessageId, content);
      query = "";
      includeFocusedContext = false;
      await loadThreads();
    } catch (e) {
      aiError = String(e);
      sending = false;
    }
  }

  function appendOptimisticMessages(threadId: string, assistantMessageId: string, content: string) {
    const now = new Date().toISOString();
    const userMessage: AiMessage = {
      id: crypto.randomUUID(),
      role: "user",
      content,
      createdAt: now,
      metadata: { includeFocusedContext },
    };
    const pendingContent = pendingDeltas.get(assistantMessageId) ?? "";
    const pendingError = pendingErrors.get(assistantMessageId);
    const assistantMessage: AiMessage = {
      id: assistantMessageId,
      role: "assistant",
      content: pendingError ? `Error: ${pendingError}` : pendingContent,
      createdAt: now,
      metadata: null,
    };
    pendingDeltas.delete(assistantMessageId);
    pendingErrors.delete(assistantMessageId);

    if (activeThread?.id === threadId) {
      activeThread = {
        ...activeThread,
        updatedAt: now,
        messages: [...activeThread.messages, userMessage, assistantMessage],
      };
    } else {
      activeThread = {
        id: threadId,
        title: titleFrom(content),
        createdAt: now,
        updatedAt: now,
        messages: [userMessage, assistantMessage],
      };
    }
    if (pendingDone.has(assistantMessageId) || pendingError) {
      sending = false;
      activeRequestId = null;
      pendingDone.delete(assistantMessageId);
    }
  }

  function handleAiEvent(event: AiEvent) {
    if (!activeThread || event.threadId !== activeThread.id) {
      if (event.type === "delta") {
        pendingDeltas.set(
          event.assistantMessageId,
          (pendingDeltas.get(event.assistantMessageId) ?? "") + event.content
        );
      } else if (event.type === "done") {
        pendingDone.add(event.assistantMessageId);
      } else if (event.type === "error") {
        pendingErrors.set(event.assistantMessageId, event.message);
      }
      return;
    }
    if (event.type === "delta") {
      activeThread = {
        ...activeThread,
        messages: activeThread.messages.map((message) =>
          message.id === event.assistantMessageId
            ? { ...message, content: message.content + event.content }
            : message
        ),
      };
    } else if (event.type === "done") {
      sending = false;
      activeRequestId = null;
      loadThreads();
    } else if (event.type === "error") {
      sending = false;
      activeRequestId = null;
      aiError = event.message;
      activeThread = {
        ...activeThread,
        messages: activeThread.messages.map((message) =>
          message.id === event.assistantMessageId && !message.content
            ? { ...message, content: `Error: ${event.message}` }
            : message
        ),
      };
      loadThreads();
    }
  }

  function titleFrom(content: string): string {
    const title = content.trim().replace(/\s+/g, " ");
    return title.length > 48 ? `${title.slice(0, 48)}...` : title || "New chat";
  }

  function shortCwd(cwd: string): string {
    const home = cwd.startsWith("/Users/") || cwd.startsWith("/home/");
    if (home) {
      const parts = cwd.split("/");
      return "~/" + parts.slice(3).join("/");
    }
    return cwd;
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}>
  <div class="panel" onclick={(e) => e.stopPropagation()}>
    <div class="search-row">
      <span class="search-icon">{mode === "chat" ? "#" : "⌕"}</span>
      <input
        class="search-input"
        bind:this={inputEl}
        bind:value={query}
        placeholder={mode === "chat" ? "Ask AI..." : "Search sessions..."}
        onkeydown={handleKeydown}
      />
    </div>

    {#if mode === "chat"}
      <div class="chat-toolbar">
        <button
          class="context-toggle"
          class:active={includeFocusedContext}
          disabled={sending}
          onclick={() => (includeFocusedContext = !includeFocusedContext)}
        >
          {includeFocusedContext ? "Focused terminal attached" : "Attach focused terminal"}
        </button>
        {#if activeRequestId}
          <span class="streaming-dot">Streaming</span>
        {/if}
      </div>

      <div class="chat-body">
        {#if activeThread}
          {#each activeThread.messages as message (message.id)}
            <div class="message" class:user={message.role === "user"} class:assistant={message.role === "assistant"}>
              <div class="message-role">{message.role === "user" ? "You" : "AI"}</div>
              <MarkdownMessage content={message.content || (message.role === "assistant" && sending ? "Thinking..." : "")} />
            </div>
          {/each}
        {:else}
          <div class="chat-empty">Type a message and press Enter.</div>
        {/if}
        {#if aiError}
          <div class="ai-error">{aiError}</div>
        {/if}
      </div>
    {:else}
      <div class="results">
        {#each paletteItems as item, i}
          {#if item.type === "ask-ai"}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="result-item action-item"
              class:selected={i === selectedIdx}
              class:disabled={item.disabled}
              onclick={() => activatePaletteItem(item)}
            >
              <span class="action-mark">#</span>
              <span class="session-name">Ask AI</span>
              <span class="cwd">{item.disabled ? item.reason : "Start an AI instruction"}</span>
            </div>
          {:else if item.type === "new-session"}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="result-item action-item"
              class:selected={i === selectedIdx}
              onclick={() => activatePaletteItem(item)}
            >
              <span class="action-mark">+</span>
              <span class="session-name">New Session</span>
              <span class="cwd">Create a terminal session</span>
            </div>
          {:else if item.type === "session"}
            {@const session = item.session}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="result-item"
              class:selected={i === selectedIdx}
              onclick={() => onSelect(session.id, session.thermalState as "Hot" | "Warm")}
            >
              <span class="color-dot" style="background: {colorMap[session.color] ?? '#666'};"></span>
              <span class="session-name">{session.name}</span>
              <span class="thermal-badge" class:warm={session.thermalState === 'Warm'}>{thermalLabel[session.thermalState] ?? session.thermalState}</span>
              <span class="cwd">{shortCwd(session.cwd)}</span>
              <button
                class="row-action danger-action"
                type="button"
                title="Kill session"
                aria-label="Kill session {session.name}"
                onclick={(e) => requestKillSession(e, session)}
              >X</button>
            </div>
          {/if}
        {/each}

        {#if filteredThreads.length > 0}
          <div class="section-label">Chat Threads</div>
          {#each filteredThreads as thread}
            {@const itemIndex = paletteItems.findIndex((item) => item.type === "thread" && item.thread.id === thread.id)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="result-item thread-item"
              class:selected={itemIndex === selectedIdx}
              onclick={() => openThread(thread.id)}
            >
              <span class="action-mark">#</span>
              <span class="thread-copy">
                <span class="thread-title">{thread.title}</span>
                <span class="thread-preview">{thread.lastMessagePreview}</span>
              </span>
              <button
                class="row-action danger-action"
                type="button"
                title="Delete chat thread"
                aria-label="Delete chat thread {thread.title}"
                onclick={(e) => deleteThread(e, thread)}
              >&#128465;</button>
            </div>
          {/each}
        {/if}

        {#if filteredSessions.length === 0 && filteredThreads.length === 0}
          <div class="empty">No sessions or chats match</div>
        {/if}
        {#if aiError}
          <div class="ai-error">{aiError}</div>
        {/if}
      </div>
    {/if}

    <div class="footer">
      <span>↑↓ navigate</span>
      <span>↵ select/send</span>
      <span>esc {mode === "chat" ? "search" : "close"}</span>
    </div>

    {#if pendingKillSession}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div class="confirm-backdrop" onclick={() => (pendingKillSession = null)}>
        <div
          class="confirm-dialog"
          role="dialog"
          aria-modal="true"
          aria-labelledby="kill-session-title"
          tabindex="-1"
          onclick={(e) => e.stopPropagation()}
          onkeydown={handleKillDialogKeydown}
        >
          <h2 id="kill-session-title">Kill Session?</h2>
          <p>
            This will force stop <strong>{pendingKillSession.name}</strong>.
          </p>
          <div class="confirm-actions">
            <button
              class="confirm-button secondary"
              type="button"
              onclick={() => (pendingKillSession = null)}
            >Cancel</button>
            <button
              class="confirm-button danger"
              type="button"
              bind:this={killConfirmButton}
              onclick={confirmKillSession}
            >Kill</button>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 15vh;
    z-index: 200;
  }

  .panel {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 8px;
    width: 560px;
    height: 520px;
    max-width: 92vw;
    max-height: 75vh;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
    overflow: hidden;
    font-family: system-ui, -apple-system, sans-serif;
    display: flex;
    flex-direction: column;
    position: relative;
  }

  .search-row {
    display: flex;
    align-items: center;
    padding: 0.6rem 0.75rem;
    border-bottom: 1px solid #2a2a2a;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .search-icon {
    color: #666;
    font-size: 1rem;
    width: 1rem;
    text-align: center;
  }

  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: #d9d4c7;
    font-size: 0.9rem;
    font-family: inherit;
  }

  .search-input::placeholder {
    color: #555;
  }

  .results,
  .chat-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .results {
    padding: 0.25rem 0;
  }

  .result-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.45rem 0.75rem;
    cursor: pointer;
    font-size: 0.82rem;
  }

  .result-item:hover,
  .result-item.selected {
    background: #2a2a2a;
  }

  .result-item.disabled {
    cursor: default;
    opacity: 0.55;
  }

  .result-item.disabled:hover {
    background: transparent;
  }

  .action-mark {
    width: 18px;
    height: 18px;
    border: 1px solid #3b82f680;
    border-radius: 4px;
    color: #93c5fd;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    font-size: 0.75rem;
  }

  .color-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .session-name {
    color: #d9d4c7;
    flex-shrink: 0;
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .thermal-badge {
    font-size: 0.65rem;
    padding: 0.1rem 0.35rem;
    border-radius: 3px;
    background: #3b82f620;
    color: #3b82f6;
    flex-shrink: 0;
  }

  .thermal-badge.warm {
    background: #f9731620;
    color: #f97316;
  }

  .cwd {
    color: #666;
    font-size: 0.75rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }

  .row-action {
    width: 42px;
    height: 24px;
    border: 1px solid transparent;
    border-radius: 4px;
    background: transparent;
    color: #777;
    cursor: pointer;
    flex-shrink: 0;
    font-family: inherit;
    font-size: 0.68rem;
    opacity: 0;
    padding: 0;
    transition: opacity 0.1s, color 0.1s, border-color 0.1s, background 0.1s;
  }

  .result-item:hover .row-action,
  .result-item.selected .row-action,
  .row-action:focus-visible {
    opacity: 1;
  }

  .danger-action:hover,
  .danger-action:focus-visible {
    background: #ef444418;
    border-color: #ef444460;
    color: #fca5a5;
    outline: none;
  }

  .section-label {
    color: #555;
    font-size: 0.68rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    padding: 0.55rem 0.75rem 0.25rem;
    text-transform: uppercase;
  }

  .thread-item {
    align-items: flex-start;
  }

  .thread-copy {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    min-width: 0;
    flex: 1;
  }

  .thread-title {
    color: #d9d4c7;
    font-size: 0.8rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .thread-preview {
    color: #666;
    font-size: 0.72rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chat-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid #272727;
    padding: 0.45rem 0.75rem;
    flex-shrink: 0;
  }

  .context-toggle {
    background: #111;
    border: 1px solid #333;
    border-radius: 5px;
    color: #999;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.72rem;
    padding: 0.25rem 0.5rem;
  }

  .context-toggle.active {
    border-color: #3b82f6;
    color: #d9d4c7;
    background: #3b82f620;
  }

  .context-toggle:disabled {
    cursor: default;
    opacity: 0.55;
  }

  .streaming-dot {
    color: #60a5fa;
    font-size: 0.72rem;
  }

  .chat-body {
    padding: 0.7rem 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .message {
    border-left: 2px solid #333;
    padding-left: 0.65rem;
  }

  .message.user {
    border-left-color: #eab308;
  }

  .message.assistant {
    border-left-color: #3b82f6;
  }

  .message-role {
    color: #777;
    font-size: 0.66rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    margin-bottom: 0.25rem;
    text-transform: uppercase;
  }

  .chat-empty,
  .empty {
    padding: 1.5rem;
    text-align: center;
    color: #555;
    font-size: 0.85rem;
  }

  .ai-error {
    background: #ef444418;
    border: 1px solid #ef444440;
    border-radius: 5px;
    color: #fca5a5;
    font-size: 0.74rem;
    line-height: 1.35;
    margin: 0.4rem 0.75rem;
    padding: 0.45rem 0.55rem;
  }

  .chat-body .ai-error {
    margin: 0;
  }

  .footer {
    display: flex;
    gap: 1rem;
    padding: 0.4rem 0.75rem;
    border-top: 1px solid #2a2a2a;
    color: #444;
    font-size: 0.7rem;
    flex-shrink: 0;
  }

  .confirm-backdrop {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.58);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    z-index: 5;
  }

  .confirm-dialog {
    width: 320px;
    max-width: 100%;
    background: #191919;
    border: 1px solid #3a3a3a;
    border-radius: 8px;
    box-shadow: 0 12px 36px rgba(0, 0, 0, 0.55);
    color: #d9d4c7;
    padding: 1rem;
  }

  .confirm-dialog h2 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 0.5rem;
  }

  .confirm-dialog p {
    color: #888;
    font-size: 0.8rem;
    line-height: 1.45;
    margin: 0 0 1rem;
  }

  .confirm-dialog strong {
    color: #d9d4c7;
    font-weight: 600;
  }

  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }

  .confirm-button {
    border: 1px solid #3a3a3a;
    border-radius: 5px;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.78rem;
    padding: 0.38rem 0.7rem;
  }

  .confirm-button:focus-visible {
    outline: 2px solid #60a5fa;
    outline-offset: 2px;
  }

  .confirm-button.secondary {
    background: #111;
    color: #aaa;
  }

  .confirm-button.danger {
    background: #ef444420;
    border-color: #ef444470;
    color: #fca5a5;
  }
</style>
