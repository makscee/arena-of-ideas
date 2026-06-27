// Unit inspector — select a unit (or a status chip) on the board and see what
// it does: its abilities and current statuses, each with a description derived
// from the DSL data by the kernel's describe helpers. The replay position
// decides what shows: statuses and stats come off boardAt's projection.
// Display only; the registry and unit defs are the same data battle() ran on.

import {
  describeAbilitySegments,
  describeStatus,
  describeStatusSegments,
  type Ability,
  type AbilityDef,
  type AbilityRegistry,
  type BattleEvent,
  type BoardState,
  type BoardUnit,
  type DescribeSegment,
  type StatusRegistry,
  type UnitDef,
} from "../src/index.js";
import { statusChipStyle } from "./status-color.js";

/**
 * Unit instance id → its UnitDef. Roster units map by line order; a summoned
 * unit's def is recovered from the summon effect on its source ability — so
 * even mid-battle arrivals can show their abilities.
 */
/** A unit's ability bodies — resolved from its single `ability` ref through the
 * registry (PRD #081). */
function unitAbilities(def: UnitDef, abilities: AbilityRegistry): Ability[] {
  return [abilities[def.ability]].filter((a): a is AbilityDef => a !== undefined);
}

export function unitDefs(
  log: BattleEvent[],
  teams: { A: UnitDef[]; B: UnitDef[] },
  registry: StatusRegistry,
  abilities: AbilityRegistry,
): Map<string, UnitDef> {
  const defs = new Map<string, UnitDef>();
  for (const e of log) {
    if (e.type === "BattleStart") {
      for (const side of ["A", "B"] as const) {
        e.teams[side].forEach((r, i) => {
          const def = teams[side][i];
          if (def) defs.set(r.id, def);
        });
      }
    } else if (e.type === "Summon" && !e.resurrected && e.source !== "kernel") {
      // The ability that summoned it names the def (first summon effect). A
      // status-sourced summon reads the status's ability list; a unit-sourced
      // one resolves the holder's `ability` ref (PRD #081).
      const holder = defs.get(e.source.unit);
      const abilityList =
        e.source.status !== undefined
          ? registry[e.source.status]?.abilities
          : holder !== undefined
            ? unitAbilities(holder, abilities)
            : undefined;
      const effect = abilityList?.[e.source.ability]?.effects.find((f) => f.kind === "summon");
      if (effect?.kind === "summon") defs.set(e.unit, effect.unit);
    }
  }
  return defs;
}

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

/** Description segments → HTML. Every term links to where it's defined: a
 * status name the registry knows becomes a tappable ref revealed in-panel
 * (IA-1 — "what does Poison do?" answered where Poison is said); a Part term
 * (every trigger/interceptor/condition/selector/effect, #078 slice 3) becomes
 * an anchor to its codex Part card, so the codex is the complete tappable
 * vocabulary (the global #codex/ handler in main.ts opens the codex and
 * navigates). statusRef wins when a term is both (an applyStatus name is a
 * status AND an effect payload) — the in-panel reveal is the closer answer.
 * Everything else is the plain describe* text. */
function segmentsHtml(segs: DescribeSegment[], registry: StatusRegistry): string {
  return segs
    .map((s) => {
      if (s.statusRef !== undefined && registry[s.statusRef] !== undefined)
        return `<button type="button" class="ins-ref" data-status-ref="${esc(s.statusRef)}">${esc(s.text)}</button>`;
      if (s.partRef !== undefined) {
        const frag = `codex/part/${s.partRef.family}/${s.partRef.kind}`;
        return `<a class="ins-ref ins-partref" href="#${esc(frag)}" data-part="${esc(s.partRef.family)}:${esc(s.partRef.kind)}">${esc(s.text)}</a>`;
      }
      return esc(s.text);
    })
    .join("");
}

/** Every status the given segments reference, chased transitively through the
 * registry (a referenced status's own definition may reference further ones). */
