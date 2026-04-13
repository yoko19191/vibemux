<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import { WebglAddon } from "@xterm/addon-webgl";
  import { WebLinksAddon } from "@xterm/addon-web-links";
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
  let rendererType: 'webgl' | 'canvas' = $state('webgl');

  // Context menu state
  let contextMenu: { x: number; y: number } | null = $state(null);
  let hasSelection = $state(false);

  function closeContextMenu() {
    contextMenu = null;
  }

  function handleCopy() {
    const sel = terminal?.getSelection();
    if (sel) navigator.clipboard.writeText(sel).catch(console.error);
    closeContextMenu();
  }

  async function handlePaste() {
    try {
      const text = await navigator.clipboard.readText();
      if (text) invoke("session_write", { sessionId, data: text }).catch(console.error);
    } catch (e) {
      console.error("Paste failed:", e);
    }
    closeContextMenu();
  }

  function handleClearScreen() {
    invoke("session_write", { sessionId, data: "\x0c" }).catch(console.error);
    closeContextMenu();
  }

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

    // Web links addon — opens URLs in system browser
    const webLinksAddon = new WebLinksAddon((e, uri) => {
      e.preventDefault();
      invoke("open_url", { url: uri }).catch(console.error);
    });
    terminal.loadAddon(webLinksAddon);

    terminal.open(containerEl);

    // Try WebGL renderer, fall back to canvas
    try {
      const webglAddon = new WebglAddon();
      webglAddon.onContextLoss(() => {
        webglAddon.dispose();
        rendererType = 'canvas';
      });
      terminal.loadAddon(webglAddon);
      rendererType = 'webgl';
    } catch {
      // Canvas fallback is automatic
      rendererType = 'canvas';
    }

    fitAddon.fit();

    // Send keystrokes to backend
    terminal.onData((data: string) => {
      invoke("session_write", { sessionId, data }).catch(console.error);
    });

    // Track selection state
    terminal.onSelectionChange(() => {
      hasSelection = (terminal?.getSelection().length ?? 0) > 0;
    });

    // OSC title sequences (OSC 0 and OSC 2 set terminal title)
    terminal.parser.registerOscHandler(0, (data: string) => {
      invoke("session_set_title", { sessionId, title: data }).catch(console.error);
      return true;
    });
    terminal.parser.registerOscHandler(2, (data: string) => {
      invoke("session_set_title", { sessionId, title: data }).catch(console.error);
      return true;
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
  });

  function handleKeydown(e: KeyboardEvent) {
    const mod = navigator.platform.toUpperCase().includes("MAC") ? e.metaKey : e.ctrlKey;
    if (mod && e.key === "c" && hasSelection) {
      e.preventDefault();
      handleCopy();
    } else if (mod && e.key === "v") {
      e.preventDefault();
      handlePaste();
    }
  }

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY };
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="terminal-wrapper"
  bind:this={containerEl}
  onkeydown={handleKeydown}
  oncontextmenu={handleContextMenu}
></div>

{#if contextMenu}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="ctx-overlay" onclick={closeContextMenu}>
    <div
      class="ctx-menu"
      style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
      onclick={(e) => e.stopPropagation()}
    >
      <button class="ctx-item" disabled={!hasSelection} onclick={handleCopy}>Copy</button>
      <button class="ctx-item" onclick={handlePaste}>Paste</button>
      <div class="ctx-divider"></div>
      <button class="ctx-item" onclick={handleClearScreen}>Clear Screen</button>
    </div>
  </div>
{/if}

<style>
  .terminal-wrapper {
    width: 100%;
    height: 100%;
  }

  .ctx-overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
  }

  .ctx-menu {
    position: fixed;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 6px;
    padding: 0.25rem 0;
    min-width: 140px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
    font-family: system-ui, -apple-system, sans-serif;
  }

  .ctx-item {
    display: block;
    width: 100%;
    background: none;
    border: none;
    color: #d9d4c7;
    font-size: 0.8rem;
    padding: 0.35rem 0.75rem;
    text-align: left;
    cursor: pointer;
    font-family: inherit;
  }

  .ctx-item:hover:not(:disabled) {
    background: #3b82f620;
  }

  .ctx-item:disabled {
    color: #555;
    cursor: default;
  }

  .ctx-divider {
    height: 1px;
    background: #2a2a2a;
    margin: 0.2rem 0;
  }
</style>
