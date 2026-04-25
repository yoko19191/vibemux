<script lang="ts">
  interface Props {
    prefixKey: string;
    focusedTitle: string;
    focusedAccentColor: string;
    isMacOS: boolean;
    onNewSession: () => void;
    onSearch: () => void;
    onSettings: () => void;
  }

  let { prefixKey, focusedTitle, focusedAccentColor, isMacOS, onNewSession, onSearch, onSettings }: Props = $props();
</script>

<div class="titlebar">
  <!-- Full-coverage drag layer sits behind everything -->
  <div class="drag-layer" data-tauri-drag-region></div>

  <div class="titlebar-left">
    {#if isMacOS}
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
      title="Search Sessions (Cmd+K or {prefixKey}, /)"
      aria-label="Search Sessions"
    >⌕</button>
    <button
      class="tb-btn"
      onclick={onSettings}
      title="Settings"
      aria-label="Settings"
    >⚙</button>
  </div>
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
</style>
