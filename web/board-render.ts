// Board renderer — BoardState in, DOM out. Pure display: shapes, names,
// numbers, status chips. All state it draws was projected by the kernel's
// boardAt; nothing here knows a rule (generative-graphics pillar: units are
// cheap code-drawn SVG shapes, no image assets).

import type { BoardState, BoardUnit, Side, StatusRegistry } from "../src/index.js";
import { chipsHtml } from "./inspect.js";

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

/** Exported for the run screen — shop offers and line units wear the same
 * code-drawn shape they will wear on the battle board. */
export function shapeSvg(unitName: string, dead: boolean): string {
  const h = hashName(unitName);
  const shape = SHAPES[h % SHAPES.length];
  const hue = (h * 137.508) % 360;
  const fill = dead ? "hsl(0 0% 35%)" : `hsl(${hue.toFixed(0)} 45% 58%)`;
  const stroke = dead ? "hsl(0 0% 25%)" : `hsl(${hue.toFixed(0)} 50% 38%)`;
  return `<svg class="shape" viewBox="0 0 32 32" aria-hidden="true"><g fill="${fill}" stroke="${stroke}" stroke-width="2">${shape}</g></svg>`;
}

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

function unitCard(
  u: BoardUnit,
  displayName: string,
  registry: StatusRegistry,
  opts: { front: boolean; dead: boolean; hit: boolean; sel: boolean },
): string {
  const cls = ["unit", opts.front && "front", opts.dead && "dead", opts.hit && "hit", opts.sel && "sel"]
    .filter(Boolean)
    .join(" ");
  const chips = chipsHtml(u.statuses, registry);
  const silenced = u.silenced ? '<span class="chip mute" title="Silenced">mut</span>' : "";
  // The tooltip slot carries player-useful state, never the internal id (IA-7).
  const title = opts.dead
    ? `${displayName} — dead · tap to inspect`
    : `${displayName} — ${u.hp}/${u.maxHp} hp, ${u.pwr} pwr · tap to inspect`;
  return `
    <div class="${cls}" data-unit="${esc(u.id)}" title="${esc(title)}">
      ${opts.front ? '<span class="front-tag">front</span>' : ""}
      ${shapeSvg(u.name, opts.dead)}
      <span class="uname">${esc(displayName)}</span>
      <span class="unums"><span class="hp">${u.hp}/${u.maxHp}</span><span class="pwr">${u.pwr}</span></span>
      <span class="chips">${chips}${silenced}</span>
    </div>`;
}

function sideHtml(
  board: BoardState,
  side: Side,
  name: (id: string) => string,
  hit: Set<string>,
  registry: StatusRegistry,
  selected: string | undefined,
): string {
  const line = board.lines[side]
    .map((u, i) =>
      unitCard(u, name(u.id), registry, { front: i === 0, dead: false, hit: hit.has(u.id), sel: u.id === selected }),
    )
    .join("");
  const grave = board.graves[side]
    .map((u) =>
      unitCard(u, name(u.id), registry, { front: false, dead: true, hit: hit.has(u.id), sel: u.id === selected }),
    )
    .join("");
  // The grave row is part of a side's shape from event 0 — empty until the
  // first death — and the line survives a wipe as a placeholder, so deaths
  // never change the board's height mid-replay (audit LS-1).
  return `
    <div class="side" data-side="${side}">
      <div class="side-head"><span class="side-tag">side ${side}</span><a class="front-hint" href="#codex/rule/strike-order" title="Front units fight first — tap for the strike-order rule">front first ▸</a></div>
      <div class="line">${line || '<span class="wiped">— no one standing —</span>'}</div>
      <div class="grave"><span class="grave-tag">grave</span>${grave}</div>
    </div>`;
}

/** The board as markup — pure, so layout-stability invariants (grave rows
 * always present) are testable without a DOM. */
export function boardHtml(
  board: BoardState,
  name: (id: string) => string,
  hit: Set<string>,
  registry: StatusRegistry,
  selected?: string,
): string {
  const verdict = board.ended
    ? `<span class="verdict">${board.ended.winner === "draw" ? "draw" : `side ${board.ended.winner} wins`} · turn ${board.ended.turns}</span>`
    : `<span class="verdict dim">turn ${board.turn}</span>`;
  return `${sideHtml(board, "A", name, hit, registry, selected)}<div class="divider">${verdict}</div>${sideHtml(board, "B", name, hit, registry, selected)}`;
}

/** Replaces `root`'s content with the board: side A's line, a divider, side B's.
 * `selected` marks the unit the inspector is open on. */
export function renderBoard(
  root: HTMLElement,
  board: BoardState,
  name: (id: string) => string,
  hit: Set<string>,
  registry: StatusRegistry,
  selected?: string,
): void {
  root.innerHTML = boardHtml(board, name, hit, registry, selected);
}
