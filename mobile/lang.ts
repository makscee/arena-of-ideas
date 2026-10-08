// The language the rules are written in (M4-3): card text, keywords, battle
// captions and traces. Until the language switch lands (M4-2), a dev switch:
// `?lang=ru` in the page's address reads the rules in Russian; anything else
// is English.

import type { Lang } from "../src/describe";

let lang: Lang | undefined;

/** The rules' language for this page load. */
export function rulesLang(): Lang {
  if (lang) return lang;
  try {
    lang = new URLSearchParams(location.search).get("lang") === "ru" ? "ru" : "en";
  } catch {
    lang = "en";
  }
  return lang;
}

/** The language as describe/glossary options take it: undefined for English,
 * so English calls stay exactly as they were. */
export const rulesLangOpt = (): Lang | undefined => (rulesLang() === "ru" ? "ru" : undefined);
