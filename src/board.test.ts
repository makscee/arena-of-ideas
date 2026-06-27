// Board projection tests — boardAt is checked against the raw log itself:
// at every hp event the projected hp must equal the kernel-stamped hpAfter,
// at every status event the projected stacks must equal the stamped total.
// The projection never out-knows the log; these tests catch any drift.

import { describe, expect, test } from "vitest";
import { battle, winnerOf, TEAM_SIZE } from "./battle.js";
import { boardAt } from "./board.js";
import type { BattleInput, UnitDef } from "./types.js";
import { Necromancer, Silencer, stressRegistry, Summoner, Venomancer } from "./content/stress.js";
import { stressAbilities } from "./content/stress.js";

const dummy = (name: string, hp = 10, pwr = 2): UnitDef => ({ name, base: { hp, pwr } });

// The replay suite's stress battle: statuses, summons, resurrection, silence.
const stressBattle: BattleInput = {
  teamA: [
    Venomancer,
    Summoner,
    Necromancer,
    {
      name: "Champion",
      base: { hp: 12, pwr: 2 },
      statuses: [
        { status: "Strength", stacks: 2 },
        { status: "Shield", stacks: 3 },
      ],
    },
  ],
  teamB: [
    Silencer,
    {
      name: "FrozenBrute",
      base: { hp: 14, pwr: 4 },
      statuses: [
        { status: "Freeze", stacks: 2 },
        { status: "Curse", stacks: 1 },
      ],
    },
    {
      name: "Martyr",
      base: { hp: 6, pwr: 2 },
      statuses: [
        { status: "Blessing", stacks: 3 },
        { status: "Vitality", stacks: 2 },
      ],
    },
  ],
  seed: 42,
  statuses: stressRegistry,
  abilities: stressAbilities,
};

const findUnit = (board: ReturnType<typeof boardAt>, id: string) =>
  [...board.lines.A, ...board.lines.B, ...board.graves.A, ...board.graves.B].find((u) => u.id === id);

