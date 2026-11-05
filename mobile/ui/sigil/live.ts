// The live loop: every mounted sigil canvas is drawn by one requestAnimationFrame
// loop. It runs while at least one canvas is mounted and stops by itself when
// the last one goes (or has left the document).
import { drawSigil, type DrawOptions } from "./draw";
import type { Spec } from "./spec";

export interface LiveOptions extends Omit<DrawOptions, "t"> {
  spec: Spec;
  seed?: number; // seconds added to the clock, so a row of the same unit does not pulse in step
}
export interface SigilHandle {
  /** Change the state it is drawn in (hp, hit, fire, charge, shield, aw, spec, team ...). */
  update(patch: Partial<LiveOptions>): void;
  unmount(): void;
}

interface Entry { cv: HTMLCanvasElement; ctx: CanvasRenderingContext2D; dpr: number; o: LiveOptions }

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
  for (const e of live) {
    if (!e.cv.isConnected) { live.delete(e); continue; }
    e.ctx.setTransform(e.dpr, 0, 0, e.dpr, 0, 0);
    e.ctx.clearRect(0, 0, e.o.S, e.o.S);
    drawSigil(e.ctx, e.o.spec, { ...e.o, t: t + (e.o.seed ?? 0) });
  }
  if (live.size) raf = requestAnimationFrame(frame);
}

/** Mount a canvas in the loop. Attach it to the document in the same tick (a detached canvas is dropped on the next frame). */
export function mountSigil(cv: HTMLCanvasElement, o: LiveOptions): SigilHandle {
  const ctx = cv.getContext("2d");
  if (!ctx) return { update() {}, unmount() {} };
  const e: Entry = { cv, ctx, dpr: Math.min(2, globalThis.devicePixelRatio || 1), o: { ...o } };
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
