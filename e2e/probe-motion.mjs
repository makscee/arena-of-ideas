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

import { armGuard, check, bigBattleRun, duelistRun, finish, launch, openRun, plainShopRun, DESKTOP } from "./lib.mjs";

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

// ----------------------------------------------------------------------
// Slice 2 — hero overlays: typed badges appear/increment in sync with the
// lines, sum on multi-hit, persist through the read-pause, clear at the next
// beat. This is the feature the director twice reported missing ("no text
// appears on top of heroes when they take damage"); the probe reads the badge
// numbers straight off the DOM and ties them to the kernel Hurt truth.
// ----------------------------------------------------------------------
//
// The badge is drawn ON the affected hero card (`.unit .ov-layer .ov-dmg` etc.)
// — NOT a side panel. We assert:
//   • appear/increment in sync — a unit's red −N damage badge equals the SUM of
//     the revealed `bc-hurt` lines for that unit at every step in the beat (so
//     the number grows as Hurt lines reveal, and is absent before the first).
//   • sum on multi-hit — a beat with ≥2 Hurts on one unit shows the summed
//     total, not the last hit.
//   • persist through the read-pause — the badge is still present at beat.end.
//   • clear at the next beat — stepping into the next beat removes it.

// Legibility floor for the rendered damage badge (#065 damage-legibility fix).
// Measured layout (offset) sizes: the old red-on-red "−N" pill that drowned in
// the hit-halo is 22×14px; the enlarged, darkened badge is 27×17px. The floor
// sits strictly between, so the old size FAILS and the new PASSES — a shrink back
// toward the unreadable original turns the probe red (must-fail-first verified).
const MIN_BADGE_W = 24; // px — old 22px wide; new 27px
const MIN_BADGE_H = 15; // px — old 14px tall; new 17px

/** The damage badge total drawn on each unit at the current step:
 * data-unit → the integer in its `.ov-dmg` badge (absent → not present). */
const damageBadges = (page) =>
  page.evaluate(() => {
    const out = {};
    for (const u of document.querySelectorAll(".unit[data-unit]")) {
      const b = u.querySelector(".ov-layer .ov-dmg");
      if (b) out[u.getAttribute("data-unit")] = Number(b.textContent.replace(/[^0-9]/g, ""));
    }
    return out;
  });

/** The kernel-truth damage per unit at the current step: the SUM of the amounts
 * named on the revealed `bc-hurt` card lines, resolved by display name. Read
 * from a SECOND DOM source (the card lines) so the assertion is not a tautology
 * with the badge it checks. Line text is e.g. "Silencer takes 2 → 0 hp". */
async function expectedDamage(page, nameMap) {
  return page.evaluate((names) => {
    const labels = Object.keys(names).sort((a, b) => b.length - a.length);
    const out = {};
    for (const l of document.querySelectorAll(".beat-card .bc-line.bc-hurt")) {
      const txt = l.textContent ?? "";
      const label = labels.find((n) => txt.startsWith(n));
      if (!label) continue;
      // "… takes N" or "… takes N (M absorbed)" — the first number after "takes".
      const m = txt.match(/takes\s+(\d+)/);
      if (!m) continue;
      const unit = names[label];
      out[unit] = (out[unit] ?? 0) + Number(m[1]);
    }
    return out;
  }, nameMap);
}

