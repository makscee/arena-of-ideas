// Boss-challenge copy (#075 slice 4): the strings the run screen surfaces for
// the climb-vs-challenge decision and for every terminal end state are derived
// from the tower's shape (TOWER_HEIGHT), never typed. These tests assert the
// same pure exports the run screen renders, so a retune (a taller tower) cannot
// leave the UI lying — and each of the FOUR end reasons reads as its own state,
// the slice's "renderEnd handles all terminal states" pinned without a browser.

import { describe, expect, test } from "vitest";
import {
  InMemoryLadderStore,
  TOWER_HEIGHT,
  buy,
  challengeBoss,
  initRun,
  ladderFight,
  openLadder,
  stressRegistry,
  type LadderStore,
  type RunEndReason,
  type RunState,
  type UnitDef,
} from "../src/index.js";
import {
  bossFloorLine,
  challengeNoteLine,
  endHeadLine,
  isAboveTower,
  isSummitFloor,
} from "./run-screen.js";

describe("summit/above-tower derive from TOWER_HEIGHT", () => {
  test("the summit is exactly floor TOWER_HEIGHT", () => {
    expect(isSummitFloor(TOWER_HEIGHT)).toBe(true);
    expect(isSummitFloor(TOWER_HEIGHT - 1)).toBe(false);
    expect(isSummitFloor(TOWER_HEIGHT + 1)).toBe(false);
  });
  test("above-tower is strictly past the summit", () => {
    expect(isAboveTower(TOWER_HEIGHT)).toBe(false);
    expect(isAboveTower(TOWER_HEIGHT + 1)).toBe(true);
  });
});

describe("bossFloorLine names the floor and what its boss is", () => {
  test("a lower floor names the floor and that it carries a boss", () => {
    const line = bossFloorLine(1, true);
    expect(line).toContain("floor 1");
    expect(line).toContain(String(TOWER_HEIGHT)); // "of N" — the tower height is visible
    expect(line.toLowerCase()).toContain("boss");
  });
  test("the summit floor names the champion, not a generic boss", () => {
    const line = bossFloorLine(TOWER_HEIGHT, true);
    expect(line).toContain(`floor ${TOWER_HEIGHT}`);
    expect(line.toLowerCase()).toContain("champion");
  });
  test("above the top says there is NO boss here", () => {
    const line = bossFloorLine(TOWER_HEIGHT + 1, false);
    expect(line.toLowerCase()).toContain("no boss");
    expect(line.toLowerCase()).toContain("above the tower");
  });
});

describe("challengeNoteLine makes the decision legible — terminal + harder higher", () => {
  test("every present-boss note says it is terminal", () => {
    for (const floor of [1, TOWER_HEIGHT - 1, TOWER_HEIGHT]) {
      expect(challengeNoteLine(floor, true).toLowerCase()).toContain("terminal");
    }
  });
  test("a lower floor reads as a cash-out (easier than the summit)", () => {
    const note = challengeNoteLine(1, true).toLowerCase();
    expect(note).toContain("seat");
    expect(note).toContain("easier");
  });
  test("the summit note is the crown fight", () => {
    expect(challengeNoteLine(TOWER_HEIGHT, true).toLowerCase()).toContain("crown");
  });
  test("a vacant floor warns the challenge wins no crown", () => {
    expect(challengeNoteLine(TOWER_HEIGHT + 1, false).toLowerCase()).toContain("no crown");
  });
});

describe("endHeadLine — all four terminal reasons read distinctly", () => {
  const note = "the shipped boss falls; your team takes the seat";

  test("crown reads as the champion — an ascend over the summit", () => {
    const head = endHeadLine("crown", TOWER_HEIGHT + 1, note);
    expect(head).toContain("👑");
    expect(head.toLowerCase()).toContain("champion");
    expect(head).toContain(note);
  });
  test("seated reads as a lower floor seat, not the champion (a cash-out)", () => {
    const head = endHeadLine("seated", 1, note);
    expect(head).toContain("floor 1");
    expect(head.toLowerCase()).not.toContain("champion");
    expect(head).not.toContain("👑"); // a cash-out is not a crown
  });
  test("challenge-lost says the boss held, the run is over", () => {
    const head = endHeadLine("challenge-lost", 2, note).toLowerCase();
    expect(head).toContain("challenge lost");
    expect(head).toContain("floor 2");
    expect(head).toContain("over");
    expect(head).not.toContain("👑"); // never a crown
  });
  test("overshoot says climbed past the top, no crown", () => {
    const head = endHeadLine("overshoot", TOWER_HEIGHT + 1, note).toLowerCase();
    expect(head).toContain("overshot");
    expect(head).toContain("no crown");
    expect(head).not.toContain("👑");
  });
  test("out-of-lives keeps the climb-death wording", () => {
    const head = endHeadLine("out-of-lives", 3, note).toLowerCase();
    expect(head).toContain("out of lives");
    expect(head).toContain("over");
    expect(head).not.toContain("👑");
  });

  test("the five reasons produce five DISTINCT heads (no fall-through to a wrong state)", () => {
    const reasons: RunEndReason[] = ["crown", "seated", "challenge-lost", "overshoot", "out-of-lives"];
    const heads = reasons.map((r) => endHeadLine(r, TOWER_HEIGHT, note));
    expect(new Set(heads).size).toBe(reasons.length);
  });
});

