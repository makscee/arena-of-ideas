// The one card (PRD #015 slice 1, made uniform in #078) — every place a Unit
// OR a Status renders draws THIS markup at ONE fixed size: shop offers, the
// team line, the battle board, ladder pools, and the codex (units AND statuses).
// A Unit and a Status are the same shape (PRD #074 ontology: a bundle of Parts),
// so they share the card — a Status frames its per-stack statMods where a Unit
// frames hp/pwr. The fixed size IS the complexity budget (#078): geometry lives
// in CSS on `.unit` itself (one `--card-w`), never set per-surface. Pure
// presentation over data the kernel already produced: generative code-drawn
// shape art (hash → layered SVG, no image assets — pillar 3), framed stats, a
// level badge with fusion pips, status chips. The card owns zero rules; every
// description behind it stays kernel-derived (chipsHtml / the describe helpers).

import type { StatusRegistry } from "../src/index.js";
import { chipsHtml } from "./inspect.js";

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

/** Small string hash — picks a stable shape + hue per unit name. */
function hashName(name: string): number {
  let h = 2166136261;
  for (let i = 0; i < name.length; i++) {
    h ^= name.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

/** Hash-stable hue for a name — the colour identity the shape art wears,
 * exported so a non-unit card (the codex's status cards) can carry the same
 * scheme without re-deriving it. */
export function nameHue(name: string): number {
  return (hashName(name) * 137.508) % 360;
}

const SHAPES = [
  '<circle cx="16" cy="16" r="12"/>',
  '<rect x="5" y="5" width="22" height="22" rx="3"/>',
  '<polygon points="16,3 29,16 16,29 3,16"/>', // diamond
  '<polygon points="16,4 29,27 3,27"/>', // triangle
  '<polygon points="9,4 23,4 30,16 23,28 9,28 2,16"/>', // hexagon
  '<polygon points="16,3 29,12 24,28 8,28 3,12"/>', // pentagon
];

/** Generative shape art, layered from the name's hash alone: a soft aura, the
 * main shape, and a rotated inner accent — same name, same art, everywhere a
 * unit shows. Dead units go monochrome (the grave keeps the silhouette). */
export function shapeSvg(unitName: string, dead: boolean): string {
  const h = hashName(unitName);
  const shape = SHAPES[h % SHAPES.length]!;
  const inner = SHAPES[(h >>> 3) % SHAPES.length]!;
  const spin = ((h >>> 7) % 8) * 45; // the accent's rotation — hash-stable
  const hue = nameHue(unitName);
  const fill = dead ? "hsl(0 0% 35%)" : `hsl(${hue.toFixed(0)} 45% 58%)`;
  const stroke = dead ? "hsl(0 0% 25%)" : `hsl(${hue.toFixed(0)} 50% 38%)`;
  const aura = dead ? "hsl(0 0% 30% / 0.15)" : `hsl(${hue.toFixed(0)} 60% 60% / 0.16)`;
  const accent = dead ? "hsl(0 0% 48%)" : `hsl(${((hue + 40) % 360).toFixed(0)} 55% 74%)`;
  return (
    `<svg class="shape" viewBox="0 0 32 32" aria-hidden="true">` +
    `<circle cx="16" cy="16" r="15" fill="${aura}"/>` +
    `<g fill="${fill}" stroke="${stroke}" stroke-width="2">${shape}</g>` +
    `<g transform="rotate(${spin} 16 16) translate(16 16) scale(0.42) translate(-16 -16)" fill="${accent}" opacity="0.9">${inner}</g>` +
    `</svg>`
  );
}

export interface UnitCardOpts {
  /** What entity this card renders. The skeleton (art, name, framed stats,
   * chips) and the fixed size are IDENTICAL for both — a Status is the same
   * shape as a Unit (a bundle of Parts, PRD #074 ontology), so it wears the
   * same card at the same size (the card size IS the complexity budget, #078).
   * A `status` frames its per-stack `statMods` in the stat cells where a unit
   * frames base hp/pwr. Defaults to "unit", so every existing caller is
   * unchanged. */
  kind?: "unit" | "status";
  /** Drives the generative art — the def name, stable across levels/instances. */
  artName: string;
  /** The name shown on the card (board instances may carry a display name). */
  label: string;
  /** The unit's team (#065 item 2): tints the name by side so the player reads it
   * as "their side". Only the battle board passes it; the shop/team/ladder omit
   * it (those screens aren't two-sided) — the card contract there is unchanged. */
  side?: "A" | "B";
  /** Pre-formatted stat text: "7", or "3/9" for a board card's current/max. */
  hp: string | number;
  pwr: string | number;
  registry: StatusRegistry;
  statuses?: readonly { status: string; stacks: number }[] | undefined;
  /** Renders the level badge when given; `pips` (●●○) ride inside it. */
  level?: number;
  pips?: string;
  front?: boolean;
  dead?: boolean;
  hit?: boolean;
  sel?: boolean;
  silenced?: boolean;
  fused?: boolean;
  /** Dying-in-place (#065 slice 2): a unit whose Death landed this beat shows
   * greyed with a ✕ in its line slot until the next beat collapses it to the
   * grave — distinct from `dead` (already in the grave). */
  dying?: boolean;
  /** Death-reveal moment (#065 item 4): the dying unit plays a distinct death
   * animation (red flash + shake, settling to the grey+✕ end-state) the single
   * step its Death is revealed. Set only at that step; on later steps it stays
   * `dying` (static grey+✕). Honors prefers-reduced-motion (the anim is skipped,
   * the end-state stays) via CSS. */
  dyingNew?: boolean;
  /** Beat-overlay badge layer (#065 slice 2): pre-built typed-badge HTML drawn
   * ON the card. Empty for every non-replay surface, so the card contract the
   * shop/team/ladder rely on is unchanged. */
  overlay?: string;
  /** Persistent coin marker (#065 slice 3): pre-built HTML for the coin chip the
   * holder wears (the most recent pairing's first striker). A PERSISTENT state
   * marker, separate from the per-beat `overlay` deltas. Empty everywhere but the
   * replay board, so the shop/team/ladder card contract is unchanged. */
  marker?: string;
  /** Context hooks (run-card, lv-unit, codex-unit) — for framing/state only
   * (selection ring, opacity, layout slot). The card's WIDTH is NOT theirs to
   * set: size is fixed on `.unit` itself (#078). A surface that needs different
   * framing wraps the card; it never resizes it. */
  classes?: string;
  /** The caller's wiring, pre-escaped: `data-offer="0"`, `data-unit="A1:X"`… */
  attrs: string;
  title: string;
  /** Controls under the chips (buy button, move arrows) — caller-built HTML. */
  footer?: string;
}

/** The shared card markup. Class names and child order are the app's card
 * contract: probes, hit-target CSS, and the inspector's anchors all key off
 * `.unit`, `.uname`, `.unums`, `.chips` and the data-* attrs the caller adds. */
export function unitCardHtml(o: UnitCardOpts): string {
  const cls = [
    "unit",
    o.kind === "status" && "is-status",
    o.classes,
    o.front === true && "front",
    o.dead === true && "dead",
    o.dying === true && "dying",
    o.dyingNew === true && "dying-new",
    o.hit === true && "hit",
    o.sel === true && "sel",
    o.fused === true && "fused",
  ]
    .filter(Boolean)
    .join(" ");
  // Team tint on the name (#065 item 2): side A / side B get distinct hues so a
  // name reads as its side. Reuses the battle log's .u/.ua/.ub side palette.
  const unameCls = ["uname", o.side === "A" && "u ua", o.side === "B" && "u ub"].filter(Boolean).join(" ");
  const badge =
    o.level !== undefined
      ? `<span class="run-lvl">L${o.level}${o.pips !== undefined ? ` <span class="run-pips">${o.pips}</span>` : ""}</span>`
      : "";
  // Like every other chip, the title explains the state, not just names it.
  const silenced =
    o.silenced === true
      ? '<span class="chip mute" title="Silenced — its statuses are stripped and its own abilities are disabled for the battle">mut</span>'
      : "";
  return `
    <div class="${cls}" ${o.attrs} title="${esc(o.title)}">
      ${o.front === true ? '<span class="front-tag">front</span>' : ""}
      ${shapeSvg(o.artName, o.dead === true)}
      <span class="${unameCls}">${esc(o.label)}</span>
      ${badge}
      <span class="unums"><span class="hp">${o.hp}</span><span class="pwr">${o.pwr}</span></span>
      <span class="chips">${chipsHtml(o.statuses, o.registry)}${silenced}</span>
      ${o.footer ?? ""}
      ${o.dying === true ? '<span class="dying-x" aria-hidden="true">✕</span>' : ""}
      ${o.overlay ?? ""}
      ${o.marker ?? ""}
    </div>`;
}
