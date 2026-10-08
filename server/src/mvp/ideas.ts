// Earning ideas (mission 2, M2-3, makscee/void-board#787): 1 idea per
// MvpRules.ideaEveryRuns finished runs, held up to ideaHold. Counted from what
// the store already knows, the finished runs on the rating row (Rating.runs:
// every ending a rating counts, given up included), plus the player's
// IdeaCounts: nothing to backfill for players who played before ideas.
//
//   earned = floor(runs / every) + granted
//   held   = earned − spent − forfeited, at most `hold`
//
// Ideas earned past the cap are forfeited, written down the next time the
// counts are read (settle), so a spent idea doesn't bring back the runs
// played while full. Earning only grows between reads and every write
// settles first, so settling late gives what settling at each run would.
import type { IdeasView, MvpRules } from "../../../src/mvp/contract.js";
import type { RunDeps } from "./runs.js";
import type { IdeaCounts } from "./store.js";

type IdeaDeps = Pick<RunDeps, "store" | "rules">;

/** The idea tunables, defaults filled in. */
export function ideaRules(rules: Pick<MvpRules, "ideaEveryRuns" | "ideaHold">): { every: number; hold: number } {
  return { every: Math.max(1, rules.ideaEveryRuns ?? 3), hold: Math.max(0, rules.ideaHold ?? 3) };
}

/** Reads the player's counts, forfeits ideas earned past the cap (writing
 * only when that changed something), and returns them with the ideas held. */
function settle(deps: IdeaDeps, playerId: string): { counts: IdeaCounts; runs: number; held: number } {
  const { every, hold } = ideaRules(deps.rules);
  const runs = deps.store.rating(playerId)?.runs ?? 0;
  const counts = deps.store.ideaCounts(playerId);
  const raw = Math.floor(runs / every) + counts.granted - counts.spent - counts.forfeited;
  if (raw > hold) {
    counts.forfeited += raw - hold;
    deps.store.putIdeaCounts(playerId, counts);
  }
  // A rate raised later can earn less than was spent: none held, not negative.
  return { counts, runs, held: Math.max(0, Math.min(raw, hold)) };
}

function view(deps: IdeaDeps, s: { runs: number; held: number }): IdeasView {
  const { every, hold } = ideaRules(deps.rules);
  return { held: s.held, nextIn: s.held >= hold ? null : every - (s.runs % every) };
}

/** The player's ideas, for Home. */
export function ideasOf(deps: IdeaDeps, playerId: string): IdeasView {
  return view(deps, settle(deps, playerId));
}

/** Dev "+1 idea": one more, unless the player already holds the cap. */
export function grantIdea(deps: IdeaDeps, playerId: string): IdeasView {
  const s = settle(deps, playerId);
  if (s.held < ideaRules(deps.rules).hold) {
    s.counts.granted++;
    deps.store.putIdeaCounts(playerId, s.counts);
    s.held++;
  }
  return view(deps, s);
}

/** Spends one idea (M2-4 calls it when an idea is written): false when the
 * player holds none. */
export function spendIdea(deps: IdeaDeps, playerId: string): boolean {
  const s = settle(deps, playerId);
  if (s.held < 1) return false;
  s.counts.spent++;
  deps.store.putIdeaCounts(playerId, s.counts);
  return true;
}
