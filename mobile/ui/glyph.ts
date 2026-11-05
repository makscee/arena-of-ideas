// A unit's sigil as a small inline picture (arena#39, mission arena#34): what every
// screen draws where it used to print the unit's emoji. The table's pieces have their
// own (./piece.ts); this is for the places beside a name, on a card, in a sheet's head,
// a chip, a list row. Small ones (inline with text) are one still frame; a big one
// (a sheet's head, a card) is mounted in the shared loop and moves.
import { drawSigil, fuse, mountSigil, specOrFallback, type Spec, type Team } from "./sigil";
import { glyphSpec } from "./sigil/glyphs";
import { glyphName } from "../../src/mvp/glyph-names";

/** The live units' specs by id, set once the content loads (setCardFaces). */
let specs = new Map<string, Spec>();
export function setPortraits(units: { id: string; portrait?: Spec }[]): void {
  specs = new Map(units.flatMap((u) => (u.portrait ? [[u.id, u.portrait] as const] : [])));
}
/** A pool unit's spec by id (a version's stored id too, when the content serves it); undefined when it has none. */
export const portraitOf = (unitId: string | undefined): Spec | undefined => (unitId ? specs.get(unitId) : undefined);

export interface GlyphOptions {
  team?: Team;
  /** Mounted in the shared loop (moves) instead of one still frame. For anything bigger than a line of text. */
  live?: boolean;
  /** An extra class on the canvas. */
  cls?: string;
  /** The awoken look (the gold crown). */
  aw?: boolean;
  /** Leave the size to the stylesheet (the canvas is drawn at `px` and shown at whatever the CSS says). */
  fluid?: boolean;
}

/** A sigil on a canvas of `px` CSS px. The spec may be missing or invalid: the generic sigil is drawn then. */
export function sigilIcon(spec: unknown, px: number, o: GlyphOptions = {}): HTMLCanvasElement {
  const cv = document.createElement("canvas");
  cv.className = `sg-ic${o.cls ? ` ${o.cls}` : ""}`;
  cv.setAttribute("aria-hidden", "true");
  const S = Math.max(8, Math.round(px));
  const sp = specOrFallback(spec);
  const team = o.team ?? "you";
  if (o.live) {
    mountSigil(cv, { spec: sp, S, team, aw: o.aw ? 1 : 0, seed: Math.random() * 3 });
    cv.classList.add("live");
    if (o.fluid) { cv.style.width = ""; cv.style.height = ""; }
    return cv;
  }
  const dpr = Math.min(2, globalThis.devicePixelRatio || 1);
  cv.width = Math.round(S * dpr);
  cv.height = Math.round(S * dpr);
  if (!o.fluid) { cv.style.width = `${S}px`; cv.style.height = `${S}px`; }
  const ctx = cv.getContext("2d");
  if (ctx) {
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    drawSigil(ctx, sp, { S, team, t: 0.3, aw: o.aw ? 1 : 0 });
  }
  return cv;
}

/** The sigil of a unit-like thing: its own spec, else its pool unit's (a fusion shows its first part's body), else the generic one. */
export function unitSigil(u: { portrait?: Spec | undefined; unitId?: string | undefined; fusion?: { first: string; second?: string } | undefined; kind?: string | undefined }, px: number, o: GlyphOptions = {}): HTMLCanvasElement {
  // a fusion: one part's body and the other's mark (slice F, ./sigil fuse)
  if (u.fusion?.second && (u.kind === undefined || u.kind === "fused")) return sigilIcon(fuse(portraitOf(u.fusion.first), portraitOf(u.fusion.second)), px, o);
  return sigilIcon(u.portrait ?? portraitOf(u.fusion?.first ?? u.unitId), px, o);
}

/** A fusion's two parts side by side, small (a pair row in the Codex). A part that is missing draws the generic sigil. */
export function pairSigils(a: { id?: string; portrait?: Spec | undefined } | undefined, b: { id?: string; portrait?: Spec | undefined } | undefined, px: number): HTMLElement {
  const wrap = document.createElement("span");
  wrap.className = "sg-pair";
  wrap.append(...[a, b].map((x) => sigilIcon(x?.portrait ?? portraitOf(x?.id), px)));
  return wrap;
}

/** A glyph (the when tab, the headline badge, a status chip: ./sigil/glyphs) on a canvas of `px` CSS px, one still frame with its thick ink outline.
 * `name` is a glyph name or an older emoji (it maps through the table). Drawn bigger than its box by a quarter so the shapes fill it. */
export function glyphIcon(name: string | undefined, px: number, cls = ""): HTMLCanvasElement {
  const cv = document.createElement("canvas");
  cv.className = `sg-gl${cls ? ` ${cls}` : ""}`;
  cv.setAttribute("aria-hidden", "true");
  const key = glyphName(name);
  cv.dataset["glyph"] = key;
  const P = Math.max(8, Math.round(px));
  const dpr = Math.min(3, Math.max(2, globalThis.devicePixelRatio || 1));
  cv.width = Math.round(P * dpr);
  cv.height = Math.round(P * dpr);
  cv.style.width = `${P}px`;
  cv.style.height = `${P}px`;
  const ctx = cv.getContext("2d");
  if (ctx) {
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.translate(-P * 0.125, -P * 0.125);
    drawSigil(ctx, glyphSpec(key), { S: P * 1.25, frame: false, t: 0 });
  }
  return cv;
}
