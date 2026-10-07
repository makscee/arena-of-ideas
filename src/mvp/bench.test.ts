// The bench (round 3, R3-13; docs/round3/shop.md 13 A): 3 board slots after
// the line hold units that don't fight. Copies merge and awaken there, units
// sell and fuse from it, and reorder moves or swaps across line and bench.
import { describe, expect, it } from "vitest";
import { DEFAULT_RUN_POOL, stressAbilities, stressRegistry } from "../index.js";
import { MVP_RULES, boardSlot, boardUnit, type Decision, type LineUnit, type MvpContent, type Offer, type PlayerRef, type UnitContent } from "./contract.js";
import { lineUnitOf } from "./forms.js";
import { applyMvpDecision, initMvpRun, MvpDecisionError, runView, synthGhost, type MvpRunState } from "./run.js";

// Nine units: the stock pool, cycled, so a full line plus a full bench all differ.
const units: UnitContent[] = Array.from({ length: 9 }, (_, i) => {
  const d = DEFAULT_RUN_POOL[i % DEFAULT_RUN_POOL.length]!;
  const form = { when: d.triggers ?? [], who: d.selectors ?? [], does: d.abilities ?? [] };
  return { id: `u${i}`, name: `${d.name} ${i}`, emoji: "x", archetype: "x", tier: 1, base: d.base, forms: { sleeping: form, awoken: form } };
});
const content: MvpContent = { version: "t", units, abilities: stressAbilities, statuses: stressRegistry };
const me: PlayerRef = { id: "p", name: "me", bot: false };
const day = { day: 1, startedAt: "2026-10-05T00:00:00.000Z" };
const fuse = { name: "Test", discoveredBy: me };

const unit = (i: number, uid: string, copies = 1): LineUnit => lineUnitOf(units[i]!, uid, copies);
const run = (line: LineUnit[], bench: LineUnit[] = [], seed = 3): MvpRunState => ({ ...initMvpRun({ runId: "r", player: me, seed, content, ...day }), gold: 30, line, bench });
const offer = (unitId: string): Offer[] => [{ slot: 0, unitId, tier: 1, cost: 3 }];
const apply = (s: MvpRunState, d: Decision) => applyMvpDecision(s, d, content, { fuse }).state;
const uids = (us: LineUnit[]) => us.map((u) => u.uid);
const fullLine = () => [0, 1, 2, 3, 4].map((i) => unit(i, `l${i}`));

