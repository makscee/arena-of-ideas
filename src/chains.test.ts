// Golden battles for line-order chains (mission #574, slice 3): triggered units
// fire front to back, each unit reacts to an event at most once, a cascade stops
// at the step cap with a visible ChainCapped event, and the chain-link events
// (shield gained, healed, power gained, status applied, summoned, ally died)
// carry 3-unit cascades whose causedBy walk reads like the battle viewer's trace.

import { describe, expect, test } from "vitest";
import { battle, DEFAULT_CHAIN_STEP_CAP, toJSONL } from "./battle.js";
import { renderReplay } from "./replay.js";
import { validateTeam } from "./validate.js";
import type { AbilityDef, AbilityRegistry, BattleEvent, BattleInput, UnitDef, When } from "./types.js";
import { stressAbilities, stressRegistry } from "./content/stress.js";

const ab = (name: string, family: AbilityDef["family"], effects: AbilityDef["effects"]): AbilityDef => ({ name, family, effects });
const n = (value: number) => ({ kind: "const" as const, value });

const abilities: AbilityRegistry = {
  ...stressAbilities,
  GiveShield: ab("GiveShield", "Shield", [{ kind: "applyStatus", status: "Shield", stacks: n(2) }]),
  GiveStrength: ab("GiveStrength", "Strike", [{ kind: "applyStatus", status: "Strength", stacks: n(1) }]),
  Shoot: ab("Shoot", "Strike", [{ kind: "damage", amount: n(2) }]),
  Mend: ab("Mend", "Heal", [{ kind: "heal", amount: n(1) }]),
  Envenom: ab("Envenom", "Poison", [{ kind: "applyStatus", status: "Poison", stacks: n(1) }]),
  Ping: ab("Ping", "Strike", [{ kind: "damage", amount: n(1) }]),
  Ping2: ab("Ping2", "Strike", [{ kind: "damage", amount: n(2) }]),
};

const trigger = (on: When["on"]): When => ({ kind: "trigger", on });
const unit = (name: string, hp: number, pwr: number, on: When["on"], who: UnitDef["selectors"], does: string[]): UnitDef => ({
  name,
  base: { hp, pwr },
  triggers: [trigger(on)],
  selectors: who!,
  abilities: does,
});

/** A body that only reacts to its own death, with the inert Strike action. */
const dummy = (name: string, hp: number, pwr: number): UnitDef =>
  unit(name, hp, pwr, { on: "Death", unit: "holder" }, [{ kind: "holder" }], ["Strike"]);

const Shieldbearer = unit("Shieldbearer", 6, 1, { on: "BattleStart" }, [{ kind: "holder" }], ["GiveShield"]);
const Smith = unit("Smith", 5, 1, { on: "StatusApplied", unit: "otherAlly", status: "Shield" }, [{ kind: "eventUnit" }], ["GiveStrength"]);
const Archer = unit("Archer", 4, 1, { on: "StatChanged", unit: "ally", stat: "pwr", sign: "gain" }, [{ kind: "frontEnemy" }], ["Shoot"]);
const Medic = unit("Medic", 5, 1, { on: "Hurt", unit: "otherAlly" }, [{ kind: "eventUnit" }], ["Mend"]);
const Zealot = unit("Zealot", 5, 1, { on: "Heal", unit: "otherAlly" }, [{ kind: "frontEnemy" }], ["Envenom"]);
const Caller = unit("Caller", 5, 1, { on: "Death", unit: "otherAlly" }, [{ kind: "holder" }], ["Conjure"]);
const Herald = unit("Herald", 5, 1, { on: "Summon", unit: "ally" }, [{ kind: "eventUnit" }], ["GiveShield"]);
/** Fused shape: one When and one Who, the Does of both parts. It listens to an
 * ally's Shield and gives a Shield itself, so it would re-trigger itself. */