describe("boardAt vs the raw log, event by event", () => {
  const log = battle(stressBattle);

  test("projected hp equals the kernel-stamped hpAfter at every Hurt/Heal/hp StatChanged", () => {
    let checked = 0;
    for (const e of log) {
      if ((e.type !== "Hurt" && e.type !== "Heal" && e.type !== "StatChanged") || e.hpAfter === undefined) continue;
      const u = findUnit(boardAt(log, e.id), e.unit);
      expect(u, `unit ${e.unit} at event ${e.id}`).toBeDefined();
      expect(u!.hp, `hp of ${e.unit} after event ${e.id}`).toBe(Math.max(0, e.hpAfter));
      checked++;
    }
    expect(checked).toBeGreaterThan(10);
  });

  test("projected stacks equal the stamped total/remaining at every status event", () => {
    for (const e of log) {
      if (e.type === "StatusApplied") {
        const u = findUnit(boardAt(log, e.id), e.unit)!;
        expect(u.statuses.find((s) => s.status === e.status)?.stacks).toBe(e.total);
      } else if (e.type === "StatusRemoved") {
        const u = findUnit(boardAt(log, e.id), e.unit)!;
        const stacks = u.statuses.find((s) => s.status === e.status)?.stacks;
        if (e.remaining > 0) expect(stacks).toBe(e.remaining);
        else expect(stacks).toBeUndefined();
      }
    }
  });

  test("max hp and pwr follow StatChanged.now", () => {
    let checked = 0;
    for (const e of log) {
      if (e.type !== "StatChanged") continue;
      const u = findUnit(boardAt(log, e.id), e.unit)!;
      expect(e.stat === "hp" ? u.maxHp : u.pwr).toBe(e.now);
      checked++;
    }
    expect(checked).toBeGreaterThan(0);
  });

  test("a death moves the unit to its grave with a clean corpse; the line compacts", () => {
    const death = log.find((e) => e.type === "Death")!;
    const before = boardAt(log, death.id - 1);
    const after = boardAt(log, death.id);
    const side = before.lines.A.some((u) => u.id === death.unit) ? "A" : "B";
    expect(after.lines[side].some((u) => u.id === death.unit)).toBe(false);
    expect(after.lines[side].length).toBe(before.lines[side].length - 1);
    const corpse = after.graves[side].find((u) => u.id === death.unit)!;
    expect(corpse.hp).toBe(0);
    expect(corpse.statuses).toEqual([]);
    // whoever stood behind a dead front unit is the new front
    if (before.lines[side][0]!.id === death.unit) {
      expect(after.lines[side][0]!.id).toBe(before.lines[side][1]!.id);
    }
  });

  test("a summon enters at the back; a resurrected unit leaves the grave at atHp", () => {
    for (const e of log) {
      if (e.type !== "Summon") continue;
      const board = boardAt(log, e.id);
      const line = board.lines[e.side];
      expect(line[line.length - 1]!.id).toBe(e.unit);
      if (e.resurrected) {
        const u = line[line.length - 1]!;
        expect(board.graves[e.side].some((g) => g.id === e.unit)).toBe(false);
        expect(u.hp).toBe(Math.min(e.atHp ?? 1, u.maxHp));
        expect(u.statuses).toEqual([]); // the corpse was clean
      }
    }
    expect(log.some((e) => e.type === "Summon" && e.resurrected)).toBe(true);
    expect(log.some((e) => e.type === "Summon" && !e.resurrected)).toBe(true);
  });

  test("Silenced marks the unit", () => {
    const silenced = log.find((e) => e.type === "Silenced")!;
    expect(findUnit(boardAt(log, silenced.id), silenced.unit)!.silenced).toBe(true);
  });

  test("every step is well-formed: lines capped at TEAM_SIZE, ended only at BattleEnd", () => {
    for (const e of log) {
      const board = boardAt(log, e.id);
      expect(board.lines.A.length).toBeLessThanOrEqual(TEAM_SIZE);
      expect(board.lines.B.length).toBeLessThanOrEqual(TEAM_SIZE);
      expect(board.turn).toBe(e.turn);
      expect(board.ended !== undefined).toBe(e.type === "BattleEnd");
    }
  });

  test("the final board agrees with winnerOf: the losing line is empty", () => {
    const final = boardAt(log, log.length - 1);
    const winner = winnerOf(log);
    expect(final.ended?.winner).toBe(winner);
    if (winner === "A") {
      expect(final.lines.B).toEqual([]);
      expect(final.lines.A.length).toBeGreaterThan(0);
    } else if (winner === "B") {
      expect(final.lines.A).toEqual([]);
      expect(final.lines.B.length).toBeGreaterThan(0);
    }
  });
});

describe("hp StatChanged stamp", () => {
  test("a mid-battle hp statMod stamps hpAfter (current hp), distinct from now (the new max)", () => {
    // Gains Vitality when hurt — the StatChanged lands while damage is outstanding,
    // so hpAfter (current) and now (effective max) must disagree.
    const grower: UnitDef = {
      name: "Grower",
      base: { hp: 10, pwr: 1 },
      abilities: [
        {
          whens: [{ kind: "trigger", on: { on: "Hurt", unit: "holder" } }],
          selectors: [{ kind: "holder" }],
          effects: [{ kind: "applyStatus", status: "Vitality", stacks: { kind: "const", value: 2 } }],
        },
      ],
    };
    const log = battle({ teamA: [grower], teamB: [dummy("Hitter", 12, 3)], seed: 1, statuses: stressRegistry, abilities: stressAbilities });
    const sc = log.find((e) => e.type === "StatChanged" && e.stat === "hp" && e.hpAfter !== undefined && e.hpAfter !== e.now);
    expect(sc, "an hp StatChanged with outstanding damage").toBeDefined();
    if (sc?.type !== "StatChanged") throw new Error("unreachable");
    expect(findUnit(boardAt(log, sc.id), sc.unit)!.hp).toBe(Math.max(0, sc.hpAfter!));
  });
});

describe("display clamping", () => {
  test("overkill hp shows 0, never negative", () => {
    const log = battle({ teamA: [dummy("Bruiser", 10, 9)], teamB: [dummy("Frail", 2, 1)], seed: 0 });
    const hurt = log.find((e) => e.type === "Hurt" && e.unit === "B1:Frail")!;
    expect(hurt.type === "Hurt" && hurt.hpAfter!).toBeLessThan(0); // raw log is negative
    expect(findUnit(boardAt(log, hurt.id), "B1:Frail")!.hp).toBe(0); // display clamps
  });
});
