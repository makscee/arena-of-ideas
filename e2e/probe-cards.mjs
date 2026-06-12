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
  await page.click("#run-fight");
  await page.waitForSelector('#board .unit[data-unit]');
  const board = await signature(page, '#board .side[data-side="A"] .line .unit');
  check(core(board) === core(offer), `${tag} board card = offer card skeleton`, `${board} vs ${offer}`);
  check(
    await page.$eval('#board .side[data-side="A"] .line .unit .hp', (el) => /^\d+\/\d+$/.test(el.textContent)),
    `${tag} board card shows current/max hp`,
  );
  check(
    (await page.$('#board .side[data-side="A"] .line .unit:first-child .front-tag')) !== null,
    `${tag} board front card keeps its marker`,
  );
  if (viewport === PHONE) {
    // The 5-unit line sits five abreast inside 375px — no wrap, no overflow.
    const cards = await page.$$eval('#board .side[data-side="A"] .line .unit', (els) =>
      els.map((el) => {
        const r = el.getBoundingClientRect();
        return { y: r.y, right: r.right };
      }),
    );
    check(cards.length === 5, "375px board line renders all five cards", `${cards.length}`);
    check(new Set(cards.map((c) => Math.round(c.y))).size === 1, "375px five cards share one row");
    check(
      Math.max(...cards.map((c) => c.right)) <= PHONE.width,
      "375px five-card line fits the viewport",
      `right edge ${Math.max(...cards.map((c) => c.right)).toFixed(1)}`,
    );
    check(await noHorizontalOverflow(page), "375px battle screen has no horizontal overflow");
  }
  await ctx.close();
}

await browser.close();
disarm();
finish("probe-cards");
