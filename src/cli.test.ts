// CLI tests — exercises team loading, arg parsing, sweep, and run logic
// without spawning any subprocess.

import { describe, expect, test } from "vitest";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { mkdtempSync, writeFileSync } from "node:fs";
import {
  autoplayRuns,
  formatAutoplayReport,
  formatSweepReport,
  loadTeamFile,
  parseArgs,
  runBattle,
  sweepBattles,
  validateTeamFile,
} from "./cli.js";
import { stressRegistry } from "./content/stress.js";
import { openLadder } from "./ladder.js";
import { FileLadderStore } from "./ladder-file.js";
import { BOOTSTRAP_DEPTH, BOOTSTRAP_TEAMS } from "./tunables.js";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const EXAMPLES = join(new URL(".", import.meta.url).pathname, "..", "examples");
const TEAM_A = join(EXAMPLES, "team-alpha.json");
const TEAM_B = join(EXAMPLES, "team-beta.json");

function writeTempTeam(name: string, content: string): string {
  const path = join(tmpdir(), `aoi-test-${name}-${Date.now()}.json`);
  writeFileSync(path, content, "utf8");
  return path;
}

const minimalTeam = JSON.stringify({
  units: [{ name: "Grunt", base: { hp: 5, pwr: 1 } }],
});

// ---------------------------------------------------------------------------
// 1. Team file loading and validation
// ---------------------------------------------------------------------------

describe("loadTeamFile", () => {
  test("loads example team-alpha.json", () => {
    const team = loadTeamFile(TEAM_A);
    expect(team.units.length).toBeGreaterThan(0);
    expect(team.units.length).toBeLessThanOrEqual(5);
    expect(typeof team.units[0]!.name).toBe("string");
  });

  test("loads example team-beta.json", () => {
    const team = loadTeamFile(TEAM_B);
    expect(team.units.length).toBeGreaterThan(0);
    expect(team.units.length).toBeLessThanOrEqual(5);
  });

  test("throws on missing file", () => {
    expect(() => loadTeamFile("/no/such/file.json")).toThrow(/Cannot read team file/);
  });

  test("throws on invalid JSON", () => {
    const p = writeTempTeam("bad-json", "{ not valid json }");
    expect(() => loadTeamFile(p)).toThrow(/not valid JSON/);
  });
});

describe("validateTeamFile", () => {
  test("accepts a minimal valid team object", () => {
    const data = JSON.parse(minimalTeam);
    const team = validateTeamFile(data);
    expect(team.units).toHaveLength(1);
  });

  test("rejects non-object", () => {
    expect(() => validateTeamFile([1, 2, 3])).toThrow(/expected a JSON object/);
  });

  test("rejects missing units field", () => {
    expect(() => validateTeamFile({})).toThrow(/"units" field/);
  });

  test("rejects empty units array", () => {
    expect(() => validateTeamFile({ units: [] })).toThrow(/1..5/);
  });

  test("rejects oversized units array (6)", () => {
    const units = Array.from({ length: 6 }, (_, i) => ({ name: `U${i}`, base: { hp: 1, pwr: 1 } }));
    expect(() => validateTeamFile({ units })).toThrow(/1..5/);
  });

  test("rejects unit with no name", () => {
    expect(() => validateTeamFile({ units: [{ base: { hp: 1, pwr: 1 } }] })).toThrow(/name must be a string/);
  });
});

// ---------------------------------------------------------------------------
// 2. Arg parsing
// ---------------------------------------------------------------------------

