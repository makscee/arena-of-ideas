// Battle viewer logic (mission #574, slice 9): tap-a-change traces, captions,
// playback steps and "why I lost", read off real kernel logs.

import { describe, expect, test } from "vitest";
import { battle } from "../battle.js";
import { stressAbilities, stressRegistry } from "../content/stress.js";
import type { AbilityDef, AbilityRegistry, BattleEvent, UnitDef, When } from "../types.js";
import { BEAT_MAX_MS, BEAT_MS, beatPlayOf, beatTiming, captionOf, captionSubject, changeOf, endCaption, stepsOf, traceOf, whyILost } from "./trace.js";

const ab = (name: string, family: AbilityDef["family"], effects: AbilityDef["effects"]): AbilityDef => ({ name, family, effects });
const n = (value: number) => ({ kind: "const" as const, value });

const abilities: AbilityRegistry = {
  ...stressAbilities,
  GiveShield: ab("GiveShield", "Shield", [{ kind: "applyStatus", status: "Shield", stacks: n(2) }]),
  GiveStrength: ab("GiveStrength", "Strike", [{ kind: "applyStatus", status: "Strength", stacks: n(1) }]),
  Shoot: ab("Shoot", "Strike", [{ kind: "damage", amount: n(2) }]),
  Mend: ab("Mend", "Heal", [{ kind: "heal", amount: n(1) }]),
  Envenom: ab("Envenom", "Poison", [{ kind: "applyStatus", status: "Poison", stacks: n(1) }]),
  Chill: ab("Chill", "Strike", [{ kind: "applyStatus", status: "Freeze", stacks: n(1) }]),
};

const unit = (name: string, hp: number, pwr: number, on: When["on"], who: UnitDef["selectors"], does: string[]): UnitDef => ({
  name,
  base: { hp, pwr },
  triggers: [{ kind: "trigger", on }],
  selectors: who!,
  abilities: does,
});
const dummy = (name: string, hp: number, pwr: number): UnitDef =>
  unit(name, hp, pwr, { on: "Death", unit: "holder" }, [{ kind: "holder" }], ["Strike"]);

const Shieldbearer = unit("Shieldbearer", 6, 1, { on: "BattleStart" }, [{ kind: "holder" }], ["GiveShield"]);
const Smith = unit("Smith", 5, 1, { on: "StatusApplied", unit: "otherAlly", status: "Shield" }, [{ kind: "eventUnit" }], ["GiveStrength"]);
const Archer = unit("Archer", 4, 1, { on: "StatChanged", unit: "ally", stat: "pwr", sign: "gain" }, [{ kind: "frontEnemy" }], ["Shoot"]);
const Medic = unit("Medic", 5, 1, { on: "Hurt", unit: "otherAlly" }, [{ kind: "eventUnit" }], ["Mend"]);
const Zealot = unit("Zealot", 5, 1, { on: "Heal", unit: "otherAlly" }, [{ kind: "frontEnemy" }], ["Envenom"]);

const run = (teamA: UnitDef[], teamB: UnitDef[]): BattleEvent[] => battle({ teamA, teamB, seed: 7, statuses: stressRegistry, abilities });

