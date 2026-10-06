// Round 3, note 14 (R3-20): the battle's big moments on screen. Plays rounds
// at 1× on a phone and catches, as they play, a kill's skull burst, a big
// hit's larger float and harder shake, a summon sliding in and the fatigue
// banner, with a screenshot of each first sighting. Needs a LOCAL MVP server:
//   node e2e/probe-emphasis.mjs --url http://127.0.0.1:8913/arena/ [--out e2e/.shots/emphasis] [--rounds 6]
import { mkdirSync } from "node:fs";
import { launchChromium } from "./browser.mjs";

const arg = (k, d) => { const i = process.argv.indexOf(k); return i > 0 ? process.argv[i + 1] : d; };
const url = arg("--url");
const out = arg("--out", "e2e/.shots/emphasis");
const rounds = Number(arg("--rounds", "6"));
if (!url) throw new Error("--url is required");
mkdirSync(out, { recursive: true });
const SEEN = { skull: ".bv-skull", big: ".bv-float.damage.big", shake: ".bv-card.bv-hit.bv-big", enter: ".bv-slot.bv-enter", fatigue: ".bv-banner.on.fatigue", crown: ".bv-banner.on.crown" };
const seen = {};
const browser = await launchChromium();
const page = await browser.newPage({ viewport: { width: 360, height: 640 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true });
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));
await page.goto(url, { timeout: 20_000 });
await page.getByTestId("name-input").fill(`Emph${Date.now().toString(36).slice(-4)}`);
await page.getByTestId("name-submit").click();
await page.getByTestId("play").click();
for (let round = 1; round <= rounds; round++) {
  await page.getByTestId("fight").waitFor({ timeout: 10_000 });
  for (let k = 0; k < 3; k++) {
    if ((await page.getByTestId("gold").count()) === 0) break;
    const gold = Number((await page.getByTestId("gold").textContent()).replace("g", ""));
    if (gold < 3 || (await page.getByTestId("line").locator(".card.you").count()) >= 5) break;
    await page.getByTestId("offer-0").click();
    if (await page.getByTestId("buy").isDisabled()) { await page.getByTestId("offer-close").click(); break; }
    await page.getByTestId("buy").click();
    await page.waitForFunction((g) => document.querySelector('[data-testid="gold"]')?.textContent !== `${g}g`, gold);
  }
  await page.getByTestId("fight").click();
  await page.getByTestId("battle-end").waitFor({ timeout: 10_000 });
  for (let i = 0; i < 3 && (await page.getByTestId("battle-speed").textContent()) !== "1×"; i++) await page.getByTestId("battle-speed").click();
  while (!(await page.getByTestId("battle-done").count())) {
    const now = await page.evaluate((sel) => Object.entries(sel).filter(([, s]) => document.querySelector(s)).map(([k]) => k), SEEN);
    // The shot waits a moment: a banner fades in, a skull bursts after the shake and the freeze.
    for (const k of now) if (!seen[k]) { seen[k] = round; if (k !== "enter") await page.waitForTimeout(k === "big" || k === "shake" ? 60 : k === "skull" ? 750 : 250); await page.screenshot({ path: `${out}/${k}.png` }); }
    await page.waitForTimeout(40);
  }
  await page.getByTestId("battle-done").click();
  await page.waitForFunction(() => document.querySelector('[data-testid="fight"]') || document.querySelector('[data-testid="run-over"]'));
  if (await page.getByTestId("run-over").isVisible().catch(() => false)) break;
}
await browser.close();
console.log(`seen: ${Object.keys(SEEN).map((k) => `${k} ${seen[k] ? `(round ${seen[k]})` : "no"}`).join(", ")}`);
if (errors.length) { console.log(errors.join("\n")); process.exitCode = 1; }
