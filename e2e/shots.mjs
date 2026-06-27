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
import { BASE, DESKTOP, PHONE, launch, loginViaUi, openRun, plainShopRun, bigBattleRun } from "./lib.mjs";

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

// Ideas screen (#076 slice 3) — empty state (logged out, read-only), then with
// a few ideas (logged in), then after voting (rank moved, a vote held). Each at
// desktop AND 375px, so a human confirms the list, submit box and vote pills
// fit and read at phone width. Requires the live MOCK_MODE server (the e2e
// orchestrator's `--serve`), so loginViaUi can complete a real login.
//
// FRESH SERVER PER VIEWPORT: the ideas table is global server state, so running
// both viewports against ONE server would show the phone pass the desktop pass's
// ideas + inherited votes — misleading to a reviewer. Set SHOTS_IDEAS_VIEWPORT
// to "desktop" or "phone" to capture ONE viewport only, and run the script once
// per viewport against a freshly (re)started serve, e.g.:
//   npm run e2e:stop; npm run e2e:serve  # fresh empty ideas table
//   SHOTS_IDEAS_VIEWPORT=desktop AOI_BASE_URL=… node --import tsx/esm e2e/shots.mjs
//   npm run e2e:stop; npm run e2e:serve  # fresh again
//   SHOTS_IDEAS_VIEWPORT=phone   AOI_BASE_URL=… node --import tsx/esm e2e/shots.mjs
// Unset, it captures both against the current server (fine for the battle shots
// above, which inject their own per-page state and share nothing).
const ideasViewports = [
  [DESKTOP, "desktop"],
  [PHONE, "phone"],
].filter(([, tag]) => !process.env.SHOTS_IDEAS_VIEWPORT || process.env.SHOTS_IDEAS_VIEWPORT === tag);

for (const [vp, tag] of ideasViewports) {
  const ctx = await browser.newContext({ viewport: vp, hasTouch: vp.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.addInitScript(() => localStorage.removeItem("aoi.run.v1"));
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");

  // Empty/read-only state — logged out, the login note shows.
  await page.click("#title-ideas");
  await page.waitForSelector("#ideas-view:not([hidden])");
  await page.waitForFunction(() => document.querySelector("#ideas-list").children.length > 0);
  await shot(page, `${tag}-8-ideas-empty`);

  // Walk back to the title, then log in and add a few ideas.
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");
  await loginViaUi(page, `shots-${tag}@probe.test`, `Shots ${tag}`);
  await page.click("#title-ideas");
  await page.waitForSelector("#ideas-view:not([hidden])");
  await page.click("#ideas-reveal"); // open the (collapsed) submit box
  await page.waitForSelector("#ideas-form:not([hidden])");
  for (const text of [
    "Make poison stack faster",
    "Add a draft phase before the run",
    "Let me rename my champion",
  ]) {
    await page.fill("#ideas-text", text);
    await page.click("#ideas-submit");
    await page.waitForFunction(
      (t) => [...document.querySelectorAll("#ideas-list .ideas-text")].some((e) => e.textContent === t),
      text,
    );
  }
  await shot(page, `${tag}-9-ideas-list`);

  // Vote the third (bottom) idea up — its rank moves and the up arrow reads voted.
  await page.evaluate(() => {
    const rows = [...document.querySelectorAll("#ideas-list .ideas-row")];
    rows.find((r) => r.querySelector(".ideas-text")?.textContent === "Let me rename my champion")
      .querySelector(".ideas-vote-up")
      .click();
  });
  await page.waitForFunction(
    () => document.querySelector('#ideas-list .ideas-vote-up[aria-pressed="true"]') !== null,
  );
  await shot(page, `${tag}-10-ideas-voted`);

  await ctx.close();
}

await browser.close();
console.log(`\nshots in ${outDir}`);