const Fused = unit("Fused", 9, 1, { on: "StatusApplied", unit: "ally", status: "Shield" }, [{ kind: "holder" }], ["GiveShield", "GiveStrength"]);
/** Reacts to a Poison landing on anyone, not to any other status. */
const Plaguewatcher = unit("Plaguewatcher", 5, 1, { on: "StatusApplied", unit: "any", status: "Poison" }, [{ kind: "eventUnit" }], ["Ping"]);
const Poisoner = unit("Poisoner", 5, 1, { on: "BattleStart" }, [{ kind: "frontEnemy" }], ["Envenom"]);

const run = (teamA: UnitDef[], teamB: UnitDef[], extra: Partial<BattleInput> = {}): BattleEvent[] =>
  battle({ teamA, teamB, seed: 7, statuses: stressRegistry, abilities, ...extra });

/** One event as the trace reads it: what happened, to whom, and whose ability did it. */
function show(e: BattleEvent): string {
  const by = e.source === "kernel" ? "kernel" : e.source.unit;
  switch (e.type) {
    case "Hurt": return `Hurt ${e.unit} -${e.amount} by ${by}`;
    case "Heal": return `Heal ${e.unit} +${e.amount} by ${by}`;
    case "StatusApplied": return `${e.status} on ${e.unit} by ${by}`;
    case "StatChanged": return `${e.stat} ${e.delta > 0 ? "+" : ""}${e.delta} on ${e.unit} by ${by}`;
    case "Summon": return `Summon ${e.unit} by ${by}`;
    case "Death": return `Death ${e.unit}`;
    case "Strike": return `Strike ${e.striker}→${e.defender}`;
    case "ChainCapped": return `ChainCapped root=${e.root} steps=${e.steps}`;
    default: return e.type;
  }
}

/** The causedBy walk from one event up to its root — "tap any change to trace its chain". */
function chainOf(log: BattleEvent[], id: number): string[] {
  const out: string[] = [];
  for (let e: BattleEvent | undefined = log[id]; e; e = e.causedBy === null ? undefined : log[e.causedBy]) out.push(show(e));
  return out;
}

const firingOrder = (log: BattleEvent[], from: number): string[] =>
  log.filter((e) => e.causedBy === from && e.source !== "kernel").map((e) => (e.source as { unit: string }).unit);

describe("3-unit cascades through chain-link events", () => {
  test("shield gained → power gained → damage (Shieldbearer → Smith → Archer)", () => {
    const log = run([Shieldbearer, Smith, Archer], [dummy("Dummy", 20, 1)]);
    const shot = log.find((e) => e.type === "Hurt" && e.source !== "kernel")!;
    expect(chainOf(log, shot.id)).toEqual([
      "Hurt B1:Dummy -2 by A3:Archer",
      "pwr +1 on A1:Shieldbearer by kernel",
      "Strength on A1:Shieldbearer by A2:Smith",
      "Shield on A1:Shieldbearer by A1:Shieldbearer",
      "BattleStart",
    ]);
    expect(log.some((e) => e.type === "ChainCapped")).toBe(false);
  });

  test("hurt → healed → status applied (Squire → Medic → Zealot)", () => {
    const log = run([dummy("Squire", 8, 1), Medic, Zealot], [dummy("Dummy", 30, 2)]);
    const poison = log.find((e) => e.type === "StatusApplied" && e.status === "Poison")!;
    expect(chainOf(log, poison.id)).toEqual([
      "Poison on B1:Dummy by A3:Zealot",
      "Heal A1:Squire +1 by A2:Medic",
      "Hurt A1:Squire -2 by kernel",
      "Strike B1:Dummy→A1:Squire",
      "TurnStart",
    ]);
  });

  test("ally died → summoned → shield gained (Martyr → Caller → Herald)", () => {
    const log = run([dummy("Martyr", 1, 1), Caller, Herald], [dummy("Dummy", 30, 3)]);
    const shield = log.find((e) => e.type === "StatusApplied" && e.status === "Shield")!;
    expect(chainOf(log, shield.id)).toEqual([
      "Shield on A+1:Imp by A3:Herald",
      "Summon A+1:Imp by A2:Caller",
      "Death A1:Martyr",
      "Hurt A1:Martyr -3 by kernel",
      "Strike B1:Dummy→A1:Martyr",
      "TurnStart",
    ]);
  });

  test("the cascades are golden: the full logs are pinned byte for byte", async () => {
    const goldens: Array<[string, BattleEvent[]]> = [
      ["shield-power-damage", run([Shieldbearer, Smith, Archer], [dummy("Dummy", 20, 1)])],
      ["hurt-healed-status", run([dummy("Squire", 8, 1), Medic, Zealot], [dummy("Dummy", 30, 2)])],
      ["died-summoned-shield", run([dummy("Martyr", 1, 1), Caller, Herald], [dummy("Dummy", 30, 3)])],
      ["capped-at-2", run([Shieldbearer, Smith, Archer], [dummy("Dummy", 20, 1)], { chainStepCap: 2 })],
      ["fused-reacts-once", run([Shieldbearer, Fused], [dummy("Dummy", 20, 1)])],
      ["poison-listener", run([Shieldbearer, Poisoner, Plaguewatcher], [dummy("Dummy", 20, 1)])],
    ];
    const text = goldens.map(([name, log]) => `# ${name}\n${toJSONL(log)}`).join("");
    await expect(text).toMatchFileSnapshot("./__fixtures__/golden-chains.jsonl");
  });
});

