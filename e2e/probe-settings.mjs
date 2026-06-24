// PRD #066 slice 1 — the Settings surface and the dev-mode gate. Pins, against
// the LIVE app at desktop and 375×667:
//  1. ⚙ is reachable from the title and opens Settings (account block + the
//     dev-mode toggle); home walks back.
//  2. The dev toggle is OFF by default (fresh profile) — and with it off the
//     dev tools entry is hidden on the title (no dev surface leaks).
//  3. Turning it ON reveals the dev tools entry on the title; the entry opens
//     the existing dev surface (the battle sandbox).
//  4. Turning it OFF again hides the dev tools entry on the title.
//  5. The toggle state SURVIVES a page reload — on stays on, persisted in
//     aoi.dev.v1 (the run-store key), reflected by both Settings and the title.
//
// The gate is a local convenience switch, not a security boundary — this probe
// measures only which surfaces a developer sees on this device.

import { BASE, DESKTOP, PHONE, armGuard, box, check, finish, launch } from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

/** A fresh page with NO stored run and dev mode cleared — the cold load a new
 * player meets, where the dev toggle must read off. The clear runs once before
 * the first load (a fresh context starts empty anyway), NOT as an init script:
 * the reload check below must see the dev flag the app persisted survive. */
async function openFresh(viewport) {
  const ctx = await browser.newContext({ viewport, hasTouch: viewport.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  return { ctx, page };
}

async function openSettings(page) {
  await page.click("#title-settings");
  await page.waitForSelector("#settings-view:not([hidden])");
}

async function homeToTitle(page) {
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");
}

async function scenario(viewport, tag) {
  const { ctx, page } = await openFresh(viewport);

  // 1. ⚙ reachable from the title, opening the Settings surface.
  check(await page.locator("#title-settings").isVisible(), `${tag} ⚙ Settings entry is on the title`);
  const sb = await box(page, "#title-settings");
  check(sb.height >= 44 && sb.width >= 44, `${tag} ⚙ is a ≥44px target`, `${Math.round(sb.width)}×${Math.round(sb.height)}`);
  await openSettings(page);
  check(await page.locator("#settings-view").isVisible(), `${tag} ⚙ opens the Settings surface`);
  check(await page.locator("#settings-dev-toggle").isVisible(), `${tag} Settings shows the dev-mode toggle`);
  check(await page.locator("#title-login").isVisible(), `${tag} Settings shows the account block (login/logout)`);

  // 2. The toggle is OFF by default, and dev tools are hidden on the title.
  check(!(await page.locator("#settings-dev-toggle").isChecked()), `${tag} dev toggle is OFF by default`);
  check(
    (await page.evaluate(() => localStorage.getItem("aoi.dev.v1"))) === null,
    `${tag} aoi.dev.v1 is unset by default`,
  );
  await homeToTitle(page);
  check(await page.locator("#title-dev").isHidden(), `${tag} dev tools entry hidden on the title while dev off`);

  // 3. Turn it ON → the dev tools entry appears, and opens the dev surface.
  await openSettings(page);
  await page.click("#settings-dev-toggle");
  check(await page.locator("#settings-dev-toggle").isChecked(), `${tag} the toggle reads on after the click`);
  check(
    (await page.evaluate(() => localStorage.getItem("aoi.dev.v1"))) === "1",
    `${tag} turning it on persists aoi.dev.v1=1`,
  );
  await homeToTitle(page);
  check(await page.locator("#title-dev").isVisible(), `${tag} dev tools entry revealed on the title while dev on`);
  await page.click("#title-dev");
  await page.waitForSelector("#views:not([hidden])");
  check(await page.locator("#battle-view").isVisible(), `${tag} the dev tools entry opens the existing dev surface`);
  await homeToTitle(page);

  // 4. Turn it OFF again → the dev tools entry hides.
  await openSettings(page);
  check(await page.locator("#settings-dev-toggle").isChecked(), `${tag} the toggle reflects the on state on reopen`);
  await page.click("#settings-dev-toggle");
  check(!(await page.locator("#settings-dev-toggle").isChecked()), `${tag} the toggle reads off after a second click`);
  check(
    (await page.evaluate(() => localStorage.getItem("aoi.dev.v1"))) === null,
    `${tag} turning it off clears aoi.dev.v1`,
  );
  await homeToTitle(page);
  check(await page.locator("#title-dev").isHidden(), `${tag} dev tools entry hidden again after turning dev off`);

  // 5. Persistence across reload: turn on, reload, the state survives — the
  // title shows the dev entry and Settings reflects the on toggle.
  await openSettings(page);
  await page.click("#settings-dev-toggle"); // back on
  check(await page.locator("#settings-dev-toggle").isChecked(), `${tag} dev on again before reload`);
  await page.reload({ waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  check(await page.locator("#title-dev").isVisible(), `${tag} dev entry survives a reload (still revealed)`);
  await openSettings(page);
  check(await page.locator("#settings-dev-toggle").isChecked(), `${tag} the toggle survives a reload (still on)`);

  await ctx.close();
}

for (const [viewport, tag] of [
  [PHONE, "375px"],
  [DESKTOP, "desktop"],
]) {
  await scenario(viewport, tag);
}

await browser.close();
disarm();
finish("probe-settings");
