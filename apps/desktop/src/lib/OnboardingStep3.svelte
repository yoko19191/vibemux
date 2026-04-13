<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { presetThemes } from "./presetThemes";
  import type { ThemePreset } from "./presetThemes";

  interface Props {
    onNext: () => void;
    onBack: () => void;
    onSkip: () => void;
  }

  let { onNext, onBack, onSkip }: Props = $props();

  let selectedTheme = $state<ThemePreset>(presetThemes[0]);
  let fontFamily = $state("Menlo, Monaco, 'Courier New', monospace");
  let fontSize = $state(14);

  const fontOptions = [
    { value: "Menlo, Monaco, 'Courier New', monospace", label: "Menlo (default)" },
    { value: "'JetBrains Mono', monospace", label: "JetBrains Mono" },
    { value: "'Fira Code', monospace", label: "Fira Code" },
    { value: "monospace", label: "System Monospace" },
  ];

  // Sample text lines for preview
  const sampleLines = [
    { text: "$ ls -la", color: "foreground" },
    { text: "total 48", color: "white" },
    { text: "drwxr-xr-x  8 user  staff   256 Apr 13 21:00 .", color: "blue" },
    { text: "-rw-r--r--  1 user  staff  1234 Apr 13 20:00 README.md", color: "green" },
    { text: "error: file not found", color: "red" },
  ];

  async function handleNext() {
    const themeUpdate: Record<string, string> = {
      background: selectedTheme.background,
      foreground: selectedTheme.foreground,
      cursor: selectedTheme.cursor,
      selection: selectedTheme.selection,
      black: selectedTheme.black,
      red: selectedTheme.red,
      green: selectedTheme.green,
      yellow: selectedTheme.yellow,
      blue: selectedTheme.blue,
      magenta: selectedTheme.magenta,
      cyan: selectedTheme.cyan,
      white: selectedTheme.white,
      bright_black: selectedTheme.bright_black,
      bright_red: selectedTheme.bright_red,
      bright_green: selectedTheme.bright_green,
      bright_yellow: selectedTheme.bright_yellow,
      bright_blue: selectedTheme.bright_blue,
      bright_magenta: selectedTheme.bright_magenta,
      bright_cyan: selectedTheme.bright_cyan,
      bright_white: selectedTheme.bright_white,
    };
    await invoke("config_update", {
      update: {
        theme: themeUpdate,
        terminal: { font_family: fontFamily, font_size: fontSize },
      },
    }).catch(() => {});
    onNext();
  }
</script>

<div class="step-content">
  <h2 class="title">Customize your terminal</h2>

  <div class="two-col">
    <div class="themes-col">
      <div class="col-label">Theme</div>
      <div class="theme-cards">
        {#each presetThemes as theme}
          <button
            class="theme-card"
            class:selected={selectedTheme.name === theme.name}
            onclick={() => (selectedTheme = theme)}
            style="background: {theme.background}; border-color: {selectedTheme.name === theme.name ? '#3b82f6' : '#333'};"
          >
            <span class="theme-name" style="color: {theme.foreground};">{theme.name}</span>
            <div class="color-dots">
              {#each [theme.red, theme.green, theme.yellow, theme.blue, theme.magenta, theme.cyan] as c}
                <span class="cdot" style="background: {c};"></span>
              {/each}
            </div>
          </button>
        {/each}
      </div>
    </div>

    <div class="font-col">
      <div class="col-label">Font</div>
      <select class="font-select" bind:value={fontFamily}>
        {#each fontOptions as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>

      <div class="col-label" style="margin-top: 0.75rem;">Size: {fontSize}px</div>
      <input
        type="range"
        min="10"
        max="20"
        step="1"
        bind:value={fontSize}
        class="font-slider"
      />
    </div>
  </div>

    <div class="preview" style="background: {selectedTheme.background}; font-family: {fontFamily}; font-size: {fontSize}px;">
      {#each sampleLines as line}
        {@const themeAsRecord = selectedTheme as unknown as Record<string, string>}
        <div style="color: {themeAsRecord[line.color] ?? selectedTheme.foreground};">{line.text}</div>
      {/each}
    </div>
</div>

<div class="footer">
  <div class="step-dots">
    <span class="dot"></span>
    <span class="dot"></span>
    <span class="dot active"></span>
    <span class="dot"></span>
  </div>
  <span class="step-label">Step 3 of 4</span>
  <div class="footer-actions">
    <button class="back-btn" onclick={onBack}>← Back</button>
    <button class="skip-link" onclick={onSkip}>Skip</button>
    <button class="next-btn" onclick={handleNext}>Next →</button>
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

  .two-col {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.25rem;
    margin-bottom: 1rem;
  }

  .col-label {
    color: #555;
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    margin-bottom: 0.4rem;
    font-family: system-ui, -apple-system, sans-serif;
  }

  .theme-cards {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .theme-card {
    border: 2px solid #333;
    border-radius: 6px;
    padding: 0.5rem 0.75rem;
    cursor: pointer;
    text-align: left;
    display: flex;
    align-items: center;
    justify-content: space-between;
    transition: border-color 0.15s;
  }

  .theme-name {
    font-size: 0.8rem;
    font-weight: 500;
    font-family: system-ui, -apple-system, sans-serif;
  }

  .color-dots {
    display: flex;
    gap: 3px;
  }

  .cdot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .font-select {
    width: 100%;
    background: #222;
    border: 1px solid #333;
    border-radius: 5px;
    color: #d9d4c7;
    font-size: 0.8rem;
    padding: 0.4rem 0.5rem;
    font-family: system-ui, -apple-system, sans-serif;
  }

  .font-slider {
    width: 100%;
    accent-color: #3b82f6;
  }

  .preview {
    border-radius: 6px;
    padding: 0.75rem 1rem;
    line-height: 1.5;
    min-height: 80px;
    border: 1px solid #2a2a2a;
    overflow: hidden;
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
