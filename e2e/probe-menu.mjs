// #014 — run menu + abandon/restart, and the folded-in battle-position fix.
// At desktop and 375×667, with REAL run states and the LIVE run-screen:
//  1. The menu control shows in every in-run phase (shop, battle, end), opens
//     as a fixed overlay, and opening/closing shifts nothing — fight/continue/
//     board positions are pixel-stable (the #012 layout-lock holds).
//  2. Abandon is a two-step confirm from shop, mid-battle, and end: a single
//     click never destroys the run; after confirm the screen lands on new-run
//     with the stored run cleared and a fresh run startable at once.
//  3. Reload after abandon lands on new-run — the abandoned run never revives.
//  4. Tab away mid-battle and back preserves the replay position (kills the
//     #012 gate-note re-mount-to-event-0 bug).
//  5. Static-occlusion guard (Cass #014 refutation): elementFromPoint sweep over
//     continue/fight/skip surfaces — 0% may resolve to #run-menu-button in every
//     phase, at both 375px and desktop. This guard would have caught the original
//     collision (63/420 of continue's surface stolen at 375px, seed 42).

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

const menuVisible = (page) => page.locator("#run-menu-button").isVisible();
const overlayOpen = (page) => page.locator("#run-menu-overlay").isVisible();
const scrubAt = (page) => page.locator("#scrub").inputValue();

/** Sweep `targetSel`'s bounding box with a grid of points and count how many
 * resolve to `#run-menu-button` (or a descendant of it) via elementFromPoint.
 * Returns { stolen, total } — stolen > 0 means the menu button occludes the
 * target and will intercept taps. The grid step is 5px for reliable coverage.
 *
 * Cass reproduced 63/420 stolen for #run-continue at 375px (seed 42, natural
 * run): a 5-column × 84-row grid over the 181×87px button. The guard fires at
 * stolen > 0 so even a 1px corner grab fails. */
async function sweepOcclusion(page, targetSel) {
  const b = await page.locator(targetSel).boundingBox();
  if (b === null) return { stolen: 0, total: 0 }; // element not visible — skip
  return page.evaluate(
    ([x0, y0, w, h]) => {
      const menuBtn = document.getElementById("run-menu-button");
      const step = 5;
      let stolen = 0;
      let total = 0;
      for (let x = x0 + 2; x < x0 + w - 2; x += step) {
        for (let y = y0 + 2; y < y0 + h - 2; y += step) {
          const el = document.elementFromPoint(x, y);
          if (el !== null && (el === menuBtn || menuBtn.contains(el))) stolen++;
          total++;
        }
      }
      return { stolen, total };
    },
    [b.x, b.y, b.width, b.height],
  );
}

/** Open the menu, abandon WITHOUT confirming twice, and prove the run survives
 * a single stray click; then confirm and prove it lands on new-run cleared. */
async function abandonFlow(page, tag, reopenSelector) {
  // Single click on "abandon" only arms the confirm — the run is untouched.
  await page.click("#run-menu-button");
  check(await overlayOpen(page), `${tag} menu opens as overlay`);
  check(await page.locator("#run-abandon-confirm").isHidden(), `${tag} confirm not armed before first abandon click`);
  await page.click("#run-abandon");
  check(await page.locator("#run-abandon-confirm").isVisible(), `${tag} first abandon click arms the confirm`);
  // The destructive screen is NOT showing yet: new-run is still hidden.
  check(await page.locator("#run-new").isHidden(), `${tag} single click never destroys the run`);
  // Back out — keep playing — and prove the run is intact (the phase returns).
  await page.click("#run-abandon-no");
  check(await page.locator("#run-abandon-confirm").isHidden(), `${tag} 'keep playing' disarms the confirm`);
  // Close the menu (✕) and prove the run is alive behind it.
  await page.click("#run-menu-close");
  check(await overlayOpen(page) === false, `${tag} menu closed after backing out`);
  await page.waitForSelector(reopenSelector);
  check(await page.locator(reopenSelector.replace(":not([hidden])", "")).isVisible(), `${tag} run still alive after backing out`);

  // Now really abandon: reopen → abandon → really abandon. The menu is closed
  // here, so the bottom-sheet does not cover its own button.
  await page.click("#run-menu-button");
  await page.click("#run-abandon");
  await page.click("#run-abandon-yes");
  await page.waitForSelector("#run-new:not([hidden])");
  check(await page.locator("#run-new").isVisible(), `${tag} confirm lands on new-run`);
  check(await menuVisible(page) === false, `${tag} menu control hidden on new-run`);
  check(await overlayOpen(page) === false, `${tag} menu overlay closed after abandon`);
  // The stored run is cleared: nothing under the run key.
  const stored = await page.evaluate(() => localStorage.getItem("aoi.run.v1"));
  check(stored === null, `${tag} stored run cleared after abandon`, JSON.stringify(stored));
  // A fresh run is startable immediately.
  await page.fill("#run-seed", "123");
  await page.click("#run-start");
  await page.waitForSelector("#run-shop:not([hidden])");
  check(await page.locator("#run-shop").isVisible(), `${tag} new run startable immediately after abandon`);
}

