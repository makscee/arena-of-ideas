// #078 slice 3 screenshot walk — codex behavior sentences with every term
// linked. Captures the codex unit grid (ability sentences whose terms are
// tappable codex links), then navigates to a Part card to show the tap landing.
// Run against the warm stack: node --import tsx/esm e2e/shots-codex.mjs

import { mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { BASE, DESKTOP, PHONE, launch } from "./lib.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const outDir = process.env.SHOTS_DIR ?? join(here, ".shots");
mkdirSync(outDir, { recursive: true });

const browser = await launch();

for (const [vp, tag] of [
  [DESKTOP, "desktop"],
  [PHONE, "phone"],
]) {
  const ctx = await browser.newContext({ viewport: vp, hasTouch: vp.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.addInitScript(() => localStorage.removeItem("aoi.run.v1"));

  // Codex units section — ability sentences with linked terms.
  await page.goto(`${BASE}#codex/unit/Necromancer`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#codex-view:not([hidden])");
  await page.waitForSelector("#codex-sec-units .codex-termref");
  await page.locator("#codex-sec-units .codex-entry").first().scrollIntoViewIfNeeded();
  await page.waitForTimeout(300);
  await page.screenshot({ path: join(outDir, `${tag}-codex-units.png`), fullPage: false });
  console.log(`shot ${tag}-codex-units`);

  // Statuses section — Poison's sentence, terms linked.
  await page.goto(`${BASE}#codex/status/Poison`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#codex-sec-statuses .codex-termref");
  await page.waitForTimeout(400);
  await page.screenshot({ path: join(outDir, `${tag}-codex-status.png`), fullPage: false });
  console.log(`shot ${tag}-codex-status`);

  // Tap a Part term in a unit sentence → lands on its Part card.
  await page.goto(`${BASE}#codex/unit/Necromancer`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#codex-sec-units .codex-termref");
  const partLink = page.locator('#codex-sec-units .codex-termref[href^="#codex/part/"]').first();
  const href = await partLink.getAttribute("href");
  await partLink.click();
  await page.waitForTimeout(700); // smooth scroll + highlight
  await page.screenshot({ path: join(outDir, `${tag}-codex-part-landing.png`), fullPage: false });
  console.log(`shot ${tag}-codex-part-landing (tapped ${href})`);

  await ctx.close();
}

await browser.close();
console.log(`\ncodex shots in ${outDir}`);
