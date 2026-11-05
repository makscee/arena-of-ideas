// A new unit's sigil spec from its LLM (mission arena#34, slice C, arena#37).
// Where the game makes a unit (the idea reader's archetypes, the night
// designer's fusions) the same call also writes the unit's portrait: about 12
// lines of JSON, drawn on a canvas by mobile/ui/sigil. This file holds what
// both writers share: the brief the model reads, the JSON schema it answers
// in, and `resolvePortrait`, which checks a spec with slice A's `validate`,
// asks once more with the problem list when it fails and falls back to the
// generic sigil after that. The spec rides in the unit's or design's JSON: no
// migration. The model's text is data: validate never evals anything.
import { FUNCTION_NAMES, VARS } from "../../../mobile/ui/sigil/grammar.js";
import { PALETTE_NAMES } from "../../../mobile/ui/sigil/palette.js";
import { FALLBACK_SPEC } from "../../../mobile/ui/sigil/fallback.js";
import { MAX_COPIES, MAX_SHAPES, SHAPES, validate, type Spec } from "../../../mobile/ui/sigil/spec.js";

export { FALLBACK_SPEC, validate };
export type { Spec };

/** Two specs from the UI duel, as examples: a Drummer (drum, sticks, rings that grow with Awoken, charge pips) and a Candle (a flame whose body shrinks with hp). */
const EXAMPLES: Spec[] = [
  {
    body: [
      { shape: "rect", w: 0.96, h: 0.5, rad: 0.08, y: 0.2, fill: { grad: ["owner", "owner.dark"] } },
      { shape: "line", count: 3, len: 0.42, w: 0.05, x: "-0.3+0.3*i", y: 0.43, rot: "0.5*(1-2*fr(i/2))", stroke: "gold", ol: 0 },
      { shape: "ellipse", rx: 0.48, ry: 0.15, y: "-0.05+0.03*beat", fill: "paper" },
      { shape: "line", count: 2, len: 0.62, w: 0.08, x: "pick(i,-0.12,0.12)", y: -0.06, rot: "pick(i,-0.75,0.75)*(1-0.35*beat)", stroke: "bone" },
    ],
    mark: [
      { shape: "arc", count: "2+aw", r: "0.48+0.13*i+0.05*beat", a: 1.3, w: 0.05, y: 0.05, stroke: "paper", alpha: "0.8-0.25*i", ol: 0 },
      { shape: "circle", count: 3, r: 0.08, x: "-0.22+0.22*i", y: 0.66, fill: "gold", alpha: "0.3+0.7*step(charge,i+1)", glow: "step(charge,i+1)" },
    ],
  },
  {
    body: [
      { shape: "ellipse", rx: 0.56, ry: 0.14, y: 0.66, fill: "gold" },
      { shape: "rect", w: 0.52, h: "0.22+0.56*hp", rad: 0.06, y: "0.62-(0.22+0.56*hp)/2", fill: { grad: ["paper", "bone"] } },
      { shape: "petal", len: "0.4+0.05*sin(t*9)", w: 0.2, y: "0.56-(0.22+0.56*hp)", rot: "0.08*sin(t*5)", fill: { grad: ["paper", "ember"] }, glow: 0.7 },
    ],
    mark: [{ shape: "circle", r: "0.3+0.04*sin(t*9)", y: "0.36-(0.22+0.56*hp)", fill: "gold", alpha: 0.22, ol: 0, blend: "add" }],
  },
];

