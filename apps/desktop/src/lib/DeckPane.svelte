<script lang="ts">
  import TerminalPane from "./TerminalPane.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import BusyIndicator from "./BusyIndicator.svelte";
  import type { ContextMenuItem } from "./ContextMenu.svelte";
  import type { ColorToken, ProcessState } from "./types";
  import { invoke } from "@tauri-apps/api/core";

  interface TerminalConfig {
    fontFamily?: string;
    fontSize?: number;
    lineHeight?: number;
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
    isRenaming?: boolean;
    terminalConfig?: TerminalConfig;
    onReady?: (api: { writeOutput: (data: string) => void }) => void;
    onclick?: () => void;
    ondragstart?: (e: DragEvent) => void;
    ondragover?: (e: DragEvent) => void;
    ondrop?: (e: DragEvent) => void;
    onRenameConfirm?: (name: string) => void;
    onRenameCancel?: () => void;
    onPark?: () => void;
    onClose?: () => void;
  }

  let {
    sessionId, sessionName, sessionCwd = "", terminalTitle = "", color, processState, isFocused, width,
    isRenaming = false, terminalConfig,
    onReady, onclick, ondragstart, ondragover, ondrop,
    onRenameConfirm, onRenameCancel, onPark, onClose,
  }: Props = $props();

  let displayName = $derived(sessionName || terminalTitle);
  let shortCwd = $derived(sessionCwd.replace(/^.*\/([^/]+)$/, "$1") || sessionCwd);
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

  const colorTokens: { token: ColorToken; color: string }[] = [
    { token: "Red", color: "#ef4444" },
    { token: "Orange", color: "#f97316" },
    { token: "Yellow", color: "#eab308" },
    { token: "Green", color: "#22c55e" },
    { token: "Cyan", color: "#06b6d4" },
    { token: "Blue", color: "#3b82f6" },
    { token: "Purple", color: "#a855f7" },
    { token: "Pink", color: "#ec4899" },
  ];

  let isBusy = $derived(processState?.type === "Running");
  let borderColor = $derived(colorMap[color] ?? "#666");
  let borderStyle = $derived(isFocused
    ? `2px solid ${borderColor}`
    : `1px solid ${borderColor}80`);
  let boxShadow = $derived(isFocused
    ? `0 0 0 1px ${borderColor}33, 0 4px 16px rgba(0,0,0,0.6), inset 0 1px 0 rgba(255,255,255,0.05)`
    : `0 2px 8px rgba(0,0,0,0.4)`);
  let opacity = $derived(isFocused ? 1 : 0.8);
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
      { label: "Park to Shelf", onClick: () => onPark?.() },
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
      { label: "Kill", color: "#ef4444", onClick: () => invoke("session_kill", { sessionId }).catch(console.error) },
    ];
  }

  function startRename() {
    // We do this by calling onRenameConfirm with the current name (no-op rename)
    // Actually we need a dedicated onStartRename callback — but to keep it simple,
    // we'll just call onRenameConfirm with the current name which triggers the rename flow
    // The parent sets renamingSessionId which causes isRenaming=true
    // For now, we'll use a workaround: dispatch a custom event
    const el = document.querySelector(`[data-session-id="${sessionId}"]`);
    el?.dispatchEvent(new CustomEvent("startRename", { bubbles: true, detail: { sessionId } }));
  }

  function getContextMenuItems(): ContextMenuItem[] {
    return [
      { label: "Park to Shelf", onClick: () => onPark?.() },
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
      { label: "Kill", color: "#ef4444", onClick: () => invoke("session_kill", { sessionId }).catch(console.error) },
    ];
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="deck-pane"
  class:drag-over={dragOver}
  data-session-id={sessionId}
  style="width: {width}px; border: {borderStyle}; box-shadow: {boxShadow}; opacity: {opacity};"
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
      <span class="session-name">{displayName}</span>
      {#if shortCwd}
        <span class="session-cwd">{shortCwd}</span>
      {/if}
      {#if rendererType === 'canvas'}
        <span class="renderer-badge">canvas</span>
      {/if}
      {#if isFocused && isHovered}
        <div class="header-actions" onclick={(e) => e.stopPropagation()}>
          <button class="hdr-btn" title="Park to Shelf (Ctrl+B, B)" onclick={() => onPark?.()}>⬇</button>
          <button class="hdr-btn" title="Rename (Ctrl+B, R)" onclick={startRename}>✎</button>
          <button class="hdr-btn hdr-btn-close" title="Close (Ctrl+B, X)" onclick={() => onClose?.()}>✕</button>
        </div>
      {/if}
    {/if}
  </div>
  <div class="terminal-container">
    <TerminalPane {sessionId} {terminalConfig} {onReady} onRendererType={(t) => (rendererType = t)} />
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
    height: 100%;
    box-sizing: border-box;
    overflow: hidden;
    flex-shrink: 0;
    transition: width 150ms ease-out, opacity 150ms ease-out;
    display: flex;
    flex-direction: column;
    border-radius: 4px;
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
