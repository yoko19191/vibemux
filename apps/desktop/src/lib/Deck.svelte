<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import DeckPane from "./DeckPane.svelte";
  import { calculateDeckLayout } from "./deckLayout";
  import type { SessionSnapshot, ColorToken } from "./types";

  interface Props {
    sessions: SessionSnapshot[];
    focusedSessionId: string | null;
    renamingSessionId?: string | null;
    onTerminalReady?: (sessionId: string, api: { writeOutput: (data: string) => void }) => void;
    onFocusSession?: (sessionId: string) => void;
    onRenameConfirm?: (sessionId: string, name: string) => void;
    onRenameCancel?: () => void;
  }

  let {
    sessions, focusedSessionId,
    renamingSessionId = null,
    onTerminalReady, onFocusSession,
    onRenameConfirm, onRenameCancel,
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
    return sessions.find((s) => s.id === sessionId)?.name ?? "";
  }

  function getTerminalTitle(sessionId: string): string {
    return sessions.find((s) => s.id === sessionId)?.terminalTitle ?? "";
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
      terminalTitle={getTerminalTitle(layout.sessionId)}
      color={getColor(layout.sessionId)}
      isFocused={layout.isFocused}
      width={layout.width}
      isRenaming={renamingSessionId === layout.sessionId}
      onReady={(api) => onTerminalReady?.(layout.sessionId, api)}
      onclick={() => onFocusSession?.(layout.sessionId)}
      ondragstart={(e) => handleDragStart(layout.sessionId, e)}
      ondragover={handleDragOver}
      ondrop={(e) => handleDrop(layout.sessionId, e)}
      onRenameConfirm={(name) => onRenameConfirm?.(layout.sessionId, name)}
      onRenameCancel={onRenameCancel}
    />
  {/each}
</div>

<style>
  .deck {
    display: flex;
    flex-direction: row;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }
</style>
