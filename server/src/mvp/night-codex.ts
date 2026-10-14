// The night batch into the world (makscee/void-board#1050, reopened): what
// ./night-batch.ts computes goes through here into #1049's Codex tables and
// #1052's word queue, and tonight's list reads the Codex back.
//
//   listInputFrom   the wishes of today and yesterday weight the list (a
//                   pair wished in the forge comes ahead of unwished ones);
//                   the Codex's pairs are the history: a claimed pair never
//                   again, one designed this week waits (pickPairs).
//   storeNight      a pair with ready candidates becomes ready to claim, its
//                   candidates as FusionDesigns (EN/RU names and flavour, the
//                   compiled rule as the recipe, the coined registry entries);
//                   a claimed pair is final and left alone. Candidates that
//                   wait for a word go to the word queue (queueNightResults);
//                   rejected ones stay in the batch's JSON and report only.
import type { FusionDesign } from "../../../src/mvp/codex.js";
import { codexPair, pairState } from "../../../src/mvp/codex.js";
import type { UnitForm } from "../../../src/mvp/contract.js";
import type { Candidate, ListInput, PairHistory, PairResult } from "./night-batch.js";
import type { MvpStore } from "./store.js";
import { queueNightResults } from "./words.js";

/** The compiled unit's rule as a form (the inverse of fight.ts toBattleDef). */
export function recipeOf(c: Candidate): UnitForm {
  const u = c.unit;
  return {
    when: structuredClone(u.triggers ?? []),
    who: structuredClone(u.selectors ?? []),
    does: [...(u.abilities ?? [])],
    ...(u.condition ? { condition: structuredClone(u.condition) } : {}),
    ...(u.also?.length ? { also: u.also.map((x) => ({ who: structuredClone(x.selectors ?? []), does: [...(x.abilities ?? [])] })) } : {}),
    // no stored sentence: the card renders the rule in the reader's language with today's words (#1047)
  };
}

/** A ready candidate as the Codex stores it. */
export function designOf(c: Candidate, emoji?: string): FusionDesign {
  const d: FusionDesign = {
    name: c.name.en,
    lean: c.lean === "A" ? "a" : c.lean === "B" ? "b" : "twist",
    recipe: recipeOf(c),
    texts: { ru: { name: c.name.ru, ...(c.flavour.ru ? { flavour: c.flavour.ru } : {}) } },
  };
  if (emoji) d.emoji = emoji;
  if (c.flavour.en) d.flavour = c.flavour.en;
  if (Object.keys(c.abilities).length) d.abilities = structuredClone(c.abilities);
  if (Object.keys(c.statuses).length) d.statuses = structuredClone(c.statuses);
  return d;
}

export interface StoredNight {
  /** Pairs now ready to claim. */
  ready: string[];
  /** Pairs left alone: already claimed (final). */
  claimed: string[];
  /** Word requests queued. */
  words: number;
}

/** The batch's results into the store. `emoji(a, b)` gives a pair's emoji, if any. */
export function storeNight(store: MvpStore, results: PairResult[], now: Date, emoji: (a: string, b: string) => string | undefined = () => undefined): StoredNight {
  const out: StoredNight = { ready: [], claimed: [], words: 0 };
  for (const r of results) {
    const ready = r.candidates.filter((c) => c.state === "ready");
    if (ready.length === 0) continue;
    const pair = codexPair(r.pair.a, r.pair.b);
    if (pairState(store.codexPair(pair.a, pair.b)) === "discovered") {
      out.claimed.push(r.pair.key);
      continue;
    }
    store.putCodexPair({ ...pair, candidates: ready.map((c) => designOf(c, emoji(pair.a, pair.b))), preparedAt: now.toISOString() });
    out.ready.push(r.pair.key);
  }
  out.words = queueNightResults(store, results, now);
  return out;
}

/** Tonight's list input from the world: wishes (today's and yesterday's day)
 * and the Codex's pairs as history. */
export function listInputFrom(store: MvpStore, units: ListInput["units"], now: number): Pick<ListInput, "wishes" | "history"> {
  const seq = store.currentDay()?.seq;
  const wishes: Record<string, number> = {};
  for (const s of seq === undefined ? [] : [seq, seq - 1]) for (const w of store.wishes(s)) wishes[`${w.a}+${w.b}`] = (wishes[`${w.a}+${w.b}`] ?? 0) + w.count;
  const history: Record<string, PairHistory> = {};
  const ids = new Set(units.map((u) => u.id));
  for (const p of store.codexPairs()) {
    if (!ids.has(p.a) || !ids.has(p.b)) continue;
    history[`${p.a}+${p.b}`] = { designedAt: p.preparedAt ? Date.parse(p.preparedAt) : now, ...(p.claim ? { claimed: true } : {}) };
  }
  return { wishes, history };
}
