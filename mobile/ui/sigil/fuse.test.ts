// Slice F of mission arena#34: the 3rd copy (`aw`) grows the sigil, and a fusion
// draws one unit's body with the other's mark. Visual only: the fusion's rule,
// name and stats are not read from here.
import { beforeAll, describe, expect, test, vi } from "vitest";

vi.mock("../dom", () => ({ h: () => ({}) }));
import { PORTRAITS } from "../../../src/mvp/portraits";
import type { LineUnit, MvpContent, UnitContent } from "../../../src/mvp/contract";
import { drawSigil } from "./draw";
import { FALLBACK_SPEC } from "./fallback";
import { fuse } from "./fuse";
import { MAX_SHAPES, shapeCount, validate, type Layer, type Spec } from "./spec";

/** A context that records every call with its numbers, so two drawings can be told apart. */
function recorder() {
  const log: string[] = [];
  const ctx = new Proxy({}, {
    get(_t, k: string) {
      if (k === "log") return log;
      if (k === "createLinearGradient" || k === "createRadialGradient") return () => ({ addColorStop() {} });
      return (...a: unknown[]) => { log.push(`${k}(${a.map((x) => (typeof x === "number" ? x.toFixed(4) : String(x))).join(",")})`); };
    },
    set(_t, k: string, v) { log.push(`${k}=${typeof v === "number" ? v.toFixed(4) : String(v)}`); return true; },
  });
  return ctx as unknown as CanvasRenderingContext2D & { log: string[] };
}
const draw = (spec: Spec, o: { aw?: number; frame?: boolean; t?: number } = {}): string => {
  const ctx = recorder();
  drawSigil(ctx, spec, { S: 96, t: 0.37, hp: 0.5, charge: 2, shield: 1, frame: false, ...o }); // hp 0.5: a spec that shows damage is not blank
  return ctx.log.join("\n");
};

const names = Object.keys(PORTRAITS);

describe("the 3rd copy grows the sigil", () => {
  test.each(names)("%s: awoken draws differently from sleeping, spec alone", (name) => {
    const spec = PORTRAITS[name]!;
    expect(draw(spec, { aw: 1 }), "bare spec, awoken").not.toEqual(draw(spec, { aw: 0 }));
  });
  test("with the frame, awoken adds its crown on every sigil, even the generic one", () => {
    expect(draw(FALLBACK_SPEC, { aw: 1, frame: true })).not.toEqual(draw(FALLBACK_SPEC, { aw: 0, frame: true }));
  });
  test("a sleeping unit draws the same whether aw is left out or 0", () => {
    const spec = PORTRAITS[names[0]!]!;
    expect(draw(spec, {})).toEqual(draw(spec, { aw: 0 }));
  });
});

describe("a fusion's sigil", () => {
  const A = PORTRAITS.Hornet!, B = PORTRAITS.Sniper!;
  test("takes the first part's body and the second part's mark", () => {
    const f = fuse(A, B);
    expect(f.body).toEqual(A.body);
    expect(f.mark).toEqual(B.mark);
    expect(validate(f)).toEqual({ ok: true, problems: [] });
  });
  test("uses both parents: it differs from each part, and swapping the order swaps what it takes", () => {
    const f = draw(fuse(A, B), { aw: 1 });
    expect(f).not.toEqual(draw(A, { aw: 1 }));
    expect(f).not.toEqual(draw(B, { aw: 1 }));
    expect(f).not.toEqual(draw(fuse(B, A), { aw: 1 }));
  });
  test("every pair of live units gets a valid portrait within the budget, and the parents are left as they were", () => {
    const frozen = JSON.stringify(PORTRAITS);
    for (const a of names) for (const b of names) {
      const f = fuse(PORTRAITS[a], PORTRAITS[b]);
      expect(validate(f).ok, `${a}+${b}`).toBe(true);
      expect(shapeCount(f), `${a}+${b}`).toBeLessThanOrEqual(MAX_SHAPES);
    }
    expect(JSON.stringify(PORTRAITS)).toEqual(frozen);
  });
  test("a part with no valid spec counts as the generic sigil; a part without a mark gives none", () => {
    expect(fuse(undefined, B).body).toEqual(FALLBACK_SPEC.body);
    expect(fuse(A, { nope: 1 }).mark).toEqual(FALLBACK_SPEC.mark);
    expect(fuse(A, { body: B.body }).mark).toBeUndefined();
  });
  test("a pair over the shape budget keeps the first part's whole portrait", () => {
    const wide: Layer = { shape: "circle", count: 16, r: 0.1, x: "0.01*i" };
    const a: Spec = { body: [wide, wide] }; // 32 shapes
    const b: Spec = { body: [wide], mark: [wide, wide] }; // 48 shapes alone is fine, but 32 (a's body) + 32 (b's mark) is not
    expect(validate(a).ok && validate(b).ok).toBe(true);
    expect(fuse(a, b)).toEqual(a);
  });
});

describe("a fused line unit as a piece", () => {
  // piece.ts pulls in the page's dom helpers (they read #app when they load); lineToPiece needs none of it
  let lineToPiece: typeof import("../piece").lineToPiece;
  beforeAll(async () => {
    ({ lineToPiece } = await import("../piece"));
  });
  const unit = (id: string, name: string): UnitContent =>
    ({ id, name, emoji: "x", tier: 1, base: { pwr: 2, hp: 3 }, forms: { sleeping: { when: [], does: [] }, awoken: { when: [], does: [] } }, portrait: PORTRAITS[name] }) as unknown as UnitContent;
  const content = { units: [unit("a", "Hornet"), unit("b", "Sniper")] } as unknown as Pick<MvpContent, "units">;
  const recipe = { when: [{ kind: "x" }], does: [{ kind: "free-form rule" }] };
  const fused = { uid: "u1", kind: "fused", unitId: "a", name: "Stingshot", emoji: "ab", copies: 1, form: "sleeping", stats: { pwr: 5, hp: 7 }, recipe, fusion: { first: "a", second: "b", name: "Stingshot", discoveredBy: null } } as unknown as LineUnit;

  test("draws fuse(a, b), is marked fused, and leaves the fusion's name, stats and rule alone", () => {
    const before = JSON.stringify(fused);
    const p = lineToPiece(fused, content);
    expect(p.portrait).toEqual(fuse(PORTRAITS.Hornet, PORTRAITS.Sniper));
    expect(p.fused).toBeDefined();
    expect(p.name).toBe("Stingshot");
    expect(p.stats).toEqual({ pwr: 5, hp: 7 });
    expect(JSON.stringify(fused)).toEqual(before);
    expect(fused.recipe).toBe(recipe);
  });
  test("a plain unit draws its own portrait", () => {
    const plain = { ...fused, kind: "unit", fusion: undefined, name: "Hornet", stats: { pwr: 2, hp: 3 } } as unknown as LineUnit;
    expect(lineToPiece(plain, content).portrait).toEqual(PORTRAITS.Hornet);
  });
});
