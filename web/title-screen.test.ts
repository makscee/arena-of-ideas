// Title screen (#015 slice 3; B·Arena slice B hub) — the center hub's run
// entry. The New Run primary is always present; the Continue entry is gated on
// an active run and carries its round. Both are re-read on every refresh (the
// run seam), never cached across navigations. The module only touches
// innerHTML / textContent / title / hidden — no layout, no events — so it tests
// as bare property bags.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";
import { arenaTowerHtml } from "./ladder-view.js";
import { createTitleScreen, TITLE_HUB_HIERARCHY, type TitleScreenEls } from "./title-screen.js";

const html = readFileSync(join(dirname(fileURLToPath(import.meta.url)), "index.html"), "utf8");
const titleStart = html.indexOf('id="title-view"');
const titleView = html.slice(titleStart, html.indexOf("</section>", titleStart));

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
    expect(els.newRun.textContent).toBe(TITLE_HUB_HIERARCHY.primaryRunAction.label);
    expect(els.newRun.title).toContain("synthesized seed-unit climbs");
    expect(els.newRun.title).not.toContain("climb the ladder");
    expect(els.continueRun.hidden).toBe(true);
  });

  test("the title hierarchy names Continue as stateful beside the primary run action", () => {
    expect(TITLE_HUB_HIERARCHY.continueRunAction.id).toBe("title-continue");
    expect(TITLE_HUB_HIERARCHY.ideaRoutes).toHaveLength(1);
    expect(TITLE_HUB_HIERARCHY.towerRoutes).toHaveLength(1);
    expect(TITLE_HUB_HIERARCHY.secondaryUtilityIds).toEqual([
      "title-codex",
      "title-history",
      "title-settings",
      "title-login",
      "title-logout",
      "title-dev",
    ]);
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

  test("the title hub tower hint uses empty-start/floor-1 vocabulary", () => {
    expect(titleView).toContain("Shared tower");
    expect(titleView).toContain("empty start → floor 1");
    expect(titleView).not.toContain("Strategy ladder");
    expect(titleView).not.toContain("climb ▲");
  });

  test("the tower render empty state says shared empty start, not a free pre-seeded crown", () => {
    const tower = arenaTowerHtml([]);
    expect(tower).toContain("Shared tower");
    expect(tower).toContain("production/shared tower is empty");
    expect(tower).toContain("first completed run founds floor 1");
    expect(tower.toLowerCase()).not.toContain("first crown is free");
    expect(tower.toLowerCase()).not.toContain("pre-seeded");
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
