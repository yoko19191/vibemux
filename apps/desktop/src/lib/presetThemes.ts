// Preset themes for onboarding step 3
export interface ThemePreset {
  name: string;
  background: string;
  foreground: string;
  cursor: string;
  selection: string;
  black: string;
  red: string;
  green: string;
  yellow: string;
  blue: string;
  magenta: string;
  cyan: string;
  white: string;
  bright_black: string;
  bright_red: string;
  bright_green: string;
  bright_yellow: string;
  bright_blue: string;
  bright_magenta: string;
  bright_cyan: string;
  bright_white: string;
}

export const presetThemes: ThemePreset[] = [
  {
    name: "Dark",
    background: "#111111",
    foreground: "#d9d4c7",
    cursor: "#ff6b57",
    selection: "#3b82f640",
    black: "#1a1a1a",
    red: "#ef4444",
    green: "#22c55e",
    yellow: "#eab308",
    blue: "#3b82f6",
    magenta: "#a855f7",
    cyan: "#06b6d4",
    white: "#d9d4c7",
    bright_black: "#555555",
    bright_red: "#f87171",
    bright_green: "#4ade80",
    bright_yellow: "#facc15",
    bright_blue: "#60a5fa",
    bright_magenta: "#c084fc",
    bright_cyan: "#22d3ee",
    bright_white: "#f5f5f5",
  },
  {
    name: "Solarized Dark",
    background: "#002b36",
    foreground: "#839496",
    cursor: "#268bd2",
    selection: "#073642",
    black: "#073642",
    red: "#dc322f",
    green: "#859900",
    yellow: "#b58900",
    blue: "#268bd2",
    magenta: "#d33682",
    cyan: "#2aa198",
    white: "#eee8d5",
    bright_black: "#002b36",
    bright_red: "#cb4b16",
    bright_green: "#586e75",
    bright_yellow: "#657b83",
    bright_blue: "#839496",
    bright_magenta: "#6c71c4",
    bright_cyan: "#93a1a1",
    bright_white: "#fdf6e3",
  },
  {
    name: "Monokai",
    background: "#272822",
    foreground: "#f8f8f2",
    cursor: "#f8f8f0",
    selection: "#49483e",
    black: "#272822",
    red: "#f92672",
    green: "#a6e22e",
    yellow: "#f4bf75",
    blue: "#66d9ef",
    magenta: "#ae81ff",
    cyan: "#a1efe4",
    white: "#f8f8f2",
    bright_black: "#75715e",
    bright_red: "#f92672",
    bright_green: "#a6e22e",
    bright_yellow: "#f4bf75",
    bright_blue: "#66d9ef",
    bright_magenta: "#ae81ff",
    bright_cyan: "#a1efe4",
    bright_white: "#f9f8f5",
  },
];
