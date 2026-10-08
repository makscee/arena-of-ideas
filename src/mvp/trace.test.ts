// Battle viewer logic (mission #574, slice 9): tap-a-change traces, captions,
// playback steps and "why I lost", read off real kernel logs.

import { describe, expect, test } from "vitest";
import { battle } from "../battle.js";
import { displayNames } from "../trace.js";
import { stressAbilities, stressRegistry } from "../content/stress.js";
import type { AbilityDef, AbilityRegistry, BattleEvent, UnitDef, When } from "../types.js";
import { MVP_RULES, type MvpContent, type PlayerRef } from "./contract.js";
import { fightLines } from "./fight.js";
import { lineUnitOf } from "./forms.js";
import { mvpPool } from "./units.js";
import { setTraceLang, BEAT_MAX_MS, BEAT_MS, EMPHASIS_MS, END_BEAT_MS, QUIET_BEAT_MS, beatPlayOf, beatTiming, weightsOf, captionOf, chainOf, captionSubject, changeOf, damageByUnit, endCaption, keyMomentsOf, firingOf, causeOf, beamsOf, stepsOf, sidesOf, timelineOf, timingOf, traceOf, turnLabel, turnSummaryOf, whyILost, TURN_END_MS, turnEndHoldMs, turnEndsOf, totalsPartsOf, totalsText, runningTotalsOf, runRowText, turnSoFarIds, beatIdsOf, type UnitTurnTotals } from "./trace.js";

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

  test("the attacker (M3-1): the Hedgehog's Poison lands on the striker and traces to its strike", () => {
    const Hedgehog = unit("Hedgehog", 30, 1, { on: "Hurt", unit: "holder" }, [{ kind: "attacker" }], ["Envenom"]);
    const log = run([Hedgehog], [dummy("Brute", 30, 2)]);
    const poison = log.find((e) => e.type === "StatusApplied" && e.status === "Poison")!;
    expect(poison).toMatchObject({ unit: "B1:Brute" });
    expect(traceOf(log, poison.id).text).toBe("Poison ×1 ← Hedgehog ← Brute");
    expect(captionOf(log, poison.id)).toMatch(/Brute/);
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
    // A battle the turn cap stopped says so (R3-26).
    const log: BattleEvent[] = [
      { id: 0, turn: 0, causedBy: null, source: "kernel", type: "BattleStart", teams: { A: [], B: [] } },
      { id: 1, turn: 30, causedBy: null, source: "kernel", type: "BattleEnd", winner: "draw", turns: 30, timeUp: true },
    ];
    expect(captionOf(log, 1)).toBe("Time's up: draw");
    expect(captionOf([log[0]!, { id: 1, turn: 30, causedBy: null, source: "kernel", type: "BattleEnd", winner: "draw", turns: 30 }], 1)).toBe("Draw");
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

  test("waves land 220 ms apart; a beat lasts 1.3 s, at most 2.2 s (round 3, note 14)", () => {
    expect([BEAT_MS, BEAT_MAX_MS, QUIET_BEAT_MS]).toEqual([1300, 2200, 900]);
    expect(beatTiming(1)).toEqual({ at: [0], ms: BEAT_MS });
    expect(beatTiming(3)).toEqual({ at: [0, 220, 440], ms: BEAT_MS });
    expect(beatTiming(4)).toEqual({ at: [0, 220, 440, 660], ms: 1460 });
    const long = beatTiming(30);
    expect(long.ms).toBe(BEAT_MAX_MS);
    expect(long.at.at(-1)!).toBe(1400);
  });

  test("a quiet beat (one wave of plain hits) stays at 0.9 s and is shorter than a kill", () => {
    const log = run([dummy("Squire", 20, 1)], [dummy("Dummy", 3, 1)]);
    const beats = beatPlayOf(log, stepsOf(log));
    const weights = weightsOf(log, beats);
    const plain = beats.find((b) => b.waves.length === 1 && b.waves[0]!.changes.every((c) => c.kind === "damage"))!;
    const kill = beats.find((b) => b.waves.some((w) => w.changes.some((c) => c.kind === "death")))!;
    expect(weights[plain.index]).toEqual({ quiet: true });
    expect(timingOf(plain, weights[plain.index])).toEqual({ at: [0], ms: QUIET_BEAT_MS });
    expect(weights[kill.index]!.kill).toBe(true);
    expect(timingOf(kill, weights[kill.index]).ms).toBeGreaterThanOrEqual(BEAT_MS + EMPHASIS_MS.kill);
  });

  test("R3-26: quiet beats play shorter, a repeat of the turn before shorter still; kills keep their time", () => {
    // Medic heals Squire each time Dummy's 1-PWR strike lands: the same two lines every turn.
    const log = run([dummy("Squire", 30, 0), Medic], [dummy("Dummy", 30, 1)]);
    const beats = beatPlayOf(log, stepsOf(log));
    const weights = weightsOf(log, beats);
    const trade = beats.filter((b) => b.waves.length > 1 && weights[b.index]!.quiet);
    expect(trade.length).toBeGreaterThan(1);
    const first = trade[0]!;
    expect(weights[first.index]!.repeat).toBeUndefined();
    expect(timingOf(first, weights[first.index]).ms).toBeLessThan(BEAT_MS);
    const again = trade.find((b) => weights[b.index]!.repeat)!;
    expect(again).toBeDefined();
    expect(timingOf(again, weights[again.index]).ms).toBeLessThan(timingOf(first, weights[first.index]).ms);
    // Nothing with a kill, a summon, a big hit, fatigue's first beat or the last beat is quiet.
    for (const [i, w] of weights.entries()) if (w.kill || w.big || w.summon || w.fatigue || w.last || w.end) expect([i, w.quiet, w.repeat]).toEqual([i, undefined, undefined]);
  });

  test("a kill beat is longer than the same beat without its death", () => {
    const log = run([dummy("Squire", 20, 1)], [dummy("Dummy", 3, 1)]);
    const beats = beatPlayOf(log, stepsOf(log));
    const kill = beats.find((b) => b.waves.some((w) => w.changes.some((c) => c.kind === "death")))!;
    expect(timingOf(kill, { kill: true }).ms - timingOf(kill).ms).toBe(EMPHASIS_MS.kill);
  });

  test("timingOf adds each big moment's time: kill 350, big hit 200, summon 250, first fatigue 500, last 600", () => {
    const log = run([dummy("Squire", 20, 1)], [dummy("Dummy", 3, 1)]);
    const beat = beatPlayOf(log, stepsOf(log))[0]!;
    const base = timingOf(beat).ms;
    expect(timingOf(beat, { kill: true }).ms).toBe(base + 350);
    expect(timingOf(beat, { big: true }).ms).toBe(base + 200);
    expect(timingOf(beat, { summon: true }).ms).toBe(base + 250);
    expect(timingOf(beat, { fatigue: true }).ms).toBe(base + 500);
    expect(timingOf(beat, { last: true }).ms).toBe(base + 600);
    expect(timingOf(beat, { kill: true, big: true, last: true }).ms).toBe(base + 1150);
    // The wave times don't move: the emphasis is held after the last wave.
    expect(timingOf(beat, { kill: true, last: true }).at).toEqual(timingOf(beat).at);
  });

  test("weightsOf: big hits at 4+, the last beat, and fatigue only on its first beat", () => {
    const log = run([dummy("Wall", 200, 1)], [dummy("Wall", 200, 5)]);
    const beats = beatPlayOf(log, stepsOf(log));
    const weights = weightsOf(log, beats);
    // The deciding blow's beat holds, not the empty BattleEnd beat after it.
    const decisive = beats.map((b) => b.waves.some((w) => w.changes.length)).lastIndexOf(true);
    expect(beats.at(-1)!.waves.every((w) => !w.changes.length)).toBe(true);
    expect(weights[decisive]!.last).toBe(true);
    expect(weights.filter((w) => w.last)).toHaveLength(1);
    // The empty BattleEnd beat after it is short, so the end card follows the blow's hold.
    expect(weights.at(-1)!.end).toBe(true);
    expect(weights.filter((w) => w.end)).toHaveLength(1);
    expect(timingOf(beats.at(-1)!, weights.at(-1)).ms).toBe(END_BEAT_MS);
    const big = beats.filter((b) => b.waves.some((w) => w.eventIds.some((id) => { const e = log[id]!; return e.type === "Hurt" && e.amount >= 4; })));
    expect(big.length).toBeGreaterThan(0);
    for (const b of big) expect(weights[b.index]!.big).toBe(true);
    if (log.some((e) => e.type === "Fatigue")) expect(weights.filter((w) => w.fatigue)).toHaveLength(1);
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

describe("why I lost: a loop of units reads once (R2-17)", () => {
  // Hand-made, as a playtest saw it: Virus poisons Squire whenever Guardian's
  // Shield breaks, and Guardian shields again whenever Squire takes a Poison
  // tick, turn after turn. The walk up the causes runs Virus (Poison) ←
  // Guardian (Shield) ← Virus (Poison) ← … ← Silehman, who struck first.
  const ev = (id: number, turn: number, causedBy: number | null, source: BattleEvent["source"], body: object) => ({ id, turn, causedBy, source, ...body }) as BattleEvent;
  const K = "kernel" as const;
  const virus = { unit: "B2:Virus", ability: 0 };
  const guardian = { unit: "B3:Guardian", ability: 0 };
  const tick = { unit: "A1:Squire", status: "Poison", ability: 0 };
  const shieldOn = { unit: "B3:Guardian", status: "Shield", ability: 0 };
  const log: BattleEvent[] = [
    ev(0, 0, null, K, { type: "BattleStart", teams: { A: [{ id: "A1:Squire", name: "Squire", hp: 20, pwr: 1 }], B: [{ id: "B1:Silehman", name: "Silehman", hp: 9, pwr: 2 }, { id: "B2:Virus", name: "Virus", hp: 9, pwr: 1 }, { id: "B3:Guardian", name: "Guardian", hp: 9, pwr: 1 }] } }),
    ev(1, 1, null, K, { type: "TurnStart" }),
    ev(2, 1, 1, K, { type: "Strike", striker: "B1:Silehman", defender: "A1:Squire" }),
    ev(3, 1, 2, K, { type: "Hurt", unit: "A1:Squire", amount: 2, hpAfter: 18 }),
    ev(4, 1, 3, virus, { type: "StatusApplied", unit: "A1:Squire", status: "Poison", stacks: 1, total: 1 }),
    ev(5, 1, null, K, { type: "TurnEnd" }),
    ev(6, 1, 5, tick, { type: "Hurt", unit: "A1:Squire", amount: 1, hpAfter: 17 }),
    ev(7, 1, 6, guardian, { type: "StatusApplied", unit: "B3:Guardian", status: "Shield", stacks: 1, total: 1 }),
    ev(8, 2, null, K, { type: "TurnStart" }),
    ev(9, 2, 8, shieldOn, { type: "StatusRemoved", unit: "B3:Guardian", status: "Shield", stacks: 1, remaining: 0 }),
    ev(10, 2, 9, virus, { type: "StatusApplied", unit: "A1:Squire", status: "Poison", stacks: 1, total: 1 }),
    ev(11, 2, null, K, { type: "TurnEnd" }),
    ev(12, 2, 11, tick, { type: "Hurt", unit: "A1:Squire", amount: 1, hpAfter: 16 }),
    ev(13, 2, 12, guardian, { type: "StatusApplied", unit: "B3:Guardian", status: "Shield", stacks: 1, total: 1 }),
    ev(14, 3, null, K, { type: "TurnStart" }),
    ev(15, 3, 14, shieldOn, { type: "StatusRemoved", unit: "B3:Guardian", status: "Shield", stacks: 1, remaining: 0 }),
    ev(16, 3, 15, virus, { type: "StatusApplied", unit: "A1:Squire", status: "Poison", stacks: 1, total: 1 }),
    ev(17, 3, null, K, { type: "TurnEnd" }),
    ev(18, 3, 17, tick, { type: "Hurt", unit: "A1:Squire", amount: 1, hpAfter: 15 }),
    ev(19, 3, null, K, { type: "BattleEnd", winner: "B", turns: 3 }),
  ];

  test("each unit shows once in a trace: the loop collapses, its starter stays", () => {
    // Before the guard: "−1 ← Virus (Poison) ← Guardian (Shield) ← Virus (Poison) ← Guardian (Shield) ← Virus (Poison) ← Silehman".
    expect(traceOf(log, 18).text).toBe("−1 ← Virus (Poison) ← Guardian (Shield) ← Silehman");
    expect(traceOf(log, 18).links.map((l) => l.unit)).toEqual(["B2:Virus", "B3:Guardian", "B1:Silehman"]);
  });

  test("the why-I-lost row reads the loop once and holds every tick of it", () => {
    const rows = whyILost(log, "A", 10);
    for (const r of rows) expect(new Set(r.units).size).toBe(r.units.length);
    const loop = rows.find((r) => r.names[0] === "Virus")!;
    expect(loop.text).toBe("Virus (Poison) ← Guardian (Shield) ← Silehman");
    // The second and third ticks are one row now (they read the same once deduplicated).
    expect(loop.damage).toBe(2);
  });

  test("real units never repeat a unit in a why-I-lost row", () => {
    const pool = mvpPool();
    const content: MvpContent = { version: "t", units: pool.units, abilities: pool.abilities, statuses: pool.statuses };
    const p = (id: string): PlayerRef => ({ id, name: id, bot: false });
    let r = 12345;
    const rand = (n: number) => ((r = (r * 1103515245 + 12345) % 2147483648), r % n);
    for (let seed = 0; seed < 120; seed++) {
      const line = (s: string) => Array.from({ length: 5 }, (_, k) => lineUnitOf(pool.units[rand(pool.units.length)]!, `${s}${k}`, 1 + rand(4)));
      const rec = fightLines({ player: p("a"), line: line("a") }, { player: p("b"), line: line("b") }, { battleId: "x", seed, kind: "round", round: 5, runId: null, at: "2026-10-05T00:00:00Z", content, rules: MVP_RULES });
      for (const side of ["A", "B"] as const) for (const row of whyILost(rec.log, side, 10)) expect(new Set(row.units).size, `seed ${seed}: ${row.text}`).toBe(row.units.length);
    }
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
    expect(kill.label).toMatch(/^(Turning point: )?Dummy kills (Squire|Page)( \(−\d+\))?$/);
    expect(beats[kill.beat]!.waves.some((w) => w.changes.some((c) => c.kind === "death"))).toBe(true);
  });

  test("a long cascade is the combo moment", () => {
    const log = run([Shieldbearer, Smith, Archer, Medic, Zealot], [dummy("Dummy", 30, 3), Medic, Zealot]);
    const beats = beatPlayOf(log, stepsOf(log));
    const most = Math.max(...beats.map((b) => b.waves.length));
    const combo = keyMomentsOf(log, beats).find((m) => m.kind === "combo");
    // R2-17: it names who did the most in it and what, not "Combo: 8 steps".
    if (most >= 3) expect(combo).toMatchObject({ beat: beats.find((b) => b.waves.length === most)!.index, label: `Zealot: 2 dmg in ${most} steps`, unit: "B3:Zealot" });
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

describe("a cause on every wave (R3-19)", () => {
  test("across 300 real fights every wave but the end has a cause, on a unit or the clash", () => {
    const pool = mvpPool();
    const content: MvpContent = { version: "t", units: pool.units, abilities: pool.abilities, statuses: pool.statuses };
    const p = (id: string): PlayerRef => ({ id, name: id, bot: false });
    let r = 777;
    const rand = (n: number) => ((r = (r * 1103515245 + 12345) % 2147483648), r % n);
    const seen = { strike: 0, poison: 0, fatigue: 0, ability: 0, death: 0 };
    for (let seed = 0; seed < 300; seed++) {
      const line = (s: string) => Array.from({ length: 5 }, (_, k) => lineUnitOf(pool.units[rand(pool.units.length)]!, `${s}${k}`, 1 + rand(4)));
      const { log } = fightLines({ player: p("a"), line: line("a") }, { player: p("b"), line: line("b") }, { battleId: "x", seed, kind: "round", round: 5, runId: null, at: "2026-10-05T00:00:00Z", content, rules: MVP_RULES });
      const sides = new Map(log.flatMap((e) => (e.type === "BattleStart" ? [...e.teams.A, ...e.teams.B].map((u) => [u.id, true] as const) : e.type === "Summon" ? [[e.unit, true] as const] : [])));
      for (const b of beatPlayOf(log, stepsOf(log))) {
        for (const w of b.waves) {
          const e = log[w.eventIds[0]!]!;
          const c = causeOf(log, w);
          if (e.type === "BattleEnd") {
            expect(c).toBeNull();
            continue;
          }
          expect(c, `seed ${seed}: ${e.type} #${e.id}`).not.toBeNull();
          expect(c!.at === "clash" || sides.has(c!.at), `seed ${seed}: at ${c!.at}`).toBe(true);
          expect(c!.effect).toMatch(/^(effect|status|stat|battle):/);
          if (e.type === "Hurt" && e.source === "kernel" && log[e.causedBy!]?.type === "Strike") {
            const s = log[e.causedBy!] as Extract<BattleEvent, { type: "Strike" }>;
            expect(c).toMatchObject({ at: s.striker, kind: "strike", cause: "trigger:Strike", effect: "effect:damage" });
            seen.strike++;
          }
          if (e.source !== "kernel" && e.source.status === "Poison" && e.type === "Hurt") {
            expect(c).toMatchObject({ at: e.source.unit, kind: "status", cause: "status:Poison", effect: "effect:damage" });
            seen.poison++;
          }
          if (e.type === "Hurt" && e.source === "kernel" && log[e.causedBy!]?.type === "Fatigue") {
            expect(c).toMatchObject({ at: "clash", cause: "battle:fatigue" });
            seen.fatigue++;
          }
          if (e.source !== "kernel" && !e.source.status) {
            expect(c).toMatchObject({ at: e.source.unit, kind: "ability" });
            expect(c!.cause).toMatch(/^trigger:/);
            seen.ability++;
          }
          if (e.type === "Death") {
            // A death keeps the badge of the wave that killed.
            const killer = causeOf(log, { eventIds: [e.causedBy!], changes: [] });
            expect(c, `seed ${seed}: death #${e.id}`).toMatchObject({ at: killer!.at, cause: killer!.cause });
            seen.death++;
          }
        }
      }
    }
    for (const [k, v] of Object.entries(seen)) expect(v, k).toBeGreaterThan(0);
  });

  test("an ability's cause carries its When's scope and status: Smith answers an ally getting Shield", () => {
    const log = run([Shieldbearer, Smith, Archer], [dummy("Dummy", 30, 1)]);
    const name = displayNames(log);
    const whenOf = (defs: Record<string, UnitDef>) => (ref: { unit: string; when?: number }) => defs[ref.unit.split(":")[1]!]?.triggers?.[ref.when ?? -1];
    const smith = stepsOf(log).map((s) => causeOf(log, s, whenOf({ Smith }))).find((c) => c && name(c.at) === "Smith")!;
    expect(smith).toMatchObject({ kind: "ability", cause: "trigger:StatusApplied", causeStatus: "Shield", causeScope: "otherAlly", effect: "status:Strength" });
  });
});

describe("R2-17: key moments, battle start, fatigue rows, Why's icons", () => {
  test("R3-26: fatigue's kills are its own: the falls line says Fatigue, why I lost lists it", () => {
    const log = run([dummy("Wall", 60, 0), dummy("Wall2", 60, 0)], [dummy("Wall", 60, 0), dummy("Wall2", 60, 0)]);
    const deaths = log.filter((e) => e.type === "Death");
    expect(deaths.length).toBeGreaterThan(0);
    for (const d of deaths) expect(captionOf(log, d.id)).toMatch(/^Fatigue → .* falls$/);
    const rows = whyILost(log, "A");
    const f = rows.find((r) => r.text === "Fatigue")!;
    expect(f).toBeDefined();
    expect(f.kills).toBe(deaths.filter((d) => d.type === "Death" && d.unit.startsWith("A")).length);
    expect(f.damage).toBeGreaterThan(0);
    expect(log[f.sampleEventId]).toMatchObject({ type: "Hurt" });
    // Even past three bigger enemy chains, fatigue keeps a row once it killed.
    expect(whyILost(log, "A", 1).map((r) => r.text)).toEqual(["Fatigue"]);
  });

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

  test("the turning point: the death after which the winner led for good", () => {
    // 3 v 2: Squire's death evens it, Page's puts Dummy's side ahead for good.
    const log = run([dummy("Squire", 8, 1), dummy("Page", 6, 1), dummy("Knave", 5, 1)], [dummy("Dummy", 30, 2), Medic]);
    const beats = beatPlayOf(log, stepsOf(log));
    const ms = keyMomentsOf(log, beats);
    const turning = ms.find((m) => m.label.startsWith("Turning point"))!;
    expect(turning).toMatchObject({ kind: "kill", label: "Turning point: Dummy kills Page", unit: "B1:Dummy" });
    const page = log.find((e) => e.type === "Death" && e.unit === "A2:Page")!;
    expect(beats[turning.beat]!.waves.some((w) => w.eventIds.includes(page.id))).toBe(true);
    // The other kills keep their blow's size.
    expect(ms.map((m) => m.label)).toEqual(["Dummy kills Squire (−2)", "Turning point: Dummy kills Page", "Dummy kills Knave (−2)"]);
  });

  test("a side that led from the start and never lost the lead has no turning point", () => {
    const log = run([dummy("Squire", 8, 1), dummy("Page", 6, 1)], [dummy("Dummy", 30, 2)]);
    expect(log.at(-1)).toMatchObject({ type: "BattleEnd", winner: "B" });
    const ms = keyMomentsOf(log, beatPlayOf(log, stepsOf(log)));
    // B (one unit) trails 1 v 2 until Squire falls (even), then leads once Page falls: the last kill.
    expect(ms.filter((m) => m.label.startsWith("Turning point")).map((m) => m.label)).toEqual(["Turning point: Dummy kills Page"]);
    const draw = run([dummy("Wall", 60, 0), dummy("Wall2", 60, 0)], [dummy("Wall", 60, 0), dummy("Wall2", 60, 0)]);
    expect(keyMomentsOf(draw, beatPlayOf(draw, stepsOf(draw))).some((m) => m.label.startsWith("Turning point"))).toBe(false);
  });

  test("fatigue says what it did: who it killed, once, not a row per kill", () => {
    const log = run([dummy("Wall", 60, 0), dummy("Wall2", 60, 0)], [dummy("Wall", 60, 0), dummy("Wall2", 60, 0)]);
    const ms = keyMomentsOf(log, beatPlayOf(log, stepsOf(log)));
    const f = ms.find((m) => m.kind === "fatigue")!;
    // "+3" alone read as a heal (R3-26).
    expect(f.label).toMatch(/^Fatigue kills Wall2?, \+3 more fell$/);
    expect(ms.filter((m) => m.kind === "fatigue")).toHaveLength(1);
    expect(f.label).not.toMatch(/sets in \(T\d+\)/);
  });

  test("every key moment of real fights is one short line about that fight", () => {
    const pool = mvpPool();
    const content: MvpContent = { version: "t", units: pool.units, abilities: pool.abilities, statuses: pool.statuses };
    const p = (id: string): PlayerRef => ({ id, name: id, bot: false });
    let r = 777;
    const rand = (n: number) => ((r = (r * 1103515245 + 12345) % 2147483648), r % n);
    let joins = 0;
    for (let seed = 0; seed < 60; seed++) {
      const line = (s: string) => Array.from({ length: 5 }, (_, k) => lineUnitOf(pool.units[rand(pool.units.length)]!, `${s}${k}`, 1 + rand(4)));
      const { log } = fightLines({ player: p("a"), line: line("a") }, { player: p("b"), line: line("b") }, { battleId: "x", seed, kind: "round", round: 5, runId: null, at: "2026-10-05T00:00:00Z", content, rules: MVP_RULES });
      const ms = keyMomentsOf(log, beatPlayOf(log, stepsOf(log)));
      expect(ms.length, `seed ${seed}`).toBeGreaterThan(0);
      for (const m of ms) {
        expect(m.label.length, `seed ${seed}: ${m.label}`).toBeLessThanOrEqual(46);
        // The old generic forms are gone.
        expect(m.label).not.toMatch(/^Combo: \d+ steps|^Fatigue sets in \(T/);
        // R2-17 batch F: a combo says what it did, never only "Witot: 8 steps";
        // a unit joining is a summon, not a kill (no skull).
        expect(m.label, `seed ${seed}`).not.toMatch(/^[^:]+: \d+ steps$/);
        if (/ joins$/.test(m.label)) {
          expect(m.kind, `seed ${seed}: ${m.label}`).toBe("summon");
          joins++;
        }
      }
    }
    expect(joins).toBeGreaterThan(0);
  });

  test("a combo with no damage, healing or status says what it did (R2-17 batch F)", () => {
    // Fatigue's beat: fatigue did it; the unit that only reacted isn't its moment.
    const pool = mvpPool();
    const content: MvpContent = { version: "t", units: pool.units, abilities: pool.abilities, statuses: pool.statuses };
    const p = (id: string): PlayerRef => ({ id, name: id, bot: false });
    let r = 777;
    const rand = (n: number) => ((r = (r * 1103515245 + 12345) % 2147483648), r % n);
    let blocked = 0;
    // A Shield-only combo is rare and depends on the pool: past 300 fights,
    // keep looking until one shows up.
    for (let seed = 0; seed < 300 || (blocked === 0 && seed < 2000); seed++) {
      const line = (s: string) => Array.from({ length: 5 }, (_, k) => lineUnitOf(pool.units[rand(pool.units.length)]!, `${s}${k}`, 1 + rand(4)));
      const { log } = fightLines({ player: p("a"), line: line("a") }, { player: p("b"), line: line("b") }, { battleId: "x", seed, kind: "round", round: 5, runId: null, at: "2026-10-05T00:00:00Z", content, rules: MVP_RULES });
      const beats = beatPlayOf(log, stepsOf(log));
      for (const m of keyMomentsOf(log, beats)) {
        if (m.kind !== "combo") continue;
        const ids = beats[m.beat]!.waves.flatMap((w) => w.eventIds);
        if (/blocked by Shield/.test(m.label)) {
          blocked++;
          expect(ids.some((id) => log[id]?.type === "Hurt" && (log[id] as { absorbed?: number }).absorbed)).toBe(true);
        }
        // A fatigue beat with no unit dealing anything is fatigue's moment.
        if (ids.some((id) => log[id]?.type === "Fatigue")) expect(m.label, `seed ${seed}`).toMatch(/\d+ dmg|HP|kills/);
      }
    }
    expect(blocked).toBeGreaterThan(0);
  }, 30_000); // whole fights: 1.9 s idle, past 5 s on the CI runner (R4-19)

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

describe("R4-6: per-turn totals", () => {
  test("a battle-start cascade sums per unit: Shield gained, PWR gained, damage taken", () => {
    const log = run([Shieldbearer, Smith, Archer], [dummy("Dummy", 30, 2)]);
    const beats = beatPlayOf(log, stepsOf(log));
    const start = turnSummaryOf(log, beats)[0]!;
    expect(start.turn).toBe(0);
    const at = (name: string) => start.units.find((u) => displayNames(log)(u.unit) === name);
    expect(at("Shieldbearer")).toMatchObject({ side: "A", pwr: 1, damage: 0, died: false });
    expect(at("Shieldbearer")!.statuses).toEqual([{ status: "Shield", stacks: 2 }, { status: "Strength", stacks: 1 }]);
    expect(at("Dummy")).toMatchObject({ side: "B", damage: 2, died: false, statuses: [] });
    // Each unit's eventIds are its own changes, in log order.
    for (const u of start.units) {
      expect(u.eventIds).toEqual([...u.eventIds].sort((p, q) => p - q));
      for (const id of u.eventIds) expect((log[id] as { unit?: string }).unit).toBe(u.unit);
    }
  });

  test("a hit Shield takes counts as blocked, and the Shield it spent nets out", () => {
    const log = run([Shieldbearer], [dummy("Dummy", 30, 1)]);
    const beats = beatPlayOf(log, stepsOf(log));
    const hurt = log.find((e) => e.type === "Hurt" && e.absorbed && sidesOf(log).get(e.unit) === "A");
    expect(hurt).toBeDefined();
    const t = turnSummaryOf(log, beats).find((x) => x.turn === Math.max(0, hurt!.turn))!;
    const sb = t.units.find((u) => u.eventIds.includes(hurt!.id))!;
    const hurts = log.filter((e) => e.type === "Hurt" && e.unit === sb.unit && Math.max(0, e.turn) === t.turn) as Extract<BattleEvent, { type: "Hurt" }>[];
    expect(sb.blocked).toBe(hurts.reduce((n, e) => n + (e.absorbed ?? 0), 0));
    expect(sb.damage).toBe(hurts.reduce((n, e) => n + e.amount, 0));
    const shield = sb.statuses.find((x) => x.status === "Shield");
    expect(shield?.stacks ?? 0).toBeLessThanOrEqual(0);
  });

  test("real fights: every turn's totals equal the log's own sums, each death once, no empty unit", () => {
    const pool = mvpPool();
    const content: MvpContent = { version: "t", units: pool.units, abilities: pool.abilities, statuses: pool.statuses };
    const p = (id: string): PlayerRef => ({ id, name: id, bot: false });
    let r = 4242;
    const rand = (n: number) => ((r = (r * 1103515245 + 12345) % 2147483648), r % n);
    for (let seed = 0; seed < 40; seed++) {
      const line = (s: string) => Array.from({ length: 5 }, (_, k) => lineUnitOf(pool.units[rand(pool.units.length)]!, `${s}${k}`, 1 + rand(4)));
      const { log } = fightLines({ player: p("a"), line: line("a") }, { player: p("b"), line: line("b") }, { battleId: "x", seed, kind: "round", round: 5, runId: null, at: "2026-10-05T00:00:00Z", content, rules: MVP_RULES });
      const beats = beatPlayOf(log, stepsOf(log));
      const sum = turnSummaryOf(log, beats);
      // The turns are the timeline's, beat for beat.
      expect(sum.map((t) => [t.turn, t.beats]), `seed ${seed}`).toEqual(timelineOf(log, beats).map((t) => [t.turn, t.beats]));
      // Expected: the log's events, grouped by the turn of the beat that reveals them.
      const turnOf = new Map<number, number>();
      for (const b of beats) for (let id = b.start; id <= b.end; id++) turnOf.set(id, Math.max(0, b.turn));
      const want = new Map<string, { damage: number; healed: number; pwr: number; hp: number; blocked: number; died: boolean }>();
      const key = (turn: number, unit: string) => `${turn}|${unit}`;
      for (const e of log) {
        const turn = turnOf.get(e.id);
        if (turn === undefined || !("unit" in e) || !["Hurt", "Heal", "StatChanged", "Death"].includes(e.type)) continue;
        const w = want.get(key(turn, e.unit)) ?? { damage: 0, healed: 0, pwr: 0, hp: 0, blocked: 0, died: false };
        if (e.type === "Hurt") { w.damage += e.amount; w.blocked += e.absorbed ?? 0; }
        else if (e.type === "Heal") w.healed += e.amount;
        else if (e.type === "StatChanged") w[e.stat] += e.delta;
        else if (e.type === "Death") w.died = true;
        want.set(key(turn, e.unit), w);
      }
      for (const t of sum) {
        for (const u of t.units) {
          const w = want.get(key(t.turn, u.unit)) ?? { damage: 0, healed: 0, pwr: 0, hp: 0, blocked: 0, died: false };
          expect({ damage: u.damage, healed: u.healed, pwr: u.pwr, hp: u.hp, blocked: u.blocked, died: u.died }, `seed ${seed} T${t.turn} ${u.unit}`).toEqual(w);
          expect(u.damage || u.healed || u.pwr || u.hp || u.blocked || u.statuses.length || u.died).toBeTruthy();
          for (const st of u.statuses) expect(st.stacks).not.toBe(0);
        }
      }
      // Nothing changed is left out, and every death shows once.
      for (const [k, w] of want) if (w.damage || w.healed || w.pwr || w.hp || w.blocked || w.died) expect(sum.some((t) => t.units.some((u) => key(t.turn, u.unit) === k)), `seed ${seed} ${k}`).toBe(true);
      // A unit revived and killed again in one turn shows one ✝ for that turn.
      const deaths = new Set(log.filter((e) => e.type === "Death").map((e) => key(turnOf.get(e.id) ?? Math.max(0, e.turn), (e as { unit: string }).unit)));
      expect(sum.flatMap((t) => t.units.filter((u) => u.died)).length, `seed ${seed}`).toBe(deaths.size);
    }
  });
});

describe("R4-12: turn-end summary", () => {
  const tot = (o: Partial<UnitTurnTotals>): UnitTurnTotals => ({ unit: "u", side: "A", damage: 0, healed: 0, pwr: 0, hp: 0, blocked: 0, statuses: [], died: false, eventIds: [], ...o });

  test("the hold: 1.2 s at 1×, divided by speed, none at 4×", () => {
    expect(TURN_END_MS).toBe(1200);
    expect(turnEndHoldMs(1)).toBe(1200);
    expect(turnEndHoldMs(2)).toBe(600);
    expect(turnEndHoldMs(4)).toBe(0);
  });

  test("it holds after each turn's last beat that changed a unit, never after the battle's last beat", () => {
    const log = run([Shieldbearer, Smith, Archer], [dummy("Dummy", 30, 2)]);
    const beats = beatPlayOf(log, stepsOf(log));
    const turns = turnSummaryOf(log, beats);
    const ends = turnEndsOf(turns, beats.length);
    expect(ends.size).toBeGreaterThan(1);
    for (const [beat, t] of ends) {
      expect(t.beats.at(-1)).toBe(beat);
      expect(t.units.length).toBeGreaterThan(0);
      expect(beat).toBeLessThan(beats.length - 1);
    }
    // Every turn but the one the battle ends in (and those that changed no one).
    expect(ends.size).toBe(turns.filter((t) => t.units.length && t.beats.at(-1)! < beats.length - 1).length);
    expect([...ends.keys()]).toEqual([...ends.keys()].sort((p, q) => p - q));
  });

  test("a unit's label: damage, healing, PWR/HP, blocked, then statuses; spent Shield not repeated", () => {
    expect(totalsPartsOf(tot({ damage: 7, healed: 2, statuses: [{ status: "Shield", stacks: 3 }] }))).toEqual([
      { kind: "damage", text: "−7" },
      { kind: "heal", text: "+2" },
      { kind: "status", text: "×3", status: "Shield", stacks: 3 },
    ]);
    expect(totalsText(tot({ damage: 7, healed: 2, statuses: [{ status: "Shield", stacks: 3 }] }))).toBe("−7 · +2 · Shield ×3");
    expect(totalsPartsOf(tot({ pwr: 1, hp: 2 }))).toEqual([{ kind: "buff", text: "+1/+2" }]);
    expect(totalsPartsOf(tot({ pwr: -2 }))).toEqual([{ kind: "debuff", text: "−2 PWR" }]);
    expect(totalsPartsOf(tot({ blocked: 3, statuses: [{ status: "Shield", stacks: -3 }, { status: "Poison", stacks: -1 }] }))).toEqual([
      { kind: "blocked", text: "3" },
      { kind: "status", text: "−1", status: "Poison", stacks: -1 },
    ]);
    expect(totalsText(tot({ damage: 5, died: true }))).toBe("✝ · −5");
    expect(totalsText(tot({ blocked: 2 }))).toBe("2 blocked");
  });
});

describe("R3-21: target beams", () => {
  const Rally = unit("Rally", 6, 1, { on: "BattleStart" }, [{ kind: "allAllies" }], ["GiveStrength"]);
  const beamsIn = (log: BattleEvent[]) => beatPlayOf(log, stepsOf(log)).flatMap((b) => b.waves.map((w) => ({ w, beams: beamsOf(log, w) })));

  test("a strike draws one beam, front to front, in damage", () => {
    const log = run([Shieldbearer], [dummy("Dummy", 30, 1)]);
    const hit = beamsIn(log).find(({ w }) => log[w.eventIds[0]!]!.type === "Hurt")!;
    const strike = log[log[hit.w.eventIds[0]!]!.causedBy!] as Extract<BattleEvent, { type: "Strike" }>;
    expect(hit.beams).toEqual([{ from: strike.striker, to: strike.defender, kind: "damage" }]);
  });

  test("an all-allies buff is a fan from its source to each ally", () => {
    const log = run([Rally, Smith, Archer], [dummy("Dummy", 30, 1)]);
    const fan = beamsIn(log).find(({ beams }) => beams.length > 1)!.beams;
    expect(fan.map((b) => b.to).sort()).toEqual(["A1:Rally", "A2:Smith", "A3:Archer"]);
    expect(new Set(fan.map((b) => b.from))).toEqual(new Set(["A1:Rally"]));
    expect(fan.every((b) => b.kind === "status" && b.status === "Strength")).toBe(true);
  });

  test("a self-target is a beam to its own source (a ring), not a line", () => {
    const log = run([Shieldbearer], [dummy("Dummy", 30, 1)]);
    const own = beamsIn(log).find(({ w }) => log[w.eventIds[0]!]!.type === "StatusApplied")!;
    expect(own.beams).toEqual([{ from: "A1:Shieldbearer", to: "A1:Shieldbearer", kind: "status", status: "Shield" }]);
  });

  test("fatigue beams start at the clash", () => {
    const log = run([dummy("Rock", 400, 0)], [dummy("Stone", 400, 0)]);
    const tired = beamsIn(log).filter(({ w }) => log[log[w.eventIds[0]!]!.causedBy!]?.type === "Fatigue");
    expect(tired.length).toBeGreaterThan(0);
    for (const { beams } of tired) for (const b of beams) expect(b).toMatchObject({ from: "clash", kind: "damage" });
  });

  test("a death draws no beam; a unit killed through its Shield reads as the striker's, never its own Shield", () => {
    // Brute's 9 takes Bearer's Shield 2 and kills it: Death ← Shield gone ← the hit.
    const Bearer = unit("Bearer", 4, 1, { on: "BattleStart" }, [{ kind: "holder" }], ["GiveShield"]);
    const log = run([Bearer], [dummy("Brute", 30, 9)]);
    const death = log.find((e) => e.type === "Death" && e.unit === "A1:Bearer")!;
    expect(log[death.causedBy!]).toMatchObject({ type: "StatusRemoved", status: "Shield" });
    expect(causeOf(log, { eventIds: [death.id], changes: [] })).toMatchObject({ at: "B1:Brute", kind: "strike", cause: "trigger:Strike" });
    const gone = log.find((e) => e.type === "StatusRemoved" && e.unit === "A1:Bearer")!;
    expect(causeOf(log, { eventIds: [gone.id], changes: [] })).toMatchObject({ at: "B1:Brute", effect: "status:Shield" });
    expect(beamsOf(log, { eventIds: [death.id], changes: [changeOf(death)!] })).toEqual([]);
  });
});

describe("R4-22: running totals above units", () => {
  test("real fights: over the battle the rows sum to the turn totals; each event is one beat's; a death stays its showing beat's turn", () => {
    const pool = mvpPool();
    const content: MvpContent = { version: "t", units: pool.units, abilities: pool.abilities, statuses: pool.statuses };
    const p = (id: string): PlayerRef => ({ id, name: id, bot: false });
    let r = 99;
    const rand = (n: number) => ((r = (r * 1103515245 + 12345) % 2147483648), r % n);
    for (let seed = 0; seed < 30; seed++) {
      const line = (s: string) => Array.from({ length: 5 }, (_, k) => lineUnitOf(pool.units[rand(pool.units.length)]!, `${s}${k}`, 1 + rand(4)));
      const { log } = fightLines({ player: p("a"), line: line("a") }, { player: p("b"), line: line("b") }, { battleId: "x", seed, kind: "round", round: 5, runId: null, at: "2026-10-05T00:00:00Z", content, rules: MVP_RULES });
      const beats = beatPlayOf(log, stepsOf(log));
      const own = beatIdsOf(beats);
      const all = own.flat();
      expect(new Set(all).size).toBe(all.length);
      const turns = turnSummaryOf(log, beats);
      const turnOf = new Map(turns.flatMap((t) => t.beats.map((i) => [i, t.turn] as const)));
      // A death a wave shows belongs to that wave's beat, so its turn.
      for (const b of beats) for (const w of b.waves) for (const id of w.eventIds) expect(own[b.index]).toContain(id);
      const want = new Map<string, number>(), got = new Map<string, number>();
      const bump = (m: Map<string, number>, k: string, n: number) => m.set(k, (m.get(k) ?? 0) + n);
      for (const t of turns) {
        for (const u of t.units) for (const [k, n] of [["damage", u.damage], ["heal", u.healed], ["pwr", u.pwr], ["hp", u.hp], ["blocked", u.blocked]] as const) bump(want, `${u.unit}/${k}`, n);
        for (const [unit, rs] of runningTotalsOf(log, turnSoFarIds(own, t, beats.length, () => true))) for (const x of rs) if (x.kind !== "status") bump(got, `${unit}/${x.key}`, x.value);
        // Every beat of the turn is the turn's.
        for (const i of t.beats) expect(turnOf.get(i)).toBe(t.turn);
      }
      for (const [k, n] of want) expect(got.get(k) ?? 0, k).toBe(n);
    }
  });

  test("beat by beat, wave by wave, a turn's landed events only grow", () => {
    const log = run([Shieldbearer, Smith, Archer], [dummy("Dummy", 30, 2)]);
    const beats = beatPlayOf(log, stepsOf(log));
    const own = beatIdsOf(beats);
    for (const t of turnSummaryOf(log, beats)) {
      let prev: number[] = [];
      for (const at of t.beats) {
        const b = beats[at]!;
        for (let wave = 0; wave < b.waves.length; wave++) {
          const shown = new Set(b.waves.slice(0, wave + 1).flatMap((w) => w.eventIds));
          const ids = turnSoFarIds(own, t, at, (id) => shown.has(id));
          expect(ids).toEqual(expect.arrayContaining(prev));
          prev = ids;
        }
      }
    }
  });

  test("rows keep the order their kinds first landed, beat after beat", () => {
    const log = run([Shieldbearer, Smith, Archer], [dummy("Dummy", 30, 2)]);
    const beats = beatPlayOf(log, stepsOf(log));
    for (const t of turnSummaryOf(log, beats)) {
      const seen = new Map<string, string[]>();
      for (const at of t.beats) {
        const b = beats[at]!;
        void b;
        const ids = turnSoFarIds(beatIdsOf(beats), t, at, () => true);
        for (const [unit, rs] of runningTotalsOf(log, ids)) {
          const keys = rs.map((x) => x.key);
          const was = seen.get(unit) ?? [];
          expect(keys.slice(0, was.length)).toEqual(was);
          seen.set(unit, keys);
        }
      }
    }
  });

  test("a row reads as its kind", () => {
    expect(runRowText("damage", "damage", 5)).toBe("−5");
    expect(runRowText("heal", "heal", 2)).toBe("+2");
    expect(runRowText("buff", "pwr", 1)).toBe("+1 PWR");
    expect(runRowText("debuff", "hp", -2)).toBe("−2 HP");
    expect(runRowText("status", "status:Poison", 3)).toBe("×3");
    expect(runRowText("status", "status:Shield", -1)).toBe("−1");
  });
});

// M4-3: the same captions in Russian; English stays as it was once reset.
describe("captions in Russian", () => {
  test("steps, changes and traces read in Russian, then English again", () => {
    const log = run([Shieldbearer, Smith, Archer], [dummy("Dummy", 20, 1)]);
    const shot = log.find((e) => e.type === "Hurt" && e.source !== "kernel")!;
    const english = stepsOf(log).map((s) => s.caption);
    setTraceLang("ru");
    try {
      const steps = stepsOf(log);
      expect(steps.find((s) => s.caption.includes("Сила"))!.caption).toBe("Smith → Сила ×1 на Shieldbearer (+1 АТК)");
      expect(steps.find((s) => s.caption.includes("Сила"))!.changes.map((c) => c.label)).toEqual(["Сила ×1", "+1 АТК"]);
      expect(steps.find((s) => s.caption.includes("бьёт"))!.caption).toMatch(/^\w+ бьёт \w+ → −\d+$/);
      expect(steps.at(-1)!.caption).toMatch(/Сторона [AB] побеждает|Ничья/);
      expect(stepsOf(log, undefined, undefined, { you: "A" }).at(-1)!.caption).toMatch(/Вы победили|Они победили|Ничья/);
      for (const s of steps) expect(s.caption, s.caption).not.toMatch(/\b(strikes|on|wins|absorbed|blocks|falls|appears|returns|Shield|Strength|PWR|HP)\b/);
      expect(chainOf(log, shot.id).nodes.at(-1)!.text).toMatch(/^(Начало боя|Ход \d+|Конец хода \d+)$/);
      expect(endCaption("draw")).toBe("Ничья");
    } finally {
      setTraceLang(undefined);
    }
    expect(stepsOf(log).map((s) => s.caption)).toEqual(english);
  });
});
