<script lang="ts">
  interface Props {
    prefixKey: string;
    onClose: () => void;
  }

  let { prefixKey, onClose }: Props = $props();

  let bindings = $derived([
    { key: prefixKey, description: "Enter / exit Navigation Mode" },
    { key: "h / ←", description: "Focus previous session" },
    { key: "l / →", description: "Focus next session" },
    { key: "n", description: "New session" },
    { key: "b", description: "Park current session (move to shelf)" },
    { key: "j / ↓", description: "Select next shelf session" },
    { key: "k / ↑", description: "Select previous shelf session" },
    { key: "Enter", description: "Recall selected shelf session" },
    { key: "r", description: "Rename current session" },
    { key: "/", description: "Search sessions" },
    { key: "?", description: "Show this help" },
    { key: "x", description: "Close current session (graceful)" },
    { key: "X", description: "Kill current session (force)" },
    { key: "Esc", description: "Exit Navigation Mode" },
  ]);

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" || e.key === "?") {
      e.preventDefault();
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}>
  <div class="panel" onclick={(e) => e.stopPropagation()}>
    <div class="header">
      <span class="title">Keyboard Shortcuts</span>
      <button class="close-btn" onclick={onClose}>✕</button>
    </div>

    <div class="section-label">Navigation Mode</div>
    <table class="bindings">
      <tbody>
        {#each bindings as binding}
          <tr>
            <td class="key-cell"><kbd class="key">{binding.key}</kbd></td>
            <td class="desc-cell">{binding.description}</td>
          </tr>
        {/each}
      </tbody>
    </table>

    <div class="footer">Press <kbd class="key-inline">Esc</kbd> or <kbd class="key-inline">?</kbd> to close</div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .panel {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 8px;
    width: 440px;
    max-width: 90vw;
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
    font-family: system-ui, -apple-system, sans-serif;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #2a2a2a;
  }

  .title {
    color: #d9d4c7;
    font-size: 0.9rem;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    color: #555;
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
  }

  .close-btn:hover {
    color: #d9d4c7;
    background: #2a2a2a;
  }

  .section-label {
    padding: 0.5rem 1rem 0.25rem;
    font-size: 0.65rem;
    color: #555;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .bindings {
    width: 100%;
    border-collapse: collapse;
    padding: 0 0.5rem;
  }

  .bindings tr:hover {
    background: #222;
  }

  .key-cell {
    padding: 0.3rem 0.5rem 0.3rem 1rem;
    width: 130px;
    vertical-align: middle;
  }

  .key {
    display: inline-block;
    background: #252525;
    border: 1px solid #3a3a3a;
    border-radius: 4px;
    padding: 0.1rem 0.4rem;
    font-size: 0.72rem;
    color: #aaa;
    font-family: "Menlo", "Monaco", monospace;
    white-space: nowrap;
  }

  .desc-cell {
    padding: 0.3rem 1rem 0.3rem 0.25rem;
    font-size: 0.8rem;
    color: #888;
    vertical-align: middle;
  }

  .footer {
    padding: 0.6rem 1rem;
    border-top: 1px solid #2a2a2a;
    font-size: 0.72rem;
    color: #444;
    text-align: center;
  }

  .key-inline {
    background: #252525;
    border: 1px solid #3a3a3a;
    border-radius: 3px;
    padding: 0.05rem 0.3rem;
    font-size: 0.7rem;
    color: #888;
    font-family: "Menlo", "Monaco", monospace;
  }
</style>
