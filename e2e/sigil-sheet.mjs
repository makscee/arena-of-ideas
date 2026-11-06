// Release gate 1 (arena#42): a contact sheet of every unit's sigil at 44 px, sleeping
// and awoken, both sides, for a person to look at. One self-contained HTML page (the
// engine is inlined; open it anywhere) and a screenshot of it. The page is the gate:
// this script only builds it and checks that every portrait drew.
//   npm run mvp:sigil-sheet -- [--out e2e/.shots/sheet]
import { mkdirSync, writeFileSync } from "node:fs";
import { build } from "vite";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const out = args.includes("--out") ? args[args.indexOf("--out") + 1] : "e2e/.shots/sheet";
mkdirSync(out, { recursive: true });

const res = await build({
  configFile: false, logLevel: "silent",
  build: { write: false, minify: true, target: "es2022", lib: { entry: "e2e/sigil-portraits-entry.ts", name: "Sigil", formats: ["iife"], fileName: "sigil" } },
});
const code = (Array.isArray(res) ? res[0] : res).output[0].code.replaceAll("</script", "<\\/script");

const html = `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Sigil contact sheet</title>
<style>
:root { color-scheme: dark; --bg:#2b2340; --ink:#efe9ff; --dim:#a79bc9; --line:#4a3f6b; }
* { box-sizing: border-box; }
body { margin:0; background:var(--bg); color:var(--ink); font:14px/1.4 system-ui,sans-serif; }
main { max-width:1100px; margin:0 auto; padding:16px; }
h1 { font-size:18px; margin:0 0 4px; }
p { margin:0 0 12px; color:var(--dim); }
.legend, .cell .row { display:grid; grid-template-columns:repeat(4,44px); gap:8px; }
.legend { margin:0 0 12px; position:sticky; top:0; background:var(--bg); padding:8px 0; border-bottom:1px solid var(--line); z-index:1; grid-template-columns:repeat(4,1fr); max-width:420px; }
.legend span { text-align:center; font-size:12px; color:var(--dim); }
.grid { display:grid; grid-template-columns:repeat(auto-fill,minmax(220px,1fr)); gap:10px; }
.cell { border:1px solid var(--line); border-radius:8px; padding:8px; }
.cell b { display:block; font-size:13px; margin-bottom:6px; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.cell small { color:var(--dim); font-size:11px; }
.cell .row { margin-top:2px; }
canvas { display:block; }
</style></head><body><main>
<h1>Sigil contact sheet: every unit at 44 px</h1>
<p>Each row of four: sleeping (you, them), awoken (you, them). 44 px is the smallest a sigil is drawn. Look for: two that read as the same sign, one that is a blob, a side that loses its colour. <span id="n"></span></p>
<div class="legend"><span>sleeping<br>you</span><span>sleeping<br>them</span><span>awoken<br>you</span><span>awoken<br>them</span></div>
<div class="grid" id="grid"></div>
</main>
<script>${code}</script>
<script>
const names = Object.keys(Sigil.PORTRAITS);
document.getElementById("n").textContent = names.length + " portraits.";
const grid = document.getElementById("grid");
names.forEach((name, k) => {
  const spec = Sigil.PORTRAITS[name];
  const cell = document.createElement("div"); cell.className = "cell";
  const b = document.createElement("b");
  b.textContent = name + " ";
  const sm = document.createElement("small"); sm.textContent = Sigil.shapeCount(spec) + " shapes"; b.append(sm);
  const row = document.createElement("div"); row.className = "row";
  for (const [team, aw] of [["you", 0], ["them", 0], ["you", 1], ["them", 1]]) {
    const cv = document.createElement("canvas"); row.append(cv);
    Sigil.mountSigil(cv, { spec, S: 44, team, aw, seed: k * 0.37 });
  }
  cell.append(b, row); grid.append(cell);
});
</script></body></html>`;
writeFileSync(`${out}/contact-sheet.html`, html);

const failures = [];
const browser = await launchChromium();
try {
  const page = await browser.newPage({ viewport: { width: 1000, height: 800 }, deviceScaleFactor: 2 });
  page.on("pageerror", (e) => failures.push(`page error: ${e.message}`));
  await page.goto(`file://${process.cwd()}/${out}/contact-sheet.html`);
  await page.waitForTimeout(800);
  const r = await page.evaluate(() => {
    const cvs = [...document.querySelectorAll("canvas")];
    const blank = cvs.filter((c) => !c.getContext("2d").getImageData(0, 0, c.width, c.height).data.some((v, i) => i % 4 === 3 && v > 0)).length;
    return { canvases: cvs.length, cells: document.querySelectorAll(".cell").length, blank, size: cvs[0] && [cvs[0].clientWidth, cvs[0].clientHeight] };
  });
  console.log(`${r.cells} units, ${r.canvases} canvases at ${r.size?.join("×")} px, ${r.blank} blank`);
  if (r.blank) failures.push(`${r.blank} canvases drew nothing`);
  if (!r.cells || r.canvases !== r.cells * 4) failures.push("the sheet is missing cells");
  await page.screenshot({ path: `${out}/contact-sheet.png`, fullPage: true });
} finally {
  await browser.close();
}
if (failures.length) {
  console.error("FAIL\n" + failures.map((f) => "  " + f).join("\n"));
  process.exit(1);
}
console.log(`ok: ${out}/contact-sheet.html`);
