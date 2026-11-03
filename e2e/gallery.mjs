// The piece gallery (void-board#1291, mission #1286 slice 1): `#gallery` at
// 800×450, 821×462, 390×844 and 1280×720. Every unit of the pool shows at the
// four sizes (L, M, S, XS); no piece spills out of its cell into a neighbour,
// and no when word is cut. Screenshots of the whole page and of each size
// alone go to out/gallery/. Without --url it builds the client and starts its
// own server on the rules v2 pool.
//   npm run mvp:gallery -- [--url http://127.0.0.1:PORT/arena/]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", MVP_POOL: "v2" }, stdio: ["ignore", "ignore", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

const OUT = "out/gallery";
mkdirSync(OUT, { recursive: true });
const SIZES = [
  { name: "800x450", viewport: { width: 800, height: 450 } },
  { name: "821x462", viewport: { width: 821, height: 462 } },
  { name: "390x844", viewport: { width: 390, height: 844 }, mobile: true },
  { name: "1280x720", viewport: { width: 1280, height: 720 } },
];
const failures = [];
const browser = await launchChromium();
try {
  for (const v of SIZES) {
    const ctx = await browser.newContext({ viewport: v.viewport, isMobile: !!v.mobile, hasTouch: !!v.mobile, deviceScaleFactor: 2 });
    const page = await ctx.newPage();
    const why = [];
    page.on("pageerror", (e) => why.push(`pageerror: ${e.message}`));
    await page.goto(url + "#gallery");
    await page.waitForSelector('[data-testid="gallery-unit"]', { timeout: 15000 });
    await page.evaluate(() => document.fonts.ready);
    const r = await page.evaluate(() => {
      const units = [...document.querySelectorAll('[data-testid="gallery-unit"]')];
      const bad = [];
      for (const row of units) {
        const sizes = [...row.querySelectorAll(".pc")].map((p) => p.className.match(/pc-(L|M|S|XS)/)?.[1]);
        if (sizes.join() !== "L,M,S,XS") bad.push(`${row.dataset.unit}: sizes ${sizes.join()}`);
      }
      // every part of a piece stays inside its cell, and cells never overlap
      const cells = [...document.querySelectorAll(".g-cell")];
      for (const cell of cells) {
        const c = cell.getBoundingClientRect();
        for (const part of cell.querySelectorAll(".pc > *")) {
          const b = part.getBoundingClientRect();
          if (b.left < c.left - 1 || b.right > c.right + 1 || b.top < c.top - 1 || b.bottom > c.bottom + 1)
            bad.push(`${cell.querySelector(".pc").dataset.name} ${cell.className}: ${part.className} out of its cell`);
        }
      }
      for (const tab of document.querySelectorAll(".pc-tab .tw, .pc-nm")) if (tab.scrollWidth > tab.clientWidth + 1) bad.push(`cut: ${tab.textContent}`);
      const wide = document.documentElement.scrollWidth > innerWidth + 1;
      return { units: units.length, bad, wide };
    });
    if (r.units < 60) why.push(`only ${r.units} units`);
    if (r.wide) why.push("the page scrolls sideways");
    why.push(...r.bad.slice(0, 20));
    await page.screenshot({ path: `${OUT}/gallery-${v.name}.png` });
    // the whole gallery in parts of 2000 px: a taller viewport, scrolled
    // (one capture at 2× can't be taller than the GPU's texture)
    const full = Math.ceil(await page.evaluate(() => document.querySelector(".gallery").getBoundingClientRect().height + 40));
    const step = Math.min(2000, full);
    await page.setViewportSize({ width: v.viewport.width, height: step });
    for (let y = 0, part = 1; y < full; y += step, part++) {
      await page.evaluate((top) => { scrollTo(0, top); document.getElementById("app").scrollTop = top; }, y);
      await page.screenshot({ path: `${OUT}/gallery-${v.name}-full-${part}.png` });
    }
    console.log(`${why.length ? "FAIL" : "ok  "} ${v.name}: ${r.units} units${why.length ? "\n  " + why.join("\n  ") : ""}`);
    if (why.length) failures.push(v.name);
    await ctx.close();
  }
} finally {
  await browser.close();
  child?.kill();
}
console.log(failures.length ? `gallery: ${failures.length} size(s) failed` : `gallery: ok at ${SIZES.length} sizes (${OUT}/)`);
process.exit(failures.length ? 1 : 0);
