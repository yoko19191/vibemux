<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Bell from "@lucide/svelte/icons/bell";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import Settings from "@lucide/svelte/icons/settings";
  import { isMacOS, primaryShortcutModifier, type DesktopPlatform } from "./platform";
  import type { AttentionState } from "./types";

  type ResizeDirection = "East" | "North" | "NorthEast" | "NorthWest" | "South" | "SouthEast" | "SouthWest" | "West";

  interface Props {
    prefixKey: string;
    focusedTitle: string;
    focusedAccentColor: string;
    focusedAttentionState: AttentionState;
    focusedOriginBadge: string | null;
    focusedOriginLabel: string | null;
    unreadNotifications: number;
    notificationsOpen: boolean;
    platform: DesktopPlatform;
    onNewSession: () => void;
    onSearch: () => void;
    onToggleNotifications: () => void;
    onSettings: () => void;
  }

  let {
    prefixKey,
    focusedTitle,
    focusedAccentColor,
    focusedAttentionState,
    focusedOriginBadge,
    focusedOriginLabel,
    unreadNotifications,
    notificationsOpen,
    platform,
    onNewSession,
    onSearch,
    onToggleNotifications,
    onSettings,
  }: Props = $props();

  const appWindow = getCurrentWindow();

  let maximized = $state(false);
  let isMac = $derived(isMacOS(platform));
  let showsWindowControls = $derived(!isMac);
  let searchShortcut = $derived(`${primaryShortcutModifier(platform)}+K`);
  let attentionTitle = $derived(focusedAttentionState === "NeedsInput" ? "Input needed" : "");
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
    <div class="session-title-wrap">
      {#if focusedAttentionState === "NeedsInput"}
        <span class="attention-chip" title={attentionTitle}>!</span>
      {/if}
      {#if focusedOriginBadge}
        <span class="origin-badge" title={focusedOriginLabel ?? focusedOriginBadge}>{focusedOriginBadge}</span>
      {/if}
      <span class="session-title" style="color: {focusedAccentColor};">{focusedTitle}</span>
    </div>
  </div>

  <div class="titlebar-right">
    <button
      class="tb-btn notification-btn"
      class:active={notificationsOpen}
      onclick={onToggleNotifications}
      title="Notifications ({prefixKey}, O)"
      aria-label="Notifications"
    >
      <Bell size={14} strokeWidth={2} aria-hidden="true" />
      {#if unreadNotifications > 0}
        <span class="notification-dot" aria-hidden="true"></span>
      {/if}
    </button>
    <button
      class="tb-btn"
      onclick={onNewSession}
      title="New Session ({prefixKey}, N)"
      aria-label="New Session"
    >
      <Plus size={14} strokeWidth={2} aria-hidden="true" />
    </button>
    <button
      class="tb-btn"
      onclick={onSearch}
      title="Search Sessions ({searchShortcut} or {prefixKey}, /)"
      aria-label="Search Sessions"
    >
      <Search size={14} strokeWidth={2} aria-hidden="true" />
    </button>
    <button
      class="tb-btn"
      onclick={onSettings}
      title="Settings"
      aria-label="Settings"
    >
      <Settings size={14} strokeWidth={2} aria-hidden="true" />
    </button>
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
  }

  .titlebar-left .prefix-hint,
  .traffic-light-spacer {
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
    min-width: 0;
  }

  .session-title-wrap {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.35rem;
    max-width: 52%;
    min-width: 0;
  }

  .attention-chip {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #facc15;
    color: #111;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 0.62rem;
    font-weight: 800;
    line-height: 1;
    flex-shrink: 0;
  }

  .origin-badge {
    color: #facc15;
    border: 1px solid rgba(250, 204, 21, 0.45);
    border-radius: 3px;
    padding: 0 0.22rem;
    font-size: 0.56rem;
    font-weight: 750;
    line-height: 1.2;
    flex-shrink: 0;
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
    width: 24px;
    height: 22px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: #666;
    cursor: pointer;
    padding: 0;
    border-radius: 3px;
    line-height: 1;
    transition: background 100ms ease, color 100ms ease;
  }

  .tb-btn:hover {
    background: #2a2a2a;
    color: #d9d4c7;
  }

  .tb-btn.active {
    background: #252525;
    color: #d9d4c7;
  }

  .notification-btn {
    position: relative;
  }

  .notification-dot {
    position: absolute;
    top: 4px;
    right: 4px;
    width: 6px;
    height: 6px;
    border: 1px solid #161616;
    border-radius: 50%;
    background: #ef4444;
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
