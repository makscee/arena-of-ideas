// AOI-60 acceptance probe: the production card ontology and shared inspector.
import { BASE, DESKTOP, PHONE, armGuard, check, finish, launch, lineFullRun, openRun, targetsRun } from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

for (const [viewport, tag] of [[DESKTOP, "desktop"], [PHONE, "phone"]]) {
  const ctx = await browser.newContext({ viewport, hasTouch: viewport.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.goto(`${BASE}#codex/unit/Venomancer`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#codex-view:not([hidden])");

  const kinds = await page.$$eval("#codex-container [data-card-entity]", (els) => [...new Set(els.map((el) => el.dataset.cardEntity))].sort());
  check(JSON.stringify(kinds) === JSON.stringify(["ability", "status", "summon", "unit"]), `${tag} Codex exposes exactly four card entity kinds`, kinds.join(","));
  check(await page.$$eval("#codex-sec-parts [data-card-entity]", (els) => els.length) === 0, `${tag} grammar atoms are non-card rows`);
  check(await page.$$eval("#codex-sec-parts [data-non-card]", (els) => els.length) > 0, `${tag} Trigger/Selector/Effect glossary rows are present`);
  check(await page.$$eval("[data-card-entity] [data-card-entity]", (els) => els.length) === 0, `${tag} cards never nest cards`);
  check(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), `${tag} Codex has no horizontal overflow`);

  for (const kind of ["unit", "ability", "status", "summon"]) {
    const card = page.locator(`#codex-container [data-card-entity="${kind}"]`).first();
    await card.evaluate((el) => el.click());
    await page.waitForSelector("#inspect-overlay:not([hidden])");
    check(await page.$eval("#inspect-overlay .ins-entity-kind", (el) => el.textContent.trim()) === kind, `${tag} ${kind} opens in shared inspector`);
    check(await page.$$eval(`#inspect-overlay > .unit-b.is-full[data-card-entity="${kind}"]`, (els) => els.length) === 1, `${tag} ${kind} inspector has exactly one FULL shared card`);
    check(await page.$$eval("#inspect-overlay [data-card-entity] [data-card-entity]", (els) => els.length) === 0, `${tag} ${kind} inspector contains no nested card`);
    await page.keyboard.press("Escape");
  }
  await ctx.close();
}

// Shop full/compact continuity and generated-name portrait stability.
const { ctx, page } = await openRun(browser, targetsRun(), DESKTOP);
const fullName = await page.$eval('#run-shop-row [data-card-entity="unit"]', (el) => el.dataset.entityName);
const lineName = await page.$eval('#run-line [data-card-entity="unit"]', (el) => el.dataset.entityName);
check(Boolean(fullName && lineName), "shop offers and current team use the shared Unit chassis");
check(await page.$$eval('#run-shop-row .unit:not(.unit-b),#run-line .unit:not(.unit-b)', (els) => els.length) === 0, "shop/team contain no legacy competing Unit cards");
const portraits = await page.$$eval('[data-card-entity="unit"]', (els) => els.slice(0, 8).map((el) => el.querySelector("svg.shape")?.innerHTML));
check(new Set(portraits.filter(Boolean)).size > 1, "shipped Unit portraits are distinctive and stable");

// Shop/team Unit inspection and an Ability reference both replace the one card.
await page.locator('#run-shop-row [data-card-entity="unit"]').first().click();
await page.waitForSelector('#inspect-overlay:not([hidden])');
check(await page.$$eval('#inspect-overlay > .unit-b.is-full[data-card-entity="unit"]', (els) => els.length) === 1, "shop Unit inspector has exactly one FULL shared Unit card");
const abilityRef = page.locator('#inspect-overlay [data-inspect-kind="ability"]').first();
await abilityRef.click();
check(await page.$$eval('#inspect-overlay > .unit-b.is-full[data-card-entity="ability"]', (els) => els.length) === 1, "Ability ref replaces the Unit card in the same overlay");
check(await page.$$eval('#inspect-overlay [data-card-entity] [data-card-entity]', (els) => els.length) === 0, "reference transition never nests cards");
await page.keyboard.press("Escape");
await ctx.close();

// A phone sheet's final content row scrolls fully above the persistent menu.
const phoneRun = await openRun(browser, targetsRun(), PHONE);
await phoneRun.page.locator('#run-shop-row [data-card-entity="unit"]').first().click();
await phoneRun.page.waitForSelector('#inspect-overlay.sheet:not([hidden])');
check(await phoneRun.page.$$eval('#inspect-overlay > .unit-b.is-full[data-card-entity="unit"]', (els) => els.length) === 1, "phone shop Unit inspector has exactly one FULL shared Unit card");
check(await phoneRun.page.$$eval('#inspect-overlay [data-card-entity] [data-card-entity]', (els) => els.length) === 0, "phone shop Unit inspector contains no nested card");
const lastInspectorRow = phoneRun.page.locator("#inspect-overlay > .ins-row:not([hidden]), #inspect-overlay > .ins-dim").last();
await lastInspectorRow.evaluate((el) => el.scrollIntoView({ block: "end" }));
const [lastRowBox, menuBox] = await Promise.all([
  lastInspectorRow.boundingBox(),
  phoneRun.page.locator("#run-menu-button").boundingBox(),
]);
check(lastRowBox !== null && menuBox !== null, "phone inspector last row and run menu have measurable boxes");
check(lastRowBox !== null && menuBox !== null && (
  lastRowBox.x + lastRowBox.width <= menuBox.x
  || menuBox.x + menuBox.width <= lastRowBox.x
  || lastRowBox.y + lastRowBox.height <= menuBox.y
  || menuBox.y + menuBox.height <= lastRowBox.y
), "phone inspector last content row scrolls clear of the run menu button");
await phoneRun.ctx.close();

// Battle Unit inspection uses the same full-card inspector despite compact board cards.
const battle = await openRun(browser, lineFullRun(), DESKTOP);
await battle.page.click("#run-fight");
await battle.page.waitForSelector('#board [data-card-entity="unit"]');
await battle.page.waitForSelector('#board [data-non-card="battle-event"]');
check(await battle.page.$$eval('#board [data-non-card="battle-event"]', (els) => els.length) === 1, "live battle center is one explicit non-card event panel");
check(await battle.page.$$eval('#board [data-non-card="battle-event"] [data-card-entity]', (els) => els.length) === 0, "battle-event panel does not construct or nest a card");
check(await battle.page.$$eval('#board .acting-card', (els) => els.length) === 0, "live battle rejects the retired competing acting-card class");
const battleKinds = await battle.page.$$eval('#board [data-card-entity]', (els) => [...new Set(els.map((el) => el.dataset.cardEntity))].sort().join(","));
check(battleKinds === "unit", "only stable side Units bear cards in battle", battleKinds);
await battle.page.locator('#board [data-card-entity="unit"]').first().click();
await battle.page.waitForSelector('#inspect-overlay:not([hidden])');
check(await battle.page.$$eval('#inspect-overlay > .unit-b.is-full[data-card-entity="unit"]', (els) => els.length) === 1, "battle Unit inspector has exactly one FULL shared Unit card");
await battle.ctx.close();

await browser.close();
disarm();
finish("probe-entity-cards");
