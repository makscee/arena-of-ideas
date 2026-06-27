// run-store tests — the localStorage-backed ladder and active-run persistence.
// Storage is injected, so a Map-backed stub drives the exact code main.ts
// wires to window.localStorage. Parity bar: a ladder behind localStorage and
// an InMemory one, given the same drives, hold the same pools and champion.

import { describe, expect, test } from "vitest";
import {
  InMemoryLadderStore,
  buy,
  challengeBoss,
  initRun,
  InvalidDecisionError,
  ladderFight,
  seedBootstrapTower,
  runToJSONL,
  stressAbilities,
  stressRegistry,
  type LadderStore,
  type RunState,
  type UnitDef,
} from "../src/index.js";
import {
  clearRun,
  clearSession,
  loadDevMode,
  loadRun,
  loadSession,
  loadSubmitResult,
  nextRunId,
  openLocalLadder,
  prefixedStorage,
  saveRun,
  saveSession,
  saveSubmitResult,
  setDevMode,
  type KVStorage,
} from "./run-store.js";

function fakeStorage(): KVStorage {
  const m = new Map<string, string>();
  return {
    getItem: (k) => m.get(k) ?? null,
    setItem: (k, v) => void m.set(k, v),
    removeItem: (k) => void m.delete(k),
  };
}

const TITAN: UnitDef = { name: "Titan", base: { hp: 100, pwr: 50 }, ability: "Strike" };