describe("tap a change to trace its chain", () => {
  test("a 3-unit cascade reads nearest cause first: −2 ← Archer ← Smith ← Shieldbearer", () => {
    const log = run([Shieldbearer, Smith, Archer], [dummy("Dummy", 20, 1)]);
    const shot = log.find((e) => e.type === "Hurt" && e.source !== "kernel")!;
    const t = traceOf(log, shot.id);
    expect(t.text).toBe("−2 ← Archer ← Smith ← Shieldbearer");
    expect(t.links.map((l) => [l.unit, l.side])).toEqual([
      ["A3:Archer", "A"],
      ["A2:Smith", "A"],
      ["A1:Shieldbearer", "A"],
    ]);
    expect(t.change).toEqual({ eventId: shot.id, unit: "B1:Dummy", kind: "damage", label: "−2" });
  });

  test("a strike crosses sides: Poison ← Zealot ← Medic ← Dummy (who struck first)", () => {
    const log = run([dummy("Squire", 8, 1), Medic, Zealot], [dummy("Dummy", 30, 2)]);
    const poison = log.find((e) => e.type === "StatusApplied" && e.status === "Poison")!;
    const t = traceOf(log, poison.id);
    expect(t.text).toBe("Poison ×1 ← Zealot ← Medic ← Dummy");
    expect(t.links.map((l) => l.side)).toEqual(["A", "A", "B"]);
  });

  test("a status tick traces to whoever applied the status", () => {
    const log = run([dummy("Squire", 8, 1), Medic, Zealot], [dummy("Dummy", 30, 2)]);
    const tick = log.find((e) => e.type === "Hurt" && e.source !== "kernel" && e.source.status === "Poison")!;
    expect(traceOf(log, tick.id).text).toBe("−1 ← Zealot (Poison) ← Medic ← Dummy");
    expect(captionOf(log, tick.id)).toBe("Poison → Dummy −1");
  });

  test("every change in a battle traces without throwing and ends in a named cause", () => {
    const log = run([Shieldbearer, Smith, Archer, Medic, Zealot], [dummy("Dummy", 30, 3), Medic, Zealot]);
    for (const e of log) {
      if (!changeOf(e)) continue;
      const t = traceOf(log, e.id);
      expect(t.text.length).toBeGreaterThan(0);
      expect(t.text).not.toContain("undefined");
    }
  });
});

describe("whose unit a caption is about (#587)", () => {
  test("an enemy's Freeze stopping your strike is about your unit: 'Freeze on Rose → stops its strike'", () => {
    const Froster = unit("Froster", 6, 1, { on: "BattleStart" }, [{ kind: "frontEnemy" }], ["Chill"]);
    const log = run([dummy("Rose", 6, 2)], [Froster]);
    const stop = log.find((e) => e.type === "Intercepted");
    expect(stop).toBeDefined();
    expect(captionOf(log, stop!.id)).toBe("Freeze on Rose → stops its strike");
    expect(captionSubject(log, stop!.id)).toBe("A1:Rose");
    const step = stepsOf(log).find((s) => s.eventIds.includes(stop!.id))!;
    expect(step.subjectSide).toBe("A");
    expect(step.actorSide).toBe("B"); // the Freeze came from Froster
  });

  test("a strike is about the striker; a status tick about the unit it hurts", () => {
    const log = run([dummy("Squire", 8, 1), Medic, Zealot], [dummy("Dummy", 30, 2)]);
    const hit = log.find((e) => e.type === "Hurt" && e.source === "kernel")!;
    const striker = (log[hit.causedBy!] as Extract<BattleEvent, { type: "Strike" }>).striker;
    expect(captionSubject(log, hit.id)).toBe(striker);
    const tick = log.find((e) => e.type === "Hurt" && e.source !== "kernel" && e.source.status === "Poison")!;
    expect(captionSubject(log, tick.id)).toBe("B1:Dummy");
  });
});

describe("playback steps", () => {
  test("strikes are captioned on their hurt; status stat changes merge into the status step", () => {
    const log = run([Shieldbearer, Smith, Archer], [dummy("Dummy", 20, 1)]);
    const steps = stepsOf(log);
    const strength = steps.find((s) => s.caption.includes("Strength"))!;
    expect(strength.caption).toBe("Smith → Strength ×1 on Shieldbearer (+1 PWR)");
    expect(strength.changes.map((c) => c.label)).toEqual(["Strength ×1", "+1 PWR"]);
    expect(strength.actor).toBe("A2:Smith");
    const hit = steps.find((s) => s.caption.includes("strikes"))!;
    expect(hit.caption).toMatch(/^\w+ strikes \w+ → −\d+$/);
    // steps cover every non-silent event exactly once, in log order
    const ids = steps.flatMap((s) => s.eventIds);
    expect(ids).toEqual([...ids].sort((a, b) => a - b));
    expect(new Set(ids).size).toBe(ids.length);
    expect(steps.at(-1)!.caption).toMatch(/Side [AB] wins|Draw/);
    expect(stepsOf(log, undefined, undefined, { you: "A" }).at(-1)!.caption).toMatch(/You win|They win|Draw/);
  });

  test("the end reads from the viewer's side, or names the winner when there is none", () => {
    expect(endCaption("A", { you: "A" })).toBe("You win");
    expect(endCaption("A", { you: "B" })).toBe("They win");
    expect(endCaption("B", {})).toBe("Side B wins");
    expect(endCaption("B", { sideName: (s) => (s === "A" ? "@ann" : "@bob") })).toBe("@bob wins");
    expect(endCaption("draw", { you: "B" })).toBe("Draw");
  });

  test("a death is captioned with its cause and lights the killer", () => {
    const log = run([dummy("Martyr", 1, 1)], [dummy("Dummy", 30, 3)]);
    const death = stepsOf(log).find((s) => s.caption.endsWith("falls"))!;
    expect(death.caption).toBe("Dummy → Martyr falls");
    expect(death.actor).toBe("B1:Dummy");
  });
});

