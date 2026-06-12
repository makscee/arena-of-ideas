// PRD #015 slice 4 — the leaderboard as a proper screen, plus the two carries
// from the slice-3 verification. Against the LIVE app at desktop and 375×667:
//  1. Leaderboard opens from the title with NO run started (and starts none):
//     champion front and center with its holder named, round 1's pool already
//     expanded showing climbing teams, every unit inspectable via the shared
//     inspector; home walks back to the title; no horizontal overflow; the
//     round heads and the back affordance are ≥44px taps.
//  2. Own-run markers: with an active run the player's round reads "you fight
//     here", and after a fight the player's own ghost in the pool reads "(you)".
//  3. Backing-agnostic (the #016 server-swap seam): the leaderboard module
//     imports only the kernel's ladder-store interface + presentation modules —
//     no localStorage, no run-store, no persisted-shape assumptions.
//  4. Carry — reload mid-battle keeps the parked replay position: through a
//     home-first park (unmount persists) AND a hard teardown while the replay
//     is on screen (pagehide persists).
//  5. Carry — a codex deep-link hash dies when the player leaves the codex:
//     visit #codex/... → home → reload lands on the TITLE, not back in the
//     codex (while the cold-load deep link itself keeps working).

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  BASE,
  DESKTOP,
  PHONE,
  armGuard,
  box,
  check,
  finish,
  launch,
  openRun,
  poorGoldRun,
} from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