function referencedStatuses(segs: DescribeSegment[], registry: StatusRegistry): string[] {
  const seen: string[] = [];
  const queue = segs.filter((s) => s.statusRef !== undefined).map((s) => s.statusRef!);
  while (queue.length > 0) {
    const name = queue.shift()!;
    const def = registry[name];
    if (def === undefined || seen.includes(name)) continue;
    seen.push(name);
    for (const s of describeStatusSegments(def)) if (s.statusRef !== undefined) queue.push(s.statusRef);
  }
  return seen;
}

/** Status chips, shared by every chip render site (board, shop, ladder): the
 * title carries the derived definition, not just name×count (IA-5). */
export function chipsHtml(
  statuses: readonly { status: string; stacks: number }[] | undefined,
  registry: StatusRegistry,
): string {
  return (statuses ?? [])
    .map((s) => {
      const def = registry[s.status];
      const title = `${s.status} ×${s.stacks}${def !== undefined ? ` — ${describeStatus(def)}` : ""}`;
      // Per-status colour (#065 item 2): a hash-stable hue per status name, so a
      // given status is the same colour on the chip as on its overlay badge and
      // card line. The class/data-attr contract is untouched (probes, hit-targets
      // and the inspector still key off `.chip`/`data-status`) — only an inline
      // tint is layered on, kept bright enough to read on the dim chip bg.
      return `<span class="chip" data-status="${esc(s.status)}" style="${statusChipStyle(s.status)}" title="${esc(title)}">${esc(s.status.slice(0, 3))}${s.stacks}</span>`;
    })
    .join("");
}

function findUnit(board: BoardState, id: string): { unit: BoardUnit; dead: boolean } | undefined {
  for (const side of ["A", "B"] as const) {
    const live = board.lines[side].find((u) => u.id === id);
    if (live) return { unit: live, dead: false };
    const grave = board.graves[side].find((u) => u.id === id);
    if (grave) return { unit: grave, dead: true };
  }
  return undefined;
}

export interface InspectArgs {
  unitId: string;
  /** Highlight this status row (a chip was clicked). */
  status?: string;
  board: BoardState;
  def: UnitDef | undefined;
  registry: StatusRegistry;
  /** The ability registry a unit's `ability` ref resolves through (PRD #081). */
  abilities: AbilityRegistry;
  name: (id: string) => string;
}

/** What renderUnitInspect needs — board-free, so the run screen can inspect
 * shop offers and line units with the same derived descriptions the battle
 * inspector shows. The head's state line arrives pre-formatted as HTML. */
export interface UnitInspectArgs {
  title: string;
  /** The head's state line, as HTML (hp/pwr numbers, "☠ dead", level…). */
  state: string;
  def: UnitDef | undefined;
  /** Attached (battle) or initial (shop) statuses, in order. */
  statuses: { status: string; stacks: number }[];
  registry: StatusRegistry;
  /** The ability registry a unit's `ability` ref resolves through (PRD #081). */
  abilities: AbilityRegistry;
  /** Highlight this status row (a chip was clicked). */
  highlight?: string;
  silenced?: boolean;
  /** The dim line shown when `statuses` is empty. */
  noStatuses?: string;
}

/** Render the inspector body: head, abilities, statuses — every description
 * derived from the DSL data by the kernel's describe helpers. */