describe("parseArgs", () => {
  const wrap = (args: string[]) => ["node", "cli.ts", ...args];

  test("run mode with defaults", () => {
    const p = parseArgs(wrap(["a.json", "b.json"]));
    expect(p.mode).toBe("run");
    expect(p.teamAPath).toBe("a.json");
    expect(p.teamBPath).toBe("b.json");
    expect(p.seed).toBe(0);
  });

  test("run mode with explicit seed", () => {
    const p = parseArgs(wrap(["a.json", "b.json", "--seed", "42"]));
    expect(p.mode).toBe("run");
    expect(p.seed).toBe(42);
  });

  test("sweep mode", () => {
    const p = parseArgs(wrap(["a.json", "b.json", "--sweep", "50"]));
    expect(p.mode).toBe("sweep");
    expect(p.sweepN).toBe(50);
  });

  test("throws on too few args", () => {
    expect(() => parseArgs(wrap(["a.json"]))).toThrow(/Usage/);
  });

  test("throws on unknown flag", () => {
    expect(() => parseArgs(wrap(["a.json", "b.json", "--unknown"]))).toThrow(/Unknown argument/);
  });

  test("throws on negative seed", () => {
    expect(() => parseArgs(wrap(["a.json", "b.json", "--seed", "-1"]))).toThrow(/non-negative/);
  });

  test("throws on zero sweep", () => {
    expect(() => parseArgs(wrap(["a.json", "b.json", "--sweep", "0"]))).toThrow(/positive integer/);
  });
});

// ---------------------------------------------------------------------------
// 3. Run mode — a single battle from the example files
// ---------------------------------------------------------------------------

describe("runBattle", () => {
  test("returns replay text, winner, and turns", () => {
    const result = runBattle(TEAM_A, TEAM_B, 0);
    expect(typeof result.replay).toBe("string");
    expect(result.replay).toContain("=== BATTLE ===");
    expect(["A", "B", "draw"]).toContain(result.winner);
    expect(result.turns).toBeGreaterThan(0);
  });

  test("replay contains battle end line", () => {
    const result = runBattle(TEAM_A, TEAM_B, 7);
    expect(result.replay).toMatch(/===(.*)(wins|Draw)/);
  });

  test("deterministic — same seed yields identical replay", () => {
    const r1 = runBattle(TEAM_A, TEAM_B, 13);
    const r2 = runBattle(TEAM_A, TEAM_B, 13);
    expect(r1.replay).toBe(r2.replay);
  });
});

// ---------------------------------------------------------------------------
// 4. Sweep mode — win counts must sum to N
// ---------------------------------------------------------------------------

describe("sweepBattles", () => {
  test("win counts sum to N=50", () => {
    const N = 50;
    const stats = sweepBattles(TEAM_A, TEAM_B, N);
    expect(stats.n).toBe(N);
    expect(stats.aWins + stats.bWins + stats.draws).toBe(N);
  });

  test("totalTurns is positive and at least N (every battle ≥ 1 turn)", () => {
    const N = 20;
    const stats = sweepBattles(TEAM_A, TEAM_B, N);
    expect(stats.totalTurns).toBeGreaterThanOrEqual(N);
  });

  test("formatSweepReport includes all required fields", () => {
    const stats = sweepBattles(TEAM_A, TEAM_B, 10);
    const report = formatSweepReport(stats, TEAM_A, TEAM_B);
    expect(report).toContain("Sweep:");
    expect(report).toContain("Team A");
    expect(report).toContain("Team B");
    expect(report).toContain("Draws:");
    expect(report).toContain("Avg battle length:");
  });

  test("sweep over N seeds, each a different seed, all complete", () => {
    const N = 10;
    for (let seed = 0; seed < N; seed++) {
      expect(() => runBattle(TEAM_A, TEAM_B, seed)).not.toThrow();
    }
  });
});

// ---------------------------------------------------------------------------
// 5. Autoplay mode — the policy bot plays whole runs on a file-backed ladder
// ---------------------------------------------------------------------------

