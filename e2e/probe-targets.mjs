// Slice-3 fix round, refutation 4 + residuals a/e, at 375×667: hit-area
// ownership is exclusive — reorder arrows and adjacent chips split their gap
// at the midpoint (no overlap, no dead zone, no cross-card stealing), a click
// 2px inside ◂'s effective right edge fires ◂ (even with ▸ disabled), the
// inspector ✕ is a real ≥44px box, and the buy extension stops at the card
// body so a tap just above buy inspects instead of buying.

import { PHONE, armGuard, box, check, finish, launch, openRun, targetsRun } from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

/** Owner of (x, y): which move-button/chip/card elementFromPoint resolves to. */
async function ownerAt(page, x, y) {
  return page.evaluate(
    ([px, py]) => {
      const el = document.elementFromPoint(px, py);
      if (el === null) return "none";
      const move = el.closest("[data-move]");
      if (move) return `move:${move.getAttribute("data-move")}`;
      const chip = el.closest("[data-status]");
      if (chip) return `chip:${chip.getAttribute("data-status")}`;
      const buy = el.closest("[data-buy]");
      if (buy) return `buy:${buy.getAttribute("data-buy")}`;
      const card = el.closest("[data-line],[data-offer]");
      if (card) return `card:${card.getAttribute("data-line") ?? "o" + card.getAttribute("data-offer")}`;
      return el.tagName.toLowerCase();
    },
    [x, y],
  );
}

/** Sweep the ◂▸ pair of line card `i`: exclusive midpoint ownership.
 * elementFromPoint sees the visual viewport only — scroll the pair in first.
 * A disabled arrow is pointer-events:none (it owns no taps): its side must
 * fall to the live arrow (near the seam) or the card (= inspect), never die. */
async function sweepArrows(page, i) {
  await page.locator(`[data-move="${i}:-1"]`).scrollIntoViewIfNeeded();
  const back = await box(page, `[data-move="${i}:-1"]`); // ◂
  const fwd = await box(page, `[data-move="${i}:1"]`); // ▸
  const disabled = {
    [`move:${i}:-1`]: await page.locator(`[data-move="${i}:-1"]`).isDisabled(),
    [`move:${i}:1`]: await page.locator(`[data-move="${i}:1"]`).isDisabled(),
  };
  const y = back.y + back.height / 2;
  const gapMid = (back.x + back.width + fwd.x) / 2;
  // Within ±1.5px of the geometric midpoint the 1px snap guard may hand the
  // point to either arrow (hit rects pixel-snap) — but it must be ONE of the
  // two, never dead, never anything else. Outside the band: strict midpoint.
  const TOL = 1.5;
  let bad = 0;
  let deadZone = 0;
  for (let x = Math.ceil(back.x); x <= Math.floor(fwd.x + fwd.width); x += 1) {
    const owner = await ownerAt(page, x, y);
    const want = x < gapMid ? `move:${i}:-1` : `move:${i}:1`;
    const live = Object.keys(disabled).filter((k) => !disabled[k]);
    const ok = disabled[want]
      ? live.includes(owner) || owner.startsWith("card:") // pe:none side falls through, never dead
      : owner === want || (Math.abs(x - gapMid) <= TOL && live.includes(owner));
    if (!ok) {
      if (owner.startsWith("move:")) bad += 1;
      else deadZone += 1;
      if (bad + deadZone <= 3) console.log(`  x=${x} (mid=${gapMid.toFixed(1)}): got ${owner}, want ${want}`);
    }
  }
  check(bad === 0, `card ${i} arrows: no wrong-owner point across the pair`, `${bad} stolen`);
  check(deadZone === 0, `card ${i} arrows: no dead zone across the pair`, `${deadZone} dead`);
  return { back, fwd, y, gapMid };
}

// One context per click scenario — reorders mutate the persisted run.
async function fresh() {
  return openRun(browser, targetsRun(), PHONE);
}

const unameAt = (page, i) => page.locator(`[data-line="${i}"] .uname`).textContent();

// --- arrows: sweep + the 2px-inside-◂ click (▸ enabled) -------------------
{
  const { ctx, page } = await fresh();
  const { gapMid, y } = await sweepArrows(page, 1); // middle card: both enabled
  check((await unameAt(page, 1)) === "Brawler", "middle card is Brawler before the move");
  await page.mouse.click(gapMid - 2, y); // 2px inside ◂'s effective right edge
  await page.waitForFunction(() => document.querySelector('[data-line="0"] .uname').textContent === "Brawler");
  check(true, "click 2px inside ◂'s effective right edge fires ◂ (Brawler moved to front)");
  await ctx.close();
}

