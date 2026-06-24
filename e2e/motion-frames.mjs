// Motion capture (#065) — NOT a pass/fail probe. Records the replay as a frame
// SEQUENCE so a human (or a verifier) can step through the two motion defects a
// still screenshot is blind to:
//   (i)  a single line-insert into the action card — frames captured in a burst
//        across the reveal, so the "every line re-animates" defect (all lines
//        flickering) is visible frame-to-frame, vs the fixed "only the new line
//        fades in";
//   (ii) a strike start→resolution — frames at the Strike step and the Hurt
//        step, so the "both heroes flash red, then one stays" defect is visible
//        as a red mark moving across frames, vs the fixed "only the hurt unit is
//        ever red".
// Output → e2e/.shots/motion/ (gitignored, like the screenshot walk). Commit
// THIS script, never the frames.
//
//   AOI_BASE_URL=http://localhost:5173 node --import tsx/esm e2e/motion-frames.mjs
// (or via the e2e harness origin when the dev server is not up).

import { mkdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { DESKTOP, launch, openRun, plainShopRun } from "./lib.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const outDir = process.env.SHOTS_DIR ?? join(here, ".shots", "motion");
rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

const browser = await launch();

async function intoBattle(page) {
  await page.click("#run-fight");
  await page.waitForSelector("#run-battle:not([hidden])");
  await page.waitForSelector("#board .side");
  await page.waitForSelector("#step-play");
  if ((await page.locator("#step-play").textContent())?.trim() === "pause") {
    await page.click("#step-play");
  }
  await stepTo(page, 0);
}

async function stepTo(page, n) {
  await page.evaluate((target) => {
    const scrub = document.querySelector("#scrub");
    scrub.value = String(target);
    scrub.dispatchEvent(new Event("input", { bubbles: true }));
  }, n);
}

const maxStep = (page) => page.evaluate(() => Number(document.querySelector("#scrub").max));

const beatIndex = (page) =>
  page.evaluate(() => {
    const c = document.querySelector(".beat-card");
    return c ? Number(c.getAttribute("data-beat")) : null;
  });

const lineCount = (page) => page.locator(".beat-card .bc-line").count();
const hurtLineCount = (page) => page.locator(".beat-card .bc-line.bc-hurt").count();

async function shot(page, name) {
  await page.evaluate(() => document.querySelector("#board")?.scrollIntoView({ block: "start" }));
  await page.screenshot({ path: join(outDir, `${name}.png`), fullPage: false });
  console.log(`frame ${name}`);
}

/** A short burst of frames at a fixed interval WITHOUT advancing the playhead —
 * captures the in-flight animation of whatever just rendered (the 0.22s line
 * reveal plays out across these frames). */
async function burst(page, prefix, frames = 8, everyMs = 30) {
  for (let i = 0; i < frames; i++) {
    await shot(page, `${prefix}-f${String(i).padStart(2, "0")}`);
    await page.waitForTimeout(everyMs);
  }
}

const { ctx, page } = await openRun(browser, plainShopRun(), DESKTOP);
await intoBattle(page);
const max = await maxStep(page);

// (i) A single line-insert: find two consecutive steps in one open beat where
// the line count grows, land on N (card showing k lines), then advance to N+1
// and burst-capture the reveal of the (k+1)th line. The defect shows ALL lines
// flickering across the burst; the fix shows only the last line fading in.
let insertStep = null;
for (let n = 0; n < max; n++) {
  await stepTo(page, n);
  const b0 = await beatIndex(page);
  const c0 = await lineCount(page);
  if (b0 === null || c0 === 0) continue;
  await stepTo(page, n + 1);
  const b1 = await beatIndex(page);
  const c1 = await lineCount(page);
  if (b1 === b0 && c1 === c0 + 1) {
    insertStep = n;
    break;
  }
}
if (insertStep !== null) {
  await stepTo(page, insertStep);
  await shot(page, "01-line-insert-before");
  await stepTo(page, insertStep + 1); // the new line streams in
  await burst(page, "02-line-insert-reveal"); // burst across the 0.22s reveal
  console.log(`line-insert captured at step ${insertStep}→${insertStep + 1}`);
} else {
  console.log("no single-line-insert step found to capture");
}

// (ii) A strike start→resolution: a Strike step (start, no hurt revealed yet)
// then its Hurt step (resolution). Burst at BOTH so a red mark that appears on
// the wrong unit at the start, or moves between frames, is visible.
let strikeStart = null;
for (let n = 0; n < max; n++) {
  await stepTo(page, n);
  const b0 = await beatIndex(page);
  const hurts0 = await hurtLineCount(page);
  if (b0 === null) continue;
  await stepTo(page, n + 1);
  const b1 = await beatIndex(page);
  const hurts1 = await hurtLineCount(page);
  // start = a beat step with no hurt revealed, whose NEXT step (same beat) reveals one
  if (b1 === b0 && hurts0 === 0 && hurts1 === 1) {
    strikeStart = n;
    break;
  }
}
if (strikeStart !== null) {
  await stepTo(page, strikeStart);
  await burst(page, "03-strike-start"); // the Strike step: no one should be red
  await stepTo(page, strikeStart + 1);
  await burst(page, "04-strike-resolution"); // the Hurt step: only the hurt unit red
  console.log(`strike captured at step ${strikeStart} (start) → ${strikeStart + 1} (resolution)`);
} else {
  console.log("no strike start→resolution step found to capture");
}

await ctx.close();
await browser.close();
console.log(`\nmotion frames in ${outDir} — step through f00..fNN to see the animation play out`);
