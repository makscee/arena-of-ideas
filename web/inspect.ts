// Unit inspector — select a unit (or a status chip) on the board and see what
// it does: its abilities and current statuses, each with a description derived
// from the DSL data by the kernel's describe helpers. The replay position
// decides what shows: statuses and stats come off boardAt's projection.
// Display only; the registry and unit defs are the same data battle() ran on.

import {
  describeAbility,
  describeStatus,
  type BattleEvent,
  type BoardState,
  type BoardUnit,
  type StatusRegistry,
  type UnitDef,
} from "../src/index.js";

/**
 * Unit instance id → its UnitDef. Roster units map by line order; a summoned
 * unit's def is recovered from the summon effect on its source ability — so
 * even mid-battle arrivals can show their abilities.
 */
export function unitDefs(
  log: BattleEvent[],
  teams: { A: UnitDef[]; B: UnitDef[] },
  registry: StatusRegistry,
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
      // The ability that summoned it names the def (first summon effect).
      const holder = defs.get(e.source.unit);
      const abilities = e.source.status !== undefined ? registry[e.source.status]?.abilities : holder?.abilities;
      const effect = abilities?.[e.source.ability]?.effects.find((f) => f.kind === "summon");
      if (effect?.kind === "summon") defs.set(e.unit, effect.unit);
    }
  }
  return defs;
}

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

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
  /** Highlight this status row (a chip was clicked). */
  highlight?: string;
  silenced?: boolean;
  /** The dim line shown when `statuses` is empty. */
  noStatuses?: string;
}

/** Render the inspector body: head, abilities, statuses — every description
 * derived from the DSL data by the kernel's describe helpers. */
export function renderUnitInspect(root: HTMLElement, args: UnitInspectArgs): void {
  const { title, state, def, statuses, registry, highlight, silenced, noStatuses } = args;
  const rows: string[] = [];
  rows.push(
    `<div class="ins-head"><span class="ins-name">${esc(title)}</span><span class="ins-stats">${state}</span><button type="button" id="ins-close" title="Close">✕</button></div>`,
  );
  if (silenced) rows.push(`<div class="ins-warn">⊘ silenced — its own abilities are dead for the battle</div>`);

  rows.push(`<div class="ins-k">abilities</div>`);
  const abilities = def?.abilities ?? [];
  if (abilities.length === 0) {
    rows.push(`<div class="ins-dim">none — it only strikes</div>`);
  } else {
    for (const ab of abilities) {
      rows.push(
        `<div class="ins-row ins-ab"><span class="ins-ico">⚙</span><span>${esc(describeAbility(ab))}</span></div>`,
      );
    }
  }

  rows.push(`<div class="ins-k">statuses</div>`);
  if (statuses.length === 0) {
    rows.push(`<div class="ins-dim">${esc(noStatuses ?? "none")}</div>`);
  } else {
    for (const s of statuses) {
      const sdef = registry[s.status];
      const text = sdef ? describeStatus(sdef) : "(unknown status)";
      const sel = s.status === highlight ? " sel" : "";
      rows.push(
        `<div class="ins-row${sel}"><span class="ins-ico">◉</span><span><b>${esc(s.status)} ×${s.stacks}</b> — ${esc(text)}</span></div>`,
      );
    }
  }

  root.innerHTML = rows.join("");
}

/** Render the inspector panel for the selected unit at the current position. */
export function renderInspect(root: HTMLElement, args: InspectArgs): void {
  const { unitId, status, board, def, registry, name } = args;
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
    ...(status !== undefined ? { highlight: status } : {}),
    silenced: unit.silenced,
    ...(dead ? { noStatuses: "none — the corpse is clean" } : {}),
  });
}