function playLadderRun(seed: number, runId: string, ladder: LadderStore): RunState {
  let s = buy(initRun({ seed, runId, pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
  while (s.status === "active") {
    try {
      s = ladderFight(s, ladder);
    } catch (err) {
      // Empty climb pool — challenge the floor's boss, the terminal move.
      if (err instanceof InvalidDecisionError && err.decision === "fight") {
        s = challengeBoss(s, ladder);
      } else {
        throw err;
      }
    }
  }
  return s;
}

describe("openLocalLadder", () => {
  test("same drives as InMemory → same pools, champion, and run log", () => {
    const storage = fakeStorage();
    const local = seedBootstrapTower(openLocalLadder(storage), stressRegistry, stressAbilities);
    const inMemory = seedBootstrapTower(new InMemoryLadderStore(), stressRegistry, stressAbilities);
    const logs = [local, inMemory].map((store) => runToJSONL(playLadderRun(1, "titan", store).log));
    expect(logs[0]).toBe(logs[1]);
    for (let round = 1; local.poolAt(round).length > 0 || inMemory.poolAt(round).length > 0; round++) {
      expect(local.poolAt(round)).toEqual(inMemory.poolAt(round));
    }
    expect(local.champion()).toEqual(inMemory.champion());
  });

  test("write-through: a reopened ladder holds everything, and is never reseeded", () => {
    const storage = fakeStorage();
    const first = seedBootstrapTower(openLocalLadder(storage), stressRegistry, stressAbilities);
    playLadderRun(1, "titan", first);
    const reopened = seedBootstrapTower(openLocalLadder(storage), stressRegistry, stressAbilities); // a page reload
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
    const state = buy(initRun({ seed: 7, runId: "web-1", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    const battle = { teamA: [TITAN], teamB: [TITAN], seed: 42, opponentLabel: "ghost bootstrap (round 1)" };
    saveRun(storage, state, battle);
    expect(loadRun(storage)).toEqual({ state, battle });
    saveRun(storage, state); // continue pressed: the battle record clears
    expect(loadRun(storage)).toEqual({ state });
    clearRun(storage);
    expect(loadRun(storage)).toBeNull();
  });

  test("a stored battle's replay position round-trips (#015 slice 4: reload mid-battle resumes parked)", () => {
    const storage = fakeStorage();
    const state = buy(initRun({ seed: 7, runId: "web-1", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    const battle = { teamA: [TITAN], teamB: [TITAN], seed: 42, opponentLabel: "ghost bootstrap (round 1)", position: 17 };
    saveRun(storage, state, battle);
    expect(loadRun(storage)?.battle?.position).toBe(17);
  });

  test("the local-only flag (#066 slice 4) round-trips and clears", () => {
    const storage = fakeStorage();
    const state = buy(initRun({ seed: 7, runId: "web-1", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    saveRun(storage, state); // no cheat: the flag is absent, shape stays { state }
    expect(loadRun(storage)).toEqual({ state });
    saveRun(storage, state, undefined, true); // a dev cheat marks it local-only
    expect(loadRun(storage)).toEqual({ state, localOnly: true });
    saveRun(storage, state, undefined, false); // a fresh run clears it
    expect(loadRun(storage)).toEqual({ state });
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
    const ladder = seedBootstrapTower(openLocalLadder(storage), stressRegistry, stressAbilities);
    const championBefore = ladder.champion(); // the bootstrap champion holds the spot

    // web-1 fights round 1 once, then is abandoned mid-climb. ladderFight
    // snapshots the fielded team into round 1's pool BEFORE resolving, so the
    // ghost is on the ladder the instant it fights — win, lose, or abandon.
    const s1 = buy(initRun({ seed: 1, runId: "web-1", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
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
    const reopened = seedBootstrapTower(openLocalLadder(storage), stressRegistry, stressAbilities);
    expect(reopened.champion()).toEqual(championBefore);
    expect(reopened.champion()?.runId).not.toBe("web-1");
    expect(reopened.poolAt(1).some((g) => g.runId === "web-1")).toBe(true); // the ghost persists

    // The next run (web-2) sees web-1's ghost as an eligible rival; its own
    // ghosts are excluded. After web-2 fights, web-2 excludes its OWN ghost but
    // still sees web-1's — exclusion is by runId, unaffected by the abandon.
    const s2 = buy(initRun({ seed: 2, runId: "web-2", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
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
    const ladder = seedBootstrapTower(openLocalLadder(storage), stressRegistry, stressAbilities);
    const state = buy(initRun({ seed: 7, runId: "web-1", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    ladderFight(state, ladder); // writes a ghost to the ladder key
    const battle = { teamA: [TITAN], teamB: [TITAN], seed: 42, opponentLabel: "ghost web-1 (round 1)" };
    saveRun(storage, state, battle); // mid-battle: a pending battle is stored too
    expect(loadRun(storage)).toEqual({ state, battle });

    clearRun(storage); // the abandon
    expect(loadRun(storage)).toBeNull(); // both run and pending battle gone

    // The ladder is a separate key — abandoning a run never wipes it.
    const reopened = seedBootstrapTower(openLocalLadder(storage), stressRegistry, stressAbilities);
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
    const state = buy(initRun({ seed: 1, runId: "web-1", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    expect(() => saveRun(storage, state)).toThrow();
  });
});

// ---------------------------------------------------------------------------
// #016 slice 3: namespaced storage, session token, submit verdicts
// ---------------------------------------------------------------------------

describe("prefixedStorage", () => {
  test("namespaces every key — a remote run never touches the local run's keys", () => {
    const storage = fakeStorage();
    const remote = prefixedStorage(storage, "remote:");
    const localState = buy(initRun({ seed: 1, runId: "web-1", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    const remoteState = buy(initRun({ seed: 2, runId: "web-uuid", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    saveRun(storage, localState);
    saveRun(remote, remoteState);
    expect(loadRun(storage)!.state.runId).toBe("web-1"); // logging in clobbered nothing
    expect(loadRun(remote)!.state.runId).toBe("web-uuid");
    clearRun(remote);
    expect(loadRun(storage)!.state.runId).toBe("web-1"); // and clearing one side spares the other
    expect(loadRun(remote)).toBeNull();
  });
});

describe("session token", () => {
  test("round-trips, clears, and treats empty as absent", () => {
    const storage = fakeStorage();
    expect(loadSession(storage)).toBeNull();
    saveSession(storage, "tok-123");
    expect(loadSession(storage)).toBe("tok-123");
    clearSession(storage);
    expect(loadSession(storage)).toBeNull();
    storage.setItem("aoi.session.v1", "");
    expect(loadSession(storage)).toBeNull(); // an empty token is no token
  });
});

describe("dev mode (#066 slice 1)", () => {
  test("off by default; round-trips on/off through the injected storage", () => {
    const storage = fakeStorage();
    expect(loadDevMode(storage)).toBe(false); // a fresh profile is never dev
    setDevMode(storage, true);
    expect(loadDevMode(storage)).toBe(true); // survives a "reload": same storage
    setDevMode(storage, false);
    expect(loadDevMode(storage)).toBe(false);
  });

  test("only the exact on marker reads as on — a stray value is off", () => {
    const storage = fakeStorage();
    storage.setItem("aoi.dev.v1", "true"); // not the "1" marker
    expect(loadDevMode(storage)).toBe(false);
    storage.setItem("aoi.dev.v1", "0");
    expect(loadDevMode(storage)).toBe(false);
    storage.setItem("aoi.dev.v1", "1");
    expect(loadDevMode(storage)).toBe(true);
  });

  test("turning it off clears the key (no lingering off-marker)", () => {
    const storage = fakeStorage();
    setDevMode(storage, true);
    setDevMode(storage, false);
    expect(storage.getItem("aoi.dev.v1")).toBeNull();
  });
});

describe("submit verdicts", () => {
  test("a verdict is per-runId: another run's (or corrupt) verdict reads as none", () => {
    const storage = fakeStorage();
    expect(loadSubmitResult(storage, "web-a")).toBeNull();
    saveSubmitResult(storage, { runId: "web-a", accepted: true, crowned: false });
    expect(loadSubmitResult(storage, "web-a")).toEqual({ runId: "web-a", accepted: true, crowned: false });
    expect(loadSubmitResult(storage, "web-b")).toBeNull(); // never another run's verdict
    storage.setItem("aoi.submit.v1", "{corrupt");
    expect(loadSubmitResult(storage, "web-a")).toBeNull(); // corrupt must not block a retry
  });

  test("clearRun drops the stored verdict with the run", () => {
    const storage = fakeStorage();
    saveSubmitResult(storage, { runId: "web-a", accepted: false, reason: "diverged" });
    clearRun(storage);
    expect(loadSubmitResult(storage, "web-a")).toBeNull();
  });
});
