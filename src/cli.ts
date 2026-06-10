/**
 * Arena of Ideas — battle CLI (slice 6)
 *
 * Two modes:
 *
 * RUN MODE — play a single battle and print the full replay:
 *   node --import=tsx/esm src/cli.ts <teamA.json> <teamB.json> [--seed N]
 *   Default seed: 0 (deterministic).
 *
 * SWEEP MODE — run N seeds and report win-rate distribution:
 *   node --import=tsx/esm src/cli.ts <teamA.json> <teamB.json> --sweep N
 *
 * Team file format (JSON):
 *   {
 *     "units": UnitDef[]   // 1..5 units; see src/types.ts for UnitDef schema
 *   }
 *   Any extra keys (e.g. "_comment") are silently ignored.
 *   Statuses referenced in units[].statuses must be present in the stress
 *   registry; for custom status sets pass a registry via the API directly.
 *   The CLI always uses the stressRegistry exported from src/content/stress.ts.
 *
 * All logic below (except the top-level main() entrypoint) is exported for
 * unit testing without spawning a subprocess.
 */

import { readFileSync } from "node:fs";
import { battle, renderReplay, winnerOf } from "./index.js";
import { stressRegistry } from "./content/stress.js";
import type { UnitDef } from "./types.js";
import type { Side } from "./types.js";

// ---------------------------------------------------------------------------
// Team file loading
// ---------------------------------------------------------------------------

export interface TeamFile {
  units: UnitDef[];
}

