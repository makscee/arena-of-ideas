import { describe, expect, it } from "vitest";
import { DEFAULT_RUN_POOL, stressAbilities, stressRegistry } from "../index.js";
import { MVP_RULES, type MvpContent, type PlayerRef, type UnitContent } from "./contract.js";
import { lineUnitOf } from "./forms.js";
import { applyMvpDecision, initMvpRun, MvpBadDecision, MvpDecisionError, offersAt, runView, synthGhost } from "./run.js";

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
    expect(s.offers).toHaveLength(3);
  });

  it("grows the shop 3 → 6 by round 7", () => {
    expect([1, 2, 3, 4, 5, 6, 7, 12].map((r) => offersAt(MVP_RULES, r))).toEqual([3, 4, 4, 5, 5, 5, 6, 6]);
    const { offersGrowAt: _, ...old } = { ...MVP_RULES, offers: 5 };
    expect(offersAt(old, 1)).toBe(5);
    expect(offersAt(old, 12)).toBe(5);
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

  it("puts a fused unit in the front-most of its two slots, in either order: a swap changes only the recipe", () => {
    const s0 = initMvpRun({ runId: "r", player: me, seed: 3, content, ...day });
    const [a, b, c, d] = ["u0", "u1", "u2", "u3"].map((id, i) => lineUnitOf(units.find((u) => u.id === id)!, `x${i}`, 3));
    const s = { ...s0, line: [a!, b!, c!, d!] };
    const fuse = { name: "Test", discoveredBy: me };
    const db = applyMvpDecision(s, { kind: "fuse", first: 3, second: 1 }, content, { fuse }).state;
    const bd = applyMvpDecision(s, { kind: "fuse", first: 1, second: 3 }, content, { fuse }).state;
    for (const [st, uid] of [[db, "x3"], [bd, "x1"]] as const) {
      expect(st.line.map((u) => u.uid)).toEqual(["x0", uid, "x2"]);
      expect(st.line[1]).toMatchObject({ kind: "fused" });
    }
    expect(db.line[1]!.fusion).toMatchObject({ first: "u3", second: "u1" });
    expect(bd.line[1]!.fusion).toMatchObject({ first: "u1", second: "u3" });
  });

  it("fights a ghost, logs a causal battle and turns the round", () => {
    let s = initMvpRun({ runId: "r", player: me, seed: 9, content, ...day });
    s = applyMvpDecision(s, { kind: "buy", slot: 0 }, content).state;
    const ghost = synthGhost({ content, round: 1, seed: 5, ghostId: "g", createdAt: "2026-10-05T00:00:00.000Z" });
    const step = applyMvpDecision(s, { kind: "fight" }, content, { fight: { ghost, battleId: "b", battleSeed: 11, at: "2026-10-05T00:01:00.000Z" } });
    expect(step.fight?.round).toBe(1);
    expect(step.battle?.log.at(-1)?.type).toBe("BattleEnd");
    expect(step.battle?.log.every((e) => "causedBy" in e)).toBe(true);
    // Battle units carry the whole card and the kernel instance id from the log.
    const start = step.battle!.log.find((e) => e.type === "BattleStart");
    const roster = start?.type === "BattleStart" ? start.teams : { A: [], B: [] };
    expect(step.battle!.teamA).toEqual(s.line.map((u, i) => ({ ...u, id: roster.A[i]!.id })));
    expect(step.battle!.teamB.map((u) => u.id)).toEqual(roster.B.map((r) => r.id));
    expect(step.battle!.player).toEqual(me);
    expect(step.battle).toMatchObject({ runId: "r", kind: "round", round: 1, at: "2026-10-05T00:01:00.000Z", opponent: ghost.player });
    expect(step.state.round).toBe(2);
    expect(step.state.gold).toBe(10);
    expect(step.state.hearts).toBe(5 - step.fight!.heartsLost);
  });

  it("throws MvpBadDecision, not a refusal, for a kind it doesn't know", () => {
    const s = initMvpRun({ runId: "r", player: me, seed: 1, content, ...day });
    const zap = { kind: "zap" } as unknown as Parameters<typeof applyMvpDecision>[1];
    expect(() => applyMvpDecision(s, zap, content)).toThrow(MvpBadDecision);
    expect(() => applyMvpDecision(s, zap, content)).not.toThrow(MvpDecisionError);
    expect(() => applyMvpDecision(s, { kind: "toString" } as unknown as typeof zap, content)).toThrow(MvpBadDecision);
    expect(() => applyMvpDecision({ ...s, phase: "over" }, zap, content)).toThrow(MvpBadDecision);
  });
});