/** The brief both writers put in their system prompt: what a sigil is, the closed vocabulary, the caps, two examples. */
export function sigilBrief(): string {
  return [
    "A sigil is the unit's portrait: a bold, flat silhouette with a thick ink outline, drawn from a small JSON spec. No emoji, no text, no plate or background (the game draws the stand). It reads at 44 pixels, so use big simple shapes.",
    "Spec = { body: Layer[], mark?: Layer[] }. `body` is the creature or object; `mark` is the small extra that the Awoken form and counters grow. About 12 layers in all.",
    "Space: the unit circle. x and y run -1 to 1 from the centre, y down; a body fills about radius 0.8. Angles (rot) are radians; s scales a layer.",
    `Shapes (the \`shape\` key): ${SHAPES.join(", ")}.`,
    "  circle {r}; ring {r} (an open outline: stroke, w); ellipse {rx, ry}; rect {w, h, rad (corner)}; poly {n (sides), r}; star {n (points), r, r2 (inner radius)}; petal {len, w} (a leaf or flame pointing up); line {len, w} (from the layer's origin, up); arc {r, a (arc width in radians), w}; group {layers: [...]} (a layer list drawn together; at most 3 deep).",
    "Every layer may have: count (copies, 1 to 16; inside a layer `i` is the copy number from 0 and `n` the count), x, y, rot, s, alpha (0 to 1), glow, ol (outline width; 0 for none), fill, stroke, blend (only \"add\"), note (a short label).",
    `Colours are names only: ${PALETTE_NAMES.join(", ")}. "owner" is the side's colour (teal for you, rose for them), so one spec reads for either side. A fill is a name or {"grad": [a, b]} or {"radial": [a, b]}; an open shape (ring, line, arc) takes a \`stroke\` name.`,
    `Any number may be an expression of the unit's state, a string such as "0.5+0.1*beat". Names: ${VARS.filter((v) => v !== "pi" && v !== "tau").join(" ")} pi tau. Functions: ${FUNCTION_NAMES.join(" ")}. Operators + - * / ^ and brackets, at most 160 characters. Nothing else exists: no other names, no code.`,
    "  t = seconds; beat = a pulse from 0 to 1; hp = health 0 to 1; pwr = power; charge, shield = counters 0 to 9; hit, fire = flashes after being hit or striking; die = 0 to 1 as it dies; aw = 1 once Awoken (let the spec grow then); i, j, n = copy indices.",
    `Limits: ${MAX_COPIES} copies of a layer, ${MAX_SHAPES} shapes in a portrait at once, 24 layers in a list. A spec over a limit is refused.`,
    "Make the silhouette say what the unit is (a lantern keeper: a lantern; a wolf: a wolf head), and bind at least one thing to the state (hp, beat, charge, aw) so it feels alive.",
    "Two example specs (a Drummer and a Candle):",
    ...EXAMPLES.map((s) => JSON.stringify(s)),
  ].join("\n");
}

const NUM = { type: ["number", "string"] };
const COLOUR = { type: "string", enum: [...PALETTE_NAMES] };
const GRADIENT = { type: "array", items: COLOUR, minItems: 2, maxItems: 2 };
const LAYER = {
  type: "object",
  properties: {
    shape: { type: "string", enum: [...SHAPES] },
    ...Object.fromEntries(["count", "x", "y", "rot", "s", "alpha", "glow", "ol", "r", "r2", "rx", "ry", "w", "h", "rad", "n", "len", "a"].map((k) => [k, NUM])),
    fill: { anyOf: [COLOUR, { type: "object", properties: { grad: GRADIENT, radial: GRADIENT }, additionalProperties: false }] },
    stroke: COLOUR,
    blend: { type: "string", enum: ["add"] },
    layers: { type: "array", items: { type: "object" } },
    note: { type: "string" },
  },
  required: ["shape"],
  additionalProperties: false,
};

/** The JSON schema of a spec, for the model's structured answer. The server still runs `validate` on what comes back. */
export const PORTRAIT_SCHEMA = {
  type: "object",
  properties: { body: { type: "array", items: LAYER, minItems: 1, maxItems: 24 }, mark: { type: "array", items: LAYER, maxItems: 24 } },
  required: ["body"],
  additionalProperties: false,
};

/** What is wrong with a spec as the model wrote it, as lines to show it; empty when it can ship. */
export function portraitProblems(spec: unknown): string[] {
  return spec === undefined || spec === null ? ["no portrait was given"] : validate(spec).problems;
}

/** The note on a retry: the problems, for the model's next try. */
export function portraitRetryNote(problems: string[]): string {
  return `Your last portrait spec was refused:\n${problems.slice(0, 12).map((p) => `- ${p}`).join("\n")}\nWrite the whole spec again so that none of these apply.`;
}

/**
 * A unit's portrait: `first` when it validates, else what `again` gives when
 * asked once with the problem list, else the generic sigil. `again` throwing
 * counts as a bad answer (a model error never fails the unit). The result is
 * always a spec that passed `validate` (the fallback does), copied so later
 * edits never reach the shared one.
 */
export async function resolvePortrait(first: unknown, again: ((problems: string[]) => Promise<unknown>) | undefined, log: (line: string) => void = () => {}): Promise<Spec> {
  const bad = portraitProblems(first);
  if (!bad.length) return structuredClone(first as Spec);
  if (again) {
    let second: unknown;
    try {
      second = await again(bad);
    } catch (err) {
      log(`portrait: the second try failed: ${(err as Error).message.slice(0, 200)}`);
    }
    const worse = portraitProblems(second);
    if (!worse.length) return structuredClone(second as Spec);
    log(`portrait: refused twice (${worse[0]}); the generic sigil stands in`);
  } else {
    log(`portrait: refused (${bad[0]}); the generic sigil stands in`);
  }
  return structuredClone(FALLBACK_SPEC);
}