describe("why I lost", () => {
  test("ranks the enemy chains by damage to your units plus healing to theirs", () => {
    const log = run([dummy("Squire", 8, 1), dummy("Page", 6, 1)], [dummy("Dummy", 30, 2), Medic]);
    expect(log.at(-1)).toMatchObject({ type: "BattleEnd", winner: "B" });
    const chains = whyILost(log, "A");
    expect(chains.length).toBeGreaterThan(0);
    expect(chains.length).toBeLessThanOrEqual(3);
    expect(chains[0]!.text).toBe("Dummy");
    expect(chains[0]!.damage).toBe(14); // Squire 8 + Page 6, overkill not counted
    expect(chains[0]!.kills).toBe(2);
    for (let i = 1; i < chains.length; i++) expect(chains[i - 1]!.impact).toBeGreaterThanOrEqual(chains[i]!.impact);
    // healing on their side counts for the healer's chain
    const heal = chains.find((c) => c.names[0] === "Medic");
    if (heal) expect(heal.heal).toBeGreaterThan(0);
  });

  test("two units with one name are two chains, each named by its slot", () => {
    const Gunner = unit("Gunner", 5, 1, { on: "BattleStart" }, [{ kind: "frontEnemy" }], ["Shoot"]);
    const log = run([dummy("Squire", 8, 1), dummy("Page", 6, 1)], [dummy("Dummy", 30, 2), Gunner, Gunner]);
    const chains = whyILost(log, "A", 10);
    expect(chains.filter((c) => c.names[0] === "Gunner").map((c) => [c.text, c.units[0]])).toEqual([
      ["Gunner #2", "B2:Gunner"],
      ["Gunner #3", "B3:Gunner"],
    ]);
    for (const c of chains) expect(c.hits + c.heals).toBe(c.times);
    // one row per instance and wording: no two rows read the same
    expect(new Set(chains.map((c) => c.text)).size).toBe(chains.length);
  });

  test("a unit's strikes and its ability make one row, not two that read the same", () => {
    const log = run([dummy("Squire", 8, 1), dummy("Page", 6, 1)], [dummy("Dummy", 30, 2), Medic]);
    const chains = whyILost(log, "A", 10);
    expect(new Set(chains.map((c) => c.text)).size).toBe(chains.length);
  });

  test("a row's traced hit is one that did damage, never a −0 a Shield took whole", () => {
    // Shieldbearer's Shield 2 takes Dummy's first strike (2) whole: a Hurt of 0.
    const log = run([Shieldbearer], [dummy("Dummy", 30, 2)]);
    const zero = log.find((e) => e.type === "Hurt" && e.unit === "A1:Shieldbearer" && e.amount === 0);
    expect(zero).toBeDefined();
    const row = whyILost(log, "A").find((c) => c.names[0] === "Dummy")!;
    const sample = log[row.sampleEventId]!;
    expect(sample).toMatchObject({ type: "Hurt", unit: "A1:Shieldbearer" });
    expect(sample.type === "Hurt" && sample.amount).toBeGreaterThan(0);
    expect(row.sampleEventId).toBeGreaterThan(zero!.id);
    expect(traceOf(log, row.sampleEventId).text).not.toMatch(/^−0/);
  });

  test("a row traces the chain's killing blow, not its first, smallest hit (#587)", () => {
    const log = run([dummy("Squire", 8, 1)], [dummy("Dummy", 30, 3)]);
    const row = whyILost(log, "A").find((c) => c.names[0] === "Dummy")!;
    expect(row.kills).toBe(1);
    expect(row.sampleKind).toBe("kill");
    const death = log.find((e) => e.type === "Death" && e.unit === "A1:Squire")!;
    expect(row.sampleEventId).toBe(death.causedBy);
    const firstHit = log.find((e) => e.type === "Hurt" && e.unit === "A1:Squire")!;
    expect(row.sampleEventId).toBeGreaterThan(firstHit.id);
  });

  test("a heal row's traced first change healed something", () => {
    const log = run([dummy("Squire", 8, 1), dummy("Page", 6, 1)], [dummy("Dummy", 30, 2), Medic]);
    for (const c of whyILost(log, "A", 10)) {
      const e = log[c.sampleEventId]!;
      expect(e.type === "Hurt" || e.type === "Heal").toBe(true);
      if (e.type === "Hurt" || e.type === "Heal") expect(e.amount).toBeGreaterThan(0);
    }
  });

  test("your own units' acts never show up as enemy chains", () => {
    const log = run([Shieldbearer, Smith, Archer], [dummy("Dummy", 20, 1)]);
    for (const c of whyILost(log, "A")) expect(c.names[0]).not.toMatch(/Shieldbearer|Smith|Archer/);
  });
});

