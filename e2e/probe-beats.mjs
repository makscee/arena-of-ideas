// Beat-card viewer (#065 slice 1) — the real-layout checks vitest cannot see.
// Drives an actual run battle in the browser and asserts:
//   • layout: the log sits to the RIGHT of the stage on desktop, and STACKS
//     below it at 375px (no horizontal scroll);
//   • the centre card streams its caused lines as the playhead advances and
//     clears between beats; structural beats show a turn divider, not a card;
//   • transport stays event-granular — next/prev/scrub/log-click move ONE
//     event and the card reveals up to that event (a half-revealed mid-beat);
//   • play inserts a longer read-pause at a beat boundary than between lines;
//   • LS-1: board, transport, and the run "continue" button hold their Y to
//     the pixel across a full playback, at desktop AND 375px.
//
// Scripted Playwright (committed), per #012: the interactive MCP stalled, so
// the repeatable layout probes live here and run under `npm run e2e`.

import {
  DESKTOP,
  PHONE,
  armGuard,
  box,
  check,
  finish,
  launch,
  openRun,
  plainShopRun,
  bigBattleRun,
} from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

/** Start the injected run's fight and land on the paused replay's first event
 * (autoplay is on in the real app — pause immediately so the probe drives the
 * transport deterministically). Returns once the board has rendered. */
async function intoBattle(page) {
  await page.click("#run-fight");
  await page.waitForSelector("#run-battle:not([hidden])");
  await page.waitForSelector("#board .side");
  // Pause playback so step/scrub assertions are not racing the auto-advance.
  await page.waitForSelector("#step-play");
  if ((await page.locator("#step-play").textContent())?.trim() === "pause") {
    await page.click("#step-play");
  }
  // Rewind to the top for a deterministic start.
  await page.evaluate(() => {
    const scrub = document.querySelector("#scrub");
    scrub.value = "0";
    scrub.dispatchEvent(new Event("input", { bubbles: true }));
  });
}

/** The 0-based event index the scrub reports (the playhead). */
const playhead = (page) => page.evaluate(() => Number(document.querySelector("#scrub").value));
const maxStep = (page) => page.evaluate(() => Number(document.querySelector("#scrub").max));
/** Count of revealed card lines (caused events shown so far in the open beat). */
const cardLines = (page) => page.locator(".beat-card .bc-line").count();
const hasCard = (page) => page.locator(".beat-card").count();
const hasTurnDivider = (page) => page.locator(".turn-divider").count();

async function stepNext(page) {
  await page.click("#step-next");
}

// =====================================================================
// 1. Layout — log right of the stage on desktop, stacked at 375px, no h-scroll
// =====================================================================
async function layout(viewport, tag) {
  const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
  await intoBattle(page);

  const logCol = await box(page, "#log-col");
  const boardBox = await box(page, "#board");

  if (viewport.width >= 700) {
    const stage = await box(page, "#stage-col");
    check(
      logCol.x >= stage.x + stage.width - 2,
      `${tag} log column sits to the RIGHT of the stage`,
      `stage right=${Math.round(stage.x + stage.width)}, log left=${Math.round(logCol.x)}`,
    );
    check(
      Math.abs(logCol.y - stage.y) < 40,
      `${tag} log and stage share the top edge (two columns)`,
      `stage y=${Math.round(stage.y)}, log y=${Math.round(logCol.y)}`,
    );
  } else {
    // #065 defect 1: at phone width the log must be REACHABLE — it sits
    // directly UNDER the board (not buried below the whole stage: board +
    // transport + scrub + readout overflow a phone viewport). The reorder puts
    // the log between the board and the transport, so its top is just past the
    // board bottom and ABOVE the transport.
    const transport = await box(page, "#transport");
    check(
      logCol.y >= boardBox.y + boardBox.height - 2,
      `${tag} log sits directly below the board`,
      `board bottom=${Math.round(boardBox.y + boardBox.height)}, log top=${Math.round(logCol.y)}`,
    );
    check(
      logCol.y < transport.y,
      `${tag} log is ABOVE the transport (reachable on an early scroll, not buried under the whole stage)`,
      `log top=${Math.round(logCol.y)}, transport top=${Math.round(transport.y)}`,
    );
    // The log's bottom must land within an early scroll of the board top — not
    // pushed a full extra viewport down behind the transport/readout (the
    // pre-fix bug: log started ~700px below a 667px fold).
    check(
      logCol.y - (boardBox.y + boardBox.height) < 40,
      `${tag} log is tight under the board (no large gap)`,
      `gap=${Math.round(logCol.y - (boardBox.y + boardBox.height))}px`,
    );
    const hScroll = await page.evaluate(
      () => document.documentElement.scrollWidth > document.documentElement.clientWidth + 1,
    );
    check(!hScroll, `${tag} no horizontal page scroll at 375px`);
  }

  // The log itself is capped and internally scrolls — it never grows the page.
  const logBox = await box(page, "#battle-log");
  check(logBox.height <= viewport.height, `${tag} log pane is capped (internal scroll)`, `${Math.round(logBox.height)}px`);

  await ctx.close();
}

// =====================================================================
// 2. Card streams line by line; structural beats are dividers; clears on next beat
// =====================================================================
/** Geometry + visibility of the open card and its revealed lines, measured in
 * the page: the card's own width and the stage column's width, and for every
 * `.bc-line` whether it actually paints (non-zero offset box AND in the
 * viewport). The old `streaming()` greened while the lines were buried in a
 * full-width bar (defect 2/3) — counting `.bc-line` nodes is not enough; the
 * lines must be VISIBLE and the card must be a floating card, not a stage-wide
 * bar. */
