// The closed expression grammar of a sigil spec. A number in a spec may be a
// literal or a short expression of the unit's state:
//   numbers, + - * / ^, ( ), names from VARS, calls of the FNS below.
// It is parsed by hand into closures, once per source string. Nothing here
// reaches eval, Function or any global: a name is either in VARS/FNS or a
// parse error, and lookups go through Maps, never through object keys.

export const VARS = ["t", "beat", "i", "j", "n", "hp", "pwr", "charge", "shield", "hit", "fire", "die", "aw", "pi", "tau"] as const;
export type VarName = (typeof VARS)[number];
export type Env = Record<VarName, number>;

export const MAX_SRC = 160; // characters in one expression
export const MAX_TOKENS = 80;
export const MAX_DEPTH = 8; // nested brackets / calls / negations

const TAU = Math.PI * 2;
const VAR_SET: ReadonlySet<string> = new Set(VARS);

type Fn = { arity: [min: number, max: number]; call: (...a: number[]) => number };
const FNS: ReadonlyMap<string, Fn> = new Map<string, Fn>([
  ["sin", { arity: [1, 1], call: (x) => Math.sin(x as number) }],
  ["cos", { arity: [1, 1], call: (x) => Math.cos(x as number) }],
  ["abs", { arity: [1, 1], call: (x) => Math.abs(x as number) }],
  ["min", { arity: [2, 6], call: (...a) => Math.min(...a) }],
  ["max", { arity: [2, 6], call: (...a) => Math.max(...a) }],
  ["sqrt", { arity: [1, 1], call: (x) => Math.sqrt(Math.max(0, x as number)) }],
  ["step", { arity: [2, 2], call: (a, b) => ((a as number) >= (b as number) ? 1 : 0) }],
  ["mix", { arity: [3, 3], call: (a, b, k) => (a as number) + ((b as number) - (a as number)) * (k as number) }],
  ["fr", { arity: [1, 1], call: (x) => (x as number) - Math.floor(x as number) }],
  ["clamp", { arity: [3, 3], call: (x, a, b) => Math.min(b as number, Math.max(a as number, x as number)) }],
  ["pick", { arity: [2, 17], call: (i, ...xs) => xs[Math.max(0, Math.min(xs.length - 1, Math.floor(i as number)))] as number }],
]);
export const FUNCTION_NAMES: readonly string[] = [...FNS.keys()];

export class ExprError extends Error {}

type Node = (v: Env) => number;
export type Compiled = (v: Env) => number;

const TOKEN = /\d+\.?\d*(?:e-?\d+)?|\.\d+(?:e-?\d+)?|[a-z_][a-z0-9_]*|[-+*/^(),]|\S/gi;

