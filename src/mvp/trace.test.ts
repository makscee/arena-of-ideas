// Battle viewer logic (mission #574, slice 9): tap-a-change traces, captions,
// playback steps and "why I lost", read off real kernel logs.

import { describe, expect, test } from "vitest";
import { battle } from "../battle.js";
import { displayNames } from "../trace.js";
import { stressAbilities, stressRegistry } from "../content/stress.js";
import type { AbilityDef, AbilityRegistry, BattleEvent, UnitDef, When } from "../types.js";
import { BEAT_MAX_MS, BEAT_MS, QUIET_BEAT_MS, beatPlayOf, beatTiming, captionOf, chainOf, captionSubject, changeOf, damageByUnit, endCaption, keyMomentsOf, firingOf, stepsOf, timelineOf, timingOf, traceOf, turnLabel, whyILost } from "./trace.js";

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

  test("waves land 150 ms apart; a beat lasts 1 s, at most 1.4 s", () => {
    expect(beatTiming(1)).toEqual({ at: [0], ms: BEAT_MS });
    expect(beatTiming(3)).toEqual({ at: [0, 150, 300], ms: BEAT_MS });
    const long = beatTiming(30);
    expect(long.ms).toBe(BEAT_MAX_MS);
    expect(long.at.at(-1)!).toBeLessThanOrEqual(700);
  });

  test("a quiet beat (one wave of plain hits) is shorter than a kill", () => {
    const log = run([dummy("Squire", 20, 1)], [dummy("Dummy", 3, 1)]);
    const beats = beatPlayOf(log, stepsOf(log));
    const plain = beats.find((b) => b.waves.length === 1 && b.waves[0]!.changes.every((c) => c.kind === "damage"))!;
    const kill = beats.find((b) => b.waves.some((w) => w.changes.some((c) => c.kind === "death")))!;
    expect(timingOf(plain).ms).toBe(QUIET_BEAT_MS);
    expect(timingOf(kill).ms).toBeGreaterThanOrEqual(BEAT_MS);
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
    // Shield took the whole hit: the caption shows the block, never "−0".
    expect(wave.caption).toBe("Dummy strikes Bulwark → Shield blocks 1");
    expect(wave.changes.find((c) => c.kind === "damage")?.label).toBe("1 blocked");
  });

  test("a hit that does nothing reads as \"no damage\", never −0", () => {
    const log = run([dummy("Rose", 6, 1)], [dummy("Pebble", 30, 0)]);
    const hit = log.find((e) => e.type === "Hurt" && e.unit === "A1:Rose" && e.amount === 0)!;
    expect(hit).toBeDefined();
    expect(changeOf(hit)?.label).toBe("no damage");
    expect(captionOf(log, hit.id)).toBe("Pebble strikes Rose → no damage");
    const captions = beatPlayOf(log, stepsOf(log)).flatMap((b) => b.waves.map((w) => w.caption));
    expect(captions.filter((c) => /−0\b/.test(c))).toEqual([]);
  });
});

describe("trigger badges (R2-13)", () => {
  test("a reaction names the trigger it answered and what it did", () => {
    const log = run([Shieldbearer, Smith, Archer], [dummy("Dummy", 30, 1)]);
    const firings = stepsOf(log).map((s) => firingOf(log, s)).filter((f) => f !== null);
    const name = displayNames(log);
    const smith = firings.find((f) => name(f.unit) === "Smith")!;
    expect(smith).toMatchObject({ trigger: "trigger:StatusApplied", triggerStatus: "Shield", effect: "status:Strength", effectStatus: "Strength" });
    const archer = firings.find((f) => name(f.unit) === "Archer")!;
    expect(archer).toMatchObject({ trigger: "trigger:StatChanged", effect: "effect:damage" });
    expect(firings.find((f) => name(f.unit) === "Shieldbearer")).toMatchObject({ trigger: "trigger:BattleStart", effect: "status:Shield" });
  });

  test("a strike and a status's own tick have no badge", () => {
    const log = run([dummy("Squire", 20, 1)], [dummy("Dummy", 3, 1)]);
    const strikes = stepsOf(log).filter((s) => s.changes.some((c) => c.kind === "damage"));
    expect(strikes.length).toBeGreaterThan(0);
    for (const s of strikes) expect(firingOf(log, s)).toBeNull();
  });
});

