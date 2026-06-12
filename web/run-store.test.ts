// run-store tests — the localStorage-backed ladder and active-run persistence.
// Storage is injected, so a Map-backed stub drives the exact code main.ts
// wires to window.localStorage. Parity bar: a ladder behind localStorage and
// an InMemory one, given the same drives, hold the same pools and champion.

import { describe, expect, test } from "vitest";
import {
  InMemoryLadderStore,
  buy,
  initRun,
  ladderFight,
  openLadder,
  runToJSONL,
  stressRegistry,
  type LadderStore,
  type RunState,
  type UnitDef,
} from "../src/index.js";
import { clearRun, loadRun, nextRunId, openLocalLadder, saveRun, type KVStorage } from "./run-store.js";

function fakeStorage(): KVStorage {
  const m = new Map<string, string>();
  return {
    getItem: (k) => m.get(k) ?? null,
    setItem: (k, v) => void m.set(k, v),
    removeItem: (k) => void m.delete(k),
  };
}

const TITAN: UnitDef = { name: "Titan", base: { hp: 100, pwr: 50 } };

function playLadderRun(seed: number, runId: string, ladder: LadderStore): RunState {
  let s = buy(initRun({ seed, runId, pool: [TITAN], statuses: stressRegistry }), 0);
  while (s.status === "active") s = ladderFight(s, ladder);
  return s;
}

describe("openLocalLadder", () => {
  test("same drives as InMemory → same pools, champion, and run log", () => {
    const storage = fakeStorage();
    const local = openLadder(openLocalLadder(storage), stressRegistry);
    const inMemory = openLadder(new InMemoryLadderStore(), stressRegistry);
    const logs = [local, inMemory].map((store) => runToJSONL(playLadderRun(1, "titan", store).log));
    expect(logs[0]).toBe(logs[1]);
    for (let round = 1; local.poolAt(round).length > 0 || inMemory.poolAt(round).length > 0; round++) {
      expect(local.poolAt(round)).toEqual(inMemory.poolAt(round));
    }
    expect(local.champion()).toEqual(inMemory.champion());
  });

  test("write-through: a reopened ladder holds everything, and is never reseeded", () => {
    const storage = fakeStorage();
    const first = openLadder(openLocalLadder(storage), stressRegistry);
    playLadderRun(1, "titan", first);
    const reopened = openLadder(openLocalLadder(storage), stressRegistry); // a page reload
    for (let round = 1; first.poolAt(round).length > 0; round++) {
      expect(reopened.poolAt(round)).toEqual(first.poolAt(round));
    }
    expect(reopened.champion()).toEqual(first.champion());
  });

  test("corrupt stored ladder JSON throws loudly, never a silent fresh ladder", () => {
    const storage = fakeStorage();
    storage.setItem("aoi.ladder.v1", "not json");
    expect(() => openLocalLadder(storage)).toThrow(/not valid JSON/);
  });
});

describe("active run persistence", () => {
  test("a stored run (and its pending battle) round-trips; clearRun empties it", () => {
    const storage = fakeStorage();
    expect(loadRun(storage)).toBeNull(); // fresh profile: the new-run flow
    const state = buy(initRun({ seed: 7, runId: "web-1", pool: [TITAN], statuses: stressRegistry }), 0);
    const battle = { teamA: [TITAN], teamB: [TITAN], seed: 42, opponentLabel: "ghost bootstrap (round 1)" };
    saveRun(storage, state, battle);
    expect(loadRun(storage)).toEqual({ state, battle });
    saveRun(storage, state); // continue pressed: the battle record clears
    expect(loadRun(storage)).toEqual({ state });
    clearRun(storage);
    expect(loadRun(storage)).toBeNull();
  });

  test("a corrupt stored run is refused loudly", () => {
    const storage = fakeStorage();
    storage.setItem("aoi.run.v1", '{"status":"weird"}');
    expect(() => loadRun(storage)).toThrow(/not a RunState/);
  });

  test("nextRunId counts up through the stored counter — distinct ids per run", () => {
    const storage = fakeStorage();
    expect(nextRunId(storage)).toBe("web-1");
    expect(nextRunId(storage)).toBe("web-2"); // survives "reload": same storage, same counter
  });

  test("nextRunId: corrupt counter falls back to 0 and yields web-1", () => {
    const storage = fakeStorage();
    storage.setItem("aoi.run-seq.v1", "not-a-number");
    // No ladder data — base is 0, so n = 1.
    expect(nextRunId(storage)).toBe("web-1");
    // Counter is now written correctly; subsequent calls increment normally.
    expect(nextRunId(storage)).toBe("web-2");
  });

  test("nextRunId: corrupt counter with ladder data falls back to max web-N in ladder", () => {
    const storage = fakeStorage();
    storage.setItem("aoi.run-seq.v1", "NaN");
    // Simulate ladder JSON that contains run ids web-3 and web-5.
    storage.setItem("aoi.ladder.v1", JSON.stringify({ pools: [], champion: { runId: "web-5", round: 4, team: [] } }));
    // Should scan and find max = 5, so next = web-6.
    expect(nextRunId(storage)).toBe("web-6");
    expect(nextRunId(storage)).toBe("web-7");
  });
});