// --- arrows with ▸ disabled (last card): ◂'s right half still fires ◂ ------
{
  const { ctx, page } = await fresh();
  const { gapMid, y } = await sweepArrows(page, 2); // last card: ▸ disabled
  check((await unameAt(page, 2)) === "Squire", "last card is Squire before the move");
  await page.mouse.click(gapMid - 2, y);
  await page.waitForFunction(() => document.querySelector('[data-line="1"] .uname').textContent === "Squire");
  check(true, "with ▸ disabled, 2px inside ◂'s effective edge still fires ◂ (no dead tap)");
  await ctx.close();
}

// --- arrows stay inside their own card (no cross-card stealing) ------------
{
  const { ctx, page } = await fresh();
  await page.locator('[data-line="1"]').scrollIntoViewIfNeeded();
  const card1 = await box(page, '[data-line="1"]');
  const card0 = await box(page, '[data-line="0"]');
  const back = await box(page, '[data-move="1:-1"]');
  const y = back.y + back.height / 2;
  // Just left of card 1's left edge (inside card 0 / the row gap): never card 1's ◂.
  for (const x of [card1.x - 1, card0.x + card0.width - 1]) {
    const owner = await ownerAt(page, x, y);
    check(!owner.startsWith("move:1"), `point left of card 1 (x=${Math.round(x)}) not owned by its arrows`, owner);
  }
  await ctx.close();
}

// --- adjacent chips: exclusive midpoint ownership, no dead zone ------------
{
  const { ctx, page } = await fresh();
  const chips = ['[data-line="0"] [data-status="Poison"]', '[data-line="0"] [data-status="Shield"]', '[data-line="0"] [data-status="Vitality"]'];
  await page.locator(chips[0]).scrollIntoViewIfNeeded();
  const boxes = [];
  for (const sel of chips) boxes.push(await box(page, sel));
  const names = ["Poison", "Shield", "Vitality"];
  const y = boxes[0].y + boxes[0].height / 2;
  const mids = [
    (boxes[0].x + boxes[0].width + boxes[1].x) / 2,
    (boxes[1].x + boxes[1].width + boxes[2].x) / 2,
  ];
  const TOL = 1.5; // the 1px snap guard: the seam pixel may go to either neighbour
  let bad = 0;
  let dead = 0;
  for (let x = Math.ceil(boxes[0].x); x <= Math.floor(boxes[2].x + boxes[2].width); x += 1) {
    // Expected owner: the chip whose visual-plus-half-gap region holds x.
    let want = names[2];
    if (x < mids[0]) want = names[0];
    else if (x < mids[1]) want = names[1];
    const owner = await ownerAt(page, x, y);
    const nearSeam = mids.some((m) => Math.abs(x - m) <= TOL);
    const ok = owner === `chip:${want}` || (nearSeam && owner.startsWith("chip:"));
    if (!ok) {
      if (owner.startsWith("chip:")) bad += 1;
      else dead += 1;
      if (bad + dead <= 3) console.log(`  chip sweep x=${x}: got ${owner}, want chip:${want}`);
    }
  }
  check(bad === 0, "adjacent chips: no point owned by the wrong chip", `${bad} stolen`);
  check(dead === 0, "adjacent chips: no dead zone across the strip", `${dead} dead`);
  // The exclusive-width regression Cass measured (≈30px of a 44px box): the
  // point 1px inside each chip's right visual edge belongs to that chip.
  for (let i = 0; i < 3; i += 1) {
    const owner = await ownerAt(page, boxes[i].x + boxes[i].width - 1, y);
    check(owner === `chip:${names[i]}`, `1px inside ${names[i]}'s right edge is ${names[i]}'s`, owner);
  }
  await ctx.close();
}

// --- residual a: inspector ✕ is a real ≥44px target ------------------------
{
  const { ctx, page } = await fresh();
  await page.click('[data-line="0"] .uname'); // open the inspector on the card
  await page.waitForSelector("#inspect-overlay:not([hidden])");
  const close = await box(page, "#ins-close");
  check(close.width >= 44 && close.height >= 44, "#ins-close ≥44px both axes", `${close.width}×${close.height}`);
  await ctx.close();
}

// --- residual e: a tap just above buy inspects, never buys -----------------
{
  const { ctx, page } = await fresh();
  await page.locator('[data-buy="0"]').scrollIntoViewIfNeeded();
  const buy = await box(page, '[data-buy="0"]');
  const cx = buy.x + buy.width / 2;
  const above = await ownerAt(page, cx, buy.y - 4); // past the 2.4px half-gap extension
  check(above.startsWith("card:") || above.startsWith("chip:"), "point 4px above buy belongs to the card, not buy", above);
  await page.mouse.click(cx, buy.y - 4);
  await page.waitForSelector("#inspect-overlay:not([hidden])");
  const offers = await page.locator("[data-offer]").count();
  check(offers === 3, "the tap above buy bought nothing (3 offers remain)", `${offers} offers`);
  check(true, "the tap above buy opened the inspector");
  await ctx.close();
}

await browser.close();
disarm();
finish("probe-targets");
