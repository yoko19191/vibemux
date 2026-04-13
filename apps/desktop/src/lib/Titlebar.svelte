<script lang="ts">
  interface Props {
    prefixKey: string;
    onNewSession: () => void;
    onSearch: () => void;
    onSettings: () => void;
  }

  let { prefixKey, onNewSession, onSearch, onSettings }: Props = $props();
</script>

<div class="titlebar">
  <div class="titlebar-left">
    <span class="workspace-dot"></span>
    <span class="workspace-label">1 Default</span>
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
      title="Search Sessions ({prefixKey}, /)"
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
    height: 36px;
    background: #161616;
    border-bottom: 1px solid transparent;
    border-image: linear-gradient(to right, #3b82f640, #3b82f620, transparent) 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 0.75rem;
    z-index: 100;
    font-family: system-ui, -apple-system, sans-serif;
    user-select: none;
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .workspace-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #3b82f6;
    flex-shrink: 0;
  }

  .workspace-label {
    color: #888;
    font-size: 0.75rem;
  }

  .titlebar-right {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .tb-btn {
    background: none;
    border: none;
    color: #888;
    cursor: pointer;
    font-size: 1rem;
    width: 28px;
    height: 28px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    transition: background 0.1s, color 0.1s;
  }

  .tb-btn:hover {
    background: #2a2a2a;
    color: #d9d4c7;
  }
</style>
