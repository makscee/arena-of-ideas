// The one unit card (#015 slice 1): its structure is the app-wide card
// contract, and every unit render site goes through it. Pinned two ways —
// the markup itself (child order, badge, battle affordances, escaping) and
// the call sites (no hand-rolled `<div class="unit` anywhere else).

import { readFileSync, readdirSync } from "fs";
import { dirname, resolve } from "path";
import { fileURLToPath } from "url";
import { describe, expect, test } from "vitest";
import { FAMILY_HEX, stressRegistry } from "../src/index.js";
import type { Family } from "../src/index.js";
import { nameFamily, shapeSvg, unitCardHtml } from "./unit-card.js";

const FAMILIES: Family[] = ["Poison", "Strike", "Shield", "Summon", "Arcane", "Control", "Heal"];

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

// ---------------------------------------------------------------------------
// B·Arena card (PRD #080): family + variant. The opt-in path — a call that
// names a variant/family/colour gets the chamfered, family-coloured card while
// keeping the card-contract anchors (.unit/.uname/.unums .hp/.pwr/.chips).
// ---------------------------------------------------------------------------
describe("B·Arena card (#080): family + full variant", () => {
  const base = {
    artName: "Venomancer",
    label: "Venomancer",
    hp: 6,
    pwr: 1,
    registry: stressRegistry,
    attrs: 'data-offer="0"',
    title: "Venomancer",
  } as const;

  test("opt-in: passing `family` switches to the B·Arena card; legacy stays default", () => {
    const legacy = unitCardHtml({ ...base });
    expect(legacy).not.toContain("unit-b");
    expect(legacy).toContain('<svg class="shape"'); // legacy generative art
    const b = unitCardHtml({ ...base, family: "Poison" });
    expect(b).toContain("unit-b");
    expect(b).toContain("is-full"); // variant defaults to full once opted in
  });

  test("full card carries header (name + ABILITY cap), art area, ability line, badge", () => {
    const card = unitCardHtml({
      ...base,
      family: "Poison",
      abilityLabel: "Toxic Strike",
      trigger: "On strike",
      target: "Front enemy",
      action: "Poison 2",
      statuses: [{ status: "Poison", stacks: 2 }],
    });
    expect(card).toContain('class="ub-head"');
    expect(card).toContain('class="ub-cap-t">TOXIC STRIKE</span>'); // cap-label uppercased
    expect(card).toContain('class="ub-art"'); // the 84px art area (full only)
    expect(card).toContain('class="ub-ability"');
    expect(card).toContain("On strike");
    expect(card).toContain("Front enemy");
    expect(card).toContain("Poison 2");
  });

  test("contract anchors survive: .unit, .uname, .unums with .hp/.pwr, .chips", () => {
    const card = unitCardHtml({ ...base, family: "Poison", statuses: [{ status: "Poison", stacks: 2 }] });
    expect(card).toMatch(/class="unit unit-b/);
    expect(card).toContain('class="uname">Venomancer</span>');
    expect(card).toContain('class="unums"');
    expect(card).toContain('<span class="hp">6</span>');
    expect(card).toContain('<span class="pwr">1</span>');
    expect(card).toContain('class="chips"');
  });

  test("each of the 7 families colours the border + sigil from its one hex", () => {
    for (const fam of FAMILIES) {
      const card = unitCardHtml({ ...base, family: fam });
      const hex = FAMILY_HEX[fam];
      expect(card, `${fam} sets --fam`).toContain(`style="--fam:${hex}"`);
      expect(card, `${fam} tags the family class`).toContain(`fam-${fam.toLowerCase()}`);
      // the sigil draws in the family hex (border + glyph both derive from --fam
      // in CSS; the glyph fill carries the literal hex in markup)
      expect(card, `${fam} sigil uses its hex`).toContain(`fill="${hex}"`);
    }
  });

  test("explicit `color` overrides the family hex on --fam", () => {
    const card = unitCardHtml({ ...base, family: "Poison", color: "#abcdef" });
    expect(card).toContain('style="--fam:#abcdef"');
  });

  test("degrades when `family` is absent: name→family fallback still colours the card", () => {
    // `color` alone opts in but names no family — the sigil/family class come
    // from nameFamily(artName), so the card renders coloured pre-081.
    const card = unitCardHtml({ ...base, variant: "full" });
    const fam = nameFamily("Venomancer");
    expect(fam).toBe("Poison"); // keyword heuristic
    expect(card).toContain(`fam-${fam.toLowerCase()}`);
    expect(card).toContain(`style="--fam:${FAMILY_HEX[fam]}"`);
    // every name lands on SOME family (hash fallback) — never uncoloured
    expect(FAMILIES).toContain(nameFamily("Zzx Nonsense Name"));
  });

  test("label and title still escaped on the B·Arena card", () => {
    const card = unitCardHtml({ ...base, family: "Poison", label: '<b>"x"</b>', title: '<t>"q"' });
    expect(card).not.toContain("<b>");
    expect(card).toContain("&lt;b&gt;");
    expect(card).toContain("&lt;t&gt;");
  });
});

describe("every unit render site draws through the one component", () => {
  const here = dirname(fileURLToPath(import.meta.url));
  for (const f of ["run-screen.ts", "board-render.ts", "ladder-view.ts", "codex.ts"]) {
    test(`${f} imports unit-card`, () => {
      const src = readFileSync(resolve(here, f), "utf8");
      expect(src).toMatch(/from "\.\/unit-card\.js"/);
    });
  }

  // A rogue card is impossible anywhere in web/ (slice-2 carry from Cass):
  // every idiom that could mint a `unit`-classed element outside unit-card.ts
  // is banned across the whole layer — markup strings AND the createElement
  // route the old codex used. Test files are excluded (they quote the banned
  // strings to ban them); unit-card.ts is the one legitimate source.
  const cardIdioms: [string, RegExp][] = [
    // double-quoted markup on any tag (div, span, article, li, section …)
    ["markup string double-quoted", /<[a-z][^>]*class="unit[\s"]/],
    // single-quoted markup (same breadth)
    ["markup string single-quoted", /<[a-z][^>]*class='unit[\s']/],
    ["className assignment", /className\s*=\s*["'`]unit\b/],
    // template-literal className starting with "unit " (e.g. `unit ${extra}`)
    ["className template literal", /className\s*=\s*`unit\s/],
    ["classList.add", /classList\.add\(\s*["'`]unit\b/],
    ["setAttribute(class)", /setAttribute\(\s*["'`]class["'`]\s*,\s*["'`]unit\b/],
  ];
  const sources = readdirSync(here).filter((f) => f.endsWith(".ts") && !f.endsWith(".test.ts") && f !== "unit-card.ts");
  for (const f of sources) {
    test(`${f} mints no card by any idiom`, () => {
      const src = readFileSync(resolve(here, f), "utf8");
      for (const [idiom, pattern] of cardIdioms) {
        expect(src, `${f}: hand-rolled card via ${idiom}`).not.toMatch(pattern);
      }
    });
  }
});