// ---------------------------------------------------------------------------
// The flow the run screen drives, against the REAL bootstrapped tower: the
// climb → challenge moves the run-screen wires (ladderFight / challengeBoss)
// reach each terminal state the slice promises. The run screen is a thin shell
// over these kernel calls, so reaching them here is the must-fail-first proof
// that the challenge interaction lands its outcomes — a UI that wired the wrong
// kernel call, or a kernel that stopped reaching a state, breaks this.
// ---------------------------------------------------------------------------

const fresh = (): LadderStore => openLadder(new InMemoryLadderStore(), stressRegistry);

/** A run with one overwhelming unit, so it WINS every climb and challenge —
 * the crown / overshoot paths. A weak pool unit gives the loss path. */
const strong: UnitDef = { name: "Titan", base: { hp: 200, pwr: 99 } };
const weak: UnitDef = { name: "Mouse", base: { hp: 1, pwr: 1 } };

function startRun(unit: UnitDef): RunState {
  const s = initRun({ seed: 1, runId: "me", pool: [unit], statuses: stressRegistry });
  return buy(s, 0); // field the one unit
}

/** Climb from the current floor up to `targetFloor` (each climb advances a
 * floor); a win-everything team keeps its lives so the climb always continues. */
function climbTo(s: RunState, store: LadderStore, targetFloor: number): RunState {
  while (s.status === "active" && s.round < targetFloor) {
    s = ladderFight(s, store);
  }
  return s;
}

describe("the challenge flow reaches every terminal state on the seeded tower", () => {
  test("challenging a LOWER floor's boss and winning ends 'seated' (a cash-out, not a crown)", () => {
    const store = fresh();
    let s = startRun(strong);
    // Floor 1 has a seeded boss below the champion — challenge it straight away.
    expect(store.bossAt(1)).not.toBeNull();
    s = challengeBoss(s, store);
    expect(s.status).toBe("over");
    expect(s.endedBy).toBe("seated"); // a cash-out seat, NOT a crown
    expect(s.round).toBeLessThan(TOWER_HEIGHT); // a lower seat, not the summit
    expect(store.bossAt(1)!.runId).toBe("me"); // seated in place at floor 1
  });

  test("challenging the SUMMIT champion and winning ends 'crown' and ASCENDS to floor TOWER_HEIGHT+1", () => {
    const store = fresh();
    let s = startRun(strong);
    s = climbTo(s, store, TOWER_HEIGHT);
    expect(s.status).toBe("active");
    expect(s.round).toBe(TOWER_HEIGHT);
    expect(store.bossAt(TOWER_HEIGHT)).not.toBeNull();
    s = challengeBoss(s, store);
    expect(s.endedBy).toBe("crown"); // beating the champion crowns
    expect(s.round).toBe(TOWER_HEIGHT); // fought at the summit
    expect(store.champion()).toMatchObject({ runId: "me", round: TOWER_HEIGHT + 1 }); // ascended one floor
  });

  test("challenging a boss and LOSING ends 'challenge-lost', not out-of-lives", () => {
    const store = fresh();
    let s = startRun(weak); // loses the fight
    const livesBefore = s.lives;
    s = challengeBoss(s, store);
    expect(s.status).toBe("over");
    expect(s.endedBy).toBe("challenge-lost");
    expect(livesBefore).toBeGreaterThan(1); // terminal regardless of lives left
  });

  test("climbing PAST the top then challenging the vacant floor ends 'overshoot' (no crown)", () => {
    const store = fresh();
    let s = startRun(strong);
    // Climb up to and onto the floor ABOVE the tower's top: a win advances a
    // floor every fight, so at floor TOWER_HEIGHT a climb (not a challenge)
    // carries the run to TOWER_HEIGHT+1, which the bootstrap never seeded.
    s = climbTo(s, store, TOWER_HEIGHT);
    s = ladderFight(s, store); // climb off the summit into the void above
    expect(s.round).toBe(TOWER_HEIGHT + 1);
    expect(store.bossAt(s.round)).toBeNull(); // no boss above the tower
    s = challengeBoss(s, store);
    expect(s.endedBy).toBe("overshoot");
  });
});
