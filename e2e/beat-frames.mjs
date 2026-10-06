// Battle beats with motion (R2-12): plays a few rounds on a phone viewport at
// 1×, times each battle from its first frame to its end card, and captures
// frames of the first beats (one every 100 ms), with motion and with
// prefers-reduced-motion. Needs a running MVP server:
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
    await page.getByTestId("battle-skip").waitFor({ timeout: 10_000 });
    const t0 = Date.now();
    if (round === rounds) for (let f = 0; f < 30; f++) { await page.screenshot({ path: `${out}/${motion === "reduce" ? "still" : "move"}-r${round}-f${String(f).padStart(2, "0")}.png` }); await page.waitForTimeout(100); }
    await page.getByTestId("battle-done").waitFor({ timeout: 300_000 });
    if (motion !== "reduce") times.push((Date.now() - t0) / 1000);
    await page.screenshot({ path: `${out}/${motion === "reduce" ? "still" : "move"}-r${round}-end.png` });
    await page.getByTestId("battle-done").click();
    await page.getByTestId("continue").click();
    if (await page.getByTestId("play").count()) break; // the run ended
  }
  await page.close();
}
await browser.close();
console.log(`beat frames: ${times.length} battles at 1×, ${times.map((t) => `${t.toFixed(1)} s`).join(", ")}; frames in ${out}`);
