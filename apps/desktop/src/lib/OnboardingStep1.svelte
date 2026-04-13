<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface Props {
    onComplete: (prefixKey: string) => void;
  }

  let { onComplete }: Props = $props();

  const prefixOptions = [
    { value: "ctrl+b", label: "Ctrl+B", sublabel: "tmux style", recommended: true },
    { value: "ctrl+space", label: "Ctrl+Space", sublabel: "Spacemacs style", recommended: false },
    { value: "ctrl+`", label: "Ctrl+`", sublabel: "Backtick, no conflicts", recommended: false },
    { value: "ctrl+a", label: "Ctrl+A", sublabel: "screen style", recommended: false },
  ];

  let selected = $state("ctrl+b");

  async function handleNext() {
    await invoke("config_update", { update: { keys: { prefix: selected } } }).catch(() => {});
    onComplete(selected);
  }

  async function handleSkip() {
    await invoke("config_update", { update: { keys: { prefix: "ctrl+b" }, onboarding_completed: true } }).catch(() => {});
    onComplete("ctrl+b");
  }

  async function handleClose() {
    await invoke("config_update", { update: { keys: { prefix: "ctrl+b" }, onboarding_completed: true } }).catch(() => {});
    onComplete("ctrl+b");
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      handleClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="overlay">
  <div class="panel">
    <button class="close-btn" onclick={handleClose} aria-label="Close">✕</button>

    <div class="step-content">
      <h2 class="title">Choose your Navigation Key</h2>
      <p class="explanation">Press this key combo to enter Navigation Mode, where you can switch sessions, park, recall, and more.</p>

      <div class="options-grid">
        {#each prefixOptions as opt}
          <button
            class="option-card"
            class:selected={selected === opt.value}
            onclick={() => (selected = opt.value)}
          >
            <span class="option-key">{opt.label}</span>
            <span class="option-sub">{opt.sublabel}</span>
            {#if opt.recommended}
              <span class="recommended-badge">Recommended</span>
            {/if}
          </button>
        {/each}
      </div>
    </div>

    <div class="footer">
      <div class="step-dots">
        <span class="dot active"></span>
        <span class="dot"></span>
        <span class="dot"></span>
        <span class="dot"></span>
      </div>
      <span class="step-label">Step 1 of 4</span>
      <div class="footer-actions">
        <button class="skip-link" onclick={handleSkip}>Skip</button>
        <button class="next-btn" onclick={handleNext}>Next →</button>
      </div>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 500;
    font-family: system-ui, -apple-system, sans-serif;
  }

  .panel {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 12px;
    width: 480px;
    max-width: 92vw;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.7);
    position: relative;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .close-btn {
    position: absolute;
    top: 0.75rem;
    right: 0.75rem;
    background: none;
    border: none;
    color: #555;
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    z-index: 1;
  }

  .close-btn:hover {
    color: #d9d4c7;
    background: #2a2a2a;
  }

  .step-content {
    padding: 2rem 2rem 1rem;
  }

  .title {
    color: #d9d4c7;
    font-size: 1.2rem;
    font-weight: 600;
    margin: 0 0 0.5rem;
  }

  .explanation {
    color: #888;
    font-size: 0.82rem;
    line-height: 1.5;
    margin: 0 0 1.5rem;
  }

  .options-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  .option-card {
    background: #222;
    border: 2px solid #333;
    border-radius: 8px;
    padding: 0.9rem 1rem;
    cursor: pointer;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    transition: border-color 0.15s, background 0.15s;
    position: relative;
  }

  .option-card:hover {
    border-color: #555;
    background: #282828;
  }

  .option-card.selected {
    border-color: #3b82f6;
    background: rgba(59, 130, 246, 0.08);
  }

  .option-key {
    color: #d9d4c7;
    font-size: 0.95rem;
    font-weight: 600;
    font-family: "Menlo", "Monaco", monospace;
  }

  .option-sub {
    color: #666;
    font-size: 0.72rem;
  }

  .recommended-badge {
    position: absolute;
    top: 0.4rem;
    right: 0.5rem;
    background: #3b82f620;
    color: #3b82f6;
    font-size: 0.62rem;
    padding: 0.1rem 0.35rem;
    border-radius: 3px;
    font-weight: 600;
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

  .next-btn:hover {
    background: #2563eb;
  }
</style>
