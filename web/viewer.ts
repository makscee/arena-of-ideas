// Replay viewer — steps a causal event log and shows the board at each step.
// Board state comes from the kernel's boardAt projection; cause chains come
// from the kernel's trace helpers; the inline log and the unit inspector live
// in their own modules. This file owns stepping, transport controls, selection,
// and one-line event formatting for the readout.

import {
  abilityRefDesc,
  ancestry,
  beatsOf,
  beatAtStep,
  boardAt,
  deathCauseChain,
  displayNames,
  shortDesc,
  type Beat,
  type BattleEvent,
  type NameOf,
  type StatusRegistry,
  type UnitDef,
} from "../src/index.js";
import { renderBoard, verdictHtml } from "./board-render.js";
import { beatCenterHtml } from "./beat-card.js";
import { createBattleLog } from "./battle-log.js";
import { closeInspectOverlay, openInspectOverlay, renderInspect, unitDefs } from "./inspect.js";

const BASE_STEP_MS = 350; // 1x ≈ 3 events/second (per-line reveal cadence)
// The read-pause that lands at a beat boundary: the tick that ENDS a beat
// dwells longer so the player reads the final state before the next beat opens.
// ~0.85s at 1×; both this and the per-line reveal scale with the speed control.
const BASE_PAUSE_MS = 850;

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
      // Neutral wording (battle-log.ts's): any absorb status may be the cause,
      // not just Shield — the cause line below names the actual interceptor.
      const absorbed = e.absorbed !== undefined ? ` (${e.absorbed} absorbed)` : "";
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
}

/** What the battle ran on — the viewer derives ability/status descriptions
 * from the same data, so the inspector can never drift from the rules. */
export interface BattleContent {
  teams: { A: UnitDef[]; B: UnitDef[] };
  registry: StatusRegistry;
}

export interface LoadOpts {
  /** Start playback immediately (the staged-reveal default for run battles). */
  autoplay?: boolean;
  /** Called once per load, the first time the playhead reaches the final
   * event — however it got there (playback, skip, scrub, log click). */
  onEnded?: () => void;
  /** Resume the playhead at this event index instead of 0, paused (#014: a
   * tab switch mid-battle re-mounts the same replay — it must return to where
   * the player left it, never reset to event 0). Clamped to the log; when it
   * is the final event, onEnded fires as it would on any landing there. */
  resumeAt?: number;
}

export interface Viewer {
  load(log: BattleEvent[], content: BattleContent, opts?: LoadOpts): void;
  /** Jump the playhead to the final event — the skip control's landing. */
  toEnd(): void;
  /** The current playhead index — captured before a re-mount so load's
   * resumeAt can restore it (#014 tab-switch position preservation). */
  position(): number;
  stop(): void;
  /** Detach the viewer's document-level listeners (and stop playback).
   * Anything that creates viewers more than once must destroy the old one,
   * or stale keydown handlers pile up on document. */
  destroy(): void;
}

