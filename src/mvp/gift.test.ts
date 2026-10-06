// The awakening gift (round 3, R3-15; docs/round3/README.md, "Maks's answers"):
// the copy that awakens a unit offers a free pick of 1 of 3 units from the
// highest tier open that round. While it waits, only sell, reorder and the
// pick (or skip) are allowed.
import { describe, expect, it } from "vitest";
import { DEFAULT_RUN_POOL, stressAbilities, stressRegistry } from "../index.js";
import { MVP_RULES, type Decision, type LineUnit, type MvpContent, type Offer, type PlayerRef, type Tier, type UnitContent } from "./contract.js";
import { lineUnitOf } from "./forms.js";
import { abandonRun, applyMvpDecision, checkDecision, initMvpRun, MvpBadDecision, MvpDecisionError, runView, synthGhost, type MvpRunState } from "./run.js";

// Twelve units: u0–u5 tier 1, u6–u8 tier 2, u9–u11 tier 3 (no tier 4).
const tierOf = (i: number): Tier => (i < 6 ? 1 : i < 9 ? 2 : 3) as Tier;
const units: UnitContent[] = Array.from({ length: 12 }, (_, i) => {
  const d = DEFAULT_RUN_POOL[i % DEFAULT_RUN_POOL.length]!;
  const form = { when: d.triggers ?? [], who: d.selectors ?? [], does: d.abilities ?? [] };
  return { id: `u${i}`, name: `${d.name} ${i}`, emoji: "x", tier: tierOf(i), base: d.base, forms: { sleeping: form, awoken: form } };
});
const content: MvpContent = { version: "t", units, abilities: stressAbilities, statuses: stressRegistry };
const me: PlayerRef = { id: "p", name: "me", bot: false };
const day = { day: 1, startedAt: "2026-10-05T00:00:00.000Z" };
const fuse = { name: "Test", discoveredBy: me };

const unit = (i: number, uid: string, copies = 1): LineUnit => lineUnitOf(units[i]!, uid, copies);
const run = (line: LineUnit[], bench: LineUnit[] = [], round = 1, seed = 3): MvpRunState => ({ ...initMvpRun({ runId: "r", player: me, seed, content, ...day }), round, gold: 30, line, bench });
const offers = (...ids: string[]): Offer[] => ids.map((unitId, slot) => ({ slot, unitId, tier: 1, cost: 3 }));
const apply = (s: MvpRunState, d: Decision) => applyMvpDecision(s, d, content, { fuse }).state;
/** A run whose u0 sits at 2 copies, with u0 on offer: buying slot 0 awakens it. */
const ready = (round = 1, line = [unit(0, "a", 2)], bench: LineUnit[] = []) => ({ ...run(line, bench, round), offers: offers("u0", "u1") });

