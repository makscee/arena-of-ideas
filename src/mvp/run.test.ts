import { describe, expect, it } from "vitest";
import { DEFAULT_RUN_POOL, stressAbilities, stressRegistry } from "../index.js";
import type { MvpContent, PlayerRef, UnitContent } from "./contract.js";
import { applyMvpDecision, initMvpRun, runView, synthGhost } from "./run.js";

const units: UnitContent[] = DEFAULT_RUN_POOL.map((d, i) => {
  const form = { when: d.triggers ?? [], who: d.selectors ?? [], does: d.abilities ?? [] };
  return { id: `u${i}`, name: d.name, emoji: "x", tier: 1, base: d.base, forms: { sleeping: form, awoken: form } };
});
const content: MvpContent = { version: "t", units, abilities: stressAbilities, statuses: stressRegistry };
const me: PlayerRef = { id: "p", name: "me", bot: false };
const day = { day: 1, startedAt: "2026-10-05T00:00:00.000Z" };

describe("MVP thin run", () => {
  it("starts with the MVP rules", () => {
    const s = initMvpRun({ runId: "r", player: me, seed: 1, content, ...day });
    expect(runView(s)).toMatchObject({ round: 1, hearts: 5, gold: 10, phase: "shop", line: [] });
    expect(s.offers).toHaveLength(5);
  });

  it("is deterministic and pure", () => {
    const a = initMvpRun({ runId: "r", player: me, seed: 42, content, ...day });
    const b = initMvpRun({ runId: "r", player: me, seed: 42, content, ...day });
    expect(a).toEqual(b);
    const after = applyMvpDecision(a, { kind: "buy", slot: 0 }, content).state;
    expect(a.line).toHaveLength(0);
    expect(after.line).toHaveLength(1);
    expect(after.gold).toBe(7);
  });

  it("merges a second copy for +1 PWR / +2 HP", () => {
    let s = initMvpRun({ runId: "r", player: me, seed: 3, content, ...day });
    s = { ...s, offers: [{ slot: 0, unitId: "u0", tier: 1, cost: 3 }, { slot: 1, unitId: "u0", tier: 1, cost: 3 }] };
    s = applyMvpDecision(s, { kind: "buy", slot: 0 }, content).state;
    s = applyMvpDecision(s, { kind: "buy", slot: 0 }, content).state;
    expect(s.line).toHaveLength(1);
    expect(s.line[0]!.copies).toBe(2);
    expect(s.line[0]!.stats).toEqual({ pwr: units[0]!.base.pwr + 1, hp: units[0]!.base.hp + 2 });
  });

  it("fights a ghost, logs a causal battle and turns the round", () => {
    let s = initMvpRun({ runId: "r", player: me, seed: 9, content, ...day });
    s = applyMvpDecision(s, { kind: "buy", slot: 0 }, content).state;
    const ghost = synthGhost({ content, round: 1, seed: 5, ghostId: "g", createdAt: "2026-10-05T00:00:00.000Z" });
    const step = applyMvpDecision(s, { kind: "fight" }, content, { fight: { ghost, battleId: "b", battleSeed: 11 } });
    expect(step.fight?.round).toBe(1);
    expect(step.battle?.log.at(-1)?.type).toBe("BattleEnd");
    expect(step.battle?.log.every((e) => "causedBy" in e)).toBe(true);
    expect(step.state.round).toBe(2);
    expect(step.state.gold).toBe(10);
    expect(step.state.hearts).toBe(5 - step.fight!.heartsLost);
  });
});
