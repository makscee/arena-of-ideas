import { describe, expect, test } from "vitest";
import { ancestry, boardAt } from "../src/index.js";
import { boardFirstFixture } from "./battle-fixtures.js";

describe("board-first named production fixtures", () => {
  test("strike/status fixture pins canonical families and a multi-hop chain", () => {
    const { log } = boardFirstFixture("board-first-strike-status-chain");
    expect(log.some((e) => e.type === "Strike")).toBe(true);
    expect(log.some((e) => e.type === "StatusApplied")).toBe(true);
    expect(log.some((e) => e.type === "StatusRemoved")).toBe(true);
    expect(log.some((e) => e.type === "Hurt" && e.source !== "kernel" && e.source.status === "Poison")).toBe(true);
    expect(log.some((e) => ancestry(log, e.id).length >= 3)).toBe(true);
  });

  test("summon/death fixture changes membership only on logged boundaries", () => {
    const { log } = boardFirstFixture("board-first-summon-death-advance");
    const death = log.find((e) => e.type === "Death");
    if (death?.type !== "Death") throw new Error("fixture has no death");
    const summon = log.find((e) => e.type === "Summon" && e.causedBy === death.id);
    if (summon?.type !== "Summon") throw new Error("fixture has no death-caused summon");
    const beforeDeath = boardAt(log, death.id - 1);
    const atDeath = boardAt(log, death.id);
    expect([...beforeDeath.lines.A, ...beforeDeath.lines.B].some((u) => u.id === death.unit)).toBe(true);
    expect([...atDeath.graves.A, ...atDeath.graves.B].some((u) => u.id === death.unit)).toBe(true);
    const beforeSummon = boardAt(log, summon.id - 1);
    const atSummon = boardAt(log, summon.id);
    expect([...beforeSummon.lines.A, ...beforeSummon.lines.B].some((u) => u.id === summon.unit)).toBe(false);
    expect([...atSummon.lines.A, ...atSummon.lines.B].some((u) => u.id === summon.unit)).toBe(true);
  });
});
