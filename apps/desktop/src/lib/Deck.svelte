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
    theme?: Record<string, string>;
  }

  interface Props {
    sessions: SessionSnapshot[];
    focusedSessionId: string | null;
    renamingSessionId?: string | null;
    terminalConfig?: TerminalConfig;
    prefixKeyMatcher?: PrefixKeyMatcher;
    onTerminalReady?: (sessionId: string, api: { writeOutput: (data: string) => void; resetAndResize: () => void; focus: () => void; blur: () => void }) => void;
    onFocusSession?: (sessionId: string) => void;
    onRenameConfirm?: (sessionId: string, name: string) => void;
    onRenameCancel?: () => void;
    onStartRename?: (sessionId: string) => void;
    onPark?: (sessionId: string) => void;
    onClose?: (sessionId: string) => void;
  }

  let {
    sessions, focusedSessionId,
    renamingSessionId = null, terminalConfig, prefixKeyMatcher,
    onTerminalReady, onFocusSession,
    onRenameConfirm, onRenameCancel, onStartRename,
    onPark, onClose,
  }: Props = $props();

  let containerEl: HTMLDivElement;
  let containerWidth = $state(0);
  let resizeObserver: ResizeObserver | null = null;
  let draggedSessionId: string | null = null;

  let layouts = $derived(
    calculateDeckLayout(
      containerWidth,
      sessions.map((s) => s.id),
      focusedSessionId,
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

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
  }

  async function handleDrop(targetSessionId: string, e: DragEvent) {
    e.preventDefault();
    if (!draggedSessionId || draggedSessionId === targetSessionId) {
      draggedSessionId = null;
      return;
    }

    const ids = sessions.map((s) => s.id);
    const fromIdx = ids.indexOf(draggedSessionId);
    const toIdx = ids.indexOf(targetSessionId);

    if (fromIdx === -1 || toIdx === -1) {
      draggedSessionId = null;
      return;
    }

    // Reorder
    const reordered = [...ids];
    reordered.splice(fromIdx, 1);
    reordered.splice(toIdx, 0, draggedSessionId);

    try {
      await invoke("session_reorder", { sessionIds: reordered });
    } catch (err) {
      console.error("Failed to reorder sessions:", err);
    }

    draggedSessionId = null;
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
      {prefixKeyMatcher}
      onReady={(api) => onTerminalReady?.(layout.sessionId, api)}
      onclick={() => onFocusSession?.(layout.sessionId)}
      ondragstart={(e) => handleDragStart(layout.sessionId, e)}
      ondragover={handleDragOver}
      ondrop={(e) => handleDrop(layout.sessionId, e)}
      onRenameConfirm={(name) => onRenameConfirm?.(layout.sessionId, name)}
      onRenameCancel={onRenameCancel}
      onStartRename={() => onStartRename?.(layout.sessionId)}
      onPark={() => onPark?.(layout.sessionId)}
      onClose={() => onClose?.(layout.sessionId)}
    />
  {/each}
</div>

<style>
  .deck {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }
</style>
