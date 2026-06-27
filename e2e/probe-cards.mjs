// PRD #015 slice 1 / #078 — the one uniform unit card. Pins, against the LIVE
// app, the card-contract on the surfaces that still wear the LEGACY card.
//
// #080 update: the SHOP moved its offers (full) + team line (compact) to the new
// B·Arena family card — those two variants are pinned by probe-card.mjs. The
// LADDER champ strip and the BATTLE BOARD still render the legacy uniform card
// (their per-feature restyle is 083/085), so the "one card, one size" contract
// is pinned HERE for the surfaces that still share it:
//  1. Structure: the ladder card and the battle-board card share the SAME legacy
//     skeleton (shape art, name, framed hp/pwr, chips).
//  2. Battle affordances survive: current/max hp on board cards, front tag.
//  3. 375px stays clean: no horizontal overflow, and the 5-unit board line still
//     sits five abreast (one vertical column) inside the viewport.
//  4. The inspector overlay still anchors to the (now compact) line card.

import {
  DESKTOP,
  PHONE,
  armGuard,
  box,
  check,
  finish,
  launch,
  lineFullRun,
  openRun,
  targetsRun,
} from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

/** The card's structural signature: which shared pieces it carries, and the
 * order of its direct children — identical across contexts iff one component
 * rendered them all. */
async function signature(page, sel) {
  return page.$eval(sel, (card) => {
    const pieces = ["svg.shape", ".uname", ".unums .hp", ".unums .pwr", ".chips"]
      .filter((w) => card.querySelector(w) !== null)
      .join(",");
    const order = [...card.children]
      .map((c) => (c.tagName.toLowerCase() === "svg" ? "shape" : (c.getAttribute("class") ?? "").split(" ")[0]))
      .filter((c) => ["shape", "uname", "unums", "chips", "front-tag", "run-lvl"].includes(c))
      .join(",");
    return `${pieces} | ${order}`;
  });
}

const noHorizontalOverflow = (page) =>
  page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1);

const LEGACY_SKELETON = "svg.shape,.uname,.unums .hp,.unums .pwr,.chips";

// ---------- shop wears the #080 cards; the ladder keeps the legacy card -------

for (const [viewport, tag] of [
  [PHONE, "375px"],
  [DESKTOP, "desktop"],
]) {
  const { ctx, page } = await openRun(browser, targetsRun(), viewport);

  // The shop's two surfaces now mount the #080 family card (full + compact) —
  // their detail is pinned by probe-card.mjs; here we just confirm they swapped.
  check(
    (await page.$('#run-shop-row [data-offer="0"].unit-b.is-full')) !== null,
    `${tag} shop offers render the #080 FULL card`,
  );
  check(
    (await page.$('#run-line [data-line="0"].unit-b.is-compact')) !== null,
    `${tag} team line renders the #080 COMPACT card`,
  );

  // The ladder champ strip still wears the legacy uniform card + skeleton.
  const ladder = await signature(page, ".lv-champ .unit");
  check(ladder.startsWith(LEGACY_SKELETON), `${tag} ladder card keeps the legacy skeleton`, ladder);

  if (viewport === PHONE) {
    check(await noHorizontalOverflow(page), `${tag} shop screen has no horizontal overflow`);
  }

  // Inspector still anchors to the (compact) line card: popover pinned to it on
  // a desk, bottom sheet at phone width.
  await page.click('#run-line [data-line="0"] .uname');
  await page.waitForSelector("#inspect-overlay:not([hidden])");
  const overlay = await box(page, "#inspect-overlay");
  if (viewport === DESKTOP) {
    const card = await box(page, '#run-line [data-line="0"]');
    const gapBelow = overlay.y - (card.y + card.height);
    const gapAbove = card.y - (overlay.y + overlay.height);
    check(
      (gapBelow >= 0 && gapBelow <= 12) || (gapAbove >= 0 && gapAbove <= 12),
      `${tag} inspector popover pinned to the card`,
      `gapBelow ${gapBelow.toFixed(1)}, gapAbove ${gapAbove.toFixed(1)}`,
    );
  } else {
    const sheet = await page.$eval("#inspect-overlay", (el) => el.classList.contains("sheet"));
    check(sheet, `${tag} inspector opens as the bottom sheet`);
    check(
      Math.abs(overlay.y + overlay.height - viewport.height) <= 1,
      `${tag} sheet sits on the viewport bottom`,
      `bottom ${(overlay.y + overlay.height).toFixed(1)} vs ${viewport.height}`,
    );
  }
  await page.keyboard.press("Escape");
  await ctx.close();
}

// ---------- the battle board wears the legacy card --------------------------

for (const [viewport, tag] of [
  [PHONE, "375px"],
  [DESKTOP, "desktop"],
]) {
  const { ctx, page } = await openRun(browser, lineFullRun(), viewport);
  await page.click("#run-fight");
  await page.waitForSelector("#board .unit[data-unit]");
  const board = await signature(page, '#board .side[data-side="A"] .line .unit');
  check(board.startsWith(LEGACY_SKELETON), `${tag} board card keeps the legacy skeleton`, board);

  // The board renders the legacy uniform card and is the one surface that SCALES
  // it to fit the focal three-column replay stage (#065): a wrapper resize, never
  // larger than the one fixed legacy size (7rem ≈ 112px), and at 375px it shrinks
  // further (the --side-col phone override) so 5v5 fits the stage.
  const boardW = (await box(page, '#board .side[data-side="A"] .line .unit:first-child')).width;
  check(boardW <= 113, `${tag} board card never exceeds the fixed legacy size (#078)`, `board ${boardW.toFixed(1)}`);
  check(
    await page.$eval('#board .side[data-side="A"] .line .unit .hp', (el) => /^\d+\/\d+$/.test(el.textContent)),
    `${tag} board card shows current/max hp`,
  );
  check(
    (await page.$('#board .side[data-side="A"] .line .unit:first-child .front-tag')) !== null,
    `${tag} board front card keeps its marker`,
  );
  if (viewport === PHONE) {
    const cards = await page.$$eval('#board .side[data-side="A"] .line .unit', (els) =>
      els.map((el) => {
        const r = el.getBoundingClientRect();
        return { x: r.x, y: r.y, right: r.right };
      }),
    );
    check(cards.length === 5, "375px board line renders all five cards", `${cards.length}`);
    check(
      new Set(cards.map((c) => Math.round(c.x))).size === 1,
      "375px five cards share one column (vertical team column, #065 redesign)",
      `distinct x = ${new Set(cards.map((c) => Math.round(c.x))).size}`,
    );
    check(
      new Set(cards.map((c) => Math.round(c.y))).size === 5,
      "375px the five cards stack vertically (distinct Ys, front on top)",
      `distinct y = ${new Set(cards.map((c) => Math.round(c.y))).size}`,
    );
    check(
      Math.max(...cards.map((c) => c.right)) <= PHONE.width,
      "375px the team column fits the viewport",
      `right edge ${Math.max(...cards.map((c) => c.right)).toFixed(1)}`,
    );
    check(await noHorizontalOverflow(page), "375px battle screen has no horizontal overflow");
  }
  await ctx.close();
}

await browser.close();
disarm();
finish("probe-cards");
