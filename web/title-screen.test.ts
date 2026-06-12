// Title screen (#015 slice 3): the Play/Continue entry is never cached — it
// re-reads the run seam on every refresh — and the ornament is the shared
// generative art (one shape per pool unit, capped), not a hand-drawn asset.

import { describe, expect, test } from "vitest";
import { createTitleScreen, type TitleScreenEls } from "./title-screen.js";

/** The two elements the screen touches, as bare property bags — the module
 * only reads/writes innerHTML, textContent and title (no layout, no events). */
function makeEls() {
  return {
    ornament: { innerHTML: "" },
    play: { textContent: "", title: "" },
  };
}

describe("createTitleScreen", () => {
  test("refresh re-reads the run seam: Play without a run, Continue run with one — and back", () => {
    const els = makeEls();
    let active = false;
    const screen = createTitleScreen(els as unknown as TitleScreenEls, {
      unitNames: ["Brawler"],
      hasActiveRun: () => active,
    });
    screen.refresh();
    expect(els.play.textContent).toBe("Play");
    active = true; // a run started elsewhere — the next show must say Continue
    screen.refresh();
    expect(els.play.textContent).toBe("Continue run");
    active = false; // abandoned/finished — back to Play, never sticky
    screen.refresh();
    expect(els.play.textContent).toBe("Play");
  });

  test("the ornament is one shared shape per unit, capped at a single row's worth", () => {
    const names = ["A", "B", "C", "D", "E", "F", "G", "H", "I"]; // more than the cap
    const els = makeEls();
    createTitleScreen(els as unknown as TitleScreenEls, {
      unitNames: names,
      hasActiveRun: () => false,
    });
    const shapes = els.ornament.innerHTML.match(/<svg class="shape"/g) ?? [];
    expect(shapes.length).toBe(7);
    expect(els.ornament.innerHTML).not.toMatch(/<img/); // code-drawn only, no assets
  });
});
