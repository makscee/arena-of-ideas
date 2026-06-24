// Motion probes (#065) — the two defects a STILL screenshot is blind to, made
// catchable. Still shots froze a single frame; a re-animating line or a
// both-then-one red flash only exists across frames. These two probes read the
// motion state directly from the live DOM as the playhead advances, so a
// regression reddens an assertion instead of slipping past a static capture.
//
//   • Defect A — the line fade-in must animate ONLY the newly revealed line.
//     The card re-renders its whole inner HTML each step; arming `bc-line-in`
//     on every `.bc-line` re-ran the reveal on all of them every step. We read
//     each line's computed `animation-name`: an already-shown line must read
//     `none` (it is static), and only the just-revealed line may read
//     `bc-line-in`. Pre-fix: every line reads `bc-line-in` at every step → FAIL.
//
//   • Defect B — the red hit mark must equal the kernel truth at every step:
//     a unit is marked iff it has a Hurt event at or before the playhead within
//     the open beat. The kernel deals strike damage one-directionally (battle.ts
//     kernelConsequences: a Strike proposes exactly one Hurt, on the defender),
//     and each Strike is its own beat — so at a strike's START the Strike event
//     hurts no one and NOBODY may be red; at resolution only the hurt unit(s)
//     may be red. Pre-fix `subjectsOf` reddened BOTH the striker and defender at
//     the Strike step, then only the defender at the Hurt step (both-then-one).
//     Truth is read from the revealed `bc-hurt` card lines (each names its unit,
//     mapped to a board card's data-unit by the shared display name); the marked
//     set is read from `.unit.hit[data-unit]`. The two sets must be EQUAL.
//
// Run under `npm run e2e` (the probe-*.mjs glob). A companion capture script,
// e2e/motion-frames.mjs, writes step-through frames for a human to scrub.

import { armGuard, check, finish, launch, openRun, plainShopRun, DESKTOP } from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

/** Start the injected run's fight and land paused on event 0 — the transport
 * driven deterministically, never racing autoplay. */
