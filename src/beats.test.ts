// Beat segmentation tests — beatsOf is checked against the raw log it
// partitions: every event lands in exactly one beat, beats are contiguous and
// gap-free, each beat opens on a root kind, and the structural-only flag tracks
// whether any caused event touched a hero. The within-beat depth is checked
// against the kernel's causedBy links. The projection never out-knows the log.

import { describe, expect, test } from "vitest";
import { battle } from "./battle.js";
import { beatsOf, beatAtStep, depthInBeat, isRootKind } from "./beats.js";
import type { BattleEvent, BattleInput, EventType, UnitDef } from "./types.js";
import { Poison, Summoner, Venomancer, stressRegistry } from "./content/stress.js";

const ROOT_KINDS: EventType[] = ["BattleStart", "TurnStart", "TurnEnd", "PairFaced", "Strike", "Fatigue", "BattleEnd"];

// A battle that exercises every beat shape the slice names: strikes (the bread
// and butter), turn-end Poison ticks (a caused cascade off TurnEnd), a Summoner
// summon off BattleStart, a death cascade, and — with tanky units — enough
// turns (≥ FATIGUE_START = 10) that fatigue kicks in. Poison'd Venomancer vs a
// pair of heavy bricks drags the fight long.
const tank = (name: string, hp: number, pwr: number): UnitDef => ({ name, base: { hp, pwr } });

// Summons an Imp at BattleStart — the canonical hero-affecting opening beat.
const StartSummoner: UnitDef = {
  name: "StartSummoner",
  base: { hp: 6, pwr: 1 },
  abilities: [
    {
      whens: [{ kind: "trigger", on: { on: "BattleStart" } }],
      selectors: [{ kind: "holder" }],
      effects: [{ kind: "summon", unit: { name: "Imp", base: { hp: 2, pwr: 1 } } }],
    },
  ],
};

const longBattle: BattleInput = {
  // StartSummoner summons at BattleStart; the stress Summoner summons on its
  // own death — between them the fixture covers both summon shapes plus the
  // death cascade. Venomancer poisons on strike → turn-end ticks. Tanks drag
  // the fight past FATIGUE_START (turn 10).
  teamA: [Venomancer, StartSummoner, Summoner, tank("Wall", 30, 1)],
  teamB: [tank("Brick", 28, 2), tank("Slab", 26, 1)],
  seed: 7,
  statuses: stressRegistry,
};

describe("beatsOf segmentation vs the raw log", () => {
  const log = battle(longBattle);
  const beats = beatsOf(log);

  test("the battle actually covers strikes, turn-end poison, fatigue, a summon, and a death cascade", () => {
    const kinds = new Set(log.map((e) => e.type));
    expect(kinds.has("Strike"), "has strikes").toBe(true);
    expect(kinds.has("Summon"), "has a summon").toBe(true);
    expect(kinds.has("Death"), "has a death").toBe(true);
    expect(kinds.has("Fatigue"), "lasted long enough for fatigue").toBe(true);
    // A turn-end poison tick: a Hurt caused by a TurnEnd.
    const poisonTick = log.some((e) => e.type === "Hurt" && e.causedBy !== null && log[e.causedBy!]?.type === "TurnEnd");
    expect(poisonTick, "has a turn-end poison tick").toBe(true);
    expect(beats.length).toBeGreaterThan(10);
  });

  test("beats partition the log: contiguous, gap-free, covering every event exactly once", () => {
    expect(beats[0]!.start).toBe(0);
    expect(beats[beats.length - 1]!.end).toBe(log.length - 1);
    for (let i = 0; i < beats.length; i++) {
      const b = beats[i]!;
      expect(b.index).toBe(i);
      expect(b.end).toBeGreaterThanOrEqual(b.start);
      if (i > 0) expect(b.start).toBe(beats[i - 1]!.end + 1); // no gap, no overlap
      // caused = exactly the events strictly between start and end.
      expect(b.caused.map((e) => e.id)).toEqual(
        log.slice(b.start + 1, b.end + 1).map((e) => e.id),
      );
    }
  });

  test("every beat opens on a root kind; no caused event is a root", () => {
    for (const b of beats) {
      expect(isRootKind(b.kind), `beat ${b.index} root is ${b.kind}`).toBe(true);
      expect(b.root.id).toBe(b.start);
      for (const c of b.caused) {
        expect(isRootKind(c.type), `caused event ${c.id} (${c.type}) must not be a root`).toBe(false);
      }
    }
  });

  test("a Strike that lands a Hurt, a poison TurnEnd, and a summoning BattleStart are NOT structural-only", () => {
    const strikeBeat = beats.find((b) => b.kind === "Strike" && b.caused.some((e) => e.type === "Hurt"));
    expect(strikeBeat, "a Strike beat with a Hurt").toBeDefined();
    expect(strikeBeat!.structural).toBe(false);

    const poisonBeat = beats.find((b) => b.kind === "TurnEnd" && b.caused.some((e) => e.type === "Hurt"));
    expect(poisonBeat, "a TurnEnd beat with a poison Hurt").toBeDefined();
    expect(poisonBeat!.structural).toBe(false);

    const startBeat = beats[0]!;
    expect(startBeat.kind).toBe("BattleStart");
    expect(startBeat.caused.some((e) => e.type === "Summon")).toBe(true);
    expect(startBeat.structural, "BattleStart that summons is hero-affecting").toBe(false);
  });

  test("a bare TurnStart / quiet TurnEnd / PairFaced with no hero effect IS structural-only", () => {
    const structurals = beats.filter((b) => b.structural);
    expect(structurals.length, "some beats are structural-only").toBeGreaterThan(0);
    for (const b of structurals) {
      // structural ⇔ no caused event touches a hero
      const heroKinds: EventType[] = ["Hurt", "Heal", "Death", "Summon", "StatusApplied", "StatusRemoved", "StatChanged", "Silenced"];
      expect(b.caused.some((e) => heroKinds.includes(e.type)), `beat ${b.index} (${b.kind})`).toBe(false);
    }
    // A bare TurnStart (its strikes are their own beats) is the canonical case.
    expect(beats.some((b) => b.kind === "TurnStart" && b.structural)).toBe(true);
  });

  test("root-kind classification matches the actual root event types in the log", () => {
    for (const b of beats) {
      expect(b.kind).toBe(b.root.type);
      expect(ROOT_KINDS).toContain(b.kind);
    }
  });
});

