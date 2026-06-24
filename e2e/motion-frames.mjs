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
import { DESKTOP, duelistRun, launch, openRun, plainShopRun } from "./lib.mjs";

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

// (iii) Hero OVERLAY badges (#065 slice 2) — the feature the director twice
// reported missing. Capture, on the Duelist run (which lands TWO hits on one
// enemy in a single Strike beat), the moment a damage badge appears on a hero
// and then increments 1 → 2 as the second Hurt line reveals; then a step into
// the NEXT beat where the badge has cleared. Read 05→07 in order: 05 shows the
// hero with no badge (strike start), 06 the −1 badge as the first Hurt reveals,
// 07 the badge incremented to −2 as the second Hurt reveals, 08 the next beat
// with the badge gone.
{
  const { ctx, page } = await openRun(browser, duelistRun(), DESKTOP);
  await intoBattle(page);
  const dmax = await maxStep(page);
  // Find a Strike beat that reveals two Hurt lines on ONE unit across its steps.
  const hurtCountsByUnit = async () =>
    page.evaluate(() => {
      const counts = {};
      for (const l of document.querySelectorAll(".beat-card .bc-line.bc-hurt")) {
        const m = (l.textContent ?? "").match(/^(\S+)/);
        if (m) counts[m[1]] = (counts[m[1]] ?? 0) + 1;
      }
      return counts;
    });
  let twoHitStart = null;
  for (let n = 0; n < dmax; n++) {
    await stepTo(page, n);
    const b0 = await beatIndex(page);
    if (b0 === null) continue;
    // Walk to the end of this beat counting the max hits any one unit takes.
    let maxOne = 0;
    let m = n;
    while (m <= dmax) {
      await stepTo(page, m);
      if ((await beatIndex(page)) !== b0) break;
      const counts = await hurtCountsByUnit();
      maxOne = Math.max(maxOne, ...Object.values(counts), 0);
      m++;
    }
    if (maxOne >= 2) {
      twoHitStart = n;
      break;
    }
  }
  if (twoHitStart !== null) {
    // n = beat start (no hurt). Find the two consecutive hurt-reveal steps.
    await stepTo(page, twoHitStart);
    await shot(page, "05-overlay-strike-start"); // hero present, no badge yet
    const b0 = await beatIndex(page);
    let firstHurt = null;
    for (let m = twoHitStart + 1; m <= dmax; m++) {
      await stepTo(page, m);
      if ((await beatIndex(page)) !== b0) break;
      if ((await hurtLineCount(page)) >= 1) {
        firstHurt = m;
        break;
      }
    }
    if (firstHurt !== null) {
      await stepTo(page, firstHurt);
      await shot(page, "06-overlay-badge-appears"); // −1 badge appears
      // Advance within the beat until a unit shows two hurt lines.
      for (let m = firstHurt + 1; m <= dmax; m++) {
        await stepTo(page, m);
        if ((await beatIndex(page)) !== b0) break;
        const counts = await hurtCountsByUnit();
        if (Object.values(counts).some((c) => c >= 2)) {
          await shot(page, "07-overlay-badge-increments"); // badge now −2 (summed)
          // One more step into the NEXT beat: the badge clears.
          for (let k = m + 1; k <= dmax; k++) {
            await stepTo(page, k);
            if ((await beatIndex(page)) !== b0) {
              await shot(page, "08-overlay-badge-cleared-next-beat");
              break;
            }
          }
          break;
        }
      }
    }
    console.log(`overlay badges captured from beat at step ${twoHitStart}`);
  } else {
    console.log("no two-hit beat found to capture overlay increment");
  }
  await ctx.close();
}

// (iv) A DEATH beat — a hero greys + ✕ in place, with its death overlay, for
// the rest of the beat. Read 09: the dying unit shows the ✕ mark on its card in
// the line area (not the grave). Captured from the plain run, which kills a unit.
{
  const { ctx, page } = await openRun(browser, plainShopRun(), DESKTOP);
  await intoBattle(page);
  const pmax = await maxStep(page);
  let deathStep = null;
  for (let n = 0; n <= pmax; n++) {
    await stepTo(page, n);
    const hasDeath = await page.evaluate(() => !!document.querySelector(".beat-card .bc-line.bc-death"));
    const hasDying = await page.evaluate(() => !!document.querySelector(".unit.dying .dying-x"));
    if (hasDeath && hasDying) {
      deathStep = n;
      break;
    }
  }
  if (deathStep !== null) {
    await stepTo(page, deathStep);
    await shot(page, "09-death-greys-x-in-place");
    console.log(`death-in-place captured at step ${deathStep}`);
  } else {
    console.log("no death-in-place step found to capture");
  }
  await ctx.close();
}

await browser.close();
console.log(`\nmotion frames in ${outDir} — step through f00..fNN to see the animation play out`);
