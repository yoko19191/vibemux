<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import DeckPane from "./DeckPane.svelte";
  import { calculateDeckLayout, type DeckPaneLayout } from "./deckLayout";
  import type { SessionSnapshot, ColorToken } from "./types";

  interface Props {
    sessions: SessionSnapshot[];
    focusedSessionId: string | null;
    onTerminalReady?: (sessionId: string, api: { writeOutput: (data: string) => void }) => void;
    onFocusSession?: (sessionId: string) => void;
  }

  let { sessions, focusedSessionId, onTerminalReady, onFocusSession }: Props = $props();

  let containerEl: HTMLDivElement;
  let containerWidth = $state(0);
  let resizeObserver: ResizeObserver | null = null;

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

  onMount(() => {
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
      color={getColor(layout.sessionId)}
      isFocused={layout.isFocused}
      width={layout.width}
      onReady={(api) => onTerminalReady?.(layout.sessionId, api)}
      onclick={() => onFocusSession?.(layout.sessionId)}
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
