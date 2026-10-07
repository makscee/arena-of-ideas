// Battle beats with motion (R2-12): plays a few rounds on a phone viewport at
// any speed (the one the viewer gets, or --speed 1|2|4; R2-17), times each
// battle from its first frame to its end card against its plan at that
// speed, and captures
// frames of the first beats (one every 100 ms), with motion and with
// prefers-reduced-motion. It also samples every frame of the 1× battles and
// reports how long each damage float stays visible (one cut short by the next
// wave's render would last under 0.5 s at 1×, under 0.5 s / speed faster:
// round 3's float runs 0.9 s, visible for about 0.75 s). It prints the
// median battle's length on screen at each speed it played (round 3, note 14:
// about 38 s at 1×, 19 s at 2×).
// Each run registers its own names (names are unique per server). Needs a
// running MVP server:
//   node e2e/beat-frames.mjs --url http://127.0.0.1:8911/arena/ [--out e2e/.shots/beats] [--rounds 3] [--speed 2]
import { mkdirSync } from "node:fs";
import { launchChromium } from "./browser.mjs";

const arg = (k, d) => { const i = process.argv.indexOf(k); return i > 0 ? process.argv[i + 1] : d; };
const url = arg("--url");
const out = arg("--out", "e2e/.shots/beats");
const rounds = Number(arg("--rounds", "3"));
/** 0: the speed the viewer gets (1×, 2× from round 4); else click to it. */
const wantSpeed = Number(arg("--speed", "0"));
const TAG = Date.now().toString(36).slice(-4);
if (!url) throw new Error("--url is required");
mkdirSync(out, { recursive: true });
const browser = await launchChromium();
const times = [];
/** Per battle with motion: the playback planned at its speed (s), what the
 * screen took, and the speed. */
const plans = [];
const speeds = [];
/** Damage floats' visible runs, each as a fraction of its battle's 1× float (ms × speed); runs that grew into the next wave's float, apart. */
const floatMs = [];
const mergedMs = [];
/** A damage float visible for less than this (at 1×) was cut short, unless the next float on its card replaced it. */
const FLOAT_MIN_MS = 500;
/** In the page: every animation frame, which damage floats show (opacity > 0.3),
 * keyed by card and label; a key's visible run ends when it stops showing. */
