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
 * a new effect); the last turn (5) adds a death. */
function loopLog(last = 5): BattleEvent[] {
  const log: BattleEvent[] = [];
  let turn = 0;
  const add = (causedBy: number | null, source: "kernel" | AbilityRef, body: EventBody) => {
    log.push({ id: log.length, turn, causedBy, source, ...body } as BattleEvent);
    return log.length - 1;
  };
  const by = (unit: string, ability = 0, status?: string): AbilityRef => ({ unit, ability, ...(status ? { status } : {}) });
  const roster = (side: string, names: string[]) => names.map((n, i) => ({ id: `${side}${i + 1}:${n}`, name: n, hp: 30, pwr: 2 }));
  add(null, "kernel", { type: "BattleStart", teams: { A: roster("A", ["Rose", "Nurse"]), B: roster("B", ["Ogre", "Imp"]) } });
  for (turn = 1; turn <= last; turn++) {
    add(null, "kernel", { type: "TurnStart" });
    const target = turn === 3 ? "A2:Nurse" : "A1:Rose";
    const strike = add(null, "kernel", { type: "Strike", striker: "B1:Ogre", defender: target });
    const hurt = add(strike, "kernel", { type: "Hurt", unit: target, amount: turn + 1, hpAfter: 20 });
    if (turn === last) add(hurt, "kernel", { type: "Death", unit: target });
    else add(hurt, by("A2:Nurse"), { type: "Heal", unit: target, amount: 1, hpAfter: 21 });
    const end = add(null, "kernel", { type: "TurnEnd" });
    add(end, by("B2:Imp", 0, "Poison"), { type: "Hurt", unit: "B2:Imp", amount: turn, hpAfter: 30 - turn });
  }
  turn = last;
  add(null, "kernel", { type: "BattleEnd", winner: "B", turns: last });
  return log;
}

const rowsOf = (items: LogItem[]): LogRow[] => items.flatMap((x) => (isLogFold(x) ? x.rows : [x]));

describe("the Log folds a turn that repeats the one before it (R4-13)", () => {
  const log = loopLog();
  const rows = logRowsOf(log, beatPlayOf(log, stepsOf(log)));
  const items = foldTurnsOf(log, rows);
  const folds = items.filter(isLogFold);

  test("turns 2–4 repeat the turn before them, amounts and targets aside, and read as one fold T2–T4; 1 and 5 stay rows", () => {
    expect(folds.map((f) => [f.turn, f.turnTo])).toEqual([[2, 4]]);
    expect([...new Set(items.filter((x) => !isLogFold(x)).map((x) => x.turn))]).toEqual([1, 5]);
  });

  test("a fold sums its run of turns: damage, heals, deaths", () => {
    const f = folds[0]!;
    expect(f.damage).toEqual({ A: 3 + 4 + 5, B: 2 + 3 + 4 });
    expect(f.heals).toEqual({ A: 3, B: 0 });
    expect(f.deaths).toEqual([]);
    expect(f.blocked).toBe(0);
    expect(f.caption).toBe(`Same as last turn ×3: −21 damage, +3 healed · ${f.rows.length} rows`);
  });

  test("a fold keeps its turns' rows, in order: every row once", () => {
    expect(rowsOf(items)).toEqual(rows);
    expect(folds[0]!.rows).toEqual(rows.filter((r) => r.turn >= 2 && r.turn <= 4));
  });

  test("one repeated turn alone reads as before, without a count", () => {
    const short = loopLog(3);
    const one = foldTurnsOf(short, logRowsOf(short, beatPlayOf(short, stepsOf(short)))).filter(isLogFold);
    expect(one.map((f) => [f.turn, f.turnTo])).toEqual([[2, 2]]);
    expect(one[0]!.caption).toMatch(/^Same as last turn: −5 damage/);
  });

  test("a death breaks the loop", () => {
    expect(rows.some((r) => r.turn === 5 && r.caption.includes("falls"))).toBe(true);
    expect(folds.some((f) => f.turnTo === 5)).toBe(false);
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
      const fs = items.filter(isLogFold);
      for (const [k, x] of fs.entries()) {
        expect(x.rows.length).toBeGreaterThanOrEqual(2);
        expect(rows.filter((r) => r.turn >= x.turn && r.turn <= x.turnTo)).toEqual(x.rows);
        const per = rows.filter((r) => r.turn === x.turn - 1).length;
        for (let t = x.turn; t <= x.turnTo; t++) expect(rows.filter((r) => r.turn === t)).toHaveLength(per);
        // A run is whole: the next fold never starts on the turn right after this one.
        if (fs[k + 1]) expect(fs[k + 1]!.turn).toBeGreaterThan(x.turnTo + 1);
      }
      folded += fs.length;
      for (const x of fs) longest = Math.max(longest, x.turnTo - x.turn + 1);
    }
    expect(folded).toBeGreaterThan(0);
    expect(longest).toBeGreaterThanOrEqual(3);
  }, 60_000);
});
