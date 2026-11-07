// Draws one portrait: the shared frame every unit gets (a stand in the side's
// colour, an awoken halo, hit flash, fire ring, death shards), the spec's body
// and mark in the unit circle, and the reactions on top. Canvas 2D, no assets.
// Spec numbers go through the closed grammar; the caps (16 copies a layer,
// 48 shapes a portrait) are enforced here as well as in validate().
import { evalNum, makeEnv, type Env } from "./grammar";
import { GOLD, INK, PAPER, color, ownerColor, type Team } from "./palette";
import { MAX_COPIES, MAX_SHAPES, type Layer, type Paint, type Spec } from "./spec";

type Ctx = CanvasRenderingContext2D;
const TAU = Math.PI * 2;

export interface DrawState {
  t?: number; // seconds, drives idle motion
  aw?: number; // 0 or 1: the 3rd copy, awoken
  hp?: number; // 0..1
  pwr?: number;
  charge?: number; // 0..9
  shield?: number; // 0..9
  hit?: number; // 0..1, decays
  fire?: number; // 0..1, decays
  die?: number; // 0..1, grows
}
export interface DrawOptions extends DrawState {
  S: number; // canvas side in CSS px
  team?: Team;
  frame?: boolean; // the shared frame and reactions around the spec; false draws the bare spec (pixel checks)
}

export const beatAt = (t: number): number => Math.pow(Math.abs(Math.sin(Math.PI * t * 1.5)), 6);
const fin = (x: unknown, lo: number, hi: number, d: number): number => (typeof x === "number" && Number.isFinite(x) ? Math.min(hi, Math.max(lo, x)) : d);
const own = (o: object, k: string): unknown => (Object.hasOwn(o, k) ? (o as Record<string, unknown>)[k] : undefined);

function paint(ctx: Ctx, p: Paint | undefined, team: Team, size: number): string | CanvasGradient | null {
  if (p === undefined) return color("owner", team);
  if (typeof p === "string") return color(p, team);
  if (typeof p !== "object" || p === null) return null;
  const radial = own(p, "radial") as [string, string] | undefined;
  const g = radial ?? (own(p, "grad") as [string, string] | undefined);
  if (!Array.isArray(g)) return null;
  const a = color(g[0], team), b = color(g[1], team);
  if (!a || !b) return null;
  const gr = radial ? ctx.createRadialGradient(0, -size * 0.3, size * 0.05, 0, 0, size * 1.1) : ctx.createLinearGradient(0, -size, 0, size);
  gr.addColorStop(0, a);
  gr.addColorStop(1, b);
  return gr;
}

interface Run { ctx: Ctx; env: Env; team: Team; R: number; left: number }

function layer(run: Run, L: Layer): void {
  const { ctx, env } = run;
  const group = own(L, "shape") === "group";
  const count = Math.max(1, Math.min(MAX_COPIES, Math.round(evalNum(L.count, env, 1))));
  const pi = env.i, pn = env.n, pj = env.j;
  for (let i = 0; i < count && run.left > 0; i++) {
    if (group) env.j = i;
    env.i = i; env.n = count;
    const alpha = Math.min(1, Math.max(0, evalNum(L.alpha, env, 1)));
    if (alpha <= 0.01) continue;
    ctx.save();
    ctx.translate(evalNum(L.x, env) * run.R, evalNum(L.y, env) * run.R);
    ctx.rotate(evalNum(L.rot, env));
    const s = evalNum(L.s, env, 1);
    ctx.scale(s, s);
    ctx.globalAlpha *= alpha;
    if (L.blend === "add") ctx.globalCompositeOperation = "lighter";
    if (group) {
      if (Array.isArray(L.layers)) for (const c of L.layers) if (typeof c === "object" && c !== null) layer(run, c);
      env.i = i; env.n = count;
    } else {
      shape(run, L);
      run.left--;
    }
    ctx.restore();
  }
  env.i = pi; env.n = pn; env.j = pj;
}

