// PRD #015 slice 1 — the one pretty unit card. Pins, against the LIVE app:
//  1. Structure: shop offer, line card, ladder card, and battle-board card all
//     share the SAME card skeleton (shape art, name, framed hp/pwr, chips) —
//     one component serving every context; line cards add the level badge +
//     fusion pips, the front card its marker.
//  2. Battle affordances survive: current/max hp on board cards, front tag.
//  3. 375px stays clean: no horizontal overflow in shop or battle, and the
//     5-unit board line still sits five abreast inside the viewport.
//  4. The inspector overlay still anchors to the new cards (popover pinned to
//     the card on a desk, bottom sheet at phone width).

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

// The shared skeleton, stripped of context extras (front-tag, run-lvl).
const core = (sig) =>
  sig
    .split(" | ")
    .map((part) => part.split(",").filter((c) => c !== "front-tag" && c !== "run-lvl").join(","))
    .join(" | ");

// ---------- shop / line / ladder share the one card ------------------------

for (const [viewport, tag] of [
  [PHONE, "375px"],
  [DESKTOP, "desktop"],
]) {
  const { ctx, page } = await openRun(browser, targetsRun(), viewport);
  const offer = await signature(page, '#run-shop-row [data-offer="0"]');
  const line = await signature(page, '#run-line [data-line="0"]');
  const ladder = await signature(page, ".lv-champ .unit");
  check(
    offer.startsWith("svg.shape,.uname,.unums .hp,.unums .pwr,.chips"),
    `${tag} offer card carries the full skeleton`,
    offer,
  );
  check(core(line) === core(offer), `${tag} line card = offer card skeleton`, `${line} vs ${offer}`);
  check(core(ladder) === core(offer), `${tag} ladder card = offer card skeleton`, `${ladder} vs ${offer}`);

  // ONE fixed size (#078): the card size IS the complexity budget, so every
  // surface renders the card at the SAME width — geometry is baked into .unit,
  // not set per-surface. This must FAIL against the old per-surface CSS where
  // the shop card was 7rem, the ladder card 5.6rem (≠), and the codex card
  // 8.5rem — different sizes on every screen.
  const offerW = (await box(page, '#run-shop-row [data-offer="0"]')).width;
  const lineW = (await box(page, '#run-line [data-line="0"]')).width;
  const ladderW = (await box(page, ".lv-champ .unit:first-child")).width;
  check(
    Math.abs(lineW - offerW) <= 1 && Math.abs(ladderW - offerW) <= 1,
    `${tag} shop / line / ladder cards share ONE fixed width (#078)`,
    `offer ${offerW.toFixed(1)}, line ${lineW.toFixed(1)}, ladder ${ladderW.toFixed(1)}`,
  );

  check(line.includes("front-tag") && line.includes("run-lvl"), `${tag} front line card adds marker + level badge`, line);
  check(
    await page.$eval('#run-line [data-line="0"] .run-lvl .run-pips', (el) => el.textContent !== ""),
    `${tag} fusion pips ride the level badge`,
  );
  if (viewport === PHONE) {
    check(await noHorizontalOverflow(page), `${tag} shop screen has no horizontal overflow`);
  }

  // Inspector anchors to the new card: popover pinned to it on a desk, bottom
  // sheet at phone width.
  await page.click('#run-line [data-line="0"] .uname');
  await page.waitForSelector("#inspect-overlay:not([hidden])");
  const overlay = await box(page, "#inspect-overlay");
  if (viewport === DESKTOP) {
    const card = await box(page, '#run-line [data-line="0"]');
    const gapBelow = overlay.y - (card.y + card.height);
    const gapAbove = card.y - (overlay.y + overlay.height);
    check(
      (gapBelow >= 0 && gapBelow <= 10) || (gapAbove >= 0 && gapAbove <= 10),
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

// ---------- the battle board wears the same card ----------------------------

for (const [viewport, tag] of [
  [PHONE, "375px"],
  [DESKTOP, "desktop"],
]) {
  const { ctx, page } = await openRun(browser, lineFullRun(), viewport);
  const offer = await signature(page, '#run-shop-row [data-offer="0"]');
  const offerW = (await box(page, '#run-shop-row [data-offer="0"]')).width;
  await page.click("#run-fight");
  await page.waitForSelector('#board .unit[data-unit]');
  const board = await signature(page, '#board .side[data-side="A"] .line .unit');
  check(core(board) === core(offer), `${tag} board card = offer card skeleton`, `${board} vs ${offer}`);
  // The board renders the SAME card (#078) but is the one surface that SCALES it
  // to fit its focal three-column replay stage (striker | beat-card lane |
  // striker, #065): a wrapper resize, not a different card. The budget invariant
  // that holds at BOTH widths: the board card is never LARGER than the one fixed
  // catalog size — it never exceeds the card budget, and at 375px shrinks
  // further (the documented `--side-col` phone override) so 5v5 fits the stage.
  const boardW = (await box(page, '#board .side[data-side="A"] .line .unit:first-child')).width;
  check(
    boardW <= offerW + 1,
    `${tag} board card never exceeds the one fixed card size (scaled to its focal stage, #078)`,
    `board ${boardW.toFixed(1)} ≤ fixed ${offerW.toFixed(1)}`,
  );
  check(
    await page.$eval('#board .side[data-side="A"] .line .unit .hp', (el) => /^\d+\/\d+$/.test(el.textContent)),
    `${tag} board card shows current/max hp`,
  );
  check(
    (await page.$('#board .side[data-side="A"] .line .unit:first-child .front-tag')) !== null,
    `${tag} board front card keeps its marker`,
  );
  if (viewport === PHONE) {
    // #065 redesign: each team is a VERTICAL column — the line stacks its cards
    // straight down (front on top), one card wide. So the five units share one
    // COLUMN (one x, distinct stacked Ys), not one row, and the column fits the
    // 375px stage with no overflow.
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
