export interface DeckPaneLayout {
  sessionId: string;
  width: number;
  isFocused: boolean;
  /** CSS left offset in px */
  left: number;
  /** z-index for stacking order */
  zIndex: number;
}

/** How many px of a non-focused pane peeks out from behind the focused one */
const PEEK_WIDTH = 48;

export function calculateDeckLayout(
  containerWidth: number,
  sessionIds: string[],
  focusedSessionId: string | null,
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

  const focusedId = focusedSessionId ?? sessionIds[0];
  const focusedIdx = sessionIds.indexOf(focusedId);

  const leftIds = sessionIds.slice(0, focusedIdx);
  const rightIds = sessionIds.slice(focusedIdx + 1);

  const leftPeekTotal = leftIds.length * PEEK_WIDTH;
  const rightPeekTotal = rightIds.length * PEEK_WIDTH;
  const focusedWidth = containerWidth - leftPeekTotal - rightPeekTotal;

  const layouts: DeckPaneLayout[] = [];

  // Left stack: each pane is full-ish width but shifted far left so only
  // its RIGHT edge peeks out. Closer to focused = higher z-index.
  leftIds.forEach((id, i) => {
    const paneWidth = focusedWidth; // same width as focused
    // Position so that only PEEK_WIDTH of the right edge is visible
    // The visible strip starts at i * PEEK_WIDTH, so the pane's right edge
    // aligns to (i + 1) * PEEK_WIDTH → left = (i+1)*PEEK - paneWidth
    const paneLeft = (i + 1) * PEEK_WIDTH - paneWidth;
    layouts.push({
      sessionId: id,
      width: paneWidth,
      isFocused: false,
      left: paneLeft,
      zIndex: i + 1, // further left = lower z
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
  // its LEFT edge peeks out. Closer to focused = higher z-index.
  rightIds.forEach((id, i) => {
    const paneWidth = focusedWidth;
    const rightEdge = leftPeekTotal + focusedWidth;
    // The visible strip starts at rightEdge + i * PEEK_WIDTH
    const paneLeft = rightEdge + i * PEEK_WIDTH;
    layouts.push({
      sessionId: id,
      width: paneWidth,
      isFocused: false,
      left: paneLeft,
      zIndex: rightIds.length - i, // closer to focused = higher z
    });
  });

  return layouts;
}
