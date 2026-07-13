import { statusActionsOf, unitActionsOf } from "../src/types.js";
// Unit inspector — select a unit (or a status chip) on the board and see what
// it does: its abilities and current statuses, each with a description derived
// from the DSL data by the kernel's describe helpers. The replay position
// decides what shows: statuses and stats come off boardAt's projection.
// Display only; the registry and unit defs are the same data battle() ran on.

import {
  abilityChips,
  describeAbilitySegments,
  describeStatus,
  describeStatusSegments,
  type Ability,
  type AbilityRegistry,
  type BattleEvent,
  type BoardState,
  type BoardUnit,
  type DescribeSegment,
  type Family,
  type StatusDef,
  type StatusRegistry,
  type UnitDef,
} from "../src/index.js";
import { triggerIcon } from "./glyphs.js";
import { chipsHtml } from "./status-chips.js";
import { nameFamily, unitCardHtml } from "./unit-card.js";

export { chipsHtml } from "./status-chips.js";

/**
 * Unit instance id → its UnitDef. Roster units map by line order; a summoned
 * unit's def is recovered from the summon effect on its source ability — so
 * even mid-battle arrivals can show their abilities.
 */
/** A unit's ability bodies — resolved from its single `ability` ref through the
 * registry (PRD #081). */
function unitAbilities(def: UnitDef, abilities: AbilityRegistry): Ability[] {
  return unitActionsOf(def, abilities);
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

function statusTag(def: StatusDef): string {
  const mods = (["pwr", "hp"] as const)
    .flatMap((stat) => def.statMods?.[stat] ? [`${def.statMods[stat]! > 0 ? "+" : ""}${def.statMods[stat]} ${stat}/stack`] : []);
  return mods.length > 0 ? `stacks · ${mods.join(" · ")}` : "stacks";
}

function statusInspectEntity(name: string, def: StatusDef): InspectEntity {
  const action = statusActionsOf(def)[0];
  const chips = action !== undefined ? abilityChips(action) : undefined;
  return { kind: "status", name, summary: describeStatus(def), tag: statusTag(def), family: nameFamily(def.name), trigger: chips?.trigger, target: chips?.target, action: chips?.action };
}

function inspectRefAttrs(entity: InspectEntity): string {
  return [
    `data-inspect-kind="${entity.kind}"`,
    `data-inspect-name="${esc(entity.name)}"`,
    `data-inspect-summary="${esc(entity.summary ?? "")}"`,
    entity.family !== undefined ? `data-inspect-family="${entity.family}"` : "",
    entity.tag !== undefined ? `data-inspect-tag="${esc(entity.tag)}"` : "",
    entity.hp !== undefined ? `data-inspect-hp="${esc(String(entity.hp))}"` : "",
    entity.pwr !== undefined ? `data-inspect-pwr="${esc(String(entity.pwr))}"` : "",
    entity.trigger !== undefined ? `data-inspect-trigger="${esc(entity.trigger)}"` : "",
    entity.target !== undefined ? `data-inspect-target="${esc(entity.target)}"` : "",
    entity.action !== undefined ? `data-inspect-action="${esc(entity.action)}"` : "",
  ].filter(Boolean).join(" ");
}

/** Description segments link card-bearing references back into the same
 * overlay. Grammar atoms remain Codex rows. */
function segmentsHtml(segs: DescribeSegment[], registry: StatusRegistry): string {
  return segs
    .map((s) => {
      if (s.statusRef !== undefined && registry[s.statusRef] !== undefined) {
        const def = registry[s.statusRef]!;
        return `<button type="button" class="ins-ref entity-ref" data-status-ref="${esc(s.statusRef)}" ${inspectRefAttrs(statusInspectEntity(s.statusRef, def))}>${esc(s.text)}</button>`;
      }
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
  /** Current values shown on the full shared Unit card. */
  hp: string | number;
  pwr: string | number;
  /** Optional non-card state/explanation (dead, level, shop price…). */
  state?: string | undefined;
  /** Run progression repeated on the full shared card for touch/resume reading. */
  progression?: { state: string; progress: string } | undefined;
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
  const { title, hp, pwr, state, progression, def, statuses, registry, abilities: abilityRegistry, highlight, silenced, noStatuses } = args;
  const rows: string[] = [];
  const abilities = def !== undefined ? unitAbilities(def, abilityRegistry) : [];
  const primary = abilities[0];
  const primaryId = def?.abilities?.[0] ?? def?.ability;
  const primaryDef = primaryId !== undefined ? abilityRegistry[primaryId] : undefined;
  const primaryChips = primary !== undefined ? abilityChips(primary) : undefined;
  rows.push(`<div class="ins-head"><span class="ins-entity-kind">unit</span>${state !== undefined ? `<span class="ins-stats">${state}</span>` : ""}<button type="button" id="ins-close" title="Close">✕</button></div>`);
  rows.push(unitCardHtml({
    surface: "full",
    kind: "unit",
    artName: def?.name ?? title,
    label: title,
    hp,
    pwr,
    registry,
    statuses,
    family: primaryDef?.family ?? nameFamily(def?.name ?? title),
    variant: "full",
    ...(primaryDef !== undefined ? { abilityLabel: primaryDef.name } : {}),
    ...(primaryChips?.trigger !== undefined ? { trigger: primaryChips.trigger } : {}),
    ...(primaryChips?.triggerGlyph !== undefined ? { triggerGlyph: primaryChips.triggerGlyph } : {}),
    ...(primaryChips?.target !== undefined ? { target: primaryChips.target } : {}),
    ...(primaryChips?.action !== undefined ? { action: primaryChips.action } : {}),
    ...(silenced !== undefined ? { silenced } : {}),
    ...(progression !== undefined ? { progression: progression.state, progress: progression.progress } : {}),
    attrs: "data-inspector-card",
    title,
  }));
  if (silenced) rows.push(`<div class="ins-warn" data-non-card="explanation">⊘ silenced — its own abilities are dead for the battle</div>`);

  rows.push(`<div class="ins-k" data-non-card="explanation">abilities</div>`);
  const mentioned: DescribeSegment[] = [];
  if (abilities.length === 0) {
    rows.push(`<div class="ins-dim" data-non-card="explanation">none — it only strikes</div>`);
  } else {
    const abilityIds = def?.abilities ?? (def?.ability !== undefined ? [def.ability] : []);
    for (const [abilityIndex, ab] of abilities.entries()) {
      const segs = describeAbilitySegments(ab);
      const abilityName = abilityIds[abilityIndex] ?? `Ability ${abilityIndex + 1}`;
      const chips = abilityChips(ab);
      const abilityDef = abilityRegistry[abilityName];
      mentioned.push(...segs);
      rows.push(`<div class="ins-row ins-ab" data-non-card="explanation"><button type="button" class="entity-ref ins-ability-ref" ${inspectRefAttrs({ kind: "ability", name: abilityName, summary: segs.map((s) => s.text).join(""), family: abilityDef?.family, tag: abilityDef?.family, action: chips.action })}>${esc(abilityName)}</button><span>${segmentsHtml(segs, registry)}</span></div>`);
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
      const statusName = sdef !== undefined
        ? `<button type="button" class="entity-ref ins-status-ref" ${inspectRefAttrs(statusInspectEntity(s.status, sdef))}>${esc(s.status)} ×${s.stacks}</button>`
        : `<b>${esc(s.status)} ×${s.stacks}</b>`;
      rows.push(
        `<div class="ins-row${sel}" data-status-row="${esc(s.status)}" data-non-card="explanation"><span class="ins-ico">◉</span><span>${statusName} — ${segmentsHtml(segs, registry)}</span></div>`,
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

export type InspectEntityKind = "unit" | "ability" | "status" | "summon";

export interface InspectEntity {
  kind: InspectEntityKind;
  name: string;
  summary?: string | undefined;
  family?: Family | undefined;
  tag?: string | undefined;
  hp?: string | number | undefined;
  pwr?: string | number | undefined;
  trigger?: string | undefined;
  triggerGlyph?: string | undefined;
  target?: string | undefined;
  action?: string | undefined;
  statuses?: readonly { status: string; stacks: number }[] | undefined;
  registry?: StatusRegistry | undefined;
}

/** Every card-bearing reference replaces the overlay with exactly one full
 * shared chassis. Any prose around it is explicitly non-card explanation. */
export function renderEntityInspect(root: HTMLElement, entity: InspectEntity): void {
  const registry = entity.registry ?? {};
  root.innerHTML =
    `<div class="ins-head"><span class="ins-entity-kind">${esc(entity.kind)}</span><button type="button" id="ins-close" title="Close">✕</button></div>` +
    unitCardHtml({
      surface: "full",
      kind: entity.kind,
      artName: entity.name,
      label: entity.name,
      hp: entity.hp ?? "—",
      pwr: entity.pwr ?? "—",
      registry,
      ...(entity.statuses !== undefined ? { statuses: entity.statuses } : {}),
      family: entity.family ?? nameFamily(entity.name),
      variant: "full",
      ...(entity.tag !== undefined ? { tag: entity.tag } : {}),
      ...(entity.kind !== "ability" && entity.trigger !== undefined ? { trigger: entity.trigger } : {}),
      ...(entity.triggerGlyph !== undefined ? { triggerGlyph: entity.triggerGlyph } : {}),
      ...(entity.kind !== "ability" && entity.target !== undefined ? { target: entity.target } : {}),
      ...(entity.action !== undefined || (entity.kind === "ability" && entity.summary !== undefined) ? { action: entity.action ?? entity.summary! } : {}),
      attrs: "data-inspector-card",
      title: entity.name,
    }) +
    (entity.summary ? `<div class="ins-row ins-summary" data-non-card="explanation">${esc(entity.summary)}</div>` : "");
}

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
    const ref = target.closest<HTMLElement>("[data-inspect-kind]");
    if (ref !== null) {
      ev.stopPropagation(); // rendering removes the clicked node before document's outside-tap handler runs
      const kind = ref.dataset.inspectKind as InspectEntityKind;
      renderEntityInspect(el, {
        kind,
        name: ref.dataset.inspectName ?? "Entity",
        summary: ref.dataset.inspectSummary,
        family: ref.dataset.inspectFamily as Family | undefined,
        tag: ref.dataset.inspectTag,
        hp: ref.dataset.inspectHp,
        pwr: ref.dataset.inspectPwr,
        trigger: ref.dataset.inspectTrigger,
        target: ref.dataset.inspectTarget,
        action: ref.dataset.inspectAction,
      });
      positionOverlay();
    }
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
    hp: `${unit.hp}/${unit.maxHp}`,
    pwr: unit.pwr,
    ...(dead ? { state: `<span class="ins-dead">${triggerIcon("death")} dead</span>` } : {}),
    def,
    statuses: unit.statuses,
    registry,
    abilities,
    ...(status !== undefined ? { highlight: status } : {}),
    silenced: unit.silenced,
    ...(dead ? { noStatuses: "none — the corpse is clean" } : {}),
  });
}
