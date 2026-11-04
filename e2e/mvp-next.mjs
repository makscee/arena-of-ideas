// NEXT ROUND's first click (arena#32, third round): with a fight's result in the
// tray, one real mouse click at the button's centre, as soon as the button is
// visible, begins the next round, after a win and after a loss, at 800×450 and
// 1280×720, with the API answering at once and 1500 ms late (then the button
// shows pressed within 100 ms). Never a second click. Without --url it builds the
// client and starts a scenes server.
//   npm run mvp:next -- [--url http://127.0.0.1:PORT/arena/] [--cg] [--only run]   (--cg: the CrazyGames build, real runs only: it has no scenes)
import { spawn, execFileSync } from "node:child_process";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
let url = opt("url");
const cg = args.includes("--cg");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit", env: { ...process.env, ...(cg ? { VITE_CG: "1" } : {}) } });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_TUNER: "instant", ARENA_CODEX: "1", MVP_POOL: "v2", ...(cg ? { ARENA_CG: "1", ARENA_GUEST_JOINS_PER_IP: "100", ARENA_GUEST_JOINS_PER_HOUR: "300" } : {}) }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

// A scene's fixture run (a win, a loss), and a real run from PLAY with Gambler and Medic bought (Fight stays off until two units stand).
const CASES = [["scene", "battle-win"], ["scene", "battle-loss"], ["run", "pair"]].filter(([kind]) => !cg || kind === "run");
const SIZES = [[800, 450], [1280, 720]];
const DELAYS = [0, 1500];
// How the result is reached: E, the battle ending on its own, or E and then the card closed (✕), which leaves Next round in the controls (a real run only: a scene has no way on).
const HOWS = ["e", "natural", "close"];
const failures = [];
const browser = await launchChromium();
try {
  for (const [kind, scene] of CASES) for (const how of HOWS.filter((x) => x !== "close" || kind === "run")) for (const [w, h] of SIZES) for (const delay of DELAYS) {
    const label = `${kind === "run" ? "run " : ""}${scene} via ${how} ${w}x${h} delay ${delay}`;
    const ctx = await browser.newContext({ viewport: { width: w, height: h } });
    const page = await ctx.newPage();
    const why = [];
    page.on("pageerror", (e) => why.push(`pageerror: ${e.message}`));
    if (kind === "scene") {
      await page.goto(`${url}#scene=${scene}&lang=en&speed=4`, { timeout: 120_000 });
      await page.waitForSelector('[data-testid="battle-you"]', { timeout: 60_000 });
      // Delay the API only once the scene is up, so the scene itself loads at once.
      if (delay) await page.route(/\/api\/v1\/runs\//, async (r) => { await new Promise((x) => setTimeout(x, delay)); await r.continue().catch(() => {}); });
    } else {
      await page.addInitScript(() => localStorage.setItem("arena.battleSpeed", "4")); // a fight that ends by itself, soon
      await page.goto(url, { timeout: 120_000 });
      if (!cg) await page.getByTestId("start").waitFor({ timeout: 20_000 });
      // The CrazyGames build has no PLAY screen and a slow first load: Enter until the coach is up.
      for (let i = 0; i < 40 && !(await page.getByTestId("coach").count()); i++) { if (i === 0 || cg) await page.keyboard.press("Enter"); await page.waitForTimeout(500); }
      await page.getByTestId("coach").waitFor({ timeout: 20_000 });
      if (scene === "pair") {
        const desk = w >= 1024;
        for (const [name, slot] of [["Gambler", 0], ["Medic", 1]]) {
          await page.locator(`[data-testid="offers"] [data-testid^="offer-"][data-name="${name}"]`).click();
          if (desk) await page.keyboard.press(String(slot + 1)); else await page.getByTestId(`line-${slot}-empty`).click();
          await page.waitForTimeout(150);
        }
      }
      if (delay) await page.route(/\/api\/v1\/runs\//, async (r) => { await new Promise((x) => setTimeout(x, delay)); await r.continue().catch(() => {}); });
      await page.getByTestId("fight").click();
      // An empty line is warned about once (the shop's own rule): Fight again.
      if (!(await page.waitForSelector('[data-testid="battle-you"]', { timeout: 3_000 }).then(() => true, () => false))) await page.getByTestId("fight").click();
      await page.waitForSelector('[data-testid="battle-you"]', { timeout: 20_000 });
      await page.unroute(/\/api\/v1\/runs\//).catch(() => {});
      if (delay) await page.route(/\/api\/v1\/runs\//, async (r) => { await new Promise((x) => setTimeout(x, delay)); await r.continue().catch(() => {}); });
    }
    await page.waitForTimeout(800);
    const btn = how === "close" ? "battle-next" : "battle-done";
    if (how !== "natural") await page.keyboard.press("e"); // E ends the battle: the result takes the tray
    if (how === "close") {
      await page.waitForSelector('[data-testid="end-close"]', { state: "visible", timeout: 20_000 }).catch(() => why.push("no result card"));
      await page.getByTestId("end-close").click().catch(() => {});
    }
    await page.waitForSelector(`[data-testid="${btn}"]`, { state: "visible", timeout: how === "natural" ? 90_000 : 20_000 }).catch(() => why.push(`no ${btn} button`));
    // As soon as it is visible: one real click at its centre, no waiting for it to settle.
    const box = await page.evaluate((id) => { const b = document.querySelector(`[data-testid="${id}"]`)?.getBoundingClientRect(); return b ? { x: b.left + b.width / 2, y: b.top + b.height / 2 } : null; }, btn);
    if (box) {
      const t0 = Date.now();
      await page.mouse.click(box.x, box.y);
      // Began: the shop is up, or (late API) the button shows it was pressed within 100 ms.
      // A page that navigated under the check (a scene's Continue reloads the app) has begun.
      const began = () => page.evaluate(() => !!document.querySelector('[data-testid="fight"], [data-testid="offers"], [data-testid="crown-foe"], [data-testid="run-end"]')).catch(() => true);
      const pressed = () => page.evaluate((id) => { const b = document.querySelector(`[data-testid="${id}"]`); return !b || b.disabled || b.classList.contains("pressed") || b.getAttribute("aria-busy") === "true"; }, btn).catch(() => true);
      let ok = false;
      for (let i = 0; i < 60 && !ok; i++) {
        ok = (await began()) || (delay > 0 && Date.now() - t0 <= 400 && (await pressed()));
        if (!ok) await page.waitForTimeout(25);
        if (delay === 0 && Date.now() - t0 > 2500) break;
        if (delay > 0 && Date.now() - t0 > 400 && !ok) break;
      }
      if (!ok) {
        // Not begun by the first click: would a second one? That is the bug.
        await page.mouse.click(box.x, box.y);
        await page.waitForSelector('[data-testid="fight"], [data-testid="offers"], [data-testid="crown-foe"], [data-testid="run-end"]', { timeout: delay + 5000 }).then(() => why.push("the first click did nothing; a second one began the round"), () => why.push("the next round never began"));
      }
    }
    console.log(`${why.length ? "FAIL" : "ok  "} ${label}${why.length ? ": " + why.join("; ") : ""}`);
    if (why.length) failures.push(`${label}: ${why.join("; ")}`);
    await ctx.close();
  }
} finally {
  await browser.close();
  child?.kill();
}
if (failures.length) { console.error(`\n${failures.length} NEXT ROUND cases failed`); process.exit(1); }
console.log("mvp next: ok");
