import { describe, expect, test } from "vitest";
import duel from "./duel-specs.json";
import { FALLBACK_SPEC } from "./fallback";
import { specOrFallback } from "./index";
import { isPaletteName, color, PALETTE_NAMES } from "./palette";
import { MAX_LAYERS, MAX_SHAPES, shapeCount, validate, type Layer, type Spec } from "./spec";

const ok = (s: unknown) => validate(s);
const body = (...layers: unknown[]) => ({ body: layers });

describe("validate: good specs", () => {
  test.each(Object.entries(duel))("the duel's %s passes", (_name, spec) => {
    const v = validate(spec);
    expect(v.problems).toEqual([]);
    expect(v.ok).toBe(true);
  });
  test("the fallback passes", () => {
    expect(validate(FALLBACK_SPEC)).toEqual({ ok: true, problems: [] });
  });
  test("a mark is optional; every shape is known", () => {
    for (const shape of ["circle", "ring", "ellipse", "rect", "poly", "star", "petal", "line", "arc"]) expect(ok(body({ shape })).ok, shape).toBe(true);
    expect(ok(body({ shape: "group", layers: [{ shape: "circle" }] })).ok).toBe(true);
  });
});

describe("validate: schema", () => {
  test("not an object, no body, empty body", () => {
    for (const s of [null, undefined, 1, "x", [], [{ shape: "circle" }], {}, { body: [] }, { body: {} }, { body: "x" }, { mark: [{ shape: "circle" }] }]) expect(ok(s).ok, JSON.stringify(s)).toBe(false);
  });
  test("unknown top-level and layer keys", () => {
    expect(ok({ ...body({ shape: "circle" }), evil: 1 }).problems.join()).toMatch(/unknown key evil/);
    expect(ok(body({ shape: "circle", onclick: "x" })).problems.join()).toMatch(/unknown key onclick/);
  });
  test("unknown shape, missing shape, non-object layer", () => {
    expect(ok(body({ shape: "heart" })).problems.join()).toMatch(/unknown shape heart/);
    expect(ok(body({})).ok).toBe(false);
    expect(ok(body("circle")).ok).toBe(false);
    expect(ok(body(null)).ok).toBe(false);
    expect(ok(body({ shape: "constructor" })).ok).toBe(false);
    expect(ok(body({ shape: "__proto__" })).ok).toBe(false);
  });
  test("only a group has layers; layers is a list", () => {
    expect(ok(body({ shape: "circle", layers: [] })).ok).toBe(false);
    expect(ok(body({ shape: "group", layers: "x" })).ok).toBe(false);
  });
  test("blend and note", () => {
    expect(ok(body({ shape: "circle", blend: "add" })).ok).toBe(true);
    expect(ok(body({ shape: "circle", blend: "screen" })).ok).toBe(false);
    expect(ok(body({ shape: "circle", note: "a bell" })).ok).toBe(true);
    expect(ok(body({ shape: "circle", note: "x".repeat(500) })).ok).toBe(false);
    expect(ok(body({ shape: "circle", note: 5 })).ok).toBe(false);
  });
});

describe("validate: hostile specs", () => {
  test("a __proto__ key from JSON.parse is an unknown key, and does not pollute", () => {
    const evil = JSON.parse('{"body":[{"shape":"circle","__proto__":{"polluted":1}}],"__proto__":{"x":1}}');
    const v = validate(evil);
    expect(v.ok).toBe(false);
    expect(v.problems.join()).toMatch(/__proto__/);
    expect(({} as Record<string, unknown>).polluted).toBeUndefined();
  });
  test("names that live on every object are not palette colours", () => {
    for (const name of ["constructor", "__proto__", "toString", "hasOwnProperty", "valueOf", "prototype"]) {
      expect(isPaletteName(name), name).toBe(false);
      expect(color(name, "you")).toBeNull();
      expect(ok(body({ shape: "circle", fill: name })).ok, name).toBe(false);
      expect(ok(body({ shape: "line", stroke: name })).ok, name).toBe(false);
    }
  });
  test("expressions that reach for the host", () => {
    for (const x of ["constructor", "__proto__", "process", "process.exit(1)", "globalThis", "eval('1')", "Function('return 1')()", "foo(1)", "sin(process)"]) {
      expect(ok(body({ shape: "circle", r: x })).ok, x).toBe(false);
      expect(ok(body({ shape: "circle", count: x })).ok, x).toBe(false);
      expect(ok(body({ shape: "group", x, layers: [{ shape: "circle" }] })).ok, x).toBe(false);
    }
  });
  test("an expression field that is not a number or a string", () => {
    for (const v of [null, true, {}, [], [1], () => 1]) expect(ok(body({ shape: "circle", r: v })).ok).toBe(false);
    expect(ok(body({ shape: "circle", r: Infinity })).ok).toBe(false);
    expect(ok(body({ shape: "circle", r: NaN })).ok).toBe(false);
  });
  test("deep nesting of expressions and of groups", () => {
    expect(ok(body({ shape: "circle", r: "(".repeat(2000) + "1" + ")".repeat(2000) })).ok).toBe(false);
    let g: Layer = { shape: "circle" };
    for (let i = 0; i < 2000; i++) g = { shape: "group", layers: [g] };
    expect(ok({ body: [g] }).ok).toBe(false);
  });
  test("huge counts", () => {
    expect(ok(body({ shape: "circle", count: 1e9 })).problems.join()).toMatch(/outside 1–16/);
    expect(ok(body({ shape: "circle", count: 17 })).ok).toBe(false);
    expect(ok(body({ shape: "circle", count: 0 })).ok).toBe(false);
    expect(ok(body({ shape: "circle", count: 16 })).ok).toBe(true);
    // an expression count is clamped at draw time and counted against the budget here
    expect(shapeCount(body({ shape: "circle", count: "99999" }) as Spec)).toBe(16);
    expect(ok(body({ shape: "circle", count: "99999" })).ok).toBe(true);
    expect(ok(body(...[1, 2, 3, 4].map(() => ({ shape: "circle", count: "99999" })))).problems.join()).toMatch(/shapes drawn/);
  });
  test("nested counts multiply: 16 x 16 groups is over the budget", () => {
    const v = ok(body({ shape: "group", count: 16, layers: [{ shape: "circle", count: 16 }] }));
    expect(v.ok).toBe(false);
    expect(v.problems.join()).toMatch(/shapes drawn \(limit 48\)/);
  });
  test("too many layers is refused before anything is walked", () => {
    const many = Array.from({ length: 100000 }, () => ({ shape: "circle" }));
    const t = Date.now();
    expect(ok({ body: many }).ok).toBe(false);
    expect(ok({ body: [{ shape: "circle" }], mark: many }).ok).toBe(false);
    expect(Date.now() - t).toBeLessThan(500);
    expect(MAX_LAYERS * 2).toBeGreaterThan(0);
  });
  test("validate never throws on garbage", () => {
    const loop: Record<string, unknown> = { shape: "group" };
    loop.layers = [loop];
    for (const s of [loop, { body: [loop] }, { body: new Proxy({}, { get() { throw new Error("x"); } }) }, Object.create(null), new Date(), /x/, Symbol.iterator]) {
      expect(() => validate(s)).not.toThrow();
    }
    expect(validate({ body: [loop] }).ok).toBe(false);
  });
});

