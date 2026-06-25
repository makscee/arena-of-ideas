/**
 * Arena of Ideas — battle CLI (slice 6; autoplay in run-loop slice 3)
 *
 * Three modes:
 *
 * RUN MODE — play a single battle and print the full replay:
 *   node --import=tsx/esm src/cli.ts <teamA.json> <teamB.json> [--seed N]
 *   Default seed: 0 (deterministic).
 *
 * SWEEP MODE — run N seeds and report win-rate distribution:
 *   node --import=tsx/esm src/cli.ts <teamA.json> <teamB.json> --sweep N
 *
 * AUTOPLAY MODE — a deterministic policy bot plays whole runs on a ladder file:
 *   node --import=tsx/esm src/cli.ts autoplay <ladder.json> [--seed N] [--runs N] [--log <path>]
 *   The ladder file is created (bootstrap-seeded) if missing. N runs play
 *   sequentially with seeds seed..seed+N−1, so pools fill and the crown can
 *   change hands over a sweep. --log writes the concatenated run logs as
 *   JSONL — the determinism artifact: same ladder starting state + same seed
 *   → byte-identical.
 *
 * Team file format (JSON):
 *   {
 *     "units": UnitDef[]   // 1..5 units; see src/types.ts for UnitDef schema
 *   }
 *   Any extra keys (e.g. "_comment") are silently ignored.
 *   Statuses referenced in units[].statuses must be present in the stress
 *   registry; for custom status sets pass a registry via the API directly.
 *   The CLI always uses the stressRegistry exported from src/content/stress.ts.
 *   Team content runs through the validator (src/validate.ts) before battle():
 *   unknown kinds, wrong-context parts, and dangling references fail loudly.
 *
 * All logic below (except the top-level main() entrypoint) is exported for
 * unit testing without spawning a subprocess.
 */

import { readFileSync, writeFileSync } from "node:fs";
import {
  battle,
  buy,
  challengeBoss,
  initRun,
  InvalidDecisionError,
  ladderFight,
  openLadder,
  renderReplay,
  reroll,
  runToJSONL,
  sweep,
  winnerOf,
  DEFAULT_RUN_POOL,
  REROLL_COST,
  TEAM_SIZE,
  UNIT_COST,
} from "./index.js";
import type { LadderStore, RunEvent, RunEventType, RunInput, RunState, SweepStats } from "./index.js";
import { FileLadderStore } from "./ladder-file.js";
import { stressRegistry } from "./content/stress.js";
import { assertValidContent } from "./validate.js";
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
  // Light structural check first (readable errors for malformed files)…
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
  // …then the content validator: a typo'd part would be silently inert in
  // battle(), so it must fail loudly here, before the kernel ever sees it.
  assertValidContent(units, stressRegistry, `${label} units`);
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

export type { SweepStats } from "./index.js";