describe("one beat at a time (R2-12)", () => {
  const Coach = unit("Coach", 6, 1, { on: "BattleStart" }, [{ kind: "allAllies" }], ["GiveStrength"]);
  const Bulwark = unit("Bulwark", 6, 1, { on: "BattleStart" }, [{ kind: "holder" }], ["GiveShield"]);

  test("waves land 150 ms apart; a beat lasts 1.2 s, at most 1.5 s", () => {
    expect(beatTiming(1)).toEqual({ at: [0], ms: BEAT_MS });
    expect(beatTiming(3)).toEqual({ at: [0, 150, 300], ms: BEAT_MS });
    const long = beatTiming(30);
    expect(long.ms).toBe(BEAT_MAX_MS);
    expect(long.at.at(-1)!).toBeLessThanOrEqual(700);
  });

  test("every step's events play exactly once, inside their own beat", () => {
    const log = run([Shieldbearer, Smith, Archer, Medic, Zealot], [dummy("Dummy", 30, 3), Medic, Zealot]);
    const steps = stepsOf(log);
    const beats = beatPlayOf(log, stepsOf(log));
    expect(beats.length).toBeLessThan(steps.length);
    const played = beats.flatMap((b) => b.waves.flatMap((w) => w.eventIds));
    expect(played.sort((p, q) => p - q)).toEqual(steps.flatMap((s) => s.eventIds).sort((p, q) => p - q));
    for (const b of beats) {
      expect(b.waves.length).toBeGreaterThan(0);
      for (const id of b.waves.flatMap((w) => w.eventIds)) expect(id >= b.start && id <= b.end).toBe(true);
    }
  });

  test("a buff on all allies plays as one wave", () => {
    const log = run([Coach, dummy("Squire", 8, 1), dummy("Page", 8, 1)], [dummy("Dummy", 30, 1)]);
    const first = beatPlayOf(log, stepsOf(log))[0]!;
    const buff = first.waves.find((w) => w.changes.some((c) => c.label === "Strength ×1"))!;
    expect(new Set(buff.changes.filter((c) => c.kind === "status").map((c) => c.unit))).toEqual(new Set(["A1:Coach", "A2:Squire", "A3:Page"]));
    expect(buff.caption).toBe("Coach → Strength ×1 on Coach, Squire, Page");
  });

  test("Shield that absorbs a hit fades in the hit's wave, not its own", () => {
    const log = run([Bulwark], [dummy("Dummy", 30, 1)]);
    const fade = log.find((e) => e.type === "StatusRemoved" && e.status === "Shield" && e.causedBy !== null && log[e.causedBy]?.type === "Hurt")!;
    expect(fade).toBeDefined();
    const wave = beatPlayOf(log, stepsOf(log)).flatMap((b) => b.waves).find((w) => w.eventIds.includes(fade.id))!;
    expect(wave.eventIds[0]).toBe(fade.causedBy);
    expect(wave.caption).toMatch(/absorbed\), Shield −1$/);
  });
});