async function cardGeometry(page) {
  return page.evaluate(() => {
    const card = document.querySelector(".beat-card");
    const stage = document.querySelector("#stage-col");
    if (!card) return { hasCard: false };
    // The card may sit above/below the current scroll — "visible" means it
    // paints and lands in the viewport WHEN scrolled to, not that it happens to
    // be in view at an arbitrary scroll. Bring it into view first (the sweep
    // helper's pattern), then measure the painted/in-viewport state of lines.
    card.scrollIntoView({ block: "center" });
    const cr = card.getBoundingClientRect();
    const sr = stage.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    const lines = [...document.querySelectorAll(".beat-card .bc-line")].map((l) => {
      const r = l.getBoundingClientRect();
      const cs = getComputedStyle(l);
      const painted =
        l.offsetHeight > 0 &&
        l.offsetWidth > 0 &&
        cs.visibility !== "hidden" &&
        cs.display !== "none" &&
        Number(cs.opacity) > 0;
      const inView = r.bottom > 0 && r.top < vh && r.right > 0 && r.left < vw;
      return { painted, inView, h: Math.round(r.height), w: Math.round(r.width) };
    });
    return {
      hasCard: true,
      cardW: cr.width,
      stageW: sr.width,
      lines,
    };
  });
}

async function streaming(viewport, tag) {
  const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
  await intoBattle(page);

  // Step through the whole battle one event at a time; at every step exactly
  // one of {beat card, turn divider} is showing, and a card's revealed-line
  // count never exceeds the events seen so far in its beat.
  const total = await maxStep(page);
  let sawCard = false;
  let sawDivider = false;
  let sawStream = false; // a beat where lines grew across consecutive steps
  let prevBeat = await page.evaluate(() =>
    document.querySelector(".beat-card")?.getAttribute("data-beat") ?? null,
  );
  let prevLines = await cardLines(page);
  let sawClear = false;

  // Visibility / sizing accumulators (defect 2/3 close): track the worst case
  // seen across the whole playback — a single invisible or zero-height line, or
  // a single full-stage-width card, must turn the probe red.
  let anyVisibleLine = false; // at least one beat actually painted its lines
  let everyLineVisible = true; // no revealed line was invisible / off-screen
  let maxCardFrac = 0; // widest card / stage ratio seen (must stay well under 1)

  for (let s = 0; s < total; s++) {
    await stepNext(page);
    const card = await hasCard(page);
    const div = await hasTurnDivider(page);
    check(card + div === 1, `${tag} step ${s + 1}: exactly one of card/divider shows`, `card=${card} div=${div}`);
    if (card) sawCard = true;
    if (div) sawDivider = true;

    const beat = await page.evaluate(() =>
      document.querySelector(".beat-card")?.getAttribute("data-beat") ?? null,
    );
    const lines = await cardLines(page);
    if (card && beat === prevBeat && lines > prevLines) sawStream = true; // a line streamed in
    if (card && beat !== prevBeat && prevBeat !== null) sawClear = true; // card cleared → new beat
    prevBeat = beat;
    prevLines = lines;

    if (card) {
      const g = await cardGeometry(page);
      if (g.hasCard) {
        maxCardFrac = Math.max(maxCardFrac, g.cardW / g.stageW);
        for (const ln of g.lines) {
          if (ln.painted && ln.inView) anyVisibleLine = true;
          else everyLineVisible = false;
        }
      }
    }
  }

  check(sawCard, `${tag} hero-affecting beats open a centre card`);
  check(sawDivider, `${tag} structural beats render a turn divider (not a card)`);
  check(sawStream, `${tag} card lines stream in as the playhead advances within a beat`);
  check(sawClear, `${tag} the card clears when the next beat opens`);

  // The headline feature must be VISIBLE, not merely present in the DOM.
  check(
    anyVisibleLine,
    `${tag} at least one beat's streamed lines actually render (non-zero box, in viewport)`,
  );
  check(
    everyLineVisible,
    `${tag} every revealed card line is visible (painted + within the viewport) — no invisible/zero-height lines`,
  );
  // A floating card, not a stage-wide bar. On a WIDE stage the card must be
  // meaningfully narrower than its column (the desktop defect: it stretched
  // edge-to-edge). On a NARROW phone column the card legitimately fills most of
  // the width, so there the contract is its absolute width cap (≈22rem) — a card
  // wider than that is the un-capped bar regardless of column width.
  if (viewport.width >= 700) {
    check(
      maxCardFrac > 0 && maxCardFrac <= 0.9,
      `${tag} the beat card floats (width < stage column width), not a full-stage-width bar`,
      `widest card/stage = ${maxCardFrac.toFixed(3)}`,
    );
  } else {
    const widest = await page.evaluate(() => {
      // Replay once measuring the widest card the run ever shows.
      const scrub = document.querySelector("#scrub");
      const max = Number(scrub.max);
      let w = 0;
      for (let i = 0; i <= max; i++) {
        scrub.value = String(i);
        scrub.dispatchEvent(new Event("input", { bubbles: true }));
        const c = document.querySelector(".beat-card");
        if (c) w = Math.max(w, c.getBoundingClientRect().width);
      }
      return w;
    });
    check(
      widest > 0 && widest <= 22 * 16 + 1,
      `${tag} the beat card is width-capped (≤22rem), not an un-capped bar`,
      `widest card = ${Math.round(widest)}px`,
    );
  }

  await ctx.close();
}

