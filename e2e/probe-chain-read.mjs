// #913: a fight that opens on a long Start chain (a champion's), played at 1×
// on a phone (360) and a desktop (1280). Measures the jank (frames over
// 50 ms, long tasks, render ms) while the chain plays, saves a frame
// sequence, and checks the chain reads: a step chip on the acting card,
// "Chain n/N" on the caption, the chain's summary, and Skip chain.
// Needs `npm run mvp:build` first.   node e2e/probe-chain-read.mjs [outdir] [--measure-only]
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { launchChromium } from "./browser.mjs";
import { localeOf } from "./lang.mjs";
const args = process.argv.slice(2);
const measureOnly = args.includes("--measure-only");
const out = args.find((x) => !x.startsWith("--")) ?? "e2e/.shots/chain-read";
mkdirSync(out, { recursive: true });
const rec = join(mkdtempSync(join(tmpdir(), "chain-read-")), "chain.json");
execFileSync("node", ["--import", "tsx/esm", "e2e/long-chain-battle.ts", rec], { stdio: ["ignore", "inherit", "inherit"] });
const chained = JSON.parse(readFileSync(rec, "utf8"));
const port = 18000 + Math.floor(Math.random() * 1000);
const child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", STATIC_DIR: "mobile/dist" }, stdio: ["ignore", "ignore", "inherit"] });
const url = `http://127.0.0.1:${port}/arena/`;
let ok = true;
const fail = (m) => { ok = false; console.log(`  FAIL ${m}`); };
try {
  for (let i = 0; i < 120; i++) { try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {} await new Promise((r) => setTimeout(r, 500)); }
  const browser = await launchChromium();
  const runs = [
    { label: "phone-360-en", lang: "en", viewport: { width: 360, height: 640 }, mobile: true },
    { label: "phone-360-ru", lang: "ru", viewport: { width: 360, height: 640 }, mobile: true },
    { label: "desktop-1280-en", lang: "en", viewport: { width: 1280, height: 800 }, mobile: false },
  ];
  for (const r of runs) {
    const page = await browser.newPage({ viewport: r.viewport, deviceScaleFactor: r.mobile ? 2 : 1, locale: localeOf(r.lang), isMobile: r.mobile, hasTouch: r.mobile });
    await page.addInitScript(() => {
      try { localStorage.setItem("arena.battleSpeed", "1"); } catch {}
      // Frame gaps and long tasks, from the fight's first caption on.
      window.__perf = { gaps: [], long: [] };
      let last = 0;
      const tick = (t) => { if (window.__perfOn) { if (last) window.__perf.gaps.push(t - last); last = t; } else last = 0; requestAnimationFrame(tick); };
      requestAnimationFrame(tick);
      try { new PerformanceObserver((l) => { if (window.__perfOn) for (const e of l.getEntries()) window.__perf.long.push(e.duration); }).observe({ type: "longtask", buffered: false }); } catch {}
    });
    await page.route((u) => /\/battles\/[^/]+$/.test(u.pathname), async (route) => {
      const res = await route.fetch();
      const body = await res.json();
      await route.fulfill({ response: res, json: { ...body, ...chained, battleId: body.battleId, player: body.player, runId: body.runId, kind: body.kind, round: body.round } });
    });
    await page.goto(url);
    await page.getByTestId("name-input").fill(`Chain${r.label.replace(/\W/g, "")}`.slice(0, 16));
    await page.getByTestId("name-submit").click();
    await page.getByTestId("play").click();
    await page.getByTestId("offer-0").click(); await page.getByTestId("offer-0").click();
    await page.getByTestId("buy").click();
    await page.getByTestId("fight").click();
    await page.getByTestId("caption").waitFor();
    // A phone's CPU: 4× slower than this Mac.
    const cdp = await page.context().newCDPSession(page);
    await cdp.send("Emulation.setCPUThrottlingRate", { rate: 4 });
    if (process.env.PROFILE) { await cdp.send("Profiler.enable"); await cdp.send("Profiler.start"); }
    await cdp.send("Performance.enable");
    const metric = async () => Object.fromEntries((await cdp.send("Performance.getMetrics")).metrics.map((m) => [m.name, m.value]));
    const m0 = await metric();
    await page.evaluate(() => { window.__perfOn = true; window.__bvRenders = []; });
    const seen = { step: 0, chip: 0, summary: "", maxN: 0 };
    const t0 = Date.now();
    let f = 0;
    // The Start chain, frame by frame (every ~350 ms), up to 14 s.
    while (Date.now() - t0 < 14000) {
      const s = await page.evaluate(() => ({
        cap: document.querySelector("[data-testid=caption]")?.textContent ?? "",
        chip: document.querySelectorAll("[data-testid=chain-chip]").length,
        n: document.querySelector("[data-testid=caption-chain]")?.textContent ?? "",
        sum: document.querySelector("[data-testid=chain-summary]")?.textContent ?? "",
      }));
      if (Date.now() - t0 > 9000) await page.evaluate(() => { window.__perfOn = false; });
      if (s.n) seen.step++;
      if (s.chip) seen.chip++;
      const m = /(\d+)\s*\/\s*(\d+)/.exec(s.n);
      if (m) seen.maxN = Math.max(seen.maxN, Number(m[1]));
      if (s.sum && !seen.summary) seen.summary = s.sum;
      if (s.sum) break;
      await page.waitForTimeout(350);
    }
    const perf = await page.evaluate(() => ({ ...window.__perf, renders: window.__bvRenders ?? [] }));
    const m1 = await metric();
    await cdp.send("Emulation.setCPUThrottlingRate", { rate: 1 });
    if (process.env.PROFILE) {
      // Self time by function (a dev aid): where a redraw's time goes.
      const { profile } = await cdp.send("Profiler.stop");
      const self = new Map();
      const byId = new Map(profile.nodes.map((n) => [n.id, n]));
      const dt = profile.timeDeltas;
      profile.samples.forEach((id, i) => { const n = byId.get(id); const k = `${n.callFrame.functionName || "(anon)"}:${n.callFrame.lineNumber}`; self.set(k, (self.get(k) ?? 0) + (dt[i] ?? 0)); });
      console.log("  top self:", [...self].sort((p, q) => q[1] - p[1]).slice(0, 14).map(([k, v]) => `${k} ${Math.round(v / 1000)}ms`).join(", "));
    }
    const gaps = perf.gaps.slice().sort((p, q) => p - q);
    const p = (q) => Math.round(gaps[Math.floor(q * (gaps.length - 1))] ?? 0);
    const rs = perf.renders.slice().sort((p, q) => p - q);
    console.log(`${r.label}: frames ${gaps.length}, over 50 ms ${gaps.filter((g) => g > 50).length}, p95 ${p(0.95)} ms, max ${p(1)} ms; long tasks ${perf.long.length} (${Math.round(perf.long.reduce((s, x) => s + x, 0))} ms); layouts ${m1.LayoutCount - m0.LayoutCount} (${Math.round((m1.LayoutDuration - m0.LayoutDuration) * 1000)} ms), style recalcs ${m1.RecalcStyleCount - m0.RecalcStyleCount}; render ${rs.length} × ${rs.length ? `median ${rs[rs.length >> 1].toFixed(1)} ms, max ${rs.at(-1).toFixed(1)} ms` : "n/a"}`);
    console.log(`  chain: steps seen ${seen.maxN}, chip frames ${seen.chip}, summary "${seen.summary.trim()}"`);
    // A frame sequence of the Start chain, from a replay (a screenshot stalls the page, so not while measuring).
    await page.getByTestId("battle-replay").click().catch(() => {});
    for (let f = 0; f < 14; f++) {
      await page.waitForTimeout(f ? 650 : 1200);
      await page.screenshot({ path: `${out}/${r.label}-${String(f).padStart(2, "0")}.png` });
      if (await page.getByTestId("chain-summary").isVisible().catch(() => false)) break;
    }
    if (!measureOnly) {
      if (seen.maxN < 4) fail("no 'Chain n/N' on the caption");
      if (!seen.chip) fail("no step chip on an acting card");
      if (!seen.summary) fail("no chain summary after the Start chain");
      if (r.lang === "ru" && seen.summary && /[A-Za-z]{4,}/.test(seen.summary.replace(/@\S+/g, ""))) fail(`summary not in Russian: ${seen.summary}`);
      // Skip chain: Replay, then skip the Start chain at once.
      await page.getByTestId("battle-replay").click().catch(() => {});
      const skip = page.getByTestId("chain-skip");
      try {
        await skip.waitFor({ state: "visible", timeout: 5000 });
        await skip.click();
        await page.getByTestId("chain-summary").waitFor({ timeout: 3000 });
        console.log("  skip chain: OK");
      } catch { fail("Skip chain missing or didn't reach the summary"); }
      await page.waitForTimeout(700); // the summary fades in
      await page.screenshot({ path: `${out}/${r.label}-skip.png` });
    }
    await page.close();
  }
  await browser.close();
} finally {
  child.kill();
}
console.log(ok ? "chain read probe: OK" : "chain read probe: FAILED");
process.exitCode = ok ? 0 : 1;
