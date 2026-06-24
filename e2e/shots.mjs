// Screenshot walk (not a pass/fail probe): drives the real flow and captures
// PNGs at every key state, at desktop AND 375px, so a human (or the dispatching
// agent) actually LOOKS at the rendered layout — positional probes pass while
// the layout is visibly broken. Output → e2e/.shots/ (gitignored); inspect them.
//
// Run against the live dev server:  AOI_BASE_URL=http://localhost:5173 \
//   node --import tsx/esm e2e/shots.mjs

import { mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { DESKTOP, PHONE, launch, openRun, plainShopRun, bigBattleRun } from "./lib.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const outDir = process.env.SHOTS_DIR ?? join(here, ".shots");
mkdirSync(outDir, { recursive: true });

const browser = await launch();

async function pauseAtTop(page) {
  await page.click("#run-fight");
  await page.waitForSelector("#run-battle:not([hidden])");
  await page.waitForSelector("#board .side");
  await page.waitForSelector("#step-play");
  if ((await page.locator("#step-play").textContent())?.trim() === "pause") {
    await page.click("#step-play");
  }
  await page.evaluate(() => {
    const scrub = document.querySelector("#scrub");
    scrub.value = "0";
    scrub.dispatchEvent(new Event("input", { bubbles: true }));
  });
}

async function stepTo(page, n) {
  await page.evaluate((target) => {
    const scrub = document.querySelector("#scrub");
    scrub.value = String(target);
    scrub.dispatchEvent(new Event("input", { bubbles: true }));
  }, n);
}

const maxStep = (page) => page.evaluate(() => Number(document.querySelector("#scrub").max));

async function shot(page, name) {
  await page.screenshot({ path: join(outDir, `${name}.png`), fullPage: false });
  console.log(`shot ${name}`);
}

for (const [vp, tag] of [
  [DESKTOP, "desktop"],
  [PHONE, "phone"],
]) {
  const { ctx, page } = await openRun(browser, plainShopRun(), vp);
  await shot(page, `${tag}-1-shop`);

  await pauseAtTop(page);
  await shot(page, `${tag}-2-battle-start`);

  const total = await maxStep(page);
  await stepTo(page, Math.min(6, Math.floor(total / 3)));
  await shot(page, `${tag}-3-battle-mid`);

  await stepTo(page, Math.floor(total * 0.7));
  await shot(page, `${tag}-4-battle-late`);

  await page.click("#run-skip").catch(() => {});
  await page.waitForTimeout(400);
  await shot(page, `${tag}-5-outcome`);

  await ctx.close();
}

// A near-max ASYMMETRIC matchup (full five-unit side A vs the bootstrap
// opponent on side B) — the #065 slice-1 crush case. Captured at battle start
// (all units alive, the widest the board ever is) and mid-stream (a beat card
// open between the two full lines) so a human confirms BOTH teams render their
// cards at full, readable size with no crush/overlap/overflow.
for (const [vp, tag] of [
  [DESKTOP, "desktop"],
  [PHONE, "phone"],
]) {
  const { ctx, page } = await openRun(browser, bigBattleRun(), vp);
  await pauseAtTop(page);
  // The board is the subject here (not the transport): scroll the board to the
  // top of the viewport so both full lines + the centre lane are captured.
  const toBoard = async () =>
    page.evaluate(() => document.querySelector("#board").scrollIntoView({ block: "start" }));
  await toBoard();
  await shot(page, `${tag}-6-bigteam-start`);
  const total = await maxStep(page);
  await stepTo(page, Math.min(6, Math.floor(total / 3)));
  await toBoard();
  await shot(page, `${tag}-7-bigteam-mid`);
  await ctx.close();
}

await browser.close();
console.log(`\nshots in ${outDir}`);
