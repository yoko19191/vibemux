/**
 * Handles replay events for recalled sessions.
 * Writes buffered output to xterm in chunks to avoid UI freeze.
 */

const CHUNK_SIZE = 4096; // bytes per frame
const RESTORING_THRESHOLD_MS = 200;

interface ReplayState {
  chunks: string[];
  timer: ReturnType<typeof setTimeout> | null;
  restoringTimer: ReturnType<typeof setTimeout> | null;
  isReplaying: boolean;
  onRestoring: (active: boolean) => void;
  writeOutput: (data: string) => void;
}

const activeReplays = new Map<string, ReplayState>();

export function onReplayStart(
  sessionId: string,
  writeOutput: (data: string) => void,
  onRestoring: (active: boolean) => void,
) {
  // Cancel any existing replay for this session
  cancelReplay(sessionId);

  const state: ReplayState = {
    chunks: [],
    timer: null,
    restoringTimer: null,
    isReplaying: true,
    onRestoring,
    writeOutput,
  };

  // Show "Restoring..." indicator after threshold
  state.restoringTimer = setTimeout(() => {
    if (state.isReplaying) {
      onRestoring(true);
    }
  }, RESTORING_THRESHOLD_MS);

  activeReplays.set(sessionId, state);
}

export function onReplayChunk(sessionId: string, data: string) {
  const state = activeReplays.get(sessionId);
  if (!state) return;

  // Split large chunks into smaller pieces
  for (let i = 0; i < data.length; i += CHUNK_SIZE) {
    state.chunks.push(data.slice(i, i + CHUNK_SIZE));
  }

  // Schedule draining if not already scheduled
  if (!state.timer) {
    scheduleNextChunk(sessionId, state);
  }
}

export function onReplayEnd(sessionId: string) {
  const state = activeReplays.get(sessionId);
  if (!state) return;

  // Drain remaining chunks synchronously (they're already small)
  if (state.timer) {
    clearTimeout(state.timer);
    state.timer = null;
  }
  for (const chunk of state.chunks) {
    state.writeOutput(chunk);
  }
  state.chunks = [];

  finishReplay(sessionId, state);
}

export function cancelReplay(sessionId: string) {
  const state = activeReplays.get(sessionId);
  if (!state) return;
  if (state.timer) clearTimeout(state.timer);
  if (state.restoringTimer) clearTimeout(state.restoringTimer);
  state.isReplaying = false;
  activeReplays.delete(sessionId);
}

export function isReplaying(sessionId: string): boolean {
  return activeReplays.has(sessionId);
}

function scheduleNextChunk(sessionId: string, state: ReplayState) {
  state.timer = setTimeout(() => {
    state.timer = null;
    const chunk = state.chunks.shift();
    if (chunk) {
      state.writeOutput(chunk);
    }
    if (state.chunks.length > 0) {
      scheduleNextChunk(sessionId, state);
    }
  }, 0);
}

function finishReplay(sessionId: string, state: ReplayState) {
  if (state.restoringTimer) {
    clearTimeout(state.restoringTimer);
    state.restoringTimer = null;
  }
  state.isReplaying = false;
  state.onRestoring(false);
  activeReplays.delete(sessionId);
}
