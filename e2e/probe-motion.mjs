// Motion probes (#082 slice D) — the across-frame truths a STILL screenshot is
// blind to, made catchable. The acting-card battle shows WHO acts and WHO is
// hit by moving the ACTING / TARGET ribbons and the lit trace chip as the
// playhead advances; a still shot of one frame can't see a ribbon that lands on
// the wrong card during the transition, or two chips lit at once. These probes
// read that state directly from the live DOM step by step, so a regression
// reddens an assertion instead of slipping past a static capture.
//
//   • The ACTING ribbon marks exactly the acting unit — the side card with
//     `is-acting` must equal the centre acting card's `data-acting`, and there
//     is at most one of each at every step.
//   • The TARGET ribbon marks the struck unit (a different card from ACTING),
//     never the actor itself.
//   • A beat with no actor (a phase caption) wears NO ribbon — neither ACTING
//     nor TARGET — so the marks vanish cleanly between fights, not linger.
//   • Exactly one trace chip is lit (`is-cur`) at every step, and it tracks the
//     beat as the playhead crosses a boundary (the mark MOVES across frames).
//
// Run under `npm run e2e` (the probe-*.mjs glob).

import { armGuard, check, bigBattleRun, finish, launch, openRun, plainShopRun, DESKTOP } from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

/** Start the injected run's fight and land paused on event 0. */
async function intoBattle(page) {
  await page.click("#run-fight");
  await page.waitForSelector("#run-battle:not([hidden])");
  await page.waitForSelector("#board .bv-side");
  await page.waitForSelector("#step-play");
  if ((await page.locator("#step-play").textContent())?.trim() === "pause") {
    await page.click("#step-play");
  }
  await stepTo(page, 0);
}

async function stepTo(page, n) {
  await page.evaluate((target) => {
    const scrub = document.querySelector("#scrub");
    scrub.value = String(target);
    scrub.dispatchEvent(new Event("input", { bubbles: true }));
  }, n);
}

const maxStep = (page) => page.evaluate(() => Number(document.querySelector("#scrub").max));

/** The per-step ribbon + chip state read straight off the DOM. */
const frameState = (page) =>
  page.evaluate(() => {
    const acting = [...document.querySelectorAll(".unit-b.is-acting[data-unit]")].map((u) => u.getAttribute("data-unit"));
    const target = [...document.querySelectorAll(".unit-b.is-target[data-unit]")].map((u) => u.getAttribute("data-unit"));
    const card = document.querySelector(".acting-card");
    const phase = document.querySelector(".acting-phase");
    const curChips = [...document.querySelectorAll(".trace-strip .tr-chip.is-cur")].map((c) => c.getAttribute("data-id"));
    return {
      acting,
      target,
      cardActing: card ? card.getAttribute("data-acting") : null,
      isPhase: phase !== null,
      curChips,
    };
  });

// =====================================================================
// 1. ACTING / TARGET ribbons track the kernel truth as the playhead advances.
// =====================================================================
async function ribbonTruth(run, label, page) {
  const total = await maxStep(page);
  let actingMismatch = "";
  let doubleMark = "";
  let actingEqTarget = "";
  let phaseLeak = "";
  let sawActing = false;
  let sawTarget = false;
  let beatMoves = false;
  let prevCardActing = null;

  for (let s = 0; s <= total; s++) {
    await stepTo(page, s);
    const f = await frameState(page);

    if (f.acting.length > 1) doubleMark ||= `@${s}: ${f.acting.length} ACTING ribbons`;
    if (f.target.length > 1) doubleMark ||= `@${s}: ${f.target.length} TARGET ribbons`;
    if (f.acting.length === 1) sawActing = true;
    if (f.target.length === 1) sawTarget = true;
    if (f.acting.length === 1 && f.target.length === 1 && f.acting[0] === f.target[0]) {
      actingEqTarget ||= `@${s}: ${f.acting[0]} wears BOTH ACTING and TARGET`;
    }

    if (f.cardActing !== null) {
      // A Strike beat: the centre names the actor; the side ribbon must agree.
      if (f.acting.length !== 1 || f.acting[0] !== f.cardActing) {
        actingMismatch ||= `@${s}: card acting=${f.cardActing}, side ribbon=${JSON.stringify(f.acting)}`;
      }
    }
    if (f.isPhase) {
      // A beat with no actor: no ribbons at all.
      if (f.acting.length > 0 || f.target.length > 0) {
        phaseLeak ||= `@${s}: phase step still shows ribbons acting=${JSON.stringify(f.acting)} target=${JSON.stringify(f.target)}`;
      }
    }
    if (prevCardActing !== null && f.cardActing !== null && f.cardActing !== prevCardActing) beatMoves = true;
    prevCardActing = f.cardActing;
  }

  check(actingMismatch === "", `${label} the ACTING ribbon matches the centre card's actor at every step`, actingMismatch);
  check(doubleMark === "", `${label} at most one ACTING and one TARGET ribbon at every step`, doubleMark);
  check(actingEqTarget === "", `${label} the actor never also wears the TARGET ribbon`, actingEqTarget);
  check(phaseLeak === "", `${label} a phase step (no actor) shows no ribbons`, phaseLeak);
  check(sawActing, `${label} an ACTING ribbon appears during the battle`);
  check(sawTarget, `${label} a TARGET ribbon appears during the battle`);
  check(beatMoves, `${label} the acting card's actor MOVES across a beat boundary (across-frame)`);
}

// =====================================================================
// 2. Exactly one trace chip is lit, and it tracks the beat as the playhead moves.
// =====================================================================
async function chipTruth(page) {
  const total = await maxStep(page);
  let badCount = "";
  let chipMoves = false;
  let prev = null;
  for (let s = 0; s <= total; s++) {
    await stepTo(page, s);
    const f = await frameState(page);
    if (f.curChips.length !== 1) badCount ||= `@${s}: ${f.curChips.length} lit chips`;
    const cur = f.curChips[0] ?? null;
    if (prev !== null && cur !== null && cur !== prev) chipMoves = true;
    prev = cur;
  }
  check(badCount === "", `exactly one trace chip is lit at every step`, badCount);
  check(chipMoves, `the lit trace chip moves as the playhead crosses beats (across-frame)`);
}

// --- scenarios ---
{
  const { ctx, page } = await openRun(browser, plainShopRun(), DESKTOP);
  await intoBattle(page);
  await ribbonTruth(plainShopRun(), "1v5", page);
  await ctx.close();
}
{
  const { ctx, page } = await openRun(browser, bigBattleRun(), DESKTOP);
  await intoBattle(page);
  await ribbonTruth(bigBattleRun(), "5v3", page);
  await ctx.close();
}
{
  const { ctx, page } = await openRun(browser, plainShopRun(), DESKTOP);
  await intoBattle(page);
  await chipTruth(page);
  await ctx.close();
}

await browser.close();
disarm();
finish("probe-motion");
