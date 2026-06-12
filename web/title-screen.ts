// Title screen (PRD #015 slice 3) — the landing every player meets. Pure
// presentation: the ornament row reuses the shared generative shape art
// (unit-card.ts — code-drawn, no assets), and the one stateful entry (Play /
// Continue run) reads the run screen's state through a seam at refresh time.
// Navigation itself stays in main.ts (showView); login is an inert placeholder
// until PRD #016 wires real auth behind #title-login.

import { shapeSvg } from "./unit-card.js";

export interface TitleScreenEls {
  /** The ornament strip — filled once with one shape per pool unit. */
  ornament: HTMLElement;
  /** The primary entry: "Play" with no active run, "Continue run" with one. */
  play: HTMLButtonElement;
}

export interface TitleScreenDeps {
  /** Names that drive the ornament art — the draftable pool, hash-stable. */
  unitNames: readonly string[];
  /** Whether a run is in progress (the run screen's seam) — read on every
   * refresh, so abandon/end land back here already reading "Play". */
  hasActiveRun(): boolean;
}

export interface TitleScreen {
  /** Re-read the run state and set the Play/Continue entry. Called every time
   * the title shows — the label is never cached across navigations. */
  refresh(): void;
}

/** How many shapes the ornament row carries — enough to read as a parade of
 * the cast, few enough to stay one line at 375px. */
const ORNAMENT_COUNT = 7;

export function createTitleScreen(els: TitleScreenEls, deps: TitleScreenDeps): TitleScreen {
  els.ornament.innerHTML = deps.unitNames
    .slice(0, ORNAMENT_COUNT)
    .map((name) => shapeSvg(name, false))
    .join("");

  return {
    refresh(): void {
      const active = deps.hasActiveRun();
      els.play.textContent = active ? "Continue run" : "Play";
      els.play.title = active
        ? "Pick the run back up exactly where it left off"
        : "Start a new run — shop, fight, climb the ladder";
    },
  };
}
