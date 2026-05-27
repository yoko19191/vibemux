/**
 * Terminal font helpers.
 *
 * Config stores `font_family` as a CSS-style stack (e.g.
 * `"JetBrains Mono", "Symbols Nerd Font", Menlo, Monaco, monospace`)
 * so that:
 *   1. Missing alphanumeric glyphs cascade through real monospace fonts
 *      instead of falling through to the browser's proportional sans
 *      (which would desync xterm cell metrics).
 *   2. Icon glyphs in the PUA range cascade to Symbols Nerd Font (gated
 *      via unicode-range in fonts.css) regardless of which primary font
 *      the user picked. Users running starship / lazygit / nvim etc. get
 *      working icons without configuring anything.
 *
 * The Settings UI lets the user pick or type a single primary family —
 * these helpers convert between the user-facing primary and the stored
 * full stack.
 */

const ICON_FALLBACK = '"Symbols Nerd Font"';
const SYSTEM_FALLBACKS = ['Menlo', 'Monaco', '"Courier New"', 'monospace'] as const;

export function extractPrimaryFamily(stack: string | undefined | null): string {
  if (!stack) return 'Menlo';
  const first = stack.split(',')[0]?.trim() ?? '';
  return first.replace(/^['"]|['"]$/g, '') || 'Menlo';
}

export function buildFontStack(primary: string): string {
  const trimmed = primary.trim();
  if (!trimmed) return [ICON_FALLBACK, ...SYSTEM_FALLBACKS].join(', ');

  // If the user typed/pasted a stack of their own, respect it but ensure
  // both icon fallback and a generic monospace terminator are present.
  if (trimmed.includes(',')) {
    const parts = trimmed.split(',').map((p) => p.trim()).filter(Boolean);
    const hasIconFallback = parts.some(
      (p) => p.replace(/^['"]|['"]$/g, '').toLowerCase() === 'symbols nerd font',
    );
    const hasMono = parts.some((p) => /\bmonospace\b/i.test(p));
    const out = [...parts];
    if (!hasIconFallback) {
      // Insert icon fallback just before the generic terminator (or at end).
      const monoIdx = out.findIndex((p) => /\bmonospace\b/i.test(p));
      if (monoIdx >= 0) out.splice(monoIdx, 0, ICON_FALLBACK);
      else out.push(ICON_FALLBACK);
    }
    if (!hasMono) out.push('monospace');
    return out.join(', ');
  }

  const quoted = /\s/.test(trimmed) ? `"${trimmed}"` : trimmed;
  const sysTail = SYSTEM_FALLBACKS.filter(
    (f) => f.replace(/^['"]|['"]$/g, '').toLowerCase() !== trimmed.toLowerCase(),
  );
  // Order: primary → icon fallback → system mono fallbacks → monospace
  // The icon fallback's unicode-range gates it to PUA, so it never
  // intercepts ASCII even though it sits ahead of system fonts.
  const monoIdx = sysTail.findIndex((f) => f === 'monospace');
  const beforeMono = monoIdx >= 0 ? sysTail.slice(0, monoIdx) : sysTail;
  const monoTerm = monoIdx >= 0 ? sysTail.slice(monoIdx) : ['monospace'];
  return [quoted, ICON_FALLBACK, ...beforeMono, ...monoTerm].join(', ');
}
