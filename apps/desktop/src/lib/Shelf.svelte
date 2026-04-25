<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import ShelfCard from "./ShelfCard.svelte";
  import type { SessionSnapshot } from "./types";

  interface Props {
    sessions: SessionSnapshot[];
    onRecall?: (sessionId: string) => void;
    selectedIdx?: number | null;
    onRename?: (sessionId: string, name: string) => void;
    onSetColor?: (sessionId: string, color: string) => void;
    onClose?: (sessionId: string) => void;
    onKill?: (sessionId: string) => void;
  }

  let { sessions, onRecall, selectedIdx = null, onRename, onSetColor, onClose, onKill }: Props = $props();

  let draggedSessionId: string | null = $state(null);
  let dragInsertIdx: number | null = $state(null);

  function handleDragStart(sessionId: string, e: DragEvent) {
    draggedSessionId = sessionId;
    e.dataTransfer?.setData("text/plain", sessionId);
    if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
  }

  function handleDragOver(targetIdx: number, e: DragEvent) {
    e.preventDefault();
    if (!draggedSessionId) return;
    const targetId = sessions[targetIdx]?.id;
    if (targetId === draggedSessionId) { dragInsertIdx = null; return; }
    dragInsertIdx = targetIdx;
  }

  function cleanupDrag() {
    draggedSessionId = null;
    dragInsertIdx = null;
  }

  async function handleDrop(targetIdx: number, e: DragEvent) {
    e.preventDefault();
    if (!draggedSessionId) { cleanupDrag(); return; }

    const ids = sessions.map((s) => s.id);
    const fromIdx = ids.indexOf(draggedSessionId);
    if (fromIdx === -1 || fromIdx === targetIdx) { cleanupDrag(); return; }

    const reordered = [...ids];
    reordered.splice(fromIdx, 1);
    const adjustedIdx = targetIdx > fromIdx ? targetIdx - 1 : targetIdx;
    reordered.splice(adjustedIdx, 0, draggedSessionId);

    try {
      await invoke("session_reorder_warm", { sessionIds: reordered });
    } catch (err) {
      console.error("Failed to reorder warm sessions:", err);
    }

    cleanupDrag();
  }
</script>

{#if sessions.length > 0}
  <div class="detached">
    <div class="detached-label">DETACHED</div>
    <div class="detached-cards">
      {#each sessions as session, i (session.id)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        {#if dragInsertIdx === i && draggedSessionId}
          <div class="shelf-insert-line"></div>
        {/if}
        <div
          class:selected={selectedIdx === i}
          draggable="true"
          ondragstart={(e) => handleDragStart(session.id, e)}
          ondragover={(e) => handleDragOver(i, e)}
          ondrop={(e) => handleDrop(i, e)}
          ondragend={cleanupDrag}
          onclick={() => onRecall?.(session.id)}
        >
          <ShelfCard
            {session}
            onRename={(name) => onRename?.(session.id, name)}
            onSetColor={(color) => onSetColor?.(session.id, color)}
            onClose={() => onClose?.(session.id)}
            onKill={() => onKill?.(session.id)}
          />
        </div>
      {/each}
    </div>
  </div>
{/if}

<style>
  .detached {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    height: 31px;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0 0.75rem;
    background: rgba(17, 17, 17, 0.95);
    border-top: 1px solid #2a2a2a;
    z-index: 40;
    font-family: system-ui, -apple-system, sans-serif;
    overflow: hidden;
  }

  .detached-label {
    font-size: 0.6rem;
    font-weight: 700;
    color: #555;
    letter-spacing: 0.08em;
    flex-shrink: 0;
  }

  .detached-cards {
    display: flex;
    gap: 0.5rem;
    overflow-x: auto;
    flex: 1;
    scrollbar-width: none;
  }

  .detached-cards::-webkit-scrollbar {
    display: none;
  }

  .selected :global(.detached-card) {
    outline: 2px solid #3b82f6;
    outline-offset: -2px;
  }

  .shelf-insert-line {
    width: 2px;
    height: 17px;
    background: #3b82f6;
    box-shadow: 0 0 4px rgba(59, 130, 246, 0.5);
    flex-shrink: 0;
    border-radius: 1px;
    align-self: center;
  }
</style>
