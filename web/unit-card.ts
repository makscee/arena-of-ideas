// The one unit card (PRD #015 slice 1) — every place a unit renders draws
// THIS markup: shop offers, the team line, the battle board, ladder pools.
// Pure presentation over data the kernel already produced: generative
// code-drawn shape art (hash → layered SVG, no image assets — pillar 3),
// framed hp/pwr, a level badge with fusion pips, status chips. The card owns
// zero rules; every description behind it stays kernel-derived (chipsHtml /
// the inspector's describe helpers).

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
  const hue = (h * 137.508) % 360;
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
  /** Drives the generative art — the def name, stable across levels/instances. */
  artName: string;
  /** The name shown on the card (board instances may carry a display name). */
  label: string;
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
  /** Context classes (run-card, lv-unit) — widths are the context's to size. */
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
    o.classes,
    o.front === true && "front",
    o.dead === true && "dead",
    o.hit === true && "hit",
    o.sel === true && "sel",
    o.fused === true && "fused",
  ]
    .filter(Boolean)
    .join(" ");
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
      <span class="uname">${esc(o.label)}</span>
      ${badge}
      <span class="unums"><span class="hp">${o.hp}</span><span class="pwr">${o.pwr}</span></span>
      <span class="chips">${chipsHtml(o.statuses, o.registry)}${silenced}</span>
      ${o.footer ?? ""}
    </div>`;
}
