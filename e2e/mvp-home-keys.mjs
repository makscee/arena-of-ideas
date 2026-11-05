// Home's keys and Home's single copy (arena#31): Enter on Home plays (or resumes) from a fresh load, from
// Home after the ≡ menu closed, after the Workshop window closed (its Back, Esc and the browser's Back) and
// after the Codex window closed; and while a window is open over Home, Home is in the page exactly once
// (the kept table under the window), never a second, fresh copy with a second key handler.
//   npm run mvp:home-keys -- [--url http://127.0.0.1:8841/arena/]
import { spawn, execFileSync } from "node:child_process";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", ARENA_CODEX: "1", MVP_POOL: "v2", ARENA_TUNER: "instant" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
  for (let i = 0; i < 300; i++) {
    try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
    await new Promise((r) => setTimeout(r, 200));
  }
}

const errors = [];
const browser = await launchChromium();
const homes = (page) => page.evaluate(() => document.querySelectorAll('[data-testid="home"]').length);
const settle = (page) => page.waitForTimeout(500);

async function fresh(label, W = 1280, H = 720) {
  const ctx = await browser.newContext({ viewport: { width: W, height: H } });
  const fail = (m) => errors.push(`${label}: ${m}`);
  const page = await ctx.newPage();
  page.on("pageerror", (e) => fail(`pageerror: ${e.message}`));
  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").waitFor({ timeout: 15_000 });
  await page.getByTestId("name-input").fill(`Keys${label.replace(/\W/g, "")}${Date.now() % 100000}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").waitFor({ timeout: 15_000 });
  await settle(page);
  return { ctx, page, fail };
}
/** Home is in the page once, and Enter opens the run's shop (PLAY, or RESUME when a run waits). */
async function enterPlays(page, fail, where) {
  const n = await homes(page);
  if (n !== 1) fail(`${where}: Home is in the page ${n} times, not once`);
  await page.keyboard.press("Enter");
  await page.getByTestId("hud").waitFor({ timeout: 6_000 }).catch(() => fail(`${where}: Enter on Home does not open round 1`));
}
/** Home was left (to the title menu) with a run waiting. */
async function backHome(page) {
  await page.getByTestId("menu-open").click();
  await page.getByTestId("menu-title").click();
  await page.getByTestId("home").waitFor({ timeout: 10_000 });
  await settle(page);
}

try {
  // 1. A fresh load.
  {
    const { ctx, page, fail } = await fresh("fresh");
    await enterPlays(page, fail, "fresh load");
    // 2. RESUME: the run waits; Enter resumes it, from a fresh load of the page too.
    await backHome(page);
    if (!/resume/i.test((await page.getByTestId("play").textContent()) ?? "")) fail("the button does not read RESUME while a run waits");
    await enterPlays(page, fail, "RESUME");
    await backHome(page);
    await page.reload();
    await page.getByTestId("play").waitFor({ timeout: 15_000 });
    await settle(page);
    await enterPlays(page, fail, "RESUME after a reload");
    await ctx.close();
  }

  // 3. The ≡ menu: open by the plate and by Esc, close by Esc, by its close button; Enter plays after each.
  for (const [how, open, close] of [
    ["plate / Esc", (p) => p.getByTestId("settings").click(), (p) => p.keyboard.press("Escape")],
    ["Esc / close button", (p) => p.keyboard.press("Escape"), (p) => p.getByTestId("sheet-close").click()],
  ]) {
    const { ctx, page, fail } = await fresh(`menu ${how}`);
    await open(page);
    await page.getByTestId("settings-sheet").waitFor({ timeout: 4_000 }).catch(() => fail("the ≡ menu did not open"));
    const n = await homes(page);
    if (n !== 1) fail(`the ≡ menu is open: Home is in the page ${n} times`);
    await close(page);
    await page.getByTestId("settings-sheet").waitFor({ state: "detached", timeout: 4_000 }).catch(() => fail("the ≡ menu did not close"));
    await settle(page);
    await enterPlays(page, fail, `after the ≡ menu (${how})`);
    await ctx.close();
  }

  // 4. The windows over Home: the Workshop (its Back, Esc, the browser's Back) and the Codex (Back, Esc, the browser's Back).
  const windows = [
    ["Workshop", (p) => p.getByTestId("ideas").click(), "ideas-held", [["Back button", (p) => p.getByTestId("ideas-back").click()], ["Esc", (p) => p.keyboard.press("Escape")], ["browser Back", (p) => p.goBack()]]],
    ["Codex", (p) => p.keyboard.press("c"), "codex-units", [["Back button", (p) => p.getByTestId("codex-back").click()], ["Esc", (p) => p.keyboard.press("Escape")], ["browser Back", (p) => p.goBack()]]],
  ];
  for (const [name, open, marker, closers] of windows) {
    for (const withRun of [false, true]) {
      for (const [how, close] of closers) {
        const label = `${name} ${how}${withRun ? " with a run" : ""}`;
        const { ctx, page, fail } = await fresh(label);
        if (withRun) {
          await page.getByTestId("play").click();
          await page.getByTestId("hud").waitFor({ timeout: 10_000 });
          await backHome(page);
        }
        await open(page);
        await page.getByTestId(marker).waitFor({ timeout: 8_000 }).catch(() => fail(`${name} did not open`));
        await settle(page);
        const n = await homes(page);
        if (n !== 1) fail(`${name} is open: Home is in the page ${n} times, not once (the kept table)`);
        await close(page);
        await page.getByTestId(marker).waitFor({ state: "detached", timeout: 6_000 }).catch(() => fail(`${name} did not close (${how})`));
        await page.getByTestId("home").first().waitFor({ timeout: 8_000 }).catch(() => fail(`Home is not back after ${name} (${how})`));
        await settle(page);
        await enterPlays(page, fail, `after the ${name} window (${how})`);
        await ctx.close();
      }
    }
  }
} catch (e) {
  errors.push(`${e instanceof Error ? e.stack : e}`);
} finally {
  await browser.close();
  child?.kill();
}
if (errors.length) {
  console.error(errors.map((e) => `FAIL ${e}`).join("\n"));
  process.exit(1);
}
console.log("mvp home keys: ok");
