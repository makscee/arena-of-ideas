// The checks that need pixels, so they run in a browser (the e2e), not in vitest:
// a portrait stays inside its circle, fills enough of it at 44 px, and moves.
import { drawSigil, type DrawOptions } from "./draw";
import type { Spec } from "./spec";

export interface Check { name: string; pass: boolean; why: string }

function pixels(spec: Spec, S: number, st: Partial<DrawOptions>): Uint8ClampedArray {
  const cv = document.createElement("canvas");
  cv.width = cv.height = S;
  const x = cv.getContext("2d", { willReadFrequently: true }) as CanvasRenderingContext2D;
  drawSigil(x, spec, { S, frame: false, ...st });
  return x.getImageData(0, 0, S, S).data;
}

export function pixelChecks(spec: Spec): Check[] {
  const out: Check[] = [];
  let outside = 0, inside = 0;
  for (const st of [{ t: 0 }, { t: 0.33, hp: 0.3, charge: 3, aw: 1 }, { t: 1.1, hit: 1, fire: 1 }, { t: 2.2, shield: 3 }]) {
    const S = 120, d = pixels(spec, S, st), R = S * 0.4;
    for (let y = 0; y < S; y++) for (let x = 0; x < S; x++) {
      if ((d[(y * S + x) * 4 + 3] as number) < 128) continue;
      if (Math.hypot(x - S / 2, y - S / 2) > R * 1.04) outside++; else inside++;
    }
  }
  out.push({ name: "inside-circle", pass: outside / Math.max(1, inside) < 0.01, why: `${((100 * outside) / Math.max(1, inside)).toFixed(1)}% outside` });
  const S = 44, d = pixels(spec, S, { t: 0 }), R = S * 0.4;
  let cov = 0, tot = 0;
  for (let y = 0; y < S; y++) for (let x = 0; x < S; x++) {
    if (Math.hypot(x - S / 2, y - S / 2) > R) continue;
    tot++;
    if ((d[(y * S + x) * 4 + 3] as number) > 128) cov++;
  }
  const f = cov / tot;
  out.push({ name: "fills-44px", pass: f > 0.2 && f < 0.85, why: `silhouette fills ${Math.round(f * 100)}% of its circle (20–85%)` });
  const a = pixels(spec, 64, { t: 0.1 }), b = pixels(spec, 64, { t: 0.45 });
  let diff = 0;
  for (let k = 3; k < a.length; k += 4) {
    if (Math.abs((a[k] as number) - (b[k] as number)) > 30 || Math.abs((a[k - 3] as number) - (b[k - 3] as number)) > 30 || Math.abs((a[k - 2] as number) - (b[k - 2] as number)) > 30) diff++;
  }
  out.push({ name: "alive", pass: diff > 8, why: `${diff} pixels move between two idle frames` });
  return out;
}
