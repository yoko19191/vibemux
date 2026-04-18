export interface DeckPaneLayout {
  sessionId: string;
  width: number;
  isFocused: boolean;
  /** CSS left offset in px */
  left: number;
  /** z-index for stacking order */
  zIndex: number;
}

export interface DeckLayoutOptions {
  focusedPaneWidth?: number;
}

const DEFAULT_FOCUSED_RATIO = 0.6;
const MIN_FOCUSED_WIDTH = 260;
const MIN_PEEK_WIDTH = 16;
const MAX_PEEK_WIDTH = 160;

export function calculateDeckLayout(
  containerWidth: number,
  sessionIds: string[],
  focusedSessionId: string | null,
  options: DeckLayoutOptions = {},
): DeckPaneLayout[] {
  if (sessionIds.length === 0) return [];

  if (sessionIds.length === 1) {
    return [
      {
        sessionId: sessionIds[0],
        width: containerWidth,
        isFocused: true,
        left: 0,
        zIndex: 10,
      },
    ];
  }

  const focusedId = focusedSessionId && sessionIds.includes(focusedSessionId) ? focusedSessionId : sessionIds[0];
  const focusedIdx = sessionIds.indexOf(focusedId);

  const leftIds = sessionIds.slice(0, focusedIdx);
  const rightIds = sessionIds.slice(focusedIdx + 1);

  const sideCount = leftIds.length + rightIds.length;
  const focusedRatio = clamp(options.focusedPaneWidth ?? DEFAULT_FOCUSED_RATIO, 0.3, 0.9);
  const targetFocusedWidth = containerWidth * focusedRatio;
  const maxPeekForMinFocus = sideCount > 0
    ? Math.max(0, (containerWidth - Math.min(MIN_FOCUSED_WIDTH, containerWidth)) / sideCount)
    : 0;
  const rawPeekWidth = sideCount > 0 ? (containerWidth - targetFocusedWidth) / sideCount : 0;
  const peekWidth = sideCount > 0
    ? Math.min(clamp(rawPeekWidth, MIN_PEEK_WIDTH, MAX_PEEK_WIDTH), maxPeekForMinFocus)
    : 0;
  const leftPeekTotal = leftIds.length * peekWidth;
  const rightPeekTotal = rightIds.length * peekWidth;
  const focusedWidth = Math.max(0, containerWidth - leftPeekTotal - rightPeekTotal);

  const layouts: DeckPaneLayout[] = [];

  // Left stack: each pane is full-ish width but shifted far left so only
  // its RIGHT edge peeks out. Further from focused = higher z-index so
  // every pane's peek strip is visible (not covered by the one closer to focused).
  leftIds.forEach((id, i) => {
    const paneWidth = focusedWidth;
    const paneLeft = (i + 1) * peekWidth - paneWidth;
    layouts.push({
      sessionId: id,
      width: paneWidth,
      isFocused: false,
      left: paneLeft,
      zIndex: leftIds.length - i, // further from focused = higher z
    });
  });

  // Focused pane
  layouts.push({
    sessionId: focusedId,
    width: focusedWidth,
    isFocused: true,
    left: leftPeekTotal,
    zIndex: 10,
  });

  // Right stack: each pane is full-ish width but shifted far right so only
  // its LEFT edge peeks out. Further from focused = higher z-index so
  // every pane's peek strip is visible.
  rightIds.forEach((id, i) => {
    const paneWidth = focusedWidth;
    const rightEdge = leftPeekTotal + focusedWidth;
    const paneLeft = rightEdge + i * peekWidth;
    layouts.push({
      sessionId: id,
      width: paneWidth,
      isFocused: false,
      left: paneLeft,
      zIndex: i + 1, // further from focused = higher z
    });
  });

  return layouts;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}
