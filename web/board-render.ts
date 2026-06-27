// Board renderer — BoardState in, DOM out. Pure display: the one shared unit
// card (unit-card.ts), live-battle flavoured: current/max hp, hit highlight,
// silenced chip, front marker. All state it draws was projected by the
// kernel's boardAt; nothing here knows a rule.

import type { BeatOverlay, BoardState, BoardUnit, Side, StatusRegistry } from "../src/index.js";
import { overlayHasContent } from "../src/index.js";
import { statusColorStyle } from "./status-color.js";
import { unitCardHtml } from "./unit-card.js";
import { abilityLineFor, familyOf, type ActingCtx } from "./acting.js";

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
function overlayBadgesHtml(o: BeatOverlay | undefined, newKeys: Set<string> | undefined): string {
  if (!o || !overlayHasContent(o)) return "";
  // Only a NEWLY-appearing (or changed) badge plays the reveal — `ov-new` arms
  // the `ov-pop` animation in CSS, exactly as `bc-line-new` does for card lines.
  // The overlay layer re-renders its whole HTML each step, so without this every
  // badge re-ran its pop each step a new badge appeared (the director's "all of
  // them play the fade in when a new one appears"). `newKeys` (from
  // newBadgeKeysAt) names the keys whose number just moved; the rest paint static.
  // Each key mirrors badgeValues (`dmg`/`heal`/`stat:<n>`/`status:<n>`) so the
  // diff and the render never drift. A status badge carries a hash-stable colour
  // (#065 item 2: same status → same hue everywhere) via statusColorStyle, which
  // tints the badge family's --ov-fg/--ov-ring to the per-status hue.
  const fresh = (key: string): string => (newKeys?.has(key) === true ? " ov-new" : "");
  const badges: string[] = [];
  if (o.damage > 0) badges.push(`<span class="ov-badge ov-dmg${fresh("dmg")}">−${o.damage}</span>`);
  if (o.heal > 0) badges.push(`<span class="ov-badge ov-heal${fresh("heal")}">+${o.heal}</span>`);
  for (const [stat, delta] of Object.entries(o.statChanges)) {
    if (delta === 0) continue;
    badges.push(`<span class="ov-badge ov-stat${fresh(`stat:${stat}`)}">${delta > 0 ? "+" : ""}${delta} ${esc(stat)}</span>`);
  }
  for (const [status, delta] of Object.entries(o.statusDeltas)) {
    if (delta === 0) continue;
    badges.push(
      `<span class="ov-badge ov-status${fresh(`status:${status}`)}" style="${statusColorStyle(status)}">${delta > 0 ? "+" : ""}${delta} ${esc(status)}</span>`,
    );
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
  /** unit id → the overlay-badge keys that just appeared/changed THIS step, so
   * only a newly-revealed badge plays its reveal (the rest stay static — mirrors
   * the card-line `bc-line-new` fix). Absent/empty → every badge paints static. */
  newBadges?: Map<string, Set<string>>;
  /** unit ids whose Death is REVEALED this step (died flips false→true) — they
   * play the distinct death animation once, at the death-reveal moment (#065
   * item 4). On any later step in the beat they keep the static grey+✕ end-state. */
  dyingNew?: Set<string>;
}

/** The persistent coin marker (#065 slice 3): a unit that HOLDS the coin (the
 * most recent pairing's first striker — `coinHolderAt`) wears a small coin chip
 * pinned to its card. Unlike the per-beat damage/heal/status badges, this is a
 * PERSISTENT STATE marker: it stays on the holder across that pairing's strikes
 * and only moves when the next PairFaced re-flips the coin — so it gets its own
 * layer (`.coin-marker`, top-RIGHT) and a distinct gold look, never the red
 * `.ov-layer` delta pills. Empty for a non-holder; the shop/team/ladder never
 * pass a holder, so the card contract is unchanged for them. */
function coinMarkerHtml(holds: boolean): string {
  if (!holds) return "";
  return `<span class="coin-marker" aria-hidden="true" title="Holds the coin — struck first this pairing">◉</span>`;
}

function unitCard(
  u: BoardUnit,
  displayName: string,
  side: Side,
  registry: StatusRegistry,
  opts: { front: boolean; dead: boolean; dying: boolean; dyingNew?: boolean; hit: boolean; sel: boolean; overlay?: BeatOverlay | undefined; newKeys?: Set<string> | undefined; coin?: boolean },
): string {
  // The tooltip slot carries player-useful state, never the internal id (IA-7).
  const title = opts.dead
    ? `${displayName} — dead · tap to inspect`
    : `${displayName} — ${u.hp}/${u.maxHp} hp, ${u.pwr} pwr · tap to inspect`;
  return unitCardHtml({
    artName: u.name,
    label: displayName,
    side, // tints the name by team (#065 item 2)
    hp: `${u.hp}/${u.maxHp}`,
    pwr: u.pwr,
    statuses: u.statuses,
    registry,
    front: opts.front,
    dead: opts.dead,
    dying: opts.dying,
    dyingNew: opts.dyingNew === true,
    hit: opts.hit,
    sel: opts.sel,
    silenced: u.silenced,
    attrs: `data-unit="${esc(u.id)}"`,
    title,
    overlay: overlayBadgesHtml(opts.overlay, opts.newKeys),
    marker: coinMarkerHtml(opts.coin === true),
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
  coinHolder: string | undefined,
): string {
  const ov = (id: string): BeatOverlay | undefined => overlays?.by.get(id);
  const nb = (id: string): Set<string> | undefined => overlays?.newBadges?.get(id);
  const dyingNow = (id: string): boolean => overlays?.dyingNew?.has(id) === true;
  const holdsCoin = (id: string): boolean => coinHolder !== undefined && id === coinHolder;
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
        unitCard(u, name(u.id), side, registry, {
          front: i === 0,
          dead: false,
          dying: false,
          hit: hit.has(u.id),
          sel: u.id === selected,
          overlay: ov(u.id),
          newKeys: nb(u.id),
          coin: holdsCoin(u.id),
        }),
      )
      .join("") +
    dyingFromGrave
      .map((u) =>
        unitCard(u, name(u.id), side, registry, {
          front: false,
          dead: false,
          dying: true,
          dyingNew: dyingNow(u.id),
          hit: hit.has(u.id),
          sel: u.id === selected,
          overlay: ov(u.id),
          newKeys: nb(u.id),
          coin: holdsCoin(u.id),
        }),
      )
      .join("");
  const grave = restGrave
    .map((u) =>
      unitCard(u, name(u.id), side, registry, {
        front: false,
        dead: true,
        dying: false,
        hit: hit.has(u.id),
        sel: u.id === selected,
        coin: holdsCoin(u.id),
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
  coinHolder?: string,
): string {
  const center = centerHtml ?? `<div class="divider">${verdictHtml(board)}</div>`;
  return `${sideHtml(board, "A", name, hit, registry, selected, overlays, coinHolder)}<div class="stage-center">${center}</div>${sideHtml(board, "B", name, hit, registry, selected, overlays, coinHolder)}`;
}

/** Replaces `root`'s content with the board: side A's line, the stage centre,
 * side B's. `selected` marks the unit the inspector is open on; `centerHtml`
 * is the beat card / turn divider for the current beat; `overlays` carries the
 * per-unit beat badges + dying-in-place state (#065 slice 2); `coinHolder` is
 * the unit that wears the persistent coin marker (#065 slice 3). */
export function renderBoard(
  root: HTMLElement,
  board: BoardState,
  name: (id: string) => string,
  hit: Set<string>,
  registry: StatusRegistry,
  selected?: string,
  centerHtml?: string,
  overlays?: UnitOverlays,
  coinHolder?: string,
): void {
  root.innerHTML = boardHtml(board, name, hit, registry, selected, centerHtml, overlays, coinHolder);
}

// ============================================================================
// #082 slice D — the "compact board + acting full card" battle. The board is a
// flex column: a header bar (vs ghost · seed · Turn N), a 3-column grid of
// COMPACT side cards (with ACTING/TARGET/USED state) flanking the centre acting
// card, and a bottom trace strip. Pure presentation over the same boardAt
// projection; the acting card / trace strip HTML is computed in acting.ts and
// injected here so this file stays the single DOM writer.
// ============================================================================

/** The header-bar facts (#082 slice D). `opponent`/`seed` ride in from the run
 * (the viewer doesn't know them); `turn` comes off the board projection. */
export interface BattleHeader {
  opponent?: string | undefined;
  seed?: number | undefined;
  turn: number;
  ended?: { winner: Side | "draw"; turns: number } | undefined;
}

/** Per-unit battle state for the side cards (#082 slice D). */
export interface BattleAnnotations {
  acting?: string | undefined;
  target?: string | undefined;
  used: Set<string>;
}

/** A COMPACT side card with its battle state (ACTING/TARGET ribbon, USED dim). */
function sideCardB(
  u: BoardUnit,
  side: Side,
  front: boolean,
  dead: boolean,
  ctx: ActingCtx,
  anno: BattleAnnotations,
  registry: StatusRegistry,
  selected: string | undefined,
): string {
  const family = familyOf(ctx, u.id);
  const isActing = anno.acting === u.id;
  const isTarget = anno.target === u.id && !isActing;
  const used = anno.used.has(u.id) && !isActing && !isTarget;
  const topTag = isActing
    ? '<div class="ub-state st-acting">ACTING</div>'
    : isTarget
      ? '<div class="ub-state st-target">TARGET</div>'
      : "";
  const stateCls = isActing ? "is-acting" : isTarget ? "is-target" : "";
  const title = dead
    ? `${ctx.name(u.id)} — dead · tap to inspect`
    : `${ctx.name(u.id)} — ${u.hp}/${u.maxHp} hp, ${u.pwr} pwr · tap to inspect`;
  return unitCardHtml({
    variant: "compact",
    family,
    ...abilityLineFor(ctx, u.id),
    artName: u.name,
    label: ctx.name(u.id),
    side,
    hp: u.hp,
    pwr: u.pwr,
    statuses: u.statuses,
    registry,
    front,
    dead,
    sel: u.id === selected,
    silenced: u.silenced,
    used,
    topTag,
    classes: stateCls,
    attrs: `data-unit="${esc(u.id)}"`,
    title,
  });
}

function sideColumnB(
  board: BoardState,
  side: Side,
  ctx: ActingCtx,
  anno: BattleAnnotations,
  registry: StatusRegistry,
  selected: string | undefined,
): string {
  const lineUnits = board.lines[side];
  const dead = board.graves[side];
  const head =
    side === "A"
      ? '<div class="bv-side-head sh-a">◤ You</div>'
      : '<div class="bv-side-head sh-b">Ghost ◥</div>';
  const cards =
    lineUnits.map((u, i) => sideCardB(u, side, i === 0, false, ctx, anno, registry, selected)).join("") +
    dead.map((u) => sideCardB(u, side, false, true, ctx, anno, registry, selected)).join("");
  return `<div class="bv-side" data-side="${side}">${head}<div class="bv-stack">${cards || '<span class="bv-wiped">— no one standing —</span>'}</div></div>`;
}

/** The header bar: `vs ghost · <name>` · `seed N` · right-aligned `Turn N`. */
function headerHtml(h: BattleHeader): string {
  const opp = h.opponent !== undefined && h.opponent !== "" ? ` · <span class="bv-ghost">${esc(h.opponent)}</span>` : "";
  const seed = h.seed !== undefined ? `<span class="bv-seed">seed ${h.seed}</span>` : "";
  const turn = h.ended
    ? `<span class="bv-turn">${h.ended.winner === "draw" ? "Draw" : `Side ${h.ended.winner} wins`}</span>`
    : `<span class="bv-turn">Turn ${h.turn}</span>`;
  return `<div class="bv-header"><span class="bv-vs">vs ghost${opp}</span>${seed}${turn}</div>`;
}

export interface RenderBattleArgs {
  board: BoardState;
  ctx: ActingCtx;
  anno: BattleAnnotations;
  registry: StatusRegistry;
  selected?: string | undefined;
  centerHtml: string; // the acting card (acting.ts)
  traceHtml: string; // the bottom strip (acting.ts)
  header: BattleHeader;
}

/** The whole `#board` for the #082 acting-card battle. */
export function battleHtml(a: RenderBattleArgs): string {
  const grid = `<div class="bv-grid">${sideColumnB(a.board, "A", a.ctx, a.anno, a.registry, a.selected)}<div class="stage-center">${a.centerHtml}</div>${sideColumnB(a.board, "B", a.ctx, a.anno, a.registry, a.selected)}</div>`;
  return `${headerHtml(a.header)}${grid}${a.traceHtml}`;
}

export function renderBattle(root: HTMLElement, a: RenderBattleArgs): void {
  root.innerHTML = battleHtml(a);
}
