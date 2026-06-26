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

/** A unit id resolved to its bare display name plus its team side — lets the
 * coin card tint the two strikers by side (#065 item 2) from structured ids,
 * never a regex over narrated text. Undefined side → no tint (unknown unit). */
export type NameTint = (id: string) => { name: string; side: "A" | "B" | undefined };

/** Side → battle-log tint class suffix (` ua`/` ub`), matching the log palette
 * so a team colour means the same thing on the card, the log, and the coin. */
function tintCls(side: "A" | "B" | undefined): string {
  return side === "A" ? " ua" : side === "B" ? " ub" : "";
}

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
  nameTint?: NameTint,
): string {
  const at = beatAtStep(beats, step);
  if (!at) return `<div class="divider">${fallback}</div>`;
  const { beat } = at;

  // A PairFaced opens a coin-flip card (#065 slice 3): the pairing forms and the
  // coin lands on the unit that strikes first. PairFaced carries no caused events
  // (the kernel emits it, then a separate Strike), so it would otherwise read as a
  // structural-only turn divider — instead it gets its own card naming the two
  // strikers and which won the coin. The card streams nothing (no caused lines),
  // so the streaming/line model below is untouched; this is a self-contained root.
  if (beat.root.type === "PairFaced") {
    const r = beat.root;
    // The two strikers are team-tinted (#065 item 2) so the player reads which
    // side won the coin at a glance — built from structured ids, not regex over
    // free text. `nameTint` returns the escaped, side-tinted name span; with no
    // tint (text-replay callers) it falls back to the plain narrated line.
    const pair = nameTint
      ? `<span class="u${tintCls(nameTint(r.a).side)}">${esc(nameTint(r.a).name)}</span> faces ` +
        `<span class="u${tintCls(nameTint(r.b).side)}">${esc(nameTint(r.b).name)}</span> — the coin says ` +
        `<span class="u${tintCls(nameTint(r.first).side)}">${esc(nameTint(r.first).name)}</span> strikes first`
      : esc(text(r));
    return `
    <div class="beat-card coin-card" data-beat="${beat.index}" data-coin-winner="${esc(r.first)}">
      <div class="bc-title coin-title"><span class="coin-pip" aria-hidden="true">◉</span> coin flip</div>
      <div class="coin-pair">${pair}</div>
    </div>`;
  }

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
      // Stamp the line's SUBJECT instance id (the kernel `unit` it narrates) so a
      // consumer resolves "which hero" from the structured id, never a regex over
      // the bare display name — two units that share a name (e.g. A1:Brawler vs
      // B5:Brawler) read identically in the line text, so the id is the only
      // disambiguator. Mirrors the board card's own `data-unit`; absent on lines
      // with no single subject (e.g. Strike/Fatigue), which name no hero to mark.
      const subj = lineSubject(e);
      const subjAttr = subj === undefined ? "" : ` data-unit="${esc(subj)}"`;
      return `<div class="bc-line ${cls}${fresh}" data-id="${e.id}"${subjAttr} style="--d:${depth}">${esc(text(e))}</div>`;
    })
    .join("");

  return `
    <div class="beat-card" data-beat="${beat.index}">
      <div class="bc-title" data-id="${beat.root.id}">${esc(rootTitle(beat, text))}</div>
      <div class="bc-lines">${lines}</div>
    </div>`;
}

/** The instance id of the single hero a caused line is ABOUT — the kernel `unit`
 * the line narrates ("Brawler takes 1" → the hurt unit's id). Events with no one
 * hero subject (a Strike names two; Fatigue/turn structure name none) return
 * undefined, so those lines carry no `data-unit` and mark no card. This is the
 * structured handle a consumer keys off instead of regex-matching the bare,
 * collision-prone display name in the line text. */
function lineSubject(e: BattleEvent): string | undefined {
  switch (e.type) {
    case "Hurt":
    case "Heal":
    case "Death":
    case "Summon":
    case "StatusApplied":
    case "StatusRemoved":
    case "StatChanged":
    case "Silenced":
      return e.unit;
    default:
      return undefined;
  }
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