async function badgesInSyncWithLines(page, { requireMultiHit }) {
  const max = await maxStep(page);
  const nameMap = await page.evaluate(() => {
    const m = {};
    for (const u of document.querySelectorAll(".side .unit[data-unit]")) {
      const label = u.querySelector(".uname")?.textContent?.trim();
      if (label) m[label] = u.getAttribute("data-unit");
    }
    return m;
  });

  const eqMap = (a, b) => {
    const ka = Object.keys(a).sort();
    const kb = Object.keys(b).sort();
    if (ka.join("|") !== kb.join("|")) return false;
    return ka.every((k) => a[k] === b[k]);
  };

  let mismatches = 0;
  let firstMismatch = "";
  let badgeSteps = 0; // steps where at least one damage badge is on a hero
  let multiHitSeen = false; // a beat where one unit's badge summed ≥2 hits
  let clearSeen = false; // a badge present in one beat, gone at the next step's new beat
  // Legibility-size accumulator (#065 damage-legibility fix): the widest/tallest
  // rendered damage badge seen across the walk. The original badge was ~13×8px —
  // small enough that, sat on the red hit-halo as a red-on-red pill, the −N number
  // drowned in the glow (the director's twice-reported "no text on heroes"). The
  // fix enlarged + darkened it; this records the painted size so a future shrink
  // back toward that reddened size turns the probe red. Measured ON a hero card
  // (`.unit .ov-layer .ov-dmg`), so it is the real rendered pill, not the markup.
  let maxBadgeW = 0;
  let maxBadgeH = 0;

  let prevBeat = null;
  let prevHadBadge = false;
  for (let n = 0; n <= max; n++) {
    await stepTo(page, n);
    const badges = await damageBadges(page);
    const expected = await expectedDamage(page, nameMap);
    if (!eqMap(badges, expected)) {
      mismatches++;
      if (firstMismatch === "")
        firstMismatch = `step ${n}: badges=${JSON.stringify(badges)} expected=${JSON.stringify(expected)}`;
    }
    if (Object.keys(badges).length > 0) badgeSteps++;
    // Record the rendered size of the damage badge at this step (if shown). The
    // pop animation scales 0.6→1 over 0.22s; the offset (layout) box is immune to
    // that transform, so it reports the SETTLED size even on a mid-pop frame. Take
    // the max across the walk so the widest "−N" the run shows sets the floor.
    const dim = await page.evaluate(() => {
      const b = document.querySelector(".unit[data-unit] .ov-layer .ov-dmg");
      if (!b) return null;
      // offset size is the LAYOUT box — unaffected by the `ov-pop` scale transform,
      // so it reports the settled badge size even on a mid-pop frame.
      return { w: b.offsetWidth, h: b.offsetHeight };
    });
    if (dim) {
      maxBadgeW = Math.max(maxBadgeW, dim.w);
      maxBadgeH = Math.max(maxBadgeH, dim.h);
    }
    // Multi-hit: an expected total that exceeds any single hit implies ≥2 summed.
    // We detect it structurally — a unit whose damage total is present AND the
    // beat revealed ≥2 hurt lines for it.
    const multi = await page.evaluate((names) => {
      const labels = Object.keys(names).sort((a, b) => b.length - a.length);
      const counts = {};
      for (const l of document.querySelectorAll(".beat-card .bc-line.bc-hurt")) {
        const txt = l.textContent ?? "";
        const label = labels.find((x) => txt.startsWith(x));
        if (label) counts[names[label]] = (counts[names[label]] ?? 0) + 1;
      }
      return Object.values(counts).some((c) => c >= 2);
    }, nameMap);
    if (multi && Object.keys(badges).length > 0) multiHitSeen = true;

    const curBeat = await beatIndex(page);
    const hasBadge = Object.keys(badges).length > 0;
    // Clear at the next beat: previous step had a badge in beat X, this step is
    // beat Y≠X and shows no badge for the previously-badged unit.
    if (prevBeat !== null && curBeat !== null && curBeat !== prevBeat && prevHadBadge && !hasBadge) {
      clearSeen = true;
    }
    prevBeat = curBeat;
    prevHadBadge = hasBadge;
  }

  // Persistence: re-walk to find a beat with a damage badge and assert the badge
  // is STILL present at that beat's last step (the read-pause dwells there).
  // beat.end = the step before the beat index changes (or the final step).
  let persistOk = false;
  let persistDetail = "no badged beat found";
  {
    let beatStartBadge = null; // { beat, unit } at first badge sighting
    for (let n = 0; n <= max; n++) {
      await stepTo(page, n);
      const badges = await damageBadges(page);
      const beat = await beatIndex(page);
      const units = Object.keys(badges);
      if (beat !== null && units.length > 0 && beatStartBadge === null) {
        beatStartBadge = { beat, unit: units[0], n };
        continue;
      }
      if (beatStartBadge !== null) {
        const nextBeat = await beatIndex(page);
        if (nextBeat !== beatStartBadge.beat) {
          // n-1 was the last step of the badged beat; re-check it.
          await stepTo(page, n - 1);
          const endBadges = await damageBadges(page);
          persistOk = endBadges[beatStartBadge.unit] !== undefined;
          persistDetail = `beat ${beatStartBadge.beat}: badge on ${beatStartBadge.unit} present at its last step (read-pause)=${persistOk}`;
          break;
        }
      }
    }
  }

  check(
    badgeSteps > 0,
    "overlay: damage badges actually render ON heroes during the battle",
    `${badgeSteps} step(s) show a −N damage badge on a hero card`,
  );
  check(
    mismatches === 0,
    "overlay: each hero's red −N badge EQUALS the summed revealed Hurt lines at every step",
    mismatches === 0 ? `all ${max + 1} steps consistent` : `${mismatches} mismatch(es); first: ${firstMismatch}`,
  );
  if (requireMultiHit) {
    check(
      multiHitSeen,
      "overlay: a multi-hit beat sums into one badge total (≥2 hurt lines, one badge)",
      multiHitSeen ? "found a beat with ≥2 hurts on one unit carrying a summed badge" : "no multi-hit beat reached",
    );
  }
  check(persistOk, "overlay: a damage badge PERSISTS through the read-pause (present at beat.end)", persistDetail);
  check(clearSeen, "overlay: badges CLEAR at the next beat (badged → next beat shows none)", clearSeen ? "badge present in a beat, gone at the next beat" : "no clear transition observed");
  // Minimum legible size (#065 damage-legibility fix). A badge below this is the
  // ~13×8px red-on-red pill that drowned in the hit-halo; the enlarged, darkened
  // badge clears it comfortably. Asserting a floor (not the exact size) keeps the
  // pill free to grow with a longer number but reddens the probe if someone
  // shrinks it back toward the unreadable original. Must-fail-first: lowering the
  // badge `font-size`/`padding` toward the old values drops it under this floor.
  check(
    maxBadgeW >= MIN_BADGE_W && maxBadgeH >= MIN_BADGE_H,
    `overlay: the damage badge is rendered at a legible size (≥${MIN_BADGE_W}×${MIN_BADGE_H}px), not the old red-on-red sliver`,
    `widest damage badge seen = ${Math.round(maxBadgeW)}×${Math.round(maxBadgeH)}px`,
  );
}

