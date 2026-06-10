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
});