export function sweepBattles(teamAPath: string, teamBPath: string, n: number): SweepStats {
  const a = loadTeamFile(teamAPath);
  const b = loadTeamFile(teamBPath);
  // The distribution itself is the kernel's sweep helper — shared with the
  // web gauntlet, so browser and CLI can never disagree on a win-rate.
  return sweep({ teamA: a.units, teamB: b.units, statuses: stressRegistry }, n);
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
// Autoplay mode — a policy bot plays whole runs against a file-backed ladder
// ---------------------------------------------------------------------------

/** The pool autoplay runs draft from — the shared default (tunables.ts), so
 * autoplay and the web run screen fill the same ladder with the same meta. */
export const AUTOPLAY_POOL: UnitDef[] = DEFAULT_RUN_POOL;

const ofType = <T extends RunEventType>(log: readonly RunEvent[], t: T): Extract<RunEvent, { type: T }>[] =>
  log.filter((e): e is Extract<RunEvent, { type: T }> => e.type === t);

/** One greedy shop pass: stack copies first (levels compound), then fill the
 * line, and reroll dead gold — gold a full line of strangers can't spend —
 * while a buy could still follow. The policy is deterministic given the state
 * and draws no randomness of its own, so everything random in an autoplay run
 * (shop rolls, battle seeds, opponent draws) comes off the run's one seeded
 * stream and a replay is byte-identical. Gold strictly decreases every step,
 * so the pass terminates. */
export function shopGreedily(state: RunState): RunState {
  let s = state;
  while (s.gold >= UNIT_COST) {
    const stack = s.offers.findIndex((o) => s.team.some((u) => u.name === o.name));
    if (stack >= 0) {
      s = buy(s, stack);
    } else if (s.team.length < TEAM_SIZE && s.offers.length > 0) {
      s = buy(s, 0);
    } else if (s.gold >= UNIT_COST + REROLL_COST) {
      s = reroll(s); // dead gold: full line, no copy on offer — dig for one
    } else {
      break;
    }
  }
  return s;
}

/** Play one whole run with the greedy policy — shop, then climb the ladder,
 * until the run ends. A climb (ladderFight) draws a same-floor ghost; when the
 * floor has no climb opponent left it rejects loudly, and the only move is to
 * challenge the floor's boss — the terminal move that ends the run, won
 * (crown) or lost (challenge-lost / out-of-lives never applies to a
 * challenge). So the policy climbs while it can and challenges the boss the
 * moment a climb is refused. */
export function playPolicyRun(input: RunInput, ladder: LadderStore): RunState {
  let s = initRun(input);
  while (s.status === "active") {
    s = shopGreedily(s);
    try {
      s = ladderFight(s, ladder);
    } catch (err) {
      // The one expected rejection: no climb opponent at this floor. Any other
      // error is a real fault and propagates. The boss challenge is terminal,
      // so this is the run's last move whichever way it goes.
      if (err instanceof InvalidDecisionError && err.decision === "fight") {
        s = challengeBoss(s, ladder);
      } else {
        throw err;
      }
    }
  }
  return s;
}

export interface AutoplayResult {
  state: RunState;
  /** The raw run log as JSONL — the determinism artifact. */
  jsonl: string;
}

/** Play `runs` sequential policy runs against one ladder. Run i's seed is
 * seed+i — derived but deterministic, so a whole sweep replays from a single
 * seed — and its runId carries the seed, keeping runs on a shared ladder
 * distinct (own-ghost exclusion is by runId). */
export function autoplayRuns(ladder: LadderStore, seed: number, runs: number): AutoplayResult[] {
  const results: AutoplayResult[] = [];
  for (let i = 0; i < runs; i++) {
    const runSeed = seed + i;
    const state = playPolicyRun(
      { seed: runSeed, runId: `auto-${runSeed}`, pool: AUTOPLAY_POOL, statuses: stressRegistry },
      ladder,
    );
    results.push({ state, jsonl: runToJSONL(state.log) });
  }
  return results;
}

/** One run's summary, human-readable: how far it climbed, the fight record,
 * the final line, and what happened to the crown. */
export function formatRunSummary(state: RunState): string {
  const fights = ofType(state.log, "FightFought");
  const won = fights.filter((f) => f.winner === "A").length;
  const lost = fights.filter((f) => f.winner === "B").length;
  const drawn = fights.length - won - lost;
  const head =
    state.endedBy === "crown" ? "crowned" : state.endedBy === "challenge-lost" ? "challenge lost" : "out of lives";
  const lines = [
    `Run ${state.runId} (seed ${state.seed}): ${head} at round ${state.round} — ${won}W/${lost}L/${drawn}D, ${state.lives} ${state.lives === 1 ? "life" : "lives"} left`,
    `  line:  ${state.team.map((u) => `${u.name} L${u.level}`).join(", ")}`,
  ];
  const crowned = ofType(state.log, "Crowned")[0];
  if (crowned !== undefined) {
    lines.push(`  crown: ${crowned.dethroned === null ? "the spot was vacant" : `dethroned ${crowned.dethroned}`} — ${state.runId} reigns`);
  }
  return lines.join("\n");
}

/** The whole autoplay report: a summary per run, then the ladder after —
 * champion lineage and pool sizes, so a sweep shows the pools filling. */
export function formatAutoplayReport(results: readonly AutoplayResult[], ladder: LadderStore): string {
  const lines = results.map((r) => formatRunSummary(r.state));
  const crowns = results.filter((r) => r.state.endedBy === "crown");
  if (crowns.length > 0) {
    const first = ofType(crowns[0]!.state.log, "Crowned")[0]!.dethroned ?? "(vacant)";
    lines.push(`Champion lineage: ${[first, ...crowns.map((r) => r.state.runId)].join(" → ")}`);
  }
  const pools: string[] = [];
  for (let r = 1; ladder.poolAt(r).length > 0; r++) pools.push(`r${r}:${ladder.poolAt(r).length}`);
  const champ = ladder.champion();
  lines.push(`Ladder: champion ${champ === null ? "(vacant)" : champ.runId} | pools ${pools.join(" ")}`);
  return lines.join("\n");
}

// ---------------------------------------------------------------------------
// Arg parsing
// ---------------------------------------------------------------------------

export interface ParsedArgs {
  mode: "run" | "sweep" | "autoplay";
  teamAPath: string;
  teamBPath: string;
  seed: number;     // run + autoplay modes
  sweepN: number;   // sweep mode
  ladderPath: string;     // autoplay mode
  runs: number;           // autoplay mode
  logPath: string | null; // autoplay mode
}

const USAGE =
  "Usage:\n" +
  "  Run mode:      battle <teamA.json> <teamB.json> [--seed N]\n" +
  "  Sweep mode:    battle <teamA.json> <teamB.json> --sweep N\n" +
  "  Autoplay mode: battle autoplay <ladder.json> [--seed N] [--runs N] [--log <path>]";

export function parseArgs(argv: string[]): ParsedArgs {
  const args = argv.slice(2); // drop node + script

  if (args[0] === "autoplay") {
    return parseAutoplayArgs(args.slice(1));
  }

  if (args.length < 2) {
    throw new Error(USAGE);
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
    return { mode: "sweep", teamAPath, teamBPath, seed: 0, sweepN, ladderPath: "", runs: 0, logPath: null };
  }
  return { mode: "run", teamAPath, teamBPath, seed, sweepN: 0, ladderPath: "", runs: 0, logPath: null };
}

function parseAutoplayArgs(args: string[]): ParsedArgs {
  if (args.length < 1) {
    throw new Error(USAGE);
  }
  const ladderPath = args[0]!;
  let seed = 0;
  let runs = 1;
  let logPath: string | null = null;

  for (let i = 1; i < args.length; i++) {
    if (args[i] === "--seed") {
      const val = Number(args[i + 1]);
      if (!Number.isFinite(val) || val < 0) throw new Error("--seed must be a non-negative integer");
      seed = Math.floor(val);
      i++;
    } else if (args[i] === "--runs") {
      const val = Number(args[i + 1]);
      if (!Number.isFinite(val) || val < 1) throw new Error("--runs N must be a positive integer");
      runs = Math.floor(val);
      i++;
    } else if (args[i] === "--log") {
      const val = args[i + 1];
      if (val === undefined || val.startsWith("--")) throw new Error("--log needs a file path");
      logPath = val;
      i++;
    } else {
      throw new Error(`Unknown argument: ${args[i]}`);
    }
  }
  return { mode: "autoplay", teamAPath: "", teamBPath: "", seed, sweepN: 0, ladderPath, runs, logPath };
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

  try {
    if (parsed.mode === "run") {
      const result = runBattle(parsed.teamAPath, parsed.teamBPath, parsed.seed);
      process.stdout.write(result.replay);
      process.stdout.write(
        `\nWinner: ${result.winner === "draw" ? "Draw" : "Side " + result.winner}  (${result.turns} turns, seed ${parsed.seed})\n`,
      );
    } else if (parsed.mode === "sweep") {
      const stats = sweepBattles(parsed.teamAPath, parsed.teamBPath, parsed.sweepN);
      const report = formatSweepReport(stats, parsed.teamAPath, parsed.teamBPath);
      process.stdout.write(report + "\n");
    } else {
      const store = openLadder(new FileLadderStore(parsed.ladderPath), stressRegistry);
      const results = autoplayRuns(store, parsed.seed, parsed.runs);
      process.stdout.write(formatAutoplayReport(results, store) + "\n");
      if (parsed.logPath !== null) {
        writeFileSync(parsed.logPath, results.map((r) => r.jsonl).join(""), "utf8");
      }
    }
  } catch (err) {
    process.stderr.write((err as Error).message + "\n");
    process.exit(1);
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
