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
  type AbilityRegistry,
  type Beat,
  type BattleEvent,
  type NameOf,
  type StatusRegistry,
  type UnitDef,
} from "../src/index.js";
import { renderBattle, type BattleAnnotations, type BattleHeader } from "./board-render.js";
import {
  actingCardHtml,
  actingModelAt,
  actingUnitAt,
  traceChipsAt,
  traceStripHtml,
  usedThisTurnAt,
  type ActingCtx,
} from "./acting.js";
import { familySigil } from "./unit-card.js";
import { sideMap, type SideOf } from "./battle-log.js";
import { closeInspectOverlay, openInspectOverlay, renderInspect, unitDefs } from "./inspect.js";

const escHtml = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

const BASE_STEP_MS = 350; // 1x ≈ 3 events/second (per-line reveal cadence)
// The read-pause that lands at a beat boundary: the tick that ENDS a beat
// dwells longer so the player reads the final state before the next beat opens.
// ~0.85s at 1×; both this and the per-line reveal scale with the speed control.
const BASE_PAUSE_MS = 850;

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
}

/** What the battle ran on — the viewer derives ability/status descriptions
 * from the same data, so the inspector can never drift from the rules. */
export interface BattleContent {
  teams: { A: UnitDef[]; B: UnitDef[] };
  registry: StatusRegistry;
  /** The ability registry a unit's `ability` ref resolves through (PRD #081). */
  abilities: AbilityRegistry;
  /** Header facts the viewer can't derive from the log (#082 slice D): the
   * ghost/opponent label and the battle seed. Both optional — absent → the
   * header shows "vs ghost" with no name and omits the seed. */
  meta?: { opponent?: string | undefined; seed?: number | undefined } | undefined;
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
  let sideOf: SideOf = () => undefined;
  let step = 0;
  let timer: number | undefined;
  let defs = new Map<string, UnitDef>();
  let registry: StatusRegistry = {};
  let abilities: AbilityRegistry = {};
  let meta: { opponent?: string | undefined; seed?: number | undefined } | undefined;
  let selected: { unit: string; status?: string } | undefined;
  // The event whose cross-beat cause trace the readout shows (#065 slice 4).
  // Decoupled from the playhead: clicking a card line or a right-log row picks
  // an event and the readout traces ITS ancestry — within-beat causality reads
  // off the card, this is the deep trace on demand. Undefined → neutral panel.
  let selectedEvent: number | undefined;
  let onEnded: (() => void) | undefined;
  let endedNotified = false;

  const playing = () => timer !== undefined;

  /** The pure-presentation context the acting-card/side-card model reads. */
  function ctx(): ActingCtx {
    return { defs, abilities, registry, name, sideOf: (id) => sideOf(id) };
  }

