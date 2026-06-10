// Inline battle log — one compact line per event, appended in step with the
// playhead. Each event family gets an icon and one palette colour; unit names
// are tinted by side. Lines are built once per load and shown/hidden by
// syncTo, so scrubbing back truncates the log to the same point boardAt shows.
// Clicking a line drives the playhead there (the event panel surfaces its
// causal trace). Display only: every number comes off the event.

import { abilityRefDesc, type BattleEvent, type NameOf, type Side } from "../src/index.js";

export type SideOf = (unitId: string) => Side | undefined;

/** Unit id → side, read off BattleStart rosters and Summon events. */
export function sideMap(log: BattleEvent[]): SideOf {
  const sides = new Map<string, Side>();
  for (const e of log) {
    if (e.type === "BattleStart") {
      for (const side of ["A", "B"] as const) for (const r of e.teams[side]) sides.set(r.id, side);
    } else if (e.type === "Summon") {
      sides.set(e.unit, e.side);
    }
  }
  return (id) => sides.get(id);
}

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

interface Line {
  id: number; // event id this line narrates
  el: HTMLElement;
}

export interface BattleLog {
  load(log: BattleEvent[], name: NameOf): void;
  /** Show exactly the lines for events 0..step and highlight the current one. */
  syncTo(step: number): void;
}

export function createBattleLog(root: HTMLElement, onPick: (eventId: number) => void): BattleLog {
  let lines: Line[] = [];
  let shown = 0;
  let current: HTMLElement | undefined;

  root.addEventListener("click", (ev) => {
    const line = (ev.target as HTMLElement).closest("[data-id]");
    if (line) onPick(Number(line.getAttribute("data-id")));
  });

  function build(e: BattleEvent, name: NameOf, sideOf: SideOf): HTMLElement | null {
    const unit = (id: string): string => {
      const side = sideOf(id);
      return `<span class="u${side === "A" ? " ua" : side === "B" ? " ub" : ""}">${esc(name(id))}</span>`;
    };
    const hp = (after: number | undefined): string => (after === undefined ? "" : ` → ${Math.max(0, after)} hp`);
    const make = (cls: string, icon: string, html: string): HTMLElement => {
      const el = document.createElement("div");
      el.className = `log-line ${cls}`;
      el.setAttribute("data-id", String(e.id));
      el.innerHTML = `<span class="log-ico" aria-hidden="true">${icon}</span><span class="log-text">${html}</span>`;
      return el;
    };
    switch (e.type) {
      case "BattleStart":
        return make("ev-mark", "◆", `battle begins — ${e.teams.A.length} vs ${e.teams.B.length}`);
      case "TurnStart":
        return make("ev-turn", "", `— turn ${e.turn} —`);
      case "TurnEnd":
        return null; // structure only; its consequences (poison ticks, fatigue) speak for themselves
      case "PairFaced":
        return make("ev-dim", "◐", `${unit(e.a)} faces ${unit(e.b)} — ${unit(e.first)} strikes first`);
      case "Strike":
        return make("ev-strike", "⚔", `${unit(e.striker)} strikes ${unit(e.defender)}`);
      case "Hurt": {
        const absorbed = e.absorbed !== undefined ? ` (${e.absorbed} absorbed)` : "";
        return make("ev-hurt", "▼", `${unit(e.unit)} takes ${e.amount}${absorbed}${hp(e.hpAfter)}`);
      }
      case "Heal":
        return make("ev-heal", "▲", `${unit(e.unit)} heals ${e.amount}${hp(e.hpAfter)}`);
      case "Death":
        return make("ev-death", "☠", `${unit(e.unit)} dies`);
      case "Summon":
        return e.resurrected
          ? make("ev-summon", "✦", `${unit(e.unit)} rises from the grave at ${e.atHp ?? 1} hp`)
          : make("ev-summon", "✦", `${unit(e.unit)} is summoned (${e.hp} hp, ${e.pwr} pwr)`);
      case "StatusApplied":
        return make("ev-status", "◉", `${unit(e.unit)} gains ${esc(e.status)} ×${e.stacks} (${e.total})`);
      case "StatusRemoved":
        return e.remaining > 0
          ? make("ev-status", "○", `${unit(e.unit)}: ${esc(e.status)} −${e.stacks} (${e.remaining} left)`)
          : make("ev-status", "○", `${unit(e.unit)}: ${esc(e.status)} spent`);
      case "StatChanged":
        return make("ev-status", "Δ", `${unit(e.unit)} ${e.stat} ${e.delta >= 0 ? "+" : ""}${e.delta} → ${e.now}`);
      case "Silenced":
        return make("ev-warn", "⊘", `${unit(e.unit)} is silenced`);
      case "Fatigue":
        return make("ev-warn", "⌛", `fatigue ${e.amount} wears everyone down`);
      case "ChainBlocked":
        return make("ev-dim", "⊘", `chain stopped: ${esc(abilityRefDesc(e.ability, name))} stays quiet`);
      case "Intercepted": {
        const on = e.unit !== undefined ? ` on ${unit(e.unit)}` : "";
        return make("ev-block", "⛨", `${esc(abilityRefDesc(e.by, name))} blocks a ${e.original}${on}`);
      }
      case "BattleEnd":
        return make(
          "ev-mark",
          "◆",
          e.winner === "draw" ? `draw after ${e.turns} turns` : `side ${e.winner} wins · turn ${e.turns}`,
        );
    }
  }

  return {
    load(log: BattleEvent[], name: NameOf): void {
      const sideOf = sideMap(log);
      root.innerHTML = "";
      shown = 0;
      current = undefined;
      lines = [];
      for (const e of log) {
        const el = build(e, name, sideOf);
        if (el) lines.push({ id: e.id, el });
      }
    },

    syncTo(step: number): void {
      // Lines are in event order; show every line whose event has applied.
      let target = lines.length;
      while (target > 0 && lines[target - 1]!.id > step) target--;
      while (shown < target) root.append(lines[shown++]!.el);
      while (shown > target) lines[--shown]!.el.remove();
      current?.classList.remove("cur");
      current = shown > 0 ? lines[shown - 1]!.el : undefined;
      if (current) {
        current.classList.add("cur");
        // Keep the playhead's line in view by scrolling the log pane only —
        // scrollIntoView could yank the whole page along during playback.
        const above = current.offsetTop < root.scrollTop;
        const below = current.offsetTop + current.offsetHeight > root.scrollTop + root.clientHeight;
        if (above) root.scrollTop = current.offsetTop;
        else if (below) root.scrollTop = current.offsetTop + current.offsetHeight - root.clientHeight;
      }
    },
  };
}