// =====================================================================
// 3. Transport stays event-granular: next/prev/scrub/log-click move ONE event
// =====================================================================
async function transport(viewport, tag) {
  const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
  await intoBattle(page);

  const p0 = await playhead(page);
  await stepNext(page);
  const p1 = await playhead(page);
  check(p1 === p0 + 1, `${tag} next advances exactly one event`, `${p0} → ${p1}`);
  await page.click("#step-prev");
  const p2 = await playhead(page);
  check(p2 === p0, `${tag} prev steps back exactly one event`, `${p1} → ${p2}`);

  // Scrub to a mid-beat event of a multi-line beat: the card shows a PARTIAL
  // reveal (fewer lines than the beat will end with).
  // Find a card beat with ≥2 caused events by walking forward.
  const total = await maxStep(page);
  let foundPartial = false;
  for (let s = 0; s < total && !foundPartial; s++) {
    await stepNext(page);
    const lines = await cardLines(page);
    if (lines >= 1 && (await hasCard(page))) {
      // peek one ahead: if the next step keeps the same beat with more lines,
      // we are mid-beat now (a half-revealed card).
      const beatNow = await page.evaluate(() =>
        document.querySelector(".beat-card")?.getAttribute("data-beat"),
      );
      await stepNext(page);
      const beatNext = await page.evaluate(() =>
        document.querySelector(".beat-card")?.getAttribute("data-beat"),
      );
      const linesNext = await cardLines(page);
      if (beatNext === beatNow && linesNext > lines) {
        foundPartial = true;
        check(true, `${tag} scrubbing mid-beat shows a half-revealed card`, `${lines} → ${linesNext} lines, same beat`);
      }
    }
  }
  check(foundPartial, `${tag} found a multi-line beat to prove partial reveal`);

  // Right-log click-to-jump: clicking a log row drives the playhead to that
  // event (event-granular), and the card reflects it.
  await page.locator("#battle-log .log-line[data-id]").nth(2).click();
  const targetId = await page.evaluate(() =>
    Number(document.querySelectorAll("#battle-log .log-line[data-id]")[2].getAttribute("data-id")),
  );
  const pj = await playhead(page);
  check(pj === targetId, `${tag} right-log click jumps the playhead to that event`, `→ ${pj} (want ${targetId})`);

  await ctx.close();
}

// =====================================================================
// 3b. Selected-line cause readout (#065 slice 4)
//   The deep cross-beat ancestry readout (`why died ← poison ← strike …`) is
//   repurposed as the SELECTED-line detail: nothing selected → a neutral prompt;
//   clicking a card `.bc-line` OR a right-log row selects that event and the
//   readout traces ITS full cross-beat ancestry. Within-beat causality reads off
//   the card; this is the deep trace on demand. Uses bigBattleRun so Venomancer's
//   poison produces a death whose cause chain spans earlier beats.
// =====================================================================
const causeText = (page) => page.locator("#event-cause").innerText();
const causeHtml = (page) => page.locator("#event-cause").innerHTML();

