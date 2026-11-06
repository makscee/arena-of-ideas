// The pool's portraits in a real browser (arena#36): every unit and summoned body
// has a spec that validates, stays in its circle, fills 44 px and is alive, and no
// two share a silhouette at 44 px. Writes a contact sheet to look at.
//   npm run mvp:portraits -- [--out e2e/.shots/portraits] [--show-pairs 12]
import { mkdirSync } from "node:fs";
import { build } from "vite";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const arg = (k, d) => (args.includes(k) ? args[args.indexOf(k) + 1] : d);
const out = arg("--out", "e2e/.shots/portraits");
const showPairs = Number(arg("--show-pairs", 8));
/** Two silhouettes that overlap by more than this (intersection over union of their 44 px masks) are the same sign. */
const MAX_OVERLAP = Number(arg("--max-overlap", 0.8));
mkdirSync(out, { recursive: true });

const res = await build({
  configFile: false, logLevel: "silent",
  build: { write: false, minify: false, target: "es2022", lib: { entry: "e2e/sigil-portraits-entry.ts", name: "Sigil", formats: ["iife"], fileName: "sigil" } },
});
const code = (Array.isArray(res) ? res[0] : res).output[0].code;

const failures = [];
const browser = await launchChromium();
try {
  const page = await browser.newPage({ viewport: { width: 1000, height: 700 }, deviceScaleFactor: 2 });
  page.on("pageerror", (e) => failures.push(`page error: ${e.message}`));
  await page.setContent("<!doctype html><body style='margin:0;background:#2b2340;color:#ccc;font:11px sans-serif'><div id=sheet style='display:flex;flex-wrap:wrap;gap:4px;padding:8px'></div>");
  await page.addScriptTag({ content: code });

  const report = await page.evaluate(() => {
    const names = Object.keys(Sigil.PORTRAITS);
    const rows = names.map((name) => {
      const spec = Sigil.PORTRAITS[name];
      const v = Sigil.validate(spec);
      return { name, valid: v.ok, problems: v.problems, checks: v.ok ? Sigil.pixelChecks(spec) : [] };
    });
    // 44 px silhouettes of the idle (sleeping) form, bare spec: alpha mask
    const S = 44;
    const mask = (spec) => {
      const cv = document.createElement("canvas"); cv.width = cv.height = S;
      const x = cv.getContext("2d", { willReadFrequently: true });
      Sigil.drawSigil(x, spec, { S, frame: false, t: 0 });
      const d = x.getImageData(0, 0, S, S).data, m = new Uint8Array(S * S);
      for (let k = 0; k < S * S; k++) m[k] = d[k * 4 + 3] >= 128 ? 1 : 0;
      return m;
    };
    const masks = names.map((n) => mask(Sigil.PORTRAITS[n]));
    const pairs = [];
    for (let a = 0; a < names.length; a++) for (let b = a + 1; b < names.length; b++) {
      let i = 0, u = 0;
      for (let k = 0; k < S * S; k++) { const p = masks[a][k], q = masks[b][k]; if (p & q) i++; if (p | q) u++; }
      pairs.push({ a: names[a], b: names[b], iou: u ? i / u : 0 });
    }
    pairs.sort((p, q) => q.iou - p.iou);
    // the contact sheet: 44 px (you, them) and 96 px (awoken)
    const sheet = document.getElementById("sheet");
    names.forEach((name, k) => {
      const cell = document.createElement("div");
      cell.style.cssText = "width:112px;text-align:center";
      for (const [size, team, aw] of [[44, "you", 0], [44, "them", 0], [96, "you", k % 2]]) {
        const cv = document.createElement("canvas"); cell.append(cv);
        Sigil.mountSigil(cv, { spec: Sigil.PORTRAITS[name], S: size, team, seed: k * 0.37, aw });
      }
      const label = document.createElement("div"); label.textContent = name; cell.append(label);
      sheet.append(cell);
    });
    return { rows, pairs: pairs.slice(0, 40), count: names.length };
  });
  await page.waitForTimeout(800);

  for (const r of report.rows) {
    if (!r.valid) failures.push(`${r.name}: validate: ${r.problems.join("; ")}`);
    for (const c of r.checks) if (!c.pass) failures.push(`${r.name}: ${c.name}: ${c.why}`);
  }
  console.log(`${report.count} portraits, ${report.rows.reduce((n, r) => n + r.checks.length, 0)} pixel checks`);
  console.log("closest silhouettes at 44 px (overlap):");
  for (const p of report.pairs.slice(0, showPairs)) console.log(`  ${p.iou.toFixed(2)}  ${p.a} / ${p.b}`);
  for (const p of report.pairs) if (p.iou > MAX_OVERLAP) failures.push(`${p.a} and ${p.b} share a silhouette at 44 px (overlap ${p.iou.toFixed(2)} > ${MAX_OVERLAP})`);
  await page.screenshot({ path: `${out}/portraits-sheet.png`, fullPage: true });
} finally {
  await browser.close();
}

if (failures.length) {
  console.error("FAIL\n" + failures.map((f) => "  " + f).join("\n"));
  process.exit(1);
}
console.log("ok");
