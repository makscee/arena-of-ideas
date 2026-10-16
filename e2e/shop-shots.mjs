// The shop at the four sizes the redesign is checked at (void-board#1293):
// 800×450 and 821×462 (the CrazyGames frames), a 390×844 phone, 1440×900 desktop.
//   node e2e/shop-shots.mjs --url http://127.0.0.1:PORT/arena/ [--out dir] [--scene shop-r8]
import { launchChromium } from "./browser.mjs";
import { mkdirSync } from "node:fs";

const args = process.argv.slice(2);
const opt = (n, d) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : d; };
const url = opt("url");
const out = opt("out", "e2e/.shots/shop");
const scenes = opt("scene", "shop-r8,shop-r1,shop-bench-full").split(",");
mkdirSync(out, { recursive: true });
const SIZES = [
  { name: "800", viewport: { width: 800, height: 450 }, touch: true },
  { name: "821", viewport: { width: 821, height: 462 }, touch: false },
  { name: "phone", viewport: { width: 390, height: 844 }, touch: true },
  { name: "desktop", viewport: { width: 1440, height: 900 }, touch: false },
];
const browser = await launchChromium();
try {
  for (const v of SIZES) {
    const ctx = await browser.newContext({ viewport: v.viewport, isMobile: v.touch, hasTouch: v.touch, deviceScaleFactor: 2 });
    const page = await ctx.newPage();
    page.on("pageerror", (e) => console.log(`pageerror ${v.name}: ${e.message}`));
    for (const s of scenes) {
      await page.goto("about:blank");
      await page.goto(`${url}#scene=${s}&lang=en`, { timeout: 120_000 });
      await page.waitForSelector('[data-testid="fight"]', { timeout: 60_000 });
      await page.waitForTimeout(1500);
      const over = await page.evaluate(() => ({ sw: document.documentElement.scrollWidth, sh: document.documentElement.scrollHeight, w: innerWidth, h: innerHeight }));
      console.log(`${v.name} ${s}: page ${over.sw}×${over.sh} in ${over.w}×${over.h}`);
      await page.screenshot({ path: `${out}/${s}-${v.name}.png` });
    }
    await ctx.close();
  }
} finally { await browser.close(); }
