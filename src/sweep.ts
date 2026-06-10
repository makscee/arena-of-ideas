// Seed sweep — the same matchup across a seed range, aggregated into a
// win-rate distribution. A matchup is a distribution, not a memorizable
// result (SPEC §3); this module is the one place that distribution is
// computed, shared by the CLI's --sweep mode and the web gauntlet.
//
// sweepSeeds() is a generator so an interactive caller can drain it in
// chunks (keeping a UI responsive) without re-implementing any of the
// outcome bookkeeping; sweep() drains it in one go for batch callers.

import { battle, winnerOf } from "./battle.js";
import type { BattleInput, Side } from "./types.js";

/** A sweep runs one matchup at many seeds — everything of BattleInput but the seed. */
export type SweepInput = Omit<BattleInput, "seed">;

/** One battle's contribution to the distribution. */
export interface SweepOutcome {
  seed: number;
  winner: Side | "draw";
  turns: number;
}

export interface SweepStats {
  n: number;
  aWins: number;
  bWins: number;
  draws: number;
  totalTurns: number;
}

/** The aggregate plus every per-seed outcome, so a consumer can pick an
 * example seed for any result class (watch a win / a loss / a draw). */
export interface SweepResult extends SweepStats {
  outcomes: SweepOutcome[];
}

/** Run one seed of a sweep: winner + the turn the battle was decided on. */
export function sweepOutcome(input: SweepInput, seed: number): SweepOutcome {
  const log = battle({ ...input, seed });
  const end = log[log.length - 1];
  const turns = end && end.type === "BattleEnd" ? end.turns : 0;
  return { seed, winner: winnerOf(log), turns };
}

/** Yield outcomes for seeds startSeed .. startSeed+n−1, in order. */
export function* sweepSeeds(input: SweepInput, n: number, startSeed = 0): Generator<SweepOutcome> {
  for (let i = 0; i < n; i++) yield sweepOutcome(input, startSeed + i);
}

/** Fold outcomes into the distribution stats. */
export function summarizeSweep(outcomes: readonly SweepOutcome[]): SweepStats {
  const stats: SweepStats = { n: outcomes.length, aWins: 0, bWins: 0, draws: 0, totalTurns: 0 };
  for (const o of outcomes) {
    if (o.winner === "A") stats.aWins++;
    else if (o.winner === "B") stats.bWins++;
    else stats.draws++;
    stats.totalTurns += o.turns;
  }
  return stats;
}

/** Run the full sweep at once (CLI / batch path). */
export function sweep(input: SweepInput, n: number, startSeed = 0): SweepResult {
  const outcomes = [...sweepSeeds(input, n, startSeed)];
  return { ...summarizeSweep(outcomes), outcomes };
}
