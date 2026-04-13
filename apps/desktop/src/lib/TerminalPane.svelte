<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import { WebglAddon } from "@xterm/addon-webgl";
  import { invoke } from "@tauri-apps/api/core";

  interface Props {
    sessionId: string;
    onReady?: (api: { writeOutput: (data: string) => void }) => void;
  }

  let { sessionId, onReady }: Props = $props();

  let containerEl: HTMLDivElement;
  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let resizeObserver: ResizeObserver | null = null;

  onMount(() => {
    terminal = new Terminal({
      scrollback: 10_000,
      theme: {
        background: "#111111",
        foreground: "#d9d4c7",
        cursor: "#ff6b57",
      },
      fontFamily: "Menlo, Monaco, 'Courier New', monospace",
      fontSize: 14,
      allowProposedApi: true,
    });

    fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);

    terminal.open(containerEl);

    // Try WebGL renderer, fall back to canvas
    try {
      const webglAddon = new WebglAddon();
      webglAddon.onContextLoss(() => {
        webglAddon.dispose();
      });
      terminal.loadAddon(webglAddon);
    } catch {
      // Canvas fallback is automatic
    }

    fitAddon.fit();

    // Send keystrokes to backend
    terminal.onData((data: string) => {
      invoke("session_write", { sessionId, data }).catch(console.error);
    });

    // Resize observer
    resizeObserver = new ResizeObserver(() => {
      if (fitAddon) {
        fitAddon.fit();
        const dims = fitAddon.proposeDimensions();
        if (dims) {
          invoke("session_resize", {
            sessionId,
            cols: dims.cols,
            rows: dims.rows,
          }).catch(console.error);
        }
      }
    });
    resizeObserver.observe(containerEl);

    // Notify parent that terminal is ready
    onReady?.({
      writeOutput: (data: string) => terminal?.write(data),
    });
  });

  onDestroy(() => {
    resizeObserver?.disconnect();
    terminal?.dispose();
    terminal = null;
    fitAddon = null;
  });
</script>

<div class="terminal-pane" bind:this={containerEl}></div>

<style>
  .terminal-pane {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }
</style>
