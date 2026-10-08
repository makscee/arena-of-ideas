// R2-8 / R2-17 terms on a desktop (1440×900, a mouse): hovering a unit's
// trigger shows its rule in a tooltip; clicking a term pins its rule in a
// popover under it (not a modal sheet), which Esc and a click outside close,
// and Esc then never reaches the shop (no run menu). Part of mvp:check.
// Without --url it builds the mobile client and starts the MVP server on a
// free port, as the phone and desktop e2e do.
//   npm run mvp:terms -- [--url http://127.0.0.1:<port>/arena/] [--out dir]
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
let url = opt("url");
const out = opt("out") ?? "e2e/.shots/terms-desktop";
mkdirSync(out, { recursive: true });
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
  for (let i = 0; i < 300; i++) { // up to 60 s: m1 and m4 are shared, and under load the server starts slowly
    try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
    await new Promise((r) => setTimeout(r, 200));
  }
}
const browser = await launchChromium();
const errors = [];
try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && errors.push(`console: ${m.text()}`));
  await page.goto(url);
  await page.getByTestId("name-input").fill(`TermTester${Date.now() % 100000}`);
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

  // A click pins the rule in a popover just under (or over) the term.
  await trig.click();
  const pop = page.getByTestId("term-popover");
  await pop.waitFor({ timeout: 2000 }).catch(() => errors.push("clicking a term opens no popover"));
  if (await page.getByTestId("overlay").count()) errors.push("clicking a term opens a modal sheet, not a popover");
  if (!(await pop.getByTestId("term-tip").count())) errors.push("the popover has no rule");
  if (!(await pop.getByTestId("term-codex").count())) errors.push("the popover has no Open in Codex");
  const [t, p] = [await trig.boundingBox(), await pop.boundingBox()];
  if (t && p && !(Math.abs(p.y - (t.y + t.height)) < 16 || Math.abs(p.y + p.height - t.y) < 16)) errors.push(`the popover isn't pinned to the term (term y ${Math.round(t.y)}, popover ${Math.round(p.y)}–${Math.round(p.y + p.height)})`);
  await page.screenshot({ path: `${out}/click-trigger.png` });
  // Esc closes it, and only it: no run menu opens under it.
  await page.keyboard.press("Escape");
  await pop.waitFor({ state: "detached", timeout: 2000 }).catch(() => errors.push("Esc doesn't close the popover"));
  if (await page.getByTestId("run-menu").count()) errors.push("Esc on the popover also opened the run menu");
  // A click outside closes it too.
  await trig.click();
  await pop.waitFor({ timeout: 2000 });
  await page.mouse.click(700, 860);
  await pop.waitFor({ state: "detached", timeout: 2000 }).catch(() => errors.push("a click outside doesn't close the popover"));
  // Open in Codex from the popover lands on the term's row.
  await trig.click();
  await pop.getByTestId("term-codex").click();
  await page.locator(".kw-row.landed").waitFor({ timeout: 5000 }).catch(() => errors.push("Open in Codex from the popover lands on no row"));
  if (await pop.count()) errors.push("the popover stays open over the Codex");
} finally {
  await browser.close();
  child?.kill();
}
if (errors.length) { console.error(errors.join("\n")); process.exit(1); }
console.log("terms desktop: ok");
