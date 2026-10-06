// Battle beats with motion (R2-12): plays a few rounds on a phone viewport at
// 1×, times each battle from its first frame to its end card, and captures
// frames of the first beats (one every 100 ms), with motion and with
// prefers-reduced-motion. It also samples every frame of the 1× battles and
// reports how long each damage float stays visible (one cut short by the next
// wave's render would last under 0.4 s). Needs a running MVP server:
//   node e2e/beat-frames.mjs --url http://127.0.0.1:8911/arena/ [--out e2e/.shots/beats] [--rounds 3]
import { mkdirSync } from "node:fs";
import { launchChromium } from "./browser.mjs";

const arg = (k, d) => { const i = process.argv.indexOf(k); return i > 0 ? process.argv[i + 1] : d; };
const url = arg("--url");
const out = arg("--out", "e2e/.shots/beats");
const rounds = Number(arg("--rounds", "3"));
if (!url) throw new Error("--url is required");
mkdirSync(out, { recursive: true });
const browser = await launchChromium();
const times = [];
/** Per 1× battle: the playback the viewer planned (s) and what the screen took. */
const plans = [];
const floatMs = [];
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
    for (const [key, t] of open) if (!seen.has(key)) { runs.push(now - t); open.delete(key); }
    if (document.querySelector('[data-testid="battle-done"]')) { window.__floatRuns = runs; return; }
    requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
};
for (const motion of ["no-preference", "reduce"]) {
  const page = await browser.newPage({ viewport: { width: 360, height: 640 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true, reducedMotion: motion });
  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").fill(`Beats${motion === "reduce" ? "Still" : "Move"}`);
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
    const t0 = Date.now();
    if (motion !== "reduce") await page.evaluate(sampleFloats);
    if (round === rounds) for (let f = 0; f < 30; f++) { await page.screenshot({ path: `${out}/${motion === "reduce" ? "still" : "move"}-r${round}-f${String(f).padStart(2, "0")}.png` }); await page.waitForTimeout(100); }
    await page.getByTestId("battle-done").waitFor({ timeout: 300_000 });
    if (motion !== "reduce") {
      times.push((Date.now() - t0) / 1000);
      plans.push(Number(await page.locator(".bv-controls").getAttribute("data-plan-ms")) / 1000);
    }
    if (motion !== "reduce") floatMs.push(...(await page.waitForFunction(() => window.__floatRuns).then((h) => h.jsonValue())));
    await page.screenshot({ path: `${out}/${motion === "reduce" ? "still" : "move"}-r${round}-end.png` });
    await page.getByTestId("battle-done").click();
    await page.getByTestId("continue").click();
    if (await page.getByTestId("play").count()) break; // the run ended
  }
  await page.close();
}
await browser.close();
const sorted = [...floatMs].sort((p, q) => p - q);
console.log(`damage floats: ${sorted.length} seen, median ${Math.round(sorted[Math.floor(sorted.length / 2)] ?? 0)} ms visible, ${sorted.filter((ms) => ms < 400).length} under 0.4 s`);
const med = [...times].sort((p, q) => p - q)[Math.floor((times.length - 1) / 2)] ?? 0;
console.log(`planned vs on screen: ${plans.map((p, i) => `${p.toFixed(1)}→${times[i].toFixed(1)} s`).join(", ")}`);
console.log(`1× median ${med.toFixed(1)} s`);
console.log(`beat frames: ${times.length} battles at 1×, ${times.map((t) => `${t.toFixed(1)} s`).join(", ")}; frames in ${out}`);
