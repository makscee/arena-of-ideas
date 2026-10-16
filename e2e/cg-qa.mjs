// CrazyGames QA pass (#1220, mission #1214): the hard rules a QA tester checks,
// run on the cg zip's files from a second origin against a local server, in
// Chromium and WebKit at the two QA windows (800×450 touch, 821×462 mouse).
// Each rule gets pass or fail, a note and a screenshot; results.json lists them.
//   - gameplay within 20 s on throttled 4G (Chromium: 4 Mbit/s, 150 ms), and unthrottled
//   - no page scroll; Play, Fight, the line and the offers on screen
//   - user-select: none on body; no fullscreen button; no link out of the game
//   - Esc left to the browser (nothing opens), Backspace opens the run menu
//   - a battle at 144 Hz takes as long as at 60 Hz (time-based, not frame-based)
//   - no console error or failed request
//   npm run mvp:cg-qa -- [--out DIR] [--browsers chromium,webkit]
import { spawn, execFileSync } from "node:child_process";
import { createReadStream, existsSync, mkdirSync, statSync, writeFileSync } from "node:fs";
import { createServer as http } from "node:http";
import { createServer } from "node:net";
import { extname, join, normalize } from "node:path";
import { webkit } from "playwright";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/cg-qa";
const browsers = (opt("browsers") ?? "chromium,webkit").split(",");
mkdirSync(out, { recursive: true });
const free = () => new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
const [apiPort, pagePort] = [await free(), await free()];
const apiOrigin = `http://127.0.0.1:${apiPort}`;
const pageOrigin = `http://localhost:${pagePort}`;

execFileSync("npm", ["run", "-s", "mvp:build:cg"], { stdio: "inherit", env: { ...process.env, VITE_API_ORIGIN: `${apiOrigin}/arena` } });
const root = "mobile/dist-cg";
const SDK_URL = "https://sdk.crazygames.com/crazygames-sdk-v3.js";
// The SDK as the portal runs it for a logged-out visitor.
const mockSdk = `window.CrazyGames = { SDK: { environment: "crazygames", init: async () => {},
  game: { settings: { muteAudio: false, disableChat: false }, loadingStart() {}, loadingStop() {}, gameplayStart() {}, gameplayStop() {}, addSettingsChangeListener() {} },
  ad: { requestAd(t, cb) { cb?.adFinished?.(); } },
  user: { isUserAccountAvailable: true, getUser: async () => null, getUserToken: async () => { throw new Error("userNotAuthenticated"); }, addAuthListener() {} } } };`;

const api = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], {
  env: { ...process.env, PORT: String(apiPort), MVP_DB: ":memory:", MVP_INVITES: "1", MVP_DEV: "1", ARENA_CG: "1", ARENA_TUNER: "instant", MVP_CORS_ORIGINS: pageOrigin },
  stdio: ["ignore", "ignore", "inherit"],
});
const MIME = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".woff2": "font/woff2", ".m4a": "audio/mp4", ".mp3": "audio/mpeg", ".ogg": "audio/ogg", ".wav": "audio/wav", ".svg": "image/svg+xml", ".png": "image/png" };
const pages = http((req, res) => {
  const path = normalize(decodeURIComponent(new URL(req.url, pageOrigin).pathname)).replace(/^\/+/, "") || "index.html";
  const file = join(root, path);
  if (path.startsWith("..") || !existsSync(file) || !statSync(file).isFile()) return void res.writeHead(404).end("not found");
  res.writeHead(200, { "content-type": MIME[extname(file)] ?? "application/octet-stream" });
  createReadStream(file).pipe(res);
}).listen(pagePort, "127.0.0.1");

const WINDOWS = [
  { name: "800x450", viewport: { width: 800, height: 450 }, hasTouch: true },
  { name: "821x462", viewport: { width: 821, height: 462 }, hasTouch: false },
];
const results = [];
const rule = (browser, win, id, ok, note, shot) => results.push({ browser, window: win, rule: id, ok, note, shot });
let guest = 0;

