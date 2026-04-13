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
  }

  let { sessionId, color, isFocused, width, onReady, onclick }: Props = $props();

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
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="deck-pane"
  style="width: {width}px; border-color: {borderColor}; opacity: {opacity};"
  onclick={onclick}
>
  <TerminalPane {sessionId} {onReady} />
</div>

<style>
  .deck-pane {
    height: 100%;
    border-top: 2px solid;
    box-sizing: border-box;
    overflow: hidden;
    flex-shrink: 0;
    transition: width 150ms ease-out, opacity 150ms ease-out;
  }
</style>
