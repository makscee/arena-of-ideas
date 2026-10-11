// The player's language (M4-2): the screens (i18n/), the rules' words (M4-3's
// card text, keywords, captions, traces) and the server's text all follow it.
// The switch in settings wins, kept on this device; else the phone's
// language (`ru…` → Russian, else English). `?lang=ru|en` in the page's
// address overrides both for this load (e2e, dev).

import type { Lang } from "../src/describe";

export const LANGS = ["en", "ru"] as const;
const LANG_KEY = "arena.lang";

const isLang = (x: unknown): x is Lang => x === "en" || x === "ru";

/** The language from what the page knows: the address's `?lang=`, then the
 * saved switch, then the phone's language. */
export function pickLang(query: string | null, saved: string | null, phone: string | undefined): Lang {
  if (isLang(query)) return query;
  if (isLang(saved)) return saved;
  return /^ru\b/i.test(phone ?? "") ? "ru" : "en";
}

let lang: Lang | undefined;

/** The player's language for this page load. */
export function uiLang(): Lang {
  if (lang) return lang;
  // Off the page (vitest, node's own navigator) it's English.
  if (typeof document === "undefined") return (lang = "en");
  let query: string | null = null;
  let saved: string | null = null;
  let phone: string | undefined;
  try {
    query = new URLSearchParams(location.search).get("lang");
  } catch {
    /* no page (tests) */
  }
  try {
    saved = localStorage.getItem(LANG_KEY);
  } catch {
    /* private mode: the phone's language each load */
  }
  try {
    phone = navigator.language;
  } catch {
    /* no navigator */
  }
  lang = pickLang(query, saved, phone);
  return lang;
}

/** Where the language switch goes next: the address without its `?lang=`
 * (it would win over the choice), or null when that is the address itself.
 * Then only a reload redraws the page: `location.replace` to the same address
 * with a `#` in it (the own `#invite=` link after joining) is a hash
 * navigation, which reloads nothing (#919). */
export function langAddress(href: string): string | null {
  const url = new URL(href);
  if (!url.searchParams.has("lang")) return null;
  url.searchParams.delete("lang");
  return url.toString();
}

/** The settings switch: keeps the choice on this device, then reloads the
 * page so every screen, card and caption is drawn in it (the run waits on the
 * server; Continue brings it back). */
export function chooseLang(next: Lang): void {
  try {
    localStorage.setItem(LANG_KEY, next);
  } catch {
    /* private mode: this load only */
  }
  lang = next;
  try {
    const to = langAddress(location.href);
    if (to) location.replace(to);
    else location.reload();
  } catch {
    /* no page (tests) */
  }
}

/** The rules' language (M4-3): the player's language. */
export const rulesLang = (): Lang => uiLang();

/** The language as describe/glossary options take it: undefined for English,
 * so English calls stay exactly as they were. */
export const rulesLangOpt = (): Lang | undefined => (uiLang() === "ru" ? "ru" : undefined);
