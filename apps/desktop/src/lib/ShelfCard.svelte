<script lang="ts">
  import type { SessionSnapshot, ColorToken, AttentionState } from "./types";

  interface Props {
    session: SessionSnapshot;
  }

  let { session }: Props = $props();

  const colorMap: Record<ColorToken, string> = {
    Red: "#ef4444",
    Orange: "#f97316",
    Yellow: "#eab308",
    Green: "#22c55e",
    Cyan: "#06b6d4",
    Blue: "#3b82f6",
    Purple: "#a855f7",
    Pink: "#ec4899",
  };

  function formatTime(iso: string): string {
    const d = new Date(iso);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffSec = Math.floor(diffMs / 1000);
    if (diffSec < 60) return `${diffSec}s ago`;
    const diffMin = Math.floor(diffSec / 60);
    if (diffMin < 60) return `${diffMin}m ago`;
    const diffHr = Math.floor(diffMin / 60);
    return `${diffHr}h ago`;
  }

  function attentionBadge(state: AttentionState): { label: string; color: string } | null {
    switch (state) {
      case "Active": return { label: "●", color: "#3b82f6" };
      case "NeedsInput": return { label: "Needs Input", color: "#eab308" };
      case "Failed": return { label: "Failed", color: "#ef4444" };
      case "Done": return { label: "Done", color: "#6b7280" };
      default: return null;
    }
  }

  let dotColor = $derived(colorMap[session.color] ?? "#666");
  let badge = $derived(attentionBadge(session.attentionState));
  let shortCwd = $derived(session.cwd.replace(/^.*\/([^/]+)$/, "$1") || session.cwd);
</script>

<div class="shelf-card" style="border-color: {dotColor};">
  <div class="card-header">
    <span class="color-dot" style="background: {dotColor};"></span>
    <span class="session-name">{session.name}</span>
    {#if badge}
      <span class="badge" style="color: {badge.color};">{badge.label}</span>
    {/if}
  </div>
  <div class="card-cwd">{shortCwd}</div>
  <div class="card-time">{formatTime(session.lastActivityAt)}</div>
</div>

<style>
  .shelf-card {
    flex-shrink: 0;
    width: 160px;
    padding: 0.4rem 0.5rem;
    border: 1px solid;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.04);
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    transition: background 100ms;
  }

  .shelf-card:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    overflow: hidden;
  }

  .color-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .session-name {
    font-size: 0.75rem;
    font-weight: 600;
    color: #d9d4c7;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .badge {
    font-size: 0.65rem;
    font-weight: 600;
    flex-shrink: 0;
  }

  .card-cwd {
    font-size: 0.65rem;
    color: #666;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .card-time {
    font-size: 0.6rem;
    color: #555;
  }
</style>