const sampleFloats = () => {
  const runs = [];
  const open = new Map();
  const tick = () => {
    const now = performance.now();
    const seen = new Set();
    for (const f of document.querySelectorAll(".bv-float.damage")) {
      if (Number(getComputedStyle(f).opacity) <= 0.3) continue;
      const key = `${f.closest(".bv-card")?.dataset.unit ?? "?"}|${f.textContent}`;
      seen.add(key);
      if (!open.has(key)) open.set(key, now);
    }
    for (const [key, t] of open) if (!seen.has(key)) { runs.push({ card: key.split("|")[0], ms: now - t, end: now }); open.delete(key); }
    // A float that grows ("−2" → "−5" as the next wave hits the same card) ends one key and
    // starts the next on that card: that is a merge, not a float cut short.
    if (document.querySelector('[data-testid="battle-done"]')) { window.__floatRuns = runs.map((r) => ({ ms: r.ms, merged: runs.some((x) => x !== r && x.card === r.card && x.end - x.ms >= r.end - 120 && x.end - x.ms <= r.end + 200) })); return; }
    requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
};
for (const motion of ["no-preference", "reduce"]) {
  const page = await browser.newPage({ viewport: { width: 360, height: 640 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true, reducedMotion: motion });
  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").fill(`Beats${motion === "reduce" ? "Still" : "Move"}${TAG}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").click();
  for (let round = 1; round <= rounds; round++) {
    await page.getByTestId("fight").waitFor({ timeout: 10_000 });
    for (let k = 0; k < 3; k++) {
      if ((await page.getByTestId("gold").count()) === 0) break;
      const gold = Number((await page.getByTestId("gold").textContent()).replace("g", ""));
      if (gold < 3 || (await page.getByTestId("line").locator(".card.you").count()) >= 5) break;
      await page.getByTestId("offer-0").click();
      if (await page.getByTestId("buy").isDisabled()) { await page.getByTestId("offer-close").click(); break; }
      await page.getByTestId("buy").click();
      await page.waitForFunction((g) => document.querySelector('[data-testid="gold"]')?.textContent !== `${g}g`, gold);
    }
    await page.getByTestId("fight").click();
    await page.getByTestId("battle-end").waitFor({ timeout: 10_000 });
    const speedOf = async () => Number(((await page.getByTestId("battle-speed").textContent()) ?? "1").replace("×", ""));
    for (let i = 0; wantSpeed && i < 3 && (await speedOf()) !== wantSpeed; i++) await page.getByTestId("battle-speed").click();
    const speed = await speedOf();
    const t0 = Date.now();
    if (motion !== "reduce") await page.evaluate(sampleFloats);
    if (round === rounds) for (let f = 0; f < 30; f++) { await page.screenshot({ path: `${out}/${motion === "reduce" ? "still" : "move"}-r${round}-f${String(f).padStart(2, "0")}.png` }); await page.waitForTimeout(100); }
    await page.getByTestId("battle-done").waitFor({ timeout: 300_000 });
    if (motion !== "reduce") {
      times.push((Date.now() - t0) / 1000);
      plans.push(Number(await page.locator(".bv-controls").getAttribute("data-plan-ms")) / 1000 / speed);
      speeds.push(speed);
    }
    if (motion !== "reduce") for (const r of await page.waitForFunction(() => window.__floatRuns).then((h) => h.jsonValue())) (r.merged && r.ms * speed < FLOAT_MIN_MS ? mergedMs : floatMs).push(r.ms * speed);
    await page.screenshot({ path: `${out}/${motion === "reduce" ? "still" : "move"}-r${round}-end.png` });
    await page.getByTestId("battle-done").click();
    // Done goes straight on to the next shop, or to the run's end.
    await page.waitForFunction(() => document.querySelector('[data-testid="fight"]') || document.querySelector('[data-testid="run-over"]'));
    if (await page.getByTestId("run-over").isVisible().catch(() => false)) break; // the run ended
  }
  await page.close();
}
await browser.close();
const sorted = [...floatMs].sort((p, q) => p - q);
// Float times are scaled to 1× (ms × speed), so the bar holds at any speed.
const cut = sorted.filter((ms) => ms < FLOAT_MIN_MS).length;
console.log(`damage floats: ${sorted.length} seen, median ${Math.round(sorted[Math.floor(sorted.length / 2)] ?? 0)} ms visible at 1×, ${cut} under ${FLOAT_MIN_MS / 1000} s at 1× (${sorted.filter((ms) => ms < FLOAT_MIN_MS).map((ms) => Math.round(ms)).join(", ")}); ${mergedMs.length} grew into the next wave's float`);
const med = [...times].sort((p, q) => p - q)[Math.floor((times.length - 1) / 2)] ?? 0;
console.log(`planned vs on screen: ${plans.map((p, i) => `${p.toFixed(1)}→${times[i].toFixed(1)} s at ${speeds[i]}×`).join(", ")}`);
console.log(`median ${med.toFixed(1)} s`);
for (const sp of [...new Set(speeds)].sort()) {
  const at = times.filter((_, i) => speeds[i] === sp).sort((p, q) => p - q);
  console.log(`at ${sp}×: ${at.length} battles, median ${(at[Math.floor((at.length - 1) / 2)] ?? 0).toFixed(1)} s on screen`);
}
// Every planned playback should run as planned (within 15% and 1.5 s): motion never slows the timer.
const late = plans.filter((p, i) => times[i] > p * 1.15 + 1.5);
if (late.length) { console.log(`${late.length} battles ran long against their plan`); process.exitCode = 1; }
if (cut) process.exitCode = 1;
console.log(`beat frames: ${times.length} battles (${speeds.map((sp) => `${sp}×`).join(", ")}), ${times.map((t) => `${t.toFixed(1)} s`).join(", ")}; frames in ${out}`);