export function renderUnitInspect(root: HTMLElement, args: UnitInspectArgs): void {
  const { title, state, def, statuses, registry, abilities: abilityRegistry, highlight, silenced, noStatuses } = args;
  const rows: string[] = [];
  rows.push(
    `<div class="ins-head"><span class="ins-name">${esc(title)}</span><span class="ins-stats">${state}</span><button type="button" id="ins-close" title="Close">✕</button></div>`,
  );
  if (silenced) rows.push(`<div class="ins-warn">⊘ silenced — its own abilities are dead for the battle</div>`);

  rows.push(`<div class="ins-k">abilities</div>`);
  const abilities = def !== undefined ? unitAbilities(def, abilityRegistry) : [];
  const mentioned: DescribeSegment[] = []; // every segment shown — its refs get definition rows below
  if (abilities.length === 0) {
    rows.push(`<div class="ins-dim">none — it only strikes</div>`);
  } else {
    for (const ab of abilities) {
      const segs = describeAbilitySegments(ab);
      mentioned.push(...segs);
      rows.push(`<div class="ins-row ins-ab"><span class="ins-ico">⚙</span><span>${segmentsHtml(segs, registry)}</span></div>`);
    }
  }

  rows.push(`<div class="ins-k">statuses</div>`);
  if (statuses.length === 0) {
    rows.push(`<div class="ins-dim">${esc(noStatuses ?? "none")}</div>`);
  } else {
    for (const s of statuses) {
      const sdef = registry[s.status];
      const segs = sdef ? describeStatusSegments(sdef) : [{ text: "(unknown status)" }];
      mentioned.push(...segs);
      const sel = s.status === highlight ? " sel" : "";
      rows.push(
        `<div class="ins-row${sel}" data-status-row="${esc(s.status)}"><span class="ins-ico">◉</span><span><b>${esc(s.status)} ×${s.stacks}</b> — ${segmentsHtml(segs, registry)}</span></div>`,
      );
    }
  }

  // Definitions for every status the text above mentions but the unit does
  // not carry — hidden until its ref is tapped, revealed in this same panel.
  const attached = new Set(statuses.map((s) => s.status));
  for (const name of referencedStatuses(mentioned, registry)) {
    if (attached.has(name)) continue;
    rows.push(
      `<div class="ins-row ins-refdef sel" data-status-def="${esc(name)}" hidden>` +
        `<span class="ins-ico">◉</span><span><b>${esc(name)}</b> — ${segmentsHtml(describeStatusSegments(registry[name]!), registry)}</span></div>`,
    );
  }

  root.innerHTML = rows.join("");
}

// ---------------------------------------------------------------------------
// The one inspector overlay (LS-3 / IA-2). A single instance app-wide: a
// popover pinned to the clicked card on a desk, a bottom sheet at phone
// width. position: fixed — opening or closing never reflows the page, and it
// is always where the user clicked. Owners (shop, ladder, viewer) render
// their content in; the overlay owns closing (✕, Escape, outside tap) and
// status-ref taps, so every call site gets them for free.
// ---------------------------------------------------------------------------

export interface InspectOverlayArgs {
  /** The clicked card — the desktop popover pins to it. Owners re-resolve it
   * on every render (innerHTML re-renders replace card nodes). */
  anchor: HTMLElement | null;
  /** Renders the panel body (renderUnitInspect / renderInspect). */
  render: (body: HTMLElement) => void;
  /** The overlay closed itself (✕, Escape, outside tap, another owner opened,
   * a screen change) — the owner clears its selection state here. */
  onClose: () => void;
}

const PHONE_WIDTH = "(max-width: 700px)";
const EDGE_MARGIN = 8; // px the popover keeps from the viewport edge
const ANCHOR_GAP = 6; // px between the card and the popover

let overlayEl: HTMLElement | undefined;
let current: { key: string; args: InspectOverlayArgs } | undefined;
let openedThisTask = false; // the click that opened must not read as an outside tap

/** Show the inspector overlay, replacing whatever it held. `key` names the
 * owner: a different owner's open dismisses the previous one (single
 * instance); the same owner's open is an in-place update. */
export function openInspectOverlay(key: string, args: InspectOverlayArgs): void {
  const el = ensureOverlay();
  if (current !== undefined && current.key !== key) {
    const prev = current;
    current = undefined;
    prev.args.onClose(); // the other owner clears its selection
  }
  current = { key, args };
  el.hidden = false;
  args.render(el);
  positionOverlay();
  openedThisTask = true;
  window.setTimeout(() => {
    openedThisTask = false;
  }, 0);
}

/** Owner-initiated close — its selection cleared through its own logic. Only
 * the open panel's owner may close it; no onClose echo. */
export function closeInspectOverlay(key: string): void {
  if (current === undefined || current.key !== key) return;
  current = undefined;
  if (overlayEl !== undefined) overlayEl.hidden = true;
}

/** Close from the overlay's side (✕, Escape, outside tap, screen change) —
 * fires the owner's onClose so its selection state follows. */
export function dismissInspectOverlay(): void {
  if (current === undefined) return;
  const prev = current;
  current = undefined;
  if (overlayEl !== undefined) overlayEl.hidden = true;
  prev.args.onClose();
}

