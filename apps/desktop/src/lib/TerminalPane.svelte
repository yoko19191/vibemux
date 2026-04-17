<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import "@xterm/xterm/css/xterm.css";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import { WebglAddon } from "@xterm/addon-webgl";
  import { WebLinksAddon } from "@xterm/addon-web-links";
  import { SerializeAddon } from "@xterm/addon-serialize";
  import { invoke } from "@tauri-apps/api/core";
  import { matchesPrefixKey } from "./keymap";
  import type { PrefixKeyMatcher } from "./keymap";
  import ContextMenu from "./ContextMenu.svelte";
  import type { ContextMenuItem } from "./ContextMenu.svelte";

  interface TerminalConfig {
    fontFamily?: string;
    fontSize?: number;
    lineHeight?: number;
    theme?: Record<string, string>;
  }

  interface Props {
    sessionId: string;
    terminalConfig?: TerminalConfig;
    prefixKeyMatcher?: PrefixKeyMatcher;
    onReady?: (api: {
      writeOutput: (data: string) => void;
      triggerResize: () => void;
      serialize: () => string;
      focus: () => void;
      blur: () => void;
    }) => void;
    onRendererType?: (type: 'webgl' | 'canvas') => void;
  }

  let { sessionId, terminalConfig, prefixKeyMatcher, onReady, onRendererType }: Props = $props();

  let containerEl: HTMLDivElement;
  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let serializeAddon: SerializeAddon | null = null;
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
      theme: terminalConfig?.theme ?? {
        background: "#111111",
        foreground: "#d9d4c7",
        cursor: "#ff6b57",
      },
      fontFamily: terminalConfig?.fontFamily ?? "Menlo, Monaco, 'Courier New', monospace",
      fontSize: terminalConfig?.fontSize ?? 14,
      lineHeight: terminalConfig?.lineHeight ?? 1.2,
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

    serializeAddon = new SerializeAddon();
    terminal.loadAddon(serializeAddon);

    terminal.open(containerEl);

    // Let the prefix key bypass xterm so it bubbles to the window handler
    terminal.attachCustomKeyEventHandler((e: KeyboardEvent) => {
      if (prefixKeyMatcher && matchesPrefixKey(e, prefixKeyMatcher)) {
        return false; // don't process — let it propagate to window
      }
      return true;
    });

    // Try WebGL renderer, fall back to canvas
    try {
      const webglAddon = new WebglAddon();
      webglAddon.onContextLoss(() => {
        webglAddon.dispose();
        rendererType = 'canvas';
        onRendererType?.('canvas');
      });
      terminal.loadAddon(webglAddon);
      rendererType = 'webgl';
      onRendererType?.('webgl');
    } catch {
      // Canvas fallback is automatic
      rendererType = 'canvas';
      onRendererType?.('canvas');
    }

    // Defer initial fit past the deck-pane CSS transition (150ms) so xterm
    // measures the final container size, not the animated intermediate size.
    fitAddon.fit();
    setTimeout(() => {
      fitAddon?.fit();
      const dims = fitAddon?.proposeDimensions();
      if (dims) {
        invoke("session_resize", { sessionId, cols: dims.cols, rows: dims.rows }).catch(console.error);
      }
    }, 200);

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
      triggerResize: () => {
        if (!terminal || !fitAddon) return;
        fitAddon.fit();
        const dims = fitAddon.proposeDimensions();
        if (dims) {
          invoke("session_resize", { sessionId, cols: dims.cols, rows: dims.rows }).catch(console.error);
        }
      },
      serialize: () => serializeAddon?.serialize() ?? "",
      focus: () => terminal?.focus(),
      blur: () => terminal?.blur(),
    });
  });

  // Apply config changes to running terminal
  $effect(() => {
    if (!terminal || !terminalConfig) return;
    if (terminalConfig.fontFamily) terminal.options.fontFamily = terminalConfig.fontFamily;
    if (terminalConfig.fontSize) terminal.options.fontSize = terminalConfig.fontSize;
    if (terminalConfig.lineHeight) terminal.options.lineHeight = terminalConfig.lineHeight;
    if (terminalConfig.theme) terminal.options.theme = terminalConfig.theme;
    // Re-fit after font/size changes
    fitAddon?.fit();
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

  function getContextMenuItems(): ContextMenuItem[] {
    return [
      { label: "Copy", disabled: !hasSelection, onClick: handleCopy },
      { label: "Paste", onClick: handlePaste },
      { type: "separator" },
      { label: "Clear Screen", onClick: handleClearScreen },
    ];
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
  <ContextMenu
    position={contextMenu}
    items={getContextMenuItems()}
    onClose={closeContextMenu}
  />
{/if}

<style>
  .terminal-wrapper {
    width: 100%;
    height: 100%;
  }
</style>
