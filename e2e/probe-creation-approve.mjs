// PRD #013 slice 4 — the creation-loop browser walk. Proves the seam end to
// end through the LIVE web shell: a fixture candidate is APPROVED (injected via
// the same aoi.approved.v1 override the web merges on top of the committed
// registry), a NEW run is started from the run-new form, and the approved unit
//   1. appears in a shop offer of that run (deterministic seed — no luck),
//   2. is inspectable via the overlay (its DSL renders, creator-blind kernel),
//   3. is listed in the codex with its "made by …" credit line.
//
// The unit is force-found at a seed where it lands in round-1's initial offers
// (seed 4, computed offline against the merged pool); a few rerolls are the
// fallback so a pool-order drift never flakes the probe. Nothing here regresses
// the existing probes — it injects a NEW pool unit and a fresh run, touching no
// shared fixture.

import { chromium } from "playwright";
import { armGuard, check, finish } from "./lib.mjs";

const BASE = process.env.AOI_BASE_URL ?? "http://localhost:5280";
const DESKTOP = { width: 1280, height: 800 };

// The fixture candidate's NEW unit, as it would land in the approved registry:
// a plain UnitDef plus the _creator credit the codex shows.
const PROBELING = {
  name: "Probeling",
  base: { hp: 10, pwr: 3 },
  statuses: [{ status: "Poison", stacks: 2 }],
  _creator: "probe-fixture",
};
const APPROVED_OVERRIDE = JSON.stringify({ units: [PROBELING] });
const SEED = 4; // Probeling is in round-1's initial offers at this seed

const disarm = armGuard();
const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: DESKTOP });
const page = await ctx.newPage();
page.setDefaultTimeout(15_000);

// Approve the fixture candidate (override merged onto the committed registry),
// and ensure no stored run so the run-new form is the landing panel.
await page.addInitScript((approved) => {
  localStorage.setItem("aoi.approved.v1", approved);
  localStorage.removeItem("aoi.run.v1");
}, APPROVED_OVERRIDE);
await page.goto(BASE, { waitUntil: "domcontentloaded" });
// The app lands on the title (#015 slice 3); Play opens the new-run form.
await page.waitForSelector("#title-view:not([hidden])");
await page.click("#title-play");
await page.waitForSelector("#run-new:not([hidden])");

// --- start a NEW run at the deterministic seed -----------------------------
await page.fill("#run-seed", String(SEED));
await page.click("#run-new-form button[type=submit]");
await page.waitForSelector("#run-shop:not([hidden])");

// --- 1. the approved unit appears in a shop offer --------------------------
const offerNames = () => page.$$eval("#run-shop-row [data-offer] .uname", (els) => els.map((e) => e.textContent));
let names = await offerNames();
let rerolls = 0;
// Fallback: reroll (10g starting, 1g each) until it surfaces — bounded.
while (!names.includes("Probeling") && rerolls < 6) {
  await page.click("#run-reroll");
  await page.waitForTimeout(50);
  names = await offerNames();
  rerolls++;
}
check(names.includes("Probeling"), "approved unit appears in a shop offer of a new run", `offers=[${names}] after ${rerolls} reroll(s)`);

// --- 2. it is inspectable via the overlay ----------------------------------
const offerIndex = names.indexOf("Probeling");
await page.click(`#run-shop-row [data-offer="${offerIndex}"] .uname`);
await page.waitForSelector("#inspect-overlay:not([hidden])");
const insName = await page.locator("#inspect-overlay .ins-name").textContent();
check(insName === "Probeling", "inspector overlay opens on the approved unit", `name=${insName}`);
const insBody = await page.locator("#inspect-overlay").textContent();
check(/Poison/i.test(insBody), "inspector renders the unit's DSL (Poison status)", insBody.slice(0, 120));
await page.click("#ins-close");
await page.waitForSelector("#inspect-overlay", { state: "hidden" });

// --- 3. the codex lists it with its creator credit -------------------------
await page.click("#home-button");
await page.waitForSelector("#title-view:not([hidden])");
await page.click("#title-codex");
await page.waitForSelector("#codex-view:not([hidden])");
const codexCard = page.locator("#codex-unit-Probeling");
check((await codexCard.count()) === 1, "codex lists the approved unit", `cards=${await codexCard.count()}`);
const credit = await codexCard.locator(".codex-entry-credit").textContent().catch(() => null);
check(credit === "made by probe-fixture", "codex shows the unit's creator credit", `credit=${credit}`);

await ctx.close();
await browser.close();
disarm();
finish("probe-creation-approve");