describe("validate: palette", () => {
  test("named colours, owner colours and gradients of them", () => {
    for (const c of PALETTE_NAMES) expect(ok(body({ shape: "circle", fill: c, stroke: c })).ok, c).toBe(true);
    expect(ok(body({ shape: "circle", fill: { grad: ["owner", "owner.dark"] } })).ok).toBe(true);
    expect(ok(body({ shape: "circle", fill: { radial: ["paper", "bone"] } })).ok).toBe(true);
  });
  test("anything else", () => {
    for (const c of ["#fff", "red", "rgb(0,0,0)", "owner.darker", "url(x)", "", "Gold"]) expect(ok(body({ shape: "circle", fill: c })).ok, c).toBe(false);
    for (const f of [{ grad: ["owner"] }, { grad: ["owner", "red"] }, { grad: ["owner", "gold", "ink"] }, { grad: "owner" }, { solid: ["owner", "gold"] }, { grad: ["a", "b"], radial: ["a", "b"] }, 5, true, null, []]) {
      expect(ok(body({ shape: "circle", fill: f })).ok, JSON.stringify(f)).toBe(false);
    }
    expect(ok(body({ shape: "line", stroke: { grad: ["owner", "gold"] } })).ok).toBe(false);
  });
  test("the same name is a different colour per side", () => {
    expect(color("owner", "you")).not.toBe(color("owner", "them"));
    expect(color("gold", "you")).toBe(color("gold", "them"));
  });
});

describe("validate: the shape budget", () => {
  test("48 is fine, 49 is not", () => {
    const n = (k: number) => body({ shape: "circle", count: 16 }, { shape: "circle", count: 16 }, { shape: "circle", count: 16 }, ...(k ? [{ shape: "circle", count: k }] : []));
    expect(ok(n(0)).ok).toBe(true);
    expect(ok(n(1)).problems.join()).toMatch(/49 shapes/);
  });
  test("body and mark share the budget", () => {
    const s = { body: [{ shape: "circle", count: 16 }, { shape: "circle", count: 16 }], mark: [{ shape: "circle", count: 16 }, { shape: "circle" }] };
    expect(ok(s).ok).toBe(false);
  });
  test("a group is not itself a shape; its children are counted per copy", () => {
    expect(shapeCount(body({ shape: "group", count: 3, layers: [{ shape: "circle" }, { shape: "circle" }] }) as Spec)).toBe(6);
  });
  test("a layer that is invisible costs nothing", () => {
    expect(shapeCount(body({ shape: "circle", count: 16, alpha: 0 }) as Spec)).toBe(0);
  });
  test("a count that depends on state is counted at its busiest sampled state", () => {
    expect(shapeCount(body({ shape: "circle", count: "2+aw" }) as Spec)).toBe(3);
    expect(shapeCount(body({ shape: "circle", count: "1+charge" }) as Spec)).toBe(4);
  });
  test("the duel's specs stay within the budget", () => {
    for (const [name, s] of Object.entries(duel)) expect(shapeCount(s as unknown as Spec), name).toBeLessThanOrEqual(MAX_SHAPES);
  });
});

describe("specOrFallback", () => {
  test("a valid spec is kept; anything else is the generic sigil", () => {
    expect(specOrFallback(duel.Medic)).toBe(duel.Medic);
    expect(specOrFallback(undefined)).toBe(FALLBACK_SPEC);
    expect(specOrFallback({ body: [{ shape: "nope" }] })).toBe(FALLBACK_SPEC);
  });
});
