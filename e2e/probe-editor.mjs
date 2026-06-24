// PRD #066 slice 2 — the Battle Editor. Drives the LIVE app (no DOM stubbing,
// the loop exercised end to end) at desktop and 375×667:
//   1. Dev mode on → the title's dev entry opens the Battle Editor (two team
//      columns, a seed control, Fight).
//   2. Build two teams: use a quick loader on one side, the palette to place a
//      unit on the other, and apply a per-slot stat override.
//   3. Lock a seed and Fight — the shared viewer mounts and a replay plays
//      (the battle log fills, the playhead reaches the last event).
//   4. Edit a team (bump a stat hard) and Fight again UNDER LOCK — the replay
//      differs from the first run at the SAME seed (the edit changed the
//      result). This is the slice's whole point: a locked seed isolates the
//      edit, so a different log proves the edit, not RNG, moved the outcome.
//
// The replay signature is the live viewer's own output — the full battle-log
// text plus the event count — never anything the probe computes.

import { BASE, DESKTOP, PHONE, armGuard, box, check, finish, launch } from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

/** Fresh page, dev mode pre-enabled via the run-store key, landed on the
 * Battle Editor through the real dev entry (the path a developer walks). */
async function openEditor(viewport) {
  const ctx = await browser.newContext({ viewport, hasTouch: viewport.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.addInitScript(() => localStorage.setItem("aoi.dev.v1", "1"));
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  await page.click("#title-dev");
  await page.waitForSelector("#editor-view:not([hidden])");
  return { ctx, page };
}

/** Drive the borrowed viewer to its last event and read its replay signature:
 * the full battle-log text and the "event M/M" label. Setting #scrub to its max
 * and firing input is the viewer's own seek seam (viewer.ts) — deterministic,
 * no race on autoplay. */
async function replaySignature(page) {
  await page.waitForSelector("#be-mount #result:not([hidden])");
  await page.waitForSelector("#be-mount #battle-log .log-text");
  await page.evaluate(() => {
    const scrub = document.querySelector("#be-mount #scrub");
    scrub.value = scrub.max;
    scrub.dispatchEvent(new Event("input", { bubbles: true }));
  });
  const label = await page.locator("#be-mount #step-label").innerText();
  const log = await page.locator("#be-mount #battle-log").innerText();
  return { label, log };
}

/** Count the units currently placed in a column. The column ids are lowercase
 * (#be-col-a / #be-col-b); ID selectors are case-sensitive, so normalise. */
async function slotCount(page, side) {
  return page.locator(`#be-col-${side.toLowerCase()} .be-slot`).count();
}

/** Wait until a column holds exactly `n` slots, then return the count. A
 * loader/palette re-render is synchronous in the app, but selectOption/click
 * resolve before the handler's DOM swap settles — poll the live count rather
 * than read it one-shot. */
async function waitSlots(page, side, n) {
  for (let i = 0; i < 60; i++) {
    if ((await slotCount(page, side)) === n) return n;
    await page.waitForTimeout(50);
  }
  return slotCount(page, side); // one last read; the check reports the mismatch
}

async function scenario(viewport, tag) {
  const { ctx, page } = await openEditor(viewport);

  // 1. The editor surface: two columns, a Fight button, a seed control.
  check(await page.locator("#be-col-a").isVisible(), `${tag} Team A column is shown`);
  check(await page.locator("#be-col-b").isVisible(), `${tag} Team B column is shown`);
  check(await page.locator("#be-fight").isVisible(), `${tag} Fight button is shown`);
  check(await page.locator("#be-lock").isChecked(), `${tag} the seed is LOCKED by default`);
  const fb = await box(page, "#be-fight");
  check(fb.height >= 30, `${tag} Fight is a real target`, `${Math.round(fb.height)}px`);

  // 2a. Quick loader on Team A: clear it, then load a shipped template — the
  // count goes from 0 back to a full template.
  await page.click("#be-col-a .be-clear");
  check((await waitSlots(page, "A", 0)) === 0, `${tag} clear empties Team A`);
  await page.selectOption(
    "#be-col-a .be-load",
    await page.locator('#be-col-a .be-load option[value^="shipped:"]').first().getAttribute("value"),
  );
  const aLoaded = await waitSlots(page, "A", 3); // Team Alpha/Beta ship 3 units
  check(aLoaded === 3, `${tag} a shipped-template quick loader fills Team A`, `${aLoaded} units`);

  // 2b. Palette place on Team B: clear it, open the palette, place units.
  await page.click("#be-col-b .be-clear");
  check((await waitSlots(page, "B", 0)) === 0, `${tag} clear empties Team B`);
  await page.click('#be-col-b [data-add="B"]');
  await page.waitForSelector(".palette:not([hidden]) [data-pick]");
  check(await page.locator(".palette").isVisible(), `${tag} the unit palette opens`);
  await page.click(".palette [data-pick]"); // place the first pool unit
  check((await waitSlots(page, "B", 1)) === 1, `${tag} palette pick places a unit in Team B`);
  // Place two more so B is a real team, not a single unit.
  for (let k = 2; k <= 3; k++) {
    await page.click('#be-col-b [data-add="B"]');
    await page.waitForSelector(".palette:not([hidden]) [data-pick]");
    await page.click(".palette [data-pick]");
    await waitSlots(page, "B", k);
  }
  check((await slotCount(page, "B")) === 3, `${tag} Team B has 3 units after palette picks`);

  // 2c. Per-slot stat override on Team A's front unit — read it back.
  const aHp = page.locator('#be-col-a .be-slot[data-i="0"] input[data-field="hp"]');
  await aHp.fill("9");
  await aHp.dispatchEvent("input");
  check((await aHp.inputValue()) === "9", `${tag} a per-slot hp override sticks on Team A`);

  // 3. Lock a known seed and fight — capture the first replay's signature.
  check(await page.locator("#be-lock").isChecked(), `${tag} seed still locked before the first fight`);
  await page.fill("#be-seed", "12345");
  await page.click("#be-fight");
  const first = await replaySignature(page);
  check(/event \d+\/\d+/.test(first.label), `${tag} the first fight plays a replay`, first.label);
  check(first.log.length > 0, `${tag} the first fight's battle log filled`);
  check(
    (await page.locator("#be-seed").inputValue()) === "12345",
    `${tag} the locked seed is unchanged by the first fight`,
  );

  // 4. Edit Team A's front unit HARD (overwhelming power) and re-fight UNDER
  //    LOCK — the same seed against a changed team must replay differently.
  const aPwr = page.locator('#be-col-a .be-slot[data-i="0"] input[data-field="pwr"]');
  await aPwr.fill("99");
  await aPwr.dispatchEvent("input");
  const aHpBuff = page.locator('#be-col-a .be-slot[data-i="0"] input[data-field="hp"]');
  await aHpBuff.fill("99");
  await aHpBuff.dispatchEvent("input");
  check(await page.locator("#be-lock").isChecked(), `${tag} seed still locked before the re-fight`);
  await page.click("#be-fight");
  const second = await replaySignature(page);
  check(
    (await page.locator("#be-seed").inputValue()) === "12345",
    `${tag} the seed held across the edit→re-fight (lock isolates the edit)`,
  );
  check(
    second.log !== first.log || second.label !== first.label,
    `${tag} the edit changed the replay at the SAME seed (locked)`,
    `${first.label} → ${second.label}`,
  );

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
finish("probe-editor");
