// Coin-holder projection tests (#065 slice 3) — coinHolderAt is the pure
// projection the coin marker and the coin-flip card draw from. Two layers,
// mirroring the slice-1/2 style:
//
//  1. A PROPERTY SWEEP over real battles (many seeds × reference matchups). At
//     every step the holder must equal an INDEPENDENT re-derivation — the
//     `first` of the most recent `PairFaced` at or before the step (null before
//     the first). And the three behaviours the slice names are asserted against
//     the real log: the holder is correct RIGHT AFTER a PairFaced, PERSISTS
//     across that pairing's intervening strikes, and FLIPS at the next PairFaced.
//
//  2. TARGETED unit tests on hand-built logs: null-before-first-pairing, the
//     coin landing on `first`, persistence across strikes, and a re-flip.

import { describe, expect, test } from "vitest";
import { battle } from "./battle.js";
import { coinHolderAt } from "./beats.js";
import type { BattleEvent, BattleInput, EventBody, UnitDef } from "./types.js";
import { Summoner, Venomancer, Silencer, Necromancer, stressRegistry } from "./content/stress.js";
import { stressAbilities } from "./content/stress.js";

const tank = (name: string, hp: number, pwr: number): UnitDef => ({ name, base: { hp, pwr }, ability: "Strike" });

// A spread of real battles. Deaths advance fresh front units, so each battle
// faces several pairings — every battle re-flips the coin at least once, and
// the long ones trade many strikes within a pairing (the persistence case).
const matchups: { name: string; input: BattleInput }[] = [];
for (const seed of [1, 3, 7, 11, 23, 42, 99]) {
  matchups.push({
    name: `venom/summon/necro vs bricks seed ${seed}`,
    input: {
      teamA: [Venomancer, Summoner, Necromancer, tank("Wall", 24, 1)],
      teamB: [Silencer, tank("Brick", 22, 2), tank("Slab", 20, 1)],
      seed,
      statuses: stressRegistry,
  abilities: stressAbilities,
    },
  });
}

/** Independent re-derivation: the `first` of the most recent PairFaced at or
 * before `step`, null before any. No shared helper with the implementation. */
function expectedHolder(log: BattleEvent[], step: number): string | null {
  let holder: string | null = null;
  for (const e of log) {
    if (e.id > step) break;
    if (e.type === "PairFaced") holder = e.first;
  }
  return holder;
}

describe("coinHolderAt matches an independent re-derivation at every step", () => {
  for (const { name, input } of matchups) {
    test(name, () => {
      const log = battle(input);
      const pairings = log.filter((e) => e.type === "PairFaced");
      expect(pairings.length, `${name} faces ≥2 pairings (so the coin re-flips)`).toBeGreaterThanOrEqual(2);

      for (let step = 0; step < log.length; step++) {
        expect(coinHolderAt(log, step), `${name} step ${step}`).toBe(expectedHolder(log, step));
      }
    });
  }

  test("null before the first pairing — the coin has no holder until a pair is faced", () => {
    for (const { name, input } of matchups) {
      const log = battle(input);
      const firstPair = log.find((e) => e.type === "PairFaced")!;
      expect(firstPair, `${name} has a PairFaced`).toBeDefined();
      // Every step strictly before the first PairFaced has no holder.
      for (let step = 0; step < firstPair.id; step++) {
        expect(coinHolderAt(log, step), `${name} step ${step} (pre-first-pairing)`).toBeNull();
      }
      // And it lands the instant the pairing is faced.
      const pf = firstPair as Extract<BattleEvent, { type: "PairFaced" }>;
      expect(coinHolderAt(log, firstPair.id)).toBe(pf.first);
    }
  });

  test("the holder is correct right after a PairFaced, PERSISTS across that pairing's strikes, and FLIPS at the next", () => {
    let sawPersist = false;
    let sawReflip = false;
    for (const { input } of matchups) {
      const log = battle(input);
      const pairs = log.filter((e): e is Extract<BattleEvent, { type: "PairFaced" }> => e.type === "PairFaced");
      for (let p = 0; p < pairs.length; p++) {
        const pf = pairs[p]!;
        // Right after this PairFaced the holder is its `first`.
        expect(coinHolderAt(log, pf.id)).toBe(pf.first);
        // PERSISTS across every step until the NEXT PairFaced (or the log end):
        // the intervening Strikes (and their Hurts/Deaths) do not move the coin.
        const next = pairs[p + 1];
        const until = next ? next.id - 1 : log.length - 1;
        const intervening = log.slice(pf.id, until + 1);
        if (intervening.some((e) => e.type === "Strike")) sawPersist = true;
        for (let step = pf.id; step <= until; step++) {
          expect(coinHolderAt(log, step), `holder persists through step ${step}`).toBe(pf.first);
        }
        // FLIPS at the next PairFaced.
        if (next) {
          expect(coinHolderAt(log, next.id)).toBe(next.first);
          if (next.first !== pf.first) sawReflip = true;
        }
      }
    }
    expect(sawPersist, "some pairing trades a strike with the coin held steady").toBe(true);
    expect(sawReflip, "some next pairing re-flips the coin to a DIFFERENT holder").toBe(true);
  });
});

