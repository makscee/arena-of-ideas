// The sigil engine's pixel checks (arena#35), in a real browser: every spec of
// the duel and the generic fallback stays inside its circle, fills 44 px and is
// alive; the live loop draws many canvases with one requestAnimationFrame
// callback per frame. Nothing here needs the game server: the engine is bundled
// and loaded into a blank page.
//   npm run mvp:sigil -- [--out e2e/.shots/sigil]
import { mkdirSync, readFileSync } from "node:fs";
import { build } from "vite";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const out = args.includes("--out") ? args[args.indexOf("--out") + 1] : "e2e/.shots/sigil";
mkdirSync(out, { recursive: true });

const res = await build({
  configFile: false, logLevel: "silent",
  build: { write: false, minify: false, target: "es2022", lib: { entry: "mobile/ui/sigil/browser.ts", name: "Sigil", formats: ["iife"], fileName: "sigil" } },
});
const code = (Array.isArray(res) ? res[0] : res).output[0].code;
const duel = JSON.parse(readFileSync("mobile/ui/sigil/duel-specs.json", "utf8"));
const specs = { ...duel, "(fallback)": null };

const failures = [];
const browser = await launchChromium();
try {
  const page = await browser.newPage({ viewport: { width: 800, height: 450 }, deviceScaleFactor: 2 });
  page.on("pageerror", (e) => failures.push(`page error: ${e.message}`));
  await page.setContent("<!doctype html><body style='margin:0;background:#2b2340'><div id=sheet style='display:flex;flex-wrap:wrap;gap:6px;padding:8px'></div>");
  await page.evaluate(() => {
    let n = 0, frames = 0, perFrame = 0, max = 0;
    const raf = window.requestAnimationFrame.bind(window);
    window.__raf = () => ({ calls: n, maxPerFrame: max });
    window.requestAnimationFrame = (f) => { n++; perFrame++; max = Math.max(max, perFrame); return raf((t) => { frames++; perFrame = 0; f(t); }); };
  });
  await page.addScriptTag({ content: code });

  // 1. pixel checks: the duel's specs and the fallback
  const results = await page.evaluate((specs) => {
    const rows = [];
    for (const [name, spec] of Object.entries(specs)) {
      const s = spec ?? Sigil.FALLBACK_SPEC;
      const v = Sigil.validate(s);
      rows.push({ name, valid: v.ok, problems: v.problems, checks: v.ok ? Sigil.pixelChecks(s) : [] });
    }
    return rows;
  }, specs);
  for (const r of results) {
    if (!r.valid) failures.push(`${r.name}: validate: ${r.problems.join("; ")}`);
    for (const c of r.checks) if (!c.pass) failures.push(`${r.name}: ${c.name}: ${c.why}`);
  }
  console.log(`pixel checks: ${results.length} specs, ${results.reduce((n, r) => n + r.checks.length, 0)} checks`);

  // 2. the live loop: all specs on screen at 44 px and 96 px, both sides, one rAF callback per frame
  await page.evaluate((specs) => {
    const sheet = document.getElementById("sheet");
    for (const S of [44, 96]) for (const team of ["you", "them"]) for (const [name, spec] of Object.entries(specs)) {
      const cv = document.createElement("canvas");
      cv.title = name;
      sheet.append(cv);
      Sigil.mountSigil(cv, { spec: spec ?? Sigil.FALLBACK_SPEC, S, team, seed: sheet.children.length * 0.37, aw: S === 96 && team === "you" ? 1 : 0 });
    }
  }, specs);
  await page.waitForTimeout(1200);
  const live = await page.evaluate(() => {
    const cvs = [...document.querySelectorAll("canvas")];
    const blank = cvs.filter((c) => !c.getContext("2d").getImageData(0, 0, c.width, c.height).data.some((v, i) => i % 4 === 3 && v > 0)).map((c) => c.title);
    return { canvases: cvs.length, blank, raf: window.__raf(), mounted: Sigil.mountedCount() };
  });
  console.log(`live loop: ${live.canvases} canvases, rAF ${JSON.stringify(live.raf)}`);
  if (live.mounted !== live.canvases) failures.push(`loop holds ${live.mounted} of ${live.canvases} canvases`);
  if (live.blank.length) failures.push(`blank canvases: ${live.blank.join(", ")}`);
  if (live.raf.maxPerFrame > 1) failures.push(`${live.raf.maxPerFrame} requestAnimationFrame callbacks queued in one frame (want 1)`);
  await page.screenshot({ path: `${out}/contact-sheet.png`, fullPage: true });
} finally {
  await browser.close();
}

if (failures.length) {
  console.error("FAIL\n" + failures.map((f) => "  " + f).join("\n"));
  process.exit(1);
}
console.log("ok");
