// The battle Log's rows (round 4, R4-3): a beat's waves grouped so long
// fights stay readable, every merged event still reachable from Why.

import { describe, expect, test } from "vitest";
import { battle } from "../battle.js";
import type { AbilityRef, BattleEvent, EventBody } from "../types.js";
import { MVP_RULES } from "./contract.js";
import { toBattleDef } from "./fight.js";
import { fuseUnits, lineUnitOf } from "./forms.js";
import { beatPlayOf, logRowsOf, stepsOf } from "./trace.js";
import { mvpPool } from "./units.js";

/** A hand-built log: one turn end whose cascade has every case. */
function turnEndLog(): BattleEvent[] {
  const log: BattleEvent[] = [];
  const add = (causedBy: number | null, source: "kernel" | AbilityRef, body: EventBody) => {
    log.push({ id: log.length, turn: 1, causedBy, source, ...body } as BattleEvent);
    return log.length - 1;
  };
  const by = (unit: string, ability = 0, status?: string): AbilityRef => ({ unit, ability, ...(status ? { status } : {}) });
  const roster = (side: string, names: string[]) => names.map((n, i) => ({ id: `${side}${i + 1}:${n}`, name: n, hp: 9, pwr: 1 }));
  add(null, "kernel", { type: "BattleStart", teams: { A: roster("A", ["Nurse", "Rose", "Ace"]), B: roster("B", ["Ogre", "Imp"]) } });
  add(null, "kernel", { type: "TurnStart" });
  const end = add(null, "kernel", { type: "TurnEnd" });
  // R2: Poison ticks three units; one fade, one tick Shield takes whole.
  add(end, by("B1:Ogre", 0, "Poison"), { type: "Hurt", unit: "B1:Ogre", amount: 4, hpAfter: 5 });
  add(end, by("B1:Ogre", 0, "Poison"), { type: "StatusRemoved", unit: "B1:Ogre", status: "Poison", stacks: 1, remaining: 0 });
  const blocked = add(end, by("B2:Imp", 0, "Poison"), { type: "Hurt", unit: "B2:Imp", amount: 0, absorbed: 4, hpAfter: 9 });
  add(blocked, by("B2:Imp", 0, "Shield"), { type: "StatusRemoved", unit: "B2:Imp", status: "Shield", stacks: 4, remaining: 0 });
  const rose = add(end, by("A2:Rose", 0, "Poison"), { type: "Hurt", unit: "A2:Rose", amount: 4, hpAfter: 5 });
  // R1 + R4: Nurse shields every ally after each hit and heal; between, a heal.
  for (const u of ["A1:Nurse", "A2:Rose", "A3:Ace"]) add(rose, by("A1:Nurse"), { type: "StatusApplied", unit: u, status: "Shield", stacks: 1, total: 1 });
  const heal = add(rose, by("A1:Nurse", 1), { type: "Heal", unit: "A2:Rose", amount: 1, hpAfter: 6 });
  for (const u of ["A1:Nurse", "A2:Rose", "A3:Ace"]) add(heal, by("A1:Nurse"), { type: "StatusApplied", unit: u, status: "Shield", stacks: 1, total: 2 });
  // R3: Ace hits both enemies; Blessing saves the Imp between the two hits.
  const hit = add(end, by("A3:Ace"), { type: "Hurt", unit: "B2:Imp", amount: 12, hpAfter: -3 });
  const save = add(hit, by("B2:Imp", 0, "Blessing"), { type: "Intercepted", by: by("B2:Imp", 0, "Blessing"), original: "Death", unit: "B2:Imp" });
  add(save, by("B2:Imp", 0, "Blessing"), { type: "Heal", unit: "B2:Imp", amount: 4, hpAfter: 1 });
  add(save, by("B2:Imp", 0, "Blessing"), { type: "StatusRemoved", unit: "B2:Imp", status: "Blessing", stacks: 1, remaining: 0 });
  add(end, by("A3:Ace"), { type: "Hurt", unit: "B1:Ogre", amount: 12, hpAfter: -7 });
  add(log.length - 1, "kernel", { type: "Death", unit: "B1:Ogre" });
  add(null, "kernel", { type: "BattleEnd", winner: "A", turns: 1 });
  return log;
}

