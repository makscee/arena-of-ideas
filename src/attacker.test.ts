// "The attacker" (mission 3, M3-1, makscee/void-board#801): a Who for whoever
// dealt the triggering hit. A strike's attacker is the striker; an ability's
// damage, the unit holding the ability. Poison and other status damage have
// no attacker, and a dead attacker is nobody: the effect does nothing.

import { describe, expect, test } from "vitest";
import { battle, hurtAttacker } from "./battle.js";
import { stressAbilities, stressRegistry } from "./content/stress.js";
import type { AbilityDef, AbilityRegistry, BattleEvent, UnitDef, When } from "./types.js";

const ab = (name: string, family: AbilityDef["family"], effects: AbilityDef["effects"]): AbilityDef => ({ name, family, effects });
const n = (value: number) => ({ kind: "const" as const, value });

const abilities: AbilityRegistry = {
  ...stressAbilities,
  Envenom: ab("Envenom", "Poison", [{ kind: "applyStatus", status: "Poison", stacks: n(1) }]),
  Shoot: ab("Shoot", "Strike", [{ kind: "damage", amount: n(2) }]),
  Raise: ab("Raise", "Summon", [{ kind: "resurrect", hp: n(1) }]),
};

const unit = (name: string, hp: number, pwr: number, on: When["on"], who: UnitDef["selectors"], does: string[]): UnitDef => ({
  name, base: { hp, pwr }, triggers: [{ kind: "trigger", on }], selectors: who!, abilities: does,
});
const dummy = (name: string, hp: number, pwr: number): UnitDef => unit(name, hp, pwr, { on: "Death", unit: "holder" }, [{ kind: "holder" }], ["Strike"]);

/** The concept's Hedgehog: when hurt → the attacker → poison. */
const Hedgehog = unit("Hedgehog", 30, 1, { on: "Hurt", unit: "holder" }, [{ kind: "attacker" }], ["Envenom"]);

const run = (teamA: UnitDef[], teamB: UnitDef[]): BattleEvent[] => battle({ teamA, teamB, seed: 7, statuses: stressRegistry, abilities, turnCap: 3 });
const idOf = (log: BattleEvent[], name: string): string => {
  const start = log[0]!;
  if (start.type !== "BattleStart") throw new Error("no BattleStart");
  return [...start.teams.A, ...start.teams.B].find((r) => r.name === name)!.id;
};
/** The Poisons the Hedgehog's own ability applied, with the Hurt each answered. */
const answers = (log: BattleEvent[], hog: string) =>
  log.flatMap((e) => (e.type === "StatusApplied" && e.source !== "kernel" && e.source.unit === hog && e.status === "Poison" ? [{ on: e.unit, hurt: log[e.causedBy!]! }] : []));

describe("the attacker", () => {
  test("after a strike: the poison lands on the striker", () => {
    const log = run([Hedgehog], [dummy("Brute", 30, 2)]);
    const hog = idOf(log, "Hedgehog"), brute = idOf(log, "Brute");
    const a = answers(log, hog);
    expect(a.length).toBeGreaterThan(0);
    const first = a[0]!;
    expect(first.on).toBe(brute);
    expect(first.hurt.type).toBe("Hurt");
    expect(log[first.hurt.causedBy!]).toMatchObject({ type: "Strike", striker: brute });
    expect(hurtAttacker(first.hurt, log)).toBe(brute);
  });

  test("after an ability's damage: the ability's holder, wherever it stands", () => {
    const Archer = unit("Archer", 30, 0, { on: "BattleStart" }, [{ kind: "frontEnemy" }], ["Shoot"]);
    const log = run([Hedgehog], [dummy("Wall", 30, 0), Archer]);
    const hog = idOf(log, "Hedgehog"), archer = idOf(log, "Archer");
    const shot = log.find((e) => e.type === "Hurt" && e.unit === hog && e.source !== "kernel")!;
    expect(hurtAttacker(shot, log)).toBe(archer);
    expect(answers(log, hog).find((x) => x.hurt.id === shot.id)?.on).toBe(archer);
  });

  test("after poison: no attacker, nothing happens", () => {
    const Poisoner = unit("Poisoner", 30, 0, { on: "BattleStart" }, [{ kind: "frontEnemy" }], ["Envenom"]);
    const log = run([Hedgehog], [dummy("Wall", 30, 0), Poisoner]);
    const hog = idOf(log, "Hedgehog");
    const ticks = log.filter((e) => e.type === "Hurt" && e.unit === hog && e.source !== "kernel" && e.source.status === "Poison");
    expect(ticks.length).toBeGreaterThan(0);
    for (const t of ticks) expect(hurtAttacker(t, log)).toBeUndefined();
    const tickIds = new Set(ticks.map((t) => t.id));
    expect(answers(log, hog).filter((x) => tickIds.has(x.hurt.id))).toEqual([]);
    // The Poisoner, who applied it, is never answered: only the Wall that strikes.
    expect(answers(log, hog).map((x) => x.on)).not.toContain(idOf(log, "Poisoner"));
  });

  test("fatigue has no attacker", () => {
    const log = battle({ teamA: [dummy("A", 9, 0)], teamB: [dummy("B", 9, 0)], seed: 1, statuses: stressRegistry, abilities });
    const fatigue = log.filter((e) => e.type === "Hurt" && e.source === "kernel" && log[e.causedBy!]?.type === "Fatigue");
    expect(fatigue.length).toBeGreaterThan(0);
    for (const h of fatigue) expect(hurtAttacker(h, log)).toBeUndefined();
  });

  test("attacker dead: nothing (a revive on the attacker raises nobody)", () => {
    // The Bomber's last act, as it dies, is a shot at the Raiser.
    const Bomber = unit("Bomber", 1, 0, { on: "Death", unit: "holder" }, [{ kind: "frontEnemy" }], ["Shoot"]);
    const Raiser = unit("Raiser", 30, 3, { on: "Hurt", unit: "holder" }, [{ kind: "attacker" }], ["Raise"]);
    const log = run([Raiser], [Bomber, dummy("Back", 30, 0)]);
    const raiser = idOf(log, "Raiser"), bomber = idOf(log, "Bomber");
    const shot = log.find((e) => e.type === "Hurt" && e.unit === raiser && e.source !== "kernel" && e.source.unit === bomber)!;
    expect(shot).toBeDefined();
    expect(hurtAttacker(shot, log)).toBe(bomber);
    expect(log.some((e) => e.type === "Summon" && e.resurrected && e.unit === bomber)).toBe(false);
  });

  test("events that aren't a hurt have no attacker", () => {
    const log = run([Hedgehog], [dummy("Brute", 30, 2)]);
    expect(log.filter((e) => e.type !== "Hurt").every((e) => hurtAttacker(e, log) === undefined)).toBe(true);
  });
});
