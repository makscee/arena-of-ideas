// Title screen (#015 slice 3; B·Arena slice B hub) — the center hub's run
// entry. The New Run primary is always present; the Continue entry is gated on
// an active run and carries its round. Both are re-read on every refresh (the
// run seam), never cached across navigations. The module only touches
// innerHTML / textContent / title / hidden — no layout, no events — so it tests
// as bare property bags.

import { describe, expect, test } from "vitest";
import { createTitleScreen, type TitleScreenEls } from "./title-screen.js";

/** The elements the screen touches, as bare bags. */
function makeEls() {
  return {
    ornament: { innerHTML: "" },
    newRun: { textContent: "", title: "" },
    continueRun: { textContent: "", title: "", hidden: false },
  };
}

describe("createTitleScreen", () => {
  test("New Run is always labelled; Continue is hidden with no active run", () => {
    const els = makeEls();
    const screen = createTitleScreen(els as unknown as TitleScreenEls, {
      unitNames: ["Brawler"],
      hasActiveRun: () => false,
      activeRound: () => null,
    });
    screen.refresh();
    expect(els.newRun.textContent).toMatch(/New Run/);
    expect(els.continueRun.hidden).toBe(true);
  });

  test("refresh re-reads the run seam: Continue appears with its round, then hides again", () => {
    const els = makeEls();
    let active = false;
    let round = 0;
    const screen = createTitleScreen(els as unknown as TitleScreenEls, {
      unitNames: ["Brawler"],
      hasActiveRun: () => active,
      activeRound: () => (active ? round : null),
    });
    screen.refresh();
    expect(els.continueRun.hidden).toBe(true);
    active = true; // a run started elsewhere — the next show must reveal Continue
    round = 3;
    screen.refresh();
    expect(els.continueRun.hidden).toBe(false);
    expect(els.continueRun.textContent).toBe("Continue · Round 3");
    active = false; // abandoned/finished — Continue is never sticky
    screen.refresh();
    expect(els.continueRun.hidden).toBe(true);
  });

  test("the ornament is one shared shape per unit, capped at a single row's worth", () => {
    const names = ["A", "B", "C", "D", "E", "F", "G", "H", "I"]; // more than the cap
    const els = makeEls();
    createTitleScreen(els as unknown as TitleScreenEls, {
      unitNames: names,
      hasActiveRun: () => false,
      activeRound: () => null,
    });
    const shapes = els.ornament.innerHTML.match(/<svg class="shape"/g) ?? [];
    expect(shapes.length).toBe(7);
    expect(els.ornament.innerHTML).not.toMatch(/<img/); // code-drawn only, no assets
  });
});
