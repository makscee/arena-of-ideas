import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, test } from "vitest";
import { stressRegistry } from "./content/stress.js";
import { BOOTSTRAP_RUN_ID, InMemoryLadderStore, PersistedLadderStore, emptyLadderData, openLadder, parseLadderData } from "./ladder.js";
import type { LadderStore, TeamSnapshot } from "./ladder.js";
import { FileLadderStore } from "./ladder-file.js";
import { buy, challengeBoss, fight, initRun, InvalidDecisionError, ladderFight, reorder, reroll, runToJSONL } from "./run.js";
import type { RunEvent, RunInput, RunState } from "./run.js";
import { BOSS_TEAMS, BOOTSTRAP_DEPTH, BOOTSTRAP_TEAMS, STARTING_LIVES } from "./tunables.js";
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

/** Buy the one unit, then climb the ladder until the run ends. A climb draws a
 * same-floor ghost; when a floor has no climb opponent left, ladderFight
 * rejects loudly and the only move is to challenge the floor's boss — the
 * terminal move. So this mirrors the CLI policy: climb while possible, then
 * challenge the boss as the run's last act. */
function playLadderRun(inp: RunInput, ladder: LadderStore): RunState {
  let s = buy(initRun(inp), 0);
  while (s.status === "active") {
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

function freshLadder(): LadderStore {
  return openLadder(new InMemoryLadderStore(), stressRegistry);
}

// ---------------------------------------------------------------------------
// 1. Bootstrap: an empty ladder seeds rounds 1..DEPTH from the shipped teams
// ---------------------------------------------------------------------------

describe("bootstrap", () => {
  // The full bootstrap tower (075-3): floors 1..BOOTSTRAP_DEPTH carry a climb
  // pool AND a seated boss whose team is ALSO in that floor's pool as a ghost,
  // so each floor's pool is its climb teams plus one boss-ghost. The summit boss
  // sits one floor higher (BOOTSTRAP_DEPTH+1) with no pool-ghost — the guard a
  // first-ever run must beat to crown.
  const poolSizeAt = (floor: number) => BOOTSTRAP_TEAMS[floor - 1]!.length + 1; // climb teams + the boss-ghost

  test("openLadder seeds every floor 1..BOOTSTRAP_DEPTH with a climb pool and a boss-ghost", () => {
    const store = freshLadder();
    for (let round = 1; round <= BOOTSTRAP_DEPTH; round++) {
      const pool = store.poolAt(round);
      expect(pool.length).toBe(poolSizeAt(round)); // climb teams + the boss-ghost
      pool.forEach((g, i) => {
        expect(g).toMatchObject({ runId: BOOTSTRAP_RUN_ID, round, seq: i });
        expect(validateTeam(g.team, stressRegistry)).toEqual([]); // a first run's opponents pass the gate
      });
      // The boss seated on this floor IS the pool's last ghost (snapshot-then-seat),
      // so demoting it leaves its team drawable as a climb ghost on the floor.
      const boss = store.bossAt(round)!;
      expect(boss).toMatchObject({ runId: BOOTSTRAP_RUN_ID, round });
      expect(boss.team.map((u) => u.name)).toEqual(BOSS_TEAMS[round - 1]!.map((u) => u.name));
      expect(pool[pool.length - 1]!.team.map((u) => u.name)).toEqual(boss.team.map((u) => u.name));
    }
  });

  test("the summit boss is seated on floor BOOTSTRAP_DEPTH+1 with NO pool-ghost — the guard", () => {
    const store = freshLadder();
    const summitFloor = BOOTSTRAP_DEPTH + 1;
    // Empty pool on the summit floor: a run that sailed up the climb floors finds
    // no ghost to climb here, so it MUST challenge the boss — the crown is earned.
    expect(store.poolAt(summitFloor)).toEqual([]);
    const summit = store.bossAt(summitFloor)!;
    expect(summit).toMatchObject({ runId: BOOTSTRAP_RUN_ID, round: summitFloor });
    expect(summit.team.map((u) => u.name)).toEqual(BOSS_TEAMS[BOOTSTRAP_DEPTH]!.map((u) => u.name));
  });

  test("the champion derives to the summit — a fresh ladder's spot is never vacant", () => {
    const champ = freshLadder().champion()!;
    expect(champ).toMatchObject({ runId: BOOTSTRAP_RUN_ID, round: BOOTSTRAP_DEPTH + 1 });
    expect(champ.team.map((u) => u.name)).toEqual(BOSS_TEAMS[BOOTSTRAP_DEPTH]!.map((u) => u.name));
    expect(validateTeam(champ.team, stressRegistry)).toEqual([]); // the gate covers the seat too
  });

  test("a non-empty ladder is never reseeded", () => {
    const store = freshLadder();
    openLadder(store, stressRegistry);
    expect(store.poolAt(1).length).toBe(poolSizeAt(1));
  });

  test("a played-on ladder keeps its earned champion across opens", () => {
    const store = freshLadder();
    playLadderRun(input(1, "titan", TITAN), store); // dethrones the bootstrap champion
    openLadder(store, stressRegistry);
    expect(store.champion()!.runId).toBe("titan"); // never reseated
  });

  test("every seeded boss + climb team passes the content gate", () => {
    const store = freshLadder();
    for (let floor = 1; floor <= BOOTSTRAP_DEPTH + 1; floor++) {
      const boss = store.bossAt(floor)!;
      expect(validateTeam(boss.team, stressRegistry)).toEqual([]);
    }
    for (let floor = 1; floor <= BOOTSTRAP_DEPTH; floor++) {
      for (const g of store.poolAt(floor)) expect(validateTeam(g.team, stressRegistry)).toEqual([]);
    }
  });

  test("a bootstrap team failing the content gate fails at open, loudly", () => {
    // An empty registry leaves Venomancer's Poison dangling — the gate must
    // catch it at seed time, not seed-dependently mid-run on an unlucky draw.
    expect(() => openLadder(new InMemoryLadderStore(), {})).toThrow(ValidationError);
    expect(() => openLadder(new InMemoryLadderStore(), {})).toThrow(/bootstrap round 1 team 0/);
  });

  test("a first-ever run climbs a real pool at every floor, then earns the crown by beating the summit boss", () => {
    // The "fresh ladder never auto-crowns trivially" property (075's original
    // pin): a strong run climbs floors 1..BOOTSTRAP_DEPTH (a real fight each) and
    // crowns ONLY by challenging the seated summit — dethroning someone, never a
    // free vacant-spot seat.
    const store = freshLadder();
    const s = playLadderRun(input(1, "titan", TITAN), store);
    expect(s).toMatchObject({ status: "over", endedBy: "crown" });
    expect(s.round).toBe(BOOTSTRAP_DEPTH + 1); // crowned at the summit, not a higher vacant floor
    // One fight per climbed floor plus the summit challenge — a real climb, no skip.
    expect(ofType(s.log, "FightFought").length).toBe(BOOTSTRAP_DEPTH + 1);
    const crowned = ofType(s.log, "Crowned")[0]!;
    expect(crowned).toMatchObject({ floor: BOOTSTRAP_DEPTH + 1, dethroned: BOOTSTRAP_RUN_ID }); // earned, not vacant
  });

  test("a first-ever weak run never auto-crowns — it loses the summit challenge", () => {
    // The dual of the property: a losing-record run still advances a floor per
    // climb (a loss costs a life but moves up), reaches the summit, and CHALLENGES
    // it — and loses, ending challenge-lost, never crowning through a vacant spot.
    const store = freshLadder();
    const s = playLadderRun(input(1, "grunt", GRUNT), store);
    expect(s).toMatchObject({ status: "over", endedBy: "challenge-lost", round: BOOTSTRAP_DEPTH + 1 });
    expect(ofType(s.log, "Crowned")).toEqual([]); // no free crown
    expect(store.bossAt(BOOTSTRAP_DEPTH + 1)!.runId).toBe(BOOTSTRAP_RUN_ID); // summit stands
  });

  test("demoting a SEEDED lower boss keeps its team in the floor's pool as a climb ghost", () => {
    // The demote-keeps-ghost invariant for bootstrap bosses (not only run-won
    // ones): challenge floor 1's seeded boss head-on and win; the dethroned
    // bootstrap boss's team must still be drawable as a climb ghost on floor 1.
    const store = freshLadder();
    const floor1Boss = store.bossAt(1)!;
    const bossNames = floor1Boss.team.map((u) => u.name);
    const s = challengeBoss(buy(initRun(input(1, "titan", TITAN)), 0), store);
    expect(s).toMatchObject({ status: "over", endedBy: "crown", round: 1 });
    expect(store.bossAt(1)!.runId).toBe("titan"); // challenger seated over the slot
    // The dethroned bootstrap boss's team is still in floor 1's pool, drawable.
    const ghosts = store.poolAt(1).filter((g) => g.runId === BOOTSTRAP_RUN_ID && g.team.map((u) => u.name).join(",") === bossNames.join(","));
    expect(ghosts.length).toBe(1);
    expect(ghosts[0]!.team.map((u) => u.name)).toEqual(BOSS_TEAMS[0]!.map((u) => u.name));
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
    // The run's own ghost enters after floor 1's seeded ghosts: the climb teams
    // AND the floor-1 boss-ghost (075-3's per-floor boss in the pool).
    expect(ghost).toMatchObject({ round: 1, seq: BOOTSTRAP_TEAMS[0]!.length + 1 });
    expect(ghost.team.map((u) => u.name)).toEqual(["Titan"]);
    // The log shows the order: Snapshotted, then the draw, then the fight.
    const types = s.log.map((e) => e.type);
    expect(types.indexOf("Snapshotted")).toBeLessThan(types.indexOf("OpponentDrawn"));
    expect(types.indexOf("OpponentDrawn")).toBeLessThan(types.indexOf("FightFought"));
  });

  test("ghosts persist after the run dies", () => {
    const store = new InMemoryLadderStore();
    // Every floor the run reaches holds an unbeatable wall ghost, so the run
    // loses each climb and dies out-of-lives on the climb (never reaching a
    // vacant floor it could auto-seat). Its own ghost is still left at each
    // round it fought — snapshot-before-fight, even on a doomed run.
    for (let round = 1; round <= STARTING_LIVES; round++) {
      store.addSnapshot({ runId: "wall", round, seq: 0, team: [vanilla("Wall", 1000, 500)] });
    }
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

  test("a gate-failing draw aborts before the run's own ghost persists", () => {
    const store = new InMemoryLadderStore();
    // A poisoned pool: one ghost whose status resolves in no registry. The
    // gate runs on the draw BEFORE addSnapshot, so a retried fight must not
    // grow the pool with the aborted attempt's ghost on every try.
    store.addSnapshot({
      runId: "evil",
      round: 1,
      seq: 0,
      team: [{ name: "Hexed", base: { hp: 5, pwr: 1 }, statuses: [{ status: "Bogus", stacks: 1 }] }],
    });
    const s0 = buy(initRun(input(3, "titan", TITAN)), 0);
    expect(() => ladderFight(s0, store)).toThrow(ValidationError);
    expect(() => ladderFight(s0, store)).toThrow(ValidationError); // the retry…
    expect(store.poolAt(1).map((g) => g.runId)).toEqual(["evil"]); // …never grew the pool
  });
});

// ---------------------------------------------------------------------------
// 3. Opponent draw: seeded, deterministic, own ghosts excluded
// ---------------------------------------------------------------------------

describe("opponent draw", () => {
  test("a run's own ghosts are excluded from its draw", () => {
    for (let seed = 0; seed < 20; seed++) {
      const store = freshLadder();
      // Floor 1 already holds its seeded ghosts (climb teams + the boss-ghost) at
      // seq 0..seeded-1; append the run's OWN ghosts after them, as if it had
      // re-opened mid-ladder. Only the seeded bootstrap ghosts are draw candidates
      // — the run's own are excluded — so the draw always lands on a bootstrap ghost.
      const seeded = BOOTSTRAP_TEAMS[0]!.length + 1; // climb teams + boss-ghost
      for (const seq of [seeded, seeded + 1, seeded + 2]) {
        store.addSnapshot({ runId: "self", round: 1, seq, team: [TITAN] });
      }
      const s = ladderFight(buy(initRun(input(seed, "self", TITAN)), 0), store);
      const drawn = ofType(s.log, "OpponentDrawn")[0]!;
      expect(drawn.opponent).toBe(BOOTSTRAP_RUN_ID);
      expect(drawn.candidates).toBe(seeded); // own ghosts excluded, the seeded bootstrap ghosts remain
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
// 5. The boss challenge: the explicit terminal move (challengeBoss)
// ---------------------------------------------------------------------------

describe("boss challenge", () => {
  test("a fresh ladder's crown is earned — the floor's boss must fall", () => {
    const store = freshLadder();
    const s = playLadderRun(input(1, "titan", TITAN), store);
    // Rounds 1..DEPTH climbed bootstrap ghosts; past them the climb is refused
    // (only the run's own ghosts remain) so the run challenges the floor's boss
    // — the seated bootstrap champion — and seats over it.
    expect(s).toMatchObject({ status: "over", endedBy: "crown" });
    expect(ofType(s.log, "BossChallenged")[0]).toMatchObject({ floor: s.round, boss: BOOTSTRAP_RUN_ID });
    expect(ofType(s.log, "Crowned")[0]).toMatchObject({ floor: s.round, dethroned: BOOTSTRAP_RUN_ID });
    expect(s.log[s.log.length - 1]).toMatchObject({ type: "RunEnded", reason: "crown" });
    expect(store.bossAt(s.round)).toMatchObject({ runId: "titan", round: s.round });
    expect(store.champion()).toMatchObject({ runId: "titan", round: s.round });
  });

  test("a win seats the challenger and the old boss's ghost stays in the floor's pool", () => {
    const store = new InMemoryLadderStore();
    // A beatable boss seated on floor 1, with its own ghost already in floor 1's
    // pool (as a prior run would have left). Challenging directly (the floor's
    // climb pool is the boss's ghost — but challengeBoss does not draw a climb,
    // it fights the SEATED boss): a win seats the challenger over the slot, and
    // the unseated boss's ghost must remain in the pool — pool-only, not removed.
    store.setBoss(1, { runId: "weak-boss", round: 1, seq: 0, team: [GRUNT] });
    store.addSnapshot({ runId: "weak-boss", round: 1, seq: 0, team: [GRUNT] }); // its ghost in the pool
    const s = challengeBoss(buy(initRun(input(1, "titan", TITAN)), 0), store);
    expect(s).toMatchObject({ status: "over", endedBy: "crown", round: 1 });
    expect(ofType(s.log, "Crowned")[0]).toMatchObject({ floor: 1, dethroned: "weak-boss" });
    expect(store.bossAt(1)).toMatchObject({ runId: "titan" }); // challenger seated over the slot
    expect(store.poolAt(1).some((g) => g.runId === "weak-boss")).toBe(true); // unseated boss's ghost stays
    expect(store.poolAt(1).some((g) => g.runId === "titan")).toBe(true); // challenger's ghost is there too
  });

  test("a lost challenge ends the run without seating — terminal, lives or not", () => {
    const store = freshLadder();
    // A boss no Titan beats, on the floor just past the bootstrap climb. Titan
    // climbs the ghost rounds, then challenges and loses — the run ends, the
    // boss stands, and no Crowned event is emitted. Titan still has lives, so
    // this proves a lost challenge is terminal regardless of lives.
    const bossFloor = BOOTSTRAP_DEPTH + 1;
    store.setBoss(bossFloor, { runId: "wall", round: bossFloor, seq: 0, team: [vanilla("Wall", 1000, 500)] });
    const s = playLadderRun(input(2, "titan", TITAN), store);
    expect(s).toMatchObject({ status: "over", endedBy: "challenge-lost" });
    expect(s.lives).toBeGreaterThan(0); // a life remained — the loss was terminal anyway
    expect(ofType(s.log, "BossChallenged")[0]).toMatchObject({ boss: "wall" });
    expect(ofType(s.log, "Crowned")).toEqual([]); // no seat
    expect(s.log[s.log.length - 1]).toMatchObject({ type: "RunEnded", reason: "challenge-lost" });
    expect(store.bossAt(bossFloor)!.runId).toBe("wall"); // boss stands
  });

  test("a draw on a challenge is also terminal, challenge-lost, no seat", () => {
    const store = new InMemoryLadderStore();
    // A boss whose 0-power team can never kill, against a 0-power challenger:
    // the battle draws. A draw is not a win, so the run ends challenge-lost.
    const stalemate = vanilla("Inert", 50, 0);
    store.setBoss(1, { runId: "boss", round: 1, seq: 0, team: [stalemate] });
    const s = challengeBoss(buy(initRun(input(4, "me", vanilla("Mirror", 50, 0))), 0), store);
    expect(ofType(s.log, "FightFought")[0]!.winner).toBe("draw");
    expect(s).toMatchObject({ status: "over", endedBy: "challenge-lost" });
    expect(store.bossAt(1)!.runId).toBe("boss"); // unseated
  });

  test("a vacant floor auto-seats — the kept kernel edge, no battle", () => {
    const store = new InMemoryLadderStore(); // unopened: floor 1 is vacant
    const s = challengeBoss(buy(initRun(input(1, "titan", TITAN)), 0), store);
    expect(s).toMatchObject({ status: "over", endedBy: "crown", round: 1 });
    expect(ofType(s.log, "BossChallenged")[0]).toMatchObject({ floor: 1, boss: null });
    expect(ofType(s.log, "FightFought")).toEqual([]); // crowned without a battle
    expect(ofType(s.log, "Crowned")[0]).toMatchObject({ floor: 1, dethroned: null });
    expect(store.bossAt(1)).toMatchObject({ runId: "titan", round: 1 });
    expect(store.champion()).toMatchObject({ runId: "titan", round: 1 });
  });

  test("the snapshot precedes the challenge: the challenger ghosts into its floor's pool first", () => {
    const store = new InMemoryLadderStore();
    const s = challengeBoss(buy(initRun(input(1, "titan", TITAN)), 0), store);
    // On a win, the seated boss IS the snapshotted ghost — same team, in the pool.
    expect(store.poolAt(1).some((g) => g.runId === "titan")).toBe(true);
    const types = s.log.map((e) => e.type);
    expect(types.indexOf("Snapshotted")).toBeLessThan(types.indexOf("BossChallenged"));
  });

  test("a won challenge rejects every further decision — the run is over", () => {
    const store = new InMemoryLadderStore();
    const over = challengeBoss(buy(initRun(input(1, "titan", TITAN)), 0), store);
    expect(over.status).toBe("over");
    for (const d of [
      () => buy(over, 0),
      () => fight(over, [GRUNT]),
      () => ladderFight(over, freshLadder()),
      () => challengeBoss(over, store),
    ]) {
      expect(d).toThrow(InvalidDecisionError);
      expect(d).toThrow(/run is over \(crown\)/);
    }
  });

  test("challengeBoss on an empty line throws — buy a unit first", () => {
    const store = new InMemoryLadderStore();
    expect(() => challengeBoss(initRun(input(1, "titan", TITAN)), store)).toThrow(/line is empty/);
  });

  test("challengeBoss never mutates its input state", () => {
    const store = freshLadder();
    const s0 = buy(initRun(input(7, "titan", TITAN)), 0);
    const snapshot = JSON.stringify(s0);
    challengeBoss(s0, store);
    expect(JSON.stringify(s0)).toBe(snapshot);
  });

  test("a seated boss persists across runs; the next crown seats one floor higher", () => {
    // The open-ended tower's shape: each crowned run leaves a climb ghost on its
    // own floor, so the next runner climbs that ghost and seats one floor above.
    // The earlier boss seat is never removed — it persists; the champion (the
    // highest seat) advances. A boss falls from the SUMMIT only by being out-
    // climbed, never silently overwritten in place.
    const store = freshLadder();
    const titan = playLadderRun(input(1, "titan", TITAN), store);
    const titanFloor = titan.round;
    expect(store.bossAt(titanFloor)).toMatchObject({ runId: "titan" });
    const goliath = playLadderRun(input(2, "goliath", GOLIATH), store);
    expect(goliath).toMatchObject({ status: "over", endedBy: "crown" });
    expect(goliath.round).toBeGreaterThan(titanFloor); // seated above titan
    expect(store.bossAt(titanFloor)).toMatchObject({ runId: "titan" }); // titan's seat persists
    expect(store.champion()).toMatchObject({ runId: "goliath", round: goliath.round }); // the new summit
  });

  test("a directly-challenged boss falls only to a winner", () => {
    // Seat a boss on a vacant floor and challenge it head-on (no climb ghost in
    // the way): a beatable boss is dethroned and replaced; an unbeatable one
    // stands and the challenge is lost. Same floor, opposite outcomes.
    const weak = new InMemoryLadderStore();
    weak.setBoss(1, { runId: "weak", round: 1, seq: 0, team: [GRUNT] });
    const won = challengeBoss(buy(initRun(input(1, "titan", TITAN)), 0), weak);
    expect(won.endedBy).toBe("crown");
    expect(weak.bossAt(1)).toMatchObject({ runId: "titan" }); // beaten boss replaced

    const strong = new InMemoryLadderStore();
    strong.setBoss(1, { runId: "strong", round: 1, seq: 0, team: [vanilla("Wall", 1000, 500)] });
    const lost = challengeBoss(buy(initRun(input(1, "titan", TITAN)), 0), strong);
    expect(lost.endedBy).toBe("challenge-lost");
    expect(strong.bossAt(1)).toMatchObject({ runId: "strong" }); // unbeaten boss stands
  });

  test("the unseated boss's team stays in the pools as a ghost", () => {
    const store = freshLadder();
    const titan = playLadderRun(input(1, "titan", TITAN), store);
    playLadderRun(input(2, "goliath", GOLIATH), store);
    const rounds = Array.from({ length: titan.round }, (_, i) => i + 1);
    const titanGhosts = rounds.flatMap((r) => store.poolAt(r).filter((g) => g.runId === "titan"));
    expect(titanGhosts.length).toBe(titan.round); // one per round it climbed, challenge round included
    expect(titanGhosts.every((g) => g.team[0]!.name === "Titan")).toBe(true);
  });

  test("champion() derives the boss of the highest occupied floor", () => {
    const store = new InMemoryLadderStore(); // unopened: an empty tower
    expect(store.champion()).toBeNull(); // no floor occupied
    // Seat bosses out of floor order, to prove the read is by max floor, not by
    // insertion: floor 3 is the summit whichever order the seats arrive in.
    store.setBoss(2, { runId: "mid", round: 2, seq: 0, team: [vanilla("Mid", 10, 5)] });
    store.setBoss(5, { runId: "top", round: 5, seq: 0, team: [vanilla("Top", 30, 15)] });
    store.setBoss(1, { runId: "low", round: 1, seq: 0, team: [vanilla("Low", 5, 1)] });
    expect(store.champion()).toMatchObject({ runId: "top", round: 5 });
    expect(store.bossAt(2)).toMatchObject({ runId: "mid" }); // lower floors keep their bosses
    expect(store.bossAt(4)).toBeNull(); // a vacant floor reads null

    // Multi-digit floors: the summit is the NUMERIC max, not the lexical one —
    // the tower is open-ended (PRD 075) and climbs past floor 9. Lexically
    // "10" < "7" and "100" < "11" < "9", so a string comparison would crown the
    // wrong floor here; deriveChampion's Number() keeps floor 100 the summit.
    const wide = new InMemoryLadderStore();
    for (const floor of [9, 11, 7, 100, 10]) {
      wide.setBoss(floor, { runId: `f${floor}`, round: floor, seq: 0, team: [vanilla(`F${floor}`, 10, 5)] });
    }
    expect(wide.champion()).toMatchObject({ runId: "f100", round: 100 });
  });

});

// ---------------------------------------------------------------------------
// 5b. ladderFight is a pure climb: an empty draw rejects, and never grows the pool
// ---------------------------------------------------------------------------

describe("ladderFight empty draw", () => {
  test("an empty climb draw throws InvalidDecisionError naming the floor", () => {
    const store = new InMemoryLadderStore(); // unopened: round-1 pool is empty
    const s0 = buy(initRun(input(1, "titan", TITAN)), 0);
    expect(() => ladderFight(s0, store)).toThrow(InvalidDecisionError);
    expect(() => ladderFight(s0, store)).toThrow(/no climb opponent at floor 1 — challenge the boss instead/);
  });

  test("a rejected empty draw does NOT grow the pool — no ghost on the aborted attempt", () => {
    const store = new InMemoryLadderStore();
    // Pre-seed round 1 with only the run's OWN ghost: candidates (others) are
    // empty, so the climb is rejected — and the rejection must come BEFORE the
    // snapshot, so retrying never appends another own-ghost on each try.
    store.addSnapshot({ runId: "titan", round: 1, seq: 0, team: [TITAN] });
    const s0 = buy(initRun(input(1, "titan", TITAN)), 0);
    expect(() => ladderFight(s0, store)).toThrow(/no climb opponent/);
    expect(() => ladderFight(s0, store)).toThrow(/no climb opponent/); // the retry…
    expect(store.poolAt(1).length).toBe(1); // …never grew the pool past the pre-seeded ghost
  });

  test("a climb loss costs a life; 0 lives ends the run out-of-lives", () => {
    const store = new InMemoryLadderStore();
    // Every floor the run reaches holds an unbeatable wall: each climb is a loss
    // costing a life, round after round, until the last life is spent — a pure
    // climb death, no boss challenge ever (every floor has a climb opponent).
    for (let round = 1; round <= STARTING_LIVES; round++) {
      store.addSnapshot({ runId: "wall", round, seq: 0, team: [vanilla("Wall", 1000, 500)] });
    }
    let s = buy(initRun(input(3, "grunt", GRUNT)), 0);
    while (s.status === "active") s = ladderFight(s, store);
    expect(s).toMatchObject({ status: "over", endedBy: "out-of-lives", lives: 0 });
    expect(ofType(s.log, "Crowned")).toEqual([]); // a climb death never crowns
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
    const a = play();
    // The replayed run ends in a boss challenge — so this byte-comparison
    // covers the challengeBoss path (snapshot, BossChallenged, Crowned, RunEnded).
    expect(a.endedBy).toBe("crown");
    expect(ofType(a.log, "BossChallenged").length).toBe(1);
    expect(runToJSONL(a.log)).toBe(runToJSONL(play().log));
  });
});

// ---------------------------------------------------------------------------
// 7. File backing: JSON on disk behind the same interface
// ---------------------------------------------------------------------------

describe("file backing", () => {
  const dir = mkdtempSync(join(tmpdir(), "ladder-"));

  test("write through one store, read through a fresh one — equal", () => {
    const path = join(dir, "roundtrip.json");
    const store = openLadder(new FileLadderStore(path), stressRegistry);
    const ghost: TeamSnapshot = { runId: "titan", round: 1, seq: BOOTSTRAP_TEAMS[0]!.length + 1, team: [TITAN] }; // after climb teams + boss-ghost
    store.addSnapshot(ghost);
    store.setBoss(ghost.round, ghost);
    const reread = new FileLadderStore(path);
    expect(reread.poolAt(1)).toEqual(store.poolAt(1));
    expect(reread.champion()).toEqual(store.champion());
    expect(reread.poolAt(BOOTSTRAP_DEPTH + 2)).toEqual([]); // untouched rounds past the summit stay empty
  });

  test("a ladder run plays byte-identically on file and in-memory backings", () => {
    const onFile = playLadderRun(input(1, "titan", TITAN), openLadder(new FileLadderStore(join(dir, "parity.json")), stressRegistry));
    const inMemory = playLadderRun(input(1, "titan", TITAN), freshLadder());
    expect(runToJSONL(onFile.log)).toBe(runToJSONL(inMemory.log));
  });

  test("stored ghosts and champion are isolated from later caller mutation", () => {
    const path = join(dir, "clone.json");
    const store = new FileLadderStore(path);
    const ghost: TeamSnapshot = { runId: "m", round: 1, seq: 0, team: [vanilla("Keeper", 5, 1)] };
    store.addSnapshot(ghost);
    store.setBoss(ghost.round, ghost);
    ghost.team[0]!.name = "Corrupted"; // mutation after write must not reach the store…
    expect(store.poolAt(1)[0]!.team[0]!.name).toBe("Keeper");
    expect(store.champion()!.team[0]!.name).toBe("Keeper");
    store.addSnapshot({ runId: "m", round: 2, seq: 0, team: [TITAN] }); // …or the next disk persist
    expect(new FileLadderStore(path).champion()!.team[0]!.name).toBe("Keeper");
  });

  test("a missing file opens an empty ladder; a corrupt one throws loudly", () => {
    expect(new FileLadderStore(join(dir, "missing.json")).poolAt(1)).toEqual([]);
    const corrupt = join(dir, "corrupt.json");
    writeFileSync(corrupt, "not json", "utf8");
    expect(() => new FileLadderStore(corrupt)).toThrow(/not valid JSON/);
  });
});

// ---------------------------------------------------------------------------
// 8. The seq precondition: enforced by both backings, not a comment
// ---------------------------------------------------------------------------

describe("addSnapshot seq precondition", () => {
  test("a desynced seq throws in both backings and lands nowhere", () => {
    const backings: LadderStore[] = [
      new InMemoryLadderStore(),
      new FileLadderStore(join(mkdtempSync(join(tmpdir(), "ladder-seq-")), "seq.json")),
    ];
    for (const store of backings) {
      store.addSnapshot({ runId: "a", round: 1, seq: 0, team: [TITAN] });
      // A wrong seq means the caller drew from a pool other than the one it
      // is writing to — rejected, not silently appended under a false ordinal.
      expect(() => store.addSnapshot({ runId: "a", round: 1, seq: 2, team: [TITAN] })).toThrow(/desyncs/);
      expect(store.poolAt(1).length).toBe(1);
    }
  });
});

// ---------------------------------------------------------------------------
// 9. The pool gate: initRun rejects bad content and duplicate names
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

// ---------------------------------------------------------------------------
// 10. PersistedLadderStore: the shared engine of every persistent backing
// ---------------------------------------------------------------------------

describe("PersistedLadderStore", () => {
  /** A localStorage-like medium: the last persisted JSON string. */
  function memoryMedium(): { read(): string | null; store(): LadderStore } {
    let raw: string | null = null;
    return {
      read: () => raw,
      store: () =>
        new PersistedLadderStore(raw === null ? emptyLadderData() : parseLadderData(raw, "memory"), (d) => {
          raw = JSON.stringify(d);
        }),
    };
  }

  test("same drives as InMemory → same pools and champion, byte-identical run logs", () => {
    const medium = memoryMedium();
    const persisted = openLadder(medium.store(), stressRegistry);
    const inMemory = freshLadder();
    const logs = [persisted, inMemory].map((store) => {
      playLadderRun(input(1, "titan", TITAN), store);
      return runToJSONL(playLadderRun(input(2, "goliath", GOLIATH), store).log);
    });
    expect(logs[0]).toBe(logs[1]);
    for (let round = 1; persisted.poolAt(round).length > 0 || inMemory.poolAt(round).length > 0; round++) {
      expect(persisted.poolAt(round)).toEqual(inMemory.poolAt(round));
    }
    expect(persisted.champion()).toEqual(inMemory.champion());
  });

  test("every mutation writes through — a store reopened from the medium is equal", () => {
    const medium = memoryMedium();
    const store = openLadder(medium.store(), stressRegistry);
    playLadderRun(input(1, "titan", TITAN), store);
    const reopened = medium.store(); // parses the last persisted JSON
    for (let round = 1; store.poolAt(round).length > 0; round++) {
      expect(reopened.poolAt(round)).toEqual(store.poolAt(round));
    }
    expect(reopened.champion()).toEqual(store.champion());
  });

  test("corrupt stored ladders are refused loudly, not silently reset", () => {
    expect(() => parseLadderData("not json", "memory")).toThrow(/not valid JSON/);
    expect(() => parseLadderData('{"champion":null}', "memory")).toThrow(/no pools object/);
  });

  test("a multi-floor boss map round-trips byte-equivalent through file and localStorage backings", () => {
    // Build a tower with bosses on several floors plus a couple of pools, seat
    // them through a store, and check both persistent media round-trip the same
    // bytes — the boss map is not a single spot but a per-floor record now.
    const dir = mkdtempSync(join(tmpdir(), "ladder-tower-"));
    const seat = (store: LadderStore) => {
      store.addSnapshot({ runId: "a", round: 1, seq: 0, team: [TITAN] });
      store.setBoss(2, { runId: "b", round: 2, seq: 0, team: [vanilla("Two", 20, 10)] });
      store.setBoss(7, { runId: "c", round: 7, seq: 0, team: [GOLIATH] }); // the summit
      store.setBoss(4, { runId: "d", round: 4, seq: 0, team: [GRUNT] });
    };

    // File backing: seat, then read the bytes off disk.
    const path = join(dir, "tower.json");
    const fileStore = new FileLadderStore(path);
    seat(fileStore);
    const fileBytes = readFileSync(path, "utf8");

    // localStorage-like backing: the same seats, the persisted string.
    const medium = memoryMedium();
    const memStore = medium.store();
    seat(memStore);
    const memBytes = medium.read()!;

    // Both backings derive the same champion (floor 7, the highest occupied)…
    expect(fileStore.champion()).toMatchObject({ runId: "c", round: 7 });
    expect(memStore.champion()).toEqual(fileStore.champion());
    // …and the serialized LadderData is the same object, modulo formatting:
    // the file backing pretty-prints, the medium does not, so compare parsed.
    expect(JSON.parse(memBytes)).toEqual(JSON.parse(fileBytes));
    // Reopened from its own bytes, each backing is byte-stable: persist again
    // off a fresh parse yields identical bytes (no drift through the round-trip).
    medium.store(); // parses the persisted string without throwing
    expect(JSON.stringify(parseLadderData(memBytes, "memory"))).toBe(memBytes);
    const reread = new FileLadderStore(path);
    reread.addSnapshot({ runId: "a", round: 1, seq: 1, team: [TITAN] }); // a write re-persists
    expect(reread.bossAt(7)).toMatchObject({ runId: "c", round: 7 }); // bosses survived the reopen
    expect(reread.bossAt(4)).toMatchObject({ runId: "d" });
  });
});
