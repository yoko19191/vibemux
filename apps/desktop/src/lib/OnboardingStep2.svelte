<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  interface Props {
    onNext: (shell: string) => void;
    onBack: () => void;
    onSkip: () => void;
  }

  let { onNext, onBack, onSkip }: Props = $props();

  let shells: string[] = $state([]);
  let selected = $state("");

  onMount(async () => {
    try {
      const detected = await invoke<string[]>("detect_shells");
      shells = detected;
      const envShell = (window as any).__TAURI_INTERNALS__?.metadata?.currentDir ?? "";
      // Try to match $SHELL from env — we can't read env directly in frontend,
      // so just pre-select the first item or /bin/zsh if present
      const preferred = shells.find((s) => s.endsWith("/zsh")) ?? shells[0] ?? "";
      selected = preferred;
    } catch {
      shells = [];
    }
  });
</script>

<div class="step-content">
  <h2 class="title">Choose your default shell</h2>

  {#if shells.length === 0}
    <p class="empty">No shells detected.</p>
  {:else}
    <ul class="shell-list">
      {#each shells as shell}
        <li>
          <button
            class="shell-item"
            class:selected={selected === shell}
            onclick={() => (selected = shell)}
          >
            <span class="shell-path">{shell}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<div class="footer">
  <div class="step-dots">
    <span class="dot"></span>
    <span class="dot active"></span>
    <span class="dot"></span>
    <span class="dot"></span>
  </div>
  <span class="step-label">Step 2 of 4</span>
  <div class="footer-actions">
    <button class="back-btn" onclick={onBack}>← Back</button>
    <button class="skip-link" onclick={onSkip}>Skip</button>
    <button
      class="next-btn"
      disabled={!selected}
      onclick={async () => {
        if (selected) {
          await invoke("config_update", { update: { shell: { default: selected } } }).catch(() => {});
          onNext(selected);
        }
      }}
    >Next →</button>
  </div>
</div>

<style>
  .step-content {
    padding: 2rem 2rem 1rem;
    flex: 1;
  }

  .title {
    color: #d9d4c7;
    font-size: 1.2rem;
    font-weight: 600;
    margin: 0 0 1rem;
  }

  .empty {
    color: #555;
    font-size: 0.82rem;
  }

  .shell-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    max-height: 220px;
    overflow-y: auto;
  }

  .shell-item {
    width: 100%;
    background: #222;
    border: 2px solid #333;
    border-radius: 6px;
    padding: 0.6rem 0.9rem;
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s, background 0.15s;
  }

  .shell-item:hover {
    border-color: #555;
    background: #282828;
  }

  .shell-item.selected {
    border-color: #3b82f6;
    background: rgba(59, 130, 246, 0.08);
  }

  .shell-path {
    color: #d9d4c7;
    font-size: 0.85rem;
    font-family: "Menlo", "Monaco", monospace;
  }

  .footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 2rem;
    border-top: 1px solid #2a2a2a;
    gap: 1rem;
  }

  .step-dots {
    display: flex;
    gap: 0.35rem;
    align-items: center;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #333;
  }

  .dot.active {
    background: #3b82f6;
  }

  .step-label {
    color: #555;
    font-size: 0.72rem;
    flex: 1;
  }

  .footer-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .back-btn {
    background: none;
    border: 1px solid #333;
    color: #888;
    cursor: pointer;
    font-size: 0.8rem;
    padding: 0.35rem 0.7rem;
    border-radius: 5px;
  }

  .back-btn:hover {
    border-color: #555;
    color: #d9d4c7;
  }

  .skip-link {
    background: none;
    border: none;
    color: #555;
    cursor: pointer;
    font-size: 0.8rem;
    padding: 0.3rem 0.5rem;
    border-radius: 4px;
  }

  .skip-link:hover {
    color: #888;
  }

  .next-btn {
    background: #3b82f6;
    border: none;
    color: white;
    cursor: pointer;
    font-size: 0.82rem;
    font-weight: 600;
    padding: 0.45rem 1rem;
    border-radius: 6px;
    transition: background 0.15s;
  }

  .next-btn:hover:not(:disabled) {
    background: #2563eb;
  }

  .next-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
