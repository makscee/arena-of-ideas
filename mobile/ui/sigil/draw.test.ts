import { describe, expect, test } from "vitest";
import duel from "./duel-specs.json";
import { drawSigil } from "./draw";
import { FALLBACK_SPEC } from "./fallback";
import { MAX_SHAPES, shapeCount, type Spec } from "./spec";

// A canvas context that records calls and never draws. Pixel checks run in the browser e2e.
function fakeCtx() {
  const calls: string[] = [];
  const props: Record<string, number> = { globalAlpha: 1 };
  const ctx = new Proxy({}, {
    get(_t, k: string) {
      if (k === "calls") return calls;
      if (k === "globalAlpha") return props.globalAlpha;
      if (k === "createLinearGradient" || k === "createRadialGradient") return () => ({ addColorStop() {} });
      return (...a: unknown[]) => { calls.push(`${k}(${a.map((x) => (typeof x === "number" && !Number.isFinite(x) ? "BAD" : "")).join("")})`); };
    },
    set(_t, k: string, v) { if (typeof v === "number" && !Number.isFinite(v)) calls.push(`BAD ${k}`); if (k === "globalAlpha") props[k] = v; return true; },
  });
  return ctx as unknown as CanvasRenderingContext2D & { calls: string[] };
}

describe("drawSigil", () => {
  test("draws every duel spec and the fallback without a bad number reaching the canvas", () => {
    const states = [{}, { aw: 1, charge: 3, shield: 3, hp: 0.2 }, { hit: 1, fire: 1, die: 0.6 }, { t: 12.5, team: "them" as const }];
    for (const [name, s] of [...Object.entries(duel), ["fallback", FALLBACK_SPEC]] as [string, Spec][]) {
      for (const st of states) {
        const ctx = fakeCtx();
        const n = drawSigil(ctx, s, { S: 64, ...st });
        expect(n, name).toBeGreaterThan(0);
        expect(n, name).toBeLessThanOrEqual(MAX_SHAPES);
        expect(ctx.calls.filter((c) => c.includes("BAD")), name).toEqual([]);
      }
    }
  });
  test("what it draws matches what validate counted (busiest sampled state)", () => {
    for (const [name, s] of Object.entries(duel) as [string, Spec][]) {
      const drawn = Math.max(
        ...[{}, { aw: 1, charge: 3, hp: 0.3, t: 0.33 }, { hit: 1, fire: 1, t: 1.1 }, { shield: 3, pwr: 9, t: 2.2 }, { aw: 1, charge: 3, shield: 3, hit: 1, fire: 1, die: 0.5, hp: 0, pwr: 9, t: 3.7 }].map((st) => drawSigil(fakeCtx(), s, { S: 60, frame: false, ...st })),
      );
      expect(drawn, name).toBeLessThanOrEqual(shapeCount(s));
    }
  });
  test("the caps hold even for a spec that never went through validate", () => {
    const over = { body: Array.from({ length: 10 }, () => ({ shape: "circle", count: 16 })) } as unknown as Spec;
    expect(drawSigil(fakeCtx(), over, { S: 64 })).toBe(MAX_SHAPES);
    const nested = { body: [{ shape: "group", count: 99999, layers: [{ shape: "group", count: 99999, layers: [{ shape: "circle", count: 99999 }] }] }] } as unknown as Spec;
    expect(drawSigil(fakeCtx(), nested, { S: 64 })).toBe(MAX_SHAPES);
  });
  test("a hostile spec draws what it can and never throws", () => {
    const evil = {
      body: [
        { shape: "circle", r: "constructor", x: "process", fill: "__proto__", count: "globalThis" },
        { shape: "heart" }, null, 5, "x",
        { shape: "group", layers: "x" },
        { shape: "group", layers: [null, { shape: "circle", r: "(((((((((((((1)))))))))))))" }] },
        { shape: "circle", r: { valueOf() { throw new Error("x"); } } },
      ],
      mark: "no",
    } as unknown as Spec;
    expect(() => drawSigil(fakeCtx(), evil, { S: 64 })).not.toThrow();
    expect(() => drawSigil(fakeCtx(), null as unknown as Spec, { S: 64 })).not.toThrow();
    expect(() => drawSigil(fakeCtx(), {} as Spec, { S: NaN, t: Infinity, hp: NaN, hit: -5, fire: 1e9 })).not.toThrow();
  });
  test("state outside its range is clamped, not drawn", () => {
    const ctx = fakeCtx();
    drawSigil(ctx, FALLBACK_SPEC, { S: 64, hit: 1e9, fire: -1e9, die: NaN, aw: 7, hp: -3, charge: 1e9, shield: Infinity, t: NaN });
    expect(ctx.calls.filter((c) => c.includes("BAD"))).toEqual([]);
  });
  test("frame:false draws the bare spec: no stand, halo, ring or shards", () => {
    const bare = fakeCtx(), framed = fakeCtx();
    drawSigil(bare, FALLBACK_SPEC, { S: 64, frame: false, aw: 1, fire: 1, die: 0.5, hit: 1 });
    drawSigil(framed, FALLBACK_SPEC, { S: 64, aw: 1, fire: 1, die: 0.5, hit: 1 });
    expect(framed.calls.length).toBeGreaterThan(bare.calls.length + 40);
  });
  test("the same spec reads for either side", () => {
    expect(drawSigil(fakeCtx(), FALLBACK_SPEC, { S: 64, team: "them" })).toBe(drawSigil(fakeCtx(), FALLBACK_SPEC, { S: 64, team: "you" }));
  });
});
