<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { SessionSnapshot } from "./types";

  interface Props {
    defaultCwd: string;
    onCreated?: (snapshot: SessionSnapshot) => void;
    onCancel?: () => void;
  }

  let { defaultCwd, onCreated, onCancel }: Props = $props();

  let name = $state("shell");
  let cwd = $derived.by(() => cwdOverride ?? defaultCwd);
  let cwdOverride = $state<string | null>(null);
  let commandType = $state<"shell" | "command">("shell");
  let program = $state("");
  let args = $state("");
  let errorMsg = $state<string | null>(null);
  let submitting = $state(false);
  let nameInput: HTMLInputElement;

  function handleCwdInput(e: Event) {
    cwdOverride = (e.target as HTMLInputElement).value;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onCancel?.();
    }
  }

  async function handleSubmit() {
    errorMsg = null;
    submitting = true;

    try {
      const payload: Record<string, unknown> = {
        name,
        cwd,
        commandType,
      };

      if (commandType === "command") {
        payload.program = program;
        payload.args = args
          .split(" ")
          .map((s) => s.trim())
          .filter(Boolean);
      }

      const snapshot: SessionSnapshot = await invoke("session_create", { payload });
      onCreated?.(snapshot);
    } catch (e) {
      errorMsg = String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onCancel}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <form class="panel" onclick={(e) => e.stopPropagation()} onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    <h2>New Session</h2>

    <label>
      Name
      <!-- svelte-ignore a11y_autofocus -->
      <input type="text" bind:this={nameInput} bind:value={name} placeholder="session name" autofocus />
    </label>

    <label>
      Working Directory
      <input type="text" value={cwd} oninput={handleCwdInput} placeholder="/path/to/dir" />
    </label>

    <fieldset>
      <legend>Command Type</legend>
      <label class="radio">
        <input type="radio" bind:group={commandType} value="shell" />
        Shell
      </label>
      <label class="radio">
        <input type="radio" bind:group={commandType} value="command" />
        Custom Command
      </label>
    </fieldset>

    {#if commandType === "command"}
      <label>
        Program
        <input type="text" bind:value={program} placeholder="e.g. python3" />
      </label>
      <label>
        Arguments
        <input type="text" bind:value={args} placeholder="e.g. -m http.server 8000" />
      </label>
    {/if}

    {#if errorMsg}
      <div class="error">{errorMsg}</div>
    {/if}

    <div class="actions">
      <button type="button" class="cancel" onclick={onCancel}>Cancel</button>
      <button type="submit" class="submit" disabled={submitting}>
        {submitting ? "Creating..." : "Create"}
      </button>
    </div>
  </form>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .panel {
    background: #1a1a2e;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 1.5rem;
    width: 400px;
    max-width: 90vw;
    color: #d9d4c7;
    font-family: system-ui, -apple-system, sans-serif;
  }

  h2 {
    margin: 0 0 1rem;
    font-size: 1.2rem;
    font-weight: 500;
  }

  label {
    display: block;
    margin-bottom: 0.75rem;
    font-size: 0.85rem;
    color: #999;
  }

  input[type="text"] {
    display: block;
    width: 100%;
    margin-top: 0.25rem;
    padding: 0.5rem;
    background: #111;
    border: 1px solid #444;
    border-radius: 4px;
    color: #d9d4c7;
    font-size: 0.9rem;
    box-sizing: border-box;
  }

  input[type="text"]:focus {
    outline: none;
    border-color: #3b82f6;
  }

  fieldset {
    border: 1px solid #333;
    border-radius: 4px;
    padding: 0.5rem 0.75rem;
    margin: 0 0 0.75rem;
  }

  legend {
    font-size: 0.85rem;
    color: #999;
    padding: 0 0.25rem;
  }

  .radio {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    margin-right: 1rem;
    color: #d9d4c7;
    font-size: 0.9rem;
  }

  .error {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid #ef4444;
    border-radius: 4px;
    padding: 0.5rem 0.75rem;
    margin-bottom: 0.75rem;
    color: #ef4444;
    font-size: 0.85rem;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }

  button {
    padding: 0.5rem 1rem;
    border-radius: 4px;
    border: none;
    font-size: 0.9rem;
    cursor: pointer;
  }

  .cancel {
    background: #333;
    color: #d9d4c7;
  }

  .cancel:hover {
    background: #444;
  }

  .submit {
    background: #3b82f6;
    color: white;
  }

  .submit:hover:not(:disabled) {
    background: #2563eb;
  }

  .submit:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
