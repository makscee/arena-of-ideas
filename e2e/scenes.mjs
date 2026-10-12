// The test scenes' check (#1026): opens every #scene= link headless on a
// scenes server (server/src/mvp/scenes.ts), on a phone in EN and RU and on a
// desktop where the row says its layout differs, and fails on a page error, a
// console error, a failed API call, a scene the server couldn't build or a
// blank screen. Without --url it builds the client and starts its own server.
//   npm run mvp:scenes -- [--url http://127.0.0.1:PORT/arena-lab/] [--only shop-r3,gift] [--out e2e/.shots/scenes]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/scenes";
const only = opt("only")?.split(",");
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_TUNER: "instant" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) { // up to 60 s on a loaded machine
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

const scenes = (await (await fetch(url + "api/v1/dev/scenes")).json()).filter((s) => !only || only.includes(s.id));
const views = [
  { name: "phone-en", viewport: { width: 360, height: 640 }, isMobile: true, hasTouch: true, lang: "en" },
  { name: "phone-ru", viewport: { width: 360, height: 640 }, isMobile: true, hasTouch: true, lang: "ru" },
  { name: "desktop", viewport: { width: 1280, height: 800 }, isMobile: false, hasTouch: false, lang: "en", desktopOnly: true },
];
const failures = [];
const browser = await launchChromium();
try {
  for (const s of scenes) {
    for (const v of views) {
      if (v.desktopOnly && !s.desktop) continue;
      const ctx = await browser.newContext({ viewport: v.viewport, isMobile: v.isMobile, hasTouch: v.hasTouch, deviceScaleFactor: 1 });
      const page = await ctx.newPage();
      const errs = [];
      page.on("pageerror", (e) => errs.push(`pageerror: ${e.message}`));
      page.on("console", (m) => m.type() === "error" && errs.push(`console: ${m.text()}`));
      page.on("response", (r) => r.url().includes("/api/v1/") && r.status() >= 400 && errs.push(`HTTP ${r.status()} ${new URL(r.url()).pathname}`));
      const started = Date.now();
      await page.goto(`${url}#scene=${s.id}&lang=${v.lang}`);
      // Loaded: the screen shows something other than the scene's error line.
      const shown = await page.waitForFunction(() => {
        const app = document.querySelector("#app");
        return app && app.textContent.trim().length > 20 && document.querySelector("#app h1, #app [data-testid], #app .card, #app canvas, #app svg");
      }, null, { timeout: 120_000 }).then(() => true, () => false);
      await page.waitForTimeout(1500);
      const scenErr = await page.evaluate(() => [...document.querySelectorAll("#app h1")].some((h) => h.textContent.startsWith("Scene ")));
      await page.screenshot({ path: `${out}/${s.id}-${v.name}.png` });
      const ms = Date.now() - started;
      const why = [...errs, ...(shown ? [] : ["blank screen"]), ...(scenErr ? ["the scene's error line"] : [])];
      console.log(`${why.length ? "FAIL" : "ok  "} ${s.id} ${v.name} ${ms} ms${why.length ? ": " + why.join("; ") : ""}`);
      if (why.length) failures.push(`${s.id} ${v.name}: ${why.join("; ")}`);
      await ctx.close();
    }
  }
} finally {
  await browser.close();
  child?.kill();
}
console.log(`${scenes.length} scenes, ${failures.length} failing; screenshots in ${out}`);
if (failures.length) process.exit(1);