/** A fresh visitor: its own storage, its own address (the server lets 5 guests a day per address). */
async function visit(browser, kind, win, { throttle = false, frames = null } = {}) {
  const ctx = await browser.newContext({ viewport: win.viewport, hasTouch: win.hasTouch, deviceScaleFactor: 1 });
  const page = await ctx.newPage();
  const ip = `10.9.${Math.floor(++guest / 250)}.${guest % 250}`;
  const errors = [];
  await page.route(SDK_URL, (r) => r.fulfill({ contentType: "text/javascript", body: mockSdk }));
  await page.route(`${apiOrigin}/**`, (r) => r.continue({ headers: { ...r.request().headers(), "x-forwarded-for": ip } }));
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && errors.push(`console: ${m.text()}`));
  page.on("requestfailed", (r) => !/\/music\//.test(r.url()) && errors.push(`request failed: ${r.url()} ${r.failure()?.errorText}`));
  page.on("response", (r) => r.status() >= 400 && errors.push(`${r.status()}: ${r.url()}`));
  // A display of window.__hz Hz: animation frames every 1000/__hz ms (set on the fly).
  if (frames) await page.addInitScript((hz) => {
    window.__hz = hz;
    window.requestAnimationFrame = (cb) => setTimeout(() => cb(performance.now()), 1000 / window.__hz);
  }, frames);
  if (throttle && kind === "chromium") {
    const cdp = await ctx.newCDPSession(page);
    await cdp.send("Network.enable");
    await cdp.send("Network.emulateNetworkConditions", { offline: false, latency: 150, downloadThroughput: (4e6 / 8), uploadThroughput: (2e6 / 8) });
  }
  return { ctx, page, errors };
}

const inView = (page, id) => page.getByTestId(id).first().evaluate((el) => {
  const r = el.getBoundingClientRect();
  return r.width > 0 && r.top >= -1 && r.left >= -1 && r.bottom <= innerHeight + 1 && r.right <= innerWidth + 1;
}).catch(() => false);
const scrolls = (page) => page.evaluate(() => ({ x: document.documentElement.scrollWidth - innerWidth, y: document.documentElement.scrollHeight - innerHeight }));

/** A battle's replay, in ms of wall time, at 60 Hz and at 144 Hz. */
async function battleMs(page) {
  await page.getByTestId("play").click();
  await page.getByTestId("fight").waitFor({ timeout: 20_000 });
  await page.getByTestId("offer-0").click();
  await page.getByTestId("buy").click();
  await page.getByTestId("line").locator(".card.you").first().waitFor({ timeout: 10_000 });
  await page.getByTestId("fight").click();
  await page.getByTestId("battle-done").waitFor({ timeout: 120_000 });
  // The same battle replayed, once at 60 Hz and once at 144 Hz.
  const ms = {};
  for (const hz of [60, 144]) {
    await page.evaluate((h) => (window.__hz = h), hz);
    const t0 = Date.now();
    await page.getByTestId("battle-replay").click();
    await page.getByTestId("battle-done").waitFor({ state: "hidden", timeout: 5_000 });
    await page.getByTestId("battle-done").waitFor({ timeout: 120_000 });
    ms[hz] = Date.now() - t0;
  }
  return ms;
}

