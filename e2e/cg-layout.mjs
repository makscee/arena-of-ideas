// CrazyGames' QA windows (#1217, mission #1214): Home, the shop, a battle, the
// Codex and the Forge at 800×450 (mobile landscape) and 821×462 (desktop
// windowed) fit with no page scroll, and their main controls (Fight, the line,
// the offers, Continue) sit inside the viewport. The same screens are shot at
// 1280×720 and 390×844 too, to compare (no checks there: those layouts are
// the desktop's and the phone's, unchanged).
// Without --url it builds the client and starts its own scenes server.
// Keys at 821×462 (a window with a mouse): the shop takes them by their place
// on the keyboard (KeyboardEvent.code), as a Russian or AZERTY layout sends
// them: 1 buys, R rerolls, Space fights. Esc opens the run's menu; with --cg
// (a client built with VITE_CG=1) Esc does nothing and Backspace opens it.
//   npm run mvp:cg-layout -- [--url http://127.0.0.1:PORT/arena/] [--out e2e/.shots/cg-layout] [--only shop,home,keys] [--cg]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/cg-layout";
const only = opt("only")?.split(",");
const cg = args.includes("--cg");
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit", env: { ...process.env, ...(cg ? { VITE_CG: "1" } : {}) } });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_TUNER: "instant", ARENA_CODEX: "1", ARENA_CG: "1" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

/** A screen: its scene, what shows it is up, and the controls that must be on screen. */
const SCREENS = [
  { name: "home", scene: "home", ready: "home-actions", must: ["home-actions", "champion"] },
  { name: "shop", scene: "shop-r8", ready: "fight", must: ["fight", "line", "offers", "gold"] },
  { name: "shop-bench", scene: "shop-bench-full", ready: "fight", must: ["fight", "line", "offers", "bench"] },
  { name: "battle", scene: "battle-win", ready: "battle-you", must: ["battle-you", "battle-them"] },
  { name: "codex", scene: "codex-units", ready: "codex-units", must: ["codex-search"] },
  { name: "forge", scene: "forge-ready", ready: "forge-ready", must: ["forge-card-0"] },
].filter((s) => !only || only.includes(s.name));
const SIZES = [
  { name: "800x450", viewport: { width: 800, height: 450 }, isMobile: true, hasTouch: true, check: true },
  { name: "821x462", viewport: { width: 821, height: 462 }, isMobile: false, hasTouch: false, check: true },
  { name: "1280x720", viewport: { width: 1280, height: 720 }, isMobile: false, hasTouch: false },
  { name: "390x844", viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true },
];

