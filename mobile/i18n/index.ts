// The client's words (mission #810, M4-1): every player-facing string lives in
// a catalog (en.ts), keyed, with {params} and plurals, so a second language
// is a second table. t("ideas.held", { n: 2 }) reads "2 ideas".
import { en, type Key } from "./en";

/** A message with plural forms, picked by Intl.PluralRules on params.n:
 * English uses one / other; Russian adds few / many (1 идея, 2 идеи, 5 идей). */
export type Plural = { zero?: string; one: string; two?: string; few?: string; many?: string; other: string };
export type Msg = string | Plural;
export type Params = Record<string, string | number>;

let lang = "en";
let table: Record<string, Msg> = en;
let rules = new Intl.PluralRules(lang);

/** Switch the catalog (M4-2 adds Russian). Missing keys fall back to English. */
export function setLang(code: string, messages: Partial<Record<Key, Msg>>): void {
  lang = code;
  table = { ...en, ...messages } as Record<string, Msg>;
  rules = new Intl.PluralRules(code);
}

export function currentLang(): string {
  return lang;
}

/** The message for `key`, its {name} params filled in. A plural message picks
 * its form by `params.n`. */
export function t(key: Key, params: Params = {}): string {
  const msg = table[key] ?? en[key] ?? key;
  const text = typeof msg === "string" ? msg : pluralForm(msg, Number(params.n ?? 0));
  return text.replace(/\{(\w+)\}/g, (all, name: string) => (name in params ? String(params[name]) : all));
}

function pluralForm(msg: Plural, n: number): string {
  return msg[rules.select(n) as keyof Plural] ?? msg.other;
}

/** A catalog sentence with elements in it: "{who} stays champion" with an
 * @name element at {who}. Splits the text on each {name} in `nodes`. */
export function withNodes(text: string, nodes: Record<string, Node>): (Node | string)[] {
  return text.split(/\{(\w+)\}/).map((part, i) => (i % 2 ? (nodes[part] ?? `{${part}}`) : part)).filter((x) => x !== "");
}
