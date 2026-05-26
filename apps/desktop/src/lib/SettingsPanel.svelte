<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { presetThemes, type ThemePreset } from "./presetThemes.js";
  import { detectDesktopPlatform, isMacOS } from "./platform";
  import type {
    AiConfig,
    LayoutConfig,
    SessionCapabilities,
    SessionProfile,
    SessionProfileKind,
    TerminalConfig,
    ThemeConfig,
    UserConfig,
    SshConfigHost,
  } from "./types";

  interface Props {
    onClose?: () => void;
    onConfigChange?: (config: UserConfig) => void;
    initialTab?: "terminal" | "theme" | "layout" | "keys" | "profiles" | "privacy" | "ai";
  }

  let { onClose, onConfigChange, initialTab = "terminal" }: Props = $props();

  let config: UserConfig | null = $state(null);
  let activeTab: "terminal" | "theme" | "layout" | "keys" | "profiles" | "privacy" | "ai" = $state("terminal");
  let saving = $state(false);
  let systemFonts: string[] = $state([]);
  let capabilities: SessionCapabilities | null = $state(null);
  let aiModels: string[] = $state([]);
  let aiModelsLoading = $state(false);
  let aiModelsError: string | null = $state(null);
  const platform = detectDesktopPlatform();
  const isMac = isMacOS(platform);

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
  let initialTabApplied = false;

  $effect(() => {
    if (!initialTabApplied) {
      activeTab = initialTab === "privacy" && !isMac ? "terminal" : initialTab;
      initialTabApplied = true;
    }
  });

  $effect(() => {
    if (activeTab === "privacy" && !isMac) {
      activeTab = "terminal";
    }
  });

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
      capabilities = await invoke<SessionCapabilities>("detect_session_capabilities");
    } catch (e) {
      console.error("Failed to load config:", e);
    }
  }

  async function loadAiModels() {
    if (!config?.ai?.base_url || !config?.ai?.api_key) {
      aiModelsError = "Add a Base URL and API Key first.";
      return;
    }
    aiModelsLoading = true;
    aiModelsError = null;
    try {
      aiModels = await invoke<string[]>("ai_list_models");
    } catch (e) {
      aiModelsError = String(e);
    } finally {
      aiModelsLoading = false;
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

  function handleAiChange(field: keyof AiConfig, value: string | boolean) {
    applyUpdate({ ai: { [field]: value } });
  }

  function parseArgs(value: string) {
    return value
      .split(" ")
      .map((part) => part.trim())
      .filter(Boolean);
  }

  function createProfile(kind: SessionProfileKind) {
    const shell = config?.shell.default || capabilities?.shells[0] || "/bin/zsh";
    const profile: SessionProfile = {
      id: `${kind}-${Date.now()}`,
      name: kind === "local_shell" ? "Local Shell" : kind === "wsl" ? "WSL" : kind === "ssh" ? "SSH" : "Command",
      kind,
      cwd: kind === "ssh" ? undefined : undefined,
      shell: kind === "local_shell" ? shell : kind === "wsl" ? "bash" : undefined,
      program: kind === "command" ? "python3" : undefined,
      args: [],
      distro: kind === "wsl" ? capabilities?.wslDistros[0] : undefined,
      host: kind === "ssh" ? "" : undefined,
      ssh_config_host: undefined,
      user: undefined,
      port: kind === "ssh" ? 22 : undefined,
      identity_file: undefined,
      remote_cwd: undefined,
    };
    const items = [...(config?.profiles.items ?? []), profile];
    applyUpdate({
      profiles: {
        ...config?.profiles,
        default_profile_id: config?.profiles.default_profile_id ?? profile.id,
        items,
      },
    });
  }

  function updateProfile(profile: SessionProfile, patch: Partial<SessionProfile>) {
    if (!config) return;
    const items = config.profiles.items.map((item) =>
      item.id === profile.id ? { ...item, ...patch } : item,
    );
    applyUpdate({ profiles: { ...config.profiles, items } });
  }

  function deleteProfile(profile: SessionProfile) {
    if (!config) return;
    const items = config.profiles.items.filter((item) => item.id !== profile.id);
    const default_profile_id =
      config.profiles.default_profile_id === profile.id
        ? items[0]?.id ?? null
        : config.profiles.default_profile_id;
    const last_used_profile_id =
      config.profiles.last_used_profile_id === profile.id
        ? default_profile_id
        : config.profiles.last_used_profile_id;
    applyUpdate({ profiles: { ...config.profiles, default_profile_id, last_used_profile_id, items } });
  }

  function setDefaultProfile(profileId: string) {
    if (!config) return;
    applyUpdate({ profiles: { ...config.profiles, default_profile_id: profileId } });
  }

  function profileKindLabel(kind: SessionProfileKind) {
    if (kind === "local_shell") return "Local";
    return kind.toUpperCase();
  }

  function sshConfigLabel(host: SshConfigHost) {
    const target = host.user ? `${host.user}@${host.alias}` : host.alias;
    return host.hostname ? `${target} -> ${host.hostname}` : target;
  }

  function applySshConfigHost(profile: SessionProfile, alias: string) {
    if (!alias) {
      updateProfile(profile, { ssh_config_host: null });
      return;
    }
    const host = capabilities?.sshConfigHosts.find((candidate) => candidate.alias === alias);
    if (!host) return;
    updateProfile(profile, {
      host: host.alias,
      ssh_config_host: host.alias,
      user: null,
      port: null,
      identity_file: null,
    });
  }

  async function openMacPrivacyPane() {
    try {
      await invoke("open_url", {
        url: "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles",
      });
    } catch (e) {
      console.error("Failed to open macOS privacy settings:", e);
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
      <button class="tab" class:active={activeTab === "profiles"} onclick={() => (activeTab = "profiles")}>Profiles</button>
      {#if isMac}
        <button class="tab" class:active={activeTab === "privacy"} onclick={() => (activeTab = "privacy")}>Privacy</button>
      {/if}
      <button class="tab" class:active={activeTab === "ai"} onclick={() => (activeTab = "ai")}>AI</button>
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
          <label class="field">
            <span>Alternate Scroll Mode</span>
            <select
              value={config.terminal.alternate_scroll_mode ?? "off"}
              onchange={(e) => handleTerminalChange("alternate_scroll_mode", (e.target as HTMLSelectElement).value)}
            >
              <option value="off">Off — wheel does nothing in TUIs (claude, codex)</option>
              <option value="arrows">Arrow keys — wheel scrolls less / man / vim</option>
            </select>
          </label>
        </div>
      {:else if activeTab === "theme"}
        <div class="section">
          <div class="preset-themes-grid">
            {#each presetThemes as preset}
              {@const isSelected = config.theme.background === preset.background && config.theme.foreground === preset.foreground}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="preset-card"
                class:selected={isSelected}
                onclick={() => applyUpdate({ theme: preset })}
                title={preset.name}
              >
                <div class="preset-swatches">
                  <div class="swatch" style="background: {preset.background}"></div>
                  <div class="swatch" style="background: {preset.foreground}"></div>
                  <div class="swatch" style="background: {preset.cursor}"></div>
                </div>
                <span class="preset-name">{preset.name}</span>
              </div>
            {/each}
          </div>
          <div class="section-divider"></div>
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
      {:else if activeTab === "profiles"}
        <div class="section">
          <div class="field">
            <span>Default Profile</span>
            <select
              value={config.profiles.default_profile_id ?? ""}
              onchange={(e) => setDefaultProfile((e.target as HTMLSelectElement).value)}
            >
              {#each config.profiles.items as profile}
                <option value={profile.id}>{profile.name} · {profileKindLabel(profile.kind)}</option>
              {/each}
            </select>
          </div>

          <div class="profile-actions">
            <button class="secondary-btn" onclick={() => createProfile("local_shell")}>Add Local</button>
            {#if capabilities?.platform === "windows"}
              <button class="secondary-btn" onclick={() => createProfile("wsl")}>Add WSL</button>
            {/if}
            <button class="secondary-btn" onclick={() => createProfile("ssh")}>Add SSH</button>
            <button class="secondary-btn" onclick={() => createProfile("command")}>Add Command</button>
          </div>

          <div class="profiles-list">
            {#each config.profiles.items as profile (profile.id)}
              <div class="profile-row">
                <div class="profile-row-header">
                  <span
                    class:kind-local={profile.kind === "local_shell"}
                    class:kind-ssh={profile.kind === "ssh"}
                    class:kind-command={profile.kind === "command"}
                    class:kind-wsl={profile.kind === "wsl"}
                    class="profile-kind"
                  >
                    {profileKindLabel(profile.kind)}
                  </span>
                  {#if config.profiles.default_profile_id === profile.id}
                    <span class="profile-default">Default</span>
                  {:else}
                    <button class="link-btn" onclick={() => setDefaultProfile(profile.id)}>Make Default</button>
                  {/if}
                  <button class="link-btn danger" onclick={() => deleteProfile(profile)}>Delete</button>
                </div>

                <label class="field">
                  <span>Name</span>
                  <input
                    type="text"
                    value={profile.name}
                    onchange={(e) => updateProfile(profile, { name: (e.target as HTMLInputElement).value })}
                  />
                </label>

                {#if profile.kind !== "ssh"}
                  <label class="field">
                    <span>Local Working Directory</span>
                    <input
                      type="text"
                      value={profile.cwd ?? ""}
                      placeholder="Use current directory"
                      onchange={(e) => updateProfile(profile, { cwd: (e.target as HTMLInputElement).value || null })}
                    />
                  </label>
                {/if}

                {#if profile.kind === "local_shell"}
                  <label class="field">
                    <span>Shell</span>
                    <select
                      value={profile.shell ?? config.shell.default}
                      onchange={(e) => updateProfile(profile, { shell: (e.target as HTMLSelectElement).value })}
                    >
                      {#each capabilities?.shells ?? [] as shell}
                        <option value={shell}>{shell}</option>
                      {/each}
                      {#if profile.shell && !(capabilities?.shells ?? []).includes(profile.shell)}
                        <option value={profile.shell}>{profile.shell}</option>
                      {/if}
                    </select>
                  </label>
                {:else if profile.kind === "wsl"}
                  <div class="field-grid">
                    <label class="field">
                      <span>Distro</span>
                      <input
                        type="text"
                        value={profile.distro ?? ""}
                        placeholder="Ubuntu"
                        onchange={(e) => updateProfile(profile, { distro: (e.target as HTMLInputElement).value || null })}
                      />
                    </label>
                    <label class="field">
                      <span>Shell</span>
                      <input
                        type="text"
                        value={profile.shell ?? ""}
                        placeholder="bash"
                        onchange={(e) => updateProfile(profile, { shell: (e.target as HTMLInputElement).value || null })}
                      />
                    </label>
                  </div>
                  <label class="field">
                    <span>Linux Working Directory</span>
                    <input
                      type="text"
                      value={profile.remote_cwd ?? ""}
                      placeholder="~"
                      onchange={(e) => updateProfile(profile, { remote_cwd: (e.target as HTMLInputElement).value || null })}
                    />
                  </label>
                {:else if profile.kind === "ssh"}
                  {#if (capabilities?.sshConfigHosts ?? []).length > 0}
                    <label class="field">
                      <span>SSH Config</span>
                      <select
                        value={profile.ssh_config_host ?? ""}
                        onchange={(e) => applySshConfigHost(profile, (e.target as HTMLSelectElement).value)}
                      >
                        <option value="">Manual SSH fields</option>
                        {#each capabilities?.sshConfigHosts ?? [] as host}
                          <option value={host.alias}>{sshConfigLabel(host)}</option>
                        {/each}
                      </select>
                    </label>
                  {/if}
                  {#if !profile.ssh_config_host}
                    <div class="profile-field-row">
                      <label class="field">
                        <span>Host</span>
                        <input
                          type="text"
                          value={profile.host ?? ""}
                          placeholder="example.com"
                          onchange={(e) => updateProfile(profile, { host: (e.target as HTMLInputElement).value || null })}
                        />
                      </label>
                      <label class="field">
                        <span>User</span>
                        <input
                          type="text"
                          value={profile.user ?? ""}
                          placeholder="deploy"
                          onchange={(e) => updateProfile(profile, { user: (e.target as HTMLInputElement).value || null })}
                        />
                      </label>
                    </div>
                    <div class="profile-field-row">
                      <label class="field">
                        <span>Port</span>
                        <input
                          type="number"
                          min="1"
                          max="65535"
                          value={profile.port ?? 22}
                          onchange={(e) => updateProfile(profile, { port: parseInt((e.target as HTMLInputElement).value) || null })}
                        />
                      </label>
                      <label class="field">
                        <span>Identity File</span>
                        <input
                          type="text"
                          value={profile.identity_file ?? ""}
                          placeholder="~/.ssh/id_ed25519"
                          onchange={(e) => updateProfile(profile, { identity_file: (e.target as HTMLInputElement).value || null })}
                        />
                      </label>
                    </div>
                  {/if}
                  <label class="field">
                    <span>Remote Directory</span>
                    <input
                      type="text"
                      value={profile.remote_cwd ?? ""}
                      placeholder="~/app"
                      onchange={(e) => updateProfile(profile, { remote_cwd: (e.target as HTMLInputElement).value || null })}
                    />
                  </label>
                {:else}
                  <label class="field">
                    <span>Program</span>
                    <input
                      type="text"
                      value={profile.program ?? ""}
                      placeholder="python3"
                      onchange={(e) => updateProfile(profile, { program: (e.target as HTMLInputElement).value || null })}
                    />
                  </label>
                  <label class="field">
                    <span>Arguments</span>
                    <input
                      type="text"
                      value={profile.args.join(" ")}
                      placeholder="-m http.server 8000"
                      onchange={(e) => updateProfile(profile, { args: parseArgs((e.target as HTMLInputElement).value) })}
                    />
                  </label>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {:else if activeTab === "privacy"}
        <div class="section">
          <div class="privacy-callout">
            <div class="privacy-title">macOS Full Disk Access</div>
            <p>
              Terminal commands run inside Vibemux are attributed to Vibemux.app by macOS.
              Granting Full Disk Access once can prevent repeated prompts when tools scan protected folders.
            </p>
          </div>
          <button class="primary-btn" onclick={openMacPrivacyPane}>Open Full Disk Access</button>
          <div class="privacy-note">
            Add Vibemux.app in System Settings, then restart Vibemux so new shells inherit the permission.
            macOS still requires you to approve this manually.
          </div>
        </div>
      {:else if activeTab === "ai"}
        <div class="section">
          <label class="field">
            <span>Enable AI</span>
            <input
              class="toggle"
              type="checkbox"
              checked={config.ai.enabled}
              onchange={(e) => handleAiChange("enabled", (e.target as HTMLInputElement).checked)}
            />
          </label>
          <label class="field">
            <span>Base URL</span>
            <input
              type="text"
              placeholder="https://api.openai.com"
              value={config.ai.base_url}
              onchange={(e) => handleAiChange("base_url", (e.target as HTMLInputElement).value)}
            />
          </label>
          <label class="field">
            <span>API Key</span>
            <input
              type="password"
              placeholder="sk-..."
              value={config.ai.api_key}
              onchange={(e) => handleAiChange("api_key", (e.target as HTMLInputElement).value)}
            />
          </label>
          <div class="field">
            <span>Model</span>
            <div class="model-field">
              <select
                value={config.ai.model}
                onchange={(e) => handleAiChange("model", (e.target as HTMLSelectElement).value)}
              >
                <option value="">Select a model</option>
                {#each aiModels as model}
                  <option value={model} selected={model === config.ai.model}>{model}</option>
                {/each}
                {#if config.ai.model && !aiModels.includes(config.ai.model)}
                  <option value={config.ai.model} selected>{config.ai.model}</option>
                {/if}
              </select>
              <button class="secondary-btn" disabled={aiModelsLoading} onclick={loadAiModels}>
                {aiModelsLoading ? "Loading..." : "Refresh models"}
              </button>
            </div>
          </div>
          {#if aiModelsError}
            <div class="inline-error">{aiModelsError}</div>
          {/if}
          <label class="field prompt-field">
            <span>System Prompt</span>
            <textarea
              value={config.ai.system_prompt}
              rows="6"
              onchange={(e) => handleAiChange("system_prompt", (e.target as HTMLTextAreaElement).value)}
            ></textarea>
          </label>
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
    width: min(760px, calc(100vw - 48px));
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
    padding: 1rem 1.35rem 1.2rem;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    min-width: 0;
  }

  .field span {
    color: #a7a3a0;
    flex-shrink: 0;
    font-size: 0.78rem;
    font-weight: 600;
    line-height: 1.25;
    min-width: 140px;
  }

  .field input[type="text"],
  .field input[type="password"],
  .field input[type="number"],
  .field select,
  .field textarea {
    flex: 1;
    min-width: 0;
    width: 100%;
    box-sizing: border-box;
    background: #111;
    border: 1px solid #333;
    border-radius: 4px;
    color: #d9d4c7;
    font-size: 0.84rem;
    min-height: 2.25rem;
    line-height: 1.25;
    padding: 0.42rem 0.62rem;
    font-family: inherit;
  }

  .field textarea {
    min-height: 90px;
    resize: vertical;
    line-height: 1.35;
  }

  .prompt-field {
    align-items: flex-start;
  }

  .toggle {
    width: 34px;
    height: 18px;
    accent-color: #3b82f6;
  }

  .font-field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    flex: 1;
  }

  .model-field {
    display: flex;
    flex: 1;
    gap: 0.4rem;
  }

  .model-field select {
    min-width: 0;
  }

  .secondary-btn {
    background: #222;
    border: 1px solid #3a3a3a;
    border-radius: 4px;
    color: #d9d4c7;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.78rem;
    padding: 0.38rem 0.65rem;
    white-space: nowrap;
  }

  .secondary-btn:disabled {
    color: #666;
    cursor: default;
  }

  .secondary-btn:not(:disabled):hover {
    border-color: #555;
  }

  .profile-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .profiles-list {
    display: flex;
    flex-direction: column;
    gap: 1.15rem;
  }

  .profile-row {
    background: #111;
    border: 1px solid #303036;
    border-radius: 7px;
    display: flex;
    flex-direction: column;
    gap: 0.95rem;
    min-width: 0;
    overflow: hidden;
    padding: 1.15rem 1.25rem;
  }

  .profile-row-header {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    margin-bottom: 0.1rem;
  }

  .profile-row .field {
    align-items: stretch;
    flex: 0 0 auto;
    flex-direction: column;
    gap: 0.45rem;
  }

  .profile-row .field span {
    color: #a7a3a0;
    font-size: 0.82rem;
    line-height: 1.25;
    min-width: 0;
  }

  .profile-kind,
  .profile-default {
    border: 1px solid #3a3a3a;
    border-radius: 4px;
    color: #d9d4c7;
    font-size: 0.66rem;
    padding: 0.15rem 0.35rem;
  }

  .kind-local {
    border-color: rgba(34, 197, 94, 0.65);
    color: #86efac;
  }

  .kind-ssh {
    border-color: rgba(59, 130, 246, 0.7);
    color: #93c5fd;
  }

  .kind-command {
    border-color: rgba(234, 179, 8, 0.72);
    color: #fde68a;
  }

  .kind-wsl {
    border-color: rgba(6, 182, 212, 0.7);
    color: #67e8f9;
  }

  .profile-default {
    border-color: #2563eb;
    color: #93c5fd;
  }

  .link-btn {
    background: none;
    border: none;
    color: #93c5fd;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.7rem;
    padding: 0.1rem 0.25rem;
  }

  .link-btn.danger {
    color: #f87171;
    margin-left: auto;
  }

  .field-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.85rem;
  }

  .profile-field-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.85rem;
  }

  .field-grid .field {
    flex: 1 1 220px;
    min-width: 180px;
  }

  .profile-field-row .field {
    flex: 1 1 220px;
    min-width: 180px;
  }

  .primary-btn {
    align-self: flex-start;
    background: #2563eb;
    border: 1px solid #3b82f6;
    border-radius: 4px;
    color: #fff;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.75rem;
    padding: 0.35rem 0.65rem;
  }

  .primary-btn:hover {
    background: #1d4ed8;
  }

  .privacy-callout {
    background: #111;
    border: 1px solid #333;
    border-radius: 6px;
    padding: 0.65rem 0.75rem;
  }

  .privacy-title {
    color: #d9d4c7;
    font-size: 0.78rem;
    font-weight: 600;
    margin-bottom: 0.35rem;
  }

  .privacy-callout p,
  .privacy-note {
    color: #999;
    font-size: 0.72rem;
    line-height: 1.4;
    margin: 0;
  }

  .privacy-note {
    color: #777;
  }

  .inline-error {
    background: #ef444418;
    border: 1px solid #ef444440;
    border-radius: 5px;
    color: #fca5a5;
    font-size: 0.72rem;
    line-height: 1.35;
    padding: 0.45rem 0.55rem;
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

  .preset-themes-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.5rem;
    margin-bottom: 0.25rem;
  }

  .preset-card {
    background: #111;
    border: 1px solid #333;
    border-radius: 5px;
    padding: 0.4rem;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    transition: border-color 100ms;
  }

  .preset-card:hover { border-color: #555; }

  .preset-card.selected {
    border-color: #3b82f6;
    box-shadow: 0 0 0 1px #3b82f640;
  }

  .preset-swatches {
    display: flex;
    gap: 2px;
    height: 18px;
    border-radius: 3px;
    overflow: hidden;
  }

  .swatch {
    flex: 1;
  }

  .preset-name {
    font-size: 0.6rem;
    color: #999;
    text-align: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .section-divider {
    height: 1px;
    background: #2a2a2a;
    margin: 0.25rem 0;
  }
</style>
