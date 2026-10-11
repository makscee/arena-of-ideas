// The palette switch (mission 5, M5-1, makscee/void-board#889): the Toybox
// look in three palettes that share the same colour roles (style.css's token
// block). Night Plum is the default; the choice is kept on this device and
// applied at once (only the tokens change, nothing redraws). `?theme=` in the
// page's address overrides it for this load (e2e, dev).

export const THEMES = ["plum", "brass", "felt"] as const;
export type Theme = (typeof THEMES)[number];
export const DEFAULT_THEME: Theme = "plum";
const THEME_KEY = "arena.theme";

/** Each palette's ground, for the browser's bar (`<meta name="theme-color">`)
 * and the switch's swatches: [ground, your side, their side, the action].
 * The same values as style.css's token block. */
export const THEME_SWATCH: Record<Theme, readonly [string, string, string, string]> = {
  plum: ["#241d33", "#3fb9a0", "#e07a86", "#f0c24b"],
  brass: ["#1b2133", "#3fa98f", "#d9765f", "#e8b44a"],
  felt: ["#1f3229", "#5ab0d6", "#d0644f", "#e6b93d"],
};

const isTheme = (x: unknown): x is Theme => (THEMES as readonly unknown[]).includes(x);

/** The palette from what the page knows: the address's `?theme=`, then the
 * saved switch, then Night Plum. */
export function pickTheme(query: string | null, saved: string | null): Theme {
  if (isTheme(query)) return query;
  if (isTheme(saved)) return saved;
  return DEFAULT_THEME;
}

let theme: Theme | undefined;

/** This device's palette. */
export function uiTheme(): Theme {
  if (theme) return theme;
  if (typeof document === "undefined") return (theme = DEFAULT_THEME);
  let query: string | null = null;
  let saved: string | null = null;
  try {
    query = new URLSearchParams(location.search).get("theme");
  } catch {
    /* no page (tests) */
  }
  try {
    saved = localStorage.getItem(THEME_KEY);
  } catch {
    /* private mode: Night Plum each load */
  }
  theme = pickTheme(query, saved);
  return theme;
}

/** Puts the palette on the page: `<html data-theme>` picks the token set,
 * and the browser's bar follows the ground. */
export function applyTheme(next: Theme = uiTheme()): void {
  if (typeof document === "undefined") return;
  document.documentElement.dataset.theme = next;
  document.querySelector('meta[name="theme-color"]')?.setAttribute("content", THEME_SWATCH[next][0]);
}

/** The settings switch: keeps the choice on this device and applies it. */
export function chooseTheme(next: Theme): void {
  theme = next;
  try {
    localStorage.setItem(THEME_KEY, next);
  } catch {
    /* private mode: this load only */
  }
  applyTheme(next);
}