describe("abandon (#014): the run-lifecycle act, no kernel change", () => {
  // Abandon is clearRun + a client state reset; the ladder is untouched. These
  // tests pin the two semantics the PRD names: the abandoned run's ghosts (each
  // snapshotted before its fight) stay in the pools and the run never crowns,
  // and the stored run is gone so a reload lands on new-run.

  test("ladder integrity: a mid-run abandon leaves the fought rounds' ghosts, no crown, own-ghost exclusion intact next run", () => {
    const storage = fakeStorage();
    const ladder = openLadder(openLocalLadder(storage), stressRegistry);
    const championBefore = ladder.champion(); // the bootstrap champion holds the spot

    // web-1 fights round 1 once, then is abandoned mid-climb. ladderFight
    // snapshots the fielded team into round 1's pool BEFORE resolving, so the
    // ghost is on the ladder the instant it fights — win, lose, or abandon.
    const s1 = buy(initRun({ seed: 1, runId: "web-1", pool: [TITAN], statuses: stressRegistry }), 0);
    const afterFight = ladderFight(s1, ladder);
    expect(ladder.poolAt(1).some((g) => g.runId === "web-1")).toBe(true); // the ghost stays
    expect(afterFight.status).toBe("active"); // round 1 had ghosts to fight — not crowned

    // The abandon: only the stored run is cleared (the client also drops its
    // in-memory state). The ladder — pools and champion — is never touched.
    saveRun(storage, afterFight); // the run was persisted as it climbed
    expect(loadRun(storage)).not.toBeNull();
    clearRun(storage);
    expect(loadRun(storage)).toBeNull(); // reload would land on new-run

    // No crown from the abandoned run: the champion spot is exactly as it was
    // before web-1 ever fought — web-1 never reached it, abandon or not.
    const reopened = openLadder(openLocalLadder(storage), stressRegistry);
    expect(reopened.champion()).toEqual(championBefore);
    expect(reopened.champion()?.runId).not.toBe("web-1");
    expect(reopened.poolAt(1).some((g) => g.runId === "web-1")).toBe(true); // the ghost persists

    // The next run (web-2) sees web-1's ghost as an eligible rival; its own
    // ghosts are excluded. After web-2 fights, web-2 excludes its OWN ghost but
    // still sees web-1's — exclusion is by runId, unaffected by the abandon.
    const s2 = buy(initRun({ seed: 2, runId: "web-2", pool: [TITAN], statuses: stressRegistry }), 0);
    const web2Candidates = reopened.poolAt(s2.round).filter((g) => g.runId !== s2.runId);
    expect(web2Candidates.some((g) => g.runId === "web-1")).toBe(true);
    const s2AfterFight = ladderFight(s2, reopened);
    expect(s2AfterFight.log.some((e) => e.type === "OpponentDrawn")).toBe(true); // fought a ghost, not the vacant spot
    expect(reopened.poolAt(1).filter((g) => g.runId === "web-2").length).toBe(1); // web-2's own ghost now in the pool
    const web2OwnExcluded = reopened.poolAt(1).filter((g) => g.runId !== "web-2");
    expect(web2OwnExcluded.some((g) => g.runId === "web-1")).toBe(true); // web-1's ghost still eligible
  });

  test("stored-run-cleared: clearRun removes run + pending battle; ladder keys survive", () => {
    const storage = fakeStorage();
    const ladder = openLadder(openLocalLadder(storage), stressRegistry);
    const state = buy(initRun({ seed: 7, runId: "web-1", pool: [TITAN], statuses: stressRegistry }), 0);
    ladderFight(state, ladder); // writes a ghost to the ladder key
    const battle = { teamA: [TITAN], teamB: [TITAN], seed: 42, opponentLabel: "ghost web-1 (round 1)" };
    saveRun(storage, state, battle); // mid-battle: a pending battle is stored too
    expect(loadRun(storage)).toEqual({ state, battle });

    clearRun(storage); // the abandon
    expect(loadRun(storage)).toBeNull(); // both run and pending battle gone

    // The ladder is a separate key — abandoning a run never wipes it.
    const reopened = openLadder(openLocalLadder(storage), stressRegistry);
    expect(reopened.poolAt(1).some((g) => g.runId === "web-1")).toBe(true);
  });
});

describe("saveRun quota exhaustion", () => {
  function throwingStorage(): KVStorage {
    return {
      getItem: (_k) => null,
      setItem: (_k, _v) => {
        throw new DOMException("QuotaExceededError", "QuotaExceededError");
      },
      removeItem: (_k) => undefined,
    };
  }

  test("saveRun propagates a storage write failure (callers are responsible for catch)", () => {
    const storage = throwingStorage();
    const state = buy(initRun({ seed: 1, runId: "web-1", pool: [TITAN], statuses: stressRegistry }), 0);
    expect(() => saveRun(storage, state)).toThrow();
  });
});
