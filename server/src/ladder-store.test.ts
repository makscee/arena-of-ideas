/**
 * Store-contract parity for the SQLite ladder backing — the same semantics
 * the kernel pins for its in-memory and file backings (src/ladder.test.ts),
 * mirrored here because this backing needs the server DB. Plus the one
 * dimension the shared ladder adds on top of the kernel interface: ghost
 * ownership, so a user's ghosts (across all their runs) leave their own draws.
 */
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, test } from "vitest";
import {
  BOSS_TEAMS,
  BOOTSTRAP_RUN_ID,
  BOOTSTRAP_TEAMS,
  buy,
  challengeBoss,
  InMemoryLadderStore,
  initRun,
  InvalidDecisionError,
  ladderFight,
  openLadder,
  runToJSONL,
  stressAbilities,
  stressRegistry,
  TOWER_HEIGHT,
  validateTeam,
  type LadderStore,
  type RunInput,
  type RunState,
  type TeamSnapshot,
  type UnitDef,
} from "../../src/index.js";
import { openDb } from "./db.js";
import { SqliteLadderStore } from "./ladder-store.js";

function vanilla(name: string, hp: number, pwr: number): UnitDef {
  return { name, base: { hp, pwr }, ability: "Strike" };
}

// The kernel ladder tests' deterministic climbers: Titan beats the bootstrap
// teams and champion; Goliath beats those and Titan.
const TITAN = vanilla("Titan", 100, 50);
const GOLIATH = vanilla("Goliath", 200, 80);

function input(seed: number, runId: string, unit: UnitDef): RunInput {
  return { seed, runId, pool: [unit], statuses: stressRegistry, abilities: stressAbilities };
}

/** Buy the one unit, then climb to the top of the (fixed-height) tower and
 * challenge the champion. Climbing PAST the top overshoots (no crown), so the
 * policy stops at the champion's floor and challenges there — mirroring the
 * kernel's playLadderRun / the CLI's playPolicyRun. */
function playLadderRun(inp: RunInput, ladder: LadderStore): RunState {
  let s = buy(initRun(inp), 0);
  while (s.status === "active") {
    const top = ladder.champion();
    if (top !== null && s.round >= top.round) {
      s = challengeBoss(s, ladder); // at the top: challenge, don't climb past into an overshoot
      continue;
    }
    try {
      s = ladderFight(s, ladder);
    } catch (err) {
      if (err instanceof InvalidDecisionError && err.decision === "fight") {
        s = challengeBoss(s, ladder);
      } else {
        throw err;
      }
    }
  }
  return s;
}

function freshSqlite(): SqliteLadderStore {
  const store = new SqliteLadderStore(openDb(":memory:").db);
  openLadder(store, stressRegistry, stressAbilities);
  return store;
}

describe("bootstrap", () => {
  test("openLadder seeds the uniform tower; the SQLite backing keeps the champion + every boss-ghost", () => {
    // The uniform fixed-height tower (075-3): floors 1..TOWER_HEIGHT each carry a
    // climb pool plus a boss whose team is ALSO in that floor's pool (the boss-ghost).
    // Floor TOWER_HEIGHT's boss is the champion. Nothing is seeded above the top.
    //
    // The SQLite backing stores only the champion-history head and does not (yet)
    // have a per-floor boss table — bossAt reads a seated boss only on the champion's
    // own floor (see SqliteLadderStore.bossAt). openLadder setBoss's every floor, so
    // the LAST seat (floor TOWER_HEIGHT) is the champion; every floor's per-floor boss
    // is retained as its POOL-GHOST (which DOES persist). A shared-ladder run challenges
    // the champion floor, so this loses no reachable behaviour; a per-floor boss table
    // on SQLite is a follow-up if lower seats ever need to be queryable server-side.
    const store = freshSqlite();
    for (let round = 1; round <= TOWER_HEIGHT; round++) {
      const pool = store.poolAt(round);
      expect(pool.length).toBe(BOOTSTRAP_TEAMS[round - 1]!.length + 1); // climb teams + the boss-ghost
      pool.forEach((g, i) => {
        expect(g).toMatchObject({ runId: BOOTSTRAP_RUN_ID, round, seq: i });
        expect(validateTeam(g.team, stressRegistry, stressAbilities)).toEqual([]);
      });
      // The floor's boss team is its pool's last ghost — drawable, so a demote of
      // this floor's boss would leave it in the pool (the invariant, ghost-side).
      expect(pool[pool.length - 1]!.team.map((u) => u.name)).toEqual(BOSS_TEAMS[round - 1]!.map((u) => u.name));
    }
    expect(store.poolAt(TOWER_HEIGHT + 1)).toEqual([]); // nothing seeded above the tower
    const champ = store.champion()!;
    expect(champ).toMatchObject({ runId: BOOTSTRAP_RUN_ID, round: TOWER_HEIGHT });
    expect(champ.team.map((u) => u.name)).toEqual(BOSS_TEAMS[TOWER_HEIGHT - 1]!.map((u) => u.name));
  });

  test("a non-empty ladder is never reseeded; an earned champion survives reopen", () => {
    const dir = mkdtempSync(join(tmpdir(), "arena-ladder-"));
    const path = join(dir, "arena.db");
    const store = new SqliteLadderStore(openDb(path).db);
    openLadder(store, stressRegistry, stressAbilities);
    playLadderRun(input(1, "titan", TITAN), store); // dethrones the bootstrap champion
    const reopened = new SqliteLadderStore(openDb(path).db);
    openLadder(reopened, stressRegistry, stressAbilities);
    // Floor 1 held its climb teams + boss-ghost; the run added its own — no reseed.
    expect(reopened.poolAt(1).length).toBe(BOOTSTRAP_TEAMS[0]!.length + 1 + 1);
    expect(reopened.champion()!.runId).toBe("titan"); // never reseated
  });
});

