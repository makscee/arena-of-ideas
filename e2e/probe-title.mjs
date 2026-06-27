// PRD #015 slice 3 — the title screen as the landing. Pins, against the LIVE
// app at desktop and 375×667:
//  1. A fresh player lands on the title: Play (not Continue), Leaderboard,
//     Codex, an inert disabled Login, dev tab nav hidden; no horizontal
//     overflow; every title tap ≥44px effective.
//  2. Play opens the new-run form; a started run reaches the shop; walking
//     home mid-run flips the entry to "Continue run".
//  3. An active mid-shop run resumes EXACTLY through Continue — same round,
//     same gold, same line (the stored state, not a fresh run).
//  4. Mid-battle: leave via home, the entry reads Continue, resuming restores
//     the parked replay position (the #014 battleResume seam, title route).
//  5. Abandon (the #014 menu) and run-end exit both land on the title reading
//     "Play" with the stored run cleared.
//  6. Codex and Leaderboard open from the title without starting a run; the
//     leaderboard shows the champion row; home walks back. The dev entry opens
//     the Battle Editor — the one dev surface (#066 slice 5 deleted the legacy
//     tab nav + battle/gauntlet views, which are now absent from the DOM).

import {
  BASE,
  DESKTOP,
  PHONE,
  armGuard,
  box,
  check,
  endedRun,
  finish,
  launch,
  openRun,
  plainShopRun,
} from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

/** A fresh page with NO stored run — the new player's cold load. With
 * `dev` set, aoi.dev.v1 is on before boot so the dev surfaces are revealed
 * (#066 slice 1: dev tools are gated on the dev-mode toggle). */
