import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, test } from "vitest";
import { stressRegistry } from "./content/stress.js";
import { BOOTSTRAP_RUN_ID, InMemoryLadderStore, openLadder } from "./ladder.js";
import type { LadderStore, TeamSnapshot } from "./ladder.js";
import { FileLadderStore } from "./ladder-file.js";
import { buy, fight, initRun, InvalidDecisionError, ladderFight, reorder, reroll, runToJSONL } from "./run.js";
import type { RunEvent, RunInput, RunState } from "./run.js";
import { BOOTSTRAP_TEAMS, STARTING_LIVES } from "./tunables.js";
import { ValidationError, validateTeam } from "./validate.js";
import type { UnitDef } from "./types.js";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const ofType = <T extends RunEvent["type"]>(
  log: RunEvent[],
  t: T,
): Extract<RunEvent, { type: T }>[] =>
  log.filter((e): e is Extract<RunEvent, { type: T }> => e.type === t);

function vanilla(name: string, hp: number, pwr: number): UnitDef {
  return { name, base: { hp, pwr } };
}

// One-unit pools with deterministic ladder fates (probed across seeds):
// Titan beats both bootstrap teams; Goliath beats those and Titan; Grunt
// loses to everything. So a run's arc depends only on the ladder contents.
const TITAN = vanilla("Titan", 100, 50);
const GOLIATH = vanilla("Goliath", 200, 80);
const GRUNT = vanilla("Grunt", 1, 0);

function input(seed: number, runId: string, unit: UnitDef): RunInput {
  return { seed, runId, pool: [unit], statuses: stressRegistry };
}

/** Buy the one unit, then fight the ladder until the run ends. */
function playLadderRun(inp: RunInput, ladder: LadderStore): RunState {
  let s = buy(initRun(inp), 0);
  while (s.status === "active") s = ladderFight(s, ladder);
  return s;
}

function freshLadder(): LadderStore {
  return openLadder(new InMemoryLadderStore());
}

// ---------------------------------------------------------------------------
// 1. Bootstrap: an empty ladder seeds round 1 from the shipped teams
// ---------------------------------------------------------------------------

describe("bootstrap", () => {
  test("openLadder seeds round 1 of an empty ladder with valid bootstrap ghosts", () => {
    const pool = freshLadder().poolAt(1);
    expect(pool.length).toBe(BOOTSTRAP_TEAMS.length);
    pool.forEach((g, i) => {
      expect(g).toMatchObject({ runId: BOOTSTRAP_RUN_ID, round: 1, seq: i });
      expect(validateTeam(g.team, stressRegistry)).toEqual([]); // a first run's opponents pass the gate
    });
  });

  test("a non-empty ladder is never reseeded", () => {
    const store = freshLadder();
    openLadder(store);
    expect(store.poolAt(1).length).toBe(BOOTSTRAP_TEAMS.length);
  });
});

// ---------------------------------------------------------------------------
// 2. Snapshot-before-fight: the ghost enters the pool first and persists
// ---------------------------------------------------------------------------

describe("snapshot-before-fight", () => {
  test("the fielded team is ghosted into the round pool, before the draw", () => {
    const store = freshLadder();
    const s = ladderFight(buy(initRun(input(7, "titan", TITAN)), 0), store);
    const ghost = store.poolAt(1).find((g) => g.runId === "titan")!;
    expect(ghost).toMatchObject({ round: 1, seq: BOOTSTRAP_TEAMS.length });
    expect(ghost.team.map((u) => u.name)).toEqual(["Titan"]);
    // The log shows the order: Snapshotted, then the draw, then the fight.
    const types = s.log.map((e) => e.type);
    expect(types.indexOf("Snapshotted")).toBeLessThan(types.indexOf("OpponentDrawn"));
    expect(types.indexOf("OpponentDrawn")).toBeLessThan(types.indexOf("FightFought"));
  });

  test("ghosts persist after the run dies", () => {
    const store = freshLadder();
    playLadderRun(input(1, "titan", TITAN), store); // installs a champion
    const dead = playLadderRun(input(2, "grunt", GRUNT), store);
    expect(dead).toMatchObject({ status: "over", endedBy: "out-of-lives", lives: 0 });
    // One ghost per round fought, all still in their pools.
    for (let round = 1; round <= dead.round; round++) {
      expect(store.poolAt(round).some((g) => g.runId === "grunt")).toBe(true);
    }
  });

  test("ladderFight never mutates its input state", () => {
    const store = freshLadder();
    const s0 = buy(initRun(input(7, "titan", TITAN)), 0);
    const snapshot = JSON.stringify(s0);
    ladderFight(s0, store);
    expect(JSON.stringify(s0)).toBe(snapshot);
  });
});

