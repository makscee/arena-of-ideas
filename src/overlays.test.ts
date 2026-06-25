// Hero-overlay accumulation tests (#065 slice 2) — overlaysAt is the pure
// projection the typed badges draw from. Two layers of coverage:
//
//  1. A PROPERTY SWEEP over real battles (many seeds × reference matchups): at
//     the START, MIDDLE and END of every beat, the overlay map must equal an
//     INDEPENDENT re-derivation — a from-scratch fold over the open beat's
//     caused events up to that step. The projection never out-knows the log.
//     This is the slice-1 beat-test style (sweep many real battles, re-derive
//     the answer a second way) applied to overlays.
//
//  2. TARGETED unit tests on hand-built logs for the exact decisions the shape
//     encodes: damage and heal summed but kept SEPARATE (the absorb/heal-back
//     stays visible), net status/stat deltas, death, multi-hit summing on one
//     unit within a beat, and a clean reset at the next beat.

import { describe, expect, test } from "vitest";
import { battle } from "./battle.js";
import { badgeValues, beatsOf, newBadgeKeysAt, overlaysAt, type BeatOverlay } from "./beats.js";
import type { BattleEvent, BattleInput, EventBody, UnitDef } from "./types.js";
import { Summoner, Venomancer, Silencer, Necromancer, stressRegistry } from "./content/stress.js";

const tank = (name: string, hp: number, pwr: number): UnitDef => ({ name, base: { hp, pwr } });

// A spread of real battles that, between them, cover every overlay-bearing
// event: strikes (Hurt), turn-end Poison (more Hurts, summed per unit across
// ticks), Venomancer poison application (StatusApplied), Poison consumeStacks
// (StatusRemoved), Blessing/preventDeathHeal (Heal), deaths, Silencer's silence,
// and fatigue (multi-unit Hurt in one beat). Several seeds so the sweep is wide.
// A unit carrying initial statMod statuses: Strength/Vitality apply at
// BattleStart (StatusApplied) and shift effective stats (StatChanged).
const Mosaic: UnitDef = {
  name: "Mosaic",
  base: { hp: 9, pwr: 2 },
  statuses: [
    { status: "Strength", stacks: 2 },
    { status: "Vitality", stacks: 1 },
  ],
};
// Blessing's preventDeathHeal turns a lethal Hurt into a Heal (a heal-back the
// overlay must keep SEPARATE from the damage that triggered it).
const Saint: UnitDef = { name: "Saint", base: { hp: 4, pwr: 1 }, statuses: [{ status: "Blessing", stacks: 3 }] };

const matchups: { name: string; input: BattleInput }[] = [];
for (const seed of [1, 3, 7, 11, 23, 42, 99]) {
  matchups.push({
    name: `venom/summon/necro vs bricks seed ${seed}`,
    input: {
      teamA: [Venomancer, Summoner, Necromancer, tank("Wall", 24, 1)],
      teamB: [Silencer, tank("Brick", 22, 2), tank("Slab", 20, 1)],
      seed,
      statuses: stressRegistry,
    },
  });
}
// Venom in front (no silencer) so its poison lands → StatusApplied + the
// turn-end consumeStacks → StatusRemoved; Mosaic's statMod statuses → StatChanged.
for (const seed of [3, 7, 11]) {
  matchups.push({
    name: `venom front + statmods vs Mosaic seed ${seed}`,
    input: {
      teamA: [Venomancer, tank("Wall", 20, 1)],
      teamB: [Mosaic, tank("Brick", 18, 2)],
      seed,
      statuses: stressRegistry,
    },
  });
}
// Blessing on a fragile unit → a preventDeathHeal Heal event.
for (const seed of [3, 7, 11]) {
  matchups.push({
    name: `blessing heal-back vs brutes seed ${seed}`,
    input: {
      teamA: [Saint, tank("Wall", 16, 1)],
      teamB: [tank("Brute", 14, 5), tank("Brick", 12, 2)],
      seed,
      statuses: stressRegistry,
    },
  });
}

/** Independent re-derivation: fold the open beat's caused events up to `step`
 * a second way (no shared helper with the implementation's accumulator). */
