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
    scrollback?: number;
    theme?: Record<string, string>;
  }

  interface Props {
    sessionId: string;
    accentColor?: string;
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

  let { sessionId, accentColor, terminalConfig, prefixKeyMatcher, onReady, onRendererType }: Props = $props();

  let containerEl: HTMLDivElement;
  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let serializeAddon: SerializeAddon | null = null;
  let resizeObserver: ResizeObserver | null = null;
  let textareaPasteController: AbortController | null = null;
  let rendererType: 'webgl' | 'canvas' = $state('webgl');

  // Context menu state
  let contextMenu: { x: number; y: number } | null = $state(null);
  let hasSelection = $state(false);
  let pasteConfirmation: { text: string; lineCount: number } | null = $state(null);

  function closeContextMenu() {
    contextMenu = null;
  }

  function handleCopy() {
    const sel = terminal?.getSelection();
    if (sel) navigator.clipboard.writeText(sel).catch(console.error);
    closeContextMenu();
  }

  async function handlePaste() {
    await pasteFromClipboard();
    closeContextMenu();
  }

  async function pasteFromClipboard() {
    try {
      const text = await navigator.clipboard.readText();
      pasteText(text);
    } catch (e) {
      console.error("Paste failed:", e);
    }
  }

  function pasteText(rawText: string, options: { allowUnsafeMultiline?: boolean } = {}) {
    if (!terminal) return;
    const text = stripFinalLineBreak(rawText);
    if (!text) return;

    const isMultiline = hasLineBreak(text);
    if (isMultiline && !terminal.modes.bracketedPasteMode && !options.allowUnsafeMultiline) {
      pasteConfirmation = { text, lineCount: countLines(text) };
      return;
    }

    terminal.paste(text);
    terminal.focus();
  }

  function stripFinalLineBreak(text: string): string {
    if (text.endsWith("\r\n")) return text.slice(0, -2);
    if (text.endsWith("\n") || text.endsWith("\r")) return text.slice(0, -1);
    return text;
  }

  function hasLineBreak(text: string): boolean {
    return /\r\n|\r|\n/.test(text);
  }

  function countLines(text: string): number {
    return text.split(/\r\n|\r|\n/).length;
  }

  function handleNativePaste(e: ClipboardEvent) {
    e.preventDefault();
    e.stopImmediatePropagation();
    const text = e.clipboardData?.getData("text/plain") ?? "";
    pasteText(text);
  }

  function confirmPaste() {
    if (!pasteConfirmation) return;
    pasteText(pasteConfirmation.text, { allowUnsafeMultiline: true });
    pasteConfirmation = null;
  }

  function cancelPaste() {
    pasteConfirmation = null;
    terminal?.focus();
  }

  function handlePasteConfirmKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      cancelPaste();
    }
  }

  function handleClearScreen() {
    invoke("session_write", { sessionId, data: "\x0c" }).catch(console.error);
    closeContextMenu();
  }

  onMount(() => {
    const baseTheme = terminalConfig?.theme ?? {
      background: "#111111",
      foreground: "#d9d4c7",
      cursor: "#ff6b57",
    };
    const theme = accentColor
      ? { ...baseTheme, cursor: accentColor, selectionBackground: accentColor + "40" }
      : baseTheme;

    terminal = new Terminal({
      scrollback: terminalConfig?.scrollback ?? 10_000,
      theme,
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

    textareaPasteController = new AbortController();
    terminal.textarea?.addEventListener("paste", handleNativePaste, {
      capture: true,
      signal: textareaPasteController.signal,
    });

    // Let global shortcuts and clipboard operations bypass xterm's default key handling.
    terminal.attachCustomKeyEventHandler((e: KeyboardEvent) => {
      if (prefixKeyMatcher && matchesPrefixKey(e, prefixKeyMatcher)) {
        return false; // don't process — let it propagate to window
      }
      const mod = navigator.platform.toUpperCase().includes("MAC") ? e.metaKey : e.ctrlKey;
      const key = e.key.toLowerCase();
      if (mod && key === "c" && hasSelection) {
        e.preventDefault();
        handleCopy();
        return false;
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
    if (!terminal) return;
    if (terminalConfig?.fontFamily) terminal.options.fontFamily = terminalConfig.fontFamily;
    if (terminalConfig?.fontSize) terminal.options.fontSize = terminalConfig.fontSize;
    if (terminalConfig?.lineHeight) terminal.options.lineHeight = terminalConfig.lineHeight;
    if (terminalConfig?.scrollback) terminal.options.scrollback = terminalConfig.scrollback;
    const currentTheme = terminal.options.theme ?? {};
    const themeUpdate = terminalConfig?.theme ? { ...currentTheme, ...terminalConfig.theme } : { ...currentTheme };
    if (accentColor) {
      themeUpdate.cursor = accentColor;
      themeUpdate.selectionBackground = accentColor + "40";
    }
    terminal.options.theme = themeUpdate;
    fitAddon?.fit();
  });

  onDestroy(() => {
    textareaPasteController?.abort();
    resizeObserver?.disconnect();
    terminal?.dispose();
  });

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
  oncontextmenu={handleContextMenu}
></div>

{#if contextMenu}
  <ContextMenu
    position={contextMenu}
    items={getContextMenuItems()}
    onClose={closeContextMenu}
  />
{/if}

{#if pasteConfirmation}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="paste-confirm-backdrop" onclick={cancelPaste}>
    <div
      class="paste-confirm-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="paste-confirm-title"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={handlePasteConfirmKeydown}
    >
      <h2 id="paste-confirm-title">Paste multiple lines?</h2>
      <p>
        This paste contains {pasteConfirmation.lineCount} lines. Bracketed paste is not active, so
        the shell may execute commands as they are pasted.
      </p>
      <div class="paste-preview">{pasteConfirmation.text}</div>
      <div class="paste-confirm-actions">
        <button class="confirm-button secondary" type="button" onclick={cancelPaste}>Cancel</button>
        <button class="confirm-button primary" type="button" onclick={confirmPaste}>Continue Paste</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .terminal-wrapper {
    width: 100%;
    height: 100%;
  }

  .paste-confirm-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 300;
  }

  .paste-confirm-dialog {
    width: min(520px, calc(100vw - 2rem));
    background: #191919;
    border: 1px solid #3a3a3a;
    border-radius: 6px;
    color: #d9d4c7;
    font-family: system-ui, -apple-system, sans-serif;
    padding: 1rem;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.55);
  }

  .paste-confirm-dialog h2 {
    margin: 0 0 0.5rem;
    font-size: 1rem;
    font-weight: 650;
  }

  .paste-confirm-dialog p {
    margin: 0 0 0.85rem;
    color: #aaa;
    font-size: 0.85rem;
    line-height: 1.4;
  }

  .paste-preview {
    max-height: 180px;
    overflow: auto;
    white-space: pre-wrap;
    word-break: break-word;
    background: #101010;
    border: 1px solid #303030;
    border-radius: 6px;
    padding: 0.75rem;
    color: #d9d4c7;
    font-family: Menlo, Monaco, "Courier New", monospace;
    font-size: 0.78rem;
    line-height: 1.45;
  }

  .paste-confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }

  .confirm-button {
    border: 1px solid #3a3a3a;
    border-radius: 6px;
    padding: 0.45rem 0.75rem;
    font: inherit;
    font-size: 0.82rem;
    cursor: pointer;
  }

  .confirm-button.secondary {
    background: #222;
    color: #d9d4c7;
  }

  .confirm-button.primary {
    background: #3b82f6;
    border-color: #3b82f6;
    color: #fff;
  }

  .confirm-button:hover {
    filter: brightness(1.08);
  }
</style>
