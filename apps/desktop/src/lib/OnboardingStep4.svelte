<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface Props {
    prefixKey: string;
    onStart: () => void;
    onBack: () => void;
  }

  let { prefixKey, onStart, onBack }: Props = $props();

  async function handleStart() {
    await invoke("config_update", { update: { onboarding_completed: true } }).catch(() => {});
    onStart();
  }
</script>

<div class="step-content">
  <div class="checkmark">✓</div>
  <h2 class="title">You're all set!</h2>

  <div class="tip-box">
    <p class="tip">Press <kbd>{prefixKey}</kbd> to enter Navigation Mode.</p>
    <p class="tip">Try creating a new session with <kbd>{prefixKey}</kbd> then <kbd>N</kbd>.</p>
  </div>
</div>

<div class="footer">
  <div class="step-dots">
    <span class="dot"></span>
    <span class="dot"></span>
    <span class="dot"></span>
    <span class="dot active"></span>
  </div>
  <span class="step-label">Step 4 of 4</span>
  <div class="footer-actions">
    <button class="back-btn" onclick={onBack}>← Back</button>
    <button class="start-btn" onclick={handleStart}>Start →</button>
  </div>
</div>

<style>
  .step-content {
    padding: 2.5rem 2rem 1rem;
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .checkmark {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
    font-size: 1.6rem;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 1rem;
  }

  .title {
    color: #d9d4c7;
    font-size: 1.2rem;
    font-weight: 600;
    margin: 0 0 1.5rem;
    font-family: system-ui, -apple-system, sans-serif;
  }

  .tip-box {
    background: #1e1e1e;
    border: 1px solid #2a2a2a;
    border-radius: 8px;
    padding: 1rem 1.25rem;
    width: 100%;
    max-width: 340px;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .tip {
    color: #888;
    font-size: 0.82rem;
    margin: 0;
    font-family: system-ui, -apple-system, sans-serif;
    line-height: 1.5;
  }

  kbd {
    background: #252525;
    border: 1px solid #3a3a3a;
    border-radius: 4px;
    padding: 0.1rem 0.4rem;
    font-size: 0.75rem;
    color: #aaa;
    font-family: "Menlo", "Monaco", monospace;
  }

  .footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 2rem;
    border-top: 1px solid #2a2a2a;
    gap: 1rem;
    font-family: system-ui, -apple-system, sans-serif;
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

  .start-btn {
    background: #22c55e;
    border: none;
    color: white;
    cursor: pointer;
    font-size: 0.82rem;
    font-weight: 600;
    padding: 0.45rem 1.2rem;
    border-radius: 6px;
    transition: background 0.15s;
  }

  .start-btn:hover {
    background: #16a34a;
  }
</style>
