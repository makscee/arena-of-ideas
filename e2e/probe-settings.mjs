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
//  6. (#066 slice 4) In a run: the DEV panel is hidden when dev is off, shown
//     when dev is on; +gold raises the spendable gold; spawn-any-unit drops a
//     unit into the shop (the palette, reused from slice 2).
//  7. (#066 slice 5, the fresh-context verify) A non-dev player sees ONLY the
//     player entries (Play/Continue, Leaderboard, Codex, ⚙) and NO dev surface
//     by ANY path: the legacy inline picker, gauntlet view, and dev tab nav are
//     gone from the DOM, a direct dev hash does not route, and #codex/ still does.
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

/** The visible gold in the run header, as a number. */
async function shopGold(page) {
  const text = await page.locator("#run-head .run-gold").textContent();
  return Number(text.replace(/[^0-9]/g, ""));
}

/** Set dev mode, start a fresh run, land on the shop. Returns the page/ctx. */
async function startRunWithDev(viewport, on) {
  const { ctx, page } = await openFresh(viewport);
  if (on) {
    await openSettings(page);
    await page.click("#settings-dev-toggle");
    await homeToTitle(page);
  }
  await page.click("#title-play");
  await page.waitForSelector("#run-new:not([hidden])");
  await page.fill("#run-seed", "123");
  await page.click("#run-start");
  await page.waitForSelector("#run-shop:not([hidden])");
  return { ctx, page };
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
  await page.waitForSelector("#editor-view:not([hidden])");
  check(await page.locator("#editor-view").isVisible(), `${tag} the dev tools entry opens the Battle Editor (#066 slice 2)`);
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

/** #066 slice 5 — the fresh-context play-through verify, folded in: a non-dev
 * player sees ONLY the player entries (Play/Continue, Leaderboard, Codex, ⚙)
 * and NO dev surface by ANY path — the legacy surfaces (inline battle picker,
 * gauntlet view, dev tab nav) are gone from the DOM, and a direct dev hash does
 * not route to them. */
async function noLeakScenario(viewport, tag) {
  const { ctx, page } = await openFresh(viewport);

  // The title shows the player entries plus ⚙ — and NOT the dev tools entry.
  for (const sel of ["#title-play", "#title-leaderboard", "#title-codex", "#title-settings"]) {
    check(await page.locator(sel).isVisible(), `${tag} non-dev title shows ${sel}`);
  }
  check(await page.locator("#title-dev").isHidden(), `${tag} non-dev title hides the dev tools entry`);

  // The legacy dev surfaces are deleted from the DOM entirely — not merely
  // hidden. There is nothing to reveal, by any path.
  for (const [sel, name] of [
    ["#views", "the dev tab nav"],
    ["#gauntlet-view", "the standalone gauntlet view"],
    ["#controls", "the inline battle picker"],
    ["#g-challenger", "the gauntlet challenger picker"],
    ["#team-a", "the inline team-A select"],
    ["#view-battle", "the battle tab"],
    ["#view-gauntlet", "the gauntlet tab"],
  ]) {
    check((await page.locator(sel).count()) === 0, `${tag} ${name} (${sel}) is absent from the DOM`);
  }

  // A direct hash to a legacy dev view does NOT route there: only #codex/ is a
  // hash route; any other hash leaves the title showing (the dev surfaces are
  // unreachable by URL too). #battle-view survives only as the viewer's DOM
  // home, never shown as a screen — it stays hidden under a battle hash.
  for (const hash of ["#battle", "#gauntlet", "#editor", "#views"]) {
    await page.evaluate((h) => { window.location.hash = h; }, hash);
    await page.waitForSelector("#title-view:not([hidden])"); // still the title
    check(await page.locator("#title-view").isVisible(), `${tag} ${hash} does not route to a dev surface (title still shown)`);
    check(await page.locator("#editor-view").isHidden(), `${tag} ${hash} does not reveal the editor`);
    check(await page.locator("#battle-view").isHidden(), `${tag} ${hash} does not reveal the battle host`);
  }
  // The codex deep-link route still works (regression guard for the one real hash route).
  await page.evaluate(() => { window.location.hash = "#codex/"; });
  await page.waitForSelector("#codex-view:not([hidden])");
  check(await page.locator("#codex-view").isVisible(), `${tag} the codex deep-link hash still routes`);

  await ctx.close();
}

/** #066 slice 4: the in-run DEV panel — gated, and its cheats mutate the run. */
async function devPanelScenario(viewport, tag) {
  // Dev OFF: a run shows NO DEV panel — a normal player never sees cheats.
  {
    const { ctx, page } = await startRunWithDev(viewport, false);
    check(await page.locator("#run-dev").isHidden(), `${tag} DEV panel hidden in a run while dev off`);
    await ctx.close();
  }

  // Dev ON: the DEV panel is there, and its cheats take.
  const { ctx, page } = await startRunWithDev(viewport, true);
  check(await page.locator("#run-dev").isVisible(), `${tag} DEV panel shown in a run while dev on`);

  // Open the collapsible panel (a <details>) and read the starting gold.
  await page.click("#run-dev > summary");
  const gold0 = await shopGold(page);

  // +gold raises the visible, spendable gold (the shop re-renders).
  await page.click("#dev-gold-plus");
  const gold1 = await shopGold(page);
  check(gold1 === gold0 + 10, `${tag} +gold raises the run's visible gold`, `${gold0} → ${gold1}`);

  // spawn-any-unit → shop: the palette (slice 2's component) opens; a pick
  // lands a new offer in the shop row.
  const offers0 = await page.locator("#run-shop-row [data-offer]").count();
  await page.click("#dev-spawn-shop");
  await page.waitForSelector("#dev-palette:not([hidden]) [data-pick]");
  check(await page.locator("#dev-palette").isVisible(), `${tag} spawn-any-unit opens the palette`);
  await page.click("#dev-palette [data-pick]");
  await page.waitForFunction(
    (n) => document.querySelectorAll("#run-shop-row [data-offer]").length > n,
    offers0,
  );
  const offers1 = await page.locator("#run-shop-row [data-offer]").count();
  check(offers1 === offers0 + 1, `${tag} spawn-any-unit adds a unit to the shop`, `${offers0} → ${offers1}`);

  // spawn-any-unit → team: a pick lands a new line unit.
  const line0 = await page.locator("#run-line [data-line]").count();
  await page.click("#dev-spawn-team");
  await page.waitForSelector("#dev-palette:not([hidden]) [data-pick]");
  await page.click("#dev-palette [data-pick]");
  await page.waitForFunction(
    (n) => document.querySelectorAll("#run-line [data-line]").length > n,
    line0,
  );
  const line1 = await page.locator("#run-line [data-line]").count();
  check(line1 === line0 + 1, `${tag} spawn-any-unit adds a unit to the team`, `${line0} → ${line1}`);

  await ctx.close();
}

for (const [viewport, tag] of [
  [PHONE, "375px"],
  [DESKTOP, "desktop"],
]) {
  await scenario(viewport, tag);
  await noLeakScenario(viewport, tag);
  await devPanelScenario(viewport, tag);
}

await browser.close();
disarm();
finish("probe-settings");