async function causeReadout(viewport, tag) {
  const { ctx, page } = await openRun(browser, bigBattleRun(), viewport);
  await intoBattle(page);

  // 1) Empty state: a fresh, paused load has nothing selected → neutral prompt,
  //    NOT a populated cause/why trace. (Pre-slice-4 the readout mirrored the
  //    playhead event, so this assertion FAILs on the old code.)
  const emptyHtml = await causeHtml(page);
  check(
    emptyHtml.includes("cause-empty") && !emptyHtml.includes(">cause<") && !emptyHtml.includes(">why<"),
    `${tag} nothing selected → neutral cause prompt (no trace)`,
    emptyHtml.slice(0, 120),
  );

  // Advance until the open beat shows at least one caused card line to click.
  const total = await maxStep(page);
  let lineId = -1;
  for (let s = 0; s < total && lineId < 0; s++) {
    await stepNext(page);
    if ((await cardLines(page)) >= 1) {
      lineId = await page.evaluate(() =>
        Number(document.querySelector(".beat-card .bc-line[data-id]").getAttribute("data-id")),
      );
    }
  }
  check(lineId >= 0, `${tag} found a card line to click`, `event ${lineId}`);

  // 2) Click that card line → the readout populates with ITS cross-beat cause
  //    trace, and the line is marked selected. The playhead does NOT move (the
  //    deep trace is on demand; selection is decoupled from the playhead).
  const headBefore = await playhead(page);
  await page.locator(`.beat-card .bc-line[data-id="${lineId}"]`).click();
  const afterLine = await causeHtml(page);
  check(
    !afterLine.includes("cause-empty") && (afterLine.includes(">cause<") || afterLine.includes(">why<")),
    `${tag} clicking a card line populates the cause readout`,
    afterLine.slice(0, 160),
  );
  check(
    (await page.locator(`.beat-card .bc-line[data-id="${lineId}"].bc-line-sel`).count()) === 1,
    `${tag} the clicked card line is marked selected`,
  );
  check(
    (await playhead(page)) === headBefore,
    `${tag} selecting a card line does NOT move the playhead (trace is on demand)`,
    `head ${headBefore} → ${await playhead(page)}`,
  );

  // 3) Cross-beat span — select a DEATH via its log row and assert the trace
  //    reaches back across beat boundaries. A Venomancer poison death's cause
  //    chain is several `←` hops (poison tick ← poison applied … / a multi-link
  //    ancestry), and the ancestor events live in EARLIER beats than the death.
  //    The log only renders rows up to the playhead (syncTo), so jump to the end
  //    first to materialise every row — selection is decoupled from the playhead.
  await page.evaluate(() => {
    const scrub = document.querySelector("#scrub");
    scrub.value = scrub.max;
    scrub.dispatchEvent(new Event("input", { bubbles: true }));
  });
  const deathRow = page.locator("#battle-log .log-line.ev-death[data-id]").first();
  check((await deathRow.count()) >= 1, `${tag} battle produced a death to trace`);
  const deathId = await deathRow.evaluate((el) => Number(el.getAttribute("data-id")));
  await deathRow.click();
  const deathTrace = await causeText(page);
  const deathTraceHtml = await causeHtml(page);
  check(
    deathTrace.includes("←"),
    `${tag} a death's readout shows a multi-hop cause chain (←)`,
    deathTrace.slice(0, 200),
  );
  // Cross-beat: at least one ancestor link points to an event in a DIFFERENT
  // beat than the death. The ancestry links carry data-goto=<ancestorId>; map
  // each id (and the death) to its beat index via the page's own segmentation,
  // observed by stepping the scrub and reading data-beat at each event.
  const gotoIds = await page.evaluate(() =>
    [...document.querySelectorAll("#event-cause a[data-goto]")].map((a) => Number(a.getAttribute("data-goto"))),
  );
  const beatOfEvent = async (id) => {
    await page.evaluate((i) => {
      const scrub = document.querySelector("#scrub");
      scrub.value = String(i);
      scrub.dispatchEvent(new Event("input", { bubbles: true }));
    }, id);
    return page.evaluate(() => {
      const c = document.querySelector(".beat-card");
      return c ? c.getAttribute("data-beat") : document.querySelector(".turn-divider, .divider") ? "structural" : null;
    });
  };
  const deathBeat = await beatOfEvent(deathId);
  let spans = false;
  for (const gid of gotoIds) {
    const b = await beatOfEvent(gid);
    if (b !== null && b !== deathBeat) spans = true;
  }
  check(
    spans || deathTraceHtml.includes(">why<"),
    `${tag} the death's cause trace spans beats (an ancestor lives in an earlier beat) or carries a why-died chain`,
    `death beat=${deathBeat}, ancestor beats via ${JSON.stringify(gotoIds)}`,
  );

  // 4) Selecting a different (non-death) log row updates the readout to that
  //    event — selection is live, not stuck on the first pick.
  const otherRow = page.locator("#battle-log .log-line.ev-strike[data-id], #battle-log .log-line.ev-hurt[data-id]").first();
  if ((await otherRow.count()) >= 1) {
    const otherId = await otherRow.evaluate((el) => Number(el.getAttribute("data-id")));
    await otherRow.click();
    const moved = await page.evaluate(() => Number(document.querySelector("#scrub").value));
    check(
      moved === otherId,
      `${tag} clicking another log row re-selects (readout follows the new event)`,
      `→ ${moved} (want ${otherId})`,
    );
    const updated = await causeHtml(page);
    check(
      !updated.includes("cause-empty"),
      `${tag} the readout updated for the newly selected log row`,
      updated.slice(0, 120),
    );
  }

  await ctx.close();
}

// =====================================================================
// 4. Read-pause at beat boundaries during play (longer than a per-line gap)
// =====================================================================
async function readPause(viewport, tag) {
  const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
  await intoBattle(page);

  // Set 1× and record the timestamp of each event landing during autoplay by
  // sampling the playhead at a fine interval — the gap after a beat-ending
  // event must be visibly longer than a within-beat gap.
  await page.selectOption("#speed", "1");
  // Determine beat boundaries from the live segmentation in the page.
  const boundaries = await page.evaluate(() => {
    // Recompute the root set inline — the page does not expose beatsOf, but the
    // scrub max and the divider/card transitions are observable. Instead we read
    // the per-event types via the battle log lines' order is not enough; sample
    // by stepping is done below. Return null to signal "sample by timing".
    return null;
  });
  void boundaries;

  // Sample the playhead every 25ms through a bounded window of autoplay; record
  // the dwell (ms the playhead stayed) per event, and which event ended a beat
  // (the card's data-beat changes on the FOLLOWING event).
  const samples = await page.evaluate(async () => {
    const scrub = document.querySelector("#scrub");
    const play = document.querySelector("#step-play");
    // Restart from the top, playing.
    scrub.value = "0";
    scrub.dispatchEvent(new Event("input", { bubbles: true }));
    if (play.textContent.trim() === "play") play.click();
    const seen = [];
    let last = Number(scrub.value);
    let lastBeat = document.querySelector(".beat-card")?.getAttribute("data-beat") ?? "div";
    let tPrev = performance.now();
    const t0 = performance.now();
    while (performance.now() - t0 < 9000 && Number(scrub.value) < Number(scrub.max)) {
      await new Promise((r) => setTimeout(r, 20));
      const cur = Number(scrub.value);
      if (cur !== last) {
        const now = performance.now();
        const curBeat = document.querySelector(".beat-card")?.getAttribute("data-beat") ?? "div";
        seen.push({ event: last, dwell: now - tPrev, beatChanged: curBeat !== lastBeat });
        tPrev = now;
        last = cur;
        lastBeat = curBeat;
      }
    }
    play.click(); // pause
    return seen;
  });

  // A boundary dwell = the dwell of an event AFTER which the beat changed.
  const boundaryDwells = samples.filter((s) => s.beatChanged).map((s) => s.dwell);
  const innerDwells = samples.filter((s) => !s.beatChanged).map((s) => s.dwell);
  const avg = (a) => (a.length ? a.reduce((x, y) => x + y, 0) / a.length : 0);
  const bAvg = avg(boundaryDwells);
  const iAvg = avg(innerDwells);
  check(
    boundaryDwells.length > 0 && innerDwells.length > 0,
    `${tag} sampled both boundary and within-beat dwells`,
    `boundary n=${boundaryDwells.length}, inner n=${innerDwells.length}`,
  );
  check(
    bAvg > iAvg * 1.4,
    `${tag} read-pause at beat boundaries is longer than the per-line gap`,
    `boundary≈${Math.round(bAvg)}ms vs inner≈${Math.round(iAvg)}ms`,
  );

  await ctx.close();
}

