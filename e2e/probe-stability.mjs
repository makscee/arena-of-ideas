// Slice-3 fix round, refutations 1–3 (+ reroll): at 375×667 with REAL strings,
// the fight button and ladder hold their Y through every strip/shop change —
// fuse notice (wraps to 2 lines), the full-line buy error (wraps), buying from
// a 3-offer wrapped shop, and reroll. Desktop spot-check for regressions.
//
// Slice-3 close, finding 1: the LINE row reserves every card reachable this
// phase — buying a new name (2→3, 4→5 distinct) wraps a line row INSIDE the
// reserve, so fight/ladder hold their Y under the tap; the reserve itself
// recomputes only at a phase render, where scroll resets.

import { TEAM_SIZE, UNIT_COST, REROLL_COST, incomeForRound } from "../src/index.js";
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
  lineGrowthRun,
  openRun,
  plainShopRun,
  poorGoldRun,
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

// --- slice-3 close, finding 1: line growth never moves fight/ladder --------
// Three buys of new names walk the team 2→5 distinct units: at 375px that
// wraps line rows at 2→3 and 4→5. The reachable-count reserve (team + what
// gold can buy) is standing from the phase render, so every wrap lands
// inside it — fight, ladder, and the reserve itself hold to the pixel.
const lineMinH = (page) => page.evaluate(() => document.querySelector("#run-line").style.minHeight);

async function lineGrowth(viewport, tag) {
  const { ctx, page } = await openRun(browser, lineGrowthRun(), viewport);
  const y0 = await fightY(page);
  const l0 = await ladderY(page);
  const m0 = await lineMinH(page);
  check(m0 !== "", `${tag} line reserve standing at phase render`, `minH ${m0}, fight Y ${y0}`);
  for (let team = 2; team < TEAM_SIZE; team++) {
    await page.click('[data-buy="0"]');
    await page.waitForFunction((n) => document.querySelectorAll("[data-line]").length === n, team + 1);
    const y = await fightY(page);
    const l = await ladderY(page);
    const m = await lineMinH(page);
    check(y === y0, `${tag} fight Y stable across buy ${team}→${team + 1}`, `${y0} → ${y}`);
    check(l === l0, `${tag} ladder Y stable across buy ${team}→${team + 1}`, `${l0} → ${l}`);
    check(m === m0, `${tag} line reserve held across buy ${team}→${team + 1}`, `${m0} → ${m}`);
  }
  await ctx.close();
}

await lineGrowth(PHONE, "375px");
await lineGrowth(DESKTOP, "desktop");

// --- slice-3 close: the reserve recomputes ONLY at a phase render ----------
// Poor gold (one row reachable): mid-phase actions hold the small reserve —
// no burning rows gold cannot fill — and the next round's income grows it at
// the continue → shop render, where scroll has just reset to the top.
{
  const { ctx, page } = await openRun(browser, poorGoldRun(), PHONE);
  const y0 = await fightY(page);
  const m0 = await lineMinH(page);
  const oneRow = (await box(page, '[data-line="0"]')).height;
  check(parseFloat(m0) < oneRow * 2, "375px poor gold reserves only the reachable row", `minH ${m0}, card ${oneRow}px`);
  await page.click("#run-reroll"); // spends gold; the captured count holds
  await page.waitForFunction(() => document.querySelector("#run-head .run-gold").textContent !== "2 gold");
  check((await lineMinH(page)) === m0, "375px line reserve held across reroll", `${m0} → ${await lineMinH(page)}`);
  check((await fightY(page)) === y0, "375px fight Y stable across reroll", `${y0} → ${await fightY(page)}`);
  await page.click("#run-fight");
  await page.waitForSelector("#run-skip:not([hidden])");
  await page.click("#run-skip");
  await page.waitForSelector("#run-continue:not([hidden])");
  await page.click("#run-continue");
  await page.waitForSelector("#run-shop:not([hidden])");
  const m2 = await lineMinH(page);
  check(await page.evaluate(() => window.scrollY) === 0, "375px continue → shop reset scroll to the top");
  // Round 2's income re-arms gold: the reachable count (and so the reserve)
  // grows exactly here, on the phase render — derived, never hardcoded.
  const gold2 = UNIT_COST - 1 - REROLL_COST + incomeForRound(2);
  const reachable = Math.min(TEAM_SIZE, 2 + Math.floor(gold2 / UNIT_COST));
  if (reachable > 2) {
    check(parseFloat(m2) > parseFloat(m0), "375px reserve recomputed at the phase render", `${m0} → ${m2}`);
  }
  await ctx.close();
}

await browser.close();
disarm();
finish("probe-stability");
