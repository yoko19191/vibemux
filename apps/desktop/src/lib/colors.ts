import type { ColorToken } from "./types";

export const colorMap: Record<ColorToken, string> = {
  Red: "#ef4444",
  Orange: "#f97316",
  Yellow: "#eab308",
  Green: "#22c55e",
  Cyan: "#06b6d4",
  Blue: "#3b82f6",
  Purple: "#a855f7",
  Pink: "#ec4899",
};

export const colorTokens: { token: ColorToken; color: string }[] = (
  Object.keys(colorMap) as ColorToken[]
).map((token) => ({ token, color: colorMap[token] }));