function expectedOverlays(log: BattleEvent[], step: number): Map<string, BeatOverlay> {
  const beats = beatsOf(log);
  const beat = beats.find((b) => step >= b.start && step <= b.end);
  const out = new Map<string, BeatOverlay>();
  if (!beat) return out;
  const get = (u: string): BeatOverlay => {
    if (!out.has(u)) out.set(u, { damage: 0, heal: 0, statusDeltas: {}, statChanges: {}, died: false });
    return out.get(u)!;
  };
  for (const e of beat.caused) {
    if (e.id > step) continue;
    if (e.type === "Hurt") get(e.unit).damage += e.amount;
    else if (e.type === "Heal") get(e.unit).heal += e.amount;
    else if (e.type === "StatusApplied") {
      const o = get(e.unit);
      o.statusDeltas[e.status] = (o.statusDeltas[e.status] ?? 0) + e.stacks;
    } else if (e.type === "StatusRemoved") {
      const o = get(e.unit);
      o.statusDeltas[e.status] = (o.statusDeltas[e.status] ?? 0) - e.stacks;
    } else if (e.type === "StatChanged") {
      const o = get(e.unit);
      const k = e.stat === "hp" ? "maxHp" : e.stat;
      o.statChanges[k] = (o.statChanges[k] ?? 0) + e.delta;
    } else if (e.type === "Death") get(e.unit).died = true;
  }
  return out;
}

/** Sort-stable serialization so two overlay maps compare structurally. */
function norm(m: Map<string, BeatOverlay>): string {
  return JSON.stringify(
    [...m.entries()].sort(([a], [b]) => a.localeCompare(b)).map(([u, o]) => [
      u,
      o.damage,
      o.heal,
      Object.entries(o.statusDeltas).sort(),
      Object.entries(o.statChanges).sort(),
      o.died,
    ]),
  );
}

describe("overlaysAt matches an independent re-derivation at start/middle/end of every beat", () => {
  for (const { name, input } of matchups) {
    test(name, () => {
      const log = battle(input);
      const beats = beatsOf(log);
      expect(beats.length).toBeGreaterThan(3);
      for (const beat of beats) {
        // START, MIDDLE, END of the beat (and every step, for good measure on
        // the shorter beats — the sweep is cheap).
        const mid = Math.floor((beat.start + beat.end) / 2);
        for (const step of new Set([beat.start, mid, beat.end])) {
          expect(norm(overlaysAt(log, step)), `${name} beat ${beat.index} step ${step}`).toBe(
            norm(expectedOverlays(log, step)),
          );
        }
      }
      // And a full every-step sweep on the whole log — partial reveals included.
      for (let step = 0; step < log.length; step++) {
        expect(norm(overlaysAt(log, step)), `${name} step ${step}`).toBe(norm(expectedOverlays(log, step)));
      }
    });
  }

  test("the matchups actually exercise damage, heal, status, stat, and death overlays", () => {
    const seen = { damage: false, heal: false, status: false, stat: false, died: false };
    for (const { input } of matchups) {
      const log = battle(input);
      for (let step = 0; step < log.length; step++) {
        for (const o of overlaysAt(log, step).values()) {
          if (o.damage > 0) seen.damage = true;
          if (o.heal > 0) seen.heal = true;
          if (Object.values(o.statusDeltas).some((v) => v !== 0)) seen.status = true;
          if (Object.values(o.statChanges).some((v) => v !== 0)) seen.stat = true;
          if (o.died) seen.died = true;
        }
      }
    }
    expect(seen.damage, "some beat overlays a damage total").toBe(true);
    expect(seen.heal, "some beat overlays a heal total (Blessing heal-back)").toBe(true);
    expect(seen.status, "some beat overlays a status delta (poison/statmods)").toBe(true);
    expect(seen.stat, "some beat overlays a stat change (statmod buffs)").toBe(true);
    expect(seen.died, "some beat overlays a death").toBe(true);
  });
});

// ---------------------------------------------------------------------------
// Targeted hand-built logs — the exact decisions the shape encodes. These need
// no kernel: overlaysAt is pure over the event log, so a minimal log isolates
// one behaviour. Each log opens with a Strike root (a hero-affecting beat) then
// the caused events under test.
// ---------------------------------------------------------------------------

let nextId = 0;
function ev(body: EventBody, causedBy: number | null): BattleEvent {
  return { id: nextId++, turn: 1, causedBy, source: "kernel", ...body } as BattleEvent;
}

/** Build a one-beat log: a Strike root followed by `caused` bodies, each caused
 * by the root. Returns the log; ids are contiguous from 0. */