  function render(): void {
    const e = log[step];
    if (!e) return;
    const board = boardAt(log, step);
    const c = ctx();
    // The acting card for the current beat's actor (or a phase caption).
    const model = actingModelAt(log, beats, step, c);
    const sigil = model.kind === "card" ? familySigil(model.acting!.family, model.acting!.hex) : "";
    const center = actingCardHtml(model, sigil);
    // Side-card battle state: who acts, who is targeted, who already struck.
    const { acting, target } = actingUnitAt(beats, step);
    const used = usedThisTurnAt(log, beats, step);
    const anno: BattleAnnotations = {
      ...(acting !== undefined ? { acting } : {}),
      ...(target !== undefined ? { target } : {}),
      used,
    };
    const header: BattleHeader = {
      opponent: meta?.opponent,
      seed: meta?.seed,
      turn: board.turn,
      ...(board.ended ? { ended: board.ended } : {}),
    };
    const trace = traceStripHtml(traceChipsAt(log, beats, step, c));
    renderBattle(els.board, {
      board,
      ctx: c,
      anno,
      registry,
      ...(selected?.unit !== undefined ? { selected: selected.unit } : {}),
      centerHtml: center,
      traceHtml: trace,
      header,
    });
    // Keep the current trace chip in view as the playhead moves.
    els.board.querySelector<HTMLElement>(".tr-chip.is-cur")?.scrollIntoView({ block: "nearest", inline: "center" });
    els.scrub.value = String(step);
    els.stepLabel.textContent = `trigger ${step + 1}/${log.length}`;
    // The readout no longer mirrors the centre card's title (defect 5): the
    // within-beat story already reads off the card. The panel instead carries
    // what the card does NOT — the event's cross-beat CAUSE ancestry — keyed
    // off the SELECTED line/log row (#065 slice 4), not the playhead. A neutral
    // label heads it so the panel never repeats the card's headline verbatim.
    // Nothing selected → a neutral prompt; selecting a line or row traces it.
    els.eventDesc.textContent = "cause trace";
    const picked = selectedEvent !== undefined ? log[selectedEvent] : undefined;
    els.eventCause.innerHTML = picked
      ? causeHtml(picked)
      : `<div class="cause-empty">select a result row or a trace chip to trace its cause</div>`;
    // Mark the selected result row so the readout's subject is visible on the card.
    if (selectedEvent !== undefined) {
      els.board
        .querySelector<HTMLElement>(`.ac-row[data-id="${selectedEvent}"], .tr-chip[data-id="${selectedEvent}"]`)
        ?.classList.add("bc-line-sel");
    }
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
            abilities,
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

  /** Reserve the tallest board this replay will show (audit LS-1: the board must
   * never change height mid-replay, or the transport jumps under the cursor).
   * The board is now header + a 3-column grid (compact side cards | acting card |
   * side cards) + trace strip (#082 slice D). Every height-affecting state is
   * knowable up front: each side-card state change (a death dims a card, a status
   * chip lands) and each beat's END (the acting card's fullest RESULT/CHAINS).
   * Render each once and lock the outer max — a grid row is as tall as its
   * tallest cell, so this covers the max of the tallest column and the acting
   * card. No-op while the board is display:none (offsetHeight reads 0). */
  function lockBoardHeight(): void {
    els.board.style.minHeight = "";
    const steps: number[] = [];
    for (let i = 0; i < log.length; i++) {
      const t = log[i]!.type;
      if (i === 0 || t === "Death" || t === "Summon" || t === "StatusApplied" || t === "StatusRemoved" || t === "Silenced" || t === "BattleEnd") {
        steps.push(i);
      }
    }
    // Every beat ends at the acting card's fullest state — measure each end.
    for (const beat of beats) steps.push(beat.end);

    const c = ctx();
    const renderAt = (i: number): void => {
      const board = boardAt(log, i);
      const model = actingModelAt(log, beats, i, c);
      const sigil = model.kind === "card" ? familySigil(model.acting!.family, model.acting!.hex) : "";
      const { acting, target } = actingUnitAt(beats, i);
      const anno: BattleAnnotations = {
        ...(acting !== undefined ? { acting } : {}),
        ...(target !== undefined ? { target } : {}),
        used: usedThisTurnAt(log, beats, i),
      };
      const header: BattleHeader = {
        opponent: meta?.opponent,
        seed: meta?.seed,
        turn: board.turn,
        ...(board.ended ? { ended: board.ended } : {}),
      };
      renderBattle(els.board, {
        board,
        ctx: c,
        anno,
        registry,
        centerHtml: actingCardHtml(model, sigil),
        traceHtml: traceStripHtml(traceChipsAt(log, beats, i, c)),
        header,
      });
    };

    // Fractional height (getBoundingClientRect, ceil-ed) so a 534.28px natural
    // height doesn't slip a device pixel past an integer min-height (LS-1).
    let max = 0;
    for (const i of steps) {
      renderAt(i);
      max = Math.max(max, els.board.getBoundingClientRect().height);
    }
    if (max > 0) els.board.style.minHeight = `${Math.ceil(max)}px`;
  }

  /** The selected event's lineage: source, the causedBy chain, and a death's narrated why. */
  function causeHtml(e: BattleEvent): string {
    // A name resolver that returns a TEAM-TINTED, escaped span (#065 item 2): the
    // cause readout names units, so each reads as its side here too. Passed in
    // place of the plain `name` to the shared narration helpers — they only
    // interpolate the resolver's output, so the span rides through unchanged
    // (and escaping the name here closes the prior unescaped-into-innerHTML gap).
    const nameHtml: NameOf = (id) => {
      const side = sideOf(id);
      const cls = side === "A" ? "u ua" : side === "B" ? "u ub" : "u";
      return `<span class="${cls}">${escHtml(name(id))}</span>`;
    };
    const rows: string[] = [];
    if (e.source !== "kernel") rows.push(`<div><span class="k">source</span> ${abilityRefDesc(e.source, nameHtml)}</div>`);
    if (e.type === "Death" && e.causedBy !== null) {
      const why = deathCauseChain(log, e.causedBy, nameHtml);
      if (why.length > 0) rows.push(`<div><span class="k">why</span> died ← ${why.join(" ← ")}</div>`);
    }
    const chain = ancestry(log, e.id);
    if (chain.length > 0) {
      const links = chain.map((a) => `<a href="#" data-goto="${a.id}">${shortDesc(a, nameHtml)}</a>`).join(" ← ");
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

  /** Select an event for the cross-beat cause readout (#065 slice 4). Clamped
   * to the log; out-of-range ids are ignored so a stale data-id can't break it. */
  function selectEvent(id: number): void {
    if (Number.isFinite(id) && id >= 0 && id < log.length) selectedEvent = id;
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
    // A bottom trace chip BOTH scrubs the playhead to that beat (its long-
    // standing log-row contract, re-presented as the strip) AND selects that
    // event for the cross-beat cause readout (#082 slice D / #065 slice 4).
    const chipEl = target.closest<HTMLElement>(".tr-chip[data-id]");
    if (chipEl) {
      const id = Number(chipEl.getAttribute("data-id"));
      pause();
      selectEvent(id);
      goTo(id);
      return;
    }
    // An acting-card RESULT/CHAIN row carries its event id — clicking it selects
    // that event for the cause readout WITHOUT moving the playhead (the deep
    // trace is on demand). Re-clicking the selected row clears it.
    const line = target.closest<HTMLElement>(".ac-row[data-id]");
    if (line) {
      const id = Number(line.getAttribute("data-id"));
      selectedEvent = selectedEvent === id ? undefined : id;
      render();
      return;
    }
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
      sideOf = sideMap(log);
      // Resume where the player left off (#014), else the top. Clamped to the
      // log; the render below fires onEnded if this lands on the final event.
      step = opts?.resumeAt === undefined ? 0 : Math.max(0, Math.min(opts.resumeAt, log.length - 1));
      defs = unitDefs(log, content.teams, content.registry, content.abilities);
      registry = content.registry;
      abilities = content.abilities;
      meta = content.meta;
      selected = undefined;
      selectedEvent = undefined;
      onEnded = opts?.onEnded;
      endedNotified = false;
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
