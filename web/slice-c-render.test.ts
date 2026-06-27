// Slice C render units — the two reusable, hub-droppable render functions:
// the ideas creation ladder (ideasLadderHtml) and the Arena Tower floor list
// (arenaTowerHtml). Both are pure (data → markup), so they test as strings
// without a DOM, the way unit-card.ts and the ladder accordion already do.

import { describe, expect, test } from "vitest";
import type { Idea } from "../src/index.js";
import { ideasLadderHtml } from "./ideas-screen.js";
import { arenaTowerHtml, type TowerRung } from "./ladder-view.js";

const idea = (over: Partial<Idea>): Idea => ({
  id: "idea-0",
  authorId: "alice",
  text: "an idea",
  seq: 0,
  votes: {},
  status: "on-table",
  ...over,
});

describe("ideas creation ladder render", () => {
  test("a row carries an up AND a down arrow with the net score between them", () => {
    const html = ideasLadderHtml([idea({ id: "idea-1", text: "make poison stack", votes: { me: "up", you: "up" } })], {
      userId: "me",
      mode: "top",
    });
    expect(html).toContain("ideas-vote-up");
    expect(html).toContain("ideas-vote-down");
    expect(html).toContain('class="ideas-vote-count">2<');
    // the player's own up-vote reads pressed (switch-only directional state)
    expect(html).toMatch(/ideas-vote-up[^>]*aria-pressed="true"/);
  });

  test("the status pill renders the lifecycle vocabulary, and a rejected row dims", () => {
    expect(ideasLadderHtml([idea({ status: "shipped" })], { userId: null, mode: "top" })).toContain("ideas-pill-live");
    expect(ideasLadderHtml([idea({ status: "selected" })], { userId: null, mode: "top" })).toContain(
      "ideas-pill-compiling",
    );
    const rejected = ideasLadderHtml([idea({ status: "bounced", bounceReason: "sim failed" })], {
      userId: null,
      mode: "top",
    });
    expect(rejected).toContain("ideas-pill-rejected");
    expect(rejected).toContain("ideas-row is-rejected");
    expect(rejected).toContain("Bounced — sim failed");
  });

  test("New mode orders newest (highest seq) first; Top keeps the given order", () => {
    const ideas = [idea({ id: "a", text: "older", seq: 0 }), idea({ id: "b", text: "newer", seq: 5 })];
    const asNew = ideasLadderHtml(ideas, { userId: null, mode: "new" });
    expect(asNew.indexOf("newer")).toBeLessThan(asNew.indexOf("older"));
    const asTop = ideasLadderHtml(ideas, { userId: null, mode: "top" });
    expect(asTop.indexOf("older")).toBeLessThan(asTop.indexOf("newer"));
  });

  test("idea text is escaped — a row never injects markup", () => {
    const html = ideasLadderHtml([idea({ text: "<img src=x onerror=1>" })], { userId: null, mode: "top" });
    expect(html).not.toContain("<img");
    expect(html).toContain("&lt;img");
  });
});

describe("arena tower render", () => {
  const rungs: TowerRung[] = [
    {
      rank: 1,
      handle: "@shipped",
      isChamp: true,
      isYou: false,
      round: 4,
      units: [
        { name: "Titan", family: "Strike" },
        { name: "Hex", family: "Arcane" },
      ],
    },
    { rank: 2, handle: "you", isChamp: false, isYou: true, round: 1, units: [{ name: "Medic", family: "Heal" }] },
  ];

  test("one floor row per rung, champion first in gold, the player's rung in teal", () => {
    const html = arenaTowerHtml(rungs);
    expect((html.match(/data-round=/g) ?? []).length).toBe(2); // one floor row per rung
    expect(html).toContain("tower-floor is-champ");
    expect(html).toContain("tower-floor is-you");
    expect(html).toContain('class="tower-rank">1<');
    expect(html).toContain("Arena Tower");
  });

  test("each unit becomes a family-coloured sigil chip", () => {
    const html = arenaTowerHtml(rungs);
    expect((html.match(/tower-chip/g) ?? []).length).toBe(3); // 2 champion units + 1 own
    expect(html).toContain("--fam:#ff7a4d"); // Strike orange
    expect(html).toContain("--fam:#e056fd"); // Arcane magenta
    expect(html).toContain("tower-sigil");
  });

  test("an empty tower reads an empty state, never a crash", () => {
    expect(arenaTowerHtml([])).toContain("tower-empty");
  });
});
