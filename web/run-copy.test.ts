// Economy copy (slice 5, IA-6): the strings the shop surfaces — income line,
// stakes line, fusion pips — are derived from the tunables, never typed.
// These tests assert against the same exports the run screen reads, so
// retuning a knob cannot leave the UI lying.

import { describe, expect, test } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { BOOTSTRAP_RUN_ID, INCOME_PER_ROUND, STACK_THRESHOLD, incomeForRound, type RunUnit, type TeamSnapshot, type UnitDef } from "../src/index.js";
import { canOfferFusion, championPhrase, fusionPips, incomeLine, newRunChampionLine, nextFightLine, runUnitProgression, stakesLine } from "./run-screen.js";

const html = readFileSync(join(dirname(fileURLToPath(import.meta.url)), "index.html"), "utf8");

describe("incomeLine derives from incomeForRound()", () => {
  test("cites the curve's figure for the upcoming round", () => {
    for (const round of [1, 5, 20]) {
      expect(incomeLine(round)).toContain(`+${incomeForRound(round + 1)}g`);
    }
  });

  test("phrasing matches the curve's shape: 'each round' iff the curve is flat", () => {
    if (INCOME_PER_ROUND === 0) expect(incomeLine(1)).toContain("each round");
    else expect(incomeLine(1)).toContain("next round");
  });
});

describe("stakesLine", () => {
  test("names the current lives count", () => {
    expect(stakesLine(4)).toBe("a loss costs a life — 4 lives left");
    expect(stakesLine(1)).toBe("a loss costs a life — 1 life left");
  });
});

describe("run-screen tower vocabulary (PRD #110 slice 2)", () => {
  const champ: TeamSnapshot = { runId: "web-reign", round: 4, seq: 0, team: [] };
  const bootstrap: TeamSnapshot = { ...champ, runId: BOOTSTRAP_RUN_ID };

  test("the run intro names empty shared genesis, synth climbs, floor-1 founding, and lineage growth", () => {
    expect(html).toContain("Production/shared towers start empty");
    expect(html).toContain("synthesized seed-unit teams");
    expect(html).toContain("found floor 1");
    expect(html).toContain("beat the reigning champion to grow the lineage");
    expect(html).not.toContain("Climb the ladder: shop, fight a ghost of a past run each round, take the champion spot");
  });

  test("new-run status separates empty shared towers from solo/local bootstrap", () => {
    expect(newRunChampionLine(null)).toBe("production/shared tower is empty — the first completed run founds the champion at floor 1");
    expect(championPhrase(bootstrap)).toBe("the solo bootstrap champion");
    expect(newRunChampionLine(bootstrap)).toContain("solo bootstrap champion");
    expect(newRunChampionLine(null)).not.toContain("first crown is free");
    expect(championPhrase(bootstrap).toLowerCase()).not.toContain("shipped champion");
  });

  test("next-fight copy says synth fallback on empty floors, not a free vacant crown", () => {
    const empty = nextFightLine(3, 0, null);
    expect(empty).toContain("no live ghosts at floor 3");
    expect(empty).toContain("synthesized seed-unit team");
    expect(empty).toContain("found floor 1");
    expect(empty).not.toContain("fighting takes the crown");
    expect(empty).not.toContain("champion spot");
  });

  test("bootstrap next-fight copy has one article and names the solo convenience champion", () => {
    const line = nextFightLine(4, 0, bootstrap);
    expect(line).toBe("no live ghosts at floor 4 — challenge the solo bootstrap champion to take the crown and grow the lineage");
    expect(line).not.toContain("the the solo bootstrap champion");
  });

  test("championed floors point at the reigning champion and lineage growth", () => {
    const line = nextFightLine(4, 0, champ);
    expect(line).toBe("no live ghosts at floor 4 — challenge reigning champion web-reign (crowned at round 4) to take the crown and grow the lineage");
    expect(line.toLowerCase()).not.toContain("fresh ladder");
    expect(line.toLowerCase()).not.toContain("pre-seeded");
  });
});

