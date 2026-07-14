// Battle transcript and side lookup. The transcript is a non-card, fixed drawer
// keyed by the same event ids as scrub/trace/ancestry.

import { abilityRefDesc, shortDesc, type BattleEvent, type NameOf, type Side } from "../src/index.js";

export type SideOf = (unitId: string) => Side | undefined;

export function sideMap(log: BattleEvent[]): SideOf {
  const sides = new Map<string, Side>();
  for (const e of log) {
    if (e.type === "BattleStart") {
      for (const side of ["A", "B"] as const) for (const r of e.teams[side]) sides.set(r.id, side);
    } else if (e.type === "Summon") sides.set(e.unit, e.side);
  }
  return (id) => sides.get(id);
}

const esc = (s: string): string =>
  s.replace(/[&<>\"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '\"': "&quot;" })[c]!);

function eventSubject(e: BattleEvent, name: NameOf): string {
  switch (e.type) {
    case "Strike": return `${name(e.striker)} → ${name(e.defender)}`;
    case "Hurt": return `${name(e.unit)} −${e.amount} HP`;
    case "Heal": return `${name(e.unit)} +${e.amount} HP`;
    case "Death": return `${name(e.unit)} dies`;
    case "Summon": return `${name(e.unit)} ${e.resurrected ? "resurrects" : "enters"}`;
    case "StatusApplied": return `${name(e.unit)} gains ${e.status} ${e.stacks}`;
    case "StatusRemoved": return `${name(e.unit)} loses ${e.status} ${e.stacks}`;
    case "StatChanged": return `${name(e.unit)} ${e.stat} ${e.delta >= 0 ? "+" : ""}${e.delta}`;
    case "Silenced": return `${name(e.unit)} silenced`;
    case "Intercepted": return `${name(e.by.unit)} intercepted ${e.original}`;
    case "ChainBlocked": return `${name(e.ability.unit)} chain blocked`;
    case "PairFaced": return `${name(e.a)} ↔ ${name(e.b)} · ${name(e.first)} first`;
    case "Fatigue": return `${e.amount} damage`;
    case "BattleEnd": return e.winner === "draw" ? "draw" : `side ${e.winner} wins`;
    default: return shortDesc(e, name);
  }
}

export function transcriptHtml(log: BattleEvent[], current: number, name: NameOf): string {
  const rows = log.map((e) => {
    const source = e.source === "kernel" ? "kernel" : abilityRefDesc(e.source, name);
    const cause = e.causedBy === null ? "root" : `← #${e.causedBy}`;
    return `<button type="button" class="bt-row${e.id === current ? " is-current" : ""}" data-log-event="${e.id}" data-caused-by="${e.causedBy ?? ""}"><span class="bt-id">#${e.id}</span><span class="bt-type">${esc(e.type)}</span><span class="bt-subject">${esc(eventSubject(e, name))}</span><span class="bt-meta">${esc(source)} · ${cause}</span></button>`;
  }).join("");
  return `<div class="bt-head"><div><span class="bt-k">full event transcript</span><strong>${log.length} deterministic events</strong></div><button type="button" class="bt-close" aria-label="Close event transcript">✕</button></div><div class="bt-list">${rows}</div>`;
}
