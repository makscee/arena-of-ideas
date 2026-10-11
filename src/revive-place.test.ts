// R2 (#918): a revived unit returns to the place it fell from (its index in
// the line at death, clamped to the line's length then), not to the back.
// A rule (BattleInput.reviveInPlace, MvpRules.reviveInPlace): fights before
// it, which the desktop ladder and stored runs re-sim, keep the back.
// The Summon event says `slot`, and boardAt honours it, so logs written
// before it still replay their revives at the back.
import { describe, expect, test } from "vitest";
import { battle } from "./battle.js";
import { boardAt } from "./board.js";
import type { AbilityDef, BattleEvent, BattleInput, UnitDef } from "./types.js";
import { Necromancer, stressAbilities, stressRegistry } from "./content/stress.js";

const dummy = (name: string, hp = 20, pwr = 1): UnitDef => ({ name, base: { hp, pwr }, ability: "Strike" });
const Blast1: AbilityDef = { name: "Blast1", family: "Strike", effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }] };
const Blast2: AbilityDef = { name: "Blast2", family: "Strike", effects: [{ kind: "damage", amount: { kind: "const", value: 2 } }] };
const Rise: AbilityDef = { name: "Rise", family: "Summon", effects: [{ kind: "resurrect", hp: { kind: "const", value: 1 } }] };
/** Hits every enemy for `n` at battle start. */
const bomber = (name: string, n: 1 | 2): UnitDef => ({
  name,
  base: { hp: 40, pwr: 1 },
  triggers: [{ kind: "trigger", on: { on: "BattleStart" } }],
  selectors: [{ kind: "allEnemies" }],
  abilities: [n === 1 ? "Blast1" : "Blast2"],
});
/** Returns itself when it dies. */
const phoenix = (name: string, hp: number): UnitDef => ({
  name,
  base: { hp, pwr: 1 },
  triggers: [{ kind: "trigger", on: { on: "Death", unit: "holder" } }],
  selectors: [{ kind: "holder" }],
  abilities: ["Rise"],
});
const run = (teamA: UnitDef[], teamB: UnitDef[], seed = 1, reviveInPlace = true) =>
  battle({ teamA, teamB, seed, statuses: stressRegistry, abilities: { ...stressAbilities, Blast1, Blast2, Rise }, reviveInPlace } as BattleInput);
const revives = (log: BattleEvent[]) =>
  log.filter((e): e is Extract<BattleEvent, { type: "Summon" }> => e.type === "Summon" && !!e.resurrected);
const lineAt = (log: BattleEvent[], id: number) => boardAt(log, id).lines.A.map((u) => u.name);

describe("a revive returns to its place in the line", () => {
  test("[A, B, C]: B dies and is revived → [A, B, C], not [A, C, B]", () => {
    const log = run([dummy("A"), dummy("B", 1), dummy("C"), Necromancer], [bomber("Bomber", 1)]);
    const rev = revives(log)[0]!;
    expect(rev).toMatchObject({ name: "B", slot: 1 });
    expect(lineAt(log, rev.id)).toEqual(["A", "B", "C", "Necromancer"]);
  });

  test("the front unit dies and is revived → front again", () => {
    const log = run([dummy("A", 1), dummy("B"), Necromancer], [bomber("Bomber", 1)]);
    const rev = revives(log)[0]!;
    expect(rev).toMatchObject({ name: "A", slot: 0 });
    expect(lineAt(log, rev.id)).toEqual(["A", "B", "Necromancer"]);
  });

  test("the line shrank meanwhile → clamped to its length", () => {
    // Phoenix falls at index 3 to the first blast; its own revive waits behind
    // the second blast, which kills F1 and F2, so it returns at index 1.
    const log = run([dummy("Tank", 30), dummy("F1", 3), dummy("F2", 3), phoenix("Phoenix", 1)], [bomber("Bomber1", 1), bomber("Bomber2", 2)]);
    const rev = revives(log)[0]!;
    expect(rev).toMatchObject({ name: "Phoenix", slot: 1 });
    expect(lineAt(log, rev.id)).toEqual(["Tank", "Phoenix"]);
  });

  test("the engine's line agrees: the revived unit fights from its place", () => {
    // A falls from the front and comes back there, so A, not B, faces turn 1.
    const log = run([dummy("A", 1), dummy("B"), Necromancer], [bomber("Bomber", 1)]);
    const pair = log.find((e) => e.type === "PairFaced" && e.turn === 1)!;
    expect(pair.type === "PairFaced" && pair.a).toMatch(/:A$/);
  });

  test("without the rule (fights before it) a revive still goes to the back, with no `slot`", () => {
    const log = run([dummy("A"), dummy("B", 1), dummy("C"), Necromancer], [bomber("Bomber", 1)], 1, false);
    const rev = revives(log)[0]!;
    expect(rev.slot).toBeUndefined();
    expect(lineAt(log, rev.id)).toEqual(["A", "C", "Necromancer", "B"]);
  });

  test("an old log without `slot` still replays its revive at the back", () => {
    const log = run([dummy("A"), dummy("B", 1), dummy("C"), Necromancer], [bomber("Bomber", 1)]);
    const old = log.map((e) => {
      if (e.type !== "Summon") return e;
      const { slot: _slot, ...rest } = e;
      return rest as BattleEvent;
    });
    const rev = revives(old)[0]!;
    expect(lineAt(old, rev.id)).toEqual(["A", "C", "Necromancer", "B"]);
  });
});