function ensureOverlay(): HTMLElement {
  if (overlayEl !== undefined) return overlayEl;
  const el = document.createElement("div");
  el.id = "inspect-overlay";
  el.hidden = true;
  document.body.append(el);
  el.addEventListener("click", (ev) => {
    const target = ev.target as HTMLElement;
    if (target.closest("#ins-close") !== null) {
      dismissInspectOverlay();
      return;
    }
    const ref = target.closest("[data-status-ref]");
    if (ref !== null) revealStatusDef(el, ref.getAttribute("data-status-ref")!);
  });
  document.addEventListener("keydown", (ev) => {
    if (ev.key === "Escape" && current !== undefined) dismissInspectOverlay();
  });
  document.addEventListener("click", (ev) => {
    if (current === undefined || openedThisTask) return;
    const target = ev.target as Node;
    if (el.contains(target)) return;
    // A click on the anchor card is the owner's toggle, not an outside tap.
    if (current.args.anchor?.contains(target) ?? false) return;
    dismissInspectOverlay();
  });
  // The popover is fixed; the page scrolls and resizes under it.
  window.addEventListener("scroll", () => positionOverlay(), true);
  window.addEventListener("resize", () => positionOverlay());
  overlayEl = el;
  return el;
}

/** A status ref was tapped: reveal its definition in the same panel — the
 * unit's own status row if it carries it, the hidden ref row otherwise. */
function revealStatusDef(panel: HTMLElement, name: string): void {
  const row =
    panel.querySelector<HTMLElement>(`[data-status-row="${CSS.escape(name)}"]`) ??
    panel.querySelector<HTMLElement>(`[data-status-def="${CSS.escape(name)}"]`);
  if (row === null) return;
  row.hidden = false;
  row.classList.add("sel");
  positionOverlay(); // the panel grew — keep it in the viewport
  row.scrollIntoView({ block: "nearest" });
}

/** Popover above/below the anchor on a desk (clamped to the viewport),
 * bottom sheet at phone width — the CSS classes carry the two shapes. */
function positionOverlay(): void {
  if (overlayEl === undefined || current === undefined || overlayEl.hidden) return;
  const el = overlayEl;
  if (window.matchMedia(PHONE_WIDTH).matches) {
    el.classList.add("sheet");
    el.classList.remove("pop");
    el.style.left = "";
    el.style.top = "";
    return;
  }
  el.classList.add("pop");
  el.classList.remove("sheet");
  const anchor = current.args.anchor;
  if (anchor === null || !anchor.isConnected) return; // keep the last spot
  const r = anchor.getBoundingClientRect();
  const left = Math.max(EDGE_MARGIN, Math.min(r.left, window.innerWidth - el.offsetWidth - EDGE_MARGIN));
  let top = r.bottom + ANCHOR_GAP;
  if (top + el.offsetHeight > window.innerHeight - EDGE_MARGIN) {
    const above = r.top - el.offsetHeight - ANCHOR_GAP;
    top = above >= EDGE_MARGIN ? above : Math.max(EDGE_MARGIN, window.innerHeight - el.offsetHeight - EDGE_MARGIN);
  }
  el.style.left = `${left}px`;
  el.style.top = `${top}px`;
}

/** Render the inspector panel for the selected unit at the current position. */
export function renderInspect(root: HTMLElement, args: InspectArgs): void {
  const { unitId, status, board, def, registry, abilities, name } = args;
  const found = findUnit(board, unitId);
  const title = esc(name(unitId));
  if (!found) {
    root.innerHTML =
      `<div class="ins-head"><span class="ins-name">${title}</span><button type="button" id="ins-close" title="Close">✕</button></div>` +
      `<div class="ins-dim">not on the board at this point — step forward to meet it</div>`;
    return;
  }
  const { unit, dead } = found;
  renderUnitInspect(root, {
    title: name(unitId),
    state: dead ? '<span class="ins-dead">☠ dead</span>' : `${unit.hp}/${unit.maxHp} hp · ${unit.pwr} pwr`,
    def,
    statuses: unit.statuses,
    registry,
    abilities,
    ...(status !== undefined ? { highlight: status } : {}),
    silenced: unit.silenced,
    ...(dead ? { noStatuses: "none — the corpse is clean" } : {}),
  });
}
