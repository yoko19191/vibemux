<script lang="ts">
  import type { SessionSnapshot } from "./types";

  interface Props {
    sessions: SessionSnapshot[];
    onSelect: (sessionId: string, thermal: "Hot" | "Warm") => void;
    onClose: () => void;
  }

  let { sessions, onSelect, onClose }: Props = $props();

  let query = $state("");
  let selectedIdx = $state(0);
  let inputEl: HTMLInputElement | null = $state(null);

  const colorMap: Record<string, string> = {
    Red: "#ef4444",
    Orange: "#f97316",
    Yellow: "#eab308",
    Green: "#22c55e",
    Cyan: "#06b6d4",
    Blue: "#3b82f6",
    Purple: "#a855f7",
    Pink: "#ec4899",
  };

  const thermalLabel: Record<string, string> = {
    Hot: "hot",
    Warm: "warm",
    Cold: "cold",
  };

  let filtered = $derived.by(() => {
    const q = query.toLowerCase().trim();
    if (!q) return sessions;
    return sessions.filter(
      (s) =>
        s.name.toLowerCase().includes(q) ||
        s.cwd.toLowerCase().includes(q) ||
        (s.terminalTitle && s.terminalTitle.toLowerCase().includes(q))
    );
  });

  $effect(() => {
    // Reset selection when filter changes
    selectedIdx = 0;
  });

  $effect(() => {
    // Focus input on mount
    setTimeout(() => inputEl?.focus(), 0);
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIdx = Math.min(selectedIdx + 1, filtered.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIdx = Math.max(selectedIdx - 1, 0);
    } else if (e.key === "Enter") {
      e.preventDefault();
      const session = filtered[selectedIdx];
      if (session) {
        onSelect(session.id, session.thermalState as "Hot" | "Warm");
      }
    }
  }

  function shortCwd(cwd: string): string {
    const home = cwd.startsWith("/Users/") || cwd.startsWith("/home/");
    if (home) {
      const parts = cwd.split("/");
      return "~/" + parts.slice(3).join("/");
    }
    return cwd;
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}>
  <div class="panel" onclick={(e) => e.stopPropagation()}>
    <div class="search-row">
      <span class="search-icon">⌕</span>
      <input
        class="search-input"
        bind:this={inputEl}
        bind:value={query}
        placeholder="Search sessions..."
        onkeydown={handleKeydown}
      />
    </div>

    {#if filtered.length === 0}
      <div class="empty">No sessions match</div>
    {:else}
      <ul class="results">
        {#each filtered as session, i (session.id)}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <li
            class="result-item"
            class:selected={i === selectedIdx}
            onclick={() => onSelect(session.id, session.thermalState as "Hot" | "Warm")}
          >
            <span class="color-dot" style="background: {colorMap[session.color] ?? '#666'};"></span>
            <span class="session-name">{session.name}</span>
            <span class="thermal-badge" class:warm={session.thermalState === 'Warm'}>{thermalLabel[session.thermalState] ?? session.thermalState}</span>
            <span class="cwd">{shortCwd(session.cwd)}</span>
          </li>
        {/each}
      </ul>
    {/if}

    <div class="footer">
      <span>↑↓ navigate</span>
      <span>↵ select</span>
      <span>esc close</span>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 15vh;
    z-index: 200;
  }

  .panel {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 8px;
    width: 520px;
    max-width: 90vw;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
    overflow: hidden;
    font-family: system-ui, -apple-system, sans-serif;
  }

  .search-row {
    display: flex;
    align-items: center;
    padding: 0.6rem 0.75rem;
    border-bottom: 1px solid #2a2a2a;
    gap: 0.5rem;
  }

  .search-icon {
    color: #666;
    font-size: 1rem;
  }

  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: #d9d4c7;
    font-size: 0.9rem;
    font-family: inherit;
  }

  .search-input::placeholder {
    color: #555;
  }

  .results {
    list-style: none;
    margin: 0;
    padding: 0.25rem 0;
    max-height: 320px;
    overflow-y: auto;
  }

  .result-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.45rem 0.75rem;
    cursor: pointer;
    font-size: 0.82rem;
  }

  .result-item:hover,
  .result-item.selected {
    background: #2a2a2a;
  }

  .color-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .session-name {
    color: #d9d4c7;
    flex-shrink: 0;
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .thermal-badge {
    font-size: 0.65rem;
    padding: 0.1rem 0.35rem;
    border-radius: 3px;
    background: #3b82f620;
    color: #3b82f6;
    flex-shrink: 0;
  }

  .thermal-badge.warm {
    background: #f9731620;
    color: #f97316;
  }

  .cwd {
    color: #555;
    font-size: 0.75rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }

  .empty {
    padding: 1.5rem;
    text-align: center;
    color: #555;
    font-size: 0.85rem;
    font-family: system-ui, -apple-system, sans-serif;
  }

  .footer {
    display: flex;
    gap: 1rem;
    padding: 0.4rem 0.75rem;
    border-top: 1px solid #2a2a2a;
    color: #444;
    font-size: 0.7rem;
    font-family: system-ui, -apple-system, sans-serif;
  }
</style>
