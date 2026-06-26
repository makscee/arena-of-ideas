// PRD #077 slice 3 — the season-history screen, against the LIVE app at desktop
// and 375×667. The acceptance, end to end:
//   1. Reachable from a title button; renders the completed-seasons list.
//   2. The list shows each archived season's number + content version + champion.
//   3. Opening a season shows its final tower, floor by floor, champion marked.
//   4. Empty archive reads as its empty state (not an error).
//   5. Geometry: usable at 375px — rows, the back control, floor cards all fit,
//      visible at real size, ≥44px taps, nothing overflows sideways.
//
// History reads the DEVICE's local archive (aoi.season-archive.v1) — public, no
// auth. The season transition (slice 2) writes it; here the probe seeds the
// archive directly in localStorage (what slice 2 will write) so the READ surface
// is exercised end to end without a transition.

import { BASE, DESKTOP, PHONE, armGuard, box, check, finish, launch } from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

const noHorizontalOverflow = (page) =>
  page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1);

// A two-season archive, the SeasonArchiveData shape slice 2 will write:
// season 1 has a two-floor tower (champion on floor 2), season 2 a one-floor one.
const ARCHIVE = {
  seasons: [
    {
      season: 1,
      version: 1,
      finalTower: {
        bosses: {
          1: { runId: "alpha-7", round: 1, seq: 2, team: [{ name: "Grunt", base: { hp: 5, pwr: 1 } }] },
          2: { runId: "beta-3", round: 2, seq: 1, team: [{ name: "Ogre", base: { hp: 9, pwr: 3 } }] },
        },
        pools: {},
      },
    },
    {
      season: 2,
      version: 2,
      finalTower: {
        bosses: {
          1: { runId: "gamma-1", round: 1, seq: 0, team: [{ name: "Sprite", base: { hp: 3, pwr: 2 } }] },
        },
        pools: {},
      },
    },
  ],
};
const ARCHIVE_JSON = JSON.stringify(ARCHIVE, null, 2) + "\n";

async function openHistory(viewport, { seed }) {
  const ctx = await browser.newContext({ viewport, hasTouch: viewport.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.addInitScript(
    ([key, val, seedIt]) => {
      localStorage.removeItem("aoi.run.v1");
      if (seedIt) localStorage.setItem(key, val);
      else localStorage.removeItem(key);
    },
    ["aoi.season-archive.v1", ARCHIVE_JSON, seed],
  );
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  await page.click("#title-history");
  await page.waitForSelector("#history-view:not([hidden])");
  return { ctx, page };
}

const rowSeasons = (page) =>
  page.$$eval("#history-list .history-row .history-row-head", (els) => els.map((e) => e.textContent));

async function seededScenario(viewport, tag) {
  const { ctx, page } = await openHistory(viewport, { seed: true });

  // ---- 1. reachable, renders the seasons list -----------------------------
  check(await page.locator(".history-title").isVisible(), `${tag} the history screen carries its own title`);
  await page.waitForFunction(() => document.querySelectorAll("#history-list .history-row").length > 0);
  const seasons = await rowSeasons(page);
  // ---- 2. each archived season is listed (newest first) -------------------
  check(seasons.includes("Season 1") && seasons.includes("Season 2"), `${tag} both archived seasons are listed`, JSON.stringify(seasons));
  check(seasons[0] === "Season 2", `${tag} newest season ranks first`, seasons[0]);
  const listText = await page.locator("#history-list").innerText();
  check(listText.includes("content v1") && listText.includes("content v2"), `${tag} each row shows its content version`);
  check(listText.includes("beta-3") && listText.includes("gamma-1"), `${tag} each row shows its champion`);

  // ---- geometry (real visibility + sane sizes) ----------------------------
  check(await noHorizontalOverflow(page), `${tag} no horizontal overflow on the list`);
  const rowBox = await box(page, "#history-list .history-row >> nth=0");
  check(rowBox.height >= 44, `${tag} a season row is a ≥44px tap`, `${Math.round(rowBox.height)}px`);
  check(
    rowBox.x >= 0 && rowBox.x + rowBox.width <= viewport.width + 1,
    `${tag} season row sits within the viewport`,
    `x=${Math.round(rowBox.x)} right=${Math.round(rowBox.x + rowBox.width)} vw=${viewport.width}`,
  );

  // ---- 3. open a season → its final tower, floor by floor, champion marked -
  // Season 1 has the two-floor tower — open it to see both floors.
  await page.evaluate(() => {
    [...document.querySelectorAll("#history-list .history-row")]
      .find((r) => r.querySelector(".history-row-head")?.textContent === "Season 1")
      .click();
  });
  await page.waitForSelector("#history-detail:not([hidden])");
  check(await page.locator("#history-list").isHidden(), `${tag} opening a season hides the list`);
  const detailText = await page.locator("#history-detail").innerText();
  check(detailText.includes("Season 1"), `${tag} the detail names the season`);
  check(detailText.includes("content v1"), `${tag} the detail names the content version`);
  const floorCount = await page.locator("#history-detail .history-floor").count();
  check(floorCount === 2, `${tag} the final tower lists every floor`, `${floorCount} floors`);
  const firstFloor = await page.locator("#history-detail .history-floor").first().innerText();
  check(firstFloor.includes("Floor 2") && firstFloor.includes("★ champion"), `${tag} the champion floor is first and marked`, firstFloor);
  check(detailText.includes("Floor 1") && detailText.includes("alpha-7"), `${tag} the lower floor reads its boss`);
  check(await noHorizontalOverflow(page), `${tag} no horizontal overflow on the detail`);

  // ---- back to the list ---------------------------------------------------
  await page.click("#history-back");
  await page.waitForSelector("#history-list:not([hidden])");
  check(await page.locator("#history-detail").isHidden(), `${tag} back returns to the list`);

  await ctx.close();
}

async function emptyScenario(viewport, tag) {
  // ---- 4. an empty archive reads as its empty state, not an error ----------
  const { ctx, page } = await openHistory(viewport, { seed: false });
  await page.waitForSelector("#history-list .history-empty");
  check(await page.locator("#history-list .history-empty").isVisible(), `${tag} empty archive shows the empty state`);
  check((await page.locator("#history-list .history-row").count()) === 0, `${tag} empty archive lists no rows`);
  check(await noHorizontalOverflow(page), `${tag} empty archive: no horizontal overflow`);
  await ctx.close();
}

for (const [viewport, tag] of [
  [DESKTOP, "desktop"],
  [PHONE, "375px"],
]) {
  await seededScenario(viewport, tag);
  await emptyScenario(viewport, tag);
}

await browser.close();
disarm();
finish("probe-history");
