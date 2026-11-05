// The table's sigils (arena#38), in a real browser: every piece on the shop and
// battle tables draws a sigil canvas (no emoji glyph, no plate), and canvases are
// mounted and unmounted with their pieces: after many redraws the one animation
// loop draws exactly the canvases that are on the page, never more.
//   npm run mvp:sigil-table -- [--url http://127.0.0.1:PORT/arena/] [--out e2e/.shots/sigil-table]
// Without --url it builds the client and starts its own scenes server.
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/sigil-table";
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_TUNER: "instant", ARENA_CODEX: "1", ARENA_IDEA_READER: "fake", MVP_POOL: "v2" }, stdio: ["ignore", "ignore", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

const failures = [];
const check = (ok, what) => { console.log(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failures.push(what); };

/** The loop draws every mounted canvas once a frame, each with one clearRect: calls per frame is how many it draws. */
async function drawnPerFrame(page, frames = 12) {
  return page.evaluate(async (n) => {
    const w = window;
    w.__clr = 0;
    for (let i = 0; i < n; i++) await new Promise((r) => requestAnimationFrame(() => r()));
    return w.__clr / n;
  }, frames);
}
const onPage = (page) => page.evaluate(() => document.querySelectorAll("canvas.pc-sg").length);

const browser = await launchChromium();
try {
  for (const [id, view, w, h] of [["shop-r3", "phone", 390, 844], ["shop-r3", "wide", 800, 450], ["battle-six", "wide", 800, 450], ["battle-win", "phone", 390, 844]]) {
    const ctx = await browser.newContext({ viewport: { width: w, height: h }, deviceScaleFactor: 2, isMobile: w < 600, hasTouch: w < 600 });
    const page = await ctx.newPage();
    const errs = [];
    page.on("pageerror", (e) => errs.push(e.message));
    await page.addInitScript(() => {
      const clear = CanvasRenderingContext2D.prototype.clearRect;
      window.__clr = 0; window.__flash = 0; window.__ring = 0;
      CanvasRenderingContext2D.prototype.clearRect = function (...a) { window.__clr++; return clear.apply(this, a); };
      // a hit's white flash and a rule's gold ring are drawn with these colours: count them
      for (const [k, re, c] of [["fillStyle", /^rgba\(255,255,255,/, "__flash"], ["strokeStyle", /^rgba\(240,194,75,/, "__ring"]]) {
        const d = Object.getOwnPropertyDescriptor(CanvasRenderingContext2D.prototype, k);
        Object.defineProperty(CanvasRenderingContext2D.prototype, k, { ...d, set(v) { if (typeof v === "string" && re.test(v)) window[c]++; d.set.call(this, v); } });
      }
    });
    await page.goto(`${url}#scene=${id}&lang=en`, { timeout: 60_000 });
    await page.waitForSelector("canvas.pc-sg", { timeout: 60_000 });
    await page.waitForTimeout(1200);
    const name = `${id} ${view}`;
    const pieces = await page.evaluate(() => ({ pcs: document.querySelectorAll(".pc").length, withSigil: [...document.querySelectorAll(".pc")].filter((p) => p.querySelector("canvas.pc-sg")).length, emoji: document.querySelectorAll(".pc-em, .pc-em2").length }));
    check(pieces.pcs > 0 && pieces.pcs === pieces.withSigil, `${name}: every piece (${pieces.pcs}) draws a sigil`);
    check(pieces.emoji === 0, `${name}: no emoji on a piece`);
    const sizes = await page.evaluate(() => [...document.querySelectorAll("canvas.pc-sg")].map((c) => c.getBoundingClientRect().width));
    check(sizes.every((s) => s >= 30), `${name}: sigils are at least 30 px (smallest ${Math.min(...sizes).toFixed(0)})`);
    // alive: a canvas has ink
    const inked = await page.evaluate(() => [...document.querySelectorAll("canvas.pc-sg")].every((c) => { const d = c.getContext("2d").getImageData(0, 0, c.width, c.height).data; for (let i = 3; i < d.length; i += 4) if (d[i] > 0) return true; return false; }));
    check(inked, `${name}: every sigil canvas has drawn pixels`);

    // no leaks: redraw the table many times, the loop still draws what is on the page and no more
    const before = await onPage(page);
    check(Math.abs((await drawnPerFrame(page)) - before) < 0.5, `${name}: the loop draws ${before} canvases, the page has ${before}`);
    if (id.startsWith("shop")) {
      for (let i = 0; i < 6; i++) { await page.locator('[data-testid="reroll"]').click(); await page.waitForTimeout(150); }
    } else {
      for (let i = 0; i < 6; i++) { await page.keyboard.press("ArrowRight"); await page.waitForTimeout(150); }
      await page.keyboard.press("Space");
      await page.waitForTimeout(2500); // playing: the board redraws every beat
      // the battle's events reach the sigils: a hit flashes one, a rule firing bursts its ring
      for (let i = 0; i < 40 && !(await page.evaluate(() => window.__flash > 0 && window.__ring > 0)); i++) await page.waitForTimeout(500);
      const fx = await page.evaluate(() => ({ flash: window.__flash, ring: window.__ring }));
      check(fx.flash > 0, `${name}: a hit flashes a sigil (${fx.flash} frames)`);
      check(fx.ring > 0, `${name}: a rule firing bursts a ring (${fx.ring} frames)`);
    }
    await page.waitForTimeout(300);
    const after = await onPage(page);
    const loop = await drawnPerFrame(page);
    check(Math.abs(loop - after) < 0.5, `${name}: after redraws the loop draws ${loop.toFixed(1)} canvases, the page has ${after}`);
    check(errs.length === 0, `${name}: no page error${errs.length ? " (" + errs[0] + ")" : ""}`);
    await page.screenshot({ path: `${out}/${id}-${view}.png` });
    await ctx.close();
  }
  // The 3rd copy and a fusion (arena#40), on the piece gallery's states row: the awoken and the fused piece each draw
  // a sigil; the fused one is built from two parents (its canvas is not its first parent's: a sleeping one beside it).
  {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 }, deviceScaleFactor: 2 });
    const page = await ctx.newPage();
    const errs = [];
    page.on("pageerror", (e) => errs.push(e.message));
    await page.goto(`${url}#gallery`, { timeout: 60_000 });
    await page.waitForSelector('[data-testid="gallery-state"] canvas.pc-sg', { timeout: 60_000 });
    await page.waitForTimeout(1200);
    const r = await page.evaluate(() => {
      const rows = [...document.querySelectorAll('[data-testid="gallery-state"]')];
      const row = (re) => rows.find((x) => re.test(x.querySelector(".g-name").textContent));
      const L = (x) => x?.querySelector(".pc-L");
      const ink = (pc) => { const c = pc?.querySelector("canvas.pc-sg"); if (!c) return false; const d = c.getContext("2d").getImageData(0, 0, c.width, c.height).data; for (let i = 3; i < d.length; i += 4) if (d[i] > 0) return true; return false; };
      const sleeping = L(row(/^sleeping/)), awoken = L(row(/^awoken/)), fused = L(row(/^fused/));
      return {
        awokenMarked: awoken?.classList.contains("aw") ?? false, fusedMarked: fused?.classList.contains("fu") ?? false,
        ink: [sleeping, awoken, fused].map(ink), twoEmoji: document.querySelectorAll(".pc-em, .pc-em2").length,
      };
    });
    check(r.awokenMarked && r.fusedMarked, "gallery: the awoken piece is marked aw, the fused one fu");
    check(r.ink.every(Boolean), "gallery: sleeping, awoken and fused pieces all draw a sigil");
    check(r.twoEmoji === 0, "gallery: no two-emoji fusion piece");
    check(errs.length === 0, `gallery: no page error${errs.length ? " (" + errs[0] + ")" : ""}`);
    await page.locator('[data-testid="gallery-state"]').nth(1).scrollIntoViewIfNeeded();
    await page.screenshot({ path: `${out}/gallery-states.png` });
    await ctx.close();
  }
} finally {
  await browser.close();
  child?.kill();
}
console.log(failures.length ? `${failures.length} failing` : "all ok");
if (failures.length) process.exit(1);