// ----------------------------------------------------------------------
// Slice 3 — the COIN: a pure coin-holder projection (coinHolderAt) drives a
// coin-flip beat card on a PairFaced and a PERSISTENT coin marker on the holder.
// The director reported "there is no coin visual as we've planned" — this probe
// reads the marker and the flip card straight off the live DOM and proves the
// coin is a visible, attributable state that changes hands on each new pairing.
// ----------------------------------------------------------------------
//
// We assert, sweeping the transport:
//   • a PairFaced beat opens a `.coin-card` flip card naming the pairing's winner
//     (data-coin-winner), and the same step paints exactly ONE `.coin-marker` —
//     on the card whose data-unit IS that winner (the coin landed on the holder);
//   • PERSIST — across every step from a pairing's flip up to (not including) the
//     next pairing's flip, exactly one marker shows and it stays on that holder;
//   • MOVE — at the next pairing the single marker is on the NEW holder (≠ the old
//     one for a genuine re-flip), so the coin visibly changed hands.
//
// Must-fail-first: STUB_COIN_OFF=1 strips the marker via injected CSS, so the
// "marker on the holder" assertions go red (0 markers) — proving the checks bind
// to the real DOM, not a tautology. Run once with it set (expect FAIL on those),
// once without (expect PASS). The flip-card check is independent of the stub.

/** The coin state at the current step: the flip card's declared winner (or null
 * when no coin card is open) and the set of data-units wearing a VISIBLE
 * `.coin-marker`. Visibility (not mere DOM presence) is the truth — "if the coin
 * isn't visibly on a hero, it's not done" — so a display:none stub genuinely
 * reds the marker checks (must-fail-first), and a painted coin passes them. */
const coinState = (page) =>
  page.evaluate(() => {
    const card = document.querySelector(".coin-card");
    const winner = card ? card.getAttribute("data-coin-winner") : null;
    const markers = [];
    for (const m of document.querySelectorAll(".unit[data-unit] .coin-marker")) {
      const cs = getComputedStyle(m);
      const r = m.getBoundingClientRect();
      const visible = cs.display !== "none" && cs.visibility !== "hidden" && Number(cs.opacity) > 0 && r.width > 0 && r.height > 0;
      if (visible) markers.push(m.closest(".unit[data-unit]").getAttribute("data-unit"));
    }
    return { winner, markers };
  });

