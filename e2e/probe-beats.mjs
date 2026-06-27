// Acting-card battle (#082 slice D) — the real-layout checks vitest cannot see.
// Drives an actual run battle in the browser and asserts the new "compact board
// + acting full card" design:
//   • layout: the board is one column (header → 3-col grid → bottom trace strip),
//     transport below; at 375px the grid stacks with NO horizontal page scroll;
//   • the centre is the ACTING full card for a Strike beat (its RESULT rows
//     stream in as the playhead advances) or a phase caption for a beat with no
//     actor — exactly one shows, and it clears/changes between beats;
//   • transport stays event-granular: next/prev/scrub move ONE event and the
//     card reveals up to it (a half-revealed mid-beat);
//   • the bottom trace strip carries one chip per beat, the current one lit, and
//     clicking a chip scrubs the playhead to that beat;
//   • the on-demand cause readout (carried from #065 slice 4): nothing selected
//     → neutral prompt; clicking an acting-card RESULT row populates ITS cross-
//     beat ancestry without moving the playhead; a death's trace spans beats;
//   • play inserts a longer read-pause at a beat boundary than between lines;
//   • LS-1: board, transport and the run "continue" button hold their Y (and the
//     board its height) to the pixel across a full playback, desktop AND 375px.
//
// Scripted Playwright (committed), per #012: the repeatable layout probes live
// here and run under `npm run e2e`.

import { DESKTOP, PHONE, armGuard, box, check, finish, launch, openRun, plainShopRun, bigBattleRun } from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

/** Start the injected run's fight and land on the paused replay's first event
 * (autoplay is on in the real app — pause immediately so the probe drives the
 * transport deterministically). Returns once the board has rendered. */
async function intoBattle(page) {
  await page.click("#run-fight");
  await page.waitForSelector("#run-battle:not([hidden])");
  await page.waitForSelector("#board .bv-side");
  await page.waitForSelector("#step-play");
  if ((await page.locator("#step-play").textContent())?.trim() === "pause") {
    await page.click("#step-play");
  }
  await page.evaluate(() => {
    const scrub = document.querySelector("#scrub");
    scrub.value = "0";
    scrub.dispatchEvent(new Event("input", { bubbles: true }));
  });
}

const playhead = (page) => page.evaluate(() => Number(document.querySelector("#scrub").value));
const maxStep = (page) => page.evaluate(() => Number(document.querySelector("#scrub").max));
/** Count of revealed RESULT/CHAIN rows in the open acting card. */
const cardRows = (page) => page.locator(".acting-card .ac-row").count();
const hasCard = (page) => page.locator(".acting-card").count();
const hasPhase = (page) => page.locator(".acting-phase").count();
/** The beat the centre currently shows (card OR phase carry data-beat). */
const centreBeat = (page) =>
  page.evaluate(() =>
    document.querySelector(".acting-card, .acting-phase")?.getAttribute("data-beat") ?? null,
  );

async function stepNext(page) {
  await page.click("#step-next");
}

// =====================================================================
// 1. Layout — one column: header, grid, trace strip; transport below; at 375px
//    the grid stacks with no horizontal page scroll.
// =====================================================================
async function layout(viewport, tag) {
  const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
  await intoBattle(page);

  const header = await box(page, "#board .bv-header");
  const grid = await box(page, "#board .bv-grid");
  const strip = await box(page, "#board .trace-strip");
  const board = await box(page, "#board");
  const transport = await box(page, "#transport");

  check(header.y <= grid.y + 1, `${tag} header sits above the grid`, `header y=${Math.round(header.y)}, grid y=${Math.round(grid.y)}`);
  check(
    strip.y >= grid.y + grid.height - 2,
    `${tag} trace strip sits below the grid (bottom of the board)`,
    `grid bottom=${Math.round(grid.y + grid.height)}, strip y=${Math.round(strip.y)}`,
  );
  check(
    transport.y >= board.y + board.height - 2,
    `${tag} transport sits below the board`,
    `board bottom=${Math.round(board.y + board.height)}, transport y=${Math.round(transport.y)}`,
  );

  if (viewport.width >= 700) {
    const a = await box(page, '#board .bv-side[data-side="A"]');
    const b = await box(page, '#board .bv-side[data-side="B"]');
    check(a.x < b.x, `${tag} side A is left of side B`, `A x=${Math.round(a.x)}, B x=${Math.round(b.x)}`);
  }

  const hScroll = await page.evaluate(
    () => document.documentElement.scrollWidth > document.documentElement.clientWidth + 1,
  );
  check(!hScroll, `${tag} no horizontal page scroll (the trace strip scrolls itself)`);

  await ctx.close();
}

