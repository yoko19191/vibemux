<script lang="ts">
  import TerminalPane from "./TerminalPane.svelte";
  import type { ColorToken } from "./types";

  interface Props {
    sessionId: string;
    sessionName: string;
    terminalTitle?: string;
    color: ColorToken;
    isFocused: boolean;
    width: number;
    isRenaming?: boolean;
    onReady?: (api: { writeOutput: (data: string) => void }) => void;
    onclick?: () => void;
    ondragstart?: (e: DragEvent) => void;
    ondragover?: (e: DragEvent) => void;
    ondrop?: (e: DragEvent) => void;
    onRenameConfirm?: (name: string) => void;
    onRenameCancel?: () => void;
  }

  let {
    sessionId, sessionName, terminalTitle = "", color, isFocused, width,
    isRenaming = false,
    onReady, onclick, ondragstart, ondragover, ondrop,
    onRenameConfirm, onRenameCancel,
  }: Props = $props();

  let displayName = $derived(terminalTitle || sessionName);
  let rendererType: 'webgl' | 'canvas' = $state('webgl');

  const colorMap: Record<ColorToken, string> = {
    Red: "#ef4444",
    Orange: "#f97316",
    Yellow: "#eab308",
    Green: "#22c55e",
    Cyan: "#06b6d4",
    Blue: "#3b82f6",
    Purple: "#a855f7",
    Pink: "#ec4899",
  };

  let borderColor = $derived(colorMap[color] ?? "#666");
  let opacity = $derived(isFocused ? 1 : 0.8);
  let dragOver = $state(false);
  let renameValue = $state("");
  let renameInput: HTMLInputElement | null = $state(null);

  $effect(() => {
    if (isRenaming) {
      renameValue = sessionName;
      // Focus input on next tick
      setTimeout(() => renameInput?.focus(), 0);
    }
  });

  function handleRenameKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      e.stopPropagation();
      onRenameConfirm?.(renameValue.trim() || sessionName);
    } else if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      onRenameCancel?.();
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="deck-pane"
  class:drag-over={dragOver}
  style="width: {width}px; border-color: {borderColor}; opacity: {opacity};"
  onclick={onclick}
  ondragover={(e) => { e.preventDefault(); dragOver = true; ondragover?.(e); }}
  ondragleave={() => { dragOver = false; }}
  ondrop={(e) => { dragOver = false; ondrop?.(e); }}
>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="drag-handle"
    draggable="true"
    ondragstart={ondragstart}
    style="background: {borderColor};"
  ></div>
  <div class="pane-header" style="border-bottom-color: {borderColor}20;">
    {#if isRenaming}
      <!-- svelte-ignore a11y_autofocus -->
      <input
        class="rename-input"
        bind:this={renameInput}
        bind:value={renameValue}
        onkeydown={handleRenameKeydown}
        onclick={(e) => e.stopPropagation()}
      />
    {:else}
      <span class="session-name">{displayName}</span>
      {#if rendererType === 'canvas'}
        <span class="renderer-badge">canvas</span>
      {/if}
    {/if}
  </div>
  <div class="terminal-container">
    <TerminalPane {sessionId} {onReady} onRendererType={(t) => (rendererType = t)} />
  </div>
</div>

<style>
  .deck-pane {
    height: 100%;
    box-sizing: border-box;
    overflow: hidden;
    flex-shrink: 0;
    transition: width 150ms ease-out, opacity 150ms ease-out;
    display: flex;
    flex-direction: column;
  }

  .drag-handle {
    height: 3px;
    cursor: grab;
    flex-shrink: 0;
    opacity: 0.8;
  }

  .drag-handle:active {
    cursor: grabbing;
  }

  .pane-header {
    height: 22px;
    display: flex;
    align-items: center;
    padding: 0 0.5rem;
    border-bottom: 1px solid;
    flex-shrink: 0;
    overflow: hidden;
  }

  .session-name {
    font-size: 0.7rem;
    color: #888;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-family: system-ui, -apple-system, sans-serif;
    flex: 1;
    min-width: 0;
  }

  .renderer-badge {
    font-size: 0.6rem;
    padding: 0.05rem 0.3rem;
    border-radius: 3px;
    background: #55555520;
    color: #666;
    flex-shrink: 0;
    margin-left: 0.25rem;
    font-family: system-ui, -apple-system, sans-serif;
  }

  .rename-input {
    width: 100%;
    background: transparent;
    border: none;
    border-bottom: 1px solid #3b82f6;
    color: #d9d4c7;
    font-size: 0.7rem;
    font-family: system-ui, -apple-system, sans-serif;
    outline: none;
    padding: 0;
  }

  .drag-over {
    outline: 2px solid #3b82f6;
    outline-offset: -2px;
  }

  .terminal-container {
    flex: 1;
    overflow: hidden;
  }
</style>
