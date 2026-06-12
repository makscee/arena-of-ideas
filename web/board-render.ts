// Board renderer — BoardState in, DOM out. Pure display: the one shared unit
// card (unit-card.ts), live-battle flavoured: current/max hp, hit highlight,
// silenced chip, front marker. All state it draws was projected by the
// kernel's boardAt; nothing here knows a rule.

import type { BoardState, BoardUnit, Side, StatusRegistry } from "../src/index.js";
import { unitCardHtml } from "./unit-card.js";

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

function unitCard(
  u: BoardUnit,
  displayName: string,
  registry: StatusRegistry,
  opts: { front: boolean; dead: boolean; hit: boolean; sel: boolean },
): string {
  // The tooltip slot carries player-useful state, never the internal id (IA-7).
  const title = opts.dead
    ? `${displayName} — dead · tap to inspect`
    : `${displayName} — ${u.hp}/${u.maxHp} hp, ${u.pwr} pwr · tap to inspect`;
  return unitCardHtml({
    artName: u.name,
    label: displayName,
    hp: `${u.hp}/${u.maxHp}`,
    pwr: u.pwr,
    statuses: u.statuses,
    registry,
    front: opts.front,
    dead: opts.dead,
    hit: opts.hit,
    sel: opts.sel,
    silenced: u.silenced,
    attrs: `data-unit="${esc(u.id)}"`,
    title,
  });
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
