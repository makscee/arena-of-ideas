// `--lang ru|en` for the phone e2e (M4-2): the browser's locale (so the
// client picks the language as a phone does) and the catalog's words, so a
// check reads the text in the language the screen shows.
import { register } from "tsx/esm/api";

register();
const { en } = await import("../mobile/i18n/en.ts");
const { ru } = await import("../mobile/i18n/ru.ts");

/** The run's language from the command line: English unless `--lang ru`. */
export function e2eLang(args) {
  const i = args.indexOf("--lang");
  const lang = i >= 0 ? args[i + 1] : "en";
  if (lang !== "en" && lang !== "ru") throw new Error(`--lang ${lang}: en or ru`);
  return lang;
}

/** The browser locale that makes the client pick `lang` on its own. */
export const localeOf = (lang) => (lang === "ru" ? "ru-RU" : "en-US");

/** t() as the client has it, in `lang`: L("ideas.why", { n: 3 }). */
export function catalog(lang) {
  const table = lang === "ru" ? { ...en, ...ru } : en;
  const rules = new Intl.PluralRules(lang);
  return (key, params = {}) => {
    const msg = table[key];
    if (msg === undefined) throw new Error(`no catalog key ${key}`);
    const text = typeof msg === "string" ? msg : (msg[rules.select(Number(params.n ?? 0))] ?? msg.other);
    return text.replace(/\{(\w+)\}/g, (all, name) => (name in params ? String(params[name]) : all));
  };
}

/** A catalog text as a regex source, its {params} as `.+?` (for loose checks). */
export const pattern = (text) => text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&").replace(/\\\{\w+\\\}/g, ".+?");
