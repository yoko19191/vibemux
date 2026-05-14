export type DesktopPlatform = "macos" | "windows" | "linux" | "unknown";

export function detectDesktopPlatform(): DesktopPlatform {
  if (typeof navigator === "undefined") return "unknown";

  const platform = navigator.platform.toLowerCase();
  const userAgent = navigator.userAgent.toLowerCase();

  if (platform.includes("mac") || userAgent.includes("mac os")) return "macos";
  if (platform.includes("win") || userAgent.includes("windows")) return "windows";
  if (platform.includes("linux") || userAgent.includes("linux")) return "linux";

  return "unknown";
}

export function isMacOS(platform: DesktopPlatform): boolean {
  return platform === "macos";
}

export function primaryShortcutModifier(platform: DesktopPlatform): "Cmd" | "Ctrl" {
  return isMacOS(platform) ? "Cmd" : "Ctrl";
}
