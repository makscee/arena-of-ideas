// The battle Log folds repeated turns (round 4, R4-13, note 4 R5): a turn
// whose rows repeat the previous turn's reads as one summary row, its rows
// kept behind it in order.

import { describe, expect, test } from "vitest";
import { battle } from "../battle.js";
import type { AbilityRef, BattleEvent, EventBody } from "../types.js";
import { MVP_RULES } from "./contract.js";
import { toBattleDef } from "./fight.js";
import { fuseUnits, lineUnitOf } from "./forms.js";
import { beatPlayOf, foldTurnsOf, isLogFold, logRowsOf, stepsOf, type LogItem, type LogRow } from "./trace.js";
import { mvpPool } from "./units.js";

/** A hand-built looping fight: each turn Ogre strikes, Nurse heals the one
 * hit, Poison ticks on Imp. Turn 3 hits Nurse instead of Rose (a target, not
 * a new effect); turn 5 adds Rose's death. */
function loopLog(): BattleEvent[] {
  const log: BattleEvent[] = [];
  let turn = 0;
  const add = (causedBy: number | null, source: "kernel" | AbilityRef, body: EventBody) => {
    log.push({ id: log.length, turn, causedBy, source, ...body } as BattleEvent);
    return log.length - 1;
  };
  const by = (unit: string, ability = 0, status?: string): AbilityRef => ({ unit, ability, ...(status ? { status } : {}) });
  const roster = (side: string, names: string[]) => names.map((n, i) => ({ id: `${side}${i + 1}:${n}`, name: n, hp: 30, pwr: 2 }));
  add(null, "kernel", { type: "BattleStart", teams: { A: roster("A", ["Rose", "Nurse"]), B: roster("B", ["Ogre", "Imp"]) } });
  for (turn = 1; turn <= 5; turn++) {
    add(null, "kernel", { type: "TurnStart" });
    const target = turn === 3 ? "A2:Nurse" : "A1:Rose";
    const strike = add(null, "kernel", { type: "Strike", striker: "B1:Ogre", defender: target });
    const hurt = add(strike, "kernel", { type: "Hurt", unit: target, amount: turn + 1, hpAfter: 20 });
    if (turn === 5) add(hurt, "kernel", { type: "Death", unit: target });
    else add(hurt, by("A2:Nurse"), { type: "Heal", unit: target, amount: 1, hpAfter: 21 });
    const end = add(null, "kernel", { type: "TurnEnd" });
    add(end, by("B2:Imp", 0, "Poison"), { type: "Hurt", unit: "B2:Imp", amount: turn, hpAfter: 30 - turn });
  }
  turn = 5;
  add(null, "kernel", { type: "BattleEnd", winner: "B", turns: 5 });
  return log;
}

const rowsOf = (items: LogItem[]): LogRow[] => items.flatMap((x) => (isLogFold(x) ? x.rows : [x]));

describe("the Log folds a turn that repeats the one before it (R4-13)", () => {
  const log = loopLog();
  const rows = logRowsOf(log, beatPlayOf(log, stepsOf(log)));
  const items = foldTurnsOf(log, rows);
  const folds = items.filter(isLogFold);

  test("turns 2–4 repeat the turn before them, amounts and targets aside; 1 and 5 stay rows", () => {
    expect(folds.map((f) => f.turn)).toEqual([2, 3, 4]);
    expect([...new Set(items.filter((x) => !isLogFold(x)).map((x) => x.turn))]).toEqual([1, 5]);
  });

  test("a fold sums its turn: damage, heals, deaths", () => {
    const t3 = folds.find((f) => f.turn === 3)!;
    expect(t3.damage).toEqual({ A: 4, B: 3 });
    expect(t3.heals).toEqual({ A: 1, B: 0 });
    expect(t3.deaths).toEqual([]);
    expect(t3.blocked).toBe(0);
    expect(t3.caption).toBe(`Same as last turn: −7 damage, +1 healed · ${t3.rows.length} rows`);
  });

  test("a fold keeps its turn's rows, in order: every row once", () => {
    expect(rowsOf(items)).toEqual(rows);
    for (const f of folds) expect(f.rows.every((r) => r.turn === f.turn)).toBe(true);
  });

  test("a death breaks the loop", () => {
    expect(rows.some((r) => r.turn === 5 && r.caption.includes("falls"))).toBe(true);
    expect(folds.some((f) => f.turn === 5)).toBe(false);
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

describe("folding on real late-game fights (R4-13)", () => {
  test("every row kept once and in order; folds are whole turns of 2+ rows right after the turn they repeat; loops fold", () => {
    let folded = 0, longest = 0;
    for (const log of longFights(40)) {
      const rows = logRowsOf(log, beatPlayOf(log, stepsOf(log)));
      const items = foldTurnsOf(log, rows);
      expect(rowsOf(items)).toEqual(rows);
      for (const x of items.filter(isLogFold)) {
        expect(x.rows.length).toBeGreaterThanOrEqual(2);
        expect(rows.filter((r) => r.turn === x.turn)).toEqual(x.rows);
        expect(rows.filter((r) => r.turn === x.turn - 1)).toHaveLength(x.rows.length);
      }
      const n = items.filter(isLogFold).length;
      folded += n;
      longest = Math.max(longest, n);
    }
    expect(folded).toBeGreaterThan(0);
    expect(longest).toBeGreaterThanOrEqual(3);
  }, 60_000);
});