// =====================================================================
// 2. The centre: acting card (Strike beat) vs phase caption; RESULT streams;
//    clears between beats.
// =====================================================================
async function streaming(viewport, tag) {
  const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
  await intoBattle(page);

  const total = await maxStep(page);
  let sawCard = false;
  let sawPhase = false;
  let sawStream = false;
  let sawClear = false;
  let prevBeat = await centreBeat(page);
  let prevRows = await cardRows(page);
  let maxCardFrac = 0;

  for (let s = 0; s < total; s++) {
    await stepNext(page);
    const card = await hasCard(page);
    const phase = await hasPhase(page);
    check(card + phase === 1, `${tag} step ${s + 1}: exactly one of acting-card / phase shows`, `card=${card} phase=${phase}`);
    if (card) sawCard = true;
    if (phase) sawPhase = true;

    const beat = await centreBeat(page);
    const rows = await cardRows(page);
    if (card && beat === prevBeat && rows > prevRows) sawStream = true;
    if (beat !== prevBeat && prevBeat !== null) sawClear = true;
    prevBeat = beat;
    prevRows = rows;

    if (card && viewport.width >= 700) {
      const g = await page.evaluate(() => {
        const c = document.querySelector(".acting-card");
        const stage = document.querySelector(".stage-center");
        if (!c || !stage) return null;
        return { cw: c.getBoundingClientRect().width, sw: stage.getBoundingClientRect().width };
      });
      if (g) maxCardFrac = Math.max(maxCardFrac, g.cw / g.sw);
    }
  }

  check(sawCard, `${tag} a Strike beat opens the centre acting card`);
  check(sawPhase, `${tag} a beat with no actor shows a phase caption (not a card)`);
  check(sawStream, `${tag} the acting card's RESULT rows stream in as the playhead advances`);
  check(sawClear, `${tag} the centre clears/changes when the next beat opens`);
  if (viewport.width >= 700) {
    check(
      maxCardFrac > 0 && maxCardFrac <= 1.01,
      `${tag} the acting card fits within its centre column (capped, not overflowing)`,
      `widest card/stage = ${maxCardFrac.toFixed(3)}`,
    );
  }

  await ctx.close();
}

// =====================================================================
// 3. Transport stays event-granular: next/prev move ONE event; scrub mid-beat
//    yields a partial RESULT reveal; a trace chip scrubs to that beat.
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

  const total = await maxStep(page);
  let foundPartial = false;
  for (let s = 0; s < total && !foundPartial; s++) {
    await stepNext(page);
    if ((await cardRows(page)) >= 1 && (await hasCard(page))) {
      const beatNow = await centreBeat(page);
      const rowsNow = await cardRows(page);
      await stepNext(page);
      const beatNext = await centreBeat(page);
      const rowsNext = await cardRows(page);
      if (beatNext === beatNow && rowsNext > rowsNow) {
        foundPartial = true;
        check(true, `${tag} scrubbing mid-beat shows a half-revealed acting card`, `${rowsNow} → ${rowsNext} rows, same beat`);
      }
    }
  }
  check(foundPartial, `${tag} found a multi-row beat to prove partial reveal`);

  const chip = page.locator(".trace-strip .tr-chip[data-id]").nth(2);
  const targetId = await chip.evaluate((el) => Number(el.getAttribute("data-id")));
  await chip.click();
  const pj = await playhead(page);
  check(pj === targetId, `${tag} a trace chip scrubs the playhead to that beat`, `→ ${pj} (want ${targetId})`);

  await ctx.close();
}

// =====================================================================
// 3b. On-demand cause readout (carried from #065 slice 4).
// =====================================================================
const causeHtml = (page) => page.locator("#event-cause").innerHTML();
const causeText = (page) => page.locator("#event-cause").innerText();

