<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { isMacOS, primaryShortcutModifier, type DesktopPlatform } from "./platform";

  type ResizeDirection = "East" | "North" | "NorthEast" | "NorthWest" | "South" | "SouthEast" | "SouthWest" | "West";

  interface Props {
    prefixKey: string;
    focusedTitle: string;
    focusedAccentColor: string;
    platform: DesktopPlatform;
    onNewSession: () => void;
    onSearch: () => void;
    onSettings: () => void;
  }

  let { prefixKey, focusedTitle, focusedAccentColor, platform, onNewSession, onSearch, onSettings }: Props = $props();

  const appWindow = getCurrentWindow();

  let maximized = $state(false);
  let isMac = $derived(isMacOS(platform));
  let showsWindowControls = $derived(!isMac);
  let searchShortcut = $derived(`${primaryShortcutModifier(platform)}+K`);
  const resizeHandles: { direction: ResizeDirection; className: string }[] = [
    { direction: "North", className: "resize-n" },
    { direction: "South", className: "resize-s" },
    { direction: "East", className: "resize-e" },
    { direction: "West", className: "resize-w" },
    { direction: "NorthEast", className: "resize-ne" },
    { direction: "NorthWest", className: "resize-nw" },
    { direction: "SouthEast", className: "resize-se" },
    { direction: "SouthWest", className: "resize-sw" },
  ];

  onMount(() => {
    if (showsWindowControls) {
      appWindow.isMaximized().then((value) => (maximized = value)).catch(() => {});
    }
  });

  function minimizeWindow() {
    appWindow.minimize().catch(console.error);
  }

  async function toggleMaximize() {
    try {
      await appWindow.toggleMaximize();
      maximized = await appWindow.isMaximized();
    } catch (e) {
      console.error(e);
    }
  }

  function hideWindow() {
    appWindow.hide().catch(console.error);
  }

  function startResize(direction: ResizeDirection, e: PointerEvent) {
    if (!showsWindowControls) return;
    if (e.button !== 0) return;
    e.preventDefault();
    appWindow.startResizeDragging(direction).catch(console.error);
  }
</script>

<div class="titlebar">
  <!-- Full-coverage drag layer sits behind everything -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="drag-layer"
    data-tauri-drag-region
    ondblclick={() => {
      if (showsWindowControls) toggleMaximize();
    }}
  ></div>

  <div class="titlebar-left">
    {#if isMac}
      <div class="traffic-light-spacer"></div>
    {/if}
    <span class="prefix-hint">{prefixKey}</span>
  </div>

  <div class="titlebar-center">
    <span class="session-title" style="color: {focusedAccentColor};">{focusedTitle}</span>
  </div>

  <div class="titlebar-right">
    <button
      class="tb-btn"
      onclick={onNewSession}
      title="New Session ({prefixKey}, N)"
      aria-label="New Session"
    >+</button>
    <button
      class="tb-btn"
      onclick={onSearch}
      title="Search Sessions ({searchShortcut} or {prefixKey}, /)"
      aria-label="Search Sessions"
    >⌕</button>
    <button
      class="tb-btn"
      onclick={onSettings}
      title="Settings"
      aria-label="Settings"
    >⚙</button>
    {#if showsWindowControls}
      <div class="window-controls" aria-label="Window controls">
        <button class="window-btn" onclick={minimizeWindow} title="Minimize" aria-label="Minimize">-</button>
        <button
          class="window-btn"
          onclick={toggleMaximize}
          title={maximized ? "Restore" : "Maximize"}
          aria-label={maximized ? "Restore" : "Maximize"}
        >{maximized ? "▣" : "□"}</button>
        <button class="window-btn close" onclick={hideWindow} title="Close to tray" aria-label="Close to tray">×</button>
      </div>
    {/if}
  </div>

  {#if showsWindowControls}
    {#each resizeHandles as handle}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="resize-handle {handle.className}"
        onpointerdown={(e) => startResize(handle.direction, e)}
      ></div>
    {/each}
  {/if}
</div>

<style>
  .titlebar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 28px;
    background: #161616;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    z-index: 100;
    font-family: system-ui, -apple-system, sans-serif;
    user-select: none;
  }

  .drag-layer {
    position: absolute;
    inset: 0;
    z-index: 0;
  }

  .titlebar-left {
    position: absolute;
    left: 0.5rem;
    top: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    z-index: 1;
    pointer-events: none;
  }

  .traffic-light-spacer {
    width: 68px;
    flex-shrink: 0;
  }

  .prefix-hint {
    color: #555;
    font-size: 0.65rem;
    letter-spacing: 0.02em;
  }

  .titlebar-center {
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1;
    pointer-events: none;
  }

  .session-title {
    font-size: 0.7rem;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 50%;
  }

  .titlebar-right {
    position: absolute;
    right: 0.5rem;
    top: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    gap: 0.15rem;
    z-index: 1;
  }

  .tb-btn {
    background: none;
    border: none;
    color: #666;
    font-size: 0.8rem;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 3px;
    line-height: 1;
    transition: background 100ms ease, color 100ms ease;
  }

  .tb-btn:hover {
    background: #2a2a2a;
    color: #d9d4c7;
  }

  .window-controls {
    display: flex;
    align-items: stretch;
    align-self: stretch;
    margin-left: 0.3rem;
    border-left: 1px solid rgba(255, 255, 255, 0.06);
  }

  .window-btn {
    width: 34px;
    height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: #777;
    cursor: default;
    font-family: system-ui, -apple-system, sans-serif;
    font-size: 0.8rem;
    line-height: 1;
    padding: 0;
    transition: background 100ms ease, color 100ms ease;
  }

  .window-btn:hover {
    background: #2a2a2a;
    color: #d9d4c7;
  }

  .window-btn.close:hover {
    background: #c42b1c;
    color: white;
  }

  .resize-handle {
    position: fixed;
    z-index: 200;
  }

  .resize-n,
  .resize-s {
    left: 6px;
    right: 6px;
    height: 6px;
    cursor: ns-resize;
  }

  .resize-n {
    top: 0;
  }

  .resize-s {
    bottom: 0;
  }

  .resize-e,
  .resize-w {
    top: 6px;
    bottom: 6px;
    width: 6px;
    cursor: ew-resize;
  }

  .resize-e {
    right: 0;
  }

  .resize-w {
    left: 0;
  }

  .resize-ne,
  .resize-nw,
  .resize-se,
  .resize-sw {
    width: 10px;
    height: 10px;
  }

  .resize-ne {
    top: 0;
    right: 0;
    cursor: nesw-resize;
  }

  .resize-nw {
    top: 0;
    left: 0;
    cursor: nwse-resize;
  }

  .resize-se {
    right: 0;
    bottom: 0;
    cursor: nwse-resize;
  }

  .resize-sw {
    left: 0;
    bottom: 0;
    cursor: nesw-resize;
  }
</style>
