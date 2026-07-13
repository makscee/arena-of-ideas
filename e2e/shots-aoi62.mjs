// AOI-62 named governance→play acceptance walk. The harness owns the temporary
// DB and passes its path; every governance mutation goes through the local CLI,
// never a production HTTP route or localStorage content override.
import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { BASE, DESKTOP, PHONE, armGuard, check, finish, launch, loginViaUi } from "./lib.mjs";

const dbPath = process.env.AOI_E2E_DB_PATH;
if (!dbPath) throw new Error("AOI_E2E_DB_PATH is required — the orchestrator must own the temporary governance DB");
const shots = process.env.SHOTS_DIR ?? join(process.cwd(), "e2e/.evidence/aoi62-governance");
mkdirSync(shots, { recursive: true });
const disarm = armGuard(180_000);
const browser = await launch();
const cliLog = [];

function govern(...args) {
  const out = execFileSync("npm", ["run", "govern", "--", ...args], {
    cwd: process.cwd(),
    env: { ...process.env, DB_PATH: dbPath, AOI_E2E: "1" },
    encoding: "utf8",
  });
  cliLog.push({ args, out });
  return out;
}
function shot(page, name) { return page.screenshot({ path: join(shots, `${name}.png`), fullPage: true }); }
const overflow = (page) => page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth + 1);
const ideaText = "Frostbite Striker — a durable front-line frost bruiser that weakens the enemy it strikes";
const glassText = "Overpowered glass cannon — huge opening damage with almost no counterplay";
const bounceReason = "sim gauntlet: win rate above the allowed band";