describe("the end card (R2-14)", () => {
  test("damage by unit: each side's damage to the other, overkill not counted, biggest first", () => {
    const log = run([dummy("Squire", 8, 1), dummy("Page", 6, 1)], [dummy("Dummy", 30, 2), Medic]);
    const dmg = damageByUnit(log);
    const b = dmg.filter((d) => d.side === "B");
    expect(b[0]).toMatchObject({ name: "Dummy", damage: 14 }); // Squire 8 + Page 6
    expect(dmg.map((d) => d.side)).toEqual([...dmg.map((d) => d.side)].sort());
    for (const side of ["A", "B"] as const) {
      const s = dmg.filter((d) => d.side === side).map((d) => d.damage);
      expect(s).toEqual([...s].sort((p, q) => q - p));
    }
    // what A dealt equals what B's units lost (B didn't die, so no overkill)
    const aDealt = dmg.filter((d) => d.side === "A").reduce((t, d) => t + d.damage, 0);
    const bHurt = log.filter((e) => e.type === "Hurt" && e.unit.startsWith("B")).reduce((t, e) => t + (e.type === "Hurt" ? e.amount : 0), 0);
    expect(aDealt).toBe(bHurt);
  });

  test("key moments: 2–3, in battle order, each on a beat that holds it", () => {
    const log = run([dummy("Squire", 8, 1), dummy("Page", 6, 1)], [dummy("Dummy", 30, 2), Medic]);
    const beats = beatPlayOf(log, stepsOf(log));
    const ms = keyMomentsOf(log, beats);
    expect(ms.length).toBeGreaterThanOrEqual(2);
    expect(ms.length).toBeLessThanOrEqual(3);
    expect(ms.map((m) => m.beat)).toEqual([...ms.map((m) => m.beat)].sort((p, q) => p - q));
    expect(new Set(ms.map((m) => m.beat)).size).toBe(ms.length);
    const kill = ms.find((m) => m.kind === "kill")!;
    expect(kill.label).toMatch(/^Dummy kills (Squire|Page) \(−\d+\)$/);
    expect(beats[kill.beat]!.waves.some((w) => w.changes.some((c) => c.kind === "death"))).toBe(true);
  });

  test("a long cascade is the combo moment", () => {
    const log = run([Shieldbearer, Smith, Archer, Medic, Zealot], [dummy("Dummy", 30, 3), Medic, Zealot]);
    const beats = beatPlayOf(log, stepsOf(log));
    const most = Math.max(...beats.map((b) => b.waves.length));
    const combo = keyMomentsOf(log, beats).find((m) => m.kind === "combo");
    if (most >= 3) expect(combo).toMatchObject({ beat: beats.find((b) => b.waves.length === most)!.index, label: expect.stringMatching(new RegExp(`^Combo: ${most} steps`)) });
    else expect(combo).toBeUndefined();
  });
});

