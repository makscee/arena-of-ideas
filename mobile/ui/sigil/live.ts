// The live loop: every mounted sigil canvas is drawn by one requestAnimationFrame
// loop. It runs while at least one canvas is mounted and stops by itself when
// the last one goes (or has left the document).
import { drawSigil, type DrawOptions } from "./draw";
import type { Spec } from "./spec";

export interface LiveOptions extends Omit<DrawOptions, "t"> {
  spec: Spec;
  seed?: number; // seconds added to the clock, so a row of the same unit does not pulse in step
}
export type Pulse = "hit" | "fire" | "die";
export interface SigilHandle {
  /** Change the state it is drawn in (hp, hit, fire, charge, shield, aw, spec, team ...). */
  update(patch: Partial<LiveOptions>): void;
  /** A reaction the loop plays by itself: `hit` and `fire` fall from 1 to 0 over `ms`; `die` shatters it (0 to 1 in the first 45 %)
   * and then the portrait is back whole: a fallen unit stays on the board as a ghost, which the card greys.
   * `ageMs` is how long ago it began (a re-drawn board keeps a running pulse going); negative waits that long first. */
  pulse(kind: Pulse, ms: number, ageMs?: number): void;
  unmount(): void;
}

interface Entry { cv: HTMLCanvasElement; ctx: CanvasRenderingContext2D; dpr: number; o: LiveOptions; born: number; seen: boolean; pulses: Partial<Record<Pulse, { at: number; ms: number }>> }

/** How long a canvas may wait to be attached to the document before the loop drops it, ms. */
const ATTACH_GRACE_MS = 2000;
/** Where in a die pulse the portrait is most shattered. */
const DIE_PEAK = 0.45;

const live = new Set<Entry>();
let raf: number | null = null;
let t0 = 0;

function size(e: Entry): void {
  const S = e.o.S;
  e.cv.width = Math.round(S * e.dpr);
  e.cv.height = Math.round(S * e.dpr);
  e.cv.style.width = `${S}px`;
  e.cv.style.height = `${S}px`;
}

function frame(): void {
  raf = null;
  const t = (performance.now() - t0) / 1000;
  const now = performance.now();
  for (const e of live) {
    if (e.cv.isConnected) e.seen = true;
    else if (e.seen || now - e.born > ATTACH_GRACE_MS) { live.delete(e); continue; }
    else continue; // built, not attached yet: a piece waits in a detached tree for a moment
    e.ctx.setTransform(e.dpr, 0, 0, e.dpr, 0, 0);
    e.ctx.clearRect(0, 0, e.o.S, e.o.S);
    drawSigil(e.ctx, e.o.spec, { ...e.o, ...pulsed(e, now), t: t + (e.o.seed ?? 0) });
  }
  if (live.size) raf = requestAnimationFrame(frame);
}

function pulsed(e: Entry, now: number): Partial<Record<Pulse, number>> {
  const out: Partial<Record<Pulse, number>> = {};
  for (const k of ["hit", "fire", "die"] as const) {
    const p = e.pulses[k];
    if (!p) continue;
    const x = Math.min(1, Math.max(0, (now - p.at) / p.ms));
    out[k] = k === "die" ? (x < DIE_PEAK ? x / DIE_PEAK : 1 - (x - DIE_PEAK) / (1 - DIE_PEAK)) : 1 - x;
    if (now - p.at >= p.ms) delete e.pulses[k];
  }
  return out;
}

/** Mount a canvas in the loop. Attach it to the document soon (a canvas is waited for 2 s, then dropped; one that has left the document is dropped on the next frame). */
export function mountSigil(cv: HTMLCanvasElement, o: LiveOptions): SigilHandle {
  const ctx = cv.getContext("2d");
  if (!ctx) return { update() {}, pulse() {}, unmount() {} };
  const e: Entry = { cv, ctx, dpr: Math.min(2, globalThis.devicePixelRatio || 1), o: { ...o }, born: performance.now(), seen: false, pulses: {} };
  size(e);
  if (!live.size) t0 = performance.now();
  live.add(e);
  if (raf === null) raf = requestAnimationFrame(frame);
  return {
    update(patch) {
      const S = e.o.S;
      e.o = { ...e.o, ...patch };
      if (e.o.S !== S) size(e);
    },
    pulse(kind, ms, ageMs = 0) { e.pulses[kind] = { at: performance.now() - ageMs, ms: Math.max(1, ms) }; },
    unmount() { live.delete(e); },
  };
}

/** Number of canvases the loop draws. */
export const mountedCount = (): number => live.size;

/** Drop every canvas and stop the loop. */
export function resetSigils(): void {
  live.clear();
  if (raf !== null) { cancelAnimationFrame(raf); raf = null; }
}