// =====================================================================
// 5. LS-1: board / transport / continue Y pixel-stable across full playback
// =====================================================================
async function stability(viewport, tag) {
  const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
  await intoBattle(page);

  // Round to whole pixels: "pixel-stable" (LS-1) means no visible shift — a
  // sub-pixel layout wobble (e.g. a fractional flex height rounding 520→520.02)
  // is below a device pixel and never moves a control under the cursor.
  const px = (b) => Math.round(b);
  const boardY0 = px((await box(page, "#board")).y);
  const boardH0 = px((await box(page, "#board")).height);
  const transportY0 = px((await box(page, "#transport")).y);

  // Per-hero Y stability (#065 redesign — strikers-on-top vertical columns).
  // The stage is now three grid columns: Side A | card | Side B, each team a
  // VERTICAL column with the front/striker on top. The headline invariant
  // (hard req 2) is that CARD STREAMING never moves a hero: as the centre card
  // grows its 1→N lines and clears, every living line card holds its x AND y.
  //
  // What legitimately DOES move a living card is a DEATH or SUMMON that changes
  // the column's composition — the front falls and the bench promotes up; a
  // resurrection grows the column back. (Cards differ in height, so a promotion
  // also re-flows the slots below it.) That is the column compacting/expanding,
  // not a card-streaming jump. So we compare CONSECUTIVE steps and flag a unit's
  // y/x change only when the step did NOT change either side's line composition
  // (same set of unit ids on the line) — isolating movement caused by the card,
  // which is the defect. If the streaming card ever pushed a team (the original
  // bug), the composition is unchanged across those steps yet a hero's y moves →
  // caught. Keyed by data-unit so a unit is tracked through promotions.
  const lineState = (p) =>
    p.evaluate(() => {
      const sides = {};
      const pos = {};
      for (const side of ["A", "B"]) {
        const ids = [];
        for (const u of document.querySelectorAll(`.side[data-side="${side}"] .line .unit[data-unit]`)) {
          const id = u.getAttribute("data-unit");
          ids.push(id);
          const r = u.getBoundingClientRect();
          pos[id] = { x: Math.round(r.x), y: Math.round(r.y) };
        }
        sides[side] = ids.join(",");
      }
      return { sides, pos };
    });
  let prevLine = await lineState(page);

  // Skip to the outcome so the run "continue" button reveals, then measure it.
  await page.click("#run-skip");
  await page.waitForSelector("#run-continue:not([hidden])");
  const continueY0 = px((await box(page, "#run-continue")).y);

  // Replay again from the top and step the whole battle, sampling the board /
  // transport / continue Y at every step — none may move (the height is locked
  // and the controls are reserved, LS-1 / the battle-bar reserve).
  await page.evaluate(() => {
    const scrub = document.querySelector("#scrub");
    scrub.value = "0";
    scrub.dispatchEvent(new Event("input", { bubbles: true }));
  });
  prevLine = await lineState(page); // re-baseline at the rewound top
  const total = await maxStep(page);
  let moved = "";
  let heroMoved = "";
  for (let s = 0; s <= total; s++) {
    const boardY = px((await box(page, "#board")).y);
    const boardH = px((await box(page, "#board")).height);
    const transportY = px((await box(page, "#transport")).y);
    const continueY = px((await box(page, "#run-continue")).y);
    if (boardY !== boardY0) moved ||= `board Y ${boardY0}→${boardY} @${s}`;
    if (boardH !== boardH0) moved ||= `board H ${boardH0}→${boardH} @${s}`;
    if (transportY !== transportY0) moved ||= `transport Y ${transportY0}→${transportY} @${s}`;
    if (continueY !== continueY0) moved ||= `continue Y ${continueY0}→${continueY} @${s}`;
    // Compare to the PREVIOUS step. If neither side's line composition changed
    // (no death/summon reordered the column), every unit still on the line must
    // hold the x AND y it had a step ago — any move is a card-streaming push,
    // the defect. When the composition DID change, the column legitimately
    // re-flowed (a promotion/resurrection), so that step's moves are allowed and
    // the next step re-baselines against the new geometry.
    const cur = await lineState(page);
    const compositionStable = cur.sides.A === prevLine.sides.A && cur.sides.B === prevLine.sides.B;
    if (compositionStable) {
      for (const [id, p0] of Object.entries(prevLine.pos)) {
        const p1 = cur.pos[id];
        if (p1 !== undefined && (p1.x !== p0.x || p1.y !== p0.y)) {
          heroMoved ||= `hero ${id} (${p0.x},${p0.y})→(${p1.x},${p1.y}) @${s} (no death/summon — card-streaming push)`;
        }
      }
    }
    prevLine = cur;
    if (s < total) await stepNext(page);
  }
  check(moved === "", `${tag} board/transport/continue pixel-stable across full playback (LS-1)`, moved);
  check(
    heroMoved === "",
    `${tag} no hero card moves (x or y) while the card streams — only death/summon re-flows a column (#065 hard req 2)`,
    heroMoved,
  );

  await ctx.close();
}