function shape(run: Run, L: Layer): void {
  const { ctx, env, team, R } = run;
  const g = (k: keyof Layer, d: number) => evalNum(L[k], env, d) * R;
  ctx.beginPath();
  let size = R, open = false;
  switch (L.shape) {
    case "circle": case "ring": { const r = Math.max(0, g("r", 0.5)); size = r; ctx.arc(0, 0, r, 0, TAU); open = L.shape === "ring"; break; }
    case "ellipse": { const rx = Math.max(0, g("rx", 0.5)), ry = Math.max(0, g("ry", 0.3)); size = Math.max(rx, ry); ctx.ellipse(0, 0, rx, ry, 0, 0, TAU); break; }
    case "rect": {
      const w = Math.max(0, g("w", 0.5)), h = Math.max(0, g("h", 0.5)), rd = Math.max(0, Math.min(g("rad", 0), w / 2, h / 2));
      size = Math.max(w, h) / 2;
      if (typeof ctx.roundRect === "function") ctx.roundRect(-w / 2, -h / 2, w, h, rd); else ctx.rect(-w / 2, -h / 2, w, h);
      break;
    }
    case "poly": case "star": {
      const star = L.shape === "star";
      const n = Math.max(3, Math.min(32, Math.round(evalNum(L.n, env, 5)))), r = Math.max(0, g("r", 0.5)), r2 = star ? Math.max(0, g("r2", 0.25)) : r;
      const k = star ? n * 2 : n;
      size = r;
      for (let q = 0; q < k; q++) {
        const a = -Math.PI / 2 + (q * TAU) / k, rr = q % 2 && star ? r2 : r;
        if (q) ctx.lineTo(Math.cos(a) * rr, Math.sin(a) * rr); else ctx.moveTo(Math.cos(a) * rr, Math.sin(a) * rr);
      }
      ctx.closePath();
      break;
    }
    case "petal": { const len = g("len", 0.5), w = g("w", 0.2); size = Math.abs(len) / 2; ctx.moveTo(0, 0); ctx.quadraticCurveTo(w, -len * 0.55, 0, -len); ctx.quadraticCurveTo(-w, -len * 0.55, 0, 0); break; }
    case "line": { const len = g("len", 0.5); ctx.moveTo(0, 0); ctx.lineTo(0, -len); open = true; break; }
    case "arc": { const r = Math.max(0, g("r", 0.5)), a = evalNum(L.a, env, 1); ctx.arc(0, 0, r, -Math.PI / 2 - a / 2, -Math.PI / 2 + a / 2); open = true; break; }
    default: return;
  }
  const glow = Math.max(0, evalNum(L.glow, env, 0));
  const fillC = open ? null : paint(ctx, L.fill, team, size);
  const strokeC = open ? color(L.stroke ?? "paper", team) ?? PAPER : null;
  if (glow > 0) { ctx.shadowBlur = glow * R * 0.45; ctx.shadowColor = color(L.stroke ?? (typeof L.fill === "string" ? L.fill : "gold"), team) ?? "#fff"; }
  if (fillC) { ctx.fillStyle = fillC; ctx.fill(); }
  ctx.shadowBlur = 0;
  if (open) {
    const w = Math.max(1.4, evalNum(L.w, env, 0.06) * R);
    ctx.lineCap = "round";
    if (L.ol !== 0) { ctx.strokeStyle = INK; ctx.lineWidth = w + Math.max(2.4, 0.09 * R); ctx.stroke(); }
    if (glow > 0) ctx.shadowBlur = glow * R * 0.45;
    ctx.strokeStyle = strokeC as string; ctx.lineWidth = w; ctx.stroke();
  } else if (L.ol !== 0) {
    ctx.strokeStyle = INK; ctx.lineWidth = Math.max(1.8, evalNum(L.ol, env, 0.075) * R); ctx.lineJoin = "round"; ctx.stroke();
  }
}

