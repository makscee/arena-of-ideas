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
import type { BattleRecord, Decision, LineUnit, StatsView, UnitId } from "../../../src/mvp/contract.js";
import type { MvpRunState } from "../../../src/mvp/run.js";
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

/** The unit a decision took into the run: a shop buy or a gift pick. */
export function pickedUnit(before: Pick<MvpRunState, "offers" | "gift">, d: Decision): UnitId | undefined {
  if (d.kind === "buy") return before.offers.find((o) => o.slot === d.slot)?.unitId;
  if (d.kind === "gift" && d.pick !== null) return before.gift?.[d.pick];
  return undefined;
}

/** Slice 11's observers of the run engine: one tally write per fight and per
 * finished run, per content version and (M2-1) per day; per day also each pick. */
export function statsHooks(rt: Pick<RunDeps, "store" | "content" | "now">): RunHooks {
  // The stored day, never rolled over here: a fight just past 04:00, before
  // anything ends the day, counts for the day it was fought in.
  const daySeq = () => rt.store.currentDay()?.seq ?? 1;
  return {
    onDecision(before, d) {
      const unitId = pickedUnit(before, d);
      if (unitId) rt.store.addDayTallies(daySeq(), { runs: 0, units: [{ unitId, fights: 0, wins: 0, runs: 0, picks: 1 }] });
    },
    onFight(run, fight, battle) {
      if (isWalkover(battle)) return;
      const won = fight.outcome === "win" ? 1 : 0;
      const ids = lineUnitIds(run.line);
      rt.store.addUnitTallies(run.contentVersion, { runs: 0, units: ids.map((unitId) => ({ unitId, fights: 1, wins: won, runs: 0 })) });
      rt.store.addDayTallies(daySeq(), { runs: 0, units: ids.map((unitId) => ({ unitId, fights: 1, wins: won, runs: 0, picks: 0 })) });
    },
    onRunEnd(run) {
      const ids = lineUnitIds(run.line);
      rt.store.addUnitTallies(run.contentVersion, { runs: 1, units: ids.map((unitId) => ({ unitId, fights: 0, wins: 0, runs: 1 })) });
      rt.store.addDayTallies(daySeq(), { runs: 1, units: ids.map((unitId) => ({ unitId, fights: 0, wins: 0, runs: 1, picks: 0 })) });
    },
  };
}

/** GET /stats: the live content's unit rates (most picked first), every
 * day's champion up to today (oldest first) and the discovered fusions. A day
 * end that failed after crowning leaves tomorrow's champion stored early (a
 * slayer's team, hidden until the day ends); like /day and the Crown, it
 * isn't listed until its day starts. */
export function statsView(rt: RunDeps): StatsView {
  const t = rt.store.unitTallies(rt.content.version);
  const live = new Set(rt.content.units.map((u) => u.id));
  const units = t.units
    .filter((u: UnitTally) => live.has(u.unitId))
    .map((u) => ({ unitId: u.unitId, winRate: u.fights ? u.wins / u.fights : 0, pickRate: t.runs ? u.runs / t.runs : 0, runs: u.runs }))
    .sort((a, b) => b.pickRate - a.pickRate || b.winRate - a.winRate || a.unitId.localeCompare(b.unitId));
  const seq = rt.today().seq;
  return { units, champions: rt.store.champions().filter((c) => c.seq <= seq), fusions: rt.store.fusions() };
}