describe("autoplay", () => {
  const dir = mkdtempSync(join(tmpdir(), "aoi-autoplay-"));
  const freshFileLadder = (name: string) => openLadder(new FileLadderStore(join(dir, name)), stressRegistry);

  test("parseArgs autoplay mode with defaults and flags", () => {
    const wrap = (args: string[]) => ["node", "cli.ts", ...args];
    expect(parseArgs(wrap(["autoplay", "ladder.json"]))).toMatchObject({
      mode: "autoplay", ladderPath: "ladder.json", seed: 0, runs: 1, logPath: null,
    });
    expect(parseArgs(wrap(["autoplay", "l.json", "--seed", "7", "--runs", "5", "--log", "out.jsonl"]))).toMatchObject({
      mode: "autoplay", ladderPath: "l.json", seed: 7, runs: 5, logPath: "out.jsonl",
    });
    expect(() => parseArgs(wrap(["autoplay"]))).toThrow(/Usage/);
    expect(() => parseArgs(wrap(["autoplay", "l.json", "--runs", "0"]))).toThrow(/positive integer/);
    expect(() => parseArgs(wrap(["autoplay", "l.json", "--log"]))).toThrow(/needs a file path/);
    expect(() => parseArgs(wrap(["autoplay", "l.json", "--policy"]))).toThrow(/Unknown argument/);
  });

  test("a policy run plays a whole run to its end, headless", () => {
    const [r] = autoplayRuns(freshFileLadder("single.json"), 0, 1);
    expect(r!.state.status).toBe("over");
    expect(["crown", "challenge-lost", "out-of-lives"]).toContain(r!.state.endedBy);
    expect(r!.state.team.length).toBeGreaterThan(0);
    expect(r!.state.log[r!.state.log.length - 1]).toMatchObject({ type: "RunEnded" });
  });

  test("determinism: same ladder starting state + same seed → byte-identical run log, twice", () => {
    // Two fresh files are the same starting state (the bootstrap seed); the
    // run log JSONL is the determinism artifact and must match to the byte.
    const play = (name: string) => autoplayRuns(freshFileLadder(name), 7, 1)[0]!.jsonl;
    expect(play("det-a.json")).toBe(play("det-b.json"));
  });

  test("an N-run sweep fills the pools, and the growth persists on disk", () => {
    const path = join(dir, "sweep.json");
    const N = 4;
    const results = autoplayRuns(openLadder(new FileLadderStore(path), stressRegistry), 0, N);
    expect(results).toHaveLength(N);
    expect(results.every((r) => r.state.status === "over")).toBe(true);
    // Every run fights at round 1, so its ghost joined the round-1 pool — which
    // already held the floor-1 climb teams AND the floor-1 boss-ghost (075-3).
    const reread = new FileLadderStore(path); // growth read back from disk, not memory
    expect(reread.poolAt(1).length).toBe(BOOTSTRAP_TEAMS[0]!.length + 1 + N);
    // Runs climb to the summit floor (BOOTSTRAP_DEPTH+1) and ghost themselves there
    // when they challenge it — so that floor's pool, seeded empty, has grown.
    expect(reread.poolAt(BOOTSTRAP_DEPTH + 1).length).toBeGreaterThan(0);
  });

  test("the report reads as run summaries plus the ladder line", () => {
    const store = freshFileLadder("report.json");
    const results = autoplayRuns(store, 42, 2);
    const report = formatAutoplayReport(results, store);
    expect(report).toContain("Run auto-42 (seed 42):");
    expect(report).toContain("Run auto-43 (seed 43):");
    expect(report).toMatch(/(crowned|challenge lost|out of lives) at round \d+ — \d+W\/\d+L\/\d+D/);
    expect(report).toMatch(/line: {2}\w+ L\d+/);
    expect(report).toMatch(/Ladder: champion \S+ \| pools r1:\d+/);
  });
});

// ---------------------------------------------------------------------------
// 6. Minimal team file written to tmp — exercises the full load→battle path
// ---------------------------------------------------------------------------

describe("tmp team file round-trip", () => {
  test("battle runs from two tmp team files (no statuses needed)", () => {
    const pathA = writeTempTeam("a", JSON.stringify({ units: [{ name: "Grunt", base: { hp: 5, pwr: 2 } }] }));
    const pathB = writeTempTeam("b", JSON.stringify({ units: [{ name: "Orc", base: { hp: 4, pwr: 1 } }] }));
    const result = runBattle(pathA, pathB, 0);
    expect(["A", "B", "draw"]).toContain(result.winner);
  });
});
