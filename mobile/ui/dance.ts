// Units dance to the beat (round 4, note 5; docs/round4/viewer.md §5): the
// emoji on every shop and battle card bobs, a QuartOut bump on each beat,
// neighbours in opposite phase. CSS runs it (style.css `bob`, transform
// only); this keeps it on the beat clock. Screens rebuild their cards on
// every render, so each new emoji gets its `--bob-phase` as it lands: a
// re-render never restarts the bob. When the clock moves (the music starts,
// a new track, a seek), every bob restarts at once on the new phase.
import { app } from "./dom";
import { bobPhase, type Beat } from "./beat";
import { beatClock } from "./sound";

const EMOJI = ".card .emoji";
/** Off the stamped phase by more than this, every bob restarts in step. */
const DRIFT_MS = 30;
const CHECK_MS = 500;

/** The phase every emoji got: the beat it was keyed on. */
let keyed: Pick<Beat, "beatMs" | "origin"> | null = null;

/** Odd slots run a beat behind: a card's place among its row's cards. */
function oddSlot(emoji: Element): boolean {
  const card = emoji.closest(".card");
  const row = card?.parentElement;
  if (!card || !row) return false;
  let i = 0;
  for (const c of row.children) {
    if (c === card) return i % 2 === 1;
    if (c.classList.contains("card")) i++;
  }
  return false;
}

function stamp(el: HTMLElement, b: Pick<Beat, "beatMs" | "origin">, t: number): void {
  el.style.setProperty("--bob-phase", `${-bobPhase(b, t, oddSlot(el)).toFixed(1)}ms`);
}

/** Restarts every bob on the clock's phase now: off, a style flush, on. */
function restart(b: Beat): void {
  keyed = { beatMs: b.beatMs, origin: b.origin };
  app.style.setProperty("--beat", `${b.beatMs.toFixed(2)}ms`);
  app.dataset.dance = "off";
  void app.offsetHeight;
  const t = performance.now();
  for (const el of app.querySelectorAll<HTMLElement>(EMOJI)) stamp(el, keyed, t);
  app.dataset.dance = document.hidden ? "off" : "on";
}

/** Has the clock left the phase the bobs were keyed on? */
function drifted(b: Beat): boolean {
  if (!keyed || Math.abs(keyed.beatMs - b.beatMs) > 0.5) return true;
  const cycle = 2 * b.beatMs;
  const d = (((b.origin - keyed.origin) % cycle) + cycle) % cycle;
  return Math.min(d, cycle - d) > DRIFT_MS;
}

function check(): void {
  if (document.hidden) return;
  const b = beatClock();
  if (drifted(b)) restart(b);
}

let started = false;
/** Starts the dance; call once at start. */
export function startDance(): void {
  if (started || typeof MutationObserver === "undefined") return;
  started = true;
  restart(beatClock());
  new MutationObserver((records) => {
    if (document.hidden || !keyed) return;
    const t = performance.now();
    for (const r of records)
      for (const n of r.addedNodes) {
        if (!(n instanceof HTMLElement)) continue;
        if (n.matches(EMOJI)) stamp(n, keyed, t);
        for (const el of n.querySelectorAll<HTMLElement>(EMOJI)) stamp(el, keyed, t);
      }
    check();
  }).observe(app, { childList: true, subtree: true });
  setInterval(check, CHECK_MS);
  // A hidden tab doesn't dance; back on it, the bobs restart in step.
  document.addEventListener("visibilitychange", () => (document.hidden ? (app.dataset.dance = "off") : restart(beatClock())));
  matchMedia?.("(prefers-reduced-motion: reduce)").addEventListener?.("change", () => restart(beatClock()));
  window.__beat = () => {
    const b = beatClock();
    return { bpm: b.bpm, beatMs: b.beatMs, index: b.index, phase: b.phase, nextBeatIn: b.nextBeatIn() };
  };
}

declare global {
  interface Window {
    /** The beat clock now: e2e reads it. */
    __beat?: () => { bpm: number; beatMs: number; index: number; phase: number; nextBeatIn: number };
  }
}