async function coinVisibleAndChangesHands(page, { stubbed }) {
  const max = await maxStep(page);

  // Walk every step; record at each: the open coin-card winner (if any) and the
  // marker-bearing units. From this we reconstruct the holder timeline.
  const timeline = [];
  for (let n = 0; n <= max; n++) {
    await stepTo(page, n);
    const beat = await beatIndex(page);
    const { winner, markers } = await coinState(page);
    timeline.push({ n, beat, winner, markers });
  }

  // (a) The flip card: every PairFaced step that opens a coin-card declares a
  // winner. There must be ≥2 such cards across the battle (a coin that re-flips).
  const flips = timeline.filter((t) => t.winner !== null);
  check(
    flips.length >= 2,
    "coin: a PairFaced opens a coin-flip card on ≥2 pairings (the coin re-flips)",
    `${flips.length} coin-flip card step(s); winners=[${[...new Set(flips.map((f) => f.winner))].join(", ")}]`,
  );

  // (b) On each flip step the coin LANDS on the holder: exactly one marker, on
  // the declared winner. (Skipped under the stub — the marker is intentionally
  // off there, which is what makes the next checks fail-first.)
  let landMismatches = 0;
  let firstLand = "";
  for (const f of flips) {
    const ok = f.markers.length === 1 && f.markers[0] === f.winner;
    if (!ok && firstLand === "") firstLand = `step ${f.n}: winner=${f.winner} markers=[${f.markers.join(",")}]`;
    if (!ok) landMismatches++;
  }
  check(
    landMismatches === 0,
    "coin: the flip card's winner wears the single coin marker (the coin lands on the holder)",
    landMismatches === 0
      ? `all ${flips.length} flip step(s) land the marker on the winner`
      : `${landMismatches} mismatch(es); first: ${firstLand}`,
  );

  // (c) PERSIST + (d) MOVE. The holder is the most recent flip's winner; the
  // marker must equal exactly that holder at EVERY step after the first flip,
  // through intervening strikes, and switch at the next flip. We derive the
  // expected holder by carrying the last winner forward and compare to the marker
  // set at each step.
  let holder = null;
  let persistMismatches = 0;
  let firstPersist = "";
  let strikeStepsHeld = 0; // steps that are NOT a flip yet still show the marker (persistence proof)
  let moveSeen = false;
  let prevHolder = null;
  for (const t of timeline) {
    if (t.winner !== null) {
      if (holder !== null && t.winner !== holder) moveSeen = true; // the coin changed hands
      prevHolder = holder;
      holder = t.winner;
    }
    if (holder === null) continue; // before the first pairing — no marker expected
    const ok = t.markers.length === 1 && t.markers[0] === holder;
    if (!ok && firstPersist === "")
      firstPersist = `step ${t.n}: expected holder=${holder} markers=[${t.markers.join(",")}]`;
    if (!ok) persistMismatches++;
    if (ok && t.winner === null) strikeStepsHeld++; // held steady on a non-flip (intervening) step
  }

  check(
    persistMismatches === 0,
    "coin: the marker sits on the holder at EVERY step and stays through intervening strikes (persists)",
    persistMismatches === 0
      ? `holder consistent across all ${timeline.length} steps; ${strikeStepsHeld} intervening (non-flip) step(s) held the marker steady`
      : `${persistMismatches} mismatch(es); first: ${firstPersist}`,
  );
  check(
    strikeStepsHeld > 0,
    "coin: the marker persists across a pairing's strikes (≥1 non-flip step holds it)",
    `${strikeStepsHeld} non-flip step(s) carried the marker on the same holder`,
  );
  check(
    moveSeen,
    "coin: the marker MOVES to a new holder on a re-flip (changes hands)",
    moveSeen ? `coin changed hands (e.g. ${prevHolder} → ${holder})` : "no re-flip to a different holder observed",
  );
}

// ----------------------------------------------------------------------
// Item 1 (#065 feedback) — the hero OVERLAY BADGES must animate ONLY the newly
// appearing badge, exactly as the card LINES were fixed (defect A). The overlay
// layer re-renders its whole HTML each step, so arming `ov-pop` on every
// `.ov-badge` re-ran the pop on ALL of them whenever a new badge appeared (the
// director's "all of them play the fade in when a new one appears"). We read each
// badge's computed `animation-name`: when a SECOND badge lands on a hero, the
// first (already-shown) badge must read `none` and only the new one `ov-pop`.
// Mirror of defectALineAnimation, for badges. Must FAIL pre-fix (every badge
// reads `ov-pop` each step), PASS after.
// ----------------------------------------------------------------------