function strikeBeat(caused: EventBody[]): BattleEvent[] {
  nextId = 0;
  const root = ev({ type: "Strike", striker: "A1", defender: "B1" }, null);
  const events = caused.map((b) => ev(b, root.id));
  return [root, ...events];
}

describe("the typed-overlay decisions", () => {
  test("two Hurts on one unit within a beat SUM into one damage total", () => {
    const log = strikeBeat([
      { type: "Hurt", unit: "B1", amount: 3, hpAfter: 7 },
      { type: "Hurt", unit: "B1", amount: 4, hpAfter: 3 },
    ]);
    const o = overlaysAt(log, log.length - 1).get("B1")!;
    expect(o.damage).toBe(7); // 3 + 4 summed, not max, not last
    expect(o.heal).toBe(0);
  });

  test("damage and heal are kept SEPARATE — an absorb/heal-back nets to neither", () => {
    const log = strikeBeat([
      { type: "Hurt", unit: "B1", amount: 5, hpAfter: 5 },
      { type: "Heal", unit: "B1", amount: 5, hpAfter: 10 },
    ]);
    const o = overlaysAt(log, log.length - 1).get("B1")!;
    expect(o.damage).toBe(5); // both stay visible…
    expect(o.heal).toBe(5); // …never netted to 0
  });

  test("status deltas net per name (apply + remove on the same status)", () => {
    const log = strikeBeat([
      { type: "StatusApplied", unit: "B1", status: "Poison", stacks: 2, total: 2 },
      { type: "StatusApplied", unit: "B1", status: "Poison", stacks: 1, total: 3 },
      { type: "StatusRemoved", unit: "B1", status: "Poison", stacks: 1, remaining: 2 },
    ]);
    const o = overlaysAt(log, log.length - 1).get("B1")!;
    expect(o.statusDeltas["Poison"]).toBe(2); // +2 +1 -1
  });

  test("stat changes net per stat; an hp StatChanged reports as maxHp (not the live bar)", () => {
    const log = strikeBeat([
      { type: "StatChanged", unit: "B1", stat: "pwr", delta: 2, now: 4 },
      { type: "StatChanged", unit: "B1", stat: "pwr", delta: -1, now: 3 },
      { type: "StatChanged", unit: "B1", stat: "hp", delta: 3, now: 12, hpAfter: 12 },
    ]);
    const o = overlaysAt(log, log.length - 1).get("B1")!;
    expect(o.statChanges["pwr"]).toBe(1); // +2 -1
    expect(o.statChanges["maxHp"]).toBe(3); // hp delta keyed as maxHp
  });

  test("a Death sets died", () => {
    const log = strikeBeat([
      { type: "Hurt", unit: "B1", amount: 9, hpAfter: 0 },
      { type: "Death", unit: "B1" },
    ]);
    const o = overlaysAt(log, log.length - 1).get("B1")!;
    expect(o.died).toBe(true);
    expect(o.damage).toBe(9);
  });

  test("accumulation is partial mid-beat: only caused events with id ≤ step count", () => {
    const log = strikeBeat([
      { type: "Hurt", unit: "B1", amount: 3, hpAfter: 7 }, // id 1
      { type: "Hurt", unit: "B1", amount: 4, hpAfter: 3 }, // id 2
    ]);
    // At the first Hurt only (step = id 1) the total is 3; at id 2 it is 7.
    expect(overlaysAt(log, 1).get("B1")!.damage).toBe(3);
    expect(overlaysAt(log, 2).get("B1")!.damage).toBe(7);
    // At the root step (id 0) nothing is revealed yet → empty map.
    expect(overlaysAt(log, 0).size).toBe(0);
  });

  test("overlays RESET at the next beat — the window restarts from the new beat's start", () => {
    nextId = 0;
    const b1Root = ev({ type: "Strike", striker: "A1", defender: "B1" }, null); // id 0
    const b1Hurt = ev({ type: "Hurt", unit: "B1", amount: 6, hpAfter: 4 }, b1Root.id); // id 1
    const b2Root = ev({ type: "Strike", striker: "B1", defender: "A1" }, null); // id 2 — new beat
    const b2Hurt = ev({ type: "Hurt", unit: "A1", amount: 2, hpAfter: 8 }, b2Root.id); // id 3
    const log = [b1Root, b1Hurt, b2Root, b2Hurt];

    // End of beat 1: B1 carries 6 damage, A1 absent.
    const end1 = overlaysAt(log, 1);
    expect(end1.get("B1")!.damage).toBe(6);
    expect(end1.has("A1")).toBe(false);

    // Into beat 2: B1's 6 is GONE (new beat, fresh window), only A1's 2 shows.
    const in2 = overlaysAt(log, 3);
    expect(in2.has("B1"), "beat-1 damage cleared at the next beat").toBe(false);
    expect(in2.get("A1")!.damage).toBe(2);
  });

  test("an empty/out-of-range step yields an empty map", () => {
    expect(overlaysAt([], 0).size).toBe(0);
    const log = strikeBeat([{ type: "Hurt", unit: "B1", amount: 1, hpAfter: 1 }]);
    expect(overlaysAt(log, -1).size).toBe(0);
    expect(overlaysAt(log, 999).size).toBe(0);
  });
});