export function createViewer(els: ViewerEls): Viewer {
  let log: BattleEvent[] = [];
  let beats: Beat[] = [];
  let name: NameOf = (id) => id;
  let step = 0;
  let timer: number | undefined;
  let defs = new Map<string, UnitDef>();
  let registry: StatusRegistry = {};
  let selected: { unit: string; status?: string } | undefined;
  let onEnded: (() => void) | undefined;
  let endedNotified = false;

  const playing = () => timer !== undefined;

  const battleLog = createBattleLog(els.log, (eventId) => {
    pause();
    goTo(eventId);
  });

  function render(): void {
    const e = log[step];
    if (!e) return;
    const board = boardAt(log, step);
    const center = beatCenterHtml(log, beats, step, (ev) => describeEvent(ev, name), verdictHtml(board));
    renderBoard(els.board, board, name, subjectsOf(e), registry, selected?.unit, center);
    battleLog.syncTo(step);
    els.scrub.value = String(step);
    els.stepLabel.textContent = `event ${step + 1}/${log.length} · turn ${e.turn}`;
    // The readout no longer mirrors the centre card's title (defect 5): the
    // within-beat story already reads off the card. The panel instead carries
    // what the card does NOT — the event's cross-beat CAUSE ancestry — which
    // slice 4 will key off the selected line/log row. A neutral label heads it
    // so the panel never repeats the card's headline verbatim.
    els.eventDesc.textContent = "cause trace";
    els.eventCause.innerHTML = causeHtml(e);
    els.prev.disabled = step === 0;
    els.next.disabled = step === log.length - 1;
    els.play.textContent = playing() ? "pause" : "play";
    if (selected !== undefined) {
      // The board just re-rendered — pin the overlay to the fresh card node.
      const sel = selected;
      openInspectOverlay("viewer", {
        anchor: els.board.querySelector<HTMLElement>(`[data-unit="${sel.unit}"]`),
        onClose: () => {
          if (selected === undefined) return;
          selected = undefined;
          render();
        },
        render: (body) =>
          renderInspect(body, {
            unitId: sel.unit,
            ...(sel.status !== undefined ? { status: sel.status } : {}),
            board,
            def: defs.get(sel.unit),
            registry,
            name,
          }),
      });
    } else {
      closeInspectOverlay("viewer");
    }
    if (log.length > 0 && step === log.length - 1 && !endedNotified) {
      endedNotified = true;
      onEnded?.();
    }
  }

  /** Reserve the tallest board this replay will show. Graves fill, lines
   * wipe, chips land — all knowable up front from the log — so render each
   * height-affecting step once and lock the max in (audit LS-1: the board
   * must never change height mid-replay, or the transport jumps under the
   * cursor). No-op while the board is display:none (offsetHeight reads 0). */
  function lockBoardHeight(): void {
    els.board.style.minHeight = "";
    let max = 0;
    const measure = (i: number): void => {
      const center = beatCenterHtml(log, beats, i, (ev) => describeEvent(ev, name), verdictHtml(boardAt(log, i)));
      renderBoard(els.board, boardAt(log, i), name, new Set(), registry, undefined, center);
      // Fractional height (getBoundingClientRect), not the integer offsetHeight:
      // a natural content height of 534.28px rounds offsetHeight to 534, so a
      // min-height of 534 fails to contain it and the board grows ~0.3px at that
      // step — nudging the transport a device pixel mid-replay (LS-1). Measuring
      // the true fractional height and ceil-ing the lock keeps the reserve at or
      // above every step's real height.
      max = Math.max(max, els.board.getBoundingClientRect().height);
    };
    // Height-affecting board steps (graves fill, lines wipe, chips land).
    for (let i = 0; i < log.length; i++) {
      const t = log[i]!.type;
      if (i !== 0 && t !== "Death" && t !== "Summon" && t !== "StatusApplied" && t !== "StatusRemoved" && t !== "Silenced" && t !== "BattleEnd") continue;
      measure(i);
    }
    // The beat card grows with every line it reveals — its tallest state is the
    // last event of each hero-affecting beat. Measure those so the card area is
    // reserved up front and the stage never grows mid-beat (LS-1).
    for (const beat of beats) {
      if (!beat.structural) measure(beat.end);
    }
    if (max > 0) els.board.style.minHeight = `${Math.ceil(max)}px`;
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
    if (timer !== undefined) window.clearTimeout(timer);
    timer = undefined;
  }

  /** Dwell before advancing OFF the event at `at`: a longer read-pause when
   * that event ends a beat (the last event before the next beat opens), the
   * base per-line cadence otherwise. Both scale with the speed control. */
  function dwellAfter(at: number): number {
    const speed = Number(els.speed.value);
    const beat = beatAtStep(beats, at)?.beat;
    const endsBeat = beat !== undefined && at === beat.end && at < log.length - 1;
    return (endsBeat ? BASE_PAUSE_MS : BASE_STEP_MS) / speed;
  }

  /** Self-rescheduling tick: auto-advance one event, then arm the next at a
   * dwell that depends on whether we just finished a beat (read-pause) — a
   * fixed interval cannot vary the gap, so playback uses a chained timeout. */
  function schedule(): void {
    timer = window.setTimeout(() => {
      if (step >= log.length - 1) {
        pause();
        render();
        return;
      }
      goTo(step + 1);
      if (playing()) schedule();
    }, dwellAfter(step));
  }

  function playPause(): void {
    if (playing()) {
      pause();
    } else {
      if (step >= log.length - 1) step = 0; // play again from the top
      schedule(); // setTimeout returns its id synchronously → playing() true at once
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
    load(newLog: BattleEvent[], content: BattleContent, opts?: LoadOpts): void {
      pause();
      log = newLog;
      beats = beatsOf(log);
      name = displayNames(log);
      // Resume where the player left off (#014), else the top. Clamped to the
      // log; the render below fires onEnded if this lands on the final event.
      step = opts?.resumeAt === undefined ? 0 : Math.max(0, Math.min(opts.resumeAt, log.length - 1));
      defs = unitDefs(log, content.teams, content.registry);
      registry = content.registry;
      selected = undefined;
      onEnded = opts?.onEnded;
      endedNotified = false;
      battleLog.load(log, name);
      els.scrub.min = "0";
      els.scrub.max = String(log.length - 1);
      lockBoardHeight();
      render();
      // A resume lands paused, wherever the player was — autoplay only on a
      // fresh load (resuming and then auto-playing would override the position).
      if (opts?.autoplay === true && opts?.resumeAt === undefined && log.length > 1) playPause();
    },
    toEnd(): void {
      pause();
      goTo(log.length - 1);
    },
    position(): number {
      return step;
    },
    stop: pause,
    destroy(): void {
      pause();
      document.removeEventListener("keydown", onKeydown);
    },
  };
}
