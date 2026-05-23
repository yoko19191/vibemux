<script lang="ts">
  import { onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type {
    SessionCapabilities,
    SessionProfile,
    SessionProfileKind,
    SessionSnapshot,
    UserConfig,
  } from "./types";

  interface Props {
    defaultCwd: string;
    onCreated?: (snapshot: SessionSnapshot) => void;
    onCancel?: () => void;
  }

  let { defaultCwd, onCreated, onCancel }: Props = $props();

  let config = $state<UserConfig | null>(null);
  let capabilities = $state<SessionCapabilities | null>(null);
  let selectedProfileId = $state("");
  let sessionName = $state("shell");
  let errorMsg = $state<string | null>(null);
  let submitting = $state(false);
  let loading = $state(true);
  let profileMenuOpen = $state(false);
  let panelEl: HTMLFormElement;

  const focusableSelector = [
    'button:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
  ].join(",");

  const profiles = $derived(config?.profiles?.items ?? []);
  const profileOptions = $derived.by(() => {
    return profiles
      .filter((profile) => isKindVisible(profile.kind))
      .map((profile) => ({
        id: profile.id,
        profile,
        kind: profile.kind,
        name: profile.name,
        subtitle: profileSubtitle(profile),
        persisted: true,
      }));
  });
  const selectedOption = $derived(profileOptions.find((option) => option.id === selectedProfileId) ?? profileOptions[0] ?? null);
  const selectedProfile = $derived(
    selectedOption?.profile ?? null,
  );
  const activeKind = $derived(selectedOption?.kind ?? "local_shell");
  const sshDisabled = $derived(activeKind === "ssh" && capabilities !== null && !capabilities.sshAvailable);

  onMount(() => {
    void initialize();
    void focusInitialInput();
  });

  async function initialize() {
    loading = true;
    try {
      const [loadedConfig, loadedCapabilities] = await Promise.all([
        invoke<UserConfig>("config_get"),
        invoke<SessionCapabilities>("detect_session_capabilities"),
      ]);
      config = loadedConfig;
      capabilities = loadedCapabilities;

      const lastUsedProfile = loadedConfig.profiles.items.find(
        (profile) => profile.id === loadedConfig.profiles.last_used_profile_id,
      );
      const defaultProfile = loadedConfig.profiles.items.find(
        (profile) => profile.id === loadedConfig.profiles.default_profile_id,
      );
      const initialProfile =
        lastUsedProfile && isKindVisible(lastUsedProfile.kind, loadedCapabilities)
          ? lastUsedProfile
          : defaultProfile && isKindVisible(defaultProfile.kind, loadedCapabilities)
            ? defaultProfile
            : loadedConfig.profiles.items.find((profile) => isKindVisible(profile.kind, loadedCapabilities));

      if (initialProfile) selectProfile(initialProfile.id);
    } catch (e) {
      errorMsg = String(e);
    } finally {
      loading = false;
    }
  }

  async function focusInitialInput() {
    await tick();
    requestAnimationFrame(() => getFocusableElements()[0]?.focus());
  }

  function getFocusableElements() {
    return Array.from(panelEl?.querySelectorAll<HTMLElement>(focusableSelector) ?? [])
      .filter((el) => el.getClientRects().length > 0);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      if (profileMenuOpen) {
        profileMenuOpen = false;
      } else {
        onCancel?.();
      }
    }
  }

  function handlePanelKeydown(e: KeyboardEvent) {
    if (e.key !== "Tab") return;

    const focusable = getFocusableElements();
    if (focusable.length === 0) return;

    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    const active = document.activeElement;

    if (e.shiftKey && active === first) {
      e.preventDefault();
      last.focus();
      return;
    }

    if (!e.shiftKey && active === last) {
      e.preventDefault();
      first.focus();
    }
  }

  function selectProfile(value: string) {
    selectedProfileId = value;
    profileMenuOpen = false;
    const savedProfile = profiles.find((profile) => profile.id === value) ?? null;
    sessionName = savedProfile?.name ?? "shell";
  }

  function isKindVisible(kind: SessionProfileKind, caps: SessionCapabilities | null = capabilities) {
    return kind !== "wsl" || caps?.platform === "windows";
  }

  function profileSubtitle(profile: SessionProfile) {
    if (profile.kind === "local_shell") return profile.shell ?? "default shell";
    if (profile.kind === "wsl") return profile.distro ?? "WSL";
    if (profile.kind === "ssh") {
      const target = profile.user ? `${profile.user}@${profile.host}` : profile.host;
      return [target, profile.port ? `:${profile.port}` : ""].join("");
    }
    return [profile.program, ...(profile.args ?? [])].filter(Boolean).join(" ");
  }

  function badgeText(kind: SessionProfileKind) {
    if (kind === "local_shell") return "Local";
    return kind.toUpperCase();
  }

  async function handleSubmit() {
    errorMsg = null;
    submitting = true;

    try {
      const payload: Record<string, unknown> = {
        name: sessionName || selectedProfile?.name || "shell",
        cwd: selectedProfile?.cwd ?? defaultCwd,
      };

      if (!selectedProfile) {
        throw new Error("Select a saved profile first.");
      }
      payload.profileId = selectedProfileId;

      const snapshot: SessionSnapshot = await invoke("session_create", { payload });
      if (config) {
        await invoke<UserConfig>("config_update", {
          update: {
            profiles: {
              last_used_profile_id: selectedProfileId,
            },
          },
        });
      }
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
  <form
    bind:this={panelEl}
    class="panel"
    onclick={(e) => e.stopPropagation()}
    onkeydown={handlePanelKeydown}
    onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}
  >
    <div class="header">
      <h2>New Session</h2>
      <span>Saved Profile</span>
    </div>

    {#if loading}
      <div class="loading">Loading profiles...</div>
    {:else}
      <label>
        Session Name
        <input type="text" bind:value={sessionName} placeholder="session name" />
      </label>

      <div class="profile-picker">
        <span class="field-label">Profile</span>
        <button
          class="profile-trigger"
          type="button"
          aria-haspopup="listbox"
          aria-expanded={profileMenuOpen}
          onclick={() => (profileMenuOpen = !profileMenuOpen)}
        >
          {#if selectedOption}
            <span class:badge-local={selectedOption.kind === "local_shell"} class:badge-ssh={selectedOption.kind === "ssh"} class:badge-command={selectedOption.kind === "command"} class:badge-wsl={selectedOption.kind === "wsl"} class="kind-badge">
              {badgeText(selectedOption.kind)}
            </span>
            <span class="profile-title">{selectedOption.name}</span>
            <span class="profile-subtitle">{selectedOption.subtitle}</span>
          {:else}
            <span class="profile-title">No profile available</span>
          {/if}
          <span class="chevron">⌄</span>
        </button>

        {#if profileMenuOpen}
          <div class="profile-menu" role="listbox">
            {#each profileOptions as option}
              <button
                type="button"
                class:active={selectedProfileId === option.id}
                role="option"
                aria-selected={selectedProfileId === option.id}
                onclick={() => selectProfile(option.id)}
              >
                <span class:badge-local={option.kind === "local_shell"} class:badge-ssh={option.kind === "ssh"} class:badge-command={option.kind === "command"} class:badge-wsl={option.kind === "wsl"} class="kind-badge">
                  {badgeText(option.kind)}
                </span>
                <span class="profile-option-main">
                  <strong>{option.name}</strong>
                  <small>{option.subtitle}</small>
                </span>
                {#if option.persisted}
                  <span class="persisted-dot">Saved</span>
                {/if}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    {#if errorMsg}
      <div class="error">{errorMsg}</div>
    {/if}

    <div class="actions">
      <button type="button" class="cancel" onclick={onCancel}>Cancel</button>
      <button type="submit" class="submit" disabled={submitting || loading || sshDisabled || !selectedProfile}>
        {submitting ? "Creating..." : "Create"}
      </button>
    </div>
  </form>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.62);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .panel {
    background: #181820;
    border: 1px solid #3a3a42;
    border-radius: 8px;
    padding: 1.25rem;
    width: 520px;
    max-width: 92vw;
    color: #d9d4c7;
    font-family: system-ui, -apple-system, sans-serif;
    box-shadow: 0 24px 80px rgba(0, 0, 0, 0.45);
  }

  .header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 0.9rem;
  }

  h2 {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 600;
  }

  .header span {
    color: #8f8f9a;
    font-size: 0.78rem;
  }

  label {
    display: block;
    margin-bottom: 0.75rem;
    font-size: 0.8rem;
    color: #9a96a0;
  }

  .field-label {
    display: block;
    margin-bottom: 0.25rem;
    color: #9a96a0;
    font-size: 0.8rem;
  }

  .profile-picker {
    position: relative;
    margin-bottom: 0.75rem;
  }

  .profile-trigger {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) minmax(0, 1.25fr) auto;
    align-items: center;
    gap: 0.55rem;
    width: 100%;
    min-height: 2.6rem;
    background: #0e0e12;
    border: 1px solid #41414a;
    color: #d9d4c7;
    text-align: left;
  }

  .profile-trigger:focus {
    outline: none;
    border-color: #3b82f6;
  }

  .profile-title,
  .profile-subtitle {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .profile-subtitle {
    color: #8f8f9a;
    font-size: 0.78rem;
  }

  .chevron {
    color: #8f8f9a;
    font-size: 1rem;
    line-height: 1;
  }

  .profile-menu {
    position: absolute;
    left: 0;
    right: 0;
    top: calc(100% + 0.35rem);
    z-index: 2;
    max-height: 260px;
    overflow-y: auto;
    background: #101015;
    border: 1px solid #41414a;
    border-radius: 7px;
    box-shadow: 0 18px 50px rgba(0, 0, 0, 0.45);
    padding: 0.35rem;
  }

  .profile-menu button {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 0.55rem;
    width: 100%;
    background: transparent;
    color: #d9d4c7;
    text-align: left;
    padding: 0.45rem 0.5rem;
  }

  .profile-menu button:hover,
  .profile-menu button.active {
    background: #1b2432;
  }

  .profile-option-main {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 0.1rem;
  }

  .profile-option-main strong,
  .profile-option-main small {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .profile-option-main small,
  .persisted-dot {
    color: #8f8f9a;
    font-size: 0.7rem;
  }

  .kind-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    border: 1px solid #3a3a3a;
    min-width: 3.6rem;
    padding: 0.13rem 0.35rem;
    color: #d9d4c7;
    font-size: 0.66rem;
    font-weight: 700;
    text-transform: uppercase;
  }

  .badge-local {
    background: rgba(34, 197, 94, 0.14);
    border-color: rgba(34, 197, 94, 0.65);
    color: #86efac;
  }

  .badge-ssh {
    background: rgba(59, 130, 246, 0.16);
    border-color: rgba(59, 130, 246, 0.7);
    color: #93c5fd;
  }

  .badge-command {
    background: rgba(234, 179, 8, 0.14);
    border-color: rgba(234, 179, 8, 0.72);
    color: #fde68a;
  }

  .badge-wsl {
    background: rgba(6, 182, 212, 0.14);
    border-color: rgba(6, 182, 212, 0.7);
    color: #67e8f9;
  }

  input {
    display: block;
    width: 100%;
    margin-top: 0.25rem;
    padding: 0.52rem 0.6rem;
    background: #0e0e12;
    border: 1px solid #41414a;
    border-radius: 4px;
    color: #d9d4c7;
    font-size: 0.9rem;
    box-sizing: border-box;
  }

  input:focus {
    outline: none;
    border-color: #3b82f6;
  }

  .loading {
    border: 1px solid #34343c;
    border-radius: 6px;
    background: #111116;
    padding: 0.75rem;
    margin-bottom: 0.8rem;
  }

  .loading {
    color: #8f8f9a;
    font-size: 0.85rem;
  }

  .error {
    background: rgba(239, 68, 68, 0.14);
    border: 1px solid #ef4444;
    border-radius: 4px;
    padding: 0.5rem 0.75rem;
    margin-bottom: 0.75rem;
    color: #f87171;
    font-size: 0.85rem;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 0.25rem;
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

  @media (max-width: 560px) {
    .profile-trigger {
      grid-template-columns: auto minmax(0, 1fr) auto;
    }

    .profile-trigger .profile-subtitle {
      display: none;
    }
  }
</style>