// ---------------------------------------------------------------------------
// 3. Opponent draw: seeded, deterministic, own ghosts excluded
// ---------------------------------------------------------------------------

describe("opponent draw", () => {
  test("a run's own ghosts are excluded from its draw", () => {
    for (let seed = 0; seed < 20; seed++) {
      const store = freshLadder();
      // Pre-ghosted rounds: the run already left ghosts at round 1 (as if
      // re-opened mid-ladder); they outnumber the bootstrap pair 3:2.
      for (const seq of [2, 3, 4]) {
        store.addSnapshot({ runId: "self", round: 1, seq, team: [TITAN] });
      }
      const s = ladderFight(buy(initRun(input(seed, "self", TITAN)), 0), store);
      const drawn = ofType(s.log, "OpponentDrawn")[0]!;
      expect(drawn.opponent).toBe(BOOTSTRAP_RUN_ID);
      expect(drawn.candidates).toBe(BOOTSTRAP_TEAMS.length); // 6 in the pool, own 4 excluded
    }
  });

  test("the draw is deterministic given the run's RNG state and pool contents", () => {
    const [a, b] = [freshLadder(), freshLadder()].map((store) =>
      ladderFight(buy(initRun(input(13, "titan", TITAN)), 0), store),
    );
    expect(ofType(a!.log, "OpponentDrawn")).toEqual(ofType(b!.log, "OpponentDrawn"));
  });
});

// ---------------------------------------------------------------------------
// 4. Run-end states: out of lives, and decisions on an over run
// ---------------------------------------------------------------------------

describe("run-end states", () => {
  test("losing the last life ends the run as out-of-lives", () => {
    let s = buy(initRun({ seed: 9, pool: [GRUNT] }), 0);
    for (let i = 0; i < STARTING_LIVES; i++) s = fight(s, [vanilla("Wall", 100, 99)]);
    expect(s).toMatchObject({ status: "over", endedBy: "out-of-lives", lives: 0 });
    expect(s.log[s.log.length - 1]).toMatchObject({ type: "RunEnded", reason: "out-of-lives", lives: 0 });
    // The run ended mid-round: no fresh income, no fresh shop after the last loss.
    expect(ofType(s.log, "RoundStarted").length).toBe(STARTING_LIVES - 1);
  });

  test("every decision on an over run throws", () => {
    let s = buy(initRun({ seed: 9, pool: [GRUNT] }), 0);
    for (let i = 0; i < STARTING_LIVES; i++) s = fight(s, [vanilla("Wall", 100, 99)]);
    const over = s;
    for (const d of [
      () => buy(over, 0),
      () => reroll(over),
      () => reorder(over, 0, 0),
      () => fight(over, [GRUNT]),
      () => ladderFight(over, freshLadder()),
    ]) {
      expect(d).toThrow(InvalidDecisionError);
      expect(d).toThrow(/run is over \(out-of-lives\)/);
    }
  });
});

// ---------------------------------------------------------------------------
// 5. The champion spot: challenge, crown, persistence, dethroning
// ---------------------------------------------------------------------------

describe("champion", () => {
  test("an empty pool with a vacant spot crowns the run outright", () => {
    const store = freshLadder();
    const s = playLadderRun(input(1, "titan", TITAN), store);
    // Round 1 fought a bootstrap ghost; round 2's pool held only the run's
    // own ghost, the spot was vacant — crowned without a champion fight.
    expect(s).toMatchObject({ status: "over", endedBy: "crown", round: 2 });
    expect(ofType(s.log, "ChampionChallenged")).toEqual([]);
    expect(ofType(s.log, "Crowned")[0]).toMatchObject({ dethroned: null });
    expect(s.log[s.log.length - 1]).toMatchObject({ type: "RunEnded", reason: "crown" });
    expect(store.champion()).toMatchObject({ runId: "titan", round: 2 });
  });

  test("the champion persists across runs and falls only to a winner", () => {
    const store = freshLadder();
    playLadderRun(input(1, "titan", TITAN), store);
    expect(store.champion()!.runId).toBe("titan"); // survives the run that crowned it
    const s = playLadderRun(input(2, "goliath", GOLIATH), store);
    // Goliath fought ghosts while any remained, then beat the champion team.
    expect(ofType(s.log, "ChampionChallenged")[0]).toMatchObject({ champion: "titan" });
    expect(ofType(s.log, "Crowned")[0]).toMatchObject({ dethroned: "titan" });
    expect(s).toMatchObject({ status: "over", endedBy: "crown" });
    expect(store.champion()).toMatchObject({ runId: "goliath" });
  });

  test("the dethroned champion's team stays in the pools as a ghost", () => {
    const store = freshLadder();
    playLadderRun(input(1, "titan", TITAN), store);
    playLadderRun(input(2, "goliath", GOLIATH), store);
    const titanGhosts = [1, 2].flatMap((r) => store.poolAt(r).filter((g) => g.runId === "titan"));
    expect(titanGhosts.length).toBe(2); // one per round it fought, crown round included
    expect(titanGhosts.every((g) => g.team[0]!.name === "Titan")).toBe(true);
  });

  test("a lost champion challenge is a normal loss — the run carries on", () => {
    const store = freshLadder();
    playLadderRun(input(1, "titan", TITAN), store);
    const s = playLadderRun(input(2, "grunt", GRUNT), store);
    // Grunt outlives the ghost rounds into repeated champion challenges,
    // losing a life per challenge until out of lives — never crowned.
    expect(ofType(s.log, "ChampionChallenged").length).toBeGreaterThan(1);
    expect(ofType(s.log, "Crowned")).toEqual([]);
    expect(s).toMatchObject({ status: "over", endedBy: "out-of-lives" });
    expect(store.champion()!.runId).toBe("titan");
  });
});