async function intoBattle(page) {
  await page.click("#run-fight");
  await page.waitForSelector("#run-battle:not([hidden])");
  await page.waitForSelector("#board .side");
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

/** Per-revealed-line motion state at the current step: data-id and the computed
 * animation-name (the signal that a line is mid-reveal vs static). */
const lineStates = (page) =>
  page.evaluate(() =>
    [...document.querySelectorAll(".beat-card .bc-line")].map((l) => ({
      id: Number(l.getAttribute("data-id")),
      anim: getComputedStyle(l).animationName,
      opacity: Number(getComputedStyle(l).opacity),
    })),
  );

/** The card's open beat index (data-beat) at the current step, or null. */
const beatIndex = (page) =>
  page.evaluate(() => {
    const c = document.querySelector(".beat-card");
    return c ? Number(c.getAttribute("data-beat")) : null;
  });

// ----------------------------------------------------------------------
// Defect A — only the newly revealed line animates in.
// ----------------------------------------------------------------------
//
// Find a beat whose card reveals ≥2 lines on consecutive steps WITHOUT the beat
// changing (so step N shows lines 1..k and step N+1 shows lines 1..k+1 in the
// same card). Snapshot every line's animation-name at N, advance to N+1, and
// assert the lines present at N still read `none` (their reveal did not restart)
// while the one new line reads `bc-line-in`.
async function defectALineAnimation(page) {
  const max = await maxStep(page);

  // Walk the playhead to locate two consecutive steps inside one beat where the
  // line count grows — the moment a new line streams into an open card.
  let found = null;
  for (let n = 0; n < max; n++) {
    await stepTo(page, n);
    const beatN = await beatIndex(page);
    const linesN = await lineStates(page);
    if (beatN === null || linesN.length === 0) continue;
    await stepTo(page, n + 1);
    const beatN1 = await beatIndex(page);
    const linesN1 = await lineStates(page);
    if (beatN1 === beatN && linesN1.length === linesN.length + 1) {
      found = { n, beatN, before: linesN, after: linesN1 };
      break;
    }
  }

  if (!found) {
    check(false, "defect-A: located a beat that reveals a new line into an open card", "no such consecutive step found");
    return;
  }

  const beforeIds = new Set(found.before.map((l) => l.id));
  const newLines = found.after.filter((l) => !beforeIds.has(l.id));
  const oldLines = found.after.filter((l) => beforeIds.has(l.id));

  check(
    newLines.length === 1,
    "defect-A: exactly one new line appears at step N+1",
    `new=${newLines.length} (beat ${found.beatN}, step ${found.n}→${found.n + 1})`,
  );

  // The crux: every PRE-EXISTING line must NOT re-run its reveal. A re-render
  // that re-arms `bc-line-in` on all lines leaves every old line's animationName
  // = "bc-line-in" — the both-then-one of motion. Post-fix old lines read "none".
  const reanimated = oldLines.filter((l) => l.anim !== "none");
  check(
    reanimated.length === 0,
    "defect-A: already-revealed lines do NOT re-animate when a new line streams in",
    reanimated.length === 0
      ? `${oldLines.length} prior line(s) static (animation-name: none)`
      : `lines [${reanimated.map((l) => `#${l.id}:${l.anim}`).join(", ")}] re-ran their reveal`,
  );

  // And the new line DID animate in — the reveal still happens, just scoped.
  const newAnimated = newLines.every((l) => l.anim === "bc-line-in");
  check(
    newAnimated,
    "defect-A: the newly revealed line DOES animate in (bc-line-in)",
    newLines.map((l) => `#${l.id}:${l.anim}`).join(", "),
  );
}

// ----------------------------------------------------------------------
// Defect B — the red hit set equals the kernel's Hurt truth at every step.
// ----------------------------------------------------------------------
//
// At each step the marked set (`.unit.hit`) must equal the set of units that
// have a revealed Hurt line in the open beat (`bc-hurt`, id ≤ step). We check it
// at a strike's START (the Strike event step — nobody hurt yet → no red) and at
// its RESOLUTION (the Hurt step → exactly the hurt unit red). A "both then one"
// flash reddens the start-step assertion (two marked, zero expected).
async function defectBHitTruth(page) {
  const max = await maxStep(page);

  // label → data-unit, read off every board card (lines AND graves) so a Hurt
  // line's leading display name resolves to the unit it marks.
  const nameMap = await page.evaluate(() => {
    const m = {};
    for (const u of document.querySelectorAll(".side .unit[data-unit]")) {
      const label = u.querySelector(".uname")?.textContent?.trim();
      if (label) m[label] = u.getAttribute("data-unit");
    }
    return m;
  });

  /** The truth set at the current step: units named by a revealed `bc-hurt`
   * line, resolved via the display-name map. */
  const expectedHit = async () =>
    page.evaluate((names) => {
      const labels = Object.keys(names).sort((a, b) => b.length - a.length); // longest first: avoid prefix clashes
      const out = new Set();
      for (const l of document.querySelectorAll(".beat-card .bc-line.bc-hurt")) {
        const txt = l.textContent ?? "";
        const label = labels.find((n) => txt.startsWith(n));
        if (label) out.add(names[label]);
      }
      return [...out];
    }, nameMap);

  /** The marked set at the current step. */
  const markedHit = () =>
    page.evaluate(() => [...document.querySelectorAll(".unit.hit[data-unit]")].map((u) => u.getAttribute("data-unit")));

  const eq = (a, b) => a.length === b.length && [...a].sort().join("|") === [...b].sort().join("|");

  // Locate strike beats: a step whose card opens a beat that contains ≥1 bc-hurt
  // line somewhere, and whose FIRST revealed step shows the strike with no hurt
  // yet. We sweep every step and assert marked == expected at each — start steps
  // (zero expected) and resolution steps (the hurt unit) are both covered.
  let starts = 0;
  let resolutions = 0;
  let mismatches = 0;
  let firstMismatch = "";

  for (let n = 0; n <= max; n++) {
    await stepTo(page, n);
    const expected = await expectedHit();
    const marked = await markedHit();
    const ok = eq(marked, expected);
    if (!ok) {
      mismatches++;
      if (firstMismatch === "")
        firstMismatch = `step ${n}: marked=[${marked.join(",")}] expected=[${expected.join(",")}]`;
    }
    // Classify the step for evidence: a beat with hurts but none revealed yet at
    // this step is a strike START; a step that just revealed a hurt is a RESOLUTION.
    if (expected.length === 0) starts++;
    else resolutions++;
  }

  check(
    starts > 0 && resolutions > 0,
    "defect-B: swept both strike-start (0 hurt) and resolution (hurt) steps",
    `start-like steps=${starts}, resolution-like steps=${resolutions}`,
  );
  check(
    mismatches === 0,
    "defect-B: the red hit set EQUALS the kernel Hurt truth at every step",
    mismatches === 0 ? `all ${max + 1} steps consistent` : `${mismatches} mismatch(es); first: ${firstMismatch}`,
  );
}

// Each probe gets a fresh run/page so the transport starts clean (the battle is
// one-shot per fight — a second probe cannot re-click #run-fight on a page that
// already left the shop).
{
  const { ctx, page } = await openRun(browser, plainShopRun(), DESKTOP);
  await intoBattle(page);
  await defectALineAnimation(page);
  await ctx.close();
}
{
  const { ctx, page } = await openRun(browser, plainShopRun(), DESKTOP);
  await intoBattle(page);
  await defectBHitTruth(page);
  await ctx.close();
}

await browser.close();
disarm();
finish("probe-motion");
