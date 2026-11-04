import { describe, expect, it } from "vitest";
import { DEFAULT_RUN_POOL, stressAbilities, stressRegistry } from "../../src/index";
import { MVP_RULES, type MvpContent, type PlayerRef, type UnitContent } from "../../src/mvp/contract";
import { applyMvpDecision, initMvpRun, runView } from "../../src/mvp/run";
import { predictRun } from "./predict";

const units: UnitContent[] = DEFAULT_RUN_POOL.map((d, i) => {
  const form = { when: d.triggers ?? [], who: d.selectors ?? [], does: d.abilities ?? [] };
  return { id: `u${i}`, name: d.name, emoji: "x", archetype: "x", tier: 1, base: d.base, forms: { sleeping: form, awoken: form } };
});
const content: MvpContent = { version: "t", units, abilities: stressAbilities, statuses: stressRegistry };
const me: PlayerRef = { id: "p", name: "me", bot: false };
const fresh = () => initMvpRun({ runId: "r", player: me, seed: 7, content, day: 1, startedAt: "2026-11-04T00:00:00.000Z" });

/** The run as the client has it: the public view only. */
const view = (s: ReturnType<typeof fresh>) => runView(s);
/** A unit's uid is the server's; the table's own is a stand-in until the answer. */
const noUids = (r: ReturnType<typeof runView>) => ({ ...r, line: r.line.map((u) => ({ ...u, uid: "" })), bench: r.bench.map((u) => ({ ...u, uid: "" })) });

describe("what the table shows before the server answers", () => {
  it("a buy is the server's buy: the unit in the line, gold down, the offer gone and the rest renumbered", () => {
    const s = fresh();
    const d = { kind: "buy", slot: 1 } as const;
    const predicted = predictRun(view(s), d, content, MVP_RULES)!;
    const real = view(applyMvpDecision(s, d, content).state);
    expect(noUids(predicted)).toEqual(noUids(real));
    expect(predicted.line).toHaveLength(1);
    expect(predicted.gold).toBe(s.gold - s.offers[1]!.cost);
    expect(predicted.offers.map((o) => o.slot)).toEqual(predicted.offers.map((_, i) => i));
  });

  it("a sell, a move and a lock are exact too", () => {
    let s = fresh();
    // Two different units (a second copy would merge into the first).
    while (s.line.length < 2) s = applyMvpDecision(s, { kind: "buy", slot: s.offers.findIndex((o) => !s.line.some((u) => u.unitId === o.unitId)) }, content).state;
    for (const d of [{ kind: "sell", index: 0 }, { kind: "reorder", from: 0, to: 1 }, { kind: "lock", slot: 0 }] as const) {
      expect(noUids(predictRun(view(s), d, content, MVP_RULES)!)).toEqual(noUids(view(applyMvpDecision(s, d, content).state)));
    }
  });

  it("says nothing for what needs the server: a reroll, a fight, a refused buy", () => {
    const s = fresh();
    expect(predictRun(view(s), { kind: "reroll" }, content, MVP_RULES)).toBeNull();
    expect(predictRun(view(s), { kind: "fight" }, content, MVP_RULES)).toBeNull();
    expect(predictRun({ ...view(s), gold: 0 }, { kind: "buy", slot: 0 }, content, MVP_RULES)).toBeNull();
  });

  it("two predicted units never share a uid", () => {
    const s = fresh();
    const a = predictRun(view(s), { kind: "buy", slot: 0 }, content, MVP_RULES)!;
    const b = predictRun(a, { kind: "buy", slot: 0 }, content, MVP_RULES)!;
    expect(new Set(b.line.map((u) => u.uid)).size).toBe(b.line.length);
  });
});