async function openFresh(viewport, dev = false) {
  const ctx = await browser.newContext({ viewport, hasTouch: viewport.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.addInitScript(
    (devOn) => {
      localStorage.removeItem("aoi.run.v1");
      if (devOn) localStorage.setItem("aoi.dev.v1", "1");
      else localStorage.removeItem("aoi.dev.v1");
    },
    dev,
  );
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  return { ctx, page };
}

// B·Arena slice B: #title-play is the always-present "▸ New Run" primary; the
// run-state signal that used to live in its text now lives in the #title-continue
// entry — shown only while a run is in progress, hidden otherwise.
const continueShown = (page) => page.locator("#title-continue").isVisible();
const continueHidden = (page) => page.locator("#title-continue").isHidden();
const noHorizontalOverflow = (page) =>
  page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1);

// ---- 1+2. fresh landing, Play → run, home flips to Continue ----------------
async function freshScenario(viewport, tag) {
  const { ctx, page } = await openFresh(viewport);
  check(await page.locator("#title-view").isVisible(), `${tag} fresh load lands on the title`);
  check(await page.locator("#run-new").isHidden(), `${tag} the run form is not the landing`);
  check(await continueHidden(page), `${tag} no active run: Continue is hidden`);
  // #066 slice 6: account/login is back ON the title (Maks's gate call —
  // slice 1 had buried it in Settings). Logged out shows the Login entry, live
  // and enabled, and a click opens the email step. The logged-in case (account
  // NAME + Logout) is exercised end-to-end against the real server in
  // probe-arena; this stays the logged-out smoke check.
  check(await page.locator("#title-login").isVisible(), `${tag} login entry is on the title (logged out)`);
  check(await page.locator("#title-login").isEnabled(), `${tag} login entry is enabled (#016 wired it)`);
  check(await page.locator("#title-id").isHidden(), `${tag} identity strip hidden while logged out`);
  await page.click("#title-login");
  await page.waitForSelector("#login-email-row:not([hidden])");
  check(await page.locator("#login-panel").isVisible(), `${tag} login opens the email step on the title`);
  await page.click("#login-cancel");
  check(await page.locator("#login-panel").isHidden(), `${tag} cancel closes the login panel`);
  check(await page.locator("#title-settings").isVisible(), `${tag} Settings entry is visible on the title`);
  check(await page.locator("#title-dev").isHidden(), `${tag} dev tools entry hidden when dev mode off (#066)`);
  check(await page.locator("#home-button").isHidden(), `${tag} home link hidden on the title itself`);
  check(await noHorizontalOverflow(page), `${tag} title has no horizontal overflow`);

  // Every visible title action is a ≥44px effective target (the buttons are
  // real boxes, no pseudo-element tricks — measure the rendered rect). #title-dev
  // is gated off here (#066), so it is not measured on the fresh title.
  for (const sel of ["#title-play", "#title-leaderboard", "#title-codex", "#title-login", "#title-settings"]) {
    const b = await box(page, sel);
    check(b.height >= 44 && b.width >= 44, `${tag} ${sel} is a ≥44px target`, `${Math.round(b.width)}×${Math.round(b.height)}`);
  }

  // Play → the run-new flow; a run starts and reaches the shop.
  await page.click("#title-play");
  await page.waitForSelector("#run-new:not([hidden])");
  check(await page.locator("#run-new").isVisible(), `${tag} Play opens the new-run form`);
  await page.fill("#run-seed", "5");
  await page.click("#run-start");
  await page.waitForSelector("#run-shop:not([hidden])");
  // Home mid-run: the title now reads Continue.
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");
  check(await continueShown(page), `${tag} active run: Continue is shown`);
  await ctx.close();
}

// ---- 3. mid-shop resume is exact -------------------------------------------
async function shopResumeScenario(viewport, tag) {
  // plainShopRun: round 1, 10 gold, Brawler on the line, 3 offers.
  const ctx = await browser.newContext({ viewport, hasTouch: viewport.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.addInitScript(
    ([key, val]) => localStorage.setItem(key, val),
    ["aoi.run.v1", plainShopRun()],
  );
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  check(await continueShown(page), `${tag} stored mid-shop run: Continue is shown`);
  await page.click("#title-play");
  await page.waitForSelector("#run-shop:not([hidden])");
  const head = await page.locator("#run-head").textContent();
  check(head.includes("round 1"), `${tag} resume keeps the round`, head);
  const gold = (await page.locator("#run-head .run-gold").textContent()).replace(/[^0-9]/g, "");
  check(gold === "10", `${tag} resume keeps the gold`, head);
  check(
    (await page.locator('#run-line [data-line="0"] .uname').textContent()) === "Brawler",
    `${tag} resume keeps the line`,
  );
  await ctx.close();
}

// ---- 4. mid-battle resume preserves the replay position --------------------
async function battleResumeScenario(viewport, tag) {
  const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
  await page.click("#run-fight");
  await page.waitForSelector("#run-battle:not([hidden])");
  // Park at a KNOWN mid-replay event (the probe-menu technique): pause, rewind
  // to 0, step to event index 2.
  await page.click("#step-next");
  for (let i = 0; i < 30; i++) {
    if (await page.locator("#step-prev").isDisabled()) break;
    await page.click("#step-prev");
  }
  await page.click("#step-next");
  await page.click("#step-next");
  const posBefore = await page.locator("#scrub").inputValue();
  check(posBefore === "2", `${tag} parked at a known mid-replay event`, `event index ${posBefore}`);
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");
  check(await continueShown(page), `${tag} mid-battle: Continue is shown`);
  await page.click("#title-play");
  await page.waitForSelector("#run-battle:not([hidden])");
  const posAfter = await page.locator("#scrub").inputValue();
  check(posAfter === posBefore, `${tag} Continue resumes the battle at the parked event`, `${posBefore} → ${posAfter}`);
  await ctx.close();
}

// ---- 5. abandon and run-end exit both land on the title, reading Play ------
async function exitScenario(viewport, tag) {
  // Abandon from the shop (the #014 menu's two-step confirm).
  {
    const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
    await page.click("#run-menu-button");
    await page.click("#run-abandon");
    await page.click("#run-abandon-yes");
    await page.waitForSelector("#title-view:not([hidden])");
    check(await continueHidden(page), `${tag} abandon lands on the title with Continue hidden`);
    check(
      (await page.evaluate(() => localStorage.getItem("aoi.run.v1"))) === null,
      `${tag} abandon clears the stored run`,
    );
    await ctx.close();
  }
  // The end screen's exit button.
  {
    const { ctx, page } = await openRun(browser, endedRun(), viewport, "#run-end:not([hidden])");
    await page.click("#run-new-run");
    await page.waitForSelector("#title-view:not([hidden])");
    check(await continueHidden(page), `${tag} run-end exit lands on the title with Continue hidden`);
    check(
      (await page.evaluate(() => localStorage.getItem("aoi.run.v1"))) === null,
      `${tag} run-end exit clears the stored run`,
    );
    await ctx.close();
  }
}

// ---- 6. codex / leaderboard / dev navigation --------------------------------
async function navScenario(viewport, tag) {
  // Dev mode on so the dev tools entry is revealed (#066 slice 1 gate).
  const { ctx, page } = await openFresh(viewport, true);

  // Codex, and home back.
  await page.click("#title-codex");
  await page.waitForSelector("#codex-view:not([hidden])");
  check(await page.locator("#codex-view").isVisible(), `${tag} Codex opens from the title`);
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");

  // Leaderboard: no run started, champion row visible.
  await page.click("#title-leaderboard");
  await page.waitForSelector("#leaderboard-view:not([hidden])");
  check(
    await page.locator("#leaderboard-body .tower-floor.is-champ").isVisible(),
    `${tag} leaderboard tower shows the champion floor`,
  );
  check(
    (await page.evaluate(() => localStorage.getItem("aoi.run.v1"))) === null,
    `${tag} opening the leaderboard starts no run`,
  );
  check(await noHorizontalOverflow(page), `${tag} leaderboard has no horizontal overflow`);
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");

  // Dev: the one low-prominence entry opens the Battle Editor — the ONE dev
  // surface (#066 slice 5 deleted the legacy tab nav + battle/gauntlet views).
  await page.click("#title-dev");
  await page.waitForSelector("#editor-view:not([hidden])");
  check(await page.locator("#editor-view").isVisible(), `${tag} dev entry opens the Battle Editor (#066 slice 5: the one dev surface)`);
  // The legacy tab nav and its battle/gauntlet views are gone from the DOM —
  // even with dev on, there is no door to them.
  check(
    (await page.locator("#views").count()) === 0,
    `${tag} the legacy dev tab nav (#views) is deleted from the DOM`,
  );
  check(
    (await page.locator("#gauntlet-view").count()) === 0,
    `${tag} the standalone gauntlet view is deleted from the DOM`,
  );
  check(
    (await page.locator("#controls").count()) === 0,
    `${tag} the inline battle picker (#controls) is deleted from the DOM`,
  );
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");
  await ctx.close();
}

for (const [viewport, tag] of [
  [PHONE, "375px"],
  [DESKTOP, "desktop"],
]) {
  await freshScenario(viewport, tag);
  await shopResumeScenario(viewport, tag);
  await exitScenario(viewport, tag);
  await navScenario(viewport, tag);
}
// The battle-position seam is geometry-free — one width keeps the probe lean
// (probe-menu sweeps the same seam at both widths via the same title route).
await battleResumeScenario(PHONE, "375px");

await browser.close();
disarm();
finish("probe-title");
