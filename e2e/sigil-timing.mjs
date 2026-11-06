// Release gate 2 (arena#42): 10 units drawn for 5 s each in headless Chrome at 390×844
// with a 4× CPU throttle (the emulated mid-range phone Maks chose), frame time per
// unit reported. A shape budget is not a frame time, so both are printed.
//
// The unit is drawn the way the table draws it: mounted in the shared live loop
// (mobile/ui/sigil/live.ts) at the largest piece size (68 px face box × 1.12), awoken,
// hp 0.6, 3 charge, 2 shield, and a hit and a rule firing in turn every 500 ms: the
// busiest state a piece has. "frame" is the time the loop's requestAnimationFrame
// callback takes to draw what is mounted; "gap" is the time between two frames
// (16.7 ms on a 60 Hz screen when nothing is late). The 10 are the units that cost
// most per draw (a quick pass over all of them at the same throttle picks them), unless
// --units A,B,C names them. After them: the 10 together on one screen, as a battle has.
//   npm run mvp:sigil-timing -- [--seconds 5] [--rate 4] [--units A,B] [--out e2e/.shots/timing] [--budget 16.7]
import { execSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { build } from "vite";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const arg = (k, d) => (args.includes(k) ? args[args.indexOf(k) + 1] : d);
const seconds = Number(arg("--seconds", 5));
const rate = Number(arg("--rate", 4));
const budget = Number(arg("--budget", 16.7));
const named = arg("--units", "");
const out = arg("--out", "e2e/.shots/timing");
const PX = Math.round(68 * 1.12);
mkdirSync(out, { recursive: true });

const res = await build({
  configFile: false, logLevel: "silent",
  build: { write: false, minify: false, target: "es2022", lib: { entry: "e2e/sigil-portraits-entry.ts", name: "Sigil", formats: ["iife"], fileName: "sigil" } },
});
const code = (Array.isArray(res) ? res[0] : res).output[0].code;

const failures = [];
const browser = await launchChromium();
let report;
try {
  const ctx = await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true });
  const page = await ctx.newPage();
  page.on("pageerror", (e) => failures.push(`page error: ${e.message}`));
  const cdp = await ctx.newCDPSession(page);
  await page.setContent("<!doctype html><body style='margin:0;background:#2b2340'><div id=stage style='display:flex;flex-wrap:wrap;gap:4px;padding:8px'></div>");
  await page.addScriptTag({ content: code });
  // time the live loop's own callback and the gap between frames
  await page.evaluate(() => {
    const raf = window.requestAnimationFrame.bind(window);
    window.__t = { frames: [], gaps: [], last: 0, on: false };
    window.requestAnimationFrame = (f) => raf((now) => {
      const a = performance.now();
      f(now);
      const b = performance.now();
      if (window.__t.on) {
        window.__t.frames.push(b - a);
        if (window.__t.last) window.__t.gaps.push(now - window.__t.last);
        window.__t.last = now;
      }
    });
  });
  await cdp.send("Emulation.setCPUThrottlingRate", { rate });

  // which 10: the costliest per draw, measured throttled (or the named ones)
  const ranking = await page.evaluate(() => {
    const cv = document.createElement("canvas"); cv.width = cv.height = 152;
    const x = cv.getContext("2d");
    const rows = [];
    for (const [name, spec] of Object.entries(Sigil.PORTRAITS)) {
      const o = { S: 76, team: "you", aw: 1, hp: 0.6, pwr: 5, charge: 3, shield: 2, hit: 0.5, fire: 0.5 };
      for (let k = 0; k < 5; k++) Sigil.drawSigil(x, spec, { ...o, t: k * 0.11 });
      const a = performance.now();
      for (let k = 0; k < 60; k++) { x.setTransform(2, 0, 0, 2, 0, 0); x.clearRect(0, 0, 76, 76); Sigil.drawSigil(x, spec, { ...o, t: k * 0.083 }); }
      rows.push({ name, ms: (performance.now() - a) / 60, shapes: Sigil.shapeCount(spec) });
    }
    return rows.sort((p, q) => q.ms - p.ms);
  });
  const pick = named ? named.split(",").map((s) => s.trim()) : ranking.slice(0, 10).map((r) => r.name);
  const known = new Set(ranking.map((r) => r.name));
  for (const n of pick) if (!known.has(n)) failures.push(`no portrait named ${n}`);
  const byName = Object.fromEntries(ranking.map((r) => [r.name, r]));

  const stats = (xs) => {
    const s = [...xs].sort((a, b) => a - b);
    const q = (p) => s[Math.min(s.length - 1, Math.floor(p * s.length))] ?? 0;
    return { n: s.length, mean: s.reduce((a, b) => a + b, 0) / (s.length || 1), p95: q(0.95), max: s[s.length - 1] ?? 0 };
  };
  const run = async (names) => {
    await page.evaluate((names) => {
      Sigil.resetSigils();
      const stage = document.getElementById("stage"); stage.replaceChildren();
      window.__h = names.map((name, k) => {
        const cv = document.createElement("canvas"); stage.append(cv);
        return Sigil.mountSigil(cv, { spec: Sigil.PORTRAITS[name], S: 76, team: k % 2 ? "them" : "you", aw: 1, hp: 0.6, pwr: 5, charge: 3, shield: 2, seed: k * 0.37 });
      });
      let i = 0;
      window.__pulse = setInterval(() => { for (const h of window.__h) h.pulse(i % 2 ? "fire" : "hit", 450); i++; }, 500);
    }, names);
    await page.waitForTimeout(500); // warm up: first frames compile the expressions
    await page.evaluate(() => { const t = window.__t; t.frames = []; t.gaps = []; t.last = 0; t.on = true; });
    await page.waitForTimeout(seconds * 1000);
    const raw = await page.evaluate(() => { window.__t.on = false; clearInterval(window.__pulse); return { frames: window.__t.frames, gaps: window.__t.gaps }; });
    return { frame: stats(raw.frames), gap: stats(raw.gaps), late: raw.gaps.filter((g) => g > 2 * budget).length };
  };

  const load = execSync("uptime").toString().split("load averages:")[1]?.trim() ?? "?";
  const rows = [];
  for (const name of pick) rows.push({ name, shapes: byName[name]?.shapes, drawMs: byName[name]?.ms, ...(await run([name])) });
  const together = await run(pick);
  report = { rate, seconds, px: PX, load, rows, together, ranking: ranking.slice(0, 12) };
} finally {
  await browser.close();
}

