<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import DeckPane from "./DeckPane.svelte";
  import { calculateDeckLayout } from "./deckLayout";
  import type { SessionSnapshot, ColorToken, ProcessState } from "./types";
  import type { PrefixKeyMatcher } from "./keymap";

  interface TerminalConfig {
    fontFamily?: string;
    fontSize?: number;
    lineHeight?: number;
    scrollback?: number;
    theme?: Record<string, string>;
  }

  interface LayoutConfig {
    focusedPaneWidth: number;
    animationMs: number;
  }

  interface Props {
    sessions: SessionSnapshot[];
    focusedSessionId: string | null;
    renamingSessionId?: string | null;
    terminalConfig?: TerminalConfig;
    layoutConfig?: LayoutConfig;
    prefixKeyMatcher?: PrefixKeyMatcher;
    onTerminalReady?: (sessionId: string, api: { writeOutput: (data: string) => void; triggerResize: () => void; serialize: () => string; focus: () => void; blur: () => void }) => void;
    onFocusSession?: (sessionId: string) => void;
    onRenameConfirm?: (sessionId: string, name: string) => void;
    onRenameCancel?: () => void;
    onStartRename?: (sessionId: string) => void;
    onPark?: (sessionId: string) => void;
    onClose?: (sessionId: string) => void;
    onKill?: (sessionId: string) => void;
  }

  let {
    sessions, focusedSessionId,
    renamingSessionId = null, terminalConfig, layoutConfig = { focusedPaneWidth: 0.6, animationMs: 150 }, prefixKeyMatcher,
    onTerminalReady, onFocusSession,
    onRenameConfirm, onRenameCancel, onStartRename,
    onPark, onClose, onKill,
  }: Props = $props();

  let containerEl: HTMLDivElement;
  let containerWidth = $state(0);
  let resizeObserver: ResizeObserver | null = null;
  let draggedSessionId: string | null = $state(null);
  let dragInsertIdx: number | null = $state(null);
  let dragInsertSide: 'left' | 'right' | null = $state(null);

  let layouts = $derived(
    calculateDeckLayout(
      containerWidth,
      sessions.map((s) => s.id),
      focusedSessionId,
      { focusedPaneWidth: layoutConfig.focusedPaneWidth },
    ),
  );

  function getColor(sessionId: string): ColorToken {
    return sessions.find((s) => s.id === sessionId)?.color ?? "Blue";
  }

  function getName(sessionId: string): string {
    const s = sessions.find((s) => s.id === sessionId);
    return s?.customName ?? s?.name ?? "";
  }

  function getCwd(sessionId: string): string {
    return sessions.find((s) => s.id === sessionId)?.cwd ?? "";
  }

  function getTerminalTitle(sessionId: string): string {
    return sessions.find((s) => s.id === sessionId)?.terminalTitle ?? "";
  }

  function getProcessState(sessionId: string): ProcessState {
    return sessions.find((s) => s.id === sessionId)?.processState ?? { type: "Exited", code: null };
  }

  function handleDragStart(sessionId: string, e: DragEvent) {
    draggedSessionId = sessionId;
    e.dataTransfer?.setData("text/plain", sessionId);
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
    }
  }

  function handleDragOverPane(targetSessionId: string, e: DragEvent) {
    e.preventDefault();
    if (!draggedSessionId || draggedSessionId === targetSessionId) {
      dragInsertIdx = null;
      dragInsertSide = null;
      return;
    }
    const targetIdx = sessions.findIndex((s) => s.id === targetSessionId);
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const midX = rect.left + rect.width / 2;
    dragInsertIdx = targetIdx;
    dragInsertSide = e.clientX < midX ? 'left' : 'right';
  }

  function handleDragEnd() {
    draggedSessionId = null;
    dragInsertIdx = null;
    dragInsertSide = null;
  }

  async function handleDrop(targetSessionId: string, e: DragEvent) {
    e.preventDefault();
    if (!draggedSessionId || draggedSessionId === targetSessionId) {
      handleDragEnd();
      return;
    }

    const ids = sessions.map((s) => s.id);
    const fromIdx = ids.indexOf(draggedSessionId);
    const toIdx = ids.indexOf(targetSessionId);

    if (fromIdx === -1 || toIdx === -1) {
      handleDragEnd();
      return;
    }

    const insertAt = dragInsertSide === 'right' ? toIdx + 1 : toIdx;
    const reordered = [...ids];
    reordered.splice(fromIdx, 1);
    const adjustedInsert = insertAt > fromIdx ? insertAt - 1 : insertAt;
    reordered.splice(adjustedInsert, 0, draggedSessionId);

    try {
      await invoke("session_reorder", { sessionIds: reordered });
    } catch (err) {
      console.error("Failed to reorder sessions:", err);
    }

    handleDragEnd();
  }

  onMount(() => {
    // Seed containerWidth immediately so the first layout isn't zero-width
    containerWidth = containerEl.getBoundingClientRect().width;

    resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        containerWidth = entry.contentRect.width;
      }
    });
    resizeObserver.observe(containerEl);
  });

  onDestroy(() => {
    resizeObserver?.disconnect();
  });
</script>

<div class="deck" bind:this={containerEl}>
  {#each layouts as layout (layout.sessionId)}
    <DeckPane
      sessionId={layout.sessionId}
      sessionName={getName(layout.sessionId)}
      sessionCwd={getCwd(layout.sessionId)}
      terminalTitle={getTerminalTitle(layout.sessionId)}
      color={getColor(layout.sessionId)}
      processState={getProcessState(layout.sessionId)}
      isFocused={layout.isFocused}
      width={layout.width}
      left={layout.left}
      zIndex={layout.zIndex}
      isRenaming={renamingSessionId === layout.sessionId}
      {terminalConfig}
      animationMs={layoutConfig.animationMs}
      {prefixKeyMatcher}
      onReady={(api) => onTerminalReady?.(layout.sessionId, api)}
      onclick={() => onFocusSession?.(layout.sessionId)}
      ondragstart={(e) => handleDragStart(layout.sessionId, e)}
      ondragover={(e) => handleDragOverPane(layout.sessionId, e)}
      ondrop={(e) => handleDrop(layout.sessionId, e)}
      ondragend={handleDragEnd}
      onRenameConfirm={(name) => onRenameConfirm?.(layout.sessionId, name)}
      onRenameCancel={onRenameCancel}
      onStartRename={() => onStartRename?.(layout.sessionId)}
      onPark={() => onPark?.(layout.sessionId)}
      onClose={() => onClose?.(layout.sessionId)}
      onKill={() => onKill?.(layout.sessionId)}
    />
  {/each}
  {#if dragInsertIdx !== null && dragInsertSide !== null}
    {@const targetLayout = layouts.find((l) => l.sessionId === sessions[dragInsertIdx]?.id)}
    {#if targetLayout}
      <div
        class="drag-insert-line"
        style="left: {dragInsertSide === 'left' ? targetLayout.left : targetLayout.left + targetLayout.width}px;"
      ></div>
    {/if}
  {/if}
</div>

<style>
  .deck {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .drag-insert-line {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 2px;
    background: #3b82f6;
    box-shadow: 0 0 6px 1px rgba(59, 130, 246, 0.5);
    z-index: 100;
    pointer-events: none;
    transition: left 100ms ease-out;
  }
</style>
