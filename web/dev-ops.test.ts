// Dev cheats (#066 slice 4) — the pure mutation module, unit-tested in
// isolation (the cheat logic lives out of the DOM). Each test starts from a
// real initRun() state, so a mutation is asserted against a VALID RunState the
// rest of the system would accept.

import { describe, expect, test } from "vitest";
import {
  DEFAULT_RUN_POOL,
  TEAM_SIZE,
  initRun,
  stressAbilities,
  stressRegistry,
  type RunState,
  type UnitDef,
} from "../src/index.js";
import { addGold, setGold, spawnUnit } from "./dev-ops.js";

const fresh = (): RunState => initRun({ seed: 1, pool: DEFAULT_RUN_POOL, statuses: stressRegistry, abilities: stressAbilities });
const UNIT: UnitDef = DEFAULT_RUN_POOL[0]!;

describe("addGold / setGold", () => {
  test("addGold adds to the current gold", () => {
    const s = fresh();
    expect(addGold(s, 50).gold).toBe(s.gold + 50);
  });

  test("addGold with a negative subtracts, floored at 0", () => {
    const s = fresh();
    expect(addGold(s, -3).gold).toBe(Math.max(0, s.gold - 3));
    expect(addGold(s, -10_000).gold).toBe(0); // never goes negative
  });

  test("addGold by 0 leaves gold unchanged", () => {
    const s = fresh();
    expect(addGold(s, 0).gold).toBe(s.gold);
  });

  test("setGold sets the exact amount", () => {
    expect(setGold(fresh(), 99).gold).toBe(99);
  });

  test("setGold floors at 0 and rounds down", () => {
    expect(setGold(fresh(), -5).gold).toBe(0);
    expect(setGold(fresh(), 7.9).gold).toBe(7);
  });

  test("the input state is never mutated (pure transition)", () => {
    const s = fresh();
    const before = s.gold;
    addGold(s, 100);
    setGold(s, 0);
    expect(s.gold).toBe(before);
  });
});

describe("spawnUnit into the shop", () => {
  test("appends the unit to the offers, producing a valid shape", () => {
    const s = fresh();
    const out = spawnUnit(s, UNIT, "shop");
    expect(out.offers.length).toBe(s.offers.length + 1);
    expect(out.offers[out.offers.length - 1]!.name).toBe(UNIT.name);
    expect(s.offers.length).toBe(s.offers.length); // input untouched
  });
});

describe("spawnUnit into the team", () => {
  test("appends a buy-shaped RunUnit onto the line", () => {
    const s = fresh();
    const out = spawnUnit(s, UNIT, "team");
    expect(out.team.length).toBe(s.team.length + 1);
    const u = out.team[out.team.length - 1]!;
    expect(u.name).toBe(UNIT.name);
    expect(u.level).toBe(UNIT.level ?? 1);
    expect(u.stacks).toBe(1);
    expect(u.base).toEqual(UNIT.base);
  });

  test("the unit's base is copied, not aliased to the source def", () => {
    const s = fresh();
    const out = spawnUnit(s, UNIT, "team");
    out.team[out.team.length - 1]!.base.hp += 500;
    expect(UNIT.base.hp).not.toBe(out.team[out.team.length - 1]!.base.hp);
  });

  test("a full line is a no-op — never an over-full, invalid line", () => {
    let s = fresh();
    for (let i = 0; i < TEAM_SIZE; i++) s = spawnUnit(s, UNIT, "team");
    expect(s.team.length).toBe(TEAM_SIZE);
    const out = spawnUnit(s, UNIT, "team");
    expect(out.team.length).toBe(TEAM_SIZE); // refused, line unchanged
  });

  test("the input state is never mutated", () => {
    const s = fresh();
    const len = s.team.length;
    spawnUnit(s, UNIT, "team");
    spawnUnit(s, UNIT, "shop");
    expect(s.team.length).toBe(len);
  });
});
