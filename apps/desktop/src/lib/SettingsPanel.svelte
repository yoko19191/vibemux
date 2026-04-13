<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface TerminalConfig {
    font_family: string;
    font_size: number;
    line_height: number;
  }

  interface ThemeConfig {
    background: string;
    foreground: string;
    cursor: string;
    selection: string;
    black: string; red: string; green: string; yellow: string;
    blue: string; magenta: string; cyan: string; white: string;
    bright_black: string; bright_red: string; bright_green: string; bright_yellow: string;
    bright_blue: string; bright_magenta: string; bright_cyan: string; bright_white: string;
  }

  interface LayoutConfig {
    focused_pane_width: number;
    animation_ms: number;
    max_hot_sessions: number;
  }

  interface KeysConfig {
    prefix: string;
  }

  interface UserConfig {
    terminal: TerminalConfig;
    theme: ThemeConfig;
    layout: LayoutConfig;
    keys: KeysConfig;
  }

  interface Props {
    onClose?: () => void;
    onConfigChange?: (config: UserConfig) => void;
  }

  let { onClose, onConfigChange }: Props = $props();

  let config: UserConfig | null = $state(null);
  let activeTab: "terminal" | "theme" | "layout" | "keys" = $state("terminal");
  let saving = $state(false);
  let systemFonts: string[] = $state([]);

  const PRESET_PREFIX_KEYS = [
    { label: "Ctrl+B (tmux style)", value: "ctrl+b" },
    { label: "Ctrl+Space (Spacemacs style)", value: "ctrl+space" },
    { label: "Ctrl+` (Backtick)", value: "ctrl+`" },
    { label: "Ctrl+A (screen style)", value: "ctrl+a" },
    { label: "Cmd+Space (macOS)", value: "cmd+space" },
    { label: "Custom…", value: "__custom__" },
  ];

  let prefixDropdownValue = $state("ctrl+b");
  let customPrefixValue = $state("");
  let showCustomInput = $derived(prefixDropdownValue === "__custom__");

  $effect(() => {
    if (config?.keys?.prefix) {
      const preset = PRESET_PREFIX_KEYS.find((p) => p.value === config!.keys.prefix && p.value !== "__custom__");
      if (preset) {
        prefixDropdownValue = preset.value;
        customPrefixValue = "";
      } else {
        prefixDropdownValue = "__custom__";
        customPrefixValue = config.keys.prefix;
      }
    }
  });

  async function loadConfig() {
    try {
      config = await invoke<UserConfig>("config_get");
    } catch (e) {
      console.error("Failed to load config:", e);
    }
  }

  async function loadFonts() {
    try {
      systemFonts = await invoke<string[]>("list_monospace_fonts");
    } catch (e) {
      console.error("Failed to load fonts:", e);
      systemFonts = ["monospace", "Menlo", "Monaco", "Courier New", "JetBrains Mono", "Fira Code"];
    }
  }

  async function applyUpdate(partial: object) {
    if (saving) return;
    saving = true;
    try {
      const updated = await invoke<UserConfig>("config_update", { update: partial });
      config = updated;
      onConfigChange?.(updated);
    } catch (e) {
      console.error("Failed to update config:", e);
    } finally {
      saving = false;
    }
  }

  function handleTerminalChange(field: keyof TerminalConfig, value: string | number) {
    applyUpdate({ terminal: { [field]: value } });
  }

  function handleThemeChange(field: keyof ThemeConfig, value: string) {
    applyUpdate({ theme: { [field]: value } });
  }

  function handleLayoutChange(field: keyof LayoutConfig, value: number) {
    applyUpdate({ layout: { [field]: value } });
  }

  function handlePrefixDropdownChange(value: string) {
    prefixDropdownValue = value;
    if (value !== "__custom__") {
      applyUpdate({ keys: { prefix: value } });
    }
  }

  function handleCustomPrefixBlur() {
    const val = customPrefixValue.trim();
    if (val) {
      applyUpdate({ keys: { prefix: val } });
    }
  }

  loadConfig();
  loadFonts();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}>
  <div class="panel" onclick={(e) => e.stopPropagation()}>
    <div class="panel-header">
      <span class="panel-title">Settings</span>
      <button class="close-btn" onclick={onClose}>✕</button>
    </div>

    <div class="tabs">
      <button class="tab" class:active={activeTab === "terminal"} onclick={() => (activeTab = "terminal")}>Terminal</button>
      <button class="tab" class:active={activeTab === "theme"} onclick={() => (activeTab = "theme")}>Theme</button>
      <button class="tab" class:active={activeTab === "layout"} onclick={() => (activeTab = "layout")}>Layout</button>
      <button class="tab" class:active={activeTab === "keys"} onclick={() => (activeTab = "keys")}>Keys</button>
    </div>

    {#if config}
      {#if activeTab === "terminal"}
        <div class="section">
          <div class="field">
            <span>Font Family</span>
            <div class="font-field">
              <select
                value={config.terminal.font_family}
                onchange={(e) => handleTerminalChange("font_family", (e.target as HTMLSelectElement).value)}
              >
                {#each systemFonts as font}
                  <option value={font} selected={font === config.terminal.font_family}>{font}</option>
                {/each}
                {#if !systemFonts.includes(config.terminal.font_family)}
                  <option value={config.terminal.font_family} selected>{config.terminal.font_family}</option>
                {/if}
              </select>
              <input
                type="text"
                placeholder="or type a font name"
                value={config.terminal.font_family}
                onchange={(e) => handleTerminalChange("font_family", (e.target as HTMLInputElement).value)}
              />
            </div>
          </div>
          <label class="field">
            <span>Font Size</span>
            <input
              type="number"
              min="8" max="32"
              value={config.terminal.font_size}
              onchange={(e) => handleTerminalChange("font_size", parseInt((e.target as HTMLInputElement).value))}
            />
          </label>
          <label class="field">
            <span>Line Height</span>
            <input
              type="number"
              min="1" max="2" step="0.05"
              value={config.terminal.line_height}
              onchange={(e) => handleTerminalChange("line_height", parseFloat((e.target as HTMLInputElement).value))}
            />
          </label>
        </div>
      {:else if activeTab === "theme"}
        <div class="section">
          {#each [
            ["background", "Background"],
            ["foreground", "Foreground"],
            ["cursor", "Cursor"],
            ["selection", "Selection"],
            ["black", "Black"], ["red", "Red"], ["green", "Green"], ["yellow", "Yellow"],
            ["blue", "Blue"], ["magenta", "Magenta"], ["cyan", "Cyan"], ["white", "White"],
            ["bright_black", "Bright Black"], ["bright_red", "Bright Red"],
            ["bright_green", "Bright Green"], ["bright_yellow", "Bright Yellow"],
            ["bright_blue", "Bright Blue"], ["bright_magenta", "Bright Magenta"],
            ["bright_cyan", "Bright Cyan"], ["bright_white", "Bright White"],
          ] as [field, label]}
            <label class="field color-field">
              <span>{label}</span>
              <div class="color-row">
                <input
                  type="color"
                  value={config.theme[field as keyof ThemeConfig].slice(0, 7)}
                  oninput={(e) => handleThemeChange(field as keyof ThemeConfig, (e.target as HTMLInputElement).value)}
                />
                <input
                  type="text"
                  value={config.theme[field as keyof ThemeConfig]}
                  onchange={(e) => handleThemeChange(field as keyof ThemeConfig, (e.target as HTMLInputElement).value)}
                />
              </div>
            </label>
          {/each}
        </div>
      {:else if activeTab === "layout"}
        <div class="section">
          <label class="field">
            <span>Focused Pane Width (0–1)</span>
            <input
              type="number"
              min="0.3" max="0.9" step="0.05"
              value={config.layout.focused_pane_width}
              onchange={(e) => handleLayoutChange("focused_pane_width", parseFloat((e.target as HTMLInputElement).value))}
            />
          </label>
          <label class="field">
            <span>Animation (ms)</span>
            <input
              type="number"
              min="0" max="500"
              value={config.layout.animation_ms}
              onchange={(e) => handleLayoutChange("animation_ms", parseInt((e.target as HTMLInputElement).value))}
            />
          </label>
          <label class="field">
            <span>Max Hot Sessions</span>
            <input
              type="number"
              min="1" max="10"
              value={config.layout.max_hot_sessions}
              onchange={(e) => handleLayoutChange("max_hot_sessions", parseInt((e.target as HTMLInputElement).value))}
            />
          </label>
        </div>
      {:else if activeTab === "keys"}
        <div class="section">
          <div class="field">
            <span>Navigation Prefix Key</span>
            <select
              value={prefixDropdownValue}
              onchange={(e) => handlePrefixDropdownChange((e.target as HTMLSelectElement).value)}
            >
              {#each PRESET_PREFIX_KEYS as opt}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
          </div>
          {#if showCustomInput}
            <div class="field">
              <span>Custom key combo</span>
              <input
                type="text"
                placeholder="e.g. ctrl+shift+x"
                bind:value={customPrefixValue}
                onblur={handleCustomPrefixBlur}
              />
            </div>
          {/if}
        </div>
      {/if}
    {:else}
      <div class="loading">Loading...</div>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    z-index: 60;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: system-ui, -apple-system, sans-serif;
  }

  .panel {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 8px;
    width: 480px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #2a2a2a;
  }

  .panel-title {
    font-size: 0.9rem;
    font-weight: 600;
    color: #d9d4c7;
  }

  .close-btn {
    background: none;
    border: none;
    color: #666;
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0.2rem 0.4rem;
  }

  .close-btn:hover { color: #d9d4c7; }

  .tabs {
    display: flex;
    border-bottom: 1px solid #2a2a2a;
  }

  .tab {
    flex: 1;
    background: none;
    border: none;
    color: #666;
    cursor: pointer;
    padding: 0.5rem;
    font-size: 0.75rem;
    font-family: inherit;
    border-bottom: 2px solid transparent;
    transition: color 100ms;
  }

  .tab.active {
    color: #d9d4c7;
    border-bottom-color: #3b82f6;
  }

  .section {
    overflow-y: auto;
    padding: 0.75rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .field span {
    font-size: 0.75rem;
    color: #999;
    flex-shrink: 0;
    min-width: 140px;
  }

  .field input[type="text"],
  .field input[type="number"],
  .field select {
    flex: 1;
    background: #111;
    border: 1px solid #333;
    border-radius: 4px;
    color: #d9d4c7;
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
    font-family: inherit;
  }

  .font-field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    flex: 1;
  }

  .font-field select,
  .font-field input[type="text"] {
    width: 100%;
    background: #111;
    border: 1px solid #333;
    border-radius: 4px;
    color: #d9d4c7;
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
    font-family: inherit;
  }

  .color-field .color-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex: 1;
  }

  .color-field input[type="color"] {
    width: 28px;
    height: 24px;
    border: 1px solid #333;
    border-radius: 3px;
    padding: 1px;
    background: #111;
    cursor: pointer;
    flex-shrink: 0;
  }

  .color-field input[type="text"] {
    flex: 1;
  }

  .loading {
    padding: 1rem;
    color: #666;
    font-size: 0.8rem;
    text-align: center;
  }
</style>
