/**
 * Parses a prefix key config string like 'ctrl+b', 'cmd+space', 'ctrl+`', 'ctrl+a'
 * into a matcher object used to detect the key in keyboard events.
 */
export interface PrefixKeyMatcher {
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  key: string;
  code?: string; // optional, used for backtick
}

const isMac = typeof navigator !== "undefined" && navigator.platform.toUpperCase().includes("MAC");

export function parsePrefixKey(configString: string): PrefixKeyMatcher {
  const parts = configString.toLowerCase().split("+");
  const keyPart = parts[parts.length - 1];
  const modifiers = parts.slice(0, -1);

  const ctrlKey = modifiers.includes("ctrl");
  const altKey = modifiers.includes("alt");
  const shiftKey = modifiers.includes("shift");
  // 'cmd' maps to metaKey on macOS, ctrlKey on other platforms
  const cmdMod = modifiers.includes("cmd");
  const metaKey = cmdMod && isMac;
  const ctrlFromCmd = cmdMod && !isMac;

  let key: string;
  let code: string | undefined;

  if (keyPart === "space") {
    key = " ";
  } else if (keyPart === "`" || keyPart === "backtick") {
    key = "`";
    code = "Backquote";
  } else {
    key = keyPart;
  }

  return {
    ctrlKey: ctrlKey || ctrlFromCmd,
    metaKey,
    altKey,
    shiftKey,
    key,
    code,
  };
}

export function matchesPrefixKey(e: KeyboardEvent, matcher: PrefixKeyMatcher): boolean {
  if (e.ctrlKey !== matcher.ctrlKey) return false;
  if (e.metaKey !== matcher.metaKey) return false;
  if (e.altKey !== matcher.altKey) return false;
  if (e.shiftKey !== matcher.shiftKey) return false;
  if (matcher.code) {
    return e.key === matcher.key || e.code === matcher.code;
  }
  return e.key.toLowerCase() === matcher.key.toLowerCase();
}

/**
 * Returns a human-readable display string for the prefix key config string.
 * e.g. 'ctrl+b' -> 'Ctrl+B', 'cmd+space' -> 'Cmd+Space'
 */
export function formatPrefixKey(configString: string): string {
  return configString
    .split("+")
    .map((part) => {
      if (part === "ctrl") return "Ctrl";
      if (part === "cmd") return "Cmd";
      if (part === "alt") return "Alt";
      if (part === "shift") return "Shift";
      if (part === "space") return "Space";
      if (part === "`" || part === "backtick") return "`";
      return part.toUpperCase();
    })
    .join("+");
}
