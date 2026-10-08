// No duplicate change chip (round 4, R4-6): plays a few fights at 360×640
// and 1440×900 and samples the board while it plays. A number floating
// above a card is never drawn again on that card's change chip; paused, the
// chips show again and a tap on one opens its trace.
// Screenshots go to --out. Needs a running MVP server (never the live one):
//   node e2e/probe-dup-chip.mjs --url http://127.0.0.1:8913/arena/ [--out e2e/.shots/dup-chip] [--rounds 2]
import { mkdirSync } from "node:fs";
import { launchChromium } from "./browser.mjs";

const arg = (k, d) => { const i = process.argv.indexOf(k); return i > 0 ? process.argv[i + 1] : d; };
const url = arg("--url");
const out = arg("--out", "e2e/.shots/dup-chip");
const rounds = Number(arg("--rounds", "2"));
const TAG = Date.now().toString(36).slice(-4);
if (!url) throw new Error("--url is required");
mkdirSync(out, { recursive: true });
const browser = await launchChromium();
const errors = [];

/** In the page: each card with a float on screen, the float's text and the chip text drawn on the card. */
const sample = () => {
  const shown = (el) => { for (let x = el; x; x = x.parentElement) { const s = getComputedStyle(x); if (s.display === "none" || s.visibility === "hidden") return false; } return true; };
  const rows = [];
  for (const slot of document.querySelectorAll(".bv-slot")) {
    const f = slot.querySelector(".bv-float");
    if (!f || Number(getComputedStyle(f).opacity) < 0.05) continue;
    const pill = slot.querySelector(".bv-card .bv-changes .bv-pill");
    const lines = pill ? (pill.classList.contains("two") ? [...pill.querySelectorAll(".bv-l")] : [pill]) : [];
    rows.push({ unit: slot.querySelector(".bv-card")?.dataset.unit, float: f.textContent.trim(), chip: lines.filter(shown).map((l) => l.textContent.replace(/ …$/, "").trim()) });
  }
  return rows;
};

for (const [label, viewport, mobile] of [["phone", { width: 360, height: 640 }, true], ["desktop", { width: 1440, height: 900 }, false]]) {
  const page = await browser.newPage({ viewport, deviceScaleFactor: 2, isMobile: mobile, hasTouch: mobile });
  page.on("pageerror", (e) => errors.push(`${label}: page error ${e.message}`));
  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").fill(`Dup${label === "phone" ? "P" : "D"}${TAG}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").click();
  let floats = 0, dups = 0, pausedChips = 0, traced = 0;
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
    await page.getByTestId("battle-step").waitFor({ timeout: 10_000 });
    // Playing: sample for a while.
    let shot = false;
    for (let s = 0; s < 80; s++) {
      if (await page.getByTestId("end-card").isVisible().catch(() => false)) break;
      for (const r of await page.evaluate(sample)) {
        floats++;
        if (r.chip.includes(r.float)) { dups++; errors.push(`${label} r${round}: ${r.unit} shows ${r.float} floating and on its chip`); }
        if (!shot) { await page.screenshot({ path: `${out}/${label}-r${round}-playing.png` }); shot = true; }
      }
      await page.waitForTimeout(100);
    }
    // Paused (or stepped back into the fight): chips show, and tapping one opens its trace.
    if (await page.getByTestId("end-card").isVisible().catch(() => false)) await page.getByTestId("battle-back").click();
    else await page.getByTestId("battle-play").click();
    for (let k = 0; k < 40 && (await page.getByTestId("change").count()) === 0 && !(await page.getByTestId("battle-step").isDisabled()); k++) await page.getByTestId("battle-step").click();
    const chips = page.getByTestId("change");
    if (await chips.count()) {
      const vis = await chips.first().evaluate((b) => getComputedStyle(b.querySelector(".bv-pill")).visibility);
      if (vis !== "visible") errors.push(`${label} r${round}: paused, the chip is ${vis}`);
      else pausedChips++;
      await page.screenshot({ path: `${out}/${label}-r${round}-paused.png` });
      await chips.first().click();
      if (await page.getByTestId("trace").isVisible().catch(() => false)) traced++;
      else errors.push(`${label} r${round}: paused, tapping a chip opened no trace`);
    }
    if (!(await page.getByTestId("battle-end").isDisabled())) await page.getByTestId("battle-end").click();
    await page.getByTestId("battle-done").waitFor({ timeout: 30_000 });
    await page.getByTestId("battle-done").click();
    await page.waitForFunction(() => document.querySelector('[data-testid="fight"]') || document.querySelector('[data-testid="run-over"]'));
    if (await page.getByTestId("run-over").isVisible().catch(() => false)) break;
  }
  console.log(`${label}: ${floats} floats sampled while playing, ${dups} repeated on a chip; ${pausedChips} paused chips shown, ${traced} traced`);
  if (!floats) errors.push(`${label}: no float was seen while playing`);
  if (!traced) errors.push(`${label}: no paused chip was tapped`);
  await page.close();
}
await browser.close();
if (errors.length) {
  console.log(`${errors.length} problems:\n${[...new Set(errors)].slice(0, 40).join("\n")}`);
  process.exitCode = 1;
} else console.log("no float was repeated on a chip; paused chips show and trace");
