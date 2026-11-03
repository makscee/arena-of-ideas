// The battle's effect language (mission 1007, #1011): plays battles on a phone viewport at 1× and keeps
// one frame of each kind of effect mid-animation (damage, blocked, heal, buff, debuff, status, death,
// summon), then joins them into one strip. Fails if a kind never showed in --battles battles.
// Needs `npm run mvp:build` and a running MVP server (MVP_DEV=1; fresh :memory: db):
//   node e2e/effect-frames.mjs --url http://127.0.0.1:8911/arena/ [--out e2e/.shots/effects] [--battles 14] [--theme plum|brass|felt] [--reduce]
// With --reduce (prefers-reduced-motion) it checks each kind's chip or still digit instead (icon, number and colour, no motion).
import { execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { launchChromium } from "./browser.mjs";

const arg = (k, d) => { const i = process.argv.indexOf(k); return i > 0 ? process.argv[i + 1] : d; };
const url = arg("--url");
const out = arg("--out", "e2e/.shots/effects");
const battles = Number(arg("--battles", "14"));
const theme = arg("--theme", "plum");
const reduce = process.argv.includes("--reduce");
if (!url) throw new Error("--url is required");
mkdirSync(out, { recursive: true });
const TAG = Date.now().toString(36).slice(-4);

/** Each kind's selector for "showing right now" (moving: the animation is mid-way; still: its mark). */
const SEEN = reduce
  ? { damage: ".bv-change.damage", blocked: ".bv-pill.blocked", heal: ".bv-change.heal", buff: ".stats > .bv-up", debuff: ".stats > .bv-down", status: ".bv-change.status", death: ".bv-card.dead", summon: ".bv-change.summon" }
  : { damage: ".bv-fxn.damage", blocked: ".bv-fxb", heal: ".bv-fxn.heal", buff: ".stats > .bv-pop-up", debuff: ".stats > .bv-pop-down", status: ".bv-fxfly", death: ".bv-skull", summon: ".bv-slot.bv-enter" };
const kinds = Object.keys(SEEN);
const got = new Map();

const browser = await launchChromium();
const page = await browser.newPage({ viewport: { width: 360, height: 640 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true, reducedMotion: reduce ? "reduce" : "no-preference" });
/** A fresh player on a fresh run (a run ends after a few lost rounds). */
let players = 0;
const newPlayer = async () => {
  await page.goto(`${url}?theme=${theme}`, { timeout: 20_000 });
  await page.evaluate(() => { try { localStorage.clear(); } catch {} });
  await page.goto(`${url}?theme=${theme}`, { timeout: 20_000 });
  await page.getByTestId("name-input").fill(`Fx${TAG}${players++}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").click();
};
await newPlayer();
let fought = 0;
for (let round = 1; fought < battles && got.size < kinds.length; round++) {
  await page.getByTestId("fight").waitFor({ timeout: 10_000 });
  for (let k = 0; k < 3; k++) {
    if ((await page.getByTestId("gold").count()) === 0) break;
    const gold = Number((await page.getByTestId("gold").textContent()).replace("g", ""));
    if (gold < 3 || (await page.getByTestId("line").locator(".card.you").count()) >= 5) break;
    await page.getByTestId("offer-0").click(); await page.getByTestId("offer-0").click();
    if (await page.getByTestId("buy").isDisabled()) { await page.getByTestId("offer-close").click(); break; }
    await page.getByTestId("buy").click();
    await page.waitForFunction((g) => document.querySelector('[data-testid="gold"]')?.textContent !== `${g}g`, gold);
  }
  await page.getByTestId("fight").click();
  await page.getByTestId("battle-end").waitFor({ timeout: 10_000 });
  fought++;
  const t0 = Date.now();
  while (!(await page.getByTestId("battle-done").count()) && Date.now() - t0 < 120_000) {
    const now = await page.evaluate((sel) => Object.entries(sel).filter(([, q]) => document.querySelector(q)).map(([k]) => k), SEEN);
    const fresh = now.filter((k) => !got.has(k));
    if (fresh.length) {
      await page.waitForTimeout(reduce ? 0 : 140);
      const file = `${out}/${String(got.size).padStart(2, "0")}-${fresh[0]}.png`;
      await page.screenshot({ path: file });
      for (const k of fresh) if (!got.has(k)) got.set(k, file);
    } else await page.waitForTimeout(40);
  }
  await page.getByTestId("battle-done").click();
  await page.waitForFunction(() => document.querySelector('[data-testid="fight"]') || document.querySelector('[data-testid="run-over"]'));
  if (await page.getByTestId("run-over").isVisible().catch(() => false)) await newPlayer();
}
await browser.close();
console.log(`effects seen in ${fought} battles: ${kinds.map((k) => `${k} ${got.has(k) ? "yes" : "NO"}`).join(", ")}`);
const files = [...new Set(got.values())];
if (files.length) {
  const strip = `${out}/strip-${theme}${reduce ? "-reduced" : ""}.png`;
  try { execFileSync("ffmpeg", ["-y", "-loglevel", "error", ...files.flatMap((f) => ["-i", f]), "-filter_complex", `${files.map((_, i) => `[${i}:v]`).join("")}hstack=inputs=${files.length}`, strip]); console.log(`strip: ${strip}`); } catch { console.log("no ffmpeg: frames only"); }
}
if (got.size < kinds.length) process.exitCode = 1;