const f = (x, d = 1) => (x === undefined ? "?" : x.toFixed(d));
const lines = [
  `10 units × ${seconds} s, 390×844 @2x, ${rate}× CPU throttle, sigil ${PX} px, awoken, hit/fire every 500 ms, load ${report.load}`,
  `frame budget ${budget} ms (60 Hz). frame = the loop's draw callback; gap = between frames; late = gaps over ${2 * budget} ms`,
  "",
  `${"unit".padEnd(16)} shapes  draw   frame mean   p95   max   |  gap mean   p95   max  frames  late`,
  ...report.rows.map((r) => `${r.name.padEnd(16)} ${String(r.shapes).padStart(6)} ${f(r.drawMs, 2).padStart(5)}   ${f(r.frame.mean, 2).padStart(10)} ${f(r.frame.p95).padStart(5)} ${f(r.frame.max).padStart(5)}   |  ${f(r.gap.mean).padStart(8)} ${f(r.gap.p95).padStart(5)} ${f(r.gap.max).padStart(5)}  ${String(r.frame.n).padStart(6)}  ${String(r.late).padStart(4)}`),
  `${"all 10 at once".padEnd(16)} ${"".padStart(6)} ${"".padStart(5)}   ${f(report.together.frame.mean, 2).padStart(10)} ${f(report.together.frame.p95).padStart(5)} ${f(report.together.frame.max).padStart(5)}   |  ${f(report.together.gap.mean).padStart(8)} ${f(report.together.gap.p95).padStart(5)} ${f(report.together.gap.max).padStart(5)}  ${String(report.together.frame.n).padStart(6)}  ${String(report.together.late).padStart(4)}`,
];
const text = lines.join("\n");
console.log(text);
writeFileSync(`${out}/timing.txt`, text + "\n");
writeFileSync(`${out}/timing.json`, JSON.stringify(report, null, 2) + "\n");
if (failures.length) {
  console.error("FAIL\n" + failures.map((x) => "  " + x).join("\n"));
  process.exit(1);
}
console.log(`ok: ${out}/timing.txt`);
