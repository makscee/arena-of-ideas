// What the table shows the moment a decision is pressed, before the server answers
// (arena#28 reopened): the shop's own rules (src/mvp/run.ts, the same kernel the server
// runs) applied to the run as drawn. A buy, a sell, a move or a lock is exact; anything
// that needs the server's dice (a reroll, a gift an awakening would bring), a fusion or a
// fight is not predicted, and the caller shows a pressed state instead.
import type { Decision, MvpContent, MvpRules, RunView } from "../../src/mvp/contract";
import { applyMvpDecision, runView, type MvpRunState } from "../../src/mvp/run";

/** Uids for units that exist only until the answer arrives; the server's replace them. */
let pending = 1_000_000;

export function predictRun(run: RunView, d: Decision, content: MvpContent, rules: MvpRules): RunView | null {
  if (d.kind !== "buy" && d.kind !== "sell" && d.kind !== "reorder" && d.kind !== "lock") return null;
  try {
    const state: MvpRunState = { ...structuredClone(run), opponent: null, crownSeq: null, seed: 0, rng: 0, nextUid: pending, rules };
    const next = applyMvpDecision(state, d, content).state;
    // The seeded stream is the server's: an awakening's gift is rolled there.
    if (next.gift && !run.gift) return null;
    pending = next.nextUid;
    return runView(next);
  } catch {
    // Refused (no gold, a full line): the server says why, as it does today.
    return null;
  }
}

/** The unit a buy would change (the twin it merges into, or the new one), as the shop's own rules leave it: an
 * awakening tag reads its before and after numbers off it. The gift the awakening brings is the server's dice and
 * is ignored here. Null when the kernel refuses the buy. */
export function boughtUnit(run: RunView, slot: number, uid: string | null, content: MvpContent, rules: MvpRules): RunView["line"][number] | null {
  try {
    const state: MvpRunState = { ...structuredClone(run), opponent: null, crownSeq: null, seed: 0, rng: 0, nextUid: pending, rules };
    const next = applyMvpDecision(state, { kind: "buy", slot }, content).state;
    const all = [...next.line, ...next.bench];
    const known = new Set([...run.line, ...run.bench].map((u) => u.uid));
    return all.find((u) => (uid === null ? !known.has(u.uid) : u.uid === uid)) ?? null;
  } catch {
    return null;
  }
}