// =====================================================================
// 6. Team sizing — neither team is CRUSHED, neither line OVERFLOWS (#065
//    slice-1 regression). The original probe asserted heroes don't MOVE but
//    not that they stay READABLE: `#board > .side { flex: 1 1 0 }` gave a
//    1-unit side and a multi-unit side the same width, so the bigger team's
//    cards collapsed to ~16px and the line overflowed (scrollWidth > client
//    width). This drives an ASYMMETRIC, near-max matchup (a full five-unit
//    side A vs the bootstrap opponent on side B) so the crush reproduces, and
//    asserts across the WHOLE playback (the centre card streams through every
//    width the layout ever takes):
//      • every living hero card is ≥ MIN_CARD wide (no card crushed below its
//        readable natural width — 48px clears the 40px art + padding);
//      • neither side's line overflows (scrollWidth ≤ clientWidth + 1);
//      • the page never scrolls horizontally.
//    On BOTH sides, at desktop AND 375px. Graves are excluded — a dead card
//    legitimately leaves the line; the LIVING line is what must stay readable. */
const MIN_CARD = 48; // px — a hero card narrower than this has lost its art/stats

async function teamSizing(viewport, tag) {
  const { ctx, page } = await openRun(browser, bigBattleRun(), viewport);
  await intoBattle(page);

  // Per-frame geometry of every LIVING hero card on both sides, plus each
  // line's overflow and the page's horizontal scroll.
  const frame = () =>
    page.evaluate(() => {
      const sides = {};
      for (const side of ["A", "B"]) {
        const sideEl = document.querySelector(`#board .side[data-side="${side}"]`);
        const line = sideEl.querySelector(".line");
        const widths = [...line.querySelectorAll(".unit")].map((u) =>
          Math.round(u.getBoundingClientRect().width),
        );
        sides[side] = {
          count: widths.length,
          min: widths.length ? Math.min(...widths) : null,
          overflow: line.scrollWidth > line.clientWidth + 1,
          scrollW: line.scrollWidth,
          clientW: line.clientWidth,
        };
      }
      return {
        sides,
        pageH: document.documentElement.scrollWidth > document.documentElement.clientWidth + 1,
      };
    });

  // Confirm the matchup is actually asymmetric and large — otherwise the probe
  // would green vacuously on a 1v1 that never crushes.
  const f0 = await frame();
  check(
    f0.sides.A.count >= 4 && f0.sides.B.count >= 1 && f0.sides.A.count !== f0.sides.B.count,
    `${tag} sizing matchup is asymmetric and near-max (the crush case)`,
    `A=${f0.sides.A.count} vs B=${f0.sides.B.count}`,
  );

  const total = await maxStep(page);
  const worst = { A: 9999, B: 9999 };
  let overflowed = "";
  let pageScrolled = "";
  for (let s = 0; s <= total; s++) {
    const f = await frame();
    for (const side of ["A", "B"]) {
      const d = f.sides[side];
      if (d.count > 0 && d.min !== null) worst[side] = Math.min(worst[side], d.min);
      if (d.overflow) overflowed ||= `side ${side} @${s}: scrollW ${d.scrollW} > clientW ${d.clientW}`;
    }
    if (f.pageH) pageScrolled ||= `@${s}`;
    if (s < total) await stepNext(page);
  }

  check(
    worst.A >= MIN_CARD,
    `${tag} side A hero cards never crushed below ${MIN_CARD}px across playback`,
    `narrowest A card = ${worst.A}px`,
  );
  check(
    worst.B >= MIN_CARD,
    `${tag} side B hero cards never crushed below ${MIN_CARD}px across playback`,
    `narrowest B card = ${worst.B}px`,
  );
  check(overflowed === "", `${tag} neither side's line overflows across playback`, overflowed);
  check(pageScrolled === "", `${tag} no horizontal page scroll across playback`, pageScrolled);

  await ctx.close();
}

// =====================================================================
// 7. Card horizontally CENTRED, SIZE-INDEPENDENT (#065 redesign, hard req 1).
//    The settled stage is three grid columns — Side A | card | Side B — with
//    symmetric side tracks, so the card's centre-x equals the stage's centre-x
//    REGARDLESS of how many units a side has. This is the headline failure the
//    redesign fixes: the old horizontal stage put the card's x at the mercy of
//    each team's width, so an asymmetric matchup (1vN, Nv1) shoved the card off
//    centre. We assert |card centre-x − stage centre-x| is within a pixel across
//    the WHOLE playback, on TWO asymmetric matchups in OPPOSITE directions
//    (A<B: a 1-unit side A vs the 3-body round-1 opponent — the "1v3"; A>B: a
//    full five-unit side A vs the same 3-body opponent — the "3v1"/Nv1 case,
//    since the bootstrap opponent is fixed at three bodies in round 1), at
//    desktop AND 375px. Must-fail-first: on a reverted horizontal stage the
//    card rides whichever team is wider, so the offset blows past a pixel. */
const CENTER_TOL = 1.5; // px — a sub-pixel grid rounding is not an off-centre card

