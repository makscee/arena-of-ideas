// A unit's portrait spec and the checks it must pass before it ships.
//   Spec  { body: Layer[], mark?: Layer[] }
//   Layer { shape, count?, x?, y?, rot?, s?, alpha?, fill?, stroke?, ol?, glow?, ... }
// Numbers are literals or expressions (./grammar). validate() looks at the
// shape of the data, never draws; the pixel checks live in ./pixels.
import { evalNum, expressionProblem, makeEnv, type Env } from "./grammar";
import { isPaletteName } from "./palette";

export const SHAPES = ["circle", "ring", "ellipse", "rect", "poly", "star", "petal", "line", "arc", "group"] as const;
export type ShapeName = (typeof SHAPES)[number];

export const MAX_COPIES = 16; // copies of one layer
export const MAX_SHAPES = 48; // shapes drawn in one portrait
export const MAX_LAYERS = 24; // layers in one list (body or mark), groups' children included
export const MAX_GROUP_DEPTH = 3;
export const MAX_NOTE = 200;

export type Num = number | string;
export type Paint = string | { grad: [string, string] } | { radial: [string, string] };
export interface Layer {
  shape: ShapeName;
  count?: Num;
  x?: Num; y?: Num; rot?: Num; s?: Num; alpha?: Num; glow?: Num; ol?: Num;
  r?: Num; r2?: Num; rx?: Num; ry?: Num; w?: Num; h?: Num; rad?: Num; n?: Num; len?: Num; a?: Num;
  fill?: Paint;
  stroke?: string;
  blend?: "add";
  layers?: Layer[];
  note?: string;
}
export interface Spec { body: Layer[]; mark?: Layer[] }

const NUM_KEYS: ReadonlySet<string> = new Set(["count", "x", "y", "rot", "s", "alpha", "glow", "ol", "r", "r2", "rx", "ry", "w", "h", "rad", "n", "len", "a"]);
const KEYS: ReadonlySet<string> = new Set([...NUM_KEYS, "shape", "fill", "stroke", "blend", "layers", "note"]);
const SHAPE_SET: ReadonlySet<string> = new Set(SHAPES);

export interface Validation { ok: boolean; problems: string[] }

const isObj = (x: unknown): x is Record<string, unknown> => typeof x === "object" && x !== null && !Array.isArray(x);
const own = (o: Record<string, unknown>, k: string): unknown => (Object.hasOwn(o, k) ? o[k] : undefined);

function paintProblem(where: string, key: string, v: unknown): string | null {
  if (typeof v === "string") return isPaletteName(v) ? null : `${where}.${key}: "${v.slice(0, 24)}" is not in the palette`;
  if (key === "fill" && isObj(v)) {
    const ks = Object.keys(v);
    const g = ks.length === 1 ? own(v, ks[0] as string) : undefined;
    if ((ks[0] === "grad" || ks[0] === "radial") && Array.isArray(g) && g.length === 2) {
      for (const c of g) if (!isPaletteName(c)) return `${where}.fill.${ks[0]}: "${String(c).slice(0, 24)}" is not in the palette`;
      return null;
    }
    return `${where}.fill: an object fill is {grad:[a,b]} or {radial:[a,b]}`;
  }
  return `${where}.${key}: a palette name${key === "fill" ? " or a gradient" : ""} is needed`;
}

function layerProblems(L: unknown, where: string, depth: number, out: string[]): void {
  if (!isObj(L)) { out.push(`${where}: a layer is an object`); return; }
  const shape = own(L, "shape");
  if (typeof shape !== "string" || !SHAPE_SET.has(shape)) out.push(`${where}: unknown shape ${String(shape).slice(0, 24)}`);
  for (const key of Object.keys(L)) {
    if (!KEYS.has(key)) { out.push(`${where}: unknown key ${key.slice(0, 24)}`); continue; }
    const v = own(L, key);
    if (NUM_KEYS.has(key)) {
      const p = expressionProblem(v);
      if (p) out.push(`${where}.${key}: ${p}`);
      else if (key === "count" && typeof v === "number" && (v < 1 || v > MAX_COPIES)) out.push(`${where}.count: ${v} is outside 1–${MAX_COPIES}`);
    } else if (key === "fill" || key === "stroke") {
      const p = paintProblem(where, key, v);
      if (p) out.push(p);
    } else if (key === "blend") {
      if (v !== "add") out.push(`${where}.blend: only "add"`);
    } else if (key === "note") {
      if (typeof v !== "string" || v.length > MAX_NOTE) out.push(`${where}.note: a string of at most ${MAX_NOTE} characters`);
    }
  }
  const kids = own(L, "layers");
  if (kids !== undefined) {
    if (shape !== "group") out.push(`${where}: only a group has layers`);
    else if (!Array.isArray(kids)) out.push(`${where}.layers: a list`);
    else if (depth >= MAX_GROUP_DEPTH) out.push(`${where}: groups nested deeper than ${MAX_GROUP_DEPTH}`);
    else listProblems(kids, `${where}.layers`, depth + 1, out);
  }
}