describe("Why: the chain from a change back to the turn (R2-15)", () => {
  // The issue's example: Medic strikes Victim −1; Victim's "after this unit
  // is hit → Strength" fires; Strength brings +1 PWR.
  const Victim: UnitDef = { name: "Victim", base: { hp: 30, pwr: 1 }, triggers: [{ kind: "trigger", on: { on: "Heal", unit: "holder" } }, { kind: "trigger", on: { on: "Hurt", unit: "holder" } }], selectors: [{ kind: "holder" }], abilities: ["GiveStrength"] };
  const whenOf = (defs: Record<string, UnitDef>) => (ref: { unit: string; when?: number }) => defs[ref.unit.split(":")[1]!]?.triggers?.[ref.when ?? -1];

  test("+1 PWR ← Victim's ability (its Hurt When) ← Medic strikes Victim −1 ← Turn N", () => {
    const log = run([Victim], [dummy("Medic", 30, 1)]);
    const pwr = log.find((e) => e.type === "StatChanged" && e.stat === "pwr" && e.unit === "A1:Victim")!;
    const c = chainOf(log, pwr.id, { whenOf: whenOf({ Victim }) });
    expect(c.change?.label).toBe("+1 PWR");
    expect(c.nodes.map((s) => s.kind)).toEqual(["change", "firing", "event", "root"]);
    const [change, firing, event, root] = c.nodes;
    expect(change!.text).toBe("Victim → Strength ×1 on Victim (+1 PWR)");
    // The kernel stamped the second When (Hurt), not the first (Heal).
    expect(firing!.ref).toMatchObject({ unit: "A1:Victim", ability: 0, when: 1 });
    expect(firing!.trigger).toBe("trigger:Hurt");
    expect(firing!.side).toBe("A");
    expect(event!.text).toBe("Medic strikes Victim → −1");
    expect(event!.trigger).toBe("trigger:Hurt");
    expect(log[root!.eventId]!.type).toBe("TurnStart");
    expect(root!.text).toBe(`Turn ${log[root!.eventId]!.turn}`);
    // Every step points at an earlier (or the same) moment, so the playhead can jump there.
    for (let i = 1; i < c.nodes.length; i++) expect(c.nodes[i]!.eventId).toBeLessThanOrEqual(c.nodes[i - 1]!.eventId);
  });

  test("a Strike-When ability's hit keeps its strike: Fighter → Fodder −2 ← Fighter's ability ← Fighter strikes Fodder ← Turn N (R2-17)", () => {
    // Fighter: "After this unit strikes: deal 2 damage to the front enemy".
    const Fighter = unit("Fighter", 20, 1, { on: "Strike", striker: "holder" }, [{ kind: "frontEnemy" }], ["Shoot"]);
    const log = run([Fighter], [dummy("Fodder", 30, 0)]);
    const hit = log.find((e) => e.type === "Hurt" && e.source !== "kernel" && e.source.unit === "A1:Fighter")!;
    const c = chainOf(log, hit.id);
    expect(c.nodes.map((s) => `${s.kind}:${s.text}`)).toEqual([
      "change:Fighter → Fodder −2",
      "firing:Fighter's ability",
      "event:Fighter strikes Fodder",
      `root:Turn ${log[c.nodes.at(-1)!.eventId]!.turn}`,
    ]);
    expect(c.nodes[1]!.trigger).toBe("trigger:Strike");
    expect(log[c.nodes[2]!.eventId]!.type).toBe("Strike");
    // The strike's own hit still reads as one step with its strike.
    const own = log.find((e) => e.type === "Hurt" && e.source === "kernel" && e.unit === "B1:Fodder")!;
    expect(chainOf(log, own.id).nodes.map((s) => s.kind)).toEqual(["change", "root"]);
  });

  test("a cascade alternates firings and the events that set them off, back to the battle's start", () => {
    const log = run([Shieldbearer, Smith, Archer], [dummy("Dummy", 20, 1)]);
    const shot = log.find((e) => e.type === "Hurt" && e.source !== "kernel")!;
    const c = chainOf(log, shot.id);
    expect(c.nodes.map((s) => `${s.kind}:${s.kind === "firing" ? s.unit : s.text}`)).toEqual([
      "change:Archer → Dummy −2",
      "firing:A3:Archer",
      "event:Smith → Strength ×1 on Shieldbearer (+1 PWR)",
      "firing:A2:Smith",
      "event:Shieldbearer → Shield ×2 on Shieldbearer",
      "firing:A1:Shieldbearer",
      "root:Battle start",
    ]);
    expect(c.nodes.filter((s) => s.kind === "firing").map((s) => s.trigger)).toEqual(["trigger:StatChanged", "trigger:StatusApplied", "trigger:BattleStart"]);
  });

  test("a status tick shows the status, then goes on from where it was put on", () => {
    const log = run([dummy("Squire", 8, 1), Medic, Zealot], [dummy("Dummy", 30, 2)]);
    const tick = log.find((e) => e.type === "Hurt" && e.source !== "kernel" && e.source.status === "Poison")!;
    const c = chainOf(log, tick.id);
    const firing = c.nodes[1]!;
    expect(firing).toMatchObject({ kind: "firing", status: "Poison", text: "Poison on Dummy" });
    expect(log[firing.origin!]!.type).toBe("StatusApplied");
    expect(c.nodes[2]!.eventId).toBe(firing.origin);
    expect(c.nodes.filter((s) => s.kind === "firing" && !s.status).map((s) => s.unit)).toEqual(["A3:Zealot", "A2:Medic"]);
    expect(c.nodes.at(-1)!.kind).toBe("root");
  });

  test("every change in a battle has a chain that starts at it and never says undefined", () => {
    const log = run([Shieldbearer, Smith, Archer, Medic, Zealot], [dummy("Dummy", 30, 3), Medic, Zealot]);
    for (const e of log) {
      if (!changeOf(e)) continue;
      const c = chainOf(log, e.id);
      expect(c.nodes[0]!.kind).toBe("change");
      expect(c.nodes[0]!.eventId).toBe(e.id);
      for (const s of c.nodes) expect(s.text).not.toContain("undefined");
    }
  });

  test("firingOf reads the stamped When when it can look it up", () => {
    const log = run([Victim], [dummy("Medic", 30, 1)]);
    const st = log.find((e) => e.type === "StatusApplied" && e.source !== "kernel" && e.source.unit === "A1:Victim")!;
    const step = { eventIds: [st.id], changes: [changeOf(st)!] };
    expect(firingOf(log, step, whenOf({ Victim }))?.trigger).toBe("trigger:Hurt");
    // A Victim whose stamped When is Heal would read as Heal: the stamp wins over the event.
    expect(firingOf(log, step, () => ({ kind: "trigger", on: { on: "Heal" } }))?.trigger).toBe("trigger:Heal");
  });
});