describe("line order", () => {
  test("the units an event triggers fire front to back, by position in the line", () => {
    const cheer = (name: string) => unit(name, 5, 1, { on: "BattleStart" }, [{ kind: "holder" }], ["GiveStrength"]);
    const forward = run([cheer("X"), cheer("Y"), cheer("Z")], [dummy("Dummy", 9, 1)]);
    expect(firingOrder(forward, 0)).toEqual(["A1:X", "A2:Y", "A3:Z"]);
    const reversed = run([cheer("Z"), cheer("Y"), cheer("X")], [dummy("Dummy", 9, 1)]);
    expect(firingOrder(reversed, 0)).toEqual(["A1:Z", "A2:Y", "A3:X"]);
  });

  test("a cascade resolves breadth-first: every reaction to one event, front to back, before reactions to their results", () => {
    // Shieldbearer (A1) and a second Shieldbearer (A3) both react to BattleStart;
    // Smith (A2) reacts to their Shields only after both have fired.
    const log = run([Shieldbearer, Smith, { ...Shieldbearer, name: "Warden" }], [dummy("Dummy", 9, 1)]);
    const order = log.filter((e) => e.type === "StatusApplied").map(show);
    expect(order).toEqual([
      "Shield on A1:Shieldbearer by A1:Shieldbearer",
      "Shield on A3:Warden by A3:Warden",
      "Strength on A1:Shieldbearer by A2:Smith",
      "Strength on A3:Warden by A2:Smith",
    ]);
  });
});