describe("MVP bench (R3-13)", () => {
  it("has 3 slots after the line: board slots 5, 6, 7", () => {
    expect(MVP_RULES.benchSize).toBe(3);
    expect([4, 5, 7, 8].map((slot) => boardSlot(MVP_RULES, slot))).toEqual([{ zone: "line", index: 4 }, { zone: "bench", index: 0 }, { zone: "bench", index: 2 }, null]);
    const s = initMvpRun({ runId: "r", player: me, seed: 1, content, ...day });
    expect(runView(s).bench).toEqual([]);
    expect(boardUnit({ line: [], bench: [unit(0, "b0")] }, MVP_RULES, 5)?.uid).toBe("b0");
  });

  it("a buy with a full line lands on the bench (slot 5); with both full it is refused", () => {
    let s = { ...run(fullLine()), offers: offer("u5") };
    s = apply(s, { kind: "buy", slot: 0 });
    expect(s.line).toHaveLength(5);
    expect(s.bench.map((u) => u.unitId)).toEqual(["u5"]);
    expect(boardUnit(s, s.rules, 5)?.unitId).toBe("u5");
    const full = { ...run(fullLine(), [unit(5, "b0"), unit(6, "b1"), unit(7, "b2")]), offers: offer("u8") };
    expect(() => apply(full, { kind: "buy", slot: 0 })).toThrow(/your line and bench are full/);
  });

  it("copies merge into a bench unit, and the 3rd copy awakens it there", () => {
    let s = { ...run(fullLine(), [unit(5, "b0")]), offers: [...offer("u5"), ...offer("u5").map((o) => ({ ...o, slot: 1 }))] };
    s = apply(s, { kind: "buy", slot: 0 });
    expect(s.bench[0]).toMatchObject({ uid: "b0", copies: 2, form: "sleeping" });
    s = apply(s, { kind: "buy", slot: 0 });
    expect(s.bench).toHaveLength(1);
    expect(s.bench[0]).toMatchObject({ uid: "b0", copies: 3, form: "awoken" });
    expect(s.line).toHaveLength(5);
  });

  it("sells from bench slot 5", () => {
    const s = apply({ ...run(fullLine(), [unit(5, "b0", 3), unit(6, "b1")]), gold: 0 }, { kind: "sell", index: 5 });
    expect(uids(s.bench)).toEqual(["b1"]);
    expect(s.gold).toBe(2);
    expect(() => apply(s, { kind: "sell", index: 6 })).toThrow(MvpDecisionError);
  });

  it("reorder: swaps across line and bench, moves into an empty slot, shifts within a zone, refuses off the board", () => {
    const s = run(fullLine(), [unit(5, "b0"), unit(6, "b1")]);
    const swap = apply(s, { kind: "reorder", from: 1, to: 6 });
    expect(uids(swap.line)).toEqual(["l0", "b1", "l2", "l3", "l4"]);
    expect(uids(swap.bench)).toEqual(["b0", "l1"]);
    const move = apply(s, { kind: "reorder", from: 2, to: 7 });
    expect(uids(move.line)).toEqual(["l0", "l1", "l3", "l4"]);
    expect(uids(move.bench)).toEqual(["b0", "b1", "l2"]);
    const back = apply(move, { kind: "reorder", from: 5, to: 4 });
    expect(uids(back.line)).toEqual(["l0", "l1", "l3", "l4", "b0"]);
    expect(uids(back.bench)).toEqual(["b1", "l2"]);
    const within = apply(s, { kind: "reorder", from: 6, to: 5 });
    expect(uids(within.bench)).toEqual(["b1", "b0"]);
    expect(() => apply(s, { kind: "reorder", from: 0, to: 8 })).toThrow(MvpDecisionError);
    expect(() => apply(s, { kind: "reorder", from: 7, to: 0 })).toThrow(MvpDecisionError);
    // A full bench: `to` always holds a unit, so a line unit swaps in.
    const full = run(fullLine(), [unit(5, "b0"), unit(6, "b1"), unit(7, "b2")]);
    expect(uids(apply(full, { kind: "reorder", from: 0, to: 7 }).bench)).toEqual(["b0", "b1", "l0"]);
  });

  it("fuse: line + bench stands in the line slot; bench + bench stays on the bench", () => {
    const line = [unit(0, "l0"), unit(1, "l1", 3), unit(2, "l2")];
    const s = run(line, [unit(5, "b0"), unit(6, "b1", 3), unit(7, "b2", 3)]);
    for (const [first, second] of [[6, 1], [1, 6]] as const) {
      const f = apply(s, { kind: "fuse", first, second });
      expect(f.line[1]).toMatchObject({ kind: "fused", uid: first === 1 ? "l1" : "b1" });
      expect(uids(f.bench)).toEqual(["b0", "b2"]);
      expect(f.line).toHaveLength(3);
    }
    const bb = apply(s, { kind: "fuse", first: 7, second: 6 });
    expect(uids(bb.line)).toEqual(["l0", "l1", "l2"]);
    expect(bb.bench.map((u) => [u.uid, u.kind])).toEqual([["b0", "unit"], ["b2", "fused"]]);
    expect(bb.bench[1]!.fusion).toMatchObject({ first: "u7", second: "u6" });
  });

  it("only the line fights, and the bench persists into the next round", () => {
    const s = run([unit(0, "l0")], [unit(5, "b0", 2)]);
    const ghost = synthGhost({ content, round: 1, seed: 5, ghostId: "g", createdAt: day.startedAt });
    const step = applyMvpDecision(s, { kind: "fight" }, content, { fight: { ghost, battleId: "b", battleSeed: 11, at: day.startedAt } });
    expect(step.battle!.teamA.map((u) => u.uid)).toEqual(["l0"]);
    expect(step.state.round).toBe(2);
    expect(step.state.bench).toEqual(s.bench);
  });

  it("an old run (no benchSize, no bench field) has no bench: a buy with a full line says the line is full", () => {
    const { benchSize: _, ...oldRules } = MVP_RULES;
    const { bench: _b, ...stored } = { ...run(fullLine()), rules: oldRules, offers: offer("u5") };
    const old = stored as MvpRunState;
    expect(() => apply(old, { kind: "buy", slot: 0 })).toThrow(/the line is full/);
    expect(() => apply(old, { kind: "reorder", from: 0, to: 5 })).toThrow(MvpDecisionError);
    expect(runView(old).bench).toEqual([]);
    expect(apply(old, { kind: "sell", index: 0 }).bench).toEqual([]);
  });
});
