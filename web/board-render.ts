// Board renderer — BoardState in, DOM out. Pure display: shapes, names,
// numbers, status chips. All state it draws was projected by the kernel's
// boardAt; nothing here knows a rule (generative-graphics pillar: units are
// cheap code-drawn SVG shapes, no image assets).

import type { BoardState, BoardUnit, Side } from "../src/index.js";

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

function shapeSvg(unitName: string, dead: boolean): string {
  const h = hashName(unitName);
  const shape = SHAPES[h % SHAPES.length];
  const hue = (h * 137.508) % 360;
  const fill = dead ? "hsl(0 0% 35%)" : `hsl(${hue.toFixed(0)} 45% 58%)`;
  const stroke = dead ? "hsl(0 0% 25%)" : `hsl(${hue.toFixed(0)} 50% 38%)`;
  return `<svg class="shape" viewBox="0 0 32 32" aria-hidden="true"><g fill="${fill}" stroke="${stroke}" stroke-width="2">${shape}</g></svg>`;
}

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

function unitCard(u: BoardUnit, displayName: string, opts: { front: boolean; dead: boolean; hit: boolean }): string {
  const cls = ["unit", opts.front && "front", opts.dead && "dead", opts.hit && "hit"].filter(Boolean).join(" ");
  const chips = u.statuses
    .map(
      (s) =>
        `<span class="chip" title="${esc(s.status)} ×${s.stacks}">${esc(s.status.slice(0, 3))}${s.stacks}</span>`,
    )
    .join("");
  const silenced = u.silenced ? '<span class="chip mute" title="Silenced">mut</span>' : "";
  return `
    <div class="${cls}" data-unit="${esc(u.id)}" title="${esc(u.id)}">
      ${opts.front ? '<span class="front-tag">front</span>' : ""}
      ${shapeSvg(u.name, opts.dead)}
      <span class="uname">${esc(displayName)}</span>
      <span class="unums"><span class="hp">${u.hp}/${u.maxHp}</span><span class="pwr">${u.pwr}</span></span>
      <span class="chips">${chips}${silenced}</span>
    </div>`;
}

function sideHtml(board: BoardState, side: Side, name: (id: string) => string, hit: Set<string>): string {
  const line = board.lines[side]
    .map((u, i) => unitCard(u, name(u.id), { front: i === 0, dead: false, hit: hit.has(u.id) }))
    .join("");
  const grave = board.graves[side]
    .map((u) => unitCard(u, name(u.id), { front: false, dead: true, hit: hit.has(u.id) }))
    .join("");
  return `
    <div class="side" data-side="${side}">
      <div class="side-head"><span class="side-tag">side ${side}</span><span class="front-hint">front first ▸</span></div>
      <div class="line">${line || '<span class="wiped">— no one standing —</span>'}</div>
      ${grave ? `<div class="grave"><span class="grave-tag">grave</span>${grave}</div>` : ""}
    </div>`;
}

/** Replaces `root`'s content with the board: side A's line, a divider, side B's. */
export function renderBoard(
  root: HTMLElement,
  board: BoardState,
  name: (id: string) => string,
  hit: Set<string>,
): void {
  const verdict = board.ended
    ? `<span class="verdict">${board.ended.winner === "draw" ? "draw" : `side ${board.ended.winner} wins`} · turn ${board.ended.turns}</span>`
    : `<span class="verdict dim">turn ${board.turn}</span>`;
  root.innerHTML = `${sideHtml(board, "A", name, hit)}<div class="divider">${verdict}</div>${sideHtml(board, "B", name, hit)}`;
}