describe("the battle Log groups a beat's waves (R4-3)", () => {
  const log = turnEndLog();
  const beats = beatPlayOf(log, stepsOf(log));
  const rows = logRowsOf(log, beats);
  const captions = rows.map((r) => r.caption);

  test("R2: a status ticking on many units is one row, its fade folded in", () => {
    expect(captions).toContain("Poison ticks 3 units, −4 each, 4 blocked, 1 fades");
    expect(captions.filter((c) => c.includes("Poison"))).toHaveLength(1);
  });

  test("R1 + R4: the same actor and effect within a beat merges, summed, ×N, over all allies", () => {
    expect(captions).toContain("Nurse → Shield ×2 on all allies (×2)");
    expect(captions).toContain("Nurse → Rose +1");
  });

  test("R3 + R4: a Blessing save is one row with the hit, and the firing's targets join up", () => {
    expect(captions).toContain("Ace → −12 on all enemies → Blessing saves Imp (+4)");
    expect(captions.some((c) => c.includes("stops its death") || c === "Blessing → Imp +4")).toBe(false);
  });

  test("rows read in wave order", () => {
    const waves = beats.reduce((s, b) => s + b.waves.length, 0);
    expect(waves).toBeGreaterThanOrEqual(10);
    expect(captions).toEqual([
      "Poison ticks 3 units, −4 each, 4 blocked, 1 fades",
      "Nurse → Shield ×2 on all allies (×2)",
      "Nurse → Rose +1",
      "Ace → −12 on all enemies → Blessing saves Imp (+4)",
      "Ace → Ogre falls",
      "Side A wins",
    ]);
  });

  test("a one-unit tick and a lone hit read as before", () => {
    const one = turnEndLog().filter((e) => !(e.type === "Hurt" && e.unit !== "B1:Ogre" && e.source !== "kernel" && e.source.status === "Poison"));
    // Ids must stay the log index: rebuild only the first five events.
    const short = one.slice(0, 5).map((e, i) => ({ ...e, id: i }));
    const r = logRowsOf(short, beatPlayOf(short, stepsOf(short)));
    expect(r.map((x) => x.caption)).toEqual(["Poison ticks Ogre, −4, fades"]);
  });
});

/** Late-game fights between random lines with awoken units and fusions. */
function longFights(count: number): BattleEvent[][] {
  const pool = mvpPool();
  let seed = 11;
  const rnd = () => (seed = (seed * 1103515245 + 12345) % 2 ** 31) / 2 ** 31;
  const pick = () => pool.units[Math.floor(rnd() * pool.units.length)]!;
  const line = (tag: string) =>
    Array.from({ length: 5 }, (_, i) => {
      if (rnd() < 0.4) {
        const a = lineUnitOf(pick(), `${tag}${i}`, 3), b = lineUnitOf(pick(), `${tag}${i}b`, 3);
        try {
          return fuseUnits(a, b, { name: `${a.name}${b.name}`, discoveredBy: null }, pool as never);
        } catch {
          return a;
        }
      }
      return lineUnitOf(pick(), `${tag}${i}`, 1 + Math.floor(rnd() * 3));
    });
  return Array.from({ length: count }, (_, k) =>
    battle({ teamA: line("a").map(toBattleDef), teamB: line("b").map(toBattleDef), seed: k, abilities: pool.abilities, statuses: pool.statuses, chainStepCap: MVP_RULES.chainStepCap, ...(MVP_RULES.turnCap !== undefined ? { turnCap: MVP_RULES.turnCap } : {}) }),
  );
}

describe("the Log on real late-game fights (R4-3)", () => {
  test("fewer rows, every wave and every change in exactly one row, no identical rows in a row", () => {
    let before = 0, after = 0;
    for (const log of longFights(20)) {
      const beats = beatPlayOf(log, stepsOf(log));
      const rows = logRowsOf(log, beats);
      const waves = beats.flatMap((b) => b.waves.map((_, w) => `${b.index}:${w}`));
      expect(rows.flatMap((r) => r.waves.map((w) => `${r.beat}:${w}`)).sort()).toEqual(waves.sort());
      const changes = beats.flatMap((b) => b.waves.flatMap((w) => w.changes.map((c) => c.eventId)));
      expect(rows.flatMap((r) => r.changes.map((c) => c.eventId)).sort((x, y) => x - y)).toEqual(changes.sort((x, y) => x - y));
      for (const b of beats) {
        const inBeat = rows.filter((r) => r.beat === b.index).map((r) => r.caption);
        // The same line twice in a row is a run; a line that comes back after
        // deaths changed "all allies" is a new fact.
        expect(inBeat.filter((c, i) => c === inBeat[i - 1])).toEqual([]);
      }
      before += waves.length;
      after += rows.length;
    }
    expect(after).toBeLessThan(before * 0.85);
  }, 60_000);
});
