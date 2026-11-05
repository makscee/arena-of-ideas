// The glyphs (mission arena#34, slice G, arena#41): the small drawn marks that replace
// every emoji in a piece's chrome (the when tab, the headline badge, the status chips).
// A glyph is a sigil spec, so it is drawn by the same engine with the same thick ink
// outline and the same closed palette: bold flat shapes, no plate, no emoji, no gradients
// (a glyph reads at 11 px). Coordinates are the engine's: x right, y down, 1 is the
// radius; a glyph stays inside 0.9 so its outline is not clipped. The names are
// src/mvp/glyph-names.ts (the face's `icon`).
import type { GlyphName } from "../../../src/mvp/glyph-names";
import type { Layer, Spec } from "./spec";

/** Shapes that make one silhouette: each drawn with its outline, then again without it, so the seams between the parts vanish. */
const union = (...layers: Layer[]): Layer[] => [...layers.map((l) => ({ ol: 0.11, ...l })), ...layers.map((l) => ({ ...l, ol: 0 }))];
const body = (...layers: Layer[]): Spec => ({ body: layers });

const HEART: Layer[] = union(
  { shape: "circle", r: 0.38, x: -0.32, y: -0.22, fill: "blood" },
  { shape: "circle", r: 0.38, x: 0.32, y: -0.22, fill: "blood" },
  { shape: "poly", n: 3, r: 0.8, rot: Math.PI, y: 0.04, fill: "blood" },
);

