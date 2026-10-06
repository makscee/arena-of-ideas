// R2-8 probe: on a desktop (1440×900, a mouse), hovering a unit's trigger
// shows its rule in a tooltip, and clicking a term opens its rule sheet.
//   node e2e/probe-terms-desktop.mjs --url http://127.0.0.1:<port>/arena/ [--out dir]
import { mkdirSync } from "node:fs";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const url = opt("url");
const out = opt("out") ?? "e2e/.shots/terms-desktop";
mkdirSync(out, { recursive: true });
const browser = await launchChromium();
const errors = [];
try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  await page.goto(url);
  await page.getByTestId("name-input").fill("DeskTester");
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").click();
  await page.getByTestId("offer-0").click();
  const form = page.getByTestId("sheet-form");
  await form.waitFor();
  const trig = form.locator('[data-term^="trigger:"]').first();
  await trig.hover();
  const tip = page.getByTestId("term-tooltip");
  await tip.waitFor({ timeout: 2000 }).catch(() => errors.push("hovering the trigger shows no tooltip"));
  const text = await tip.textContent().catch(() => "");
  console.log(`tooltip on ${await trig.getAttribute("data-term")}: ${text}`);
  await page.screenshot({ path: `${out}/hover-trigger.png` });
  await page.mouse.move(5, 5);
  await tip.waitFor({ state: "detached", timeout: 2000 }).catch(() => errors.push("the tooltip stays after the mouse leaves"));
  await trig.click();
  await page.getByTestId("term-sheet").waitFor({ timeout: 2000 }).catch(() => errors.push("clicking a term opens no rule sheet"));
  await page.screenshot({ path: `${out}/click-trigger.png` });
} finally {
  await browser.close();
}
if (errors.length) { console.error(errors.join("\n")); process.exit(1); }
console.log("terms desktop: ok");