describe("a unit reacts at most once to the same event", () => {
  test("two matching Triggers on one unit fire one reaction", () => {
    const Twitchy: UnitDef = {
      name: "Twitchy",
      base: { hp: 9, pwr: 1 },
      triggers: [trigger({ on: "Hurt", unit: "holder" }), trigger({ on: "Hurt", unit: "ally" })],
      selectors: [{ kind: "frontEnemy" }],
      abilities: ["Ping"],
    };
    const log = run([Twitchy], [dummy("Dummy", 30, 1)]);
    const firstHit = log.find((e) => e.type === "Hurt" && e.unit === "A1:Twitchy")!;
    expect(log.filter((e) => e.causedBy === firstHit.id && e.source !== "kernel")).toHaveLength(1);
  });

  test("a fused unit's two Abilities each fire once, in order (one reaction, Does of both)", () => {
    const Fused: UnitDef = {
      name: "Fused",
      base: { hp: 9, pwr: 1 },
      triggers: [trigger({ on: "BattleStart" }), trigger({ on: "BattleStart" })],
      selectors: [{ kind: "holder" }],
      abilities: ["GiveShield", "GiveStrength"],
    };
    const log = run([Fused], [dummy("Dummy", 9, 1)]);
    expect(log.filter((e) => e.type === "StatusApplied").map(show)).toEqual([
      "Shield on A1:Fused by A1:Fused",
      "Strength on A1:Fused by A1:Fused",
    ]);
  });

  test("a fused unit reacts once: one Who for both Does, and it never re-triggers itself", () => {
    const log = run([Shieldbearer, Fused], [dummy("Dummy", 20, 1)]);
    const sbShield = log.find((e) => e.type === "StatusApplied" && e.source !== "kernel" && e.source.unit === "A1:Shieldbearer")!;
    // One reaction to Shieldbearer's Shield: both Does, in order, sourced to the first and second Ability.
    expect(log.filter((e) => e.causedBy === sbShield.id && e.source !== "kernel").map((e) => [show(e), (e.source as { ability: number }).ability]))
      .toEqual([["Shield on A2:Fused by A2:Fused", 0], ["Strength on A2:Fused by A2:Fused", 1]]);
    // Its own Shield wakes it again: the whole unit is blocked, neither Does fires a second time.
    expect(log.filter((e) => e.type === "StatusApplied" && e.status === "Strength")).toHaveLength(1);
    expect(log.filter((e) => e.type === "ChainBlocked").map((e) => e.type === "ChainBlocked" && e.ability)).toEqual([{ unit: "A2:Fused", ability: 0, when: 0 }]);
  });

  test("a fused unit's reaction is one cap step: both Does land before the chain is capped", () => {
    // Step 1: Shieldbearer. Step 2: Fused, both Does. Its self-wake is the dropped firing.
    const log = run([Shieldbearer, Fused], [dummy("Dummy", 20, 1)], { chainStepCap: 2 });
    const capped = log.findIndex((e) => e.type === "ChainCapped");
    expect(capped).toBeGreaterThan(0);
    expect(log.slice(0, capped).filter((e) => e.type === "StatusApplied").map(show)).toEqual([
      "Shield on A1:Shieldbearer by A1:Shieldbearer",
      "Shield on A2:Fused by A2:Fused",
      "Strength on A2:Fused by A2:Fused",
    ]);
  });

  test("a fused unit picks its Who once: a random target is shared by both Does", () => {
    const Volley = unit("Volley", 9, 1, { on: "BattleStart" }, [{ kind: "randomEnemy" }], ["Ping", "Ping2"]);
    const foes = [1, 2, 3, 4, 5].map((i) => dummy(`E${i}`, 20, 1));
    for (const seed of [1, 2, 3, 7, 11, 42]) {
      const log = battle({ teamA: [Volley], teamB: foes, seed, statuses: stressRegistry, abilities });
      const hits = log.filter((e) => e.type === "Hurt" && e.causedBy === 0);
      expect(hits.map((e) => e.type === "Hurt" && e.amount)).toEqual([1, 2]);
      expect(new Set(hits.map((e) => e.type === "Hurt" && e.unit)).size).toBe(1);
    }
  });

  test("status applied: a listener on Poison fires on Poison and ignores Shield", () => {
    const log = run([Shieldbearer, Poisoner, Plaguewatcher], [dummy("Dummy", 20, 1)]);
    const ping = log.find((e) => e.type === "Hurt" && e.source !== "kernel" && e.source.unit === "A3:Plaguewatcher")!;
    expect(chainOf(log, ping.id)).toEqual([
      "Hurt B1:Dummy -1 by A3:Plaguewatcher",
      "Poison on B1:Dummy by A2:Poisoner",
      "BattleStart",
    ]);
    expect(log.filter((e) => e.source !== "kernel" && e.source.unit === "A3:Plaguewatcher")).toHaveLength(1);
  });

  test("otherAlly: 'after an ally dies' doesn't fire on the holder's own death; 'ally' does", () => {
    const lone = run([{ ...Caller, base: { hp: 1, pwr: 1 } }], [dummy("Dummy", 30, 3)]);
    expect(lone.some((e) => e.type === "Summon")).toBe(false);
    const selfish = unit("Caller", 1, 1, { on: "Death", unit: "ally" }, [{ kind: "holder" }], ["Conjure"]);
    expect(run([selfish], [dummy("Dummy", 30, 3)]).some((e) => e.type === "Summon")).toBe(true);
  });

  test("StatChanged sign: a pwr loss doesn't match sign gain", () => {
    const Curser = unit("Curser", 6, 1, { on: "BattleStart" }, [{ kind: "frontEnemy" }], ["Hex"]);
    const log = battle({
      teamA: [Curser],
      teamB: [{ ...Archer, name: "Archer" }, dummy("Dummy", 9, 3)],
      seed: 7,
      statuses: stressRegistry,
      abilities: { ...abilities, Hex: ab("Hex", "Control", [{ kind: "applyStatus", status: "Curse", stacks: n(1) }]) },
    });
    expect(log.some((e) => e.type === "StatChanged" && e.stat === "pwr" && e.delta < 0)).toBe(true);
    expect(log.some((e) => e.source !== "kernel" && e.source.unit === "B1:Archer")).toBe(false);
  });
});

