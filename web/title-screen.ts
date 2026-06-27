// Title screen (PRD #015 slice 3; B·Arena slice B hub) — the center of the
// always-on 3-column hub. Pure presentation: the ornament row reuses the shared
// generative shape art (unit-card.ts — code-drawn, no assets), and the run entry
// reads the run screen's state through a seam at refresh time. The mockup splits
// the old single Play/Continue button into TWO: a New Run primary that is always
// present, and a Continue entry that appears only while a run is in progress and
// carries its round. Navigation itself stays in main.ts (showView); login is
// wired behind #title-login (PRD #016).

import { shapeSvg } from "./unit-card.js";

/** The always-present label on the New Run primary (the mockup's "▸ New Run").
 * Kept on #title-play so the live nav probes still reach the run by that id. */
const NEW_RUN_LABEL = "▸ New Run";

export interface TitleScreenEls {
  /** The ornament strip — filled once with one shape per pool unit. */
  ornament: HTMLElement;
  /** The teal primary — always "▸ New Run"; opens / resumes the run. */
  newRun: HTMLButtonElement;
  /** The muted Continue entry — shown only with an active run, hidden otherwise,
   * labelled with the run's round so the player reads where they left off. */
  continueRun: HTMLButtonElement;
}

export interface TitleScreenDeps {
  /** Names that drive the ornament art — the draftable pool, hash-stable. */
  unitNames: readonly string[];
  /** Whether a run is in progress (the run screen's seam) — read on every
   * refresh, so abandon/end land back here with Continue already gone. */
  hasActiveRun(): boolean;
  /** The active run's round, or null when no run is in progress — drives the
   * Continue label ("Continue · Round N"). */
  activeRound(): number | null;
}

export interface TitleScreen {
  /** Re-read the run state and set the Continue entry. Called every time the
   * title shows — the Continue label/visibility is never cached across
   * navigations. */
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

  els.newRun.textContent = NEW_RUN_LABEL; // static — the primary never changes label
  els.newRun.title = "Start a new run — shop, fight, climb the ladder";

  return {
    refresh(): void {
      const active = deps.hasActiveRun();
      els.continueRun.hidden = !active;
      if (active) {
        const round = deps.activeRound();
        els.continueRun.textContent = round !== null ? `Continue · Round ${round}` : "Continue";
        els.continueRun.title = "Pick the run back up exactly where it left off";
      }
    },
  };
}