describe("kernel-semantics parity", () => {
  test("a ladder run plays byte-identically on sqlite and in-memory backings", () => {
    const logs = [freshSqlite(), openLadder(new InMemoryLadderStore(), stressRegistry, stressAbilities)].map((store) => {
      playLadderRun(input(1, "titan", TITAN), store); // identical prior history
      return runToJSONL(playLadderRun(input(2, "goliath", GOLIATH), store).log);
    });
    expect(logs[0]).toBe(logs[1]);
  });

  test("pools and champion match the in-memory backing after the same drives", () => {
    const sqlite = freshSqlite();
    const memory = openLadder(new InMemoryLadderStore(), stressRegistry, stressAbilities);
    for (const store of [sqlite, memory]) {
      playLadderRun(input(1, "titan", TITAN), store);
      playLadderRun(input(2, "goliath", GOLIATH), store);
    }
    for (let round = 1; sqlite.poolAt(round).length > 0 || memory.poolAt(round).length > 0; round++) {
      expect(sqlite.poolAt(round)).toEqual(memory.poolAt(round));
    }
    expect(sqlite.champion()).toEqual(memory.champion());
  });

  test("a desynced seq throws and lands nowhere", () => {
    const store = new SqliteLadderStore(openDb(":memory:").db);
    store.addSnapshot({ runId: "a", round: 1, seq: 0, team: [TITAN] });
    expect(() => store.addSnapshot({ runId: "a", round: 1, seq: 2, team: [TITAN] })).toThrow(/desyncs/);
    expect(store.poolAt(1).length).toBe(1);
  });

  test("stored ghosts and champion are isolated from later caller mutation", () => {
    const store = new SqliteLadderStore(openDb(":memory:").db);
    const ghost: TeamSnapshot = { runId: "m", round: 1, seq: 0, team: [vanilla("Keeper", 5, 1)] };
    store.addSnapshot(ghost);
    store.setBoss(ghost.round, ghost);
    ghost.team[0]!.name = "Corrupted"; // mutation after write must not reach the store
    expect(store.poolAt(1)[0]!.team[0]!.name).toBe("Keeper");
    expect(store.champion()!.team[0]!.name).toBe("Keeper");
  });
});

describe("ghost ownership (the server-side dimension)", () => {
  test("poolVisibleTo excludes the user's ghosts across runs; bootstrap and others stay", () => {
    const store = freshSqlite();
    const base = store.poolLength(1);
    store.addGhost({ runId: "ada-run-1", round: 1, seq: base, team: [TITAN] }, "ada");
    store.addGhost({ runId: "ada-run-2", round: 1, seq: base + 1, team: [TITAN] }, "ada");
    store.addGhost({ runId: "bob-run-1", round: 1, seq: base + 2, team: [GOLIATH] }, "bob");

    expect(store.poolAt(1).length).toBe(base + 3); // the unfiltered (public) pool
    const adaSees = store.poolVisibleTo(1, "ada");
    expect(adaSees.map((g) => g.runId)).toEqual([
      ...Array<string>(base).fill(BOOTSTRAP_RUN_ID),
      "bob-run-1",
    ]);
    expect(adaSees[adaSees.length - 1]!.seq).toBe(base + 2); // true seqs survive the filter
    const bobSees = store.poolVisibleTo(1, "bob");
    expect(bobSees.map((g) => g.runId)).toEqual([
      ...Array<string>(base).fill(BOOTSTRAP_RUN_ID),
      "ada-run-1",
      "ada-run-2",
    ]);
  });

  test("champion history stays queryable by runId after a dethroning", () => {
    const store = freshSqlite();
    const bootstrap = store.champion()!;
    store.setChampionFor({ runId: "ada-run-1", round: 4, seq: 2, team: [TITAN] }, "ada");
    expect(store.championRecord()).toMatchObject({ userId: "ada", snap: { runId: "ada-run-1" } });
    expect(store.championByRunId(BOOTSTRAP_RUN_ID)!.snap).toEqual(bootstrap); // history, not just the seat
    expect(store.championByRunId("nobody")).toBeNull();
  });
});