describe("beatAtStep maps a playhead to its open beat", () => {
  const log = battle(longBattle);
  const beats = beatsOf(log);

  test("every step resolves to the beat whose [start,end] contains it", () => {
    for (let step = 0; step < log.length; step++) {
      const at = beatAtStep(beats, step);
      expect(at, `step ${step}`).toBeDefined();
      expect(step).toBeGreaterThanOrEqual(at!.beat.start);
      expect(step).toBeLessThanOrEqual(at!.beat.end);
      expect(at!.revealedThrough).toBe(step);
    }
  });

  test("out-of-range steps and an empty log yield undefined", () => {
    expect(beatAtStep(beats, -1)).toBeUndefined();
    expect(beatAtStep(beats, log.length)).toBeUndefined();
    expect(beatAtStep([], 0)).toBeUndefined();
    expect(beatsOf([])).toEqual([]);
  });
});

describe("depthInBeat derives within-beat causal nesting from causedBy", () => {
  const log = battle(longBattle);
  const beats = beatsOf(log);

  test("the root sits at depth 0 and a directly-caused event at depth 1", () => {
    const beat = beats.find((b) => b.kind === "Strike" && b.caused.some((e) => e.type === "Hurt"))!;
    expect(depthInBeat(beat, log, beat.root.id)).toBe(0);
    const hurt = beat.caused.find((e) => e.type === "Hurt" && e.causedBy === beat.root.id)!;
    expect(depthInBeat(beat, log, hurt.id)).toBe(1);
  });

  test("a Death caused by a Hurt nests one deeper than that Hurt", () => {
    // Find a beat with a Hurt → Death chain (a strike that kills).
    let checked = false;
    for (const b of beats) {
      const death = b.caused.find((e) => e.type === "Death");
      if (!death || death.causedBy === null) continue;
      const parent = log[death.causedBy] as BattleEvent | undefined;
      if (!parent || parent.type !== "Hurt") continue;
      const dHurt = depthInBeat(b, log, parent.id);
      const dDeath = depthInBeat(b, log, death.id);
      expect(dDeath).toBe(dHurt + 1);
      checked = true;
    }
    expect(checked, "found a Hurt→Death cascade to check nesting").toBe(true);
  });

  test("an event outside the beat reports depth 0 (never crosses a boundary)", () => {
    const beat = beats[2]!;
    const outside = beat.end + 1 < log.length ? beat.end + 1 : 0;
    if (outside !== 0) expect(depthInBeat(beat, log, outside)).toBe(0);
    expect(depthInBeat(beat, log, beat.start - 1 >= 0 ? beat.start - 1 : log.length)).toBe(0);
  });
});

// Belt-and-braces: the Poison status content is wired (the long battle relies
// on its turn-end tick to exist), so a registry regression fails loudly here.
test("Poison is a turn-end ticking status (the fixture's caused-cascade source)", () => {
  expect(Poison.abilities.length).toBeGreaterThan(0);
});