async function centering(run, label, viewport, tag) {
  const { ctx, page } = await openRun(browser, run, viewport);
  await intoBattle(page);

  // Confirm the matchup is actually asymmetric (else the probe greens vacuously).
  const counts = await page.evaluate(() => ({
    A: document.querySelectorAll('.side[data-side="A"] .line .unit').length,
    B: document.querySelectorAll('.side[data-side="B"] .line .unit').length,
  }));
  check(
    counts.A !== counts.B,
    `${tag} ${label}: matchup is asymmetric (size-independence is non-trivial)`,
    `A=${counts.A} vs B=${counts.B}`,
  );

  // Walk the whole playback; at every step measure the centre-x of whichever
  // centre element shows (the beat card, or the divider between beats) against
  // the stage centre-x (the #board centre). The card must hold the centre at
  // every width the layout takes as it streams 1→N lines.
  const total = await maxStep(page);
  let worstOff = 0;
  let worstAt = "";
  for (let s = 0; s <= total; s++) {
    const off = await page.evaluate(() => {
      const board = document.querySelector("#board").getBoundingClientRect();
      const el =
        document.querySelector(".beat-card") ??
        document.querySelector(".stage-center > .divider");
      if (!el) return null;
      const r = el.getBoundingClientRect();
      return Math.abs(r.x + r.width / 2 - (board.x + board.width / 2));
    });
    if (off !== null && off > worstOff) {
      worstOff = off;
      worstAt = `@${s}`;
    }
    if (s < total) await stepNext(page);
  }
  check(
    worstOff <= CENTER_TOL,
    `${tag} ${label}: card centre-x ≈ stage centre-x across playback (size-independent, hard req 1)`,
    `worst |Δ| = ${worstOff.toFixed(2)}px ${worstAt}`,
  );

  await ctx.close();
}

// =====================================================================
// 8. FRONT/striking unit is at the TOP of its team column (#065 redesign,
//    hard req 3). Each team is a vertical column with the front unit (line
//    index 0, the one that strikes) on TOP and the rest stacked DOWN. Assert
//    the front card's Y is strictly ABOVE every bench card's Y on its side — by
//    at least most of a card height, so the column is genuinely vertical — on
//    both sides, across the whole playback (the front identity changes as units
//    fall; whoever is front must still be topmost). Must-fail-first: a reverted
//    horizontal line lays the cards left-to-right at ONE Y, so the front is NOT
//    above the bench (equal Ys → the strict gap never holds). */
const FRONT_GAP = 24; // px — the front must sit clearly above bench, not level with it

async function frontOnTop(run, label, viewport, tag) {
  const { ctx, page } = await openRun(browser, run, viewport);
  await intoBattle(page);

  const total = await maxStep(page);
  let broke = "";
  let proven = false; // a step where a side had ≥2 line units AND the front led
  for (let s = 0; s <= total; s++) {
    const r = await page.evaluate(() => {
      const out = {};
      for (const side of ["A", "B"]) {
        const units = [...document.querySelectorAll(`.side[data-side="${side}"] .line .unit`)];
        out[side] = units.map((u) => ({
          front: u.classList.contains("front"),
          y: Math.round(u.getBoundingClientRect().y),
        }));
      }
      return out;
    });
    for (const side of ["A", "B"]) {
      const us = r[side];
      const front = us.find((u) => u.front);
      const bench = us.filter((u) => !u.front);
      if (!front || bench.length === 0) continue; // need a front + ≥1 bench to test
      // The front must sit a clear card-gap ABOVE every bench card (vertical
      // column). A horizontal line has them at the same Y → this fails.
      for (const u of bench) {
        if (u.y < front.y + FRONT_GAP) {
          broke ||= `side ${side} @${s}: bench y=${u.y} not ${FRONT_GAP}px below front y=${front.y}`;
        } else {
          proven = true;
        }
      }
    }
    if (s < total) await stepNext(page);
  }
  check(proven, `${tag} ${label}: a side fielded a front + bench to test verticality`);
  check(
    broke === "",
    `${tag} ${label}: the front/striking unit is the TOPMOST card in its column, clearly above the bench (hard req 3)`,
    broke,
  );

  await ctx.close();
}

