// Economy copy (slice 5, IA-6): the strings the shop surfaces — income line,
// stakes line, fusion pips — are derived from the tunables, never typed.
// These tests assert against the same exports the run screen reads, so
// retuning a knob cannot leave the UI lying.

import { describe, expect, test } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { BOOTSTRAP_RUN_ID, INCOME_PER_ROUND, STACK_THRESHOLD, incomeForRound, type TeamSnapshot } from "../src/index.js";
import { championPhrase, fusionPips, incomeLine, newRunChampionLine, nextFightLine, stakesLine } from "./run-screen.js";

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

  test("championed floors point at the reigning champion and lineage growth", () => {
    const line = nextFightLine(4, 0, champ);
    expect(line).toContain("reigning champion web-reign");
    expect(line).toContain("take the crown and grow the lineage");
    expect(line.toLowerCase()).not.toContain("fresh ladder");
    expect(line.toLowerCase()).not.toContain("pre-seeded");
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