/** Per-badge motion state on every hero at the current step: the unit's
 * data-unit, the badge's text (its stable identity), and its computed
 * animation-name (the signal it is mid-reveal vs static). */
const badgeStates = (page) =>
  page.evaluate(() => {
    const out = [];
    for (const u of document.querySelectorAll(".side .unit[data-unit]")) {
      const unit = u.getAttribute("data-unit");
      for (const b of u.querySelectorAll(".ov-layer .ov-badge")) {
        out.push({ unit, text: (b.textContent ?? "").trim(), anim: getComputedStyle(b).animationName });
      }
    }
    return out;
  });

async function defectABadgeAnimation(page) {
  const max = await maxStep(page);

  // Walk to two consecutive steps inside one beat where a hero's badge COUNT
  // grows (a new badge streams in alongside ≥1 already-shown badge). The big
  // battle lands a damage badge then a status/heal badge on the same unit within
  // a beat — exactly the second-badge-appears moment.
  let found = null;
  for (let n = 0; n < max; n++) {
    await stepTo(page, n);
    const beatN = await beatIndex(page);
    const before = await badgeStates(page);
    if (beatN === null) continue;
    await stepTo(page, n + 1);
    const beatN1 = await beatIndex(page);
    const after = await badgeStates(page);
    if (beatN1 !== beatN) continue;
    // A unit that had ≥1 badge at N and gained ≥1 more at N+1.
    const cntBefore = (u) => before.filter((b) => b.unit === u).length;
    const cntAfter = (u) => after.filter((b) => b.unit === u).length;
    const grower = [...new Set(after.map((b) => b.unit))].find((u) => cntBefore(u) >= 1 && cntAfter(u) === cntBefore(u) + 1);
    if (grower) {
      found = { n, unit: grower, before: before.filter((b) => b.unit === grower), after: after.filter((b) => b.unit === grower) };
      break;
    }
  }

  if (!found) {
    check(false, "item-1: located a hero who gains a 2nd badge while keeping a 1st", "no such consecutive step found");
    return;
  }

  const beforeTexts = new Set(found.before.map((b) => b.text));
  const newBadges = found.after.filter((b) => !beforeTexts.has(b.text));
  const oldBadges = found.after.filter((b) => beforeTexts.has(b.text));

  check(
    newBadges.length >= 1 && oldBadges.length >= 1,
    "item-1: a new badge appears on a hero that already wears ≥1 badge",
    `unit ${found.unit}: prior=[${found.before.map((b) => b.text).join(", ")}] → now=[${found.after.map((b) => b.text).join(", ")}] (step ${found.n}→${found.n + 1})`,
  );

  // The crux: the ALREADY-SHOWN badges must NOT re-run their pop. Pre-fix every
  // badge re-armed `ov-pop` on the re-render → old badges read "ov-pop" → FAIL.
  const reanimated = oldBadges.filter((b) => b.anim !== "none");
  check(
    reanimated.length === 0,
    "item-1: already-shown badges do NOT re-animate when a new badge appears",
    reanimated.length === 0
      ? `${oldBadges.length} prior badge(s) static (animation-name: none)`
      : `badges [${reanimated.map((b) => `"${b.text}":${b.anim}`).join(", ")}] re-ran their pop`,
  );

  // And the new badge DID animate in — the reveal still happens, just scoped.
  check(
    newBadges.every((b) => b.anim === "ov-pop"),
    "item-1: the newly appearing badge DOES animate in (ov-pop)",
    newBadges.map((b) => `"${b.text}":${b.anim}`).join(", "),
  );
}

// ----------------------------------------------------------------------
// Item 3 (#065 feedback) — the displayed hero NAME drops the "A1:"/"B3:" instance
// prefix. The card LABEL (.uname) must never match /^[AB]\d+:/, while the card's
// `data-unit` attr still carries the FULL id (the data-layer disambiguator the
// probes, kernel ids, and cause-trace keying depend on). Must FAIL pre-fix on a
// battle with a name collision (today the colliding unit shows "A1:Brawler"); the
// big battle's rosters share names, so the prefix shows pre-fix.
// ----------------------------------------------------------------------