async function causeReadout(viewport, tag) {
  const { ctx, page } = await openRun(browser, bigBattleRun(), viewport);
  await intoBattle(page);

  const emptyHtml = await causeHtml(page);
  check(
    emptyHtml.includes("cause-empty") && !emptyHtml.includes(">cause<") && !emptyHtml.includes(">why<"),
    `${tag} nothing selected → neutral cause prompt (no trace)`,
    emptyHtml.slice(0, 120),
  );

  const total = await maxStep(page);
  let rowId = -1;
  for (let s = 0; s < total && rowId < 0; s++) {
    await stepNext(page);
    if ((await cardRows(page)) >= 1) {
      rowId = await page.evaluate(() => Number(document.querySelector(".acting-card .ac-row[data-id]").getAttribute("data-id")));
    }
  }
  check(rowId >= 0, `${tag} found an acting-card row to click`, `event ${rowId}`);

  const headBefore = await playhead(page);
  await page.locator(`.acting-card .ac-row[data-id="${rowId}"]`).click();
  const afterRow = await causeHtml(page);
  check(
    !afterRow.includes("cause-empty") && (afterRow.includes(">cause<") || afterRow.includes(">why<")),
    `${tag} clicking an acting-card row populates the cause readout`,
    afterRow.slice(0, 160),
  );
  check(
    (await playhead(page)) === headBefore,
    `${tag} selecting a row does NOT move the playhead (trace is on demand)`,
    `head ${headBefore} → ${await playhead(page)}`,
  );

  // A multi-hop cause chain spans beats: jump to the end so every beat's chip
  // exists, then select each chip's beat-end event until one traces a chain.
  await page.evaluate(() => {
    const scrub = document.querySelector("#scrub");
    scrub.value = scrub.max;
    scrub.dispatchEvent(new Event("input", { bubbles: true }));
  });
  let sawChain = false;
  const chips = await page.locator(".trace-strip .tr-chip[data-id]").count();
  for (let i = 0; i < chips && !sawChain; i++) {
    await page.locator(".trace-strip .tr-chip[data-id]").nth(i).click();
    const t = await causeText(page);
    if (t.includes("←")) sawChain = true;
  }
  check(sawChain, `${tag} at least one beat's cause readout shows a multi-hop chain (←)`);

  await ctx.close();
}

// =====================================================================
// 4. Read-pause at beat boundaries during play (longer than a per-line gap).
// =====================================================================
async function readPause(viewport, tag) {
  const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
  await intoBattle(page);
  await page.selectOption("#speed", "1");

  const samples = await page.evaluate(async () => {
    const scrub = document.querySelector("#scrub");
    const play = document.querySelector("#step-play");
    scrub.value = "0";
    scrub.dispatchEvent(new Event("input", { bubbles: true }));
    if (play.textContent.trim() === "play") play.click();
    const beatOf = () => document.querySelector(".acting-card, .acting-phase")?.getAttribute("data-beat") ?? "x";
    const seen = [];
    let last = Number(scrub.value);
    let lastBeat = beatOf();
    let tPrev = performance.now();
    const t0 = performance.now();
    while (performance.now() - t0 < 9000 && Number(scrub.value) < Number(scrub.max)) {
      await new Promise((r) => setTimeout(r, 20));
      const cur = Number(scrub.value);
      if (cur !== last) {
        const now = performance.now();
        const curBeat = beatOf();
        seen.push({ event: last, dwell: now - tPrev, beatChanged: curBeat !== lastBeat });
        tPrev = now;
        last = cur;
        lastBeat = curBeat;
      }
    }
    play.click();
    return seen;
  });

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
  check(bAvg > iAvg * 1.4, `${tag} read-pause at beat boundaries is longer than the per-line gap`, `boundary≈${Math.round(bAvg)}ms vs inner≈${Math.round(iAvg)}ms`);

  await ctx.close();
}

// =====================================================================
// 5. LS-1: board / transport / continue Y + board height pixel-stable.
// =====================================================================
async function stability(viewport, tag) {
  const { ctx, page } = await openRun(browser, plainShopRun(), viewport);
  await intoBattle(page);

  const px = (b) => Math.round(b);
  const boardY0 = px((await box(page, "#board")).y);
  const boardH0 = px((await box(page, "#board")).height);
  const transportY0 = px((await box(page, "#transport")).y);

  await page.click("#run-skip");
  await page.waitForSelector("#run-continue:not([hidden])");
  const continueY0 = px((await box(page, "#run-continue")).y);

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
await readPause(DESKTOP, "desktop");
await causeReadout(DESKTOP, "desktop");

await browser.close();
disarm();
finish("probe-beats");
