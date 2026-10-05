// Basic stats (mission #574). Slice 11 owns this file: unit win and pick
// rates, champion history, the discovered fusions (read from slice 10's
// store) and players' records, served as GET /stats. mvpRuntime wires
// statsHooks() once, so its hooks see bot runs and HTTP runs alike.
//
// The rates are counted as runs go, into the store's unit tallies (per
// content version, so a day's new numbers start fresh), never by scanning
// battles per request:
// - win rate: of the fights a unit's team fought (rounds and the Crown), the
//   share it won; a draw is a fight, not a win; a walkover (either line
//   empty: no one to fight) is no fight;
// - pick rate: of the finished runs, the share that ended with it on the line.
// A fused unit counts for both its parts. Records (slays, days as champion,
// playoff wins) are each player's Rating, which slices 4 and 5 write; the
// client reads them from GET /home.
import type { BattleRecord, LineUnit, StatsView, UnitId } from "../../../src/mvp/contract.js";
import type { RunDeps, RunHooks } from "./runs.js";
import type { UnitTally } from "./store.js";

/** The unit ids a line holds, each once; a fused unit gives both parts. */
export function lineUnitIds(line: LineUnit[]): UnitId[] {
  const ids = new Set<UnitId>();
  for (const u of line) {
    if (u.kind === "fused" && u.fusion) {
      ids.add(u.fusion.first);
      ids.add(u.fusion.second);
    } else ids.add(u.unitId);
  }
  return [...ids];
}

/** A fight where a side had no units: it won or lost without a strike, so it
 * says nothing about the units (fight.ts's walkover). */
export function isWalkover(b: Pick<BattleRecord, "teamA" | "teamB">): boolean {
  return b.teamA.length === 0 || b.teamB.length === 0;
}

/** Slice 11's observers of the run engine: one tally write per fight and per finished run. */
export function statsHooks(rt: Pick<RunDeps, "store" | "content" | "now">): RunHooks {
  return {
    onFight(run, fight, battle) {
      if (isWalkover(battle)) return;
      const won = fight.outcome === "win" ? 1 : 0;
      rt.store.addUnitTallies(run.contentVersion, { runs: 0, units: lineUnitIds(run.line).map((unitId) => ({ unitId, fights: 1, wins: won, runs: 0 })) });
    },
    onRunEnd(run) {
      rt.store.addUnitTallies(run.contentVersion, { runs: 1, units: lineUnitIds(run.line).map((unitId) => ({ unitId, fights: 0, wins: 0, runs: 1 })) });
    },
  };
}

/** GET /stats: the live content's unit rates (most picked first), every
 * day's champion (oldest first) and the discovered fusions. */
export function statsView(rt: RunDeps): StatsView {
  const t = rt.store.unitTallies(rt.content.version);
  const live = new Set(rt.content.units.map((u) => u.id));
  const units = t.units
    .filter((u: UnitTally) => live.has(u.unitId))
    .map((u) => ({ unitId: u.unitId, winRate: u.fights ? u.wins / u.fights : 0, pickRate: t.runs ? u.runs / t.runs : 0, runs: u.runs }))
    .sort((a, b) => b.pickRate - a.pickRate || b.winRate - a.winRate || a.unitId.localeCompare(b.unitId));
  return { units, champions: rt.store.champions(), fusions: rt.store.fusions() };
}
