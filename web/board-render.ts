// Board renderer — BoardState in, DOM out. Pure display: the one shared unit
// card (unit-card.ts), live-battle flavoured: current/max hp, hit highlight,
// silenced chip, front marker. All state it draws was projected by the
// kernel's boardAt; nothing here knows a rule.

import type { BeatOverlay, BoardState, BoardUnit, Side, StatusRegistry } from "../src/index.js";
import { overlayHasContent } from "../src/index.js";
import { unitCardHtml } from "./unit-card.js";

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

/** The typed-badge overlay drawn ON an affected hero this beat (#065 slice 2).
 * Each badge is its own pill so a unit can carry several at once (−N damage,
 * +N heal, ±status, +N pwr, ✕ death) and the numbers live-increment as the
 * card's lines reveal — the totals come straight off `overlaysAt`, which sums
 * within the beat. Damage and heal are SEPARATE badges (the design decision:
 * an absorb that heals back stays visible as both, never netted). An empty/
 * netted-out overlay draws nothing. The death ✕ is the dying-in-place mark on
 * the card itself, so it is NOT repeated here. */
function overlayBadgesHtml(o: BeatOverlay | undefined): string {
  if (!o || !overlayHasContent(o)) return "";
  const badges: string[] = [];
  if (o.damage > 0) badges.push(`<span class="ov-badge ov-dmg">−${o.damage}</span>`);
  if (o.heal > 0) badges.push(`<span class="ov-badge ov-heal">+${o.heal}</span>`);
  for (const [stat, delta] of Object.entries(o.statChanges)) {
    if (delta === 0) continue;
    badges.push(`<span class="ov-badge ov-stat">${delta > 0 ? "+" : ""}${delta} ${esc(stat)}</span>`);
  }
  for (const [status, delta] of Object.entries(o.statusDeltas)) {
    if (delta === 0) continue;
    badges.push(`<span class="ov-badge ov-status">${delta > 0 ? "+" : ""}${delta} ${esc(status)}</span>`);
  }
  if (badges.length === 0) return "";
  return `<span class="ov-layer" aria-hidden="true">${badges.join("")}</span>`;
}

/** Per-unit display extras the replay layers on the shared card: the typed
 * beat-overlay badges and the dying-in-place flag. The shop/team/ladder pass
 * none of this — the card contract is unchanged for them. */
export interface UnitOverlays {
  /** unit id → its accumulated beat overlay (drives badges + dying mark). */
  by: Map<string, BeatOverlay>;
}

function unitCard(
  u: BoardUnit,
  displayName: string,
  registry: StatusRegistry,
  opts: { front: boolean; dead: boolean; dying: boolean; hit: boolean; sel: boolean; overlay?: BeatOverlay | undefined },
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
    dying: opts.dying,
    hit: opts.hit,
    sel: opts.sel,
    silenced: u.silenced,
    attrs: `data-unit="${esc(u.id)}"`,
    title,
    overlay: overlayBadgesHtml(opts.overlay),
  });
}

function sideHtml(
  board: BoardState,
  side: Side,
  name: (id: string) => string,
  hit: Set<string>,
  registry: StatusRegistry,
  selected: string | undefined,
  overlays: UnitOverlays | undefined,
): string {
  const ov = (id: string): BeatOverlay | undefined => overlays?.by.get(id);
  // A unit whose Death landed in the OPEN beat greys + ✕ IN PLACE for the rest
  // of its beat, then collapses to the grave at the next beat (#065 slice 2).
  // boardAt has already moved it to the grave at the Death step, so the render
  // pulls it back into the line as a `dying` card while its overlay says died;
  // the line position is the line tail (its slot index is gone once removed),
  // which keeps it on the board in the line area, not the grave. At the next
  // beat its overlay clears, the dying flag drops, and it renders as a normal
  // grave card — the collapse happens for free at the boundary.
  const dyingFromGrave = board.graves[side].filter((u) => ov(u.id)?.died === true);
  const lineUnits = board.lines[side];
  const restGrave = board.graves[side].filter((u) => ov(u.id)?.died !== true);

  const line =
    lineUnits
      .map((u, i) =>
        unitCard(u, name(u.id), registry, {
          front: i === 0,
          dead: false,
          dying: false,
          hit: hit.has(u.id),
          sel: u.id === selected,
          overlay: ov(u.id),
        }),
      )
      .join("") +
    dyingFromGrave
      .map((u) =>
        unitCard(u, name(u.id), registry, {
          front: false,
          dead: false,
          dying: true,
          hit: hit.has(u.id),
          sel: u.id === selected,
          overlay: ov(u.id),
        }),
      )
      .join("");
  const grave = restGrave
    .map((u) =>
      unitCard(u, name(u.id), registry, {
        front: false,
        dead: true,
        dying: false,
        hit: hit.has(u.id),
        sel: u.id === selected,
      }),
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

/** The verdict/turn pill the stage centre carries when no beat card is open. */
export function verdictHtml(board: BoardState): string {
  return board.ended
    ? `<span class="verdict">${board.ended.winner === "draw" ? "draw" : `side ${board.ended.winner} wins`} · turn ${board.ended.turns}</span>`
    : `<span class="verdict dim">turn ${board.turn}</span>`;
}

/** The board as markup — pure, so layout-stability invariants (grave rows
 * always present) are testable without a DOM. The stage centre (between side A
 * and side B) is `centerHtml` — the beat card or a turn divider the viewer
 * computes; default is the plain verdict divider. */
export function boardHtml(
  board: BoardState,
  name: (id: string) => string,
  hit: Set<string>,
  registry: StatusRegistry,
  selected?: string,
  centerHtml?: string,
  overlays?: UnitOverlays,
): string {
  const center = centerHtml ?? `<div class="divider">${verdictHtml(board)}</div>`;
  return `${sideHtml(board, "A", name, hit, registry, selected, overlays)}<div class="stage-center">${center}</div>${sideHtml(board, "B", name, hit, registry, selected, overlays)}`;
}

/** Replaces `root`'s content with the board: side A's line, the stage centre,
 * side B's. `selected` marks the unit the inspector is open on; `centerHtml`
 * is the beat card / turn divider for the current beat; `overlays` carries the
 * per-unit beat badges + dying-in-place state (#065 slice 2). */
export function renderBoard(
  root: HTMLElement,
  board: BoardState,
  name: (id: string) => string,
  hit: Set<string>,
  registry: StatusRegistry,
  selected?: string,
  centerHtml?: string,
  overlays?: UnitOverlays,
): void {
  root.innerHTML = boardHtml(board, name, hit, registry, selected, centerHtml, overlays);
}
