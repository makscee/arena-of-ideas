// The generic sigil for a unit with no spec of its own (or one that failed
// validate): a bold hex shield with a boss, and a ring that breathes with the beat.
import type { Spec } from "./spec";

export const FALLBACK_SPEC: Spec = {
  body: [
    { shape: "poly", n: 6, r: 0.78, rot: 0.52, s: "1+0.04*beat", fill: { grad: ["owner", "owner.dark"] } },
    { shape: "circle", r: 0.26, y: -0.02, s: "1+0.08*beat", fill: "paper" },
  ],
  mark: [
    { shape: "arc", r: 0.58, a: "1.2+0.5*beat", w: 0.06, y: 0.02, stroke: "gold" },
  ],
};
