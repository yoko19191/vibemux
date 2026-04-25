<script lang="ts">
  import TerminalPane from "./TerminalPane.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import BusyIndicator from "./BusyIndicator.svelte";
  import type { ContextMenuItem } from "./ContextMenu.svelte";
  import type { ColorToken, ProcessState } from "./types";
  import type { PrefixKeyMatcher } from "./keymap";
  import { colorMap, colorTokens } from "./colors";
  import { invoke } from "@tauri-apps/api/core";

  interface TerminalConfig {
    fontFamily?: string;
    fontSize?: number;
    lineHeight?: number;
    scrollback?: number;
    theme?: Record<string, string>;
  }

  interface Props {
    sessionId: string;
    sessionName: string;
    sessionCwd?: string;
    terminalTitle?: string;
    color: ColorToken;
    processState?: ProcessState;
    isFocused: boolean;
    width: number;
    left?: number;
    zIndex?: number;
    isRenaming?: boolean;
    terminalConfig?: TerminalConfig;
    animationMs?: number;
    prefixKeyMatcher?: PrefixKeyMatcher;
    onReady?: (api: { writeOutput: (data: string) => void; triggerResize: () => void; serialize: () => string; focus: () => void; blur: () => void }) => void;
    onclick?: () => void;
    ondragstart?: (e: DragEvent) => void;
    ondragover?: (e: DragEvent) => void;
    ondrop?: (e: DragEvent) => void;
    ondragend?: () => void;
    onRenameConfirm?: (name: string) => void;
    onRenameCancel?: () => void;
    onStartRename?: () => void;
    onPark?: () => void;
    onClose?: () => void;
    onKill?: () => void;
  }

  let {
    sessionId, sessionName, sessionCwd = "", terminalTitle = "", color, processState, isFocused, width,
    left = 0, zIndex = 1,
    isRenaming = false, terminalConfig, animationMs = 150, prefixKeyMatcher,
    onReady, onclick, ondragstart, ondragover, ondrop, ondragend,
    onRenameConfirm, onRenameCancel, onStartRename, onPark, onClose, onKill,
  }: Props = $props();

  let displayName = $derived(sessionName || terminalTitle);
  let dynamicSuffix = $derived(sessionName && terminalTitle && terminalTitle !== sessionName ? `(${terminalTitle})` : "");
  let shortCwd = $derived(sessionCwd.replace(/^.*\/([^/]+)$/, "$1") || sessionCwd);
  let rendererType: 'webgl' | 'canvas' = $state('webgl');

  let isBusy = $derived(processState?.type === "Running");
  let borderColor = $derived(colorMap[color] ?? "#666");
  let borderStyle = $derived(isFocused
    ? `2px solid ${borderColor}`
    : `2px solid ${borderColor}`);
  let boxShadow = $derived(isFocused
    ? `0 0 8px 2px ${borderColor}55, 0 6px 20px rgba(0,0,0,0.6)`
    : `inset 0 0 12px 0 ${borderColor}20, -1px 0 0 0 rgba(0,0,0,0.8), 1px 0 0 0 rgba(0,0,0,0.8)`);
  let filter = $derived(isFocused ? 'brightness(1)' : 'brightness(0.75)');
  let dragOver = $state(false);
  let renameValue = $state("");
  let renameInput: HTMLInputElement | null = $state(null);
  let isHovered = $state(false);
  let headerContextMenu: { x: number; y: number } | null = $state(null);

  $effect(() => {
    if (isRenaming) {
      renameValue = sessionName;
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

  function handleHeaderContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    headerContextMenu = { x: e.clientX, y: e.clientY };
  }

  function buildContextMenuItems(): ContextMenuItem[] {
    return [
      { label: "Detach", onClick: () => onPark?.() },
      { label: "Rename", onClick: startRename },
      {
        type: "submenu",
        label: "Change Color",
        children: colorTokens.map((c) => ({
          token: c.token,
          color: c.color,
          onSelect: () => invoke("session_set_color", { sessionId, color: c.token }).catch(console.error),
        })),
      },
      { type: "separator" },
      { label: "Close", onClick: () => onClose?.() },
      { label: "Kill", color: "#ef4444", onClick: () => onKill?.() },
    ];
  }

  function startRename() {
    onStartRename?.();
  }

  function getContextMenuItems(): ContextMenuItem[] {
    return [
      { label: "Detach", onClick: () => onPark?.() },
      { label: "Rename", onClick: startRename },
      {
        type: "submenu",
        label: "Change Color",
        children: colorTokens.map((c) => ({
          token: c.token,
          color: c.color,
          onSelect: () => invoke("session_set_color", { sessionId, color: c.token }).catch(console.error),
        })),
      },
      { type: "separator" },
      { label: "Close", onClick: () => onClose?.() },
      { label: "Kill", color: "#ef4444", onClick: () => onKill?.() },
    ];
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="deck-pane"
  class:drag-over={dragOver}
  class:is-busy={isBusy}
  data-session-id={sessionId}
  style="width: {width}px; left: {left}px; z-index: {zIndex}; border: {borderStyle}; box-shadow: {boxShadow}; filter: {filter}; --border-color: {borderColor}; --pane-transition-ms: {animationMs}ms;"
  onclick={onclick}
  onmouseenter={() => (isHovered = true)}
  onmouseleave={() => (isHovered = false)}
  ondragover={(e) => { e.preventDefault(); dragOver = true; ondragover?.(e); }}
  ondragleave={() => { dragOver = false; }}
  ondrop={(e) => { dragOver = false; ondrop?.(e); }}
>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="drag-handle"
    draggable="true"
    ondragstart={ondragstart}
    ondragend={ondragend}
    oncontextmenu={handleHeaderContextMenu}
    style="background: linear-gradient(to right, {borderColor}22, transparent);"
  ></div>
  <div
    class="pane-header"
    style="border-bottom-color: {borderColor}20;"
    oncontextmenu={handleHeaderContextMenu}
  >
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
      {#if isBusy}
        <BusyIndicator color={borderColor} />
      {/if}
      <span class="session-name" ondblclick={(e) => { e.stopPropagation(); startRename(); }}>{displayName}</span>{#if dynamicSuffix}<span class="session-dynamic">{dynamicSuffix}</span>{/if}
      {#if shortCwd}
        <span class="session-cwd">{shortCwd}</span>
      {/if}
      {#if rendererType === 'canvas'}
        <span class="renderer-badge">canvas</span>
      {/if}
      {#if isFocused && isHovered}
        <div class="header-actions" onclick={(e) => e.stopPropagation()}>
          <button class="hdr-btn" title="Detach (Ctrl+B, B)" onclick={() => onPark?.()}>⬇</button>
          <button class="hdr-btn" title="Rename (Ctrl+B, R)" onclick={startRename}>✎</button>
          <button class="hdr-btn hdr-btn-close" title="Close (Ctrl+B, X)" onclick={() => onClose?.()}>✕</button>
        </div>
      {/if}
    {/if}
  </div>
  <div class="terminal-container">
    <TerminalPane {sessionId} accentColor={borderColor} {terminalConfig} {prefixKeyMatcher} {onReady} onRendererType={(t) => (rendererType = t)} />
  </div>
</div>

{#if headerContextMenu}
  <ContextMenu
    position={headerContextMenu}
    items={getContextMenuItems()}
    onClose={() => (headerContextMenu = null)}
  />
{/if}

<style>
  .deck-pane {
    position: absolute;
    top: 0;
    height: 100%;
    box-sizing: border-box;
    overflow: hidden;
    transition: width var(--pane-transition-ms) ease-out, left var(--pane-transition-ms) ease-out, opacity var(--pane-transition-ms) ease-out, filter var(--pane-transition-ms) ease-out;
    display: flex;
    flex-direction: column;
    border-radius: 4px;
  }

  .is-busy {
    animation: busy-pulse 2s ease-in-out infinite;
  }

  @keyframes busy-pulse {
    0%, 100% { box-shadow: var(--box-shadow-base, none), 0 0 4px 1px color-mix(in srgb, var(--border-color) 30%, transparent); }
    50% { box-shadow: var(--box-shadow-base, none), 0 0 10px 3px color-mix(in srgb, var(--border-color) 60%, transparent); }
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
    text-overflow: clip;
    font-family: system-ui, -apple-system, sans-serif;
    flex: 1;
    min-width: 0;
    cursor: text;
  }

  .session-cwd {
    font-size: 0.7rem;
    color: #555;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: clip;
    font-family: system-ui, -apple-system, sans-serif;
    flex-shrink: 0;
    max-width: 40%;
    margin-left: 0.25rem;
  }

  .session-dynamic {
    font-size: 0.7rem;
    color: #666;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-family: system-ui, -apple-system, sans-serif;
    flex-shrink: 1;
    min-width: 0;
    margin-left: 0.2rem;
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

  .header-actions {
    display: flex;
    align-items: center;
    gap: 1px;
    flex-shrink: 0;
    margin-left: 0.25rem;
  }

  .hdr-btn {
    background: none;
    border: none;
    color: #666;
    cursor: pointer;
    font-size: 0.7rem;
    width: 18px;
    height: 18px;
    border-radius: 3px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    line-height: 1;
  }

  .hdr-btn:hover {
    background: #2a2a2a;
    color: #d9d4c7;
  }

  .hdr-btn-close:hover {
    background: #ef444420;
    color: #ef4444;
  }
</style>