const failures = [];
const browser = await launchChromium();
try {
  for (const s of SCREENS) {
    for (const v of SIZES) {
      const ctx = await browser.newContext({ viewport: v.viewport, isMobile: v.isMobile, hasTouch: v.hasTouch, deviceScaleFactor: 1 });
      const page = await ctx.newPage();
      const why = [];
      page.on("pageerror", (e) => why.push(`pageerror: ${e.message}`));
      await page.goto(`${url}#scene=${s.scene}&lang=en`, { timeout: 120_000 }).catch((e) => why.push(`load: ${e.message.split("\n")[0]}`));
      await page.waitForSelector(`[data-testid="${s.ready}"]`, { timeout: 60_000 }).catch(() => why.push(`never showed ${s.ready}`));
      await page.waitForTimeout(1200);
      if (v.check) {
        const m = await page.evaluate((must) => {
          const d = document.scrollingElement ?? document.documentElement;
          const W = innerWidth, H = innerHeight;
          const off = must.map((id) => {
            const el = document.querySelector(`[data-testid="${id}"]`);
            if (!el) return `${id} missing`;
            const r = el.getBoundingClientRect();
            return r.top < -1 || r.left < -1 || r.bottom > H + 1 || r.right > W + 1 ? `${id} outside (${Math.round(r.left)},${Math.round(r.top)}–${Math.round(r.right)},${Math.round(r.bottom)})` : "";
          }).filter(Boolean);
          return { sh: d.scrollHeight, sw: d.scrollWidth, H, W, off };
        }, s.must);
        if (m.sh > m.H + 1) why.push(`page scrolls: ${m.sh} px tall in ${m.H}`);
        if (m.sw > m.W + 1) why.push(`page scrolls sideways: ${m.sw} px wide in ${m.W}`);
        why.push(...m.off);
      }
      await page.screenshot({ path: `${out}/${s.name}-${v.name}.png` });
      console.log(`${why.length ? "FAIL" : "ok  "} ${s.name} ${v.name}${why.length ? ": " + why.join("; ") : ""}`);
      if (why.length) failures.push(`${s.name} ${v.name}: ${why.join("; ")}`);
      await ctx.close();
    }
  }
  if (!only || only.includes("keys")) await keys();
} finally {
  await browser.close();
  child?.kill();
}
/** The shop's keys at 821×462, sent as another layout would (key ≠ code). */
async function keys() {
  const ctx = await browser.newContext({ viewport: { width: 821, height: 462 }, deviceScaleFactor: 1 });
  const page = await ctx.newPage();
  const why = [];
  page.on("pageerror", (e) => why.push(`pageerror: ${e.message}`));
  await page.goto(`${url}#scene=shop-r1&lang=en`, { timeout: 120_000 });
  await page.waitForSelector('[data-testid="fight"]', { timeout: 60_000 });
  await page.waitForTimeout(800);
  const send = (key, code) => page.evaluate(([key, code]) => document.body.dispatchEvent(new KeyboardEvent("keydown", { key, code, bubbles: true, cancelable: true })), [key, code]);
  const gold = () => page.evaluate(() => Number(document.querySelector('[data-testid="gold"]')?.textContent?.replace(/\D/g, "") ?? NaN));
  const idle = () => page.waitForFunction(() => !document.getElementById("app").classList.contains("busy"), null, { timeout: 30_000 }).then(() => page.waitForTimeout(400));
  const sheet = () => page.evaluate(() => !!document.querySelector('[data-testid="run-menu"]'));
  const g0 = await gold();
  await send("&", "Digit1"); // AZERTY's 1
  await idle();
  const g1 = await gold();
  if (!(g1 < g0)) why.push(`Digit1 (key "&") bought nothing: gold ${g0} → ${g1}`);
  await send("к", "KeyR"); // Russian R
  await idle();
  const g2 = await gold();
  if (!(g2 < g1)) why.push(`KeyR (key "к") did not reroll: gold ${g1} → ${g2}`);
  await page.keyboard.press("Escape");
  await page.waitForTimeout(400);
  if ((await sheet()) === cg) why.push(cg ? "Esc opened the run menu in the cg build" : "Esc did not open the run menu");
  if (cg) {
    await page.keyboard.press("Backspace");
    await page.waitForTimeout(400);
    if (!(await sheet())) why.push("Backspace did not open the run menu in the cg build");
  }
  if (await sheet()) await page.keyboard.press(cg ? "Backspace" : "Escape"), await page.waitForTimeout(400);
  if (await sheet()) why.push("the run menu did not close");
  await page.screenshot({ path: `${out}/keys-821x462${cg ? "-cg" : ""}.png` });
  await send(" ", "Space");
  const fought = await page.waitForSelector('[data-testid="battle-you"]', { timeout: 30_000 }).then(() => true, () => false);
  if (!fought) why.push("Space did not fight");
  console.log(`${why.length ? "FAIL" : "ok  "} keys 821x462${cg ? " cg" : ""}${why.length ? ": " + why.join("; ") : ""}`);
  if (why.length) failures.push(`keys: ${why.join("; ")}`);
  await ctx.close();
}
console.log(`${SCREENS.length} screens × ${SIZES.length} sizes, ${failures.length} failing; screenshots in ${out}`);
if (failures.length) process.exit(1);
