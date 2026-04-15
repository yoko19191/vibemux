<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  interface AiConfig {
    enabled: boolean;
    base_url: string;
    api_key: string;
    model: string;
  }

  interface UserConfig {
    ai: AiConfig;
  }

  interface Props {
    onNext: () => void;
    onBack: () => void;
    onSkip: () => void;
  }

  let { onNext, onBack, onSkip }: Props = $props();

  let baseUrl = $state("https://api.openai.com");
  let apiKey = $state("");
  let selectedModel = $state("");
  let models: string[] = $state([]);
  let loadingModels = $state(false);
  let saving = $state(false);
  let error: string | null = $state(null);

  onMount(async () => {
    try {
      const config = await invoke<UserConfig>("config_get");
      baseUrl = config.ai?.base_url || "https://api.openai.com";
      apiKey = config.ai?.api_key || "";
      selectedModel = config.ai?.model || "";
      if (selectedModel) models = [selectedModel];
    } catch {
      // Keep defaults. This step is optional, so config load failure should not block onboarding.
    }
  });

  async function fetchModels() {
    error = null;
    if (!baseUrl.trim() || !apiKey.trim()) {
      error = "Add a Base URL and API Key first.";
      return;
    }

    loadingModels = true;
    try {
      await invoke("config_update", {
        update: {
          ai: {
            base_url: baseUrl.trim(),
            api_key: apiKey.trim(),
          },
        },
      });
      const fetched = await invoke<string[]>("ai_list_models");
      models = fetched;
      if (models.length === 0) {
        selectedModel = "";
        error = "No models were returned by this endpoint.";
      } else if (!models.includes(selectedModel)) {
        selectedModel = models[0];
      }
    } catch (e) {
      error = String(e);
    } finally {
      loadingModels = false;
    }
  }

  async function handleNext() {
    error = null;
    if (!baseUrl.trim() || !apiKey.trim()) {
      error = "Add AI credentials or choose Skip.";
      return;
    }
    if (!selectedModel) {
      error = "Fetch models and select one before continuing.";
      return;
    }

    saving = true;
    try {
      await invoke("config_update", {
        update: {
          ai: {
            enabled: true,
            base_url: baseUrl.trim(),
            api_key: apiKey.trim(),
            model: selectedModel,
          },
        },
      });
      onNext();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<div class="step-content">
  <h2 class="title">Set up Ask AI</h2>
  <p class="explanation">Connect an OpenAI-compatible endpoint now, or skip and add it later in Settings.</p>

  <label class="field">
    <span>Base URL</span>
    <input
      type="text"
      bind:value={baseUrl}
      placeholder="https://api.openai.com"
      disabled={loadingModels || saving}
    />
  </label>

  <label class="field">
    <span>API Key</span>
    <input
      type="password"
      bind:value={apiKey}
      placeholder="sk-..."
      disabled={loadingModels || saving}
    />
  </label>

  <div class="model-row">
    <label class="field model-field">
      <span>Model</span>
      <select bind:value={selectedModel} disabled={models.length === 0 || loadingModels || saving}>
        <option value="">Select a model</option>
        {#each models as model}
          <option value={model}>{model}</option>
        {/each}
      </select>
    </label>
    <button class="fetch-btn" disabled={loadingModels || saving} onclick={fetchModels}>
      {loadingModels ? "Fetching..." : "Fetch Models"}
    </button>
  </div>

  {#if error}
    <div class="inline-error">{error}</div>
  {/if}
</div>

<div class="footer">
  <div class="step-dots">
    <span class="dot"></span>
    <span class="dot"></span>
    <span class="dot"></span>
    <span class="dot active"></span>
    <span class="dot"></span>
  </div>
  <span class="step-label">Step 4 of 5</span>
  <div class="footer-actions">
    <button class="back-btn" disabled={loadingModels || saving} onclick={onBack}>← Back</button>
    <button class="skip-link" disabled={loadingModels || saving} onclick={onSkip}>Skip</button>
    <button class="next-btn" disabled={loadingModels || saving} onclick={handleNext}>
      {saving ? "Saving..." : "Next →"}
    </button>
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
    margin: 0 0 0.45rem;
  }

  .explanation {
    color: #888;
    font-size: 0.82rem;
    line-height: 1.45;
    margin: 0 0 1rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin-bottom: 0.75rem;
  }

  .field span {
    color: #777;
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  input,
  select {
    background: #111;
    border: 1px solid #333;
    border-radius: 6px;
    color: #d9d4c7;
    font-family: inherit;
    font-size: 0.8rem;
    min-height: 34px;
    padding: 0.35rem 0.55rem;
  }

  input:disabled,
  select:disabled {
    color: #666;
    opacity: 0.7;
  }

  input:focus,
  select:focus {
    border-color: #3b82f6;
    outline: none;
  }

  .model-row {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 0.6rem;
    align-items: end;
  }

  .model-field {
    margin-bottom: 0;
  }

  .fetch-btn {
    background: #222;
    border: 1px solid #3a3a3a;
    border-radius: 6px;
    color: #d9d4c7;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.78rem;
    min-height: 34px;
    padding: 0.35rem 0.7rem;
    white-space: nowrap;
  }

  .fetch-btn:hover:not(:disabled) {
    border-color: #555;
  }

  .inline-error {
    background: #ef444418;
    border: 1px solid #ef444440;
    border-radius: 5px;
    color: #fca5a5;
    font-size: 0.74rem;
    line-height: 1.35;
    margin-top: 0.8rem;
    padding: 0.45rem 0.55rem;
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

  .back-btn,
  .skip-link {
    background: none;
    color: #888;
    cursor: pointer;
    font-size: 0.8rem;
    padding: 0.35rem 0.7rem;
    border-radius: 5px;
  }

  .back-btn {
    border: 1px solid #333;
  }

  .skip-link {
    border: none;
    color: #555;
  }

  .back-btn:hover:not(:disabled) {
    border-color: #555;
    color: #d9d4c7;
  }

  .skip-link:hover:not(:disabled) {
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

  button:disabled {
    cursor: not-allowed;
    opacity: 0.45;
  }
</style>