describe("MVP awakening gift (R3-15)", () => {
  it("the copy that awakens a unit offers 3 different units from the highest tier open that round", () => {
    expect(MVP_RULES.giftChoices).toBe(3);
    for (const [round, tier] of [[1, 1], [3, 2], [6, 3], [9, 3]] as const) {
      const s = apply(ready(round), { kind: "buy", slot: 0 });
      expect(s.line[0]!.form).toBe("awoken");
      expect(s.gift).toHaveLength(3);
      expect(new Set(s.gift).size).toBe(3);
      for (const id of s.gift!) expect(units.find((u) => u.id === id)!.tier).toBe(tier);
      expect(runView(s).gift).toEqual(s.gift);
    }
  });

  it("only the awakening copy gives one: not a first buy, not a 2nd copy, not a 4th, not a fused unit's copy", () => {
    expect(apply({ ...run([]), offers: offers("u0") }, { kind: "buy", slot: 0 }).gift).toBeUndefined();
    expect(apply({ ...run([unit(0, "a")]), offers: offers("u0") }, { kind: "buy", slot: 0 }).gift).toBeUndefined();
    expect(apply({ ...run([unit(0, "a", 3)]), offers: offers("u0") }, { kind: "buy", slot: 0 }).gift).toBeUndefined();
    const fused = apply(run([unit(0, "a", 3), unit(1, "b", 3)]), { kind: "fuse", first: 0, second: 1 });
    expect(apply({ ...fused, offers: offers("u0") }, { kind: "buy", slot: 0 }).gift).toBeUndefined();
  });

  it("awakening on the bench gives one too", () => {
    const line = [1, 2, 3, 4, 5].map((i) => unit(i, `l${i}`));
    const s = apply(ready(1, line, [unit(0, "b", 2)]), { kind: "buy", slot: 0 });
    expect(s.bench[0]!.form).toBe("awoken");
    expect(s.gift).toHaveLength(3);
  });

  it("a run whose rules have no gift (stored before it) gets none", () => {
    const { giftChoices: _g, ...old } = MVP_RULES;
    const s = apply({ ...ready(), rules: old }, { kind: "buy", slot: 0 });
    expect(s.line[0]!.form).toBe("awoken");
    expect(s.gift).toBeUndefined();
  });

  it("while it waits, buy, reroll, lock, fuse and fight are refused with a reason; sell and reorder are allowed", () => {
    const s = apply(ready(1, [unit(0, "a", 2), unit(1, "b", 3), unit(2, "c", 3)]), { kind: "buy", slot: 0 });
    expect(s.gift).toBeDefined();
    const ghost = synthGhost({ content, round: 1, seed: 5, ghostId: "g", createdAt: day.startedAt });
    const ctx = { fuse, fight: { ghost, battleId: "b", battleSeed: 1, at: day.startedAt } };
    for (const d of [{ kind: "buy", slot: 0 }, { kind: "reroll" }, { kind: "lock", slot: 0 }, { kind: "fuse", first: 1, second: 2 }, { kind: "fight" }] as Decision[])
      expect(() => applyMvpDecision(s, d, content, ctx), d.kind).toThrow(/pick your awakening gift first, or skip it/);
    const sold = apply(s, { kind: "sell", index: 2 });
    expect(sold.line).toHaveLength(2);
    expect(sold.gift).toEqual(s.gift);
    const moved = apply(s, { kind: "reorder", from: 1, to: 0 });
    expect(moved.line.map((u) => u.uid)).toEqual(["b", "a", "c"]);
    expect(moved.gift).toEqual(s.gift);
  });

  it("a pick joins the line for free and clears the gift; null skips it", () => {
    const s = apply(ready(), { kind: "buy", slot: 0 });
    const gold = s.gold;
    const picked = apply(s, { kind: "gift", pick: 1 });
    expect(picked.gift).toBeUndefined();
    expect(picked.gold).toBe(gold);
    expect(picked.line).toHaveLength(2);
    expect(picked.line[1]!.unitId).toBe(s.gift![1]);
    expect(picked.line[1]!.uid).not.toBe(picked.line[0]!.uid);
    const skipped = apply(s, { kind: "gift", pick: null });
    expect(skipped.gift).toBeUndefined();
    expect(skipped.line).toEqual(s.line);
    // After the pick the shop opens again.
    expect(apply(picked, { kind: "buy", slot: 0 }).line).toHaveLength(3);
  });

  it("a pick goes to the bench when the line is full; with both full it is refused (make room), and selling makes room", () => {
    const line = [0, 1, 2, 3, 4].map((i) => unit(i, `l${i}`, i === 0 ? 2 : 1));
    // Tier 1 at round 1: u0–u5. Keep the gift off units the board already has.
    const s = { ...apply({ ...ready(1, line), rules: { ...MVP_RULES } }, { kind: "buy", slot: 0 }), gift: ["u6", "u7", "u8"] };
    const toBench = apply(s, { kind: "gift", pick: 0 });
    expect(toBench.bench.map((u) => u.unitId)).toEqual(["u6"]);
    const full = { ...s, bench: [unit(9, "b0"), unit(10, "b1"), unit(11, "b2")] };
    expect(() => apply(full, { kind: "gift", pick: 0 })).toThrow(/make room/);
    expect(apply(full, { kind: "gift", pick: null }).gift).toBeUndefined();
    const roomy = apply(full, { kind: "sell", index: 7 });
    expect(apply(roomy, { kind: "gift", pick: 2 }).bench.map((u) => u.unitId)).toEqual(["u9", "u10", "u8"]);
  });

  it("a picked copy merges like a buy, even with line and bench full, and an awakening pick gives another gift", () => {
    const line = [0, 1, 2, 3, 4].map((i) => unit(i, `l${i}`, i === 0 ? 3 : i === 1 ? 2 : 1));
    const bench = [unit(9, "b0"), unit(10, "b1"), unit(11, "b2")];
    const s: MvpRunState = { ...run(line, bench), gift: ["u6", "u2", "u1"] };
    const merged = apply(s, { kind: "gift", pick: 1 });
    expect(merged.line[2]!.copies).toBe(2);
    expect(merged.gift).toBeUndefined();
    const awoke = apply(s, { kind: "gift", pick: 2 });
    expect(awoke.line[1]!.form).toBe("awoken");
    expect(awoke.gift).toHaveLength(3);
  });

  it("a pick past the choices, or a gift decision with none waiting, is refused (409); a bad pick is unreadable (400)", () => {
    const s = apply(ready(), { kind: "buy", slot: 0 });
    expect(() => apply(s, { kind: "gift", pick: 3 })).toThrow(MvpDecisionError);
    expect(() => apply(run([]), { kind: "gift", pick: 0 })).toThrow(/no awakening gift/);
    expect(() => apply(run([]), { kind: "gift", pick: null })).toThrow(MvpDecisionError);
    for (const pick of [-1, 1.5, "0", undefined, "__proto__"]) {
      const d = { kind: "gift", pick } as unknown as Decision;
      expect(() => checkDecision(d), String(pick)).toThrow(MvpBadDecision);
      expect(() => apply(s, d)).toThrow(MvpBadDecision);
    }
    expect(() => checkDecision({ kind: "gift", pick: null })).not.toThrow();
  });

  it("the gift is drawn from the run's seed: the same run gives the same gift", () => {
    const a = apply(ready(6, undefined), { kind: "buy", slot: 0 });
    const b = apply(ready(6, undefined), { kind: "buy", slot: 0 });
    expect(a.gift).toEqual(b.gift);
  });

  it("giving up drops a waiting gift", () => {
    const s = apply(ready(), { kind: "buy", slot: 0 });
    expect(s.gift).toBeDefined();
    const over = abandonRun(s);
    expect(over.phase).toBe("over");
    expect(over.gift).toBeUndefined();
  });
});
