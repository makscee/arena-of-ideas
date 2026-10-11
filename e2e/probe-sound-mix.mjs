// void-board#922: the sound engine in a real browser. Counts every sfx voice
// the page starts (AudioBufferSourceNode.start, and when each one stops) over
// home → shop → a battle at 4×, and checks: home plays a run theme, sounds
// start at all, and no more than MAX_VOICES (+ a cut one's 40 ms fade) ring
// at once.
//   node e2e/probe-sound-mix.mjs --url http://127.0.0.1:<port>/arena/
import { chromium } from "playwright";

const url = process.argv[process.argv.indexOf("--url") + 1];
const errors = [];
const browser = await chromium.launch({ args: ["--autoplay-policy=no-user-gesture-required"] });
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && errors.push(`console: ${m.text()}`));
  await page.addInitScript(() => {
    const voices = (window.__voices = []);
    const start = AudioBufferSourceNode.prototype.start;
    const stop = AudioBufferSourceNode.prototype.stop;
    AudioBufferSourceNode.prototype.start = function (...a) {
      const t = this.context.currentTime;
      this.__v = { t, end: t + (this.buffer?.duration ?? 0) / this.playbackRate.value, rate: this.playbackRate.value, dur: this.buffer?.duration };
      voices.push(this.__v);
      return start.apply(this, a);
    };
    AudioBufferSourceNode.prototype.stop = function (when = 0, ...a) {
      if (this.__v) this.__v.end = Math.min(this.__v.end, Math.max(when, this.context.currentTime));
      return stop.call(this, when, ...a);
    };
  });
  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").waitFor({ timeout: 10_000 });
  await page.keyboard.type(`SoundMix${Date.now() % 100000}`);
  await page.keyboard.press("Enter");
  await page.getByTestId("play").waitFor();
  await page.waitForTimeout(1500);
  const home = await page.evaluate(() => window.__music?.());
  console.log("home music:", JSON.stringify(home));
  if (!/^theme\d\.m4a$/.test(home?.track ?? "")) errors.push(`home plays ${home?.track}, not a run theme`);
  await page.getByTestId("play").click();
  await page.getByTestId("fight").waitFor({ timeout: 10_000 });
  await page.waitForTimeout(800);
  for (const k of ["1", "2", "r", "1"]) (await page.keyboard.press(k), await page.waitForTimeout(500));
  const shop = await page.evaluate(() => window.__voices.length);
  console.log("shop:", shop, "voices; asked:", (await page.evaluate(() => window.__sfx ?? [])).join(" "));
  if (shop === 0) errors.push("no sfx voice started in the shop");
  await page.getByTestId("fight").click();
  await page.getByTestId("battle-speed").waitFor({ timeout: 10_000 }).catch(() => {});
  // 4×: the speed button cycles 1 → 2 → 4.
  for (let i = 0; i < 3 && (await page.getByTestId("battle-speed").textContent().catch(() => "4×")) !== "4×"; i++) await page.getByTestId("battle-speed").click();
  await page.waitForTimeout(12_000);
  const v = await page.evaluate(() => window.__voices);
  const asked = await page.evaluate(() => window.__sfx ?? []);
  let most = 0, over = 0;
  for (const a of v) {
    const n = v.filter((b) => b.t <= a.t + 1e-6 && b.end > a.t + 1e-6).length;
    most = Math.max(most, n);
    if (n > 3) over++;
  }
  console.log(`battle at ${await page.getByTestId("battle-speed").textContent().catch(() => "?")}: ${v.length} voices started for ${asked.length} cues logged (last 50), at most ${most} at once, ${over} starts over 3`);
  if (v.length <= shop) errors.push("no sfx voice started in the battle");
  if (most > 4) errors.push(`${most} voices at once`);
} finally {
  await browser.close();
}
if (errors.length) (console.log("FAIL\n" + errors.join("\n")), process.exit(1));
console.log("PASS");