// ---- shop phase: visible menu, layout-lock on open/close, abandon ----------
async function shopScenario(viewport, tag) {
  const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
  check(await menuVisible(page), `${tag} shop: menu control visible`);
  // Layout-lock: opening and closing the overlay shifts nothing below it.
  const fightY0 = (await box(page, "#run-fight")).y;
  const lineY0 = (await box(page, "#run-line")).y;
  await page.click("#run-menu-button");
  check(await overlayOpen(page), `${tag} shop: overlay open`);
  const fightY1 = (await box(page, "#run-fight")).y;
  const lineY1 = (await box(page, "#run-line")).y;
  check(fightY1 === fightY0, `${tag} shop: fight Y pixel-stable across menu open`, `${fightY0} → ${fightY1}`);
  check(lineY1 === lineY0, `${tag} shop: line Y pixel-stable across menu open`, `${lineY0} → ${lineY1}`);
  await page.click("#run-menu-close");
  check(await overlayOpen(page) === false, `${tag} shop: ✕ closes the overlay`);
  const fightY2 = (await box(page, "#run-fight")).y;
  check(fightY2 === fightY0, `${tag} shop: fight Y pixel-stable across menu close`, `${fightY0} → ${fightY2}`);
  // Escape also closes.
  await page.click("#run-menu-button");
  await page.keyboard.press("Escape");
  check(await overlayOpen(page) === false, `${tag} shop: Escape closes the overlay`);
  // Two-step abandon from shop.
  await abandonFlow(page, `${tag} shop`, "#run-shop:not([hidden])");
  await ctx.close();
}

// ---- battle phase: menu visible, position preserved across tab switch ------
async function battleScenario(viewport, tag) {
  const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
  await page.click("#run-fight");
  await page.waitForSelector("#run-battle:not([hidden])");
  // Park at a KNOWN mid-replay event so the position is deterministic: pause
  // autoplay (prev/next both pause), rewind to 0, then step forward to event 3
  // (index 2). prev/next disable at the ends, so this lands the same every run.
  await page.click("#step-next"); // pauses autoplay wherever it was
  for (let i = 0; i < 30; i++) {
    if (await page.locator("#step-prev").isDisabled()) break;
    await page.click("#step-prev");
  }
  await page.click("#step-next");
  await page.click("#step-next"); // now at event index 2
  const posBefore = await scrubAt(page);
  check(posBefore === "2", `${tag} battle: parked at a known mid-replay event`, `event index ${posBefore}`);
  check(await menuVisible(page), `${tag} battle: menu control visible`);

  // Layout-lock on the board: opening the menu shifts the board nothing.
  const boardY0 = (await box(page, "#run-battle-mount")).y;
  await page.click("#run-menu-button");
  check(await overlayOpen(page), `${tag} battle: overlay open`);
  const boardY1 = (await box(page, "#run-battle-mount")).y;
  check(boardY1 === boardY0, `${tag} battle: board Y pixel-stable across menu open`, `${boardY0} → ${boardY1}`);
  await page.keyboard.press("Escape");

  // The folded-in fix: leave the battle tab (codex) and return — the replay
  // resumes at the parked event, never event 0.
  await page.click("#view-codex");
  await page.waitForSelector("#codex-view:not([hidden])");
  await page.click("#view-run");
  await page.waitForSelector("#run-battle:not([hidden])");
  const posAfter = await scrubAt(page);
  check(posAfter === posBefore, `${tag} battle: replay position preserved across tab switch`, `${posBefore} → ${posAfter}`);

  // Abandon mid-battle: two-step, lands on new-run.
  await abandonFlow(page, `${tag} battle`, "#run-battle:not([hidden])");
  await ctx.close();
}

// ---- end phase: menu visible, abandon works --------------------------------
async function endScenario(viewport, tag) {
  const { ctx, page } = await openRun(browser, endedRun(), viewport, "#run-end:not([hidden])");
  check(await menuVisible(page), `${tag} end: menu control visible`);
  const newRunY0 = (await box(page, "#run-new-run")).y;
  await page.click("#run-menu-button");
  const newRunY1 = (await box(page, "#run-new-run")).y;
  check(newRunY1 === newRunY0, `${tag} end: new-run button Y pixel-stable across menu open`, `${newRunY0} → ${newRunY1}`);
  await page.keyboard.press("Escape");
  await abandonFlow(page, `${tag} end`, "#run-end:not([hidden])");
  await ctx.close();
}

