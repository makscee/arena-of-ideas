// M5-6: on a phone (360, 390), EN and RU, nothing that floats over the fight
// (a ±number, a stat pill, a trigger badge) covers a side's label or the
// caption, at any moment of the first fight. Plays it on a LOCAL server and
// samples the page while it plays.
// Needs `npm run mvp:build` first.   node e2e/probe-battle-floats.mjs [outdir]
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { launchChromium } from "./browser.mjs";
import { localeOf } from "./lang.mjs";
const out = process.argv[2] ?? "e2e/.shots/battle-floats";
mkdirSync(out, { recursive: true });
const port = 18000 + Math.floor(Math.random() * 1000);
const child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", STATIC_DIR: "mobile/dist" }, stdio: ["ignore", "ignore", "inherit"] });
const url = `http://127.0.0.1:${port}/arena/`;
let ok = true;
try {
  for (let i = 0; i < 120; i++) { try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {} await new Promise((r) => setTimeout(r, 500)); }
  const browser = await launchChromium();
  for (const lang of ["en", "ru"]) for (const viewport of [{ width: 360, height: 640 }, { width: 390, height: 844 }]) {
    const label = `${lang}-${viewport.width}`;
    const page = await browser.newPage({ viewport, deviceScaleFactor: 2, locale: localeOf(lang), isMobile: true, hasTouch: true });
    await page.goto(url);
    await page.getByTestId("name-input").fill(`Float${lang}${viewport.width}`);
    await page.getByTestId("name-submit").click();
    await page.getByTestId("play").click();
    for (let k = 0; k < 3; k++) {
      await page.getByTestId("offer-0").click().catch(() => {});
      await page.getByTestId("buy").click().catch(() => {});
    }
    await page.getByTestId("fight").click();
    await page.getByTestId("caption").waitFor();
    const hits = new Map();
    let shot = false;
    for (let i = 0; i < 160; i++) {
      const found = await page.evaluate(() => {
        const vis = (el) => { const r = el.getBoundingClientRect(); const s = getComputedStyle(el); return r.width > 0 && r.height > 0 && s.visibility !== "hidden" && s.display !== "none" && Number(s.opacity) > 0.05 ? r : null; };
        const targets = [...document.querySelectorAll(".bv-lab-them, .bv-lab-you, .label[class*=bv-lab], [data-testid=caption]")].map((el) => [el, vis(el)]).filter(([, r]) => r);
        const bad = [];
        for (const f of document.querySelectorAll(".bv-line *, .bv-clash *, .bv-runs *")) {
          if (f.closest(".bv-card") && !f.closest(".bv-changes, .bv-float")) continue; // the card's own face
          if (f.children.length && !f.matches(".bv-pill, .bv-badge-pill, .bv-float, .bv-run-row")) continue; // leaves and pills, not their frames
          const a = vis(f); if (!a) continue;
          for (const [el, b] of targets) if (a.left < b.right - 1 && a.right > b.left + 1 && a.top < b.bottom - 1 && a.bottom > b.top + 1) bad.push(`${String(f.className.baseVal ?? f.className).split(" ")[0] || f.tagName} "${f.textContent.trim()}" over ${el.dataset.testid ?? el.className.split(" ").slice(0, 2).join(".")}`);
        }
        return bad;
      });
      for (const b of found) hits.set(b, (hits.get(b) ?? 0) + 1);
      if (found.length && !shot) { shot = true; await page.screenshot({ path: `${out}/${label}-cover.png` }); }
      if (await page.getByTestId("end-card").isVisible().catch(() => false)) break;
      await page.waitForTimeout(50);
    }
    if (hits.size) { ok = false; console.log(`${label}: ${[...hits].map(([k, n]) => `${k} ×${n}`).join("; ")}`); }
    else console.log(`${label}: ok`);
    await page.close();
  }
  await browser.close();
} finally {
  child.kill();
}
process.exit(ok ? 0 : 1);
