// The word forge (rules v2 §7, makscee/void-board#1052): one queue of requests
// for new words from three sources, the forged list, and the Dictionary.
//
//   request   a night design, a unit batch or an idea reading asks for a word
//             instead of bending the design (requestWord). A pair that asked
//             shows sealed in the Codex: "waiting for a new word: steal".
//   forge     a word card on the board; a worker adds the word to src/words/
//             (docs/words/CARD.md) and lists it in registry/forged-words.json.
//   found     the first claimed design that uses it (foundWordsIn), credited
//             to the claimer and the pair.
//
// A word's state is derived, never stored twice: found (a row), else forged
// (the committed list), else requested (some request names it).

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { FoundWord, MvpStore, WordRequest, WordSource } from "./store.js";

export const WORD_SOURCES: readonly WordSource[] = ["idea", "night", "batch"];

/** A word as the queue knows it: trimmed, lower-cased, one space between parts. */
export const wordKey = (word: string): string => word.trim().toLowerCase().replace(/\s+/g, " ");

/** A pair is unordered: A+B and B+A are one pair (design §6). */
export const pairKey = (a: string, b: string): string => (a <= b ? `${a}+${b}` : `${b}+${a}`);

export interface ForgedWord {
  /** The word as requested ("steal"). */
  word: string;
  /** Its kind in the registry (src/words/<kind>.ts). */
  kind: string;
  /** The word card (an issue number in makscee/void-board). */
  card: number;
}

const FORGED_FILE = fileURLToPath(new URL("../../../registry/forged-words.json", import.meta.url));

/** The words forged so far, from the committed list (empty when there is none). */
export function forgedWords(file = FORGED_FILE): ForgedWord[] {
  try {
    return JSON.parse(readFileSync(file, "utf8")) as ForgedWord[];
  } catch {
    return [];
  }
}

/** What a source asks for: the word, and the part of the design or idea that needed it. */
export interface WordAsk {
  word: string;
  part: string;
}

/** Puts a request in the queue. The same source asking twice for the same word
 * for the same thing is one request. Returns whether it was new. */
export function requestWord(store: MvpStore, source: WordSource, ref: string, ask: WordAsk, now: Date): boolean {
  const word = ask.word.trim().slice(0, 60);
  if (!word || !ref) return false;
  const key = wordKey(word);
  if (store.wordRequests().some((r) => r.source === source && r.ref === ref && wordKey(r.word) === key)) return false;
  store.addWordRequest({ source, ref, word, part: ask.part.trim().slice(0, 200), createdAt: now.toISOString() });
  return true;
}

/** The words a night design (pair A+B) asks for: a design that asks for a word
 * is sealed until the word exists. */
export const requestForPair = (store: MvpStore, a: string, b: string, asks: WordAsk[], now: Date): number =>
  asks.filter((ask) => requestWord(store, "night", pairKey(a, b), ask, now)).length;

/** The words a unit of a batch asks for. */
export const requestForUnit = (store: MvpStore, unitId: string, asks: WordAsk[], now: Date): number =>
  asks.filter((ask) => requestWord(store, "batch", unitId, ask, now)).length;

export type WordState = "requested" | "forged" | "found";

export interface DictionaryEntry {
  word: string;
  state: WordState;
  /** The registry kind, once forged. */
  kind?: string;
  card?: number;
  /** The requests behind it, oldest first. */
  requests: WordRequest[];
  /** Who found it, once found. */
  found?: FoundWord;
}

/** Every word anyone asked for or forged, oldest first, with its state. */
export function dictionary(store: MvpStore, forged: ForgedWord[] = forgedWords()): DictionaryEntry[] {
  const found = new Map(store.foundWords().map((f) => [f.key, f]));
  const out = new Map<string, DictionaryEntry>();
  const entry = (word: string): DictionaryEntry => {
    const key = wordKey(word);
    let e = out.get(key);
    if (!e) out.set(key, (e = { word, state: "requested", requests: [] }));
    return e;
  };
  for (const r of store.wordRequests()) entry(r.word).requests.push(r);
  for (const f of forged) Object.assign(entry(f.word), { state: "forged" as const, kind: f.kind, card: f.card });
  for (const e of out.values()) {
    const f = found.get(wordKey(e.word));
    if (f) Object.assign(e, { state: "found" as const, found: f });
  }
  return [...out.values()];
}

/** "3 words forged, not yet found" (the Dictionary's line until they are). */
export const forgedNotFound = (entries: DictionaryEntry[]): number => entries.filter((e) => e.state === "forged").length;

/** The words still waited on (requested, not yet forged) by one pair: a pair with any is sealed. */
export function waitingOn(store: MvpStore, pair: string, entries: DictionaryEntry[] = dictionary(store)): string[] {
  return entries.filter((e) => e.state === "requested" && e.requests.some((r) => r.source === "night" && r.ref === pair)).map((e) => e.word);
}

/** The sealed pair's line in the Codex (design §7). */
export const sealedLine = (words: string[], lang: "en" | "ru" = "en"): string =>
  lang === "ru" ? `ждёт нового слова: ${words.join(", ")}` : `waiting for a new word: ${words.join(", ")}`;

/** The Dictionary's credit for a found word (design §7): "steal, found in
 * Magpie + Shadow by @kim". `name` is the claimer's display name. */
export const creditLine = (f: FoundWord, name: string, lang: "en" | "ru" = "en"): string => {
  const pair = f.pair.split("+").join(" + ");
  return lang === "ru" ? `${f.word}: найдено в ${pair}, @${name}` : `${f.word}, found in ${pair} by @${name}`;
};

/** The Dictionary's line while forged words wait for a claim: "3 words forged, not yet found". */
export const notFoundLine = (n: number, lang: "en" | "ru" = "en"): string =>
  lang === "ru" ? `выковано слов, ещё не найдено: ${n}` : `${n} word${n === 1 ? "" : "s"} forged, not yet found`;

/** The forged words a design uses: its JSON names the word's kind (`"kind":"unhurt"`). */
export function forgedWordsIn(design: unknown, forged: ForgedWord[] = forgedWords()): ForgedWord[] {
  const json = JSON.stringify(design);
  return forged.filter((f) => json.includes(`"kind":"${f.kind}"`));
}

/** A claim of `pair` by `playerId` with this design: every forged word it uses
 * and nobody has found yet is found now, credited to the claimer. Returns the
 * words found by this claim. */
export function foundWordsIn(store: MvpStore, design: unknown, pair: string, playerId: string, now: Date, forged: ForgedWord[] = forgedWords()): ForgedWord[] {
  return forgedWordsIn(design, forged).filter((f) => store.addFoundWord({ key: wordKey(f.word), word: f.word, kind: f.kind, pair, playerId, foundAt: now.toISOString() }));
}
