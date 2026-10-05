// The battle viewer (mission #574). Slice 9 owns this file: playback with
// captions (cause → effect) and the acting unit lit, tap any change to trace
// its chain up `causedBy`, "why I lost", speed 1×/2×/skip. Give the skip
// control data-testid="battle-skip": the phone e2e taps it to get through a
// fight. Name what fired with `content`: an event's source.ability is an index
// into that unit's recipe.does (BattleUnit, by its kernel id), and the id
// there keys content.abilities. Cards come from ../ui/card (card with `live`,
// and overlay(unitSheet(...)) to open both forms).
import type { BattleRecord, FightResult, MvpContent, RunView } from "../../src/mvp/contract";
import type { Side } from "../../src/types";

/** Plays one battle, then calls onDone. It needs only the record, so the
 * shop (after a fight), Home (a playoff game, slice 8) and the stats page (the
 * champion history, slice 11) all open it. The outcome comes from
 * battle.winner. `you` is the viewer's side when they fought in it; hearts and
 * why-I-lost show only when `you` and `fight` are set. `run` is the run the
 * fight belongs to, when there is one. */
export function battleScreen(a: { battle: BattleRecord; content: MvpContent; you?: Side; fight?: FightResult; run?: RunView; onDone: () => void }): void {
  // Slice 9 replaces this stub, which goes straight to the result.
  a.onDone();
}

/** The "why I lost" card for side `you`: the 2–3 enemy chains that did the
 * most. The result screen shows it after a loss, a skipped battle included.
 * Slice 9 fills it in; null until then. */
export function whyILost(_battle: BattleRecord, _content: MvpContent, _you: Side): HTMLElement | null {
  return null;
}
