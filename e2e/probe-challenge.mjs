// Boss-challenge interaction (#075 slice 7): the run-screen behaviours a pure
// copy test can't see — the terminal challenge's TWO-STEP CONFIRM and the
// CLIMB-DISABLED-at-the-champion's-floor guard, driven against the LIVE run
// screen over the real bootstrapped ladder. Hand-shaped run states (deserializeRun
// checks structure + pool, not log consistency) put the run on a chosen floor;
// every string + disabled state read here is produced by the live run screen.
//
// Two claims, both must-fail-first against the slice's acceptance:
//  1. The challenge is gated: ONE tap arms (the button reads "tap again to
//     confirm", a cancel appears) and does NOT end the run; the SECOND tap fires
//     (the run leaves the shop — a battle or the end screen). Cancel disarms.
//  2. At the champion's floor (the live top of the tower) the climb/fight button
//     is DISABLED with the overshoot-warning stakes; challenge is the only
//     forward move. Below the champion, the climb stays enabled.

import {
  BASE,
  DESKTOP,
  PHONE,
  armGuard,
  check,
  finish,
  launch,
} from "./lib.mjs";
import {
  DEFAULT_RUN_POOL,
  TOWER_HEIGHT,
  initRun,
  serializeRun,
  stressRegistry,
  stressAbilities,
} from "../src/index.js";

const disarm = armGuard();
const browser = await launch();

const byName = Object.fromEntries(DEFAULT_RUN_POOL.map((d) => [d.name, d]));
const unitOf = (def, stacks = 1, level = 1) => ({ name: def.name, base: { ...def.base }, level, stacks, def });

/** A real initRun state, hand-shaped to stand on `round` with a fielded line and
 * gold (so both the climb and the challenge controls render and are live). */
function atFloor(round) {
  const s = initRun({ seed: 7, runId: "shots", pool: DEFAULT_RUN_POOL, statuses: stressRegistry, abilities: stressAbilities });
  s.team = [unitOf(byName.Brawler), unitOf(byName.Squire)];
  s.offers = [byName.Venomancer, byName.Summoner, byName.Bulwark];
  s.gold = 10;
  s.lives = 5;
  s.round = round;
  return serializeRun(s);
}

/** Open the run screen on an injected state at `round`, at `viewport`, parked on
 * the shop. Resumes through the title exactly like a returning player. */
async function openShopAt(round, viewport) {
  const ctx = await browser.newContext({ viewport, hasTouch: viewport.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.addInitScript(([k, v]) => localStorage.setItem(k, v), ["aoi.run.v1", atFloor(round)]);
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  await page.click("#title-play");
  await page.waitForSelector("#run-shop:not([hidden])");
  await page.waitForSelector("#run-boss");
  return { ctx, page };
}

// ---------- 1. the two-step confirm gates the terminal challenge ----------

async function confirmScenario(viewport, tag) {
  // Floor 1: a lower lineage boss, so the challenge is a cash-out — terminal all
  // the same, so the confirm must gate it.
  const { ctx, page } = await openShopAt(1, viewport);

  // Resting: the cancel is hidden, the button is the resting challenge label.
  check(await page.locator("#run-challenge-cancel").isHidden(), `${tag}: cancel hidden before arming`);
  const restingLabel = (await page.locator("#run-challenge").textContent()).trim();
  check(restingLabel.toLowerCase().includes("challenge"), `${tag}: resting button reads "challenge…"`, restingLabel);

  // First tap ARMS — must NOT leave the shop (the challenge has not fired).
  await page.click("#run-challenge");
  const armedLabel = (await page.locator("#run-challenge").textContent()).trim();
  check(/tap again/i.test(armedLabel), `${tag}: one tap arms — button reads "tap again to confirm"`, armedLabel);
  check(await page.locator("#run-challenge-cancel").isVisible(), `${tag}: arming reveals the cancel control`);
  check(await page.locator("#run-shop").isVisible(), `${tag}: ONE tap does NOT fire the challenge (still on the shop)`);

  // Cancel disarms — back to the resting label, cancel hidden, still on the shop.
  await page.click("#run-challenge-cancel");
  check(await page.locator("#run-challenge-cancel").isHidden(), `${tag}: cancel disarms — cancel hidden again`);
  const afterCancel = (await page.locator("#run-challenge").textContent()).trim();
  check(afterCancel === restingLabel, `${tag}: cancel restores the resting label`, afterCancel);

  // Re-arm, then the SECOND tap fires: the run leaves the shop (a battle plays,
  // or — a vacant/zero-length edge aside — the end screen). The challenge at a
  // seated boss always fights, so the battle panel shows.
  await page.click("#run-challenge"); // arm
  await page.click("#run-challenge"); // fire
  // The challenge fires: the shop leaves and a battle plays (a seated boss is a
  // real fight) or — edge — the end screen. Wait for either to become visible.
  await page.waitForSelector("#run-battle:not([hidden]), #run-end:not([hidden])");
  const firedAway =
    (await page.locator("#run-battle").isVisible()) || (await page.locator("#run-end").isVisible());
  check(firedAway, `${tag}: the SECOND tap fires the challenge (left the shop — battle/end)`);
  check(await page.locator("#run-shop").isHidden(), `${tag}: the shop is gone once the challenge fires`);

  await ctx.close();
}

// ---------- 2. climb disabled at the champion's floor ----------

async function climbGuardScenario(viewport, tag) {
  // Below the champion (floor 1): the climb is enabled — a normal climb move.
  {
    const { ctx, page } = await openShopAt(1, viewport);
    check(
      !(await page.locator("#run-fight").isDisabled()),
      `${tag}: below the champion (floor 1) — climb ENABLED`,
    );
    const head = (await page.locator("#run-boss-head").textContent()).toLowerCase();
    check(head.includes("below the champion"), `${tag}: floor 1 boss head reads "below the champion"`, head);
    await ctx.close();
  }

  // The champion's floor (the seeded summit, TOWER_HEIGHT): the climb is DISABLED
  // (climbing past would overshoot); challenge is the only forward move.
  {
    const { ctx, page } = await openShopAt(TOWER_HEIGHT, viewport);
    check(
      await page.locator("#run-fight").isDisabled(),
      `${tag}: the champion's floor — climb DISABLED (no accidental overshoot)`,
    );
    const stakes = (await page.locator("#run-stakes").textContent()).toLowerCase();
    check(stakes.includes("champion") && stakes.includes("crown"), `${tag}: stakes line names the champion/crown`, stakes);
    // The challenge button is still live (a unit is fielded) and gold (champion).
    check(!(await page.locator("#run-challenge").isDisabled()), `${tag}: challenge stays enabled at the top`);
    check(
      await page.locator("#run-challenge.champion").count() === 1,
      `${tag}: the champion's challenge wears the gold treatment`,
    );
    const head = (await page.locator("#run-boss-head").textContent()).toLowerCase();
    check(head.includes("champion") && head.includes("top of the tower"), `${tag}: summit boss head names the champion top`, head);
    await ctx.close();
  }
}

for (const [vp, tag] of [
  [DESKTOP, "desktop"],
  [PHONE, "phone"],
]) {
  await confirmScenario(vp, tag);
  await climbGuardScenario(vp, tag);
}

await browser.close();
disarm();
finish("probe-challenge");