export const GLYPH_SPECS: Readonly<Record<GlyphName, Spec>> = {
  sunrise: body(
    { shape: "line", count: 5, len: 0.3, w: 0.13, x: "0.56*sin((i-2)*0.62)", y: "0.18-0.56*cos((i-2)*0.62)", rot: "(i-2)*0.62", stroke: "gold" },
    { shape: "circle", r: 0.46, y: 0.18, fill: "gold" },
    { shape: "rect", w: 1.7, h: 0.28, rad: 0.07, y: 0.66, fill: "paper" },
  ),
  moon: body(
    { shape: "arc", r: 0.5, a: 4.1, w: 0.36, rot: 3.93, stroke: "paper" },
    { shape: "star", n: 4, r: 0.26, r2: 0.09, x: 0.46, y: -0.42, fill: "gold" },
  ),
  flag: body(
    { shape: "rect", w: 0.14, h: 1.7, rad: 0.05, x: -0.42, fill: "bone" },
    ...union({ shape: "poly", n: 3, r: 0.62, rot: Math.PI / 2, x: -0.05, y: -0.42, fill: "blood" }),
    { shape: "rect", w: 0.62, h: 0.14, rad: 0.05, x: -0.42, y: 0.8, fill: "bone" },
  ),
  swords: body({
    shape: "group", count: 2, rot: "0.785*(1-2*i)", layers: [
      { shape: "rect", w: 0.18, h: 1.5, rad: 0.05, y: -0.1, fill: "paper" },
      { shape: "rect", w: 0.56, h: 0.14, rad: 0.05, y: 0.4, fill: "gold" },
      { shape: "rect", w: 0.13, h: 0.3, rad: 0.04, y: 0.62, fill: "bone" },
    ],
  }),
  "broken-heart": body(
    ...HEART,
    { shape: "line", len: 0.42, w: 0.1, x: 0.06, y: -0.55, rot: 3.54, stroke: "paper" },
    { shape: "line", len: 0.36, w: 0.1, x: -0.12, y: -0.15, rot: 2.38, stroke: "paper" },
    { shape: "line", len: 0.44, w: 0.1, x: 0.12, y: 0.1, rot: 3.54, stroke: "paper" },
  ),
  skull: body(
    ...union(
      { shape: "circle", r: 0.56, y: -0.14, fill: "paper" },
      { shape: "rect", w: 0.54, h: 0.46, rad: 0.1, y: 0.4, fill: "paper" },
    ),
    { shape: "circle", count: 2, r: 0.16, x: "(i*2-1)*0.23", y: -0.1, fill: "ink", ol: 0 },
    { shape: "poly", n: 3, r: 0.11, y: 0.14, fill: "ink", ol: 0 },
    { shape: "rect", w: 0.07, h: 0.2, count: 2, x: "(i*2-1)*0.1", y: 0.5, fill: "ink", ol: 0 },
  ),
  infinity: body(
    { shape: "ring", r: 0.33, x: -0.34, w: 0.2, stroke: "paper" },
    { shape: "ring", r: 0.33, x: 0.34, w: 0.2, stroke: "paper" },
  ),
  paw: body(
    ...union(
      { shape: "ellipse", rx: 0.44, ry: 0.36, y: 0.3, fill: "paper" },
      { shape: "ellipse", rx: 0.17, ry: 0.22, count: 4, x: "pick(i,-0.62,-0.21,0.21,0.62)", y: "pick(i,-0.02,-0.42,-0.42,-0.02)", rot: "pick(i,-0.4,-0.1,0.1,0.4)", fill: "paper" },
    ),
  ),
  flask: body(
    ...union(
      { shape: "rect", w: 0.26, h: 0.6, y: -0.42, fill: "paper" },
      { shape: "circle", r: 0.5, y: 0.26, fill: "venom" },
    ),
    { shape: "rect", w: 0.5, h: 0.14, rad: 0.05, y: -0.76, fill: "paper" },
    { shape: "circle", r: 0.09, x: 0.14, y: 0.14, fill: "paper", ol: 0 },
  ),
  scroll: body(
    { shape: "rect", w: 1.0, h: 1.2, rad: 0.08, fill: "paper" },
    { shape: "rect", w: 0.64, h: 0.09, count: 3, y: "-0.22+0.22*i", fill: "ink", ol: 0 },
    { shape: "rect", w: 1.3, h: 0.24, rad: 0.12, count: 2, y: "(i*2-1)*0.6", fill: "bone" },
  ),
  egg: body(
    { shape: "ellipse", rx: 0.5, ry: 0.68, fill: "bone" },
    { shape: "circle", count: 3, r: "pick(i,0.1,0.08,0.07)", x: "pick(i,-0.18,0.2,0)", y: "pick(i,-0.12,0.14,-0.4)", fill: "ember", ol: 0 },
  ),
  shield: body(
    { shape: "poly", n: 5, r: 0.86, rot: Math.PI, y: -0.02, fill: "frost" },
    { shape: "poly", n: 5, r: 0.4, rot: Math.PI, y: 0, fill: "paper", ol: 0.05 },
  ),
  fist: body(
    ...union(
      { shape: "rect", w: 0.96, h: 0.74, rad: 0.2, y: 0.22, fill: "paper" },
      { shape: "circle", count: 4, r: 0.2, x: "-0.36+0.24*i", y: -0.18, fill: "paper" },
      { shape: "ellipse", rx: 0.14, ry: 0.3, x: -0.55, y: 0.34, rot: 0.3, fill: "paper" },
    ),
    { shape: "line", count: 3, len: 0.3, w: 0.05, x: "-0.24+0.24*i", y: -0.02, rot: Math.PI, stroke: "ink", ol: 0 },
  ),
  heart: body(...HEART),
  halo: body({
    shape: "group", count: 2, s: "0.8*(1-2*i)", layers: [
      { shape: "petal", len: 0.95, w: 0.3, x: 0.06, y: 0.5, rot: 0.5, fill: "paper" },
      { shape: "petal", len: 1.25, w: 0.38, x: 0.06, y: 0.6, rot: 0.95, fill: "paper" },
    ],
  }, { shape: "ring", r: 0.22, y: -0.58, w: 0.12, stroke: "gold" }),
  bolt: body(
    ...union(
      { shape: "rect", w: 0.38, h: 0.95, rot: 0.5, x: 0.22, y: -0.36, fill: "gold" },
      { shape: "rect", w: 0.38, h: 0.95, rot: 0.5, x: -0.22, y: 0.36, fill: "gold" },
      { shape: "rect", w: 0.9, h: 0.32, rot: -0.12, y: 0, fill: "gold" },
      { shape: "poly", n: 3, r: 0.3, rot: 3.6, x: -0.48, y: 0.8, fill: "gold" },
    ),
  ),
  drop: body(
    ...union(
      { shape: "poly", n: 3, r: 0.56, y: -0.34, fill: "venom" },
      { shape: "circle", r: 0.5, y: 0.26, fill: "venom" },
    ),
    { shape: "circle", r: 0.1, x: -0.2, y: 0.3, fill: "paper", ol: 0 },
  ),
  snowflake: body(
    { shape: "line", count: 6, len: 0.78, w: 0.13, rot: "i*1.047", stroke: "frost" },
    { shape: "circle", count: 6, r: 0.11, x: "0.78*sin(i*1.047)", y: "-0.78*cos(i*1.047)", fill: "paper" },
    { shape: "circle", r: 0.2, fill: "paper" },
  ),
  curse: body(
    { shape: "poly", n: 3, r: 0.92, rot: Math.PI, y: -0.16, fill: "blood" },
    { shape: "rect", w: 0.52, h: 0.13, rad: 0.04, y: -0.3, fill: "paper", ol: 0 },
  ),
  silence: body(
    { shape: "ellipse", rx: 0.32, ry: 0.22, y: -0.04, fill: "paper" },
    { shape: "ring", r: 0.62, w: 0.16, stroke: "blood" },
    { shape: "line", len: 1.24, w: 0.16, x: -0.44, y: 0.44, rot: 0.785, stroke: "blood" },
  ),
  plus: body(
    ...union(
      { shape: "rect", w: 0.44, h: 1.4, rad: 0.1, fill: "paper" },
      { shape: "rect", w: 1.4, h: 0.44, rad: 0.1, fill: "paper" },
    ),
  ),
  repeat: body(
    { shape: "arc", r: 0.52, a: 4.5, w: 0.18, stroke: "paper" },
    ...union({ shape: "poly", n: 3, r: 0.32, x: 0.44, y: 0.37, rot: 3.82, fill: "paper" }),
  ),
  swap: body({
    shape: "group", count: 2, rot: "i*3.1416", layers: [
      { shape: "line", len: 0.92, w: 0.17, x: -0.46, y: -0.3, rot: 1.5708, stroke: "paper" },
      { shape: "poly", n: 3, r: 0.3, x: 0.46, y: -0.3, rot: 1.5708, fill: "paper" },
    ],
  }),
  mirror: body(
    { shape: "rect", w: 0.16, h: 0.4, rad: 0.05, y: 0.76, fill: "bone" },
    { shape: "ellipse", rx: 0.56, ry: 0.7, y: -0.1, fill: "gold" },
    { shape: "ellipse", rx: 0.4, ry: 0.54, y: -0.1, fill: "frost", ol: 0.05 },
    { shape: "line", len: 0.34, w: 0.07, x: -0.17, y: 0.14, rot: 0.5, stroke: "paper", ol: 0 },
  ),
  burst: body(
    { shape: "star", n: 8, r: 0.9, r2: 0.46, fill: "ember" },
    { shape: "star", n: 8, r: 0.46, r2: 0.22, rot: 0.39, fill: "gold", ol: 0.05 },
  ),
  summon: body(
    { shape: "ring", r: 0.56, w: 0.18, stroke: "arcane" },
    { shape: "star", n: 4, r: 0.42, r2: 0.12, fill: "gold" },
  ),
  grave: body(
    ...union(
      { shape: "circle", r: 0.42, y: -0.16, fill: "bone" },
      { shape: "rect", w: 0.84, h: 0.86, y: 0.3, fill: "bone" },
    ),
    { shape: "rect", w: 0.09, h: 0.5, y: -0.08, fill: "ink", ol: 0 },
    { shape: "rect", w: 0.32, h: 0.09, y: -0.12, fill: "ink", ol: 0 },
    { shape: "rect", w: 1.5, h: 0.16, rad: 0.05, y: 0.78, fill: "night" },
  ),
  lock: body(
    { shape: "arc", r: 0.3, a: 3.1416, w: 0.15, y: -0.2, stroke: "bone" },
    { shape: "rect", w: 0.15, h: 0.4, x: -0.3, y: -0.18, fill: "bone", ol: 0 },
    { shape: "rect", w: 0.15, h: 0.4, x: 0.3, y: -0.18, fill: "bone", ol: 0 },
    { shape: "rect", w: 0.96, h: 0.74, rad: 0.12, y: 0.28, fill: "gold" },
    { shape: "circle", r: 0.12, y: 0.22, fill: "ink", ol: 0 },
    { shape: "rect", w: 0.08, h: 0.24, y: 0.36, fill: "ink", ol: 0 },
  ),
  spark: body(
    { shape: "star", n: 4, r: 0.88, r2: 0.22, fill: "gold" },
    { shape: "star", n: 4, r: 0.3, r2: 0.09, x: 0.58, y: -0.52, fill: "paper" },
  ),
};

/** A glyph's spec; the plain spark for a name it does not know. */
export const glyphSpec = (name: string): Spec => (Object.hasOwn(GLYPH_SPECS, name) ? GLYPH_SPECS[name as GlyphName] : GLYPH_SPECS.spark);
