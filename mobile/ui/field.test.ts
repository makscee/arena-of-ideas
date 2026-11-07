// The battle's fixed field (arena#48): columns that never move, ghosts, summons in vacancies.
import { expect, test } from "vitest";
import type { BattleEvent } from "../../src/types";
import { fieldAt } from "./field";

const logOf = (...evs: object[]): BattleEvent[] => evs.map((e, id) => ({ id, turn: 1, causedBy: null, source: "kernel", ...e })) as unknown as BattleEvent[];
const roster = (...ids: string[]) => ids.map((id) => ({ id, name: id, hp: 5, pwr: 1 }));
const start = { type: "BattleStart", turn: 0, teams: { A: roster("a0", "a1", "a2"), B: roster("b0", "b1", "b2") } };
const shape = (log: BattleEvent[], side: "A" | "B") => fieldAt(log, log.length)[side].map((c) => `${c.id}${c.alive ? "" : `×T${c.fell}`}`);

test("a unit keeps its column; a death leaves a numbered ghost and nothing slides", () => {
  const log = logOf(start, { type: "Death", unit: "b0", turn: 1 }, { type: "Death", unit: "b1", turn: 2 });
  expect(shape(log, "B")).toEqual(["b0×T1", "b1×T2", "b2"]);
  expect(shape(log, "A")).toEqual(["a0", "a1", "a2"]);
});

test("a summon takes its fallen summoner's column, not the first ghost's", () => {
  const log = logOf(
    start,
    { type: "Death", unit: "b0", turn: 1 },
    { type: "Death", unit: "b2", turn: 3 },
    { type: "Summon", side: "B", unit: "w1", name: "Wisp", hp: 2, pwr: 1, source: { unit: "b2", ability: 0 } },
  );
  const cols = fieldAt(log, log.length).B;
  expect(cols.map((c) => c.id)).toEqual(["b0", "b1", "w1"]);
  expect(cols[2]).toMatchObject({ alive: true, displaced: "b2" });
});

test("a summon with no fallen summoner takes the first ghost's column; with none it adds a column at the back", () => {
  const log = logOf(start, { type: "Death", unit: "b1", turn: 1 }, { type: "Summon", side: "B", unit: "w1", name: "Wisp", hp: 2, pwr: 1, source: { unit: "b0", ability: 0 } });
  expect(shape(log, "B")).toEqual(["b0", "w1", "b2"]);
  const more = logOf(...log, { type: "Summon", side: "B", unit: "w2", name: "Wisp", hp: 2, pwr: 1, source: { unit: "b0", ability: 0 } });
  expect(shape(more, "B")).toEqual(["b0", "w1", "b2", "w2"]);
  expect(shape(more, "A")).toEqual(["a0", "a1", "a2"]);
});

test("a revive stands again in its own column", () => {
  const log = logOf(start, { type: "Death", unit: "a1", turn: 1 }, { type: "Summon", side: "A", unit: "a1", name: "a1", hp: 1, pwr: 1, resurrected: true, source: { unit: "a0", ability: 0 } });
  expect(shape(log, "A")).toEqual(["a0", "a1", "a2"]);
});

test("a revive whose column a summon took goes to a vacancy, else the back", () => {
  const log = logOf(
    start,
    { type: "Death", unit: "a1", turn: 1 },
    { type: "Summon", side: "A", unit: "w1", name: "Wisp", hp: 2, pwr: 1, source: { unit: "a0", ability: 0 } },
    { type: "Summon", side: "A", unit: "a1", name: "a1", hp: 1, pwr: 1, resurrected: true, source: { unit: "a0", ability: 0 } },
  );
  expect(shape(log, "A")).toEqual(["a0", "w1", "a2", "a1"]);
});

test("skipped events don't apply, and events past the cut aren't seen", () => {
  const log = logOf(start, { type: "Death", unit: "b0", turn: 1 }, { type: "Death", unit: "b1", turn: 1 });
  expect(fieldAt(log, 2, new Set([2])).B.map((c) => c.alive)).toEqual([false, true, true]);
  expect(fieldAt(log, 1).B.map((c) => c.alive)).toEqual([false, true, true]);
});
