// Codex acceptance: four shared entity cards, non-card grammar rows, links and responsive layout.
import { BASE, DESKTOP, PHONE, armGuard, check, finish, launch } from "./lib.mjs";
const disarm = armGuard();
const browser = await launch();

for (const [viewport, tag] of [[PHONE, "375px"], [DESKTOP, "desktop"]]) {
  const ctx = await browser.newContext({ viewport, hasTouch: viewport.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.goto(`${BASE}#codex/unit/Necromancer`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#codex-view:not([hidden])");

  const cols = async (selector) => page.$$eval(selector, (els) => new Set(els.filter((el) => !el.hidden).map((el) => Math.round(el.getBoundingClientRect().x))).size);
  check((await cols("#codex-sec-units .codex-entry")) >= (viewport === PHONE ? 2 : 3), `${tag} Unit grid is responsive`);
  check(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), `${tag} Codex has no horizontal overflow`);

  for (const kind of ["unit", "ability", "status", "summon"]) {
    const count = await page.$$eval(`[data-card-entity="${kind}"]`, (els) => els.length);
    check(count > 0, `${tag} ${kind} uses the shared production chassis`, `${count}`);
  }
  const cardKinds = await page.$$eval("[data-card-entity]", (els) => [...new Set(els.map((el) => el.dataset.cardEntity))].sort().join(","));
  check(cardKinds === "ability,status,summon,unit", `${tag} no fifth domain entity bears a card`, cardKinds);
  check(await page.$$eval("#codex-sec-parts [data-card-entity]", (els) => els.length) === 0, `${tag} Trigger/Selector/Effect are non-card rows`);
  check(await page.$$eval("#codex-sec-parts .codex-part-row", (els) => els.length) >= 30, `${tag} complete grammar vocabulary remains available`);
  check(await page.$$eval("[data-card-entity] [data-card-entity]", (els) => els.length) === 0, `${tag} cards do not nest`);

  const search = page.locator(".codex-search");
  await search.fill("Venomancer");
  check(await page.$eval("#codex-unit-Venomancer", (el) => !el.hidden), `${tag} filter keeps matching Unit`);
  check(await page.$eval("#codex-unit-Brawler", (el) => el.hidden), `${tag} filter hides non-match`);
  await search.fill("");

  const term = page.locator('#codex-sec-units .codex-termref[href^="#codex/part/"]').first();
  const href = await term.getAttribute("href");
  await term.click();
  await page.waitForTimeout(500);
  check(href !== null && await page.$(`${href.replace("#codex/part/", "#codex-part-").replaceAll("/", "-")}.codex-part-row`) !== null, `${tag} grammar link lands on a non-card row`);

  const statusRef = page.locator('#codex-sec-units [data-entity-ref^="status:"]').first();
  if (await statusRef.count()) {
    await statusRef.evaluate((el) => el.click());
    await page.waitForSelector("#inspect-overlay:not([hidden])");
    check(await page.$eval("#inspect-overlay .ins-entity-kind", (el) => el.textContent.trim()) === "status", `${tag} Status reference opens shared inspector`);
  }
  await ctx.close();
}

await browser.close();
disarm();
finish("probe-codex-grid");