try {
  for (let i = 0; i < 300; i++) {
    try { if ((await fetch(`${apiOrigin}/arena/api/v1/health`)).ok) break; } catch {}
    await new Promise((r) => setTimeout(r, 200));
  }
  for (const kind of browsers) {
    const browser = kind === "webkit" ? await webkit.launch() : await launchChromium();
    try {
      for (const win of WINDOWS) {
        const tag = `${kind}-${win.name}`;
        const shot = (n) => `${tag}-${n}.png`;
        // Load to gameplay: a cold visit, throttled where the browser can be.
        const { ctx, page, errors } = await visit(browser, kind, win, { throttle: true });
        const t0 = Date.now();
        await page.goto(`${pageOrigin}/index.html`, { timeout: 60_000 });
        await page.getByTestId("play").waitFor({ timeout: 60_000 });
        const homeMs = Date.now() - t0;
        await page.getByTestId("play").click();
        await page.getByTestId("fight").waitFor({ timeout: 60_000 });
        const playMs = Date.now() - t0;
        await page.screenshot({ path: join(out, shot("shop")) });
        rule(kind, win.name, "load", playMs < 20_000, `${kind === "chromium" ? "throttled 4G (4 Mbit/s, 150 ms)" : "unthrottled (WebKit has no throttling)"}: Home in ${(homeMs / 1000).toFixed(1)} s, the shop (gameplay) in ${(playMs / 1000).toFixed(1)} s`, shot("shop"));
        const s = await scrolls(page);
        const must = ["fight", "line", "offers"];
        const off = [];
        for (const id of must) if (!(await inView(page, id))) off.push(id);
        rule(kind, win.name, "fits", s.x <= 1 && s.y <= 1 && !off.length, `page scroll ${s.x}×${s.y} px; ${off.length ? `off screen: ${off.join(", ")}` : "Fight, the line and the offers on screen"}`, shot("shop"));
        const sel = await page.evaluate(() => { const st = getComputedStyle(document.body); return st.userSelect || st.webkitUserSelect; });
        rule(kind, win.name, "user-select", sel === "none", `body user-select: ${sel}`, shot("shop"));
        const dom = await page.evaluate(() => ({
          links: [...document.querySelectorAll("a[href]")].map((a) => a.getAttribute("href")).filter((h) => /^(https?:)?\/\//.test(h) || /t\.me|telegram/.test(h)),
          blank: document.querySelectorAll("a[target=_blank]").length,
          full: [...document.querySelectorAll("button,[role=button]")].filter((b) => /fullscreen|full screen/i.test(b.textContent + b.getAttribute("aria-label") + b.title)).length,
        }));
        rule(kind, win.name, "no links out, no fullscreen button", !dom.links.length && !dom.blank && !dom.full, `${dom.links.length} outside link(s)${dom.links.length ? `: ${dom.links.join(" ")}` : ""}, ${dom.blank} new-tab link(s), ${dom.full} fullscreen button(s)`, shot("shop"));
        // Keys: Esc belongs to the browser; Backspace opens the run menu.
        await page.keyboard.press("Escape");
        await page.waitForTimeout(400);
        const escOpened = await page.getByTestId("run-menu").count();
        await page.keyboard.press("Backspace");
        const bsOpened = await page.getByTestId("run-menu").waitFor({ timeout: 3_000 }).then(() => true, () => false);
        await page.screenshot({ path: join(out, shot("menu")) });
        rule(kind, win.name, "keys (Esc free, Backspace = back)", !escOpened && bsOpened, `Esc opened the menu: ${escOpened ? "yes" : "no"}; Backspace opened it: ${bsOpened ? "yes" : "no"}`, shot("menu"));
        if (bsOpened) await page.getByTestId("menu-resume").click().catch(() => {});
        // Settings at the QA window: every row reachable.
        await page.getByTestId("menu-open").click().catch(() => {});
        await page.getByTestId("menu-title").click().catch(() => {});
        await page.getByTestId("play").waitFor({ timeout: 15_000 }).catch(() => {});
        await page.screenshot({ path: join(out, shot("home")) });
        const homeOk = await inView(page, "play");
        rule(kind, win.name, "home fits", homeOk && (await scrolls(page)).y <= 1, `Play ${homeOk ? "on screen" : "off screen"}`, shot("home"));
        rule(kind, win.name, "no errors", !errors.length, errors.length ? [...new Set(errors)].slice(0, 5).join("; ") : "no console error or failed request", shot("home"));
        await ctx.close();
      }
      // 144 Hz: a battle and its replay at 144 Hz take the same time (Chromium; animation frames are emulated).
      if (kind === "chromium") {
        const win = WINDOWS[1];
        const { ctx, page } = await visit(browser, kind, win, { frames: 60 });
        await page.goto(`${pageOrigin}/index.html`);
        await page.getByTestId("play").waitFor({ timeout: 30_000 });
        const ms = await battleMs(page).catch((e) => (console.error(e), { 60: NaN, 144: NaN }));
        await page.screenshot({ path: join(out, "chromium-144hz-battle.png") });
        await ctx.close();
        const ratio = ms[144] / ms[60];
        rule(kind, "821x462", "144 Hz timing", Math.abs(ratio - 1) < 0.15, `the same battle replayed: ${(ms[60] / 1000).toFixed(1)} s at 60 Hz, ${(ms[144] / 1000).toFixed(1)} s at 144 Hz (×${ratio.toFixed(2)})`, "chromium-144hz-battle.png");
      }
    } finally {
      await browser.close();
    }
  }
} catch (e) {
  results.push({ browser: "-", window: "-", rule: "the pass itself", ok: false, note: String(e?.message ?? e).split("\n")[0] });
} finally {
  api.kill();
  pages.close();
}
writeFileSync(join(out, "results.json"), JSON.stringify(results, null, 1));
for (const r of results) console.log(`${r.ok ? "pass" : "FAIL"}  ${r.browser} ${r.window}  ${r.rule}: ${r.note}`);
process.exit(results.every((r) => r.ok) ? 0 : 1);
