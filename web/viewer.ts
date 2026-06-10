// Replay viewer — steps a causal event log and shows the board at each step.
// Board state comes from the kernel's boardAt projection; cause chains come
// from the kernel's trace helpers; the inline log and the unit inspector live
// in their own modules. This file owns stepping, transport controls, selection,
// and one-line event formatting for the readout.

import {
  abilityRefDesc,
  ancestry,
  boardAt,
  deathCauseChain,
  displayNames,
  shortDesc,
  type BattleEvent,
  type NameOf,
  type StatusRegistry,
  type UnitDef,
} from "../src/index.js";
import { renderBoard } from "./board-render.js";
import { createBattleLog } from "./battle-log.js";
import { renderInspect, unitDefs } from "./inspect.js";

const BASE_STEP_MS = 350; // 1x ≈ 3 events/second

/** One-line readout for the selected event (display only; amounts/hp come off the event). */
function describeEvent(e: BattleEvent, name: NameOf): string {
  const hp = (after: number | undefined) => (after === undefined ? "" : ` → ${Math.max(0, after)} hp`);
  switch (e.type) {
    case "BattleStart":
      return `battle begins — ${e.teams.A.length} vs ${e.teams.B.length}`;
    case "TurnStart":
      return `turn ${e.turn} begins`;
    case "TurnEnd":
      return `turn ${e.turn} ends`;
    case "PairFaced":
      return `${name(e.a)} faces ${name(e.b)} — the coin says ${name(e.first)} strikes first`;
    case "Strike":
      return `${name(e.striker)} strikes ${name(e.defender)}`;
    case "Hurt": {
      const absorbed = e.absorbed !== undefined ? ` (${e.absorbed} absorbed by Shield)` : "";
      return `${name(e.unit)} takes ${e.amount}${absorbed}${hp(e.hpAfter)}`;
    }
    case "Heal":
      return `${name(e.unit)} heals ${e.amount}${hp(e.hpAfter)}`;
    case "Death":
      return `${name(e.unit)} dies`;
    case "Summon":
      return e.resurrected
        ? `${name(e.unit)} rises from the grave at ${e.atHp ?? 1} hp (side ${e.side})`
        : `${name(e.unit)} is summoned (${e.hp} hp, ${e.pwr} pwr, side ${e.side})`;
    case "StatusApplied":
      return `${name(e.unit)} gains ${e.status} ×${e.stacks} (total ${e.total})`;
    case "StatusRemoved":
      return e.remaining > 0
        ? `${name(e.unit)}'s ${e.status} drops by ${e.stacks} to ${e.remaining}`
        : `${name(e.unit)}'s ${e.status} is spent`;
    case "StatChanged":
      return `${name(e.unit)}'s ${e.stat} ${e.delta >= 0 ? "+" : ""}${e.delta} → ${e.now}`;
    case "Silenced":
      return `${name(e.unit)} is silenced`;
    case "Fatigue":
      return `fatigue ${e.amount} wears everyone down`;
    case "ChainBlocked":
      return `chain stopped: ${abilityRefDesc(e.ability, name)} stays quiet — an ability never triggers itself`;
    case "Intercepted":
      return `${abilityRefDesc(e.by, name)} intercepts a ${e.original}${e.unit !== undefined ? ` on ${name(e.unit)}` : ""}`;
    case "BattleEnd":
      return e.winner === "draw" ? `draw after ${e.turns} turns` : `side ${e.winner} wins after ${e.turns} turns`;
  }
}

/** Units a step should visibly mark — the event's subjects. */
function subjectsOf(e: BattleEvent): Set<string> {
  switch (e.type) {
    case "Strike":
      return new Set([e.striker, e.defender]);
    case "PairFaced":
      return new Set([e.a, e.b]);
    case "Hurt":
    case "Heal":
    case "Death":
    case "Summon":
    case "StatusApplied":
    case "StatusRemoved":
    case "StatChanged":
    case "Silenced":
      return new Set([e.unit]);
    case "Intercepted":
      return new Set(e.unit !== undefined ? [e.unit] : []);
    default:
      return new Set();
  }
}

interface ViewerEls {
  board: HTMLElement;
  prev: HTMLButtonElement;
  next: HTMLButtonElement;
  play: HTMLButtonElement;
  speed: HTMLSelectElement;
  scrub: HTMLInputElement;
  stepLabel: HTMLElement;
  eventDesc: HTMLElement;
  eventCause: HTMLElement;
  log: HTMLElement;
  inspect: HTMLElement;
}

/** What the battle ran on — the viewer derives ability/status descriptions
 * from the same data, so the inspector can never drift from the rules. */
export interface BattleContent {
  teams: { A: UnitDef[]; B: UnitDef[] };
  registry: StatusRegistry;
}

export interface Viewer {
  load(log: BattleEvent[], content: BattleContent): void;
  stop(): void;
  /** Detach the viewer's document-level listeners (and stop playback).
   * Anything that creates viewers more than once must destroy the old one,
   * or stale keydown handlers pile up on document. */
  destroy(): void;
}

