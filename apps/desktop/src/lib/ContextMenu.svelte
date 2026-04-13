<script lang="ts">
  export interface ContextMenuItem {
    label?: string;
    onClick?: () => void;
    color?: string;
    disabled?: boolean;
    type?: "separator" | "submenu";
    children?: ColorPickerItem[];
  }

  export interface ColorPickerItem {
    token: string;
    color: string;
    onSelect: () => void;
  }

  interface Props {
    position: { x: number; y: number };
    items: ContextMenuItem[];
    onClose: () => void;
  }

  let { position, items, onClose }: Props = $props();

  let openSubmenuIdx: number | null = $state(null);

  function handleItemClick(item: ContextMenuItem) {
    if (item.disabled || item.type === "separator") return;
    if (item.type === "submenu") return; // handled by hover
    item.onClick?.();
    onClose();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="ctx-overlay" onclick={onClose}>
  <div
    class="ctx-menu"
    style="left: {position.x}px; top: {position.y}px;"
    onclick={(e) => e.stopPropagation()}
  >
    {#each items as item, idx}
      {#if item.type === "separator"}
        <div class="ctx-divider"></div>
      {:else if item.type === "submenu"}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="ctx-item ctx-submenu-trigger"
          onmouseenter={() => (openSubmenuIdx = idx)}
          onmouseleave={() => (openSubmenuIdx = null)}
        >
          <span>{item.label}</span>
          <span class="submenu-arrow">›</span>
          {#if openSubmenuIdx === idx && item.children}
            <div class="ctx-submenu">
              {#each item.children as child}
                <button
                  class="color-dot-btn"
                  style="background: {child.color};"
                  onclick={(e) => { e.stopPropagation(); child.onSelect(); onClose(); }}
                  title={child.token}
                  aria-label={child.token}
                ></button>
              {/each}
            </div>
          {/if}
        </div>
      {:else}
        <button
          class="ctx-item"
          disabled={item.disabled}
          style={item.color ? `color: ${item.color};` : ""}
          onclick={() => handleItemClick(item)}
        >{item.label}</button>
      {/if}
    {/each}
  </div>
</div>

<style>
  .ctx-overlay {
    position: fixed;
    inset: 0;
    z-index: 200;
  }

  .ctx-menu {
    position: fixed;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 6px;
    padding: 0.25rem 0;
    min-width: 160px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
    font-family: system-ui, -apple-system, sans-serif;
    z-index: 201;
  }

  .ctx-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    background: none;
    border: none;
    color: #d9d4c7;
    font-size: 0.8rem;
    padding: 0.35rem 0.75rem;
    text-align: left;
    cursor: pointer;
    font-family: inherit;
    box-sizing: border-box;
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

  .ctx-submenu-trigger {
    position: relative;
    cursor: default;
  }

  .ctx-submenu-trigger:hover {
    background: #3b82f620;
  }

  .submenu-arrow {
    color: #555;
    font-size: 0.9rem;
  }

  .ctx-submenu {
    position: absolute;
    left: 100%;
    top: -0.25rem;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 6px;
    padding: 0.5rem 0.6rem;
    display: flex;
    gap: 0.4rem;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
    z-index: 202;
  }

  .color-dot-btn {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    padding: 0;
    transition: transform 0.1s, border-color 0.1s;
  }

  .color-dot-btn:hover {
    transform: scale(1.25);
    border-color: rgba(255, 255, 255, 0.4);
  }
</style>