// ---------------------------------------------------------------------------
// 6. Determinism: a whole ladder run is byte-identical
// ---------------------------------------------------------------------------

describe("determinism", () => {
  test("identical seed + decisions + ladder contents → byte-identical run log", () => {
    const play = () => {
      const store = freshLadder();
      playLadderRun(input(1, "titan", TITAN), store); // identical prior history
      return playLadderRun(input(2, "goliath", GOLIATH), store);
    };
    expect(runToJSONL(play().log)).toBe(runToJSONL(play().log));
  });
});

// ---------------------------------------------------------------------------
// 7. File backing: JSON on disk behind the same interface
// ---------------------------------------------------------------------------

describe("file backing", () => {
  const dir = mkdtempSync(join(tmpdir(), "ladder-"));

  test("write through one store, read through a fresh one — equal", () => {
    const path = join(dir, "roundtrip.json");
    const store = openLadder(new FileLadderStore(path));
    const ghost: TeamSnapshot = { runId: "titan", round: 1, seq: BOOTSTRAP_TEAMS.length, team: [TITAN] };
    store.addSnapshot(ghost);
    store.setChampion(ghost);
    const reread = new FileLadderStore(path);
    expect(reread.poolAt(1)).toEqual(store.poolAt(1));
    expect(reread.champion()).toEqual(store.champion());
    expect(reread.poolAt(2)).toEqual([]); // untouched rounds stay empty
  });

  test("a ladder run plays byte-identically on file and in-memory backings", () => {
    const onFile = playLadderRun(input(1, "titan", TITAN), openLadder(new FileLadderStore(join(dir, "parity.json"))));
    const inMemory = playLadderRun(input(1, "titan", TITAN), freshLadder());
    expect(runToJSONL(onFile.log)).toBe(runToJSONL(inMemory.log));
  });

  test("a missing file opens an empty ladder; a corrupt one throws loudly", () => {
    expect(new FileLadderStore(join(dir, "missing.json")).poolAt(1)).toEqual([]);
    const corrupt = join(dir, "corrupt.json");
    writeFileSync(corrupt, "not json", "utf8");
    expect(() => new FileLadderStore(corrupt)).toThrow(/not valid JSON/);
  });
});

// ---------------------------------------------------------------------------
// 8. The pool gate: initRun rejects bad content and duplicate names
// ---------------------------------------------------------------------------

describe("pool validation", () => {
  test("invalid pool content is rejected through the content gate", () => {
    const bad = [{ name: "Bad", base: { hp: -1, pwr: 1 } }] as UnitDef[];
    expect(() => initRun({ seed: 1, pool: bad })).toThrow(ValidationError);
    expect(() => initRun({ seed: 1, pool: bad })).toThrow(/pool\[0\]\.base\.hp/);
  });

  test("duplicate unit names are rejected — the shop stacks copies by name", () => {
    const twins = [vanilla("Twin", 5, 1), vanilla("Twin", 6, 2)];
    expect(() => initRun({ seed: 1, pool: twins })).toThrow(ValidationError);
    expect(() => initRun({ seed: 1, pool: twins })).toThrow(/duplicate unit name "Twin"/);
  });

  test("an empty pool is rejected", () => {
    expect(() => initRun({ seed: 1, pool: [] })).toThrow(ValidationError);
  });
});
