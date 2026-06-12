// The one unit card (#015 slice 1): its structure is the app-wide card
// contract, and every unit render site goes through it. Pinned two ways —
// the markup itself (child order, badge, battle affordances, escaping) and
// the call sites (no hand-rolled `<div class="unit` anywhere else).

import { readFileSync } from "fs";
import { dirname, resolve } from "path";
import { fileURLToPath } from "url";
import { describe, expect, test } from "vitest";
import { stressRegistry } from "../src/index.js";
import { shapeSvg, unitCardHtml } from "./unit-card.js";

describe("unitCardHtml", () => {
  const card = unitCardHtml({
    artName: "Brawler",
    label: "Brawler",
    hp: 7,
    pwr: 3,
    statuses: [{ status: "Poison", stacks: 2 }],
    registry: stressRegistry,
    level: 2,
    pips: "●●○",
    front: true,
    classes: "run-card",
    attrs: 'data-line="0"',
    title: "Brawler — tap to inspect",
  });

  test("one structure for every context: front tag, art, name, badge, framed stats, chips — in order", () => {
    const order = ['class="front-tag"', '<svg class="shape"', 'class="uname"', 'class="run-lvl"', 'class="unums"', 'class="chips"'];
    let at = -1;
    for (const piece of order) {
      const i = card.indexOf(piece);
      expect(i, `${piece} present, after the previous piece`).toBeGreaterThan(at);
      at = i;
    }
    expect(card).toContain('data-line="0"');
    expect(card).toContain('class="unit run-card front"');
  });

  test("the level badge carries the fusion pips", () => {
    expect(card).toMatch(/class="run-lvl">L2 <span class="run-pips">●●○<\/span>/);
  });

  test("framed stats: hp and pwr each in their own cell", () => {
    expect(card).toContain('<span class="hp">7</span>');
    expect(card).toContain('<span class="pwr">3</span>');
  });

  test("battle affordances: dead/hit classes, current/max hp, silenced chip", () => {
    const c = unitCardHtml({
      artName: "X",
      label: "X",
      hp: "3/9",
      pwr: 2,
      registry: stressRegistry,
      dead: true,
      hit: true,
      silenced: true,
      attrs: 'data-unit="A1:X"',
      title: "X — dead",
    });
    expect(c).toContain('class="unit dead hit"');
    expect(c).toContain('<span class="hp">3/9</span>');
    expect(c).toContain('class="chip mute"');
  });

  test("label and title are escaped", () => {
    const c = unitCardHtml({
      artName: "a",
      label: '<b>"x"</b>',
      hp: 1,
      pwr: 1,
      registry: stressRegistry,
      attrs: "",
      title: '<t>"q"',
    });
    expect(c).not.toContain("<b>");
    expect(c).toContain("&lt;b&gt;");
    expect(c).toContain("&lt;t&gt;");
  });

  test("shape art is deterministic per name and code-drawn — no image assets (pillar 3)", () => {
    expect(shapeSvg("Brawler", false)).toBe(shapeSvg("Brawler", false));
    expect(shapeSvg("Brawler", false)).not.toBe(shapeSvg("Squire", false));
    expect(shapeSvg("Brawler", false)).toContain('<svg class="shape"');
    expect(shapeSvg("Brawler", false)).not.toMatch(/<image|url\(/);
  });
});

describe("every unit render site draws through the one component", () => {
  const here = dirname(fileURLToPath(import.meta.url));
  for (const f of ["run-screen.ts", "board-render.ts", "ladder-view.ts"]) {
    test(`${f} imports unit-card and hand-rolls no card markup`, () => {
      const src = readFileSync(resolve(here, f), "utf8");
      expect(src).toMatch(/from "\.\/unit-card\.js"/);
      expect(src).not.toContain('<div class="unit'); // the card's markup lives in unit-card.ts only
    });
  }
});
