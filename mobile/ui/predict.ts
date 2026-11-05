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