export function createViewer(els: ViewerEls): Viewer {
  let log: BattleEvent[] = [];
  let name: NameOf = (id) => id;
  let step = 0;
  let timer: number | undefined;
  let defs = new Map<string, UnitDef>();
  let registry: StatusRegistry = {};
  let selected: { unit: string; status?: string } | undefined;

  const playing = () => timer !== undefined;

  const battleLog = createBattleLog(els.log, (eventId) => {
    pause();
    goTo(eventId);
  });

  function render(): void {
    const e = log[step];
    if (!e) return;
    const board = boardAt(log, step);
    renderBoard(els.board, board, name, subjectsOf(e), selected?.unit);
    battleLog.syncTo(step);
    els.scrub.value = String(step);
    els.stepLabel.textContent = `event ${step + 1}/${log.length} · turn ${e.turn}`;
    els.eventDesc.textContent = describeEvent(e, name);
    els.eventCause.innerHTML = causeHtml(e);
    els.prev.disabled = step === 0;
    els.next.disabled = step === log.length - 1;
    els.play.textContent = playing() ? "pause" : "play";
    els.inspect.hidden = selected === undefined;
    if (selected !== undefined) {
      renderInspect(els.inspect, {
        unitId: selected.unit,
        ...(selected.status !== undefined ? { status: selected.status } : {}),
        board,
        def: defs.get(selected.unit),
        registry,
        name,
      });
    }
  }

  /** The selected event's lineage: source, the causedBy chain, and a death's narrated why. */
  function causeHtml(e: BattleEvent): string {
    const rows: string[] = [];
    if (e.source !== "kernel") rows.push(`<div><span class="k">source</span> ${abilityRefDesc(e.source, name)}</div>`);
    if (e.type === "Death" && e.causedBy !== null) {
      const why = deathCauseChain(log, e.causedBy, name);
      if (why.length > 0) rows.push(`<div><span class="k">why</span> died ← ${why.join(" ← ")}</div>`);
    }
    const chain = ancestry(log, e.id);
    if (chain.length > 0) {
      const links = chain.map((a) => `<a href="#" data-goto="${a.id}">${shortDesc(a, name)}</a>`).join(" ← ");
      rows.push(`<div><span class="k">cause</span> ${links}</div>`);
    } else {
      rows.push(`<div><span class="k">cause</span> kernel beat — nothing caused it</div>`);
    }
    return rows.join("");
  }

  function goTo(n: number): void {
    step = Math.max(0, Math.min(n, log.length - 1));
    render();
  }

  function pause(): void {
    if (timer !== undefined) window.clearInterval(timer);
    timer = undefined;
  }

  function playPause(): void {
    if (playing()) {
      pause();
    } else {
      if (step >= log.length - 1) step = 0; // play again from the top
      timer = window.setInterval(() => {
        if (step >= log.length - 1) {
          pause();
          render();
          return;
        }
        goTo(step + 1);
      }, BASE_STEP_MS / Number(els.speed.value));
    }
    render();
  }

  els.prev.addEventListener("click", () => {
    pause();
    goTo(step - 1);
  });
  els.next.addEventListener("click", () => {
    pause();
    goTo(step + 1);
  });
  els.play.addEventListener("click", playPause);
  els.speed.addEventListener("change", () => {
    if (playing()) {
      pause();
      playPause(); // restart the interval at the new speed
    }
  });
  els.scrub.addEventListener("input", () => {
    pause();
    goTo(Number(els.scrub.value));
  });
  els.eventCause.addEventListener("click", (ev) => {
    const a = (ev.target as HTMLElement).closest("a[data-goto]");
    if (!a) return;
    ev.preventDefault();
    pause();
    goTo(Number(a.getAttribute("data-goto")));
  });
  // Selecting on the board: a unit card opens the inspector (again = close);
  // a status chip opens it with that status highlighted.
  els.board.addEventListener("click", (ev) => {
    const target = ev.target as HTMLElement;
    const card = target.closest("[data-unit]");
    if (!card) return;
    const unit = card.getAttribute("data-unit")!;
    const chip = target.closest("[data-status]");
    if (chip) selected = { unit, status: chip.getAttribute("data-status")! };
    else if (selected?.unit === unit && selected.status === undefined) selected = undefined;
    else selected = { unit };
    render();
  });
  els.inspect.addEventListener("click", (ev) => {
    if (!(ev.target as HTMLElement).closest("#ins-close")) return;
    selected = undefined;
    render();
  });
  // Arrow keys step when focus is not on a control with its own arrow behavior.
  // Named so destroy() can detach it — a document-level listener must not
  // outlive its viewer.
  const onKeydown = (ev: KeyboardEvent): void => {
    if (log.length === 0 || els.board.closest("[hidden]")) return;
    const tag = (document.activeElement?.tagName ?? "").toLowerCase();
    if (tag === "input" || tag === "select" || tag === "textarea") return;
    if (ev.key === "ArrowLeft" || ev.key === "ArrowRight") {
      ev.preventDefault();
      pause();
      goTo(step + (ev.key === "ArrowRight" ? 1 : -1));
    }
  };
  document.addEventListener("keydown", onKeydown);

  return {
    load(newLog: BattleEvent[], content: BattleContent): void {
      pause();
      log = newLog;
      name = displayNames(log);
      step = 0;
      defs = unitDefs(log, content.teams, content.registry);
      registry = content.registry;
      selected = undefined;
      battleLog.load(log, name);
      els.scrub.min = "0";
      els.scrub.max = String(log.length - 1);
      render();
    },
    stop: pause,
    destroy(): void {
      pause();
      document.removeEventListener("keydown", onKeydown);
    },
  };
}
