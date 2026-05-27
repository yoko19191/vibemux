<script lang="ts">
  import type { NotificationItem, SessionSnapshot } from "./types";

  interface Props {
    notifications: NotificationItem[];
    sessions: SessionSnapshot[];
    onJump: (item: NotificationItem) => void | Promise<void>;
    onMarkRead: (id: string) => void;
    onClear: (id: string) => void;
    onClearAll: () => void;
    onClose: () => void;
  }

  let { notifications, sessions, onJump, onMarkRead, onClear, onClearAll, onClose }: Props = $props();

  let ordered = $derived([...notifications].sort((a, b) => b.createdAt.localeCompare(a.createdAt)));
  let unreadCount = $derived(notifications.filter((item) => !item.readAt).length);

  function sessionFor(item: NotificationItem): SessionSnapshot | undefined {
    return sessions.find((session) => session.id === item.sessionId);
  }

  function badgeFor(item: NotificationItem): string | null {
    return sessionFor(item)?.identity.origin.badge ?? null;
  }

  function timeAgo(iso: string): string {
    const diff = Date.now() - new Date(iso).getTime();
    const seconds = Math.max(0, Math.floor(diff / 1000));
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h`;
    return `${Math.floor(hours / 24)}d`;
  }

  function kindMarker(kind: NotificationItem["kind"]): string {
    switch (kind) {
      case "NeedsInput": return "!";
      case "Done": return "✓";
    }
  }

  function kindClass(kind: NotificationItem["kind"]): string {
    switch (kind) {
      case "NeedsInput": return "needs-input";
      case "Done": return "done";
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="notification-scrim" onclick={onClose}>
  <section class="notification-center" onclick={(e) => e.stopPropagation()}>
    <header class="notification-header">
      <div>
        <div class="title">Notifications</div>
        <div class="subtitle">{unreadCount} unread</div>
      </div>
      <div class="header-actions">
        <button class="text-btn" onclick={onClearAll} disabled={notifications.length === 0}>Clear</button>
        <button class="icon-btn" onclick={onClose} aria-label="Close notifications" title="Close">×</button>
      </div>
    </header>

    {#if ordered.length === 0}
      <div class="empty">No notifications</div>
    {:else}
      <div class="notification-list">
        {#each ordered as item (item.id)}
          {@const session = sessionFor(item)}
          {@const badge = badgeFor(item)}
          <article class:read={!!item.readAt} class="notification-row">
            <div class="marker {kindClass(item.kind)}">{kindMarker(item.kind)}</div>
            <div class="notification-copy">
              <div class="row-title">
                {#if badge}
                  <span class="origin-badge">{badge}</span>
                {/if}
                <span class="session-title">{item.title}</span>
                <span class="age">{timeAgo(item.createdAt)}</span>
              </div>
              <div class="body">{item.body}</div>
            </div>
            <div class="row-actions">
              <button class="text-btn" onclick={() => onJump(item)} disabled={!session}>Jump</button>
              {#if !item.readAt}
                <button class="icon-btn" onclick={() => onMarkRead(item.id)} aria-label="Mark read" title="Mark read">✓</button>
              {/if}
              <button class="icon-btn" onclick={() => onClear(item.id)} aria-label="Clear notification" title="Clear">×</button>
            </div>
          </article>
        {/each}
      </div>
    {/if}
  </section>
</div>

<style>
  .notification-scrim {
    position: fixed;
    inset: 28px 0 0;
    background: rgba(0, 0, 0, 0.22);
    z-index: 190;
  }

  .notification-center {
    position: absolute;
    top: 8px;
    right: 10px;
    width: 390px;
    max-width: calc(100vw - 20px);
    max-height: min(520px, calc(100vh - 46px));
    display: flex;
    flex-direction: column;
    background: #181818;
    border: 1px solid #303030;
    border-radius: 8px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.55);
    font-family: system-ui, -apple-system, sans-serif;
    overflow: hidden;
  }

  .notification-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.75rem 0.85rem;
    border-bottom: 1px solid #2a2a2a;
  }

  .title {
    color: #d9d4c7;
    font-size: 0.85rem;
    font-weight: 650;
  }

  .subtitle {
    color: #666;
    font-size: 0.68rem;
    margin-top: 0.12rem;
  }

  .header-actions,
  .row-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .notification-list {
    overflow-y: auto;
  }

  .notification-row {
    display: grid;
    grid-template-columns: 20px minmax(0, 1fr) auto;
    gap: 0.55rem;
    align-items: center;
    padding: 0.65rem 0.75rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .notification-row.read {
    opacity: 0.55;
  }

  .marker {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 0.68rem;
    font-weight: 700;
    color: #111;
  }

  .marker.needs-input {
    background: #facc15;
  }

  .marker.done {
    background: #777;
    color: #111;
  }

  .notification-copy {
    min-width: 0;
  }

  .row-title {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    min-width: 0;
  }

  .origin-badge {
    flex-shrink: 0;
    color: #facc15;
    border: 1px solid rgba(250, 204, 21, 0.45);
    border-radius: 3px;
    padding: 0 0.22rem;
    font-size: 0.58rem;
    font-weight: 750;
    line-height: 1.2;
  }

  .session-title {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #d9d4c7;
    font-size: 0.76rem;
    font-weight: 600;
  }

  .age {
    flex-shrink: 0;
    color: #555;
    font-size: 0.64rem;
  }

  .body {
    margin-top: 0.18rem;
    color: #888;
    font-size: 0.7rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .text-btn,
  .icon-btn {
    border: none;
    border-radius: 4px;
    background: transparent;
    color: #777;
    cursor: pointer;
    font-family: inherit;
    transition: background 100ms ease, color 100ms ease;
  }

  .text-btn {
    padding: 0.18rem 0.42rem;
    font-size: 0.68rem;
  }

  .icon-btn {
    width: 22px;
    height: 22px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 0.78rem;
  }

  .text-btn:hover:not(:disabled),
  .icon-btn:hover {
    background: #2a2a2a;
    color: #d9d4c7;
  }

  .text-btn:disabled {
    cursor: default;
    color: #444;
  }

  .empty {
    padding: 2.2rem 1rem;
    text-align: center;
    color: #666;
    font-size: 0.78rem;
  }
</style>
