import { describe, expect, test } from "vitest";
import { compile, evalNum, expressionProblem, makeEnv, MAX_DEPTH, MAX_SRC } from "./grammar";

const ev = (src: string | number, over = {}) => compile(src)(makeEnv(over));

describe("sigil grammar: what parses", () => {
  test("numbers, operators and precedence", () => {
    expect(ev("1+2*3")).toBe(7);
    expect(ev("(1+2)*3")).toBe(9);
    expect(ev("2^3")).toBe(8);
    expect(ev("2*3^2")).toBe(18);
    expect(ev("2^-1")).toBe(0.5);
  });
  test("variables, constants, functions", () => {
    expect(ev("hp*pwr", { hp: 0.5, pwr: 4 })).toBe(2);
    expect(ev("pi")).toBeCloseTo(Math.PI);
    expect(ev("tau")).toBeCloseTo(Math.PI * 2);
    expect(ev("clamp(5,0,1)")).toBe(1);
    expect(ev("mix(0,10,0.25)")).toBe(2.5);
    expect(ev("step(charge,i+1)", { charge: 2, i: 1 })).toBe(1);
    expect(ev("pick(i,-0.12,0.12)", { i: 1 })).toBe(0.12);
    expect(ev("pick(i,-0.12,0.12)", { i: 9 })).toBe(0.12);
    expect(ev("fr(1.25)")).toBeCloseTo(0.25);
    expect(ev("sqrt(-4)")).toBe(0);
    expect(ev("-x_" .replace("x_", "hp"), { hp: 2 })).toBe(-2);
  });
  test("a literal number needs no parsing; a non-finite one is 0", () => {
    expect(ev(3)).toBe(3);
    expect(ev(Infinity)).toBe(0);
    expect(ev(NaN)).toBe(0);
  });
  test("a result that is not finite draws as 0", () => {
    expect(ev("1/0")).toBe(0);
    expect(ev("10^999")).toBe(0);
    expect(ev("0/0")).toBe(0);
  });
  test("compiled once per source", () => {
    expect(compile("sin(t)")).toBe(compile("sin(t)"));
  });
});

describe("sigil grammar: what it refuses", () => {
  const bad = [
    "constructor", "__proto__", "prototype", "toString", "hasOwnProperty", "process", "globalThis", "window", "document", "Math", "Math.PI",
    "constructor(1)", "__proto__(1)", "process.exit(1)", "eval(1)", "Function(1)", "require(1)", "import(1)", "alert(1)",
    "this", "arguments", "t.constructor", "t['constructor']", "t[0]", "(t)(1)", "sin", "sin()", "sin(1,2)", "mix(1,2)", "pick(1)", "clamp(1)",
    "1+", "*2", "(1", "1)", "1 2", "", "  ", ";", "1;2", "a=1", "t==1", "t?1:2", "`1`", "'1'", "{}", "[]", "1,2", "1..2", "0x10", "1_000", "NaN", "Infinity", "undefined", "null", "true",
    "2^3^2", // ^ does not chain
  ];
  test.each(bad)("%j", (src) => {
    expect(expressionProblem(src), src).not.toBeNull();
  });
  test("a non-string, non-number is not an expression", () => {
    for (const v of [null, undefined, {}, [], true, () => 1, Symbol("x")]) expect(expressionProblem(v)).not.toBeNull();
  });
  test("an expression that would reach a global never runs", () => {
    const g = globalThis as Record<string, unknown>;
    g.__sigil_pwned = 0;
    for (const src of ["__sigil_pwned", "globalThis.__sigil_pwned=1", "(globalThis.__sigil_pwned=1)", "constructor.constructor('return 1')()"]) expect(() => compile(src)).toThrow();
    expect(g.__sigil_pwned).toBe(0);
    delete g.__sigil_pwned;
  });
  test("deep nesting is refused, not a stack overflow", () => {
    expect(expressionProblem("(".repeat(MAX_DEPTH + 1) + "1" + ")".repeat(MAX_DEPTH + 1))).toMatch(/nested/);
    expect(expressionProblem("(".repeat(5000) + "1" + ")".repeat(5000))).not.toBeNull();
    expect(expressionProblem("-".repeat(5000) + "1")).not.toBeNull();
    expect(expressionProblem("sin(".repeat(100) + "1" + ")".repeat(100))).not.toBeNull();
    expect(expressionProblem("(".repeat(MAX_DEPTH - 1) + "1" + ")".repeat(MAX_DEPTH - 1))).toBeNull();
  });
  test("a long expression is refused", () => {
    expect(expressionProblem("1+".repeat(MAX_SRC) + "1")).toMatch(/longer/);
    expect(expressionProblem("1+".repeat(60) + "1")).toMatch(/tokens/);
  });
  test("a literal that is not a finite number is refused", () => {
    expect(expressionProblem("1e999")).toMatch(/bad number/);
    expect(ev("99999999999999999999999")).toBeGreaterThan(0);
  });
  test("a failed parse stays failed (cached) and still throws", () => {
    expect(() => compile("nope")).toThrow();
    expect(() => compile("nope")).toThrow();
  });
});

describe("evalNum", () => {
  test("absent and unparseable fields take the default, never throw", () => {
    const env = makeEnv();
    expect(evalNum(undefined, env, 7)).toBe(7);
    expect(evalNum(null, env, 7)).toBe(7);
    expect(evalNum("constructor", env, 7)).toBe(7);
    expect(evalNum({}, env, 7)).toBe(7);
    expect(evalNum("1+1", env, 7)).toBe(2);
  });
});
