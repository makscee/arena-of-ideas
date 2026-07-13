// Slice-3 fix round, residuals b/c in the real browser: constructed content
// (injected via localStorage — nothing shipped exercises these paths) whose
// when clause names Poison and whose consumeStacks names Shield explicitly.
// Both must render as tappable refs in the live inspector at 375×667, and
// tapping the when-clause ref must reveal the status definition.

import { PHONE, armGuard, check, finish, launch, openRun, refsRun } from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

const { ctx, page } = await openRun(browser, refsRun(), PHONE);
await page.click('[data-line="0"] .uname'); // inspect the constructed Warden
await page.waitForSelector("#inspect-overlay:not([hidden])");

const refs = page.locator("#inspect-overlay .ins-ref");
const texts = await refs.allTextContents();
check(texts.includes("Poison"), "when-clause status renders as a ref", JSON.stringify(texts));
check(texts.includes("Shield"), "explicit-status consumeStacks renders as a ref", JSON.stringify(texts));

// The when clause itself carries the ref (not just an effect clause): the
// sentence around the Poison ref reads "After Poison lands on an ally". PRD #081:
// a unit references ONE ability, so the heal and the explicit-status
// consumeStacks fold into that one sentence (both refs still tappable).
const insText = await page.locator("#inspect-overlay").textContent();
check(insText.includes("After Poison lands on an ally: heal this unit for 2"), "when-clause sentence intact", JSON.stringify(insText.slice(0, 200)));
check(insText.includes("consume 2 stacks of Shield"), "consumeStacks sentence intact");

// Tap the Poison ref: references use the same app-wide inspector contract,
// replacing the Unit body with Status anatomy rather than nesting a card.
await page.locator('#inspect-overlay [data-status-ref="Poison"]').first().click();
await page.waitForSelector('#inspect-overlay .ins-entity-kind');
check(await page.$eval('#inspect-overlay .ins-entity-kind', (el) => el.textContent.trim()) === "status", "status ref transitions the shared inspector to Status");
const defText = await page.locator("#inspect-overlay").textContent();
check(
  defText.includes("At the end of each turn: deal damage equal to its stacks to the holder, then consume 1 stack of this status."),
  "Status inspector carries the verbatim describeStatus definition",
  JSON.stringify(defText),
);
check(await page.$$eval('#inspect-overlay > .unit-b.is-full[data-card-entity="status"]', (els) => els.length) === 1, "Status ref replaces the overlay with exactly one FULL shared card");
check(await page.$$eval("#inspect-overlay [data-card-entity] [data-card-entity]", (els) => els.length) === 0, "shared inspector contains no nested card");

await ctx.close();
await browser.close();
disarm();
finish("probe-refs");
