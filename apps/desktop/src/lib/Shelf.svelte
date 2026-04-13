<script lang="ts">
  import ShelfCard from "./ShelfCard.svelte";
  import type { SessionSnapshot } from "./types";

  interface Props {
    sessions: SessionSnapshot[];
    onRecall?: (sessionId: string) => void;
  }

  let { sessions, onRecall }: Props = $props();
</script>

{#if sessions.length > 0}
  <div class="shelf">
    <div class="shelf-label">SHELF</div>
    <div class="shelf-cards">
      {#each sessions as session (session.id)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div onclick={() => onRecall?.(session.id)}>
          <ShelfCard {session} />
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
</style>