function parse(src: string): Node {
  if (src.length > MAX_SRC) throw new ExprError(`"${src.slice(0, 24)}…": longer than ${MAX_SRC} characters`);
  const toks = src.match(TOKEN) ?? [];
  if (!toks.length) throw new ExprError("empty expression");
  if (toks.length > MAX_TOKENS) throw new ExprError(`"${src.slice(0, 24)}…": more than ${MAX_TOKENS} tokens`);
  let p = 0;
  let depth = 0;
  const peek = () => toks[p];
  const next = () => toks[p++];
  const fail = (m: string): never => { throw new ExprError(`"${src}": ${m}`); };
  const nest = <T>(f: () => T): T => {
    if (++depth > MAX_DEPTH) fail(`nested deeper than ${MAX_DEPTH}`);
    try { return f(); } finally { depth--; }
  };

  function atom(): Node {
    const tk = next();
    if (tk === undefined) return fail("ends early");
    if (/^[\d.]/.test(tk)) { const v = Number(tk); if (!Number.isFinite(v)) fail(`bad number ${tk}`); return () => v; }
    if (tk === "(") return nest(() => { const e = add(); if (next() !== ")") fail("missing )"); return e; });
    if (tk === "-") return nest(() => { const e = atom(); return (v) => -e(v); });
    if (/^[a-z_]/i.test(tk)) {
      if (peek() === "(") {
        next();
        const f = FNS.get(tk);
        if (!f) return fail(`unknown function ${tk}`);
        return nest(() => {
          const args: Node[] = [];
          if (peek() !== ")") { do { args.push(add()); } while (peek() === "," && next()); }
          if (next() !== ")") fail("missing )");
          if (args.length < f.arity[0] || args.length > f.arity[1]) fail(`${tk} takes ${f.arity[0]}–${f.arity[1]} arguments, not ${args.length}`);
          return (v) => f.call(...args.map((a) => a(v)));
        });
      }
      if (tk === "pi") return () => Math.PI;
      if (tk === "tau") return () => TAU;
      if (!VAR_SET.has(tk)) return fail(`unknown name ${tk}`);
      const name = tk as VarName;
      return (v) => v[name];
    }
    return fail(`unexpected ${tk}`);
  }
  function pow(): Node {
    const a = atom();
    if (peek() === "^") { next(); const b = atom(); return (v) => Math.pow(a(v), b(v)); }
    return a;
  }
  function mul(): Node {
    let a = pow();
    while (peek() === "*" || peek() === "/") {
      const op = next(), l = a, r = pow();
      a = op === "*" ? (v) => l(v) * r(v) : (v) => l(v) / r(v);
    }
    return a;
  }
  function add(): Node {
    let a = mul();
    while (peek() === "+" || peek() === "-") {
      const op = next(), l = a, r = mul();
      a = op === "+" ? (v) => l(v) + r(v) : (v) => l(v) - r(v);
    }
    return a;
  }
  const fn = add();
  if (p < toks.length) fail(`unexpected ${toks[p]}`);
  return fn;
}

const CACHE_MAX = 512;
const cache = new Map<string, Compiled | null>();

/** Parse a source string; throws ExprError. A literal number needs no parsing. */
export function compile(src: string | number): Compiled {
  if (typeof src === "number") {
    const v = Number.isFinite(src) ? src : 0;
    return () => v;
  }
  if (typeof src !== "string") throw new ExprError("an expression is a number or a string");
  const hit = cache.get(src);
  if (hit) return hit;
  if (hit === null) throw new ExprError(`"${src.slice(0, 40)}": does not parse`);
  let node: Node;
  try { node = parse(src); } catch (e) {
    if (cache.size >= CACHE_MAX) cache.clear();
    cache.set(src, null);
    throw e;
  }
  // a NaN or an infinity (1/0, sqrt of a negative is clamped, 10^999) draws as 0, never as garbage
  const fn: Compiled = (v) => { const r = node(v); return Number.isFinite(r) ? r : 0; };
  if (cache.size >= CACHE_MAX) cache.clear();
  cache.set(src, fn);
  return fn;
}

/** The first problem with an expression, or null when it parses. */
export function expressionProblem(src: unknown): string | null {
  if (typeof src !== "string" && typeof src !== "number") return "a number or an expression string is needed";
  if (typeof src === "number" && !Number.isFinite(src)) return "not a finite number";
  try { compile(src); return null; } catch (e) { return e instanceof ExprError ? e.message : "does not parse"; }
}

/** Evaluate a spec field: the default when absent or when it does not parse (a spec that passed validate never does). */
export function evalNum(src: unknown, env: Env, dflt = 0): number {
  if (src === undefined || src === null) return dflt;
  if (typeof src !== "string" && typeof src !== "number") return dflt;
  try { return compile(src)(env); } catch { return dflt; }
}

export function makeEnv(over: Partial<Env> = {}): Env {
  return { t: 0, beat: 0, i: 0, j: 0, n: 1, hp: 1, pwr: 2, charge: 0, shield: 0, hit: 0, fire: 0, die: 0, aw: 0, pi: Math.PI, tau: TAU, ...over };
}