describe("cascade step cap", () => {
  test("a capped cascade stops and logs ChainCapped, caused by the event it stopped at", () => {
    const log = run([Shieldbearer, Smith, Archer], [dummy("Dummy", 20, 1)], { chainStepCap: 2 });
    const capped = log.find((e) => e.type === "ChainCapped")!;
    expect(capped).toMatchObject({ type: "ChainCapped", root: 0, steps: 2, source: "kernel" });
    expect(chainOf(log, capped.id)).toEqual([
      "ChainCapped root=0 steps=2",
      "pwr +1 on A1:Shieldbearer by kernel",
      "Strength on A1:Shieldbearer by A2:Smith",
      "Shield on A1:Shieldbearer by A1:Shieldbearer",
      "BattleStart",
    ]);
    // Archer's firing was dropped: no damage from it, and the battle still ends.
    expect(log.some((e) => e.source !== "kernel" && e.source.unit === "A3:Archer")).toBe(false);
    expect(log[log.length - 1]!.type).toBe("BattleEnd");
    expect(renderReplay(log)).toContain("chain capped");
  });

  test("an exploding cascade stops at the default cap and the battle still ends", () => {
    // Every Echo pings all enemies after any unit is hurt: each hit fans out ×5.
    const Echo = unit("Echo", 30, 1, { on: "Hurt" }, [{ kind: "allEnemies" }], ["Ping"]);
    const team = Array.from({ length: 5 }, (_, i) => ({ ...Echo, name: `Echo${i + 1}` }));
    const log = run(team, team);
    const capped = log.filter((e) => e.type === "ChainCapped");
    expect(capped.length).toBeGreaterThan(0);
    for (const c of capped) expect(c).toMatchObject({ steps: DEFAULT_CHAIN_STEP_CAP });
    expect(log[log.length - 1]!.type).toBe("BattleEnd");
  });

  test("the cap must be a positive integer", () => {
    expect(() => run([Shieldbearer], [dummy("Dummy", 9, 1)], { chainStepCap: 0 })).toThrow(/chainStepCap/);
  });
});

describe("validate knows the chain vocabulary", () => {
  test("StatChanged triggers and otherAlly pass; a StatChanged interceptor and a bad sign are rejected", () => {
    expect(validateTeam([Smith, Archer, Caller], stressRegistry, abilities)).toEqual([]);
    const intercept = { ...Archer, triggers: [{ kind: "interceptor" as const, on: { on: "StatChanged" as const } }] };
    expect(validateTeam([intercept], stressRegistry, abilities).map((i) => i.message).join("\n")).toMatch(/can only be a trigger/);
    const badSign = { ...Archer, triggers: [trigger({ on: "StatChanged", sign: "up" as "gain" })] };
    expect(validateTeam([badSign], stressRegistry, abilities).map((i) => i.message).join("\n")).toMatch(/unknown sign/);
  });
});
