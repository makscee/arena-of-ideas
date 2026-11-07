// The glyphs in a real browser (arena#41): every name in the closed set has a spec that
// validates, draws something with an ink outline at the chip size, and no two share a
// silhouette at 16 px. Writes a contact sheet (each glyph on the tab and badge colours, at
// 11, 16 and 32 px) to look at.
//   npm run mvp:sigil-glyphs -- [--out e2e/.shots/glyphs]
import { mkdirSync } from "node:fs";
import { build } from "vite";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const arg = (k, d) => (args.includes(k) ? args[args.indexOf(k) + 1] : d);
const out = arg("--out", "e2e/.shots/glyphs");
const MAX_OVERLAP = Number(arg("--max-overlap", 0.6));
mkdirSync(out, { recursive: true });

const res = await build({
  configFile: false, logLevel: "silent",
  build: { write: false, minify: false, target: "es2022", lib: { entry: "e2e/sigil-glyphs-entry.ts", name: "Sigil", formats: ["iife"], fileName: "sigil" } },
});
const code = (Array.isArray(res) ? res[0] : res).output[0].code;
const failures = [];
const browser = await launchChromium();
try {
  const page = await browser.newPage({ viewport: { width: 980, height: 640 }, deviceScaleFactor: 3 });
  page.on("pageerror", (e) => failures.push(`page error: ${e.message}`));
  await page.setContent("<!doctype html><body style='margin:0;background:#2b2340;color:#ccc;font:11px sans-serif'><div id=sheet style='display:flex;flex-wrap:wrap;gap:6px;padding:8px'></div>");
  await page.addScriptTag({ content: code });
  const report = await page.evaluate(() => {
    const names = [...Sigil.GLYPH_NAMES];
    const TONES = ["#ffd36a", "#a3a8f7", "#f7a184", "#f2a65a", "#f5a3b0", "#d3cbe6", "#7fdcc0", "#c8b4f0", "#9fe3b4", "#9cc7f2", "#5aa0e8", "#d65db1", "#3fbfd6", "#f0953e", "#e2546b", "#9a6be0", "#47b96f", "#8b7fb0"];
    const rows = [];
    const draw = (spec, px, bg) => {
      const dpr = 3, P = px;
      const cv = document.createElement("canvas"); cv.width = cv.height = P * dpr; cv.style.cssText = `width:${P}px;height:${P}px;background:${bg};border-radius:${P / 4}px`;
      const x = cv.getContext("2d", { willReadFrequently: true });
      x.setTransform(dpr, 0, 0, dpr, 0, 0); x.translate(-P * 0.125, -P * 0.125);
      Sigil.drawSigil(x, spec, { S: P * 1.25, frame: false, t: 0 });
      return cv;
    };
    const mask = (spec) => { // colour mask at 16 px (1x): a pixel is its quantised colour, 0 when empty; plus how much of the box it covers and how much of it is ink
      const S = 16, cv = document.createElement("canvas"); cv.width = cv.height = S;
      const x = cv.getContext("2d", { willReadFrequently: true });
      x.translate(-S * 0.125, -S * 0.125);
      Sigil.drawSigil(x, spec, { S: S * 1.25, frame: false, t: 0 });
      const d = x.getImageData(0, 0, S, S).data, m = new Uint16Array(S * S);
      let cover = 0, ink = 0;
      for (let k = 0; k < S * S; k++) { if (d[k * 4 + 3] >= 128) { m[k] = 1 + (d[k * 4] >> 6) * 16 + (d[k * 4 + 1] >> 6) * 4 + (d[k * 4 + 2] >> 6); cover++; if (d[k * 4] < 70 && d[k * 4 + 1] < 60) ink++; } }
      return { m, cover: cover / (S * S), ink: cover ? ink / cover : 0 };
    };
    const sheet = document.getElementById("sheet");
    const masks = [];
    names.forEach((name, k) => {
      const spec = Sigil.GLYPH_SPECS[name];
      const v = spec ? Sigil.validate(spec) : { ok: false, problems: ["no spec"] };
      const mk = spec ? mask(spec) : { m: new Uint16Array(256), cover: 0, ink: 0 };
      masks.push(mk.m);
      rows.push({ name, ok: v.ok, problems: v.problems, shapes: spec ? Sigil.shapeCount(spec) : 0, cover: mk.cover, ink: mk.ink });
      const cell = document.createElement("div"); cell.style.cssText = "width:150px;text-align:center";
      if (spec) {
        const line = document.createElement("div"); line.style.cssText = "display:flex;gap:4px;justify-content:center;align-items:center;margin-bottom:2px";
        line.append(draw(spec, 11, TONES[k % TONES.length]), draw(spec, 16, TONES[(k + 5) % TONES.length]), draw(spec, 24, TONES[(k + 9) % TONES.length]), draw(spec, 44, "#3a3153"));
        cell.append(line);
      }
      const t = document.createElement("div"); t.textContent = name; cell.append(t);
      sheet.append(cell);
    });
    const pairs = [];
    for (let a = 0; a < names.length; a++) for (let b = a + 1; b < names.length; b++) {
      let i = 0, u = 0;
      for (let q = 0; q < 256; q++) { const p = masks[a][q], r = masks[b][q]; if (p && p === r) i++; if (p || r) u++; }
      pairs.push({ a: names[a], b: names[b], iou: u ? i / u : 0 });
    }
    pairs.sort((p, q) => q.iou - p.iou);
    return { rows, pairs: pairs.slice(0, 20) };
  });
  for (const r of report.rows) {
    if (!r.ok) failures.push(`${r.name}: validate: ${r.problems.join("; ")}`);
    if (r.cover < 0.3) failures.push(`${r.name}: covers only ${(r.cover * 100).toFixed(0)}% of its 16 px box (needs 30%)`);
    if (r.cover > 0.95) failures.push(`${r.name}: fills its box edge to edge (${(r.cover * 100).toFixed(0)}%): the outline is clipped`);
    if (r.ink < 0.12) failures.push(`${r.name}: only ${(r.ink * 100).toFixed(0)}% of it is ink outline at 16 px (needs 12%)`);
  }
  console.log(`${report.rows.length} glyphs`);
  console.log("closest pairs at 16 px (pixels the same shape and colour):");
  for (const p of report.pairs.slice(0, 6)) console.log(`  ${p.iou.toFixed(2)}  ${p.a} / ${p.b}`);
  for (const p of report.pairs) if (p.iou > MAX_OVERLAP) failures.push(`${p.a} and ${p.b} look alike at 16 px (same shape and colour in) (overlap ${p.iou.toFixed(2)} > ${MAX_OVERLAP})`);
  await page.screenshot({ path: `${out}/glyphs-sheet.png`, fullPage: true });
} finally {
  await browser.close();
}
if (failures.length) { console.error("FAIL\n" + failures.map((f) => "  " + f).join("\n")); process.exit(1); }
console.log("ok");
