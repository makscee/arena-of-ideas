// Slice-3 fix round, refutations 1–3 (+ reroll): at 375×667 with REAL strings,
// the fight button and ladder hold their Y through every strip/shop change —
// fuse notice (wraps to 2 lines), the full-line buy error (wraps), buying from
// a 3-offer wrapped shop, and reroll. Desktop spot-check for regressions.

import {
  DESKTOP,
  PHONE,
  armGuard,
  box,
  check,
  finish,
  fuseReadyRun,
  launch,
  lineFullRun,
  openRun,
  plainShopRun,
} from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

const fightY = async (page) => (await box(page, "#run-fight")).y;
const ladderY = async (page) => (await box(page, "#run-ladder")).y;
const stripH = async (page, sel) => (await box(page, sel)).height;

async function scenarios(viewport, tag) {
  // --- refutation 1: fusing buy, real notice, fight Y pinned -------------
  {
    const { ctx, page } = await openRun(browser, fuseReadyRun(), viewport);
    const y0 = await fightY(page);
    const h0 = await stripH(page, "#run-notice");
    const shopH0 = await stripH(page, "#run-shop-row");
    await page.click('[data-buy="0"]');
    await page.waitForFunction(() => document.querySelector("#run-notice").textContent.startsWith("⬆"));
    const notice = await page.locator("#run-notice").textContent();
    const y1 = await fightY(page);
    const h1 = await stripH(page, "#run-notice");
    const shopH1 = await stripH(page, "#run-shop-row");
    check(notice.includes("fused 3 copies into level 2"), `${tag} fuse notice is the real string`, JSON.stringify(notice));
    check(y1 === y0, `${tag} fight Y stable across fuse`, `${y0} → ${y1}`);
    check(h1 === h0, `${tag} notice strip height fixed across fuse`, `${h0} → ${h1}`);
    check(shopH1 === shopH0, `${tag} shop row height held by the rolled-count reserve`, `${shopH0} → ${shopH1}`);
    await ctx.close();
  }

  // --- refutation 2: real full-line error, fight + ladder pinned ---------
  {
    const { ctx, page } = await openRun(browser, lineFullRun(), viewport);
    const y0 = await fightY(page);
    const l0 = await ladderY(page);
    const e0 = await stripH(page, "#run-error");
    await page.click('[data-buy="0"]');
    await page.waitForFunction(() => document.querySelector("#run-error").textContent !== "");
    const err = await page.locator("#run-error").textContent();
    check(
      err === 'invalid decision "buy": the line is full (5) and there is no Bulwark to stack onto',
      `${tag} error is the real full-line string`,
      JSON.stringify(err),
    );
    const y1 = await fightY(page);
    const l1 = await ladderY(page);
    const e1 = await stripH(page, "#run-error");
    check(y1 === y0, `${tag} fight Y stable across error`, `${y0} → ${y1}`);
    check(l1 === l0, `${tag} ladder Y stable across error`, `${l0} → ${l1}`);
    check(e1 === e0, `${tag} error strip height fixed`, `${e0} → ${e1}`);
    await ctx.close();
  }

  // --- refutation 3 + reroll: plain buy from the wrapped 3-offer shop ----
  {
    const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
    const y0 = await fightY(page);
    const shopH0 = await stripH(page, "#run-shop-row");
    await page.click('[data-buy="0"]');
    await page.waitForFunction(() => document.querySelectorAll("[data-offer]").length === 2);
    const y1 = await fightY(page);
    const shopH1 = await stripH(page, "#run-shop-row");
    check(y1 === y0, `${tag} fight Y stable across buy (3 offers → 2)`, `${y0} → ${y1}`);
    check(shopH1 === shopH0, `${tag} shop row height stable across buy`, `${shopH0} → ${shopH1}`);
    await page.click("#run-reroll");
    await page.waitForFunction(() => document.querySelectorAll("[data-offer]").length === 3);
    const y2 = await fightY(page);
    const shopH2 = await stripH(page, "#run-shop-row");
    check(y2 === y0, `${tag} fight Y stable across reroll`, `${y0} → ${y2}`);
    check(shopH2 === shopH0, `${tag} shop row height stable across reroll`, `${shopH0} → ${shopH2}`);
    await ctx.close();
  }
}

await scenarios(PHONE, "375px");
await scenarios(DESKTOP, "desktop");

await browser.close();
disarm();
finish("probe-stability");