export function loadTeamFile(path: string): TeamFile {
  let raw: string;
  try {
    raw = readFileSync(path, "utf8");
  } catch (err) {
    throw new Error(`Cannot read team file "${path}": ${(err as NodeJS.ErrnoException).message}`);
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (err) {
    throw new Error(`Team file "${path}" is not valid JSON: ${(err as Error).message}`);
  }
  return validateTeamFile(parsed, path);
}

export function validateTeamFile(data: unknown, label = "<input>"): TeamFile {
  if (typeof data !== "object" || data === null || Array.isArray(data)) {
    throw new Error(`Team file ${label}: expected a JSON object with a "units" array`);
  }
  const obj = data as Record<string, unknown>;
  if (!Array.isArray(obj["units"])) {
    throw new Error(`Team file ${label}: missing or non-array "units" field`);
  }
  const units = obj["units"] as unknown[];
  if (units.length === 0 || units.length > 5) {
    throw new Error(`Team file ${label}: "units" must have 1..5 entries, got ${units.length}`);
  }
  // Light structural check — the engine will throw on deeper schema errors.
  for (let i = 0; i < units.length; i++) {
    const u = units[i] as Record<string, unknown>;
    if (typeof u !== "object" || u === null) {
      throw new Error(`Team file ${label}: units[${i}] is not an object`);
    }
    if (typeof u["name"] !== "string") {
      throw new Error(`Team file ${label}: units[${i}].name must be a string`);
    }
    if (typeof u["base"] !== "object" || u["base"] === null) {
      throw new Error(`Team file ${label}: units[${i}].base must be an object`);
    }
  }
  return { units: units as UnitDef[] };
}

// ---------------------------------------------------------------------------
// Run mode
// ---------------------------------------------------------------------------

export interface RunResult {
  replay: string;
  winner: Side | "draw";
  turns: number;
}

export function runBattle(teamAPath: string, teamBPath: string, seed: number): RunResult {
  const a = loadTeamFile(teamAPath);
  const b = loadTeamFile(teamBPath);
  const log = battle({ teamA: a.units, teamB: b.units, seed, statuses: stressRegistry });
  const replay = renderReplay(log);
  const winner = winnerOf(log);
  const end = log[log.length - 1];
  const turns = end && end.type === "BattleEnd" ? end.turns : 0;
  return { replay, winner, turns };
}

// ---------------------------------------------------------------------------
// Sweep mode
// ---------------------------------------------------------------------------

export interface SweepStats {
  n: number;
  aWins: number;
  bWins: number;
  draws: number;
  totalTurns: number;
}

export function sweepBattles(teamAPath: string, teamBPath: string, n: number): SweepStats {
  const a = loadTeamFile(teamAPath);
  const b = loadTeamFile(teamBPath);
  let aWins = 0;
  let bWins = 0;
  let draws = 0;
  let totalTurns = 0;
  for (let seed = 0; seed < n; seed++) {
    const log = battle({ teamA: a.units, teamB: b.units, seed, statuses: stressRegistry });
    const w = winnerOf(log);
    if (w === "A") aWins++;
    else if (w === "B") bWins++;
    else draws++;
    const end = log[log.length - 1];
    if (end && end.type === "BattleEnd") totalTurns += end.turns;
  }
  return { n, aWins, bWins, draws, totalTurns };
}

export function formatSweepReport(stats: SweepStats, teamAPath: string, teamBPath: string): string {
  const { n, aWins, bWins, draws, totalTurns } = stats;
  const pct = (x: number) => ((x / n) * 100).toFixed(1);
  const avgTurns = (totalTurns / n).toFixed(1);
  const lines = [
    `Sweep: ${n} seeds (0..${n - 1})`,
    `  Team A (${teamAPath}): ${aWins} wins (${pct(aWins)}%)`,
    `  Team B (${teamBPath}): ${bWins} wins (${pct(bWins)}%)`,
    `  Draws:                ${draws} (${pct(draws)}%)`,
    `  Total battles:        ${n}  (${aWins} + ${bWins} + ${draws} = ${aWins + bWins + draws})`,
    `  Avg battle length:    ${avgTurns} turns`,
  ];
  return lines.join("\n");
}

// ---------------------------------------------------------------------------
// Arg parsing
// ---------------------------------------------------------------------------

export interface ParsedArgs {
  mode: "run" | "sweep";
  teamAPath: string;
  teamBPath: string;
  seed: number;     // run mode
  sweepN: number;   // sweep mode
}

export function parseArgs(argv: string[]): ParsedArgs {
  const args = argv.slice(2); // drop node + script

  if (args.length < 2) {
    throw new Error(
      "Usage:\n" +
        "  Run mode:   battle <teamA.json> <teamB.json> [--seed N]\n" +
        "  Sweep mode: battle <teamA.json> <teamB.json> --sweep N",
    );
  }

  const teamAPath = args[0]!;
  const teamBPath = args[1]!;
  const rest = args.slice(2);

  let seed = 0;
  let sweepN = -1;

  for (let i = 0; i < rest.length; i++) {
    if (rest[i] === "--seed") {
      const val = Number(rest[i + 1]);
      if (!Number.isFinite(val) || val < 0) throw new Error("--seed must be a non-negative integer");
      seed = Math.floor(val);
      i++;
    } else if (rest[i] === "--sweep") {
      const val = Number(rest[i + 1]);
      if (!Number.isFinite(val) || val < 1) throw new Error("--sweep N must be a positive integer");
      sweepN = Math.floor(val);
      i++;
    } else {
      throw new Error(`Unknown argument: ${rest[i]}`);
    }
  }

  if (sweepN > 0) {
    return { mode: "sweep", teamAPath, teamBPath, seed: 0, sweepN };
  }
  return { mode: "run", teamAPath, teamBPath, seed, sweepN: 0 };
}

// ---------------------------------------------------------------------------
// Entrypoint
// ---------------------------------------------------------------------------

function main(): void {
  let parsed: ParsedArgs;
  try {
    parsed = parseArgs(process.argv);
  } catch (err) {
    process.stderr.write((err as Error).message + "\n");
    process.exit(1);
  }

  if (parsed.mode === "run") {
    try {
      const result = runBattle(parsed.teamAPath, parsed.teamBPath, parsed.seed);
      process.stdout.write(result.replay);
      process.stdout.write(
        `\nWinner: ${result.winner === "draw" ? "Draw" : "Side " + result.winner}  (${result.turns} turns, seed ${parsed.seed})\n`,
      );
    } catch (err) {
      process.stderr.write((err as Error).message + "\n");
      process.exit(1);
    }
  } else {
    try {
      const stats = sweepBattles(parsed.teamAPath, parsed.teamBPath, parsed.sweepN);
      const report = formatSweepReport(stats, parsed.teamAPath, parsed.teamBPath);
      process.stdout.write(report + "\n");
    } catch (err) {
      process.stderr.write((err as Error).message + "\n");
      process.exit(1);
    }
  }
}

// Run only when this file is the entrypoint (not when imported by tests).
// In ESM under Node, import.meta.url === the argv[1] path (with file:// scheme).
const isMain =
  typeof process !== "undefined" &&
  typeof import.meta?.url === "string" &&
  process.argv[1] !== undefined &&
  import.meta.url.endsWith(process.argv[1].replace(/^.*[\\/]/, ""));

if (isMain) main();