// ---------------------------------------------------------------------------
// newBadgeKeysAt (#065 item 1) — the diff that scopes the badge reveal to the
// ONE badge that just appeared/changed, so prior badges don't re-animate when a
// new badge lands. Keys mirror badgeValues (dmg/heal/stat:<n>/status:<n>).
// ---------------------------------------------------------------------------

describe("newBadgeKeysAt scopes the reveal to the just-appeared/changed badge", () => {
  test("badgeValues emits only the non-zero badges, keyed as the renderer draws them", () => {
    const o: BeatOverlay = {
      damage: 4,
      heal: 0, // netted out → no badge, no key
      statusDeltas: { Poison: 2, Shield: 0 }, // Shield 0 → dropped
      statChanges: { pwr: -1, maxHp: 0 }, // maxHp 0 → dropped
      died: false,
    };
    const v = badgeValues(o);
    expect([...v.keys()].sort()).toEqual(["dmg", "stat:pwr", "status:Poison"]);
    expect(v.get("dmg")).toBe(4);
    expect(v.get("status:Poison")).toBe(2);
    expect(v.get("stat:pwr")).toBe(-1);
  });

  test("at a beat's first revealed badge step, that badge is new", () => {
    const log = strikeBeat([{ type: "Hurt", unit: "B1", amount: 3, hpAfter: 7 }]); // id 1
    const fresh = newBadgeKeysAt(log, 1);
    expect([...(fresh.get("B1") ?? [])]).toEqual(["dmg"]);
  });

  test("when a SECOND distinct badge lands, ONLY it is new — the prior badge holds steady", () => {
    // id1 Hurt (dmg badge), id2 StatusApplied (status badge). At id2 only the
    // status badge just appeared; the damage badge is unchanged → NOT new. This
    // is the defect-1 fix: the already-shown badge must not re-animate.
    const log = strikeBeat([
      { type: "Hurt", unit: "B1", amount: 3, hpAfter: 7 }, // id 1
      { type: "StatusApplied", unit: "B1", status: "Poison", stacks: 2, total: 2 }, // id 2
    ]);
    const fresh = newBadgeKeysAt(log, 2);
    const keys = fresh.get("B1") ?? new Set();
    expect(keys.has("status:Poison")).toBe(true); // the new one animates
    expect(keys.has("dmg")).toBe(false); // the prior one does NOT re-animate
  });

  test("a badge whose VALUE just changed (another hit) counts as new; an unchanged one does not", () => {
    // Two Hurts on B1 and one on a bystander A1 cap (just to have a steady badge).
    const log = strikeBeat([
      { type: "Hurt", unit: "B1", amount: 3, hpAfter: 7 }, // id 1: B1 dmg 3
      { type: "Hurt", unit: "A1", amount: 2, hpAfter: 8 }, // id 2: A1 dmg 2 (B1 steady)
      { type: "Hurt", unit: "B1", amount: 4, hpAfter: 3 }, // id 3: B1 dmg → 7 (changed)
    ]);
    // At id 3: B1's damage moved 3→7 (new), A1's 2 is unchanged (not new).
    const fresh = newBadgeKeysAt(log, 3);
    expect([...(fresh.get("B1") ?? [])]).toEqual(["dmg"]);
    expect(fresh.has("A1")).toBe(false);
  });

  test("the root step (nothing revealed) has no new badges", () => {
    const log = strikeBeat([{ type: "Hurt", unit: "B1", amount: 3, hpAfter: 7 }]);
    expect(newBadgeKeysAt(log, 0).size).toBe(0);
  });
});