/** Draw a portrait into ctx, in a square of side o.S at the origin. Returns the number of spec shapes drawn. */
export function drawSigil(ctx: Ctx, spec: Spec, o: DrawOptions): number {
  const S = fin(o.S, 1, 4096, 64);
  const team: Team = o.team === "them" ? "them" : "you";
  const frame = o.frame !== false;
  const t = fin(o.t, -1e6, 1e6, 0);
  const aw = fin(o.aw, 0, 1, 0);
  const env = makeEnv({
    t, beat: beatAt(t), hp: fin(o.hp, 0, 1, 1), pwr: fin(o.pwr, 0, 99, 2), charge: fin(o.charge, 0, 9, 0), shield: fin(o.shield, 0, 9, 0),
    hit: fin(o.hit, 0, 1, 0), fire: fin(o.fire, 0, 1, 0), die: fin(o.die, 0, 1, 0), aw,
  });
  const R = S * 0.4;
  const run: Run = { ctx, env, team, R, left: MAX_SHAPES };
  ctx.save();
  ctx.translate(S / 2, S / 2);
  if (frame) {
    if (env.hit > 0) ctx.translate(Math.sin(t * 70) * env.hit * R * 0.08, Math.cos(t * 53) * env.hit * R * 0.04);
    if (env.die > 0) { ctx.globalAlpha = 1 - env.die; ctx.scale(1 + env.die * 0.25, 1 + env.die * 0.25); }
    // no plate: a thin stand in the side's colour, under the silhouette
    ctx.beginPath(); ctx.ellipse(0, R * 0.98, R * 0.5, R * 0.11, 0, 0, TAU);
    ctx.fillStyle = "rgba(0,0,0,.35)"; ctx.fill();
    ctx.lineWidth = Math.max(1.5, R * 0.04); ctx.strokeStyle = aw ? GOLD : ownerColor(team); ctx.globalAlpha *= 0.9; ctx.stroke(); ctx.globalAlpha /= 0.9;
  }
  if (Array.isArray(spec?.body)) for (const L of spec.body) if (typeof L === "object" && L !== null) layer(run, L);
  if (Array.isArray(spec?.mark)) for (const L of spec.mark) if (typeof L === "object" && L !== null) layer(run, L);
  const drawn = MAX_SHAPES - run.left;
  if (frame) {
    if (env.hit > 0) { ctx.globalCompositeOperation = "source-atop"; ctx.fillStyle = `rgba(255,255,255,${env.hit * 0.55})`; ctx.fillRect(-R * 1.3, -R * 1.3, 2.6 * R, 2.6 * R); ctx.globalCompositeOperation = "source-over"; }
    if (aw) { // awoken: a gold crown of sparks turning round the rim
      for (let q = 0; q < 6; q++) {
        const a = t * 0.6 + (q * TAU) / 6;
        ctx.save(); ctx.translate(Math.cos(a) * R * 1.06, Math.sin(a) * R * 1.06); ctx.rotate(a);
        ctx.beginPath();
        for (let k = 0; k < 8; k++) { const b = (k * TAU) / 8, rr = k % 2 ? R * 0.05 : R * 0.13; ctx.lineTo(Math.cos(b) * rr, Math.sin(b) * rr); }
        ctx.closePath();
        ctx.fillStyle = GOLD; ctx.fill(); ctx.lineWidth = 1; ctx.strokeStyle = INK; ctx.stroke(); ctx.restore();
      }
    }
    if (env.fire > 0) { // its rule fires: a gold ring bursts out of the rim
      ctx.beginPath(); ctx.arc(0, 0, R * (1.02 + 0.2 * (1 - env.fire)), 0, TAU);
      ctx.lineWidth = Math.max(2, R * 0.12 * env.fire); ctx.strokeStyle = `rgba(240,194,75,${env.fire})`;
      ctx.shadowBlur = R * 0.5; ctx.shadowColor = GOLD; ctx.stroke(); ctx.shadowBlur = 0;
    }
    if (env.die > 0) { // death: the silhouette shatters into its owner's shards
      for (let q = 0; q < 9; q++) {
        const a = (q * TAU) / 9 + 0.3, d = R * (0.4 + env.die * 0.9);
        ctx.save(); ctx.translate(Math.cos(a) * d, Math.sin(a) * d); ctx.rotate(a + env.die * 3);
        ctx.beginPath(); ctx.moveTo(0, -R * 0.14); ctx.lineTo(R * 0.09, R * 0.08); ctx.lineTo(-R * 0.09, R * 0.06); ctx.closePath();
        ctx.fillStyle = ownerColor(team); ctx.fill(); ctx.restore();
      }
    }
  }
  ctx.restore();
  return drawn;
}
