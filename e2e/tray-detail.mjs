// The tray's one detail surface (arena#47): at 800×450 and 390×844, tap each offer and each owned unit in turn on a scene
// and check that only the detail's words change: the line, the bench, the offers, Reroll, Fight and the detail itself keep
// their boxes; the detail lies in the tray, under the line; no page scroll. Needs a scenes server (arena-lab.sh or ARENA_SCENES=1).
//   node e2e/tray-detail.mjs --url http://127.0.0.1:PORT/arena/ [--out dir] [--scene shop-r3] [--shots]
import { launchChromium } from "./browser.mjs";
import { mkdirSync } from "node:fs";

const args = process.argv.slice(2);
const opt = (n, d) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : d; };
const url = opt("url");
if (!url) { console.error("usage: node e2e/tray-detail.mjs --url http://127.0.0.1:PORT/arena/"); process.exit(2); }
const scenes = opt("scene", "shop-r3,shop-r8").split(",");
const langs = opt("lang", "en,ru").split(",");
const out = opt("out", "e2e/.shots/tray-detail");
const shots = args.includes("--shots");
if (shots) mkdirSync(out, { recursive: true });
const SIZES = [
  { name: "800x450", viewport: { width: 800, height: 450 }, touch: true },
  { name: "390x844", viewport: { width: 390, height: 844 }, touch: true },
];
const STILL = ["line", "bench", "offers", "reroll", "fight", "inspector"];
const errors = [];
const browser = await launchChromium();
try {
  for (const v of SIZES) {
    const ctx = await browser.newContext({ viewport: v.viewport, isMobile: v.touch, hasTouch: v.touch, deviceScaleFactor: 2 });
    const page = await ctx.newPage();
    page.on("pageerror", (e) => errors.push(`${v.name}: page error ${e.message}`));
    for (const [scene, lang] of scenes.flatMap((sc) => langs.map((l) => [sc, l]))) {
      const at = `${v.name} ${scene} ${lang}`;
      await page.goto("about:blank");
      await page.goto(`${url}#scene=${scene}&lang=${lang}`, { timeout: 120_000 });
      await page.waitForSelector('[data-testid="fight"]', { timeout: 60_000 });
      await page.waitForTimeout(1200);
      const boxes = () => page.evaluate((ids) => Object.fromEntries(ids.map((id) => { const r = document.querySelector(`[data-testid="${id}"]`)?.getBoundingClientRect(); return [id, r ? [r.x, r.y, r.width, r.height].map((n) => Math.round(n * 10) / 10) : null]; })), STILL);
      const idle = await boxes();
      const texts = new Set([await page.getByTestId("inspector").innerText()]);
      const targets = await page.evaluate(() => [...document.querySelectorAll('[data-testid^="offer-"]:not([data-testid^="offer-gap"]), [data-testid^="line-"]:not([data-testid$="-empty"]), [data-testid^="bench-"]:not([data-testid$="-empty"])')].map((e) => e.getAttribute("data-testid")));
      let n = 0;
      for (const id of targets) {
        await page.getByTestId(id).click();
        await page.waitForTimeout(350);
        const now = await boxes();
        for (const k of STILL) {
          const [a, b] = [idle[k], now[k]];
          if (!a || !b || a.some((x, i) => Math.abs(x - b[i]) > 0.6)) errors.push(`${at}: tapping ${id} moved or resized ${k} (${JSON.stringify(a)} → ${JSON.stringify(b)})`);
        }
        const d = now.inspector, line = now.line, tray = await page.getByTestId("tray").boundingBox();
        if (d && line && d[1] < line[1] + line[3] - 0.5) errors.push(`${at}: the detail covers the line when ${id} is tapped`);
        if (d && tray && (d[0] < tray.x - 0.5 || d[1] < tray.y - 0.5 || d[0] + d[2] > tray.x + tray.width + 0.5 || d[1] + d[3] > tray.y + tray.height + 0.5)) errors.push(`${at}: the detail is outside the tray when ${id} is tapped`);
        const t = await page.getByTestId("inspector").innerText();
        if (!t.trim()) errors.push(`${at}: the detail is empty with ${id} tapped`);
        texts.add(t);
        // No text under 11 px at 800×450 (the stage's scale times its font), and the detail's words and buttons fit their boxes (RU too).
        const bad = await page.evaluate(() => {
          const k = parseFloat(document.querySelector(".stage").style.transform.replace(/[^0-9.]/g, "")) || 1;
          const out = [];
          for (const e of document.querySelectorAll(".stage *")) {
            if (e.children.length || !e.textContent.trim()) continue;
            const r = e.getBoundingClientRect();
            if (!r.width || !r.height) continue;
            const fs = parseFloat(getComputedStyle(e).fontSize) * k;
            if (innerWidth > 600 ? fs < 10.95 : fs < 11) out.push(`${fs.toFixed(1)}px "${e.textContent.trim().slice(0, 18)}"`);
          }
          const det = document.querySelector('[data-testid="inspector"]');
          for (const b of det.querySelectorAll(":scope > .row > button")) if (b.scrollWidth > b.clientWidth + 1) out.push(`button "${b.textContent.trim().slice(0, 18)}" overflows`);
          if (det.scrollWidth > det.clientWidth + 1) out.push("the detail overflows sideways");
          return out;
        });
        if (v.viewport.width === 800 && bad.length) errors.push(`${at}: ${id}: ${[...new Set(bad)].join("; ")}`);
        else if (bad.some((b) => /overflow/.test(b))) errors.push(`${at}: ${id}: ${bad.filter((b) => /overflow/.test(b)).join("; ")}`);
        const sc = await page.evaluate(() => ({ sw: document.documentElement.scrollWidth, sh: document.documentElement.scrollHeight, w: innerWidth, h: innerHeight }));
        if (sc.sw > sc.w || sc.sh > sc.h) errors.push(`${at}: the page scrolls (${sc.sw}×${sc.sh} in ${sc.w}×${sc.h})`);
        if (shots) await page.screenshot({ path: `${out}/${scene}-${v.name}-${id}.png` });
        n++;
      }
      if (texts.size < Math.min(3, targets.length)) errors.push(`${at}: the detail's words did not change from tap to tap (${texts.size} distinct)`);
      console.log(`${at}: ${n} taps, ${texts.size} distinct texts`);
    }
    await ctx.close();
  }
} finally { await browser.close(); }
if (errors.length) { console.log(errors.map((e) => `FAIL ${e}`).join("\n")); process.exit(1); }
console.log("tray detail: only the words change at both sizes");