function context(viewport, ip) {
  return browser.newContext({ viewport, hasTouch: viewport.width < 700, extraHTTPHeaders: { "x-forwarded-for": ip } });
}
async function openTitle(ctx) {
  const page = await ctx.newPage();
  page.setDefaultTimeout(20_000);
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  return page;
}
async function openIdeas(page) {
  if (!(await page.locator("#title-view").isVisible())) {
    await page.click("#home-button");
    await page.waitForSelector("#title-view:not([hidden])");
  }
  await page.click("#title-ideas");
  await page.waitForSelector("#ideas-view:not([hidden])");
  await page.waitForFunction(() => document.querySelector("#ideas-list")?.children.length > 0);
}
async function reloadIdeas(page) {
  await page.reload({ waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  await openIdeas(page);
}
const rowByText = (page, text) => page.locator("#ideas-list .ideas-row").filter({ has: page.locator(".ideas-text", { hasText: text }) });

// Named clean baseline: content v1, season 1, one archived-worthy tower seat,
// deterministic persisted identities, no ideas/votes/builds/opens from probes.
govern("fixture-reset", "--name", "aoi62-governance");

// ship-frostbiter-by-maks — submitted boundary through the real Ideas action.
const maksCtx = await context(DESKTOP, "10.62.0.1");
const maks = await openTitle(maksCtx);
await loginViaUi(maks, "maks@aoi62.test", "Maks");
await openIdeas(maks);
await maks.click("#ideas-reveal");
await maks.fill("#ideas-text", ideaText);
await maks.click("#ideas-submit");
await maks.waitForFunction((text) => [...document.querySelectorAll("#ideas-list .ideas-text")].some((e) => e.textContent === text), ideaText);
await shot(maks, "01-ship-frostbiter-by-maks-submitted-desktop");

// bounce-glass-cannon — distinct persisted author, separately named.
const glassCtx = await context(DESKTOP, "10.62.0.2");
const glass = await openTitle(glassCtx);
await loginViaUi(glass, "glass@aoi62.test", "Glass Smith");
await openIdeas(glass);
await glass.click("#ideas-reveal");
await glass.fill("#ideas-text", glassText);
await glass.click("#ideas-submit");
await glass.waitForFunction((text) => [...document.querySelectorAll("#ideas-list .ideas-text")].some((e) => e.textContent === text), glassText);
await shot(glass, "02-bounce-glass-cannon-submitted-desktop");
await glassCtx.close();

// Deterministic persisted voters: Frostbiter 5/5, glass cannon 4/5.
govern("fixture-votes", "--ship", "idea-0", "--bounce", "idea-1");
await reloadIdeas(maks);
check((await rowByText(maks, ideaText).textContent()).includes("eligible"), "ship-frostbiter-by-maks visibly reaches eligible");
check((await rowByText(maks, glassText).textContent()).includes("eligible"), "bounce-glass-cannon visibly reaches eligible");
await shot(maks, "03-both-five-votes-eligible-desktop");

govern("freeze", "--season", "1", "--version", "1");
await reloadIdeas(maks);
check((await rowByText(maks, ideaText).textContent()).includes("selected · building"), "ship case visibly selected/building");
check((await rowByText(maks, glassText).textContent()).includes("selected · building"), "bounce case visibly selected/building");
await shot(maks, "04-both-selected-building-desktop");

govern("ship", "--idea", "idea-0", "--candidate", "candidates/frostbite-striker.json", "--season", "1", "--version", "1");
govern("bounce", "--idea", "idea-1", "--reason", bounceReason, "--season", "1", "--version", "1");
govern("roll", "--season", "1", "--version", "1");

await reloadIdeas(maks);
check((await maks.locator("#ideas-season").textContent()).includes("Season 2 · content v2"), "active pointer visibly advances to season 2/content v2");
check((await rowByText(maks, ideaText).textContent()).includes("shipped"), "ship-frostbiter-by-maks visibly reaches shipped");
const bouncedText = await rowByText(maks, glassText).textContent();
check(bouncedText.includes("bounced") && bouncedText.includes(bounceReason), "bounce-glass-cannon shows exact bounced reason", bouncedText);
check(bouncedText.includes("4/5 yes"), "bounce votes survived the roll", bouncedText);
await shot(maks, "05-shipped-and-bounced-reason-desktop");

// Immutable authenticated server archive, not the device-local history.
await maks.click("#home-button");
await maks.click("#title-history");
await maks.waitForSelector("#history-view:not([hidden])");
await maks.waitForSelector('.history-row[data-season="1"]');
await shot(maks, "06-season-1-archive-list-desktop");
await maks.click('.history-row[data-season="1"]');
await maks.waitForSelector("#history-detail:not([hidden])");
check((await maks.locator("#history-detail").textContent()).includes("content v1"), "archive retains old content version");
await shot(maks, "07-season-1-archive-final-tower-desktop");

// Another authenticated persisted player can still vote on the bounced idea.
const voterCtx = await context(DESKTOP, "10.62.0.3");
const voter = await openTitle(voterCtx);
await loginViaUi(voter, "voter6@aoi62.test", "Voter 6");
await openIdeas(voter);
const bounceRow = rowByText(voter, glassText);
await bounceRow.locator(".ideas-vote-up").click();
await voter.waitForFunction((text) => {
  const row = [...document.querySelectorAll("#ideas-list .ideas-row")].find((r) => r.querySelector(".ideas-text")?.textContent === text);
  return row?.textContent?.includes("5/6 yes");
}, glassText);
check((await rowByText(voter, glassText).textContent()).includes(bounceReason), "later vote lands and exact reason remains");
await shot(voter, "08-bounce-retained-revotable-desktop");

// Pinned seed 9 was computed once against the v2 canonical pool: its initial
// offers are Summoner/Frostbiter/Summoner. No reroll/search/injection.
await voter.click("#home-button");
await voter.click("#title-play");
await voter.waitForSelector("#run-new:not([hidden])");
await voter.fill("#run-seed", "9");
await voter.click("#run-start");
await voter.waitForSelector("#run-shop:not([hidden])");
const offers = await voter.$$eval("#run-shop-row [data-offer] .uname", (els) => els.map((e) => e.textContent));
check(offers.includes("Frostbiter"), "real authenticated season-2 initial shop contains Frostbiter at pinned seed 9", JSON.stringify(offers));
const frostOffer = voter.locator("#run-shop-row [data-offer]").filter({ hasText: "Frostbiter" }).first();
check((await frostOffer.textContent()).includes("made by Maks"), "shop Unit card carries made by Maks");
await shot(voter, "09-season-2-real-shop-frostbiter-desktop");
await frostOffer.locator(".uname").click();
await voter.waitForSelector("#inspect-overlay:not([hidden])");
const inspectorText = await voter.locator("#inspect-overlay").textContent();
check(inspectorText.includes("Frostbiter") && inspectorText.includes("made by Maks"), "shop inspector contains Frostbiter and made by Maks");
await voter.screenshot({ path: join(shots, "10-frostbiter-inspector-made-by-maks-desktop.png"), fullPage: false });
await voter.keyboard.press("Escape");
await voter.waitForSelector("#inspect-overlay", { state: "hidden" });
await voter.click("#home-button");
await voter.click("#title-codex");
await voter.waitForSelector("#codex-view:not([hidden])");
await voter.fill(".codex-search", "Frostbiter");
const codex = voter.locator("#codex-unit-Frostbiter");
check((await codex.textContent()).includes("made by Maks"), "Codex carries made by Maks");
await shot(voter, "11-frostbiter-codex-made-by-maks-desktop");

// Hub is synopsis-only: no duplicate submit/vote controls, no fake creation rank.
await voter.click("#home-button");
await voter.waitForSelector("#title-view:not([hidden])");
check((await voter.locator("#title-view [data-vote-dir]").count()) === 0, "hub has no actionable vote controls");
check((await voter.locator("#title-view form#hub-ideas-form").count()) === 0, "hub has no duplicate idea form");
const hubText = await voter.locator("#title-view").textContent();
check(!/Creation ladder|Creation\s*#/i.test(hubText), "hub has no creation ladder/rank fiction");
await shot(voter, "12-read-only-hub-season-2-desktop");

// 375px basic operability/no-overflow: same server states/reason and real remote
// shop/inspector/Codex remain reachable. Reuse the authenticated bearer only.
const token = await voter.evaluate(() => localStorage.getItem("aoi.session.v1"));
const phoneCtx = await context(PHONE, "10.62.0.4");
await phoneCtx.addInitScript((t) => localStorage.setItem("aoi.session.v1", t), token);
const phone = await openTitle(phoneCtx);
await openIdeas(phone);
check(!(await overflow(phone)), "375px Ideas has no horizontal overflow");
const phoneBounce = await rowByText(phone, glassText).textContent();
check(phoneBounce.includes(bounceReason) && phoneBounce.includes("5/6 yes"), "375px reaches bounced reason and retained vote state");
check((await rowByText(phone, ideaText).textContent()).includes("shipped"), "375px reaches shipped state");
await shot(phone, "13-ideas-shipped-bounced-375px");
await phone.click("#home-button");
await phone.click("#title-play");
await phone.fill("#run-seed", "9");
await phone.click("#run-start");
await phone.waitForSelector("#run-shop:not([hidden])");
const phoneFrost = phone.locator("#run-shop-row [data-offer]").filter({ hasText: "Frostbiter" }).first();
check(await phoneFrost.isVisible(), "375px real authenticated shop reaches Frostbiter");
check(!(await overflow(phone)), "375px shop has no horizontal overflow");
await shot(phone, "14-season-2-shop-frostbiter-375px");
await phoneFrost.locator(".uname").click();
await phone.waitForSelector("#inspect-overlay:not([hidden])");
check((await phone.locator("#inspect-overlay").textContent()).includes("made by Maks"), "375px inspector reaches made by Maks");
check(!(await overflow(phone)), "375px inspector has no horizontal overflow");
await shot(phone, "15-frostbiter-inspector-375px");
await phone.keyboard.press("Escape");
await phone.waitForSelector("#inspect-overlay", { state: "hidden" });
await phone.click("#home-button");
await phone.click("#title-codex");
await phone.fill(".codex-search", "Frostbiter");
check((await phone.locator("#codex-unit-Frostbiter").textContent()).includes("made by Maks"), "375px Codex reaches made by Maks");
check(!(await overflow(phone)), "375px Codex has no horizontal overflow");
await shot(phone, "16-frostbiter-codex-375px");

writeFileSync(join(shots, "manifest.json"), JSON.stringify({ scenarios: ["ship-frostbiter-by-maks", "bounce-glass-cannon"], pinnedSeed: 9, bounceReason, cli: cliLog }, null, 2));
await phoneCtx.close();
await voterCtx.close();
await maksCtx.close();
await browser.close();
disarm();
finish("shots-aoi62 governance next-season walk");
