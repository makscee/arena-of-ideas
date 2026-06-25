// PRD #015 slice 2 — the codex card grid. Pins, against the LIVE app:
//  1. Grid layout: units sit multi-column on a desk and exactly two abreast
//     at 375px; statuses/rules grids are multi-column on a desk, single at
//     375px; no horizontal overflow at either width.
//  2. The unit entries draw the SHARED card (shape art, name, framed hp/pwr,
//     chips) — the slice-1 component, not a codex-local lookalike; status
//     cards carry a per-status colour identity (distinct swatch hues).
//  3. The search filter hides non-matching cards (and emptied sections) and
//     restores them on clear.
//  4. Deep links land on the right card — #codex/unit/X, #codex/status/X and
//     #codex/rule/X via hash navigation, plus a cold-load URL — scrolled into
//     view and flash-highlighted.
//  5. Tap targets: every deep-link anchor and the search input give ≥44px.

import { DESKTOP, PHONE, armGuard, check, finish, launch } from "./lib.mjs";

const BASE = process.env.AOI_BASE_URL ?? "http://localhost:5280";

const disarm = armGuard();
const browser = await launch();

/** Fresh page, no stored run, codex tab opened. */
async function openCodex(viewport, hash = "") {
  const ctx = await browser.newContext({ viewport, hasTouch: viewport.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.addInitScript(() => localStorage.removeItem("aoi.run.v1"));
  await page.goto(BASE + hash, { waitUntil: "domcontentloaded" });
  if (hash === "") {
    // The app lands on the title (#015 slice 3); its Codex entry navigates.
    await page.waitForSelector("#title-view:not([hidden])");
    await page.click("#title-codex");
  }
  await page.waitForSelector("#codex-view:not([hidden])");
  return { ctx, page };
}

/** Rounded y of each entry in a section's grid — first-row width = column count. */
const columnCount = (page, sec) =>
  page.$$eval(`${sec} .codex-entry`, (els) => {
    const ys = els.map((el) => Math.round(el.getBoundingClientRect().y));
    return ys.filter((y) => y === ys[0]).length;
  });

const noHorizontalOverflow = (page) =>
  page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1);

/** Wait until the element's box is inside the viewport (smooth scroll is async). */
const waitInView = (page, sel) =>
  page.waitForFunction((s) => {
    const el = document.querySelector(s);
    if (!el) return false;
    const r = el.getBoundingClientRect();
    return r.top >= -1 && r.bottom <= window.innerHeight + 1;
  }, sel);

// ---------- grid layout, shared card, search — both widths ------------------

for (const [viewport, tag] of [
  [PHONE, "375px"],
  [DESKTOP, "desktop"],
]) {
  const { ctx, page } = await openCodex(viewport);

  // -- 1. column counts and overflow --
  const unitCols = await columnCount(page, "#codex-sec-units");
  const statusCols = await columnCount(page, "#codex-sec-statuses");
  const ruleCols = await columnCount(page, "#codex-sec-rules");
  if (viewport === PHONE) {
    check(unitCols === 2, `${tag} unit grid sits two abreast`, `cols=${unitCols}`);
    check(statusCols === 1, `${tag} status grid is single-column`, `cols=${statusCols}`);
    check(ruleCols === 1, `${tag} rules grid is single-column`, `cols=${ruleCols}`);
  } else {
    check(unitCols >= 3, `${tag} unit grid is multi-column`, `cols=${unitCols}`);
    check(statusCols >= 2, `${tag} status grid is multi-column`, `cols=${statusCols}`);
    check(ruleCols >= 2, `${tag} rules grid is multi-column`, `cols=${ruleCols}`);
  }
  check(await noHorizontalOverflow(page), `${tag} codex has no horizontal overflow`);

  // -- 2. unit AND status entries draw the ONE shared card (#078) --
  // The shared-card skeleton string a card must carry to BE the shared card.
  const SHARED_SKELETON = "svg.shape,.uname,.unums .hp,.unums .pwr,.chips";
  // Null-safe: a section whose entry has NO `.unit` (the old status lookalike)
  // returns "(no .unit)" so the assertion fails cleanly instead of throwing.
  const skeletonOf = async (sel) => {
    const el = await page.$(sel);
    if (el === null) return "(no .unit)";
    return el.evaluate((card) =>
      ["svg.shape", ".uname", ".unums .hp", ".unums .pwr", ".chips"]
        .filter((w) => card.querySelector(w) !== null)
        .join(","),
    );
  };
  const widthOf = async (sel) => {
    const el = await page.$(sel);
    return el === null ? -1 : el.evaluate((e) => e.getBoundingClientRect().width);
  };
  const unitSkeleton = await skeletonOf("#codex-sec-units .codex-entry .unit");
  check(
    unitSkeleton === SHARED_SKELETON,
    `${tag} codex unit entry carries the full shared-card skeleton`,
    unitSkeleton,
  );
  // The Status card is the SAME card (#078): same .unit skeleton, not the old
  // .codex-status-entry lookalike. This must FAIL against the old codex, whose
  // status entry had a .codex-swatch + .codex-entry-name and NO .unit / svg.shape
  // / .unums — the skeleton string would have come back empty (no `.unit` to
  // match) instead of the shared skeleton.
  const statusSkeleton = await skeletonOf("#codex-sec-statuses .codex-entry .unit");
  check(
    statusSkeleton === SHARED_SKELETON,
    `${tag} codex STATUS entry carries the full shared-card skeleton (#078)`,
    statusSkeleton,
  );
  check(
    (await page.$("#codex-sec-statuses .codex-entry .unit.is-status")) !== null,
    `${tag} status card routes through unitCardHtml as kind=status (#078)`,
  );
  // The status lookalike is gone: no card-shaped .codex-swatch survives.
  check(
    (await page.$("#codex-sec-statuses .codex-swatch")) === null,
    `${tag} the old .codex-status-entry swatch lookalike is removed (#078)`,
  );
  // ONE fixed size: a status card is the SAME width as a unit card. Must FAIL
  // against the old codex where the unit card was max-width 8.5rem and the
  // status lookalike was a full-width text card — wholly different widths.
  const unitCardW = await widthOf("#codex-sec-units .codex-entry .unit");
  const statusCardW = await widthOf("#codex-sec-statuses .codex-entry .unit");
  check(
    Math.abs(statusCardW - unitCardW) <= 1,
    `${tag} codex status card shares the unit card's ONE fixed width (#078)`,
    `status ${statusCardW.toFixed(1)} vs unit ${unitCardW.toFixed(1)}`,
  );

  // -- 3. search filters the grid, sections fold, clear restores --
  await page.fill(".codex-search", "necro");
  check(await page.locator("#codex-unit-Necromancer").isVisible(), `${tag} filter keeps the matching unit card`);
  check(await page.locator("#codex-unit-Brawler").isHidden(), `${tag} filter hides a non-matching unit card`);
  check(await page.locator("#codex-sec-statuses").isHidden(), `${tag} filter folds an emptied section`);
  await page.fill(".codex-search", "");
  check(await page.locator("#codex-unit-Brawler").isVisible(), `${tag} clearing the filter restores the grid`);
  check(await page.locator("#codex-sec-statuses").isVisible(), `${tag} clearing the filter restores the sections`);

  // -- 5. tap targets: ≥44px anchors (one per section) and search input --
  for (const sec of ["units", "statuses", "rules"]) {
    const b = await page.locator(`#codex-sec-${sec} .codex-anchor`).first().boundingBox();
    check(
      b !== null && b.width >= 44 && b.height >= 44,
      `${tag} ${sec} deep-link anchor is a ≥44px target`,
      b === null ? "no box" : `${b.width.toFixed(0)}×${b.height.toFixed(0)}`,
    );
  }
  const sb = await page.locator(".codex-search").boundingBox();
  check(sb !== null && sb.height >= 44, `${tag} search input is a ≥44px target`, `h=${sb?.height.toFixed(0)}`);

  // -- 4. deep links via hash navigation: a status and a rule --
  for (const [frag, id] of [
    ["codex/status/Poison", "codex-status-Poison"],
    ["codex/rule/fusion", "codex-rule-fusion"],
  ]) {
    await page.evaluate((f) => (window.location.hash = `#${f}`), frag);
    await waitInView(page, `#${id}`);
    check(
      await page.$eval(`#${id}`, (el) => el.classList.contains("codex-highlight")),
      `${tag} deep link #${frag} reveals and highlights its card`,
    );
  }

  await ctx.close();
}

// ---------- cold-load deep link to a unit card ------------------------------

{
  const { ctx, page } = await openCodex(DESKTOP, "#codex/unit/Necromancer");
  await waitInView(page, "#codex-unit-Necromancer");
  check(
    await page.$eval("#codex-unit-Necromancer", (el) => el.classList.contains("codex-highlight")),
    "cold-load #codex/unit/Necromancer lands on the highlighted unit card",
  );
  check(
    await page.$eval("#codex-unit-Necromancer .unit svg.shape", (el) => el !== null),
    "the deep-linked unit card is the shared card (shape art present)",
  );
  await ctx.close();
}

// ---------- real tap on a unit anchor (not just bounding-box geometry) ------
// This is the Cass repro: without z-index on .codex-anchor the .unit card
// intercepts the pointer event and tap() times out.

{
  const { ctx, page } = await openCodex(PHONE);
  // Scroll the Brawler card into view so the tap lands correctly.
  await page.$eval("#codex-unit-Brawler", (el) => el.scrollIntoView({ block: "center" }));
  await page.waitForTimeout(200);
  await page.locator("#codex-unit-Brawler .codex-anchor").tap();
  const hash = await page.evaluate(() => location.hash);
  check(
    hash === "#codex/unit/Brawler",
    "tap on unit anchor updates location hash",
    `hash=${hash}`,
  );
  // Anchor glyph must be visible (not obscured by the card's opaque gradient).
  const anchorVisible = await page.locator("#codex-unit-Brawler .codex-anchor").isVisible();
  check(anchorVisible, "unit anchor glyph is visible (not occluded by card)");
  await ctx.close();
}

// ---------- deep link to a filter-hidden card clears the filter -------------
// Cass low: navigate() must reveal the card, not silently land nowhere.

{
  const { ctx, page } = await openCodex(DESKTOP);
  // Hide Brawler behind a search that matches only Necromancer.
  await page.fill(".codex-search", "necro");
  check(await page.locator("#codex-unit-Brawler").isHidden(), "pre-condition: Brawler hidden by filter");
  // Navigate via hash to Brawler while filter is active.
  await page.evaluate(() => (window.location.hash = "#codex/unit/Brawler"));
  // The card must become visible and highlighted — navigate() should clear the filter.
  await page.waitForFunction(() => {
    const el = document.querySelector("#codex-unit-Brawler");
    return el && !el.hidden;
  }, { timeout: 3000 });
  check(
    await page.locator("#codex-unit-Brawler").isVisible(),
    "deep link to filter-hidden card reveals the card (filter cleared)",
  );
  check(
    await page.$eval("#codex-unit-Brawler", (el) => el.classList.contains("codex-highlight")),
    "deep link to filter-hidden card highlights the target",
  );
  await ctx.close();
}

await browser.close();
disarm();
finish("probe-codex-grid");
