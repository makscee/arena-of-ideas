import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { aoi61ReadyRun, armGuard, DESKTOP, launch, openRun, PHONE } from "./lib.mjs";

const disarm = armGuard(150_000);
const browser = await launch();
const out = process.env.SHOTS_DIR ?? "e2e/.evidence/aoi61";
mkdirSync(out, { recursive: true });

async function shotFlow(viewport, tag) {
  let opened = await openRun(browser, aoi61ReadyRun(), viewport);
  let { ctx, page } = opened;
  const snap = (name) => page.screenshot({ path: join(out, `${tag}-${name}.png`), fullPage: true });
  const assertBadge = async (index, expected) => {
    const text = (await page.locator(`#run-line [data-line="${index}"] .run-progression`).innerText()).replace(/\s+/g, " ").trim();
    if (!text.includes(expected)) throw new Error(`${tag} expected visible badge "${expected}", got "${text}"`);
  };

  await assertBadge(0, "Base 2/3");
  await assertBadge(1, "Awakened 3/3");
  await snap("01-base-2-of-3");
  await page.click('[data-buy="0"]');
  await assertBadge(0, "Awakened 3/3");
  await page.click('[data-fusion-first="0"]');
  await snap("02-ordered-preview-a-plus-b");
  await page.click("[data-fusion-cancel]");
  await page.click('[data-fusion-first="1"]');
  await snap("03-ordered-preview-b-plus-a");
  await page.click("[data-fusion-cancel]");
  await page.click('[data-fusion-first="0"]');
  await page.click('[data-fuse="0:1"]');
  await assertBadge(0, "Fusion · Fresh 0/3");
  await snap("04-fused-zero-of-three");
  for (let i = 0; i < 3; i++) await page.click('[data-buy="0"]');
  await page.waitForSelector('[data-awaken-path="trigger"]');
  await assertBadge(0, "Fusion · Pending 3/3");
  await snap("05-blocking-choice");
  await page.click('[data-awaken-path="trigger"]');
  await assertBadge(0, "Fusion · Trigger path 3/3");
  await snap("06-trigger-awakened-doubled");
  const resumedRaw = await page.evaluate(() => localStorage.getItem("aoi.run.v1"));
  await ctx.close();
  opened = await openRun(browser, resumedRaw, viewport);
  ({ ctx, page } = opened);
  await assertBadge(0, "Fusion · Trigger path 3/3");
  await page.click('[data-buy="0"]');
  await assertBadge(0, "Fusion · Trigger path 3/3");
  await snap("07-resumed-later-literal-copy");

  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  if (overflow > 1) throw new Error(`${tag} horizontal overflow ${overflow}px`);
  await page.click("#run-fight");
  await page.waitForSelector("#run-battle:not([hidden])");
  await snap("08-battle-start");
  await page.waitForTimeout(900);
  await snap("09-battle-motion");
  await ctx.close();
}

await shotFlow(DESKTOP, "desktop");
await shotFlow(PHONE, "phone");
await browser.close();
disarm();
console.log(`shots-aoi61: wrote desktop + phone flow to ${out}`);