// ---- reload after abandon → new-run, no revival ----------------------------
async function reloadScenario(viewport, tag) {
  const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
  await page.click("#run-menu-button");
  await page.click("#run-abandon");
  await page.click("#run-abandon-yes");
  await page.waitForSelector("#run-new:not([hidden])");
  check((await page.evaluate(() => localStorage.getItem("aoi.run.v1"))) === null, `${tag} stored run cleared before reload`);
  // "Reload" via a fresh page in the SAME context (shared localStorage), with
  // NO run injected — exactly what a real reload reads after the abandon. (The
  // openRun init-script re-seeds storage on a literal reload, masking this.)
  const fresh = await ctx.newPage();
  fresh.setDefaultTimeout(15_000);
  await fresh.goto(BASE, { waitUntil: "domcontentloaded" });
  await fresh.waitForSelector("#run-new:not([hidden])");
  check(await fresh.locator("#run-new").isVisible(), `${tag} reload after abandon lands on new-run`);
  check(await fresh.locator("#run-shop").isHidden(), `${tag} reload after abandon does not revive the shop`);
  await ctx.close();
}

for (const [viewport, tag] of [
  [PHONE, "375px"],
  [DESKTOP, "desktop"],
]) {
  await shopScenario(viewport, tag);
  await battleScenario(viewport, tag);
  await endScenario(viewport, tag);
  await reloadScenario(viewport, tag);
}

// ---- static-occlusion guard (Cass #014 refutation) -------------------------
// Sweep the interactive surfaces of continue/fight/skip vs #run-menu-button in
// every phase at 375px and desktop. 0 stolen points required in all cases.
// The original defect: 63/420 of continue's surface stolen at 375px (seed 42)
// because the fixed bottom: 0.6rem button sat inside the sticky bar's zone.
//
// Fight (shop) and skip (battle pre-reveal) are checked for completeness —
// Cass found 0% for those in the original run; this guard keeps them at 0.

async function occlusionScenario(viewport, tag) {
  // --- shop phase: check #run-fight ---
  {
    const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
    const { stolen, total } = await sweepOcclusion(page, "#run-fight");
    check(stolen === 0, `${tag} shop: #run-fight not occluded by menu button`, `${stolen}/${total} surface points stolen`);
    await ctx.close();
  }

  // --- battle pre-reveal: check #run-skip ---
  {
    const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
    await page.click("#run-fight");
    await page.waitForSelector("#run-skip:not([hidden])");
    // Pause autoplay so skip stays visible long enough to sweep.
    await page.click("#step-next");
    const { stolen, total } = await sweepOcclusion(page, "#run-skip");
    check(stolen === 0, `${tag} battle pre-reveal: #run-skip not occluded by menu button`, `${stolen}/${total} surface points stolen`);
    await ctx.close();
  }

  // --- battle transport AT the mountViewer landing scroll (Cass #014 round-2) ---
  // The structural finding: a fixed bottom-right menu button overlaps the scrub
  // (and the step buttons beside it) at exactly the scroll offset mountViewer's
  // nudge lands the player on — the scrub parked just above the battle bar. The
  // round-1 raise (bottom: 7rem) only relocated the collision to that landing y.
  // Sweep scrub/step-prev/step-next here, at the landing scroll (NOT scroll-top):
  // this block FAILS on pre-fix main and passes once the button docks in the bar.
  for (const target of ["#scrub", "#step-prev", "#step-next"]) {
    const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
    await page.click("#run-fight");
    await page.waitForSelector("#run-skip:not([hidden])");
    // Pause autoplay (step-next pauses wherever it is) so the transport holds
    // still for the sweep; the landing scroll is left untouched.
    await page.click("#step-next");
    const { stolen, total } = await sweepOcclusion(page, target);
    check(stolen === 0, `${tag} battle landing: ${target} not occluded by menu button`, `${stolen}/${total} surface points stolen`);
    await ctx.close();
  }

  // --- battle revealed: check #run-continue (the Cass-reproduced collision) ---
  {
    const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
    await page.click("#run-fight");
    await page.waitForSelector("#run-skip:not([hidden])");
    await page.click("#run-skip");
    await page.waitForSelector("#run-continue:not([hidden])");
    const { stolen, total } = await sweepOcclusion(page, "#run-continue");
    check(stolen === 0, `${tag} battle revealed: #run-continue not occluded by menu button`, `${stolen}/${total} surface points stolen`);
    await ctx.close();
  }

  // --- end phase: check #run-new-run ---
  {
    const { ctx, page } = await openRun(browser, endedRun(), viewport, "#run-end:not([hidden])");
    const { stolen, total } = await sweepOcclusion(page, "#run-new-run");
    check(stolen === 0, `${tag} end: #run-new-run not occluded by menu button`, `${stolen}/${total} surface points stolen`);
    await ctx.close();
  }
}

for (const [viewport, tag] of [
  [PHONE, "375px"],
  [DESKTOP, "desktop"],
]) {
  await occlusionScenario(viewport, tag);
}

await browser.close();
disarm();
finish("probe-menu");
