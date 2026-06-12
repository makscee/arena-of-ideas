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

  // -- 2. the unit entries draw the shared slice-1 card --
  const skeleton = await page.$eval("#codex-sec-units .codex-entry .unit", (card) =>
    ["svg.shape", ".uname", ".unums .hp", ".unums .pwr", ".chips"]
      .filter((w) => card.querySelector(w) !== null)
      .join(","),
  );
  check(
    skeleton === "svg.shape,.uname,.unums .hp,.unums .pwr,.chips",
    `${tag} codex unit entry carries the full shared-card skeleton`,
    skeleton,
  );
  // Status colour identity: two status cards, two hues.
  const hues = await page.$$eval("#codex-sec-statuses .codex-entry", (els) =>
    els.slice(0, 2).map((el) => el.style.getPropertyValue("--codex-hue")),
  );
  check(
    hues.length === 2 && hues[0] !== "" && hues[0] !== hues[1],
    `${tag} status cards carry distinct colour identities`,
    `hues=[${hues}]`,
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

await browser.close();
disarm();
finish("probe-codex-grid");
