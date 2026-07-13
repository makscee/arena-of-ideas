// AOI-60 acceptance walk. The unfiltered e2e gate recreates these stable
// desktop/phone captures after wiping e2e/.evidence: Unit inspector closed/open,
// full-page 5v5 battle lines, and an approved generated Unit in shop/inspector.

import { mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { BASE, DESKTOP, PHONE, launch, lineFullRun, openRun, targetsRun } from "./lib.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const outDir = process.env.SHOTS_DIR ?? join(here, ".shots", "aoi60-acceptance");
mkdirSync(outDir, { recursive: true });

const PROBELING = {
  name: "Probeling",
  base: { hp: 10, pwr: 3 },
  ability: "Strike",
  statuses: [{ status: "Poison", stacks: 2 }],
  _creator: "probe-fixture",
};
const APPROVED_OVERRIDE = JSON.stringify({ units: [PROBELING] });
const browser = await launch();

async function shot(page, name, fullPage = false) {
  await page.screenshot({ path: join(outDir, `${name}.png`), fullPage, animations: "disabled" });
  console.log(`shot ${name}`);
}

async function assertNoHorizontalOverflow(page, label) {
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth + 1);
  if (overflow) throw new Error(`${label} has horizontal page overflow`);
}

for (const [viewport, tag] of [[DESKTOP, "desktop"], [PHONE, "phone"]]) {
  // Shared Unit inspector contract, both closed and open on the same live shop.
  {
    const { ctx, page } = await openRun(browser, targetsRun(), viewport);
    await assertNoHorizontalOverflow(page, `${tag} inspector-closed shop`);
    await shot(page, `${tag}-unit-inspector-closed`);
    await page.locator('#run-shop-row [data-card-entity="unit"]').first().click();
    await page.waitForSelector('#inspect-overlay:not([hidden]) > [data-card-entity="unit"]');
    await assertNoHorizontalOverflow(page, `${tag} inspector-open shop`);
    await shot(page, `${tag}-unit-inspector-open`);
    await page.click("#ins-close");
    await page.waitForSelector("#inspect-overlay", { state: "hidden" });
    await ctx.close();
  }

  // A deterministic 5v5 opening, captured full-page so every Unit on both
  // vertically stacked phone lines remains reviewable rather than below crop.
  {
    const { ctx, page } = await openRun(browser, lineFullRun(), viewport);
    await page.click("#run-fight");
    await page.waitForSelector("#run-battle:not([hidden])");
    await page.waitForSelector('#board [data-non-card="battle-event"], #board .acting-phase');
    if ((await page.locator("#step-play").textContent())?.trim() === "pause") await page.click("#step-play");
    await page.evaluate(() => {
      const scrub = document.querySelector("#scrub");
      scrub.value = "0";
      scrub.dispatchEvent(new Event("input", { bubbles: true }));
      document.querySelector("#board")?.scrollIntoView({ block: "start" });
    });
    const lineCounts = await page.evaluate(() => ({
      a: document.querySelectorAll('.bv-side[data-side="A"] [data-card-entity="unit"]').length,
      b: document.querySelectorAll('.bv-side[data-side="B"] [data-card-entity="unit"]').length,
    }));
    if (lineCounts.a !== 5 || lineCounts.b !== 5) throw new Error(`${tag} expected 5v5 visible lines, got ${lineCounts.a}v${lineCounts.b}`);
    await assertNoHorizontalOverflow(page, `${tag} five-Unit lines`);
    // Full-page capture paints fixed controls once at the current viewport.
    // Park at the document foot so those controls do not cover either Unit line.
    await page.evaluate(() => window.scrollTo(0, document.documentElement.scrollHeight));
    await shot(page, `${tag}-five-unit-lines-fullpage`, true);
    await ctx.close();
  }

  // Approved/generated content is self-contained in this task: no dependence
  // on an earlier probe's page-local localStorage survives the evidence wipe.
  {
    const ctx = await browser.newContext({ viewport, hasTouch: viewport.width < 700 });
    const page = await ctx.newPage();
    page.setDefaultTimeout(15_000);
    await page.addInitScript((approved) => {
      localStorage.setItem("aoi.approved.v1", approved);
      localStorage.removeItem("aoi.run.v1");
    }, APPROVED_OVERRIDE);
    await page.goto(BASE, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("#title-view:not([hidden])");
    await page.click("#title-play");
    await page.waitForSelector("#run-new:not([hidden])");
    await page.fill("#run-seed", "4");
    await page.click("#run-new-form button[type=submit]");
    await page.waitForSelector("#run-shop:not([hidden])");

    const names = () => page.$$eval("#run-shop-row [data-offer] .uname", (els) => els.map((el) => el.textContent?.trim()));
    let offers = await names();
    for (let rerolls = 0; !offers.includes("Probeling") && rerolls < 6; rerolls++) {
      await page.click("#run-reroll");
      offers = await names();
    }
    if (!offers.includes("Probeling")) throw new Error(`${tag} generated Probeling absent from offers: ${offers.join(", ")}`);
    const generated = page.locator('#run-shop-row [data-card-entity="unit"]', { hasText: "Probeling" });
    await generated.scrollIntoViewIfNeeded();
    await assertNoHorizontalOverflow(page, `${tag} generated Unit offer`);
    await shot(page, `${tag}-generated-unit-offer`);
    await generated.click();
    await page.waitForSelector('#inspect-overlay:not([hidden]) [data-entity-name="Probeling"]');
    if (tag === "phone") {
      // Exercise the sheet's real scroll-padding contract before the stable
      // capture: its final visible row must sit wholly above the fixed menu.
      const lastRow = page.locator("#inspect-overlay.sheet > :not([hidden])").last();
      await lastRow.evaluate((row) => row.scrollIntoView({ block: "end", inline: "nearest" }));
      await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
      const clearance = await page.evaluate(() => {
        const panel = document.querySelector("#inspect-overlay.sheet");
        const row = panel?.querySelector(":scope > :not([hidden]):last-child");
        const menu = document.querySelector("#run-menu-button");
        if (!(panel instanceof HTMLElement) || !(row instanceof HTMLElement) || !(menu instanceof HTMLElement)) return undefined;
        const panelRect = panel.getBoundingClientRect();
        const rowRect = row.getBoundingClientRect();
        const menuRect = menu.getBoundingClientRect();
        return {
          scrollTop: panel.scrollTop,
          rowTop: rowRect.top,
          rowRight: rowRect.right,
          rowBottom: rowRect.bottom,
          panelTop: panelRect.top,
          menuTop: menuRect.top,
          viewportWidth: window.innerWidth,
        };
      });
      if (
        clearance === undefined ||
        clearance.scrollTop <= 0 ||
        clearance.rowTop < clearance.panelTop ||
        clearance.rowRight > clearance.viewportWidth ||
        clearance.rowBottom > clearance.menuTop
      ) {
        throw new Error(`${tag} generated inspector final row is not viewport-readable and menu-clear: ${JSON.stringify(clearance)}`);
      }
      console.log(`phone generated inspector scrolled menu-clear ${JSON.stringify(clearance)}`);
    }
    await shot(page, `${tag}-generated-unit-inspector`);
    await ctx.close();
  }
}

await browser.close();
console.log(`\nAOI-60 acceptance shots in ${outDir}`);