async function namePrefixStripped(page) {
  const max = await maxStep(page);
  // Sweep every step; collect each board card's displayed label and its data-unit.
  const PREFIX = /^[A-Z]\+?\d+:/; // A1: / B3: / A+1: (a summon) — the instance prefix
  let labelsSeen = 0;
  let prefixed = 0;
  let firstPrefixed = "";
  let idMismatch = 0; // a card whose data-unit LOST its prefix (would mean we broke the data layer)
  let collisionConfirmed = false; // ≥2 cards sharing a display label (so the prefix WOULD have shown)
  for (let n = 0; n <= max; n++) {
    await stepTo(page, n);
    const cards = await page.evaluate(() => {
      const out = [];
      for (const u of document.querySelectorAll(".side .unit[data-unit]")) {
        out.push({ id: u.getAttribute("data-unit"), label: (u.querySelector(".uname")?.textContent ?? "").trim() });
      }
      return out;
    });
    const labels = cards.map((c) => c.label);
    if (new Set(labels).size < labels.length) collisionConfirmed = true;
    for (const c of cards) {
      labelsSeen++;
      if (PREFIX.test(c.label)) {
        prefixed++;
        if (firstPrefixed === "") firstPrefixed = `step ${n}: label="${c.label}" data-unit="${c.id}"`;
      }
      // The data-unit MUST keep the full prefixed id — a bare id here means the
      // strip leaked into the data layer (probe/selector/cause-trace breakage).
      if (!PREFIX.test(c.id)) idMismatch++;
    }
  }

  check(labelsSeen > 0, "item-3: swept board card labels across the battle", `${labelsSeen} card-label reads`);
  check(
    collisionConfirmed,
    "item-3: the battle has a NAME COLLISION (two cards share a display label) — so a prefix WOULD show if not stripped",
    collisionConfirmed ? "≥2 cards share a display label" : "no collision seen (probe can't prove the strip)",
  );
  check(
    prefixed === 0,
    "item-3: no displayed card label carries an A1:/B3: instance prefix",
    prefixed === 0 ? `all ${labelsSeen} labels are bare names` : `${prefixed} prefixed label(s); first: ${firstPrefixed}`,
  );
  check(
    idMismatch === 0,
    "item-3: every card's data-unit STILL carries the full instance id (the data layer is intact)",
    idMismatch === 0 ? "all data-unit attrs keep their prefix" : `${idMismatch} card(s) lost the data-unit prefix`,
  );
}

// ----------------------------------------------------------------------
// Item 4 (#065 feedback) — a DISTINCT death animation plays at the death-reveal
// step. At the step a Death is revealed the dying card carries `.dying-new` and a
// computed `animation-name` of `death-strike`; it still ends on the grey+✕ state
// (.dying + a visible .dying-x). Under reduced-motion the animation is skipped
// but the grey+✕ end-state stays. Must FAIL pre-fix (no death-strike anim exists).
// ----------------------------------------------------------------------

/** The dying cards at the current step: data-unit, whether it wears `.dying-new`,
 * its computed animation-name, and whether its ✕ end-mark is painted. */
const dyingStates = (page) =>
  page.evaluate(() => {
    const out = [];
    for (const u of document.querySelectorAll(".side .unit.dying[data-unit]")) {
      const x = u.querySelector(".dying-x");
      const xVisible = x ? getComputedStyle(x).display !== "none" && x.getBoundingClientRect().width > 0 : false;
      out.push({
        unit: u.getAttribute("data-unit"),
        isNew: u.classList.contains("dying-new"),
        anim: getComputedStyle(u).animationName,
        grey: getComputedStyle(u).filter.includes("grayscale") || Number(getComputedStyle(u).opacity) < 1,
        xVisible,
      });
    }
    return out;
  });