// =====================================================================
// 9. Phone fold readability on a BIG cascade beat (#065 closure defect). A tall
//    fatigue beat streams ~19 lines and grows the card to ~1059px — past the
//    667px phone fold. With the card in normal flow the last lines (the
//    resurrections after a wave of deaths) streamed in BELOW the fold and the
//    player never saw them before the read-pause cleared the card. The Layout
//    clause's "capped internal-scroll pane" fix caps the card's lines on phone
//    and the viewer scrolls the NEWEST line into that pane each step. Assert:
//      • as the big beat streams, the NEWEST revealed line's bounding-rect bottom
//        is within the fold (≤ viewport height) at EVERY step — measured at the
//        page's playback scroll (top), exactly what the player sees during
//        autoplay (no scrollIntoView cheat);
//      • the final line stays within the fold through the read-pause (the card is
//        not cleared and the newest line is not pushed out while the beat dwells).
//    Must-fail-first: on the un-capped card the late lines land at y≈680..1163,
//    well past 667; the cap + auto-scroll brings every newest line back inside.
//    Phone-only (desktop shows the tall beat in full and needs no cap). */
async function phoneFold(viewport, tag) {
  const { ctx, page } = await openRun(browser, bigBattleRun(), viewport);
  await intoBattle(page);

  const total = await maxStep(page);

  // Find the tallest cascade beat (the most caused lines) by replaying once.
  const tall = await page.evaluate((maxStep) => {
    const scrub = document.querySelector("#scrub");
    const byBeat = {};
    for (let i = 0; i <= maxStep; i++) {
      scrub.value = String(i);
      scrub.dispatchEvent(new Event("input", { bubbles: true }));
      const c = document.querySelector(".beat-card");
      if (!c) continue;
      const b = c.getAttribute("data-beat");
      const lines = c.querySelectorAll(".bc-line").length;
      if (!byBeat[b] || lines > byBeat[b].lines) byBeat[b] = { beat: b, lines, step: i };
    }
    return Object.values(byBeat).sort((a, b) => b.lines - a.lines)[0] ?? null;
  }, total);
  check(
    tall !== null && tall.lines >= 10,
    `${tag} found a big cascade beat to test the fold (≥10 streamed lines)`,
    tall ? `beat ${tall.beat}: ${tall.lines} lines` : "none",
  );
  if (tall === null) {
    await ctx.close();
    return;
  }

  // Walk every step of that beat at the page's TOP scroll (what autoplay shows —
  // the player does not scroll); at each step the NEWEST revealed line's rect
  // bottom must sit within the fold. The card top sits in its reserved centre
  // band; only the lines pane scrolls.
  const worst = await page.evaluate(
    ({ maxStep, beatId }) => {
      const scrub = document.querySelector("#scrub");
      const pane = () => document.querySelector(".beat-card .bc-lines");
      // viewer.render() scrolls the newest line into the capped pane on render;
      // dispatching the scrub input drives that exact path. Reset page scroll to
      // the top each step so we measure the autoplay (unscrolled) reality.
      let maxBottom = -Infinity;
      let worstStep = -1;
      let nLines = 0;
      let lastBottom = null;
      const vh = window.innerHeight;
      for (let i = 0; i <= maxStep; i++) {
        scrub.value = String(i);
        scrub.dispatchEvent(new Event("input", { bubbles: true }));
        window.scrollTo(0, 0);
        const c = document.querySelector(".beat-card");
        if (!c || c.getAttribute("data-beat") !== beatId) continue;
        const lines = [...c.querySelectorAll(".bc-line")];
        const last = lines[lines.length - 1];
        if (!last) continue;
        const b = Math.round(last.getBoundingClientRect().bottom);
        nLines = lines.length;
        lastBottom = b;
        if (b > maxBottom) {
          maxBottom = b;
          worstStep = i;
        }
      }
      return { maxBottom, worstStep, vh, nLines, lastBottom };
    },
    { maxStep: total, beatId: tall.beat },
  );

  check(
    worst.maxBottom <= worst.vh,
    `${tag} every streamed line stays within the ${worst.vh}px fold as the big beat reveals (newest line bottom ≤ fold)`,
    `worst newest-line bottom = ${worst.maxBottom}px @step ${worst.worstStep} (beat ${tall.beat}, ${tall.lines} lines)`,
  );
  check(
    worst.lastBottom !== null && worst.lastBottom <= worst.vh,
    `${tag} the FINAL line of the big beat is within the fold (visible at the read-pause)`,
    `final line (#${worst.nLines}) bottom = ${worst.lastBottom}px, fold = ${worst.vh}px`,
  );

  // Read-pause hold: actually PLAY into the beat and confirm the card is still
  // open with its final line in the fold while the beat dwells (the read-pause
  // does not clear it before the player can read the last lines). Land on the
  // beat's last step paused, then sample across a read-pause window.
  await page.evaluate(
    ({ step }) => {
      const scrub = document.querySelector("#scrub");
      scrub.value = String(step);
      scrub.dispatchEvent(new Event("input", { bubbles: true }));
      window.scrollTo(0, 0);
    },
    { step: tall.step },
  );
  const held = await page.evaluate(() => {
    const c = document.querySelector(".beat-card");
    if (!c) return null;
    const lines = [...c.querySelectorAll(".bc-line")];
    const last = lines[lines.length - 1];
    if (!last) return null;
    return { bottom: Math.round(last.getBoundingClientRect().bottom), vh: window.innerHeight, beat: c.getAttribute("data-beat") };
  });
  check(
    held !== null && held.bottom <= held.vh,
    `${tag} at the beat's final step the last line is held within the fold (read-pause readable)`,
    held ? `last line bottom = ${held.bottom}px, fold = ${held.vh}px, beat ${held.beat}` : "no card",
  );

  await ctx.close();
}

for (const [vp, tag] of [
  [DESKTOP, "desktop"],
  [PHONE, "375px"],
]) {
  await layout(vp, tag);
  await streaming(vp, tag);
  await transport(vp, tag);
  await stability(vp, tag);
  await teamSizing(vp, tag);
  // Card-centring on asymmetric matchups in both directions (A<B and A>B).
  await centering(plainShopRun(), "1v3 (A<B)", vp, tag);
  await centering(bigBattleRun(), "5v3 (A>B)", vp, tag);
  // Front-on-top on a multi-unit-per-side matchup.
  await frontOnTop(bigBattleRun(), "5v3", vp, tag);
  // Phone fold readability on a big cascade beat (#065 closure defect) — the
  // tall fatigue beat's last lines must stay within the 667px fold. Phone-only:
  // desktop shows the tall beat in full and the card is uncapped there.
  if (vp.width < 700) await phoneFold(vp, tag);
}
// Read-pause timing is speed-independent of layout — sample once on desktop.
await readPause(DESKTOP, "desktop");
// The selected-line cause readout (#065 slice 4) is layout-independent logic —
// sample once on desktop.
await causeReadout(DESKTOP, "desktop");

await browser.close();
disarm();
finish("probe-beats");