function listProblems(list: unknown[], where: string, depth: number, out: string[]): void {
  if (list.length > MAX_LAYERS) { out.push(`${where}: ${list.length} layers (limit ${MAX_LAYERS})`); return; }
  list.forEach((L, k) => layerProblems(L, `${where}[${k}]`, depth, out));
}

// ---- the shape budget: how many shapes a spec draws, at its busiest sampled state
function drawnIn(layers: unknown, env: Env, left: { n: number }): void {
  if (!Array.isArray(layers)) return;
  for (const L of layers) {
    if (!isObj(L) || left.n < 0) continue;
    const count = Math.max(1, Math.min(MAX_COPIES, Math.round(evalNum(own(L, "count"), env, 1))));
    const pi = env.i, pn = env.n, pj = env.j;
    for (let i = 0; i < count && left.n >= 0; i++) {
      if (own(L, "shape") === "group") env.j = i;
      env.i = i; env.n = count;
      if (Math.min(1, Math.max(0, evalNum(own(L, "alpha"), env, 1))) <= 0.01) continue;
      if (own(L, "shape") === "group") { drawnIn(own(L, "layers"), env, left); env.i = i; env.n = count; } else left.n--;
    }
    env.i = pi; env.n = pn; env.j = pj;
  }
}

const SAMPLES: Partial<Env>[] = [
  {},
  { t: 0.33, hp: 0.3, charge: 3, aw: 1 },
  { t: 1.1, hit: 1, fire: 1 },
  { t: 2.2, shield: 3, pwr: 9 },
  { t: 3.7, aw: 1, charge: 3, shield: 3, hit: 1, fire: 1, die: 0.5, hp: 0, pwr: 9, beat: 1 },
];

/** Shapes the spec draws at its busiest sampled state (an upper bound is not computable for arbitrary expressions; draw() stops at the cap anyway). */
export function shapeCount(spec: Spec): number {
  let worst = 0;
  for (const s of SAMPLES) {
    const left = { n: MAX_SHAPES + 1 };
    const env = makeEnv(s);
    drawnIn(spec.body, env, left);
    drawnIn(spec.mark, makeEnv(s), left);
    worst = Math.max(worst, MAX_SHAPES + 1 - left.n);
  }
  return worst;
}

/** The schema (known keys, shapes), palette, expressions and shape budget. Never throws. */
export function validate(spec: unknown): Validation {
  const problems: string[] = [];
  try {
    if (!isObj(spec)) return { ok: false, problems: ["a spec is an object {body, mark?}"] };
    for (const key of Object.keys(spec)) if (key !== "body" && key !== "mark") problems.push(`unknown key ${key.slice(0, 24)}`);
    const body = own(spec, "body"), mark = own(spec, "mark");
    if (!Array.isArray(body) || !body.length) problems.push("no body: a non-empty list of layers");
    else listProblems(body, "body", 0, problems);
    if (mark !== undefined) {
      if (!Array.isArray(mark)) problems.push("mark: a list of layers");
      else listProblems(mark, "mark", 0, problems);
    }
    if (!problems.length) {
      const n = shapeCount({ body: body as Layer[], ...(mark !== undefined ? { mark: mark as Layer[] } : {}) });
      if (n > MAX_SHAPES) problems.push(`${n} shapes drawn (limit ${MAX_SHAPES})`);
    }
  } catch {
    problems.push("the spec could not be read");
  }
  return { ok: problems.length === 0, problems };
}
