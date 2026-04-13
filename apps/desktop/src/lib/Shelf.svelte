<script lang="ts">
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
</script>

{#if sessions.length > 0}
  <div class="shelf">
    <div class="shelf-label">SHELF</div>
    <div class="shelf-cards">
      {#each sessions as session, i (session.id)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class:selected={selectedIdx === i} onclick={() => onRecall?.(session.id)}>
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
  .shelf {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.35rem 0.75rem;
    background: rgba(17, 17, 17, 0.95);
    border-top: 1px solid #2a2a2a;
    z-index: 40;
    font-family: system-ui, -apple-system, sans-serif;
    overflow: hidden;
  }

  .shelf-label {
    font-size: 0.6rem;
    font-weight: 700;
    color: #555;
    letter-spacing: 0.08em;
    flex-shrink: 0;
  }

  .shelf-cards {
    display: flex;
    gap: 0.5rem;
    overflow-x: auto;
    flex: 1;
    scrollbar-width: none;
  }

  .shelf-cards::-webkit-scrollbar {
    display: none;
  }

  .selected :global(.shelf-card) {
    outline: 2px solid #3b82f6;
    outline-offset: -2px;
  }
</style>
