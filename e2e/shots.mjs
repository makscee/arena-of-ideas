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
const runStamp = `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`;
let ipCounter = 0;

function shotContext(viewport) {
  ipCounter += 1;
  return browser.newContext({
    viewport,
    hasTouch: viewport.width < 700,
    extraHTTPHeaders: { "x-forwarded-for": `10.109.1.${ipCounter}` },
  });
}

async function freshTitle(viewport, initScript = () => localStorage.removeItem("aoi.run.v1"), initArg) {
  const ctx = await shotContext(viewport);
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.addInitScript(initScript, initArg);
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  return { ctx, page };
}

async function pauseAtTop(page) {
  await page.click("#run-fight");
  await page.waitForSelector("#run-battle:not([hidden])");
  await page.waitForSelector("#board .bv-side");
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

// Title/nav screens: the cold title, codex, leaderboard tower, and local history
// are operator-facing proof surfaces. Capture them before battle fixtures so the
// shot set shows the app's current v1 navigation, not just injected runs.
const HISTORY_ARCHIVE = JSON.stringify({
  seasons: [
    {
      season: 1,
      version: 1,
      finalTower: {
        bosses: {
          1: { runId: "alpha-7", round: 1, seq: 2, team: [{ name: "Grunt", base: { hp: 5, pwr: 1 } }] },
          2: { runId: "beta-3", round: 2, seq: 1, team: [{ name: "Ogre", base: { hp: 9, pwr: 3 } }] },
        },
        pools: {},
      },
    },
  ],
});

for (const [vp, tag] of [
  [DESKTOP, "desktop"],
  [PHONE, "phone"],
]) {
  const { ctx, page } = await freshTitle(vp, (archive) => {
    localStorage.removeItem("aoi.run.v1");
    localStorage.setItem("aoi.season-archive.v1", archive);
  }, HISTORY_ARCHIVE);

  await shot(page, `${tag}-0-title`);

  await page.click("#title-codex");
  await page.waitForSelector("#codex-view:not([hidden])");
  await shot(page, `${tag}-0a-codex`);
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");

  await page.click("#title-leaderboard");
  await page.waitForSelector("#leaderboard-view:not([hidden])");
  await shot(page, `${tag}-0b-leaderboard`);
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");

  await page.click("#title-history");
  await page.waitForSelector("#history-view:not([hidden])");
  await shot(page, `${tag}-0c-history`);

  await ctx.close();
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

// Ideas screen (#076 slice 3, current #082 one-idea-per-player/day rule) —
// capture the public read-only list, then a logged-in submit, then a vote. A
// table with several ideas is still useful for review, but each idea is seeded
// by a DISTINCT account; the screenshot walk must never fight the product's
// one-idea-per-day rule by submitting multiple ideas as one player.
async function revealSubmit(page) {
  await page.click("#ideas-reveal");
  await page.waitForSelector("#ideas-form:not([hidden])");
}

async function seedIdea(viewport, tag, index, text) {
  const { ctx, page } = await freshTitle(viewport);
  await loginViaUi(page, `shots-${tag}-seed-${index}-${runStamp}@probe.test`, `Shots ${tag} seed ${index}`);
  await page.click("#title-ideas");
  await page.waitForSelector("#ideas-view:not([hidden])");
  await revealSubmit(page);
  await page.fill("#ideas-text", text);
  await page.click("#ideas-submit");
  await page.waitForFunction(
    (t) => [...document.querySelectorAll("#ideas-list .ideas-text")].some((e) => e.textContent === t),
    text,
  );
  await ctx.close();
}

const clickVote = (page, text, dir) =>
  page.evaluate(
    ({ t, d }) => {
      const rows = [...document.querySelectorAll("#ideas-list .ideas-row")];
      rows.find((r) => r.querySelector(".ideas-text")?.textContent === t).querySelector(`.ideas-vote-${d}`).click();
    },
    { t: text, d: dir },
  );

const ideasViewports = [
  [DESKTOP, "desktop"],
  [PHONE, "phone"],
].filter(([, tag]) => !process.env.SHOTS_IDEAS_VIEWPORT || process.env.SHOTS_IDEAS_VIEWPORT === tag);

for (const [vp, tag] of ideasViewports) {
  const seeded = [
    `${tag} idea — add a draft phase`,
    `${tag} idea — let me rename my champion`,
  ];
  for (const [i, text] of seeded.entries()) await seedIdea(vp, tag, i + 1, text);

  const { ctx, page } = await freshTitle(vp);

  // Public/read-only state — logged out, the login note shows, and any ideas
  // already on the table are readable.
  await page.click("#title-ideas");
  await page.waitForSelector("#ideas-view:not([hidden])");
  await page.waitForFunction(() => document.querySelector("#ideas-list").children.length > 0);
  await shot(page, `${tag}-8-ideas-readonly`);

  // Walk back to the title, log in, and add this player's ONE idea for today.
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");
  const mine = `${tag} idea — make poison stack faster`;
  await loginViaUi(page, `shots-${tag}-main-${runStamp}@probe.test`, `Shots ${tag}`);
  await page.click("#title-ideas");
  await page.waitForSelector("#ideas-view:not([hidden])");
  await revealSubmit(page);
  await page.waitForFunction(
    (texts) => texts.every((t) => [...document.querySelectorAll("#ideas-list .ideas-text")].some((e) => e.textContent === t)),
    seeded,
  );
  await page.fill("#ideas-text", mine);
  await page.click("#ideas-submit");
  await page.waitForFunction(
    (t) => [...document.querySelectorAll("#ideas-list .ideas-text")].some((e) => e.textContent === t),
    mine,
  );
  await shot(page, `${tag}-9-ideas-list`);

  // Vote this player's idea up — its score/rank moves and the up arrow reads
  // voted. The vote path is real server state, not a DOM-only fixture.
  await clickVote(page, mine, "up");
  await page.waitForFunction(
    (t) => {
      const rows = [...document.querySelectorAll("#ideas-list .ideas-row")];
      const row = rows.find((r) => r.querySelector(".ideas-text")?.textContent === t);
      return row?.querySelector('.ideas-vote-up[aria-pressed="true"]') !== null;
    },
    mine,
  );
  await shot(page, `${tag}-10-ideas-voted`);

  await ctx.close();
}

await browser.close();
console.log(`\nshots in ${outDir}`);
