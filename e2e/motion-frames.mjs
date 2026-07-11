// Current acting-card motion walk (#082 slice D). The pass/fail motion truths
// live in probe-motion.mjs; this companion emits inspectable frame sequences at
// desktop and phone widths while independently refusing vacuous captures.
// Output is supplied by the orchestrator as SHOTS_DIR.

import { mkdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { DESKTOP, PHONE, bigBattleRun, launch, openRun } from "./lib.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const outDir = process.env.SHOTS_DIR ?? join(here, ".shots", "motion");
rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

const browser = await launch();

async function stepTo(page, n) {
  await page.evaluate((target) => {
    const scrub = document.querySelector("#scrub");
    scrub.value = String(target);
    scrub.dispatchEvent(new Event("input", { bubbles: true }));
  }, n);
}

async function state(page) {
  return page.evaluate(() => ({
    beat: document.querySelector(".acting-card, .acting-phase")?.getAttribute("data-beat") ?? null,
    actor: document.querySelector(".acting-card")?.getAttribute("data-acting") ?? null,
    acting: document.querySelector(".unit-b.is-acting")?.getAttribute("data-unit") ?? null,
    target: document.querySelector(".unit-b.is-target")?.getAttribute("data-unit") ?? null,
    chip: document.querySelector(".trace-strip .tr-chip.is-cur")?.getAttribute("data-id") ?? null,
  }));
}

async function frame(page, tag, index, label) {
  await page.evaluate(() => document.querySelector("#board")?.scrollIntoView({ block: "start" }));
  const name = `${tag}-${String(index).padStart(2, "0")}-${label}.png`;
  await page.screenshot({ path: join(outDir, name), fullPage: false });
  console.log(`frame ${name}`);
}

for (const [viewport, tag] of [[DESKTOP, "desktop"], [PHONE, "phone"]]) {
  const { ctx, page } = await openRun(browser, bigBattleRun(), viewport);
  await page.click("#run-fight");
  await page.waitForSelector("#run-battle:not([hidden])");
  await page.waitForSelector("#board .bv-side");
  if ((await page.locator("#step-play").textContent())?.trim() === "pause") await page.click("#step-play");

  const max = await page.evaluate(() => Number(document.querySelector("#scrub").max));
  let start = -1;
  let next = -1;
  let before = null;
  let after = null;
  for (let n = 0; n < max; n++) {
    await stepTo(page, n);
    const a = await state(page);
    await stepTo(page, n + 1);
    const b = await state(page);
    if (a.actor && a.acting === a.actor && a.target && a.target !== a.actor && b.beat !== a.beat && b.chip !== a.chip) {
      start = n;
      next = n + 1;
      before = a;
      after = b;
      break;
    }
  }
  if (start < 0) throw new Error(`${tag} motion fixture found no acting/target beat boundary with a moving trace chip`);

  await stepTo(page, start);
  await frame(page, tag, 0, "acting-target-before");
  // Burst without moving the playhead captures the current ribbon/card settle.
  await page.waitForTimeout(60);
  await frame(page, tag, 1, "acting-target-settle");
  await stepTo(page, next);
  await frame(page, tag, 2, "next-beat-chip-moved");
  await page.waitForTimeout(120);
  await frame(page, tag, 3, "next-beat-settle");

  const final = await state(page);
  if (final.beat === before.beat || final.chip === before.chip) {
    throw new Error(`${tag} motion capture did not cross a beat and move the current trace chip`);
  }
  console.log(`${tag}: beat ${before.beat}/${before.chip} → ${after.beat}/${after.chip}; actor=${before.actor}, target=${before.target}`);
  await ctx.close();
}

await browser.close();
console.log(`\nmotion frames in ${outDir}`);
