// M2-6 pick screens e2e (makscee/void-board#794): on a 360×640 phone, a new
// player gets "+1 idea", writes one, waits for the fake reader, sees Home's
// "💡 Your idea is ready", turns the archetypes down once ("None of these"),
// picks an archetype, then a reading (opening "See Awoken"), and My ideas
// shows it as being tested. A screenshot of each screen; the pick screens
// never scroll the page. Then the reading pick once at 1280×800, its sheet
// in the side panel. Without --url it builds the client and starts a dev
// server (fake reader, reading every 500 ms) on a free port; never point it
// at the live game.
//   npm run mvp:ideas-phone -- [--url http://127.0.0.1:PORT/arena/] [--out e2e/.shots/mvp-ideas]
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/mvp-ideas";
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", ARENA_IDEA_READER: "fake", ARENA_IDEA_READ_MS: "500" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
  for (let i = 0; i < 300; i++) {
    try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
    await new Promise((r) => setTimeout(r, 200));
  }
}
if (/makscee\.ru|twin-pogona/.test(url)) throw new Error("never against the live game");

const TAG = Date.now().toString(36).slice(-4);
const IDEA = "A hedgehog that punishes whoever hits it with its spines.";
const browser = await launchChromium();
const errors = [];
let shots = 0;

/** One player's walk at `viewport`; `full` walks every step, else only to the reading pick. */
async function walk(viewport, name, full) {
  const page = await browser.newPage({ viewport, deviceScaleFactor: 2, isMobile: viewport.width < 700, hasTouch: viewport.width < 700 });
  page.on("pageerror", (e) => errors.push(`${name}: pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && errors.push(`${name}: console: ${m.text()}`));
  const shot = (s) => page.screenshot({ path: `${out}/${String(++shots).padStart(2, "0")}-${s}.png` });
  /** The page doesn't scroll: everything fits the viewport (the sheet's own box may scroll). */
  const noScroll = async (s) => {
    const { w, h } = await page.evaluate(() => ({ w: document.documentElement.scrollWidth, h: document.documentElement.scrollHeight }));
    if (w > viewport.width) errors.push(`${s}: horizontal scroll (${w}px)`);
    if (h > viewport.height + 1) errors.push(`${s}: the page scrolls (${h}px tall)`);
  };
  const onScreen = async (s, loc) => {
    const b = await loc.boundingBox();
    if (!b || b.y < 0 || b.y + b.height > viewport.height + 0.5) errors.push(`${s}: off screen (${b ? Math.round(b.y) : "none"})`);
  };
  await page.goto(url);
  await page.getByTestId("name-input").fill(`${name}${TAG}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").waitFor();
  // Dev "+1 idea", then My ideas → New idea → Send.
  await page.locator("details.dev summary").click();
  await page.getByTestId("grant-idea").click();
  await page.waitForFunction(() => /^💡 1 idea$/.test(document.querySelector('[data-testid="ideas"]')?.textContent ?? ""), null, { timeout: 10_000 });
  await page.getByTestId("ideas").click();
  await page.getByTestId("idea-new").click();
  await page.getByTestId("idea-text").fill(IDEA);
  if (full) await shot("write");
  await page.getByTestId("idea-send").click();
  await page.getByTestId("idea-sent").waitFor();
  if (full) await shot("sent-being-read");
  // My ideas looks again on its own while the idea is read (10 s).
  await page.locator('[data-testid="idea-sent-row"][data-state="pick-archetype"]').waitFor({ timeout: 30_000 });
  if (full) {
    if ((await page.getByTestId("idea-stage").textContent()) !== "Ready: pick its archetype") errors.push(`my ideas: stage "${await page.getByTestId("idea-stage").textContent()}"`);
    await shot("ideas-ready");
    // Home: the quiet "Your idea is ready".
    await page.getByTestId("ideas-back").click();
    await page.getByTestId("play").waitFor();
    if ((await page.getByTestId("ideas").textContent()) !== "💡 Your idea is ready") errors.push(`home: ideas line "${await page.getByTestId("ideas").textContent()}"`);
    await shot("home-idea-ready");
    await page.getByTestId("ideas").click();
  }
  // The archetype pick: 3 big rows, Confirm only after a tap.
  await page.getByTestId("idea-pick").click();
  await page.getByTestId("pick-archetypes").waitFor();
  const rows = page.getByTestId("pick-archetype");
  if ((await rows.count()) !== 3) errors.push(`archetypes: ${await rows.count()} rows`);
  if (!(await page.getByTestId("pick-confirm").isDisabled())) errors.push("archetypes: Confirm before a pick");
  if (full) {
    await shot("pick-archetype");
    await noScroll("pick-archetype");
    await onScreen("pick-archetype: Confirm", page.getByTestId("pick-confirm"));
    // None of these, once: read again for other archetypes.
    const first = await rows.allTextContents();
    if (!/read it again/.test(await page.getByTestId("pick-none").textContent())) errors.push("none: the first one doesn't read again");
    await page.getByTestId("pick-none").click();
    await page.getByTestId("idea-sent").waitFor();
    await page.locator('[data-testid="idea-sent-row"][data-state="pick-archetype"]').waitFor({ timeout: 30_000 });
    await page.getByTestId("idea-pick").click();
    await page.getByTestId("pick-archetypes").waitFor();
    const again = await rows.allTextContents();
    if (again.some((t) => first.includes(t))) errors.push(`none: the same archetypes again (${again.join(" | ")})`);
    if (!/take my idea back/.test(await page.getByTestId("pick-none").textContent())) errors.push("none: the second one doesn't say it gives the idea back");
  }
  await rows.nth(1).click();
  if (await page.getByTestId("pick-confirm").isDisabled()) errors.push("archetypes: Confirm stays off after a tap");
  if (full) await shot("pick-archetype-tapped");
  await page.getByTestId("pick-confirm").click();
  await page.getByTestId("idea-sent").waitFor();
  await page.locator('[data-testid="idea-sent-row"][data-state="pick-reading"]').waitFor({ timeout: 30_000 });
  // The reading pick: 3 cards like the shop's, numbers set by simulation.
  await page.getByTestId("idea-pick").click();
  await page.getByTestId("pick-readings").waitFor();
  const cards = page.getByTestId("pick-reading");
  if ((await cards.count()) !== 3) errors.push(`readings: ${await cards.count()} cards`);
  if (full) await shot("pick-reading");
  // Tap low on the card (its top opens nothing here, but stay clear of the icon line).
  await cards.nth(0).click({ position: { x: 30, y: 60 } });
  await page.getByTestId("unit-sheet").waitFor();
  if (!/set by simulation/.test(await page.getByTestId("sheet-unset").textContent())) errors.push("reading sheet: no 'set by simulation'");
  if ((await page.locator('[data-testid="pick-detail"] .term, [data-testid="pick-detail"] [data-term]').count()) === 0) errors.push("reading sheet: no highlighted keywords");
  await shot(`${name}-pick-reading-tapped`);
  await noScroll(`${name} pick-reading`);
  await onScreen(`${name} pick-reading: Confirm`, page.getByTestId("pick-confirm"));
  await page.getByTestId("see-awoken").click();
  if (!/Awoken/.test(await page.getByTestId("sheet-state").textContent())) errors.push("see awoken: the sheet still says Sleeping");
  await shot(`${name}-pick-reading-awoken`);
  await noScroll(`${name} pick-reading-awoken`);
  if (!full) return page.close();
  await page.getByTestId("pick-confirm").click();
  await page.locator('[data-testid="idea-sent-row"][data-state="simulating"]').waitFor();
  if (!/test it overnight/.test(await page.getByTestId("idea-sent").textContent())) errors.push(`after the pick: "${await page.getByTestId("idea-sent").textContent()}"`);
  if (!/being tested/.test(await page.getByTestId("idea-stage").textContent())) errors.push(`my ideas after the pick: "${await page.getByTestId("idea-stage").textContent()}"`);
  await shot("ideas-being-tested");
  await page.close();
}

try {
  await walk({ width: 360, height: 640 }, "Pick", true);
  await walk({ width: 1280, height: 800 }, "Desk", false);
} catch (e) {
  errors.push(`threw: ${e.message}`);
} finally {
  await browser.close();
  child?.kill();
}
console.log(`${shots} screenshots in ${out}`);
if (errors.length) {
  console.error(errors.map((e) => `✗ ${e}`).join("\n"));
  process.exit(1);
}
console.log("✓ write → archetype → reading on a phone, and the reading pick on desktop");