const runUnit = (name: string, ability: string, copies = 3): RunUnit => {
  const def: UnitDef = { name, base: { pwr: 1, hp: 2 }, ability };
  return { name, base: { ...def.base }, kind: "base", copies, progression: copies >= 3 ? "Awakened" : "Base", def };
};

describe("ordered fusion controls", () => {
  test("offers both valid parent orders and no same-Ability commit path", () => {
    const alpha = runUnit("Alpha", "Strike");
    const beta = runUnit("Beta", "Heal");
    const same = runUnit("Same", "Strike");
    expect(canOfferFusion(alpha, beta)).toBe(true);
    expect(canOfferFusion(beta, alpha)).toBe(true);
    expect(canOfferFusion(alpha, same)).toBe(false);
    expect(canOfferFusion(same, alpha)).toBe(false);
  });

  test("rejects non-singleton and unawakened UI parents", () => {
    const alpha = runUnit("Alpha", "Strike");
    const multi = runUnit("Multi", "Heal");
    multi.def = { name: multi.def.name, base: multi.def.base, abilities: ["Heal", "Strike"] };
    expect(canOfferFusion(alpha, multi)).toBe(false);
    expect(canOfferFusion(alpha, runUnit("Young", "Heal", 2))).toBe(false);
  });
});

describe("visible run progression copy", () => {
  test("names base, Awakened, fresh/ordinary/pending fusion, and permanent paths", () => {
    expect(runUnitProgression(runUnit("Base", "Strike", 2))).toEqual({ state: "Base", progress: "2/3" });
    expect(runUnitProgression(runUnit("Awake", "Strike", 4))).toEqual({ state: "Awakened", progress: "3/3" });
    const composite = (meter: number, awakening?: "trigger" | "selector"): RunUnit => ({
      name: "Alpha + Beta", base: { pwr: 2, hp: 4 }, kind: "composite", copies: 0, progression: "Base",
      def: { name: "Alpha + Beta", base: { pwr: 2, hp: 4 }, abilities: ["Strike", "Heal"] },
      fusion: {
        parents: [{ name: "Alpha", def: runUnit("Alpha", "Strike").def }, { name: "Beta", def: runUnit("Beta", "Heal").def }],
        meter, ...(awakening !== undefined ? { awakening } : {}), doubled: awakening !== undefined,
      },
    });
    expect(runUnitProgression(composite(0))).toEqual({ state: "Fusion · Fresh", progress: "0/3" });
    expect(runUnitProgression(composite(1))).toEqual({ state: "Fusion", progress: "1/3" });
    expect(runUnitProgression(composite(2))).toEqual({ state: "Fusion", progress: "2/3" });
    expect(runUnitProgression(composite(3))).toEqual({ state: "Fusion · Pending", progress: "3/3" });
    expect(runUnitProgression(composite(3, "trigger"))).toEqual({ state: "Fusion · Trigger path", progress: "3/3" });
    expect(runUnitProgression(composite(3, "selector"))).toEqual({ state: "Fusion · Selector path", progress: "3/3" });
  });
});

describe("fusionPips track copies against STACK_THRESHOLD", () => {
  test("filled pips = copies held, total = the threshold", () => {
    for (let stacks = 1; stacks < STACK_THRESHOLD; stacks++) {
      const pips = fusionPips(stacks);
      expect(pips).toHaveLength(STACK_THRESHOLD);
      expect(pips.split("●").length - 1).toBe(stacks);
      expect(pips.split("○").length - 1).toBe(STACK_THRESHOLD - stacks);
    }
  });

  test("over-threshold stacks clamp instead of overflowing the row", () => {
    expect(fusionPips(STACK_THRESHOLD + 2)).toHaveLength(STACK_THRESHOLD);
  });
});
