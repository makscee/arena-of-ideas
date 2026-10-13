// The Codex as one view (rules v2 §6, §7; slice makscee/void-board#1051):
// GET /codex builds what the Codex screens draw (src/mvp/forge-view.ts) from
// the world: every live unit, every stored pair in its state with today's
// wishes and the words it waits for, the daily feed (what each night's batch
// made ready, who claimed what, which words were found) and the Dictionary
// (#1052's dictionary(): found words with credit and lineage, the forged ones
// not found yet). Pairs nobody stored are unforged; the client fills them in.
import { canonicalDesign, pairKey, pairState, type CodexPair } from "../../../src/mvp/codex.js";
import type { PlayerRef, UnitContent } from "../../../src/mvp/contract.js";
import { forgeDesignOf, forgeUnitOf, type FeedItem, type ForgePair, type ForgeView, type WordEntry } from "../../../src/mvp/forge-view.js";
import type { RunDeps } from "./runs.js";
import { dictionary, forgedNotFound, forgedWords, forgedWordsIn, waitingOn, wordKey, type ForgedWord } from "./words.js";

/** How many days the feed goes back. */
export const FEED_DAYS = 7;

export function codexOverview(deps: RunDeps, viewer: PlayerRef | undefined, forged: ForgedWord[] = forgedWords()): ForgeView {
  const content = deps.content;
  const unitOf = (id: string): UnitContent | undefined => content.units.find((u) => u.id === id);
  const nameOf = (id: string): string => deps.store.player(id)?.name ?? "?";
  const seq = deps.today().seq;
  const wishes = new Map(deps.store.wishes(seq).map((w) => [pairKey(w.a, w.b), w.count]));
  const entries = dictionary(deps.store, forged);
  const found = new Map(deps.store.foundWords().map((f) => [f.pair + "|" + f.key, f]));
  const live = new Set(content.units.map((u) => u.id));
  const stored = deps.store.codexPairs();
  const design = (p: CodexPair) => {
    const d = canonicalDesign(p)!;
    const key = pairKey(p.a, p.b);
    const isNew = forgedWordsIn(d, forged).filter((w) => found.has(key + "|" + wordKey(w.word))).map((w) => w.word);
    return forgeDesignOf(d, [unitOf(p.a), unitOf(p.b)], content, isNew);
  };

  const pairs: ForgePair[] = [];
  const keys = new Set<string>();
  for (const p of stored) {
    if (!live.has(p.a) || !live.has(p.b)) continue;
    const key = pairKey(p.a, p.b);
    keys.add(key);
    const state = pairState(p);
    const row: ForgePair = { a: p.a, b: p.b, state };
    if (state === "discovered") Object.assign(row, { design: design(p), claimedBy: p.claim!.by.name, claimedAt: p.claim!.at });
    if (state === "unforged") pairs.push(fillUnforged(row));
    else pairs.push(row);
  }
  // Wished or word-waiting pairs nobody stored yet are unforged with a count.
  for (const key of wishes.keys()) if (!keys.has(key)) addUnforged(key);
  for (const e of entries)
    if (e.state === "requested")
      for (const r of e.requests) if (r.source === "night" && !keys.has(r.ref)) addUnforged(r.ref);
  function addUnforged(key: string): void {
    const [a, b] = key.split("+") as [string, string];
    if (!live.has(a) || !live.has(b)) return;
    keys.add(key);
    pairs.push(fillUnforged({ a, b, state: "unforged" }));
  }
  function fillUnforged(row: ForgePair): ForgePair {
    const key = pairKey(row.a, row.b);
    row.wishes = wishes.get(key) ?? 0;
    row.wished = viewer ? deps.store.wishedBy(row.a, row.b, viewer.id, seq) : false;
    const waiting = waitingOn(deps.store, key, entries);
    if (waiting.length) row.waitingWord = { en: waiting.join(", "), ru: waiting.join(", ") };
    return row;
  }

  // The feed: each night's batch (pairs prepared that day), each claim, each word found.
  const since = new Date(deps.now().getTime() - FEED_DAYS * 86_400_000).toISOString();
  const feed: FeedItem[] = [];
  const nights = new Map<string, { at: string; n: number }>();
  for (const p of stored) {
    if (!p.preparedAt || p.preparedAt < since || p.candidates.length === 0) continue;
    const day = p.preparedAt.slice(0, 10);
    const night = nights.get(day) ?? { at: p.preparedAt, n: 0 };
    night.n++;
    if (p.preparedAt < night.at) night.at = p.preparedAt;
    nights.set(day, night);
  }
  for (const { at, n } of nights.values()) feed.push({ kind: "arrived", at, n });
  for (const p of stored)
    if (p.claim && p.claim.at >= since && live.has(p.a) && live.has(p.b)) feed.push({ kind: "claimed", at: p.claim.at, pair: [p.a, p.b], name: design(p).name, by: p.claim.by.name });
  for (const f of deps.store.foundWords()) {
    if (f.foundAt < since) continue;
    const [a, b] = f.pair.split("+") as [string, string];
    feed.push({ kind: "word", at: f.foundAt, word: { en: f.word, ru: f.word }, ...(live.has(a) && live.has(b) ? { pair: [a, b] as [string, string] } : {}), by: nameOf(f.playerId) });
  }
  feed.sort((x, y) => (x.at < y.at ? 1 : x.at > y.at ? -1 : 0));

  // The Dictionary: found words with credit and lineage; the forged ones sealed.
  const claimed = stored.filter((p) => p.claim);
  const words: WordEntry[] = [];
  for (const e of entries) {
    if (e.state !== "found" || !e.found) continue;
    const [a, b] = e.found.pair.split("+") as [string, string];
    const home = claimed.find((p) => pairKey(p.a, p.b) === e.found!.pair);
    if (!home || !live.has(a) || !live.has(b)) continue;
    const usedIn = claimed.filter((p) => forgedWordsIn(canonicalDesign(p), forged).some((w) => wordKey(w.word) === wordKey(e.word))).map((p) => design(p).name);
    const means = e.requests[0]?.part ?? "";
    words.push({ word: { en: e.word, ru: e.word }, means: { en: means, ru: means }, kind: "forged", foundBy: nameOf(e.found.playerId), foundIn: { pair: [a, b], name: design(home).name }, foundAt: e.found.foundAt, usedIn });
  }
  const sealed = entries.filter((e) => e.state === "forged").map((e) => ({ en: e.word, ru: e.word }));
  return { viewer: viewer?.name ?? "", units: content.units.map((u) => forgeUnitOf(u, content)), pairs, feed, words, forgedNotFound: forgedNotFound(entries), forged: sealed };
}