// ---------------------------------------------------------------------------
// Targeted hand-built logs — coinHolderAt is pure over the log, so a minimal
// log isolates each behaviour with no kernel.
// ---------------------------------------------------------------------------

let nextId = 0;
function ev(body: EventBody, causedBy: number | null): BattleEvent {
  return { id: nextId++, turn: 1, causedBy, source: "kernel", ...body } as BattleEvent;
}

describe("the coin-holder decisions", () => {
  test("null before any pairing is faced", () => {
    nextId = 0;
    const start = ev({ type: "BattleStart", teams: { A: [], B: [] } }, null); // id 0
    const ts = ev({ type: "TurnStart" }, null); // id 1
    const log = [start, ts];
    expect(coinHolderAt(log, 0)).toBeNull();
    expect(coinHolderAt(log, 1)).toBeNull();
  });

  test("the coin lands on `first` and PERSISTS across that pairing's strikes", () => {
    nextId = 0;
    const start = ev({ type: "BattleStart", teams: { A: [], B: [] } }, null); // 0
    const pf = ev({ type: "PairFaced", a: "A1", b: "B1", first: "B1" }, null); // 1 — coin → B1
    const s1 = ev({ type: "Strike", striker: "B1", defender: "A1" }, null); // 2
    const h1 = ev({ type: "Hurt", unit: "A1", amount: 3, hpAfter: 4 }, s1.id); // 3
    const s2 = ev({ type: "Strike", striker: "A1", defender: "B1" }, null); // 4
    const h2 = ev({ type: "Hurt", unit: "B1", amount: 2, hpAfter: 6 }, s2.id); // 5
    const log = [start, pf, s1, h1, s2, h2];
    // Lands on B1 at the PairFaced and STAYS B1 through both strikes and hurts.
    for (let step = pf.id; step < log.length; step++) {
      expect(coinHolderAt(log, step), `step ${step}`).toBe("B1");
    }
    // Before the PairFaced: still null.
    expect(coinHolderAt(log, 0)).toBeNull();
  });

  test("a new PairFaced RE-FLIPS the coin to the new pairing's first striker", () => {
    nextId = 0;
    const start = ev({ type: "BattleStart", teams: { A: [], B: [] } }, null); // 0
    const pf1 = ev({ type: "PairFaced", a: "A1", b: "B1", first: "A1" }, null); // 1 — coin → A1
    const s1 = ev({ type: "Strike", striker: "A1", defender: "B1" }, null); // 2
    const d1 = ev({ type: "Death", unit: "B1" }, s1.id); // 3 — B1 falls, B2 fronts
    const pf2 = ev({ type: "PairFaced", a: "A1", b: "B2", first: "B2" }, null); // 4 — re-flip → B2
    const s2 = ev({ type: "Strike", striker: "B2", defender: "A1" }, null); // 5
    const log = [start, pf1, s1, d1, pf2, s2];
    // First pairing: A1 holds through its strike and the death.
    expect(coinHolderAt(log, pf1.id)).toBe("A1");
    expect(coinHolderAt(log, s1.id)).toBe("A1");
    expect(coinHolderAt(log, d1.id)).toBe("A1"); // held until the re-flip, not the death
    // The re-flip lands the coin on the new first striker, B2.
    expect(coinHolderAt(log, pf2.id)).toBe("B2");
    expect(coinHolderAt(log, s2.id)).toBe("B2");
  });

  test("out-of-range / empty steps yield null", () => {
    expect(coinHolderAt([], 0)).toBeNull();
    nextId = 0;
    const pf = ev({ type: "PairFaced", a: "A1", b: "B1", first: "A1" }, null); // id 0
    const log = [pf];
    expect(coinHolderAt(log, -1)).toBeNull(); // before the log
    expect(coinHolderAt(log, 999)).toBe("A1"); // past the end clamps to the last holder
  });
});