async function deathAnimationPlays(page, { reducedMotion }) {
  const max = await maxStep(page);
  let revealStepFound = false;
  let animPlayed = false;
  let firstReveal = "";
  let endStateHeld = true;
  for (let n = 0; n <= max; n++) {
    await stepTo(page, n);
    const dying = await dyingStates(page);
    const fresh = dying.filter((d) => d.isNew);
    if (fresh.length > 0) {
      revealStepFound = true;
      if (firstReveal === "") firstReveal = `step ${n}: ${fresh.map((d) => d.unit).join(", ")}`;
      // At the reveal step the card must show the grey+✕ end-state regardless of
      // motion, and (motion on) be running death-strike.
      for (const d of fresh) {
        if (!(d.grey && d.xVisible)) endStateHeld = false;
        if (d.anim === "death-strike") animPlayed = true;
      }
    }
  }

  check(revealStepFound, "item-4: a death-reveal step (.dying-new) occurs in the battle", revealStepFound ? `e.g. ${firstReveal}` : "no .dying-new step seen");
  check(
    endStateHeld,
    "item-4: the dying card holds the grey + ✕ end-state at the reveal step",
    endStateHeld ? "grey + visible ✕ at every reveal step" : "a reveal step lacked grey/✕",
  );
  if (reducedMotion) {
    check(
      !animPlayed,
      "item-4 (reduced-motion): the death animation is SUPPRESSED — end-state stays grey + ✕",
      animPlayed ? "death-strike ran under reduced-motion" : "no death-strike under reduced-motion (end-state intact)",
    );
  } else {
    check(
      animPlayed,
      "item-4: the dying card plays the distinct death-strike animation at the reveal step",
      animPlayed ? "death-strike armed at the reveal step" : "no death-strike animation-name seen (still static grey+✕)",
    );
  }
}

{
  const { ctx, page } = await openRun(browser, plainShopRun(), DESKTOP);
  await intoBattle(page);
  await defectALineAnimation(page);
  await ctx.close();
}
{
  // Item 1 — the badge re-animation defect. The big battle stacks a damage badge
  // then a status/heal badge on one unit within a beat (a 2nd badge appears).
  const { ctx, page } = await openRun(browser, bigBattleRun(), DESKTOP);
  await intoBattle(page);
  await defectABadgeAnimation(page);
  await ctx.close();
}
{
  // Item 3 — the displayed name has no instance prefix. The big battle's rosters
  // share names (a collision), so a prefix WOULD show pre-fix.
  const { ctx, page } = await openRun(browser, bigBattleRun(), DESKTOP);
  await intoBattle(page);
  await namePrefixStripped(page);
  await ctx.close();
}
{
  // Item 4 — the death animation plays at the death-reveal step (motion on).
  const { ctx, page } = await openRun(browser, bigBattleRun(), DESKTOP);
  await intoBattle(page);
  await deathAnimationPlays(page, { reducedMotion: false });
  await ctx.close();
}
{
  // Item 4 — under reduced-motion the animation is suppressed but the grey+✕
  // end-state stays. Emulate the media query before the battle renders.
  const { ctx, page } = await openRun(browser, bigBattleRun(), DESKTOP);
  await page.emulateMedia({ reducedMotion: "reduce" });
  await intoBattle(page);
  await deathAnimationPlays(page, { reducedMotion: true });
  await ctx.close();
}
{
  const { ctx, page } = await openRun(browser, plainShopRun(), DESKTOP);
  await intoBattle(page);
  await defectBHitTruth(page);
  await ctx.close();
}
{
  // The plain shop battle: appear/increment-in-sync, persist, clear.
  const { ctx, page } = await openRun(browser, plainShopRun(), DESKTOP);
  await intoBattle(page);
  await badgesInSyncWithLines(page, { requireMultiHit: false });
  await ctx.close();
}
{
  // The Duelist battle lands TWO Hurts on one defender in a single Strike beat
  // — the multi-hit summing the plain battle never produces.
  const { ctx, page } = await openRun(browser, duelistRun(), DESKTOP);
  await intoBattle(page);
  await badgesInSyncWithLines(page, { requireMultiHit: true });
  await ctx.close();
}
{
  // The coin (#065 slice 3). The big battle (5 vs a smaller side) kills units, so
  // the front advances and the coin re-flips to new pairings — exactly the
  // "changes hands" behaviour. STUB_COIN_OFF=1 hides the marker via injected CSS
  // so the must-fail-first run reddens the marker checks; without it they pass.
  const stubbed = process.env.STUB_COIN_OFF === "1";
  const { ctx, page } = await openRun(browser, bigBattleRun(), DESKTOP);
  if (stubbed) await page.addStyleTag({ content: ".coin-marker { display: none !important; }" });
  await intoBattle(page);
  await coinVisibleAndChangesHands(page, { stubbed });
  await ctx.close();
}

await browser.close();
disarm();
finish("probe-motion");
