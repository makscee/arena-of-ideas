// Sim gate — the empirical balance check a creation candidate must clear.
//
// The contract of the creation loop is the *checks*, not any prompt (vision:
// "the contract is the checks, not the prompt"). This module is the balance
// half of that contract: a candidate team is sound only if, swept across many
// seeds against a fixed reference meta, its overall win-rate lands inside a
// configured band. Too low → it's filler; too high → it's overtuned (the
// "deal 999 damage every turn" failure mode). The band is empirical and lives
// in config (a tunable), never as prose in a README — slices 2/3 read the
// numbers this produces and feed them back to an AI loop, so the gate is the
// single source of truth for "balanced".
//
// Pure and deterministic, like the kernel it sits in front of: same candidate
// + same gate config → byte-identical report (the sweeps are seeded). It only
// reads content the validator already passed; it never touches battle().

import { sweep } from "./sweep.js";
import type { SweepStats } from "./sweep.js";
import type { AbilityRegistry, StatusRegistry, UnitDef } from "./types.js";

/** A named opponent in the reference meta — the bar a candidate is measured against. */
export interface ReferenceTeam {
  name: string;
  units: UnitDef[];
}

/** The gate's tunable knobs. The band is [min, max] win-rate (0..1) the
 * candidate must land in across the whole reference meta; seeds is how many
 * seeds each matchup sweeps. A candidate at exactly the band edge passes
 * (inclusive) — the edges are the design intent, not a no-man's-land.
 *
 * The pooled band alone is gameable: a candidate can hard-counter one
 * reference team to 100% and fold to another at 0%, yet sit in the pooled band
 * on average — exactly the "beat one shape, fold to another" line an AI
 * magnitude-tuner steers into. The per-matchup `floor` closes that hole: every
 * matchup must clear it, so an in-band pool means broadly playable, not lucky
 * on the average. */
export interface GateConfig {
  /** Inclusive lower bound on overall win-rate (0..1). Below → underpowered. */
  bandMin: number;
  /** Inclusive upper bound on overall win-rate (0..1). Above → overtuned. */
  bandMax: number;
  /** Inclusive per-matchup floor (0..1): every matchup's win-rate must be at
   * least this, else the verdict is "counter-folded" even when the pool is
   * in-band. Closes the gameable-pool hole. */
  floor: number;
  /** Seeds swept per matchup. Larger N tightens the win-rate estimate. */
  seeds: number;
}

/** One matchup's outcome in the gate report — the candidate vs one reference team. */
export interface MatchupResult {
  opponent: string;
  /** Seeds in which the candidate (side A) won. */
  wins: number;
  losses: number;
  draws: number;
  seeds: number;
  /** Candidate win-rate in this matchup (0..1), draws counted as non-wins. */
  winRate: number;
}

/** The full, machine-readable gate report. `pass` is the gate verdict; the
 * numbers are what a bounce feeds back to the creation loop. */
export interface GateReport {
  pass: boolean;
  /** The reason, when failing. "counter-folded" = pool was in-band but at
   * least one matchup fell below the per-matchup floor. */
  verdict: "in-band" | "underpowered" | "overtuned" | "counter-folded";
  /** Candidate win-rate across every matchup pooled together (0..1). */
  overallWinRate: number;
  totalSeeds: number;
  totalWins: number;
  totalLosses: number;
  totalDraws: number;
  band: { min: number; max: number };
  /** The per-matchup floor every matchup must clear (0..1). */
  floor: number;
  /** Opponent names whose win-rate fell below the floor (empty when none). */
  foldedTo: string[];
  matchups: MatchupResult[];
}

/** Fold one sweep (candidate as side A) into a matchup result. */
function toMatchup(opponent: string, stats: SweepStats): MatchupResult {
  return {
    opponent,
    wins: stats.aWins,
    losses: stats.bWins,
    draws: stats.draws,
    seeds: stats.n,
    winRate: stats.n === 0 ? 0 : stats.aWins / stats.n,
  };
}

/**
 * Run the gate: sweep the candidate (as side A) against every reference team
 * over `config.seeds` seeds each, pool the outcomes into an overall win-rate,
 * and judge it against the band. Seeds run 0..seeds−1 per matchup, so the
 * report is fully determined by (candidate, meta, config).
 */
export function runGate(
  candidate: UnitDef[],
  meta: readonly ReferenceTeam[],
  config: GateConfig,
  statuses: StatusRegistry,
  abilities: AbilityRegistry,
): GateReport {
  const matchups = meta.map((ref) =>
    toMatchup(ref.name, sweep({ teamA: candidate, teamB: ref.units, statuses, abilities }, config.seeds)),
  );
  let totalWins = 0;
  let totalLosses = 0;
  let totalDraws = 0;
  for (const m of matchups) {
    totalWins += m.wins;
    totalLosses += m.losses;
    totalDraws += m.draws;
  }
  const totalSeeds = totalWins + totalLosses + totalDraws;
  const overallWinRate = totalSeeds === 0 ? 0 : totalWins / totalSeeds;

  // Matchups that fall below the per-matchup floor — the gameable-pool fold.
  const foldedTo = matchups.filter((m) => m.winRate < config.floor).map((m) => m.opponent);

  // Pooled verdicts are the broad signal and take precedence: too weak or too
  // strong overall is a magnitude problem before it is a coverage problem. Only
  // when the pool sits in-band does the per-matchup floor get the final word —
  // an otherwise-balanced candidate that hard-counters one shape and folds to
  // another is "counter-folded", and the loop is told exactly which opponents.
  let verdict: GateReport["verdict"];
  if (overallWinRate < config.bandMin) verdict = "underpowered";
  else if (overallWinRate > config.bandMax) verdict = "overtuned";
  else if (foldedTo.length > 0) verdict = "counter-folded";
  else verdict = "in-band";

  return {
    pass: verdict === "in-band",
    verdict,
    overallWinRate,
    totalSeeds,
    totalWins,
    totalLosses,
    totalDraws,
    band: { min: config.bandMin, max: config.bandMax },
    floor: config.floor,
    foldedTo,
    matchups,
  };
}

/** Human-readable gate transcript — the same numbers the JSON carries, for a
 * person (or a log) to read at a glance. The machine consumer reads the
 * GateReport object; this is its rendering, kept in lockstep. */
export function formatGateReport(report: GateReport): string {
  const pct = (x: number) => (x * 100).toFixed(1) + "%";
  const lines: string[] = [];
  lines.push(`Sim gate: ${report.pass ? "PASS" : "FAIL"} (${report.verdict})`);
  lines.push(
    `  overall win-rate: ${pct(report.overallWinRate)} ` +
      `(band ${pct(report.band.min)}–${pct(report.band.max)}, ${report.totalSeeds} battles)`,
  );
  lines.push(
    `  totals: ${report.totalWins}W / ${report.totalLosses}L / ${report.totalDraws}D`,
  );
  lines.push(`  per-matchup floor: ${pct(report.floor)}`);
  if (report.foldedTo.length > 0) {
    lines.push(`  folded below floor: ${report.foldedTo.join(", ")}`);
  }
  lines.push(`  matchups:`);
  for (const m of report.matchups) {
    const flag = m.winRate < report.floor ? "  ← below floor" : "";
    lines.push(
      `    vs ${m.opponent}: ${pct(m.winRate)} (${m.wins}W/${m.losses}L/${m.draws}D over ${m.seeds} seeds)${flag}`,
    );
  }
  return lines.join("\n");
}