describe("R2-17: key moments, battle start, fatigue rows, Why's icons", () => {
  test("a fight with more in it fills its key moments up to 3, never two on one beat", () => {
    const log = run([dummy("Squire", 8, 1), dummy("Page", 6, 1), dummy("Knave", 5, 1)], [dummy("Dummy", 30, 2), Medic]);
    const beats = beatPlayOf(log, stepsOf(log));
    const ms = keyMomentsOf(log, beats);
    const deaths = log.filter((e) => e.type === "Death").length;
    expect(deaths).toBeGreaterThanOrEqual(3);
    expect(ms.length).toBe(3);
    expect(new Set(ms.map((m) => m.beat)).size).toBe(3);
    expect(ms.map((m) => m.beat)).toEqual([...ms.map((m) => m.beat)].sort((p, q) => p - q));
  });

  test("battle start is its own timeline block (turn 0), labelled Start", () => {
    const log = run([Shieldbearer, Smith, Archer], [dummy("Dummy", 30, 2)]);
    const beats = beatPlayOf(log, stepsOf(log));
    const tl = timelineOf(log, beats);
    const early = beats.filter((b) => b.turn < 1).map((b) => b.index);
    expect(early.length).toBeGreaterThan(0);
    expect(tl[0]).toMatchObject({ turn: 0, beats: early });
    expect(tl[1]!.turn).toBe(1);
    expect(turnLabel(0)).toBe("Start");
    expect(turnLabel(3)).toBe("T3");
  });

  test("a wave that hits both sides (fatigue) carries no side tag", () => {
    const log = run([dummy("Wall", 60, 0), dummy("Wall2", 60, 0)], [dummy("Wall", 60, 0), dummy("Wall2", 60, 0)]);
    const beats = beatPlayOf(log, stepsOf(log));
    const sides = new Map(log.flatMap((e) => (e.type === "BattleStart" ? [...e.teams.A.map((r) => [r.id, "A"] as const), ...e.teams.B.map((r) => [r.id, "B"] as const)] : [])));
    const mixed = beats.flatMap((b) => b.waves).filter((w) => new Set(w.changes.map((c) => sides.get(c.unit))).size > 1);
    expect(mixed.length).toBeGreaterThan(0);
    for (const w of mixed) expect(w.subjectSide).toBeNull();
  });

  test("every Why step names its event's type (a fatigue root's icon is its hourglass)", () => {
    const log = run([dummy("Wall", 60, 0)], [dummy("Wall", 60, 0)]);
    const hit = log.find((e) => e.type === "Hurt" && e.causedBy !== null && log[e.causedBy]?.type === "Fatigue");
    if (!hit) return;
    const c = chainOf(log, hit.id);
    expect(c.nodes.at(-1)).toMatchObject({ kind: "root", event: "Fatigue" });
    expect(c.nodes[0]).toMatchObject({ kind: "change", event: "Hurt" });
  });
});

describe("the desktop timeline (R2-16)", () => {
  test("one block per turn, in order, every beat in exactly one block", () => {
    const log = run([dummy("Squire", 8, 1), dummy("Page", 6, 1)], [dummy("Dummy", 30, 2), Medic]);
    const beats = beatPlayOf(log, stepsOf(log));
    const tl = timelineOf(log, beats);
    expect(tl.map((t) => t.turn)).toEqual([...new Set(tl.map((t) => t.turn))].sort((p, q) => p - q));
    expect(tl.flatMap((t) => t.beats)).toEqual(beats.map((b) => b.index));
    for (const t of tl) for (const i of t.beats) if (beats[i]!.turn >= 1) expect(beats[i]!.turn).toBe(t.turn);
  });

  test("each death is marked once, on the beat that shows it, in the fallen side", () => {
    const log = run([dummy("Squire", 8, 1), dummy("Page", 6, 1)], [dummy("Dummy", 30, 2), Medic]);
    const beats = beatPlayOf(log, stepsOf(log));
    const deaths = timelineOf(log, beats).flatMap((t) => t.marks.filter((m) => m.kind === "death"));
    expect(deaths.length).toBe(log.filter((e) => e.type === "Death").length);
    for (const m of deaths) {
      expect(m.side).toBe("A");
      expect(beats[m.beat]!.waves.some((w) => w.changes.some((c) => c.kind === "death" && c.unit === m.unit))).toBe(true);
    }
  });

  test("fatigue is marked once, on the turn it set in", () => {
    const log = run([dummy("Wall", 60, 0)], [dummy("Wall", 60, 0)]);
    const beats = beatPlayOf(log, stepsOf(log));
    const fat = timelineOf(log, beats).flatMap((t) => t.marks.filter((m) => m.kind === "fatigue").map(() => t.turn));
    const first = log.find((e) => e.type === "Fatigue");
    if (first) expect(fat).toEqual([first.turn]);
    else expect(fat).toEqual([]);
  });
});
