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

  const stage = await box(page, "#stage-col");
  const logCol = await box(page, "#log-col");

  if (viewport.width >= 700) {
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
    check(
      logCol.y >= stage.y + stage.height - 2,
      `${tag} log STACKS below the stage`,
      `stage bottom=${Math.round(stage.y + stage.height)}, log top=${Math.round(logCol.y)}`,
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
  }

  check(sawCard, `${tag} hero-affecting beats open a centre card`);
  check(sawDivider, `${tag} structural beats render a turn divider (not a card)`);
  check(sawStream, `${tag} card lines stream in as the playhead advances within a beat`);
  check(sawClear, `${tag} the card clears when the next beat opens`);

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
  const total = await maxStep(page);
  let moved = "";
  for (let s = 0; s <= total; s++) {
    const boardY = px((await box(page, "#board")).y);
    const boardH = px((await box(page, "#board")).height);
    const transportY = px((await box(page, "#transport")).y);
    const continueY = px((await box(page, "#run-continue")).y);
    if (boardY !== boardY0) moved ||= `board Y ${boardY0}→${boardY} @${s}`;
    if (boardH !== boardH0) moved ||= `board H ${boardH0}→${boardH} @${s}`;
    if (transportY !== transportY0) moved ||= `transport Y ${transportY0}→${transportY} @${s}`;
    if (continueY !== continueY0) moved ||= `continue Y ${continueY0}→${continueY} @${s}`;
    if (s < total) await stepNext(page);
  }
  check(moved === "", `${tag} board/transport/continue pixel-stable across full playback (LS-1)`, moved);

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
}
// Read-pause timing is speed-independent of layout — sample once on desktop.
await readPause(DESKTOP, "desktop");

await browser.close();
disarm();
finish("probe-beats");
