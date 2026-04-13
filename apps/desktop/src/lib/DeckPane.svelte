<script lang="ts">
  import TerminalPane from "./TerminalPane.svelte";
  import type { ColorToken } from "./types";

  interface Props {
    sessionId: string;
    color: ColorToken;
    isFocused: boolean;
    width: number;
    onReady?: (api: { writeOutput: (data: string) => void }) => void;
    onclick?: () => void;
    ondragstart?: (e: DragEvent) => void;
    ondragover?: (e: DragEvent) => void;
    ondrop?: (e: DragEvent) => void;
  }

  let { sessionId, color, isFocused, width, onReady, onclick, ondragstart, ondragover, ondrop }: Props = $props();

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
  <div class="terminal-container">
    <TerminalPane {sessionId} {onReady} />
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

  .drag-over {
    outline: 2px solid #3b82f6;
    outline-offset: -2px;
  }

  .terminal-container {
    flex: 1;
    overflow: hidden;
  }
</style>
