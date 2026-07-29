import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, test } from "vitest";
import { FileLadderStore } from "../../src/ladder-file.js";
import { InMemoryLadderStore, type LadderStore, type TeamSnapshot } from "../../src/ladder.js";
import type { UnitDef } from "../../src/types.js";
import type { ArenaApi } from "../../web/api.js";
import { RemoteLadder } from "../../web/remote-ladder.js";
import { openDb } from "./db.js";
import { SqliteLadderStore } from "./ladder-store.js";

const unit = (name: string): UnitDef => ({ name, base: { hp: 3, pwr: 1 }, ability: "Strike" });
const seat = (runId: string, floor: number): TeamSnapshot => ({ runId, round: floor, seq: 0, team: [unit(runId)] });

type StoreCase = {
  name: string;
  open(): LadderStore;
  reopen?: () => LadderStore;
};

function contract(makeCase: () => StoreCase): void {
  const entry = makeCase();

  test(`${entry.name}: floors are vacant by default`, () => {
    const store = entry.open();
    expect(store.bossAt(1)).toBeNull();
    expect(store.bossAt(99)).toBeNull();
    expect(store.champion()).toBeNull();
  });

  test(`${entry.name}: a seat snapshot must name the floor it occupies`, () => {
    const store = entry.open();
    expect(() => store.setBoss(2, seat("wrong-floor", 3))).toThrow(/seat floor 2.*snapshot round 3/);
    expect(store.bossAt(2)).toBeNull();
    expect(store.champion()).toBeNull();
  });

  test(`${entry.name}: seats are per-floor, overwrite in place, and champion derives from the highest occupied floor`, () => {
    const store = entry.open();
    store.setBoss(2, seat("low", 2));
    store.setBoss(5, seat("top", 5));
    store.setBoss(2, seat("new-low", 2));
    store.setBoss(5, seat("new-top", 5));

    expect(store.bossAt(2)).toEqual(seat("new-low", 2));
    expect(store.bossAt(3)).toBeNull();
    expect(store.bossAt(5)).toEqual(seat("new-top", 5));
    expect(store.champion()).toEqual(seat("new-top", 5));
  });

  if (entry.reopen !== undefined) {
    test(`${entry.name}: every floor seat survives reopen`, () => {
      const store = entry.open();
      store.setBoss(2, seat("low", 2));
      store.setBoss(5, seat("top", 5));
      store.setBoss(2, seat("new-low", 2));
      store.setBoss(5, seat("new-top", 5));

      const reopened = entry.reopen!();
      expect(reopened.bossAt(2)).toEqual(seat("new-low", 2));
      expect(reopened.bossAt(5)).toEqual(seat("new-top", 5));
      expect(reopened.champion()).toEqual(seat("new-top", 5));
    });
  }
}

describe("LadderStore behavioral contract", () => {
  describe("memory", () => contract(() => ({ name: "memory", open: () => new InMemoryLadderStore() })));

  describe("file", () => {
    const path = join(mkdtempSync(join(tmpdir(), "ladder-contract-file-")), "ladder.json");
    contract(() => ({ name: "file", open: () => new FileLadderStore(path), reopen: () => new FileLadderStore(path) }));
  });

  describe("remote", () => contract(() => ({
    name: "remote",
    open: () => new RemoteLadder({} as ArenaApi, "token"),
  })));

  describe("sqlite", () => {
    const path = join(mkdtempSync(join(tmpdir(), "ladder-contract-sqlite-")), "arena.db");
    contract(() => ({
      name: "sqlite",
      open: () => new SqliteLadderStore(openDb(path).db),
      reopen: () => new SqliteLadderStore(openDb(path).db),
    }));
  });
});
