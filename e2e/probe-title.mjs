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
//     leaderboard shows the champion row; home walks back. The dev entry
//     reveals the tab nav and the existing tabs keep switching views.

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

const playLabel = (page) => page.locator("#title-play").textContent();
const noHorizontalOverflow = (page) =>
  page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1);

// ---- 1+2. fresh landing, Play → run, home flips to Continue ----------------
async function freshScenario(viewport, tag) {
  const { ctx, page } = await openFresh(viewport);
  check(await page.locator("#title-view").isVisible(), `${tag} fresh load lands on the title`);
  check(await page.locator("#run-new").isHidden(), `${tag} the run form is not the landing`);
  check((await playLabel(page)) === "Play", `${tag} no active run: the entry reads Play`);
  // #066 slice 1: account/login moved into Settings (⚙). The login entry is no
  // longer on the title — it lives in the Settings surface, live and enabled,
  // and a click opens the email step (probe-arena walks the full flow; this
  // and probe-settings stay the smoke checks).
  check(await page.locator("#title-login").isHidden(), `${tag} login entry is not on the title (moved to Settings)`);
  check(await page.locator("#title-settings").isVisible(), `${tag} Settings entry is visible on the title`);
  await page.click("#title-settings");
  await page.waitForSelector("#settings-view:not([hidden])");
  check(await page.locator("#title-login").isVisible(), `${tag} Settings shows the login entry`);
  check(await page.locator("#title-login").isEnabled(), `${tag} login entry is enabled (#016 wired it)`);
  await page.click("#title-login");
  await page.waitForSelector("#login-email-row:not([hidden])");
  check(await page.locator("#login-panel").isVisible(), `${tag} login opens the email step`);
  await page.click("#login-cancel");
  check(await page.locator("#login-panel").isHidden(), `${tag} cancel closes the login panel`);
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");
  check(await page.locator("#views").isHidden(), `${tag} dev tab nav hidden on the title`);
  check(await page.locator("#title-dev").isHidden(), `${tag} dev tools entry hidden when dev mode off (#066)`);
  check(await page.locator("#home-button").isHidden(), `${tag} home link hidden on the title itself`);
  check(await noHorizontalOverflow(page), `${tag} title has no horizontal overflow`);

  // Every visible title action is a ≥44px effective target (the buttons are
  // real boxes, no pseudo-element tricks — measure the rendered rect). #title-dev
  // is gated off here (#066), so it is not measured on the fresh title.
  for (const sel of ["#title-play", "#title-leaderboard", "#title-codex", "#title-settings"]) {
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
  check((await playLabel(page)) === "Continue run", `${tag} active run: the entry reads Continue run`);
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
  check((await playLabel(page)) === "Continue run", `${tag} stored mid-shop run: entry reads Continue run`);
  await page.click("#title-play");
  await page.waitForSelector("#run-shop:not([hidden])");
  const head = await page.locator("#run-head").textContent();
  check(head.includes("round 1"), `${tag} resume keeps the round`, head);
  check(head.includes("10 gold"), `${tag} resume keeps the gold`, head);
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
  check((await playLabel(page)) === "Continue run", `${tag} mid-battle: entry reads Continue run`);
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
    check((await playLabel(page)) === "Play", `${tag} abandon lands on the title reading Play`);
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
    check((await playLabel(page)) === "Play", `${tag} run-end exit lands on the title reading Play`);
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
  check(await page.locator("#leaderboard-body .lv-champ").isVisible(), `${tag} leaderboard shows the champion row`);
  check(
    (await page.evaluate(() => localStorage.getItem("aoi.run.v1"))) === null,
    `${tag} opening the leaderboard starts no run`,
  );
  check(await noHorizontalOverflow(page), `${tag} leaderboard has no horizontal overflow`);
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");

  // Dev: one low-prominence entry reveals the tab nav and lands on the Battle
  // Editor — the one dev surface (#066 slice 2). The legacy battle/gauntlet
  // tabs stay reachable until slice 5 deletes them.
  await page.click("#title-dev");
  await page.waitForSelector("#views:not([hidden])");
  check(await page.locator("#editor-view").isVisible(), `${tag} dev entry lands on the Battle Editor (#066 slice 2)`);
  await page.click("#view-battle");
  check(await page.locator("#battle-view").isVisible(), `${tag} battle tab still switches once revealed`);
  await page.click("#view-gauntlet");
  check(await page.locator("#gauntlet-view").isVisible(), `${tag} gauntlet tab switches once revealed`);
  await page.click("#view-editor");
  check(await page.locator("#editor-view").isVisible(), `${tag} editor tab switches once revealed`);
  await page.click("#view-run");
  check(await page.locator("#run-view").isVisible(), `${tag} run tab switches once revealed`);
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");
  check(await page.locator("#views").isVisible(), `${tag} dev nav stays revealed for the session`);
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
