// The center action card — the stage's narrator. A beat opens a card titled by
// its root event; the events it caused stream in line by line (revealed up to
// the current playhead), arranged as an indented causal tree (each line nested
// under its cause via depthInBeat). A structural-only beat (a bare turn) shows
// a thin "turn N" divider instead of a card. Pure: HTML in from (beats, step),
// no DOM, no timers — the viewer owns the playhead and the reveal cadence.

import { beatAtStep, depthInBeat, type Beat, type BattleEvent, type NameOf } from "../src/index.js";

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

/** One-line text for a beat line — supplied by the viewer so the card and the
 * event readout never drift (both narrate off the same describeEvent). */
export type LineText = (e: BattleEvent) => string;

/** A short label for a beat's root, the card's title. Falls back to the full
 * line text; callers can special-case roots (e.g. a Strike headline). */
function rootTitle(beat: Beat, text: LineText): string {
  return text(beat.root);
}

/**
 * The center-slot HTML for the playhead at `step`:
 *  - a hero-affecting beat → the action card (title + revealed causal lines);
 *  - a structural-only beat → a thin "turn N" divider;
 *  - no beat (empty/out-of-range) → the plain verdict the board carries.
 * `revealedThrough` is the current step: a caused event shows once its id ≤ step,
 * so scrubbing mid-beat yields a half-revealed card (a pure function of step).
 */
export function beatCenterHtml(
  log: BattleEvent[],
  beats: Beat[],
  step: number,
  text: LineText,
  fallback: string,
): string {
  const at = beatAtStep(beats, step);
  if (!at) return `<div class="divider">${fallback}</div>`;
  const { beat } = at;

  if (beat.structural) {
    // A bare turn — the read pause lands here with nothing to narrate.
    return `<div class="divider turn-divider"><span class="turn-n">turn ${beat.root.turn}</span></div>`;
  }

  // Lines for caused events revealed so far, indented by within-beat depth.
  // The card re-renders its whole inner HTML each step, so the `bc-line-in`
  // reveal animation must be scoped to the ONE line that just appeared — else
  // every prior line re-runs its fade/slide each step (defect A: all lines
  // re-animate). The newest revealed line is the one with the greatest id ≤ step
  // (caused is in log/id order); it alone carries `bc-line-new` (which arms the
  // animation in CSS), so already-shown lines stay static across re-renders.
  const revealed = beat.caused.filter((e) => e.id <= step);
  const newestId = revealed.length > 0 ? revealed[revealed.length - 1]!.id : -1;
  const lines = revealed
    .map((e) => {
      const depth = Math.max(0, depthInBeat(beat, log, e.id) - 1); // root is depth 1's parent → 0 indent
      const cls = lineClass(e);
      const fresh = e.id === newestId ? " bc-line-new" : "";
      return `<div class="bc-line ${cls}${fresh}" data-id="${e.id}" style="--d:${depth}">${esc(text(e))}</div>`;
    })
    .join("");

  return `
    <div class="beat-card" data-beat="${beat.index}">
      <div class="bc-title" data-id="${beat.root.id}">${esc(rootTitle(beat, text))}</div>
      <div class="bc-lines">${lines}</div>
    </div>`;
}

/** Family colour for a card line, mirroring the battle log's palette. */
function lineClass(e: BattleEvent): string {
  switch (e.type) {
    case "Hurt":
      return "bc-hurt";
    case "Heal":
      return "bc-heal";
    case "Death":
      return "bc-death";
    case "Summon":
      return "bc-summon";
    case "StatusApplied":
    case "StatusRemoved":
    case "StatChanged":
      return "bc-status";
    case "Silenced":
      return "bc-warn";
    default:
      return "bc-dim";
  }
}
