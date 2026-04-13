export interface DeckPaneLayout {
  sessionId: string;
  width: number;
  isFocused: boolean;
}

const MIN_FOCUSED_WIDTH = 720;
const MIN_PERIPHERAL_WIDTH = 120;

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
      },
    ];
  }

  const focusedId = focusedSessionId ?? sessionIds[0];
  const peripheralCount = sessionIds.length - 1;

  let focusedWidth = Math.max(
    MIN_FOCUSED_WIDTH,
    containerWidth - peripheralCount * MIN_PERIPHERAL_WIDTH,
  );

  // If focused width would leave no room for peripherals, split evenly
  if (focusedWidth >= containerWidth) {
    focusedWidth = Math.floor(containerWidth * 0.6);
  }

  const remainingWidth = containerWidth - focusedWidth;
  const peripheralWidth = Math.floor(remainingWidth / peripheralCount);

  return sessionIds.map((sessionId) => ({
    sessionId,
    width: sessionId === focusedId ? focusedWidth : peripheralWidth,
    isFocused: sessionId === focusedId,
  }));
}