/** A fresh page with NO stored run — the new player's cold load. */
async function openFresh(viewport, url = BASE) {
  const ctx = await browser.newContext({ viewport, hasTouch: viewport.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.addInitScript(() => localStorage.removeItem("aoi.run.v1"));
  await page.goto(url, { waitUntil: "domcontentloaded" });
  return { ctx, page };
}

const noHorizontalOverflow = (page) =>
  page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1);

// ---- 1. the leaderboard screen, fresh player --------------------------------
async function screenScenario(viewport, tag) {
  const { ctx, page } = await openFresh(viewport);
  await page.waitForSelector("#title-view:not([hidden])");
  await page.click("#title-leaderboard");
  await page.waitForSelector("#leaderboard-view:not([hidden])");

  check(await page.locator(".lb-title").isVisible(), `${tag} the screen carries its own title`);
  check(
    (await page.evaluate(() => localStorage.getItem("aoi.run.v1"))) === null,
    `${tag} opening the leaderboard starts no run`,
  );

  // Champion front and center: the gold panel, its holder named, its team in cards.
  const champ = page.locator("#leaderboard-body .lv-champ");
  check(await champ.isVisible(), `${tag} champion panel renders`);
  const who = await champ.locator(".lv-who").textContent();
  check(who !== null && who.includes("champion"), `${tag} the crown's holder is named`, who ?? "(missing)");
  const champCards = await champ.locator(".unit").count();
  check(champCards > 0, `${tag} champion team renders as cards`, `${champCards} cards`);

  // Round 1's pool opens expanded — the climbing teams show without a tap.
  const firstPool = page.locator("#leaderboard-body .lv-round").first().locator(".lv-pool-body");
  check(await firstPool.isVisible(), `${tag} round 1's pool is open on arrival`);
  const ghostCards = await firstPool.locator(".unit").count();
  check(ghostCards > 0, `${tag} climbing teams render as cards`, `${ghostCards} cards`);

  // Inspectable: a champion card opens the shared inspector with its name.
  const firstCard = champ.locator("[data-lv]").first();
  const cardName = await firstCard.locator(".uname").textContent();
  await firstCard.click();
  await page.waitForSelector("#inspect-overlay:not([hidden])");
  const insName = await page.locator("#inspect-overlay .ins-name").textContent();
  check(insName === cardName, `${tag} champion unit opens the inspector`, `${cardName} → ${insName}`);
  await page.click("#ins-close");
  // A pool ghost's card inspects too.
  const ghostCard = firstPool.locator("[data-lv]").first();
  const ghostName = await ghostCard.locator(".uname").textContent();
  await ghostCard.click();
  await page.waitForSelector("#inspect-overlay:not([hidden])");
  check(
    (await page.locator("#inspect-overlay .ins-name").textContent()) === ghostName,
    `${tag} pool unit opens the inspector`,
  );
  await page.click("#ins-close");

  // Geometry: no sideways scroll; round heads and the way home are real taps.
  check(await noHorizontalOverflow(page), `${tag} no horizontal overflow`);
  const headBox = await box(page, "#leaderboard-body .lv-round-head >> nth=0");
  check(headBox.height >= 44, `${tag} round head is a ≥44px tap`, `${Math.round(headBox.height)}px`);
  const homeBox = await box(page, "#home-button");
  check(homeBox.height >= 44, `${tag} back-to-title is a ≥44px tap`, `${Math.round(homeBox.height)}px`);

  // Back to the title.
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");
  check(await page.locator("#title-view").isVisible(), `${tag} home walks back to the title`);
  await ctx.close();
}

// ---- 2. own-run markers ------------------------------------------------------
async function ownRunScenario(viewport, tag) {
  // An active round-1 run: its round is marked "you fight here".
  const { ctx, page } = await openRun(browser, poorGoldRun(), viewport);
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");
  await page.click("#title-leaderboard");
  await page.waitForSelector("#leaderboard-view:not([hidden])");
  const here1 = page.locator("#leaderboard-body .lv-round.here .lv-here");
  check(await here1.isVisible(), `${tag} active run's round reads "you fight here"`);

  // Fight round 1 (the snapshot lands the player's ghost in pool 1), continue
  // into round 2's shop, then look again: the ghost reads "(you)" and the
  // marker moved to round 2.
  await page.click("#home-button");
  await page.click("#title-play");
  await page.waitForSelector("#run-shop:not([hidden])");
  await page.click("#run-fight");
  await page.waitForSelector("#run-battle:not([hidden])");
  await page.click("#run-skip");
  await page.waitForSelector("#run-continue:not([hidden])");
  await page.click("#run-continue");
  await page.waitForSelector("#run-shop:not([hidden])");
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");
  await page.click("#title-leaderboard");
  await page.waitForSelector("#leaderboard-view:not([hidden])");
  const you = page.locator("#leaderboard-body .lv-round").first().locator(".lv-you");
  check(await you.isVisible(), `${tag} own ghost in the fought pool reads "(you)"`);
  const hereRound = await page.locator("#leaderboard-body .lv-round.here .lv-round-head").textContent();
  check(
    hereRound !== null && hereRound.includes("round 2"),
    `${tag} the "you fight here" marker followed the climb to round 2`,
    hereRound ?? "(missing)",
  );
  await ctx.close();
}

// ---- 3. backing-agnostic: the module imports only the ladder-store seam -----
function backingScenario() {
  const here = dirname(fileURLToPath(import.meta.url));
  const src = readFileSync(join(here, "..", "web", "ladder-view.ts"), "utf8");
  const imports = [...src.matchAll(/from "([^"]+)"/g)].map((m) => m[1]);
  const allowed = new Set(["../src/index.js", "./inspect.js", "./unit-card.js"]);
  const stray = imports.filter((i) => !allowed.has(i));
  check(stray.length === 0, `leaderboard module imports only kernel + presentation`, stray.join(", ") || "clean");
  check(!/localStorage|run-store|PersistedLadderStore/.test(src), `leaderboard module carries no backing specifics`);
}

// ---- 4. carry: reload mid-battle keeps the parked position ------------------

/** Open a seeded run, fight, and park the replay at event index 2 (the
 * probe-menu technique: pause, rewind to 0, step twice). */
async function parkMidBattle(viewport) {
  const { ctx, page } = await openRun(browser, poorGoldRun(), viewport);
  await page.click("#run-fight");
  await page.waitForSelector("#run-battle:not([hidden])");
  await page.click("#step-next"); // pauses autoplay wherever it was
  for (let i = 0; i < 30; i++) {
    if (await page.locator("#step-prev").isDisabled()) break;
    await page.click("#step-prev");
  }
  await page.click("#step-next");
  await page.click("#step-next"); // event index 2
  return { ctx, page };
}

/** A reload in the probe's terms: a FRESH page in the same context (shared
 * localStorage) with no init script — exactly what a real reload reads. (A
 * literal page.reload() would re-run openRun's init script and re-seed the
 * stored run, masking what persistence actually held.) */
async function resumeOnFreshPage(ctx, tag, label) {
  const fresh = await ctx.newPage();
  fresh.setDefaultTimeout(15_000);
  await fresh.goto(BASE, { waitUntil: "domcontentloaded" });
  await fresh.waitForSelector("#title-view:not([hidden])");
  check(
    (await fresh.locator("#title-play").textContent()) === "Continue run",
    `${tag} ${label}: title reads Continue run after reload`,
  );
  await fresh.click("#title-play");
  await fresh.waitForSelector("#run-battle:not([hidden])");
  const pos = await fresh.locator("#scrub").inputValue();
  check(pos === "2", `${tag} ${label}: reload resumes the battle at the parked event`, `event index ${pos}`);
}

async function reloadPositionScenario(viewport, tag) {
  // Leg A: park, walk home (the unmount persists), then reload.
  {
    const { ctx, page } = await parkMidBattle(viewport);
    check((await page.locator("#scrub").inputValue()) === "2", `${tag} parked at a known mid-replay event`);
    await page.click("#home-button");
    await page.waitForSelector("#title-view:not([hidden])");
    await resumeOnFreshPage(ctx, tag, "parked-from-title");
    await ctx.close();
  }
  // Leg B: park and tear the page down with the replay ON SCREEN — pagehide
  // is the only thing standing between the position and event 0.
  {
    const { ctx, page } = await parkMidBattle(viewport);
    await page.goto("about:blank"); // fires pagehide on the app's document
    await resumeOnFreshPage(ctx, tag, "hard-teardown");
    await ctx.close();
  }
}

// ---- 5. carry: a codex deep-link hash dies on leaving the codex -------------
async function staleHashScenario(viewport, tag) {
  const { ctx, page } = await openFresh(viewport, `${BASE}/#codex/status/Poison`);
  // The cold-load deep link itself still lands in the codex (regression guard
  // for the routing change that clears stale hashes).
  await page.waitForSelector("#codex-view:not([hidden])");
  check(await page.locator("#codex-view").isVisible(), `${tag} cold-load codex deep link still lands in the codex`);
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");
  check(
    (await page.evaluate(() => window.location.hash)) === "",
    `${tag} leaving the codex clears the deep-link hash`,
  );
  await page.reload({ waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  check(await page.locator("#title-view").isVisible(), `${tag} reload after leaving lands on the title`);
  check(await page.locator("#codex-view").isHidden(), `${tag} reload after leaving does not revive the codex`);
  await ctx.close();
}

for (const [viewport, tag] of [
  [PHONE, "375px"],
  [DESKTOP, "desktop"],
]) {
  await screenScenario(viewport, tag);
  await ownRunScenario(viewport, tag);
  await staleHashScenario(viewport, tag);
}
backingScenario();
// The position seam is geometry-free — one width keeps the probe lean (the
// probe-title and probe-menu resume checks sweep the in-memory seam already).
await reloadPositionScenario(PHONE, "375px");

await browser.close();
disarm();
finish("probe-leaderboard");
