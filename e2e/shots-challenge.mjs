// Screenshot walk for the boss-challenge interaction (#075 slice 4). Drives the
// real run screen across the climb-vs-challenge choice and every terminal end
// state, at desktop AND phone width, so a human LOOKS at the rendered layout.
// Output → e2e/.shots-challenge/ (gitignored). Run against the live dev server:
//   AOI_BASE_URL=http://localhost:5280 node --import tsx/esm e2e/shots-challenge.mjs
//
// The run states are hand-shaped (deserializeRun checks structure + pool, not
// log consistency) — but every string on screen (boss head, challenge note,
// end head) is produced by the LIVE run-screen from these states at render time.

import { mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  DEFAULT_RUN_POOL,
  TOWER_HEIGHT,
  initRun,
  serializeRun,
  stressRegistry,
} from "../src/index.js";
import { DESKTOP, PHONE, launch, openRun } from "./lib.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const outDir = process.env.SHOTS_DIR ?? join(here, ".shots-challenge");
mkdirSync(outDir, { recursive: true });

const byName = Object.fromEntries(DEFAULT_RUN_POOL.map((d) => [d.name, d]));
const unitOf = (def, stacks = 1, level = 1) => ({ name: def.name, base: { ...def.base }, level, stacks, def });

/** A real initRun state, then hand-shaped for the scenario. */
function shaped(mutate) {
  const s = initRun({ seed: 7, runId: "shots", pool: DEFAULT_RUN_POOL, statuses: stressRegistry });
  mutate(s);
  return serializeRun(s);
}

/** A two-unit line fielded at `round` (the floor the boss panel reads) with
 * gold to spend — so the challenge button is live and the shop renders. */
const atFloor = (round) =>
  shaped((s) => {
    s.team = [unitOf(byName.Brawler), unitOf(byName.Squire)];
    s.offers = [byName.Venomancer, byName.Summoner, byName.Bulwark];
    s.gold = 10;
    s.lives = 5;
    s.round = round;
  });

/** A finished run in `reason`, with a small post-mortem team and a log shaped
 * so the end screen's record strip + dethrone note read like a real run. */
const ended = (reason, round, extraLog = []) =>
  shaped((s) => {
    s.team = [unitOf(byName.Brawler), unitOf(byName.Squire)];
    s.offers = [];
    s.gold = 0;
    s.lives = reason === "out-of-lives" ? 0 : 3;
    s.round = round;
    s.status = "over";
    s.endedBy = reason;
    // A couple of climb fights so the W/L/D strip has marks to render.
    s.log.push(
      { id: s.log.length, round: 1, type: "FightFought", battleSeed: 1, winner: "A", turns: 4, lives: 5 },
      { id: s.log.length + 1, round: 2, type: "FightFought", battleSeed: 2, winner: "A", turns: 5, lives: 5 },
      ...extraLog.map((e, i) => ({ id: s.log.length + 2 + i, round, ...e })),
    );
  });

const browser = await launch();

async function shot(page, name) {
  await page.screenshot({ path: join(outDir, `${name}.png`), fullPage: false });
  console.log(`shot ${name}`);
}

/** Bring the boss panel to the top of the viewport so the challenge control +
 * boss team are in frame (the shop scrolls; the boss panel sits below the line). */
async function toBoss(page) {
  await page.waitForSelector("#run-boss");
  await page.evaluate(() => document.querySelector("#run-boss").scrollIntoView({ block: "center" }));
}

for (const [vp, tag] of [
  [DESKTOP, "desktop"],
  [PHONE, "phone"],
]) {
  // 1. A climbable lower floor (round 1): boss visible, climb + challenge both
  //    offered — the climb-vs-challenge choice is legible.
  {
    const { ctx, page } = await openRun(browser, atFloor(1), vp);
    await toBoss(page);
    await shot(page, `${tag}-1-floor1-choice`);
    await ctx.close();
  }

  // 2. At the summit (floor TOWER_HEIGHT): the challenge is the crown fight, and
  //    the boss panel names the champion.
  {
    const { ctx, page } = await openRun(browser, atFloor(TOWER_HEIGHT), vp);
    await toBoss(page);
    await shot(page, `${tag}-2-summit-challenge`);
    await ctx.close();
  }

  // 3. Above the top (floor TOWER_HEIGHT+1): no boss — challenging here would
  //    overshoot; the climb button must not be the only move (it can't climb).
  {
    const { ctx, page } = await openRun(browser, atFloor(TOWER_HEIGHT + 1), vp);
    await toBoss(page);
    await shot(page, `${tag}-3-above-top-overshoot-warning`);
    await ctx.close();
  }

  // 4–7. The four terminal end states, each read on its own end screen.
  const endShots = [
    ["crown", TOWER_HEIGHT, [{ type: "Crowned", floor: TOWER_HEIGHT, dethroned: "bootstrap" }], "4-end-crown-champion"],
    ["crown", 2, [{ type: "Crowned", floor: 2, dethroned: "bootstrap" }], "5-end-crown-lowerseat"],
    ["challenge-lost", 3, [{ type: "BossChallenged", floor: 3, boss: "bootstrap" }], "6-end-challenge-lost"],
    ["overshoot", TOWER_HEIGHT + 1, [{ type: "Overshot", floor: TOWER_HEIGHT + 1 }], "7-end-overshoot"],
    ["out-of-lives", 3, [], "8-end-out-of-lives"],
  ];
  for (const [reason, round, extra, name] of endShots) {
    const { ctx, page } = await openRun(browser, ended(reason, round, extra), vp, "#run-end:not([hidden])");
    await page.evaluate(() => window.scrollTo({ top: 0 }));
    await shot(page, `${tag}-${name}`);
    await ctx.close();
  }
}

await browser.close();
console.log(`\nshots in ${outDir}`);
