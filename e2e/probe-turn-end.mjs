// The turn-end summary (round 4, R4-12): plays fights at 360×640 and
// 1440×900 at 1× and watches each turn's end. After a turn's last beat the
// board holds about 1.2 s: every unit it changed keeps its running totals
// above it (R4-22, none in the card's centre), no float, chip or beam is
// drawn, the dead stand greyed with theirs; then the rows clear. Paused, the totals stay; → steps onto them; at 4× there are none.
// Screenshots go to --out. Needs a running MVP server (never the live one):
//   node e2e/probe-turn-end.mjs --url http://127.0.0.1:8913/arena/ [--out e2e/.shots/turn-end]
import { mkdirSync } from "node:fs";
import { launchChromium } from "./browser.mjs";

const arg = (k, d) => { const i = process.argv.indexOf(k); return i > 0 ? process.argv[i + 1] : d; };
const url = arg("--url");
const out = arg("--out", "e2e/.shots/turn-end");
const TAG = Date.now().toString(36).slice(-4);
if (!url) throw new Error("--url is required");
mkdirSync(out, { recursive: true });
const browser = await launchChromium();
const errors = [];

/** In the page: what the board draws now. */
const board = () => {
  const vis = (el) => { for (let x = el; x; x = x.parentElement) { const s = getComputedStyle(x); if (s.display === "none" || s.visibility === "hidden" || Number(s.opacity) < 0.05) return false; } return true; };
  return {
    turnEnd: document.querySelector(".bv-controls")?.dataset.turnEnd ?? "",
    // R4-22: the turn's totals are the running rows above each unit; none in a card's centre.
    centre: document.querySelectorAll('[data-testid="turn-total"]').length,
    totals: [...document.querySelectorAll('[data-testid="running-totals"] .bv-run:not(.out)')].filter((b) => b.style.visibility !== "hidden").map((b) => ({ unit: b.dataset.unit, text: b.textContent, dead: !!document.querySelector(`.bv-card.dead[data-unit="${CSS.escape(b.dataset.unit)}"]`) })),
    floats: [...document.querySelectorAll(".bv-float")].filter(vis).length,
    chips: document.querySelectorAll('[data-testid="change"]').length,
    badges: document.querySelectorAll('[data-testid="trigger-badge"]').length,
    beams: document.querySelectorAll('[data-testid="beams"] > *').length,
    caption: document.querySelector('[data-testid="caption"]')?.textContent ?? "",
  };
};

async function toBattle(page, name) {
  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").fill(name);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").click();
  await page.getByTestId("fight").waitFor({ timeout: 10_000 });
  for (let k = 0; k < 3; k++) {
    const gold = Number((await page.getByTestId("gold").textContent()).replace("g", ""));
    if (gold < 3) break;
    await page.getByTestId("offer-0").click();
    if (await page.getByTestId("buy").isDisabled()) { await page.getByTestId("offer-close").click(); break; }
    await page.getByTestId("buy").click();
    await page.waitForFunction((g) => document.querySelector('[data-testid="gold"]')?.textContent !== `${g}g`, gold);
  }
  await page.getByTestId("fight").click();
  await page.getByTestId("battle-step").waitFor({ timeout: 10_000 });
}

for (const [label, viewport, mobile] of [["phone", { width: 360, height: 640 }, true], ["desktop", { width: 1440, height: 900 }, false]]) {
  const page = await browser.newPage({ viewport, deviceScaleFactor: 2, isMobile: mobile, hasTouch: mobile });
  page.on("pageerror", (e) => errors.push(`${label}: page error ${e.message}`));
  await toBattle(page, `Te${label[0]}${TAG}`);
  if ((await page.getByTestId("battle-speed").textContent()) !== "1×") errors.push(`${label}: round 1 plays at ${await page.getByTestId("battle-speed").textContent()}`);
  // Playing at 1×: sample every 50 ms, timing each summary from when it shows to when it clears.
  let seen = 0, shot = 0, shotOf = "", cur = "", since = 0, holds = [], sawDead = false;
  const t0 = Date.now();
  while (Date.now() - t0 < 60_000) {
    if (await page.getByTestId("end-card").isVisible().catch(() => false)) break;
    const b = await page.evaluate(board);
    const now = Date.now();
    if (b.turnEnd !== cur) {
      if (cur) holds.push(now - since);
      cur = b.turnEnd;
      since = now;
      if (cur) seen++;
    }
    if (b.turnEnd) {
      if (!b.totals.length) errors.push(`${label} T${b.turnEnd}: the summary shows no totals`);
      if (b.centre || b.floats || b.chips || b.badges || b.beams) errors.push(`${label} T${b.turnEnd}: the summary still draws ${JSON.stringify({ centre: b.centre, floats: b.floats, chips: b.chips, badges: b.badges, beams: b.beams })}`);
      if (b.totals.some((t) => t.dead)) sawDead = true;
      if (shot < 4 && shotOf !== b.turnEnd && now - since > 300) {
        shotOf = b.turnEnd; await page.screenshot({ path: `${out}/${label}-turn-end-${++shot}.png` }); console.log(`${label} T${b.turnEnd}: ${b.caption} | ${b.totals.map((t) => `${t.unit}${t.dead ? "(dead)" : ""} ${t.text}`).join("; ")}`); }
    }
    await page.waitForTimeout(50);
  }
  if (!seen) errors.push(`${label}: no turn-end summary in a whole battle`);
  const full = holds.slice(0, -1);
  console.log(`${label}: ${seen} summaries, holds ${full.join(", ")} ms, a fallen unit's totals seen: ${sawDead}`);
  for (const ms of full) if (ms < 1000 || ms > 1700) errors.push(`${label}: a summary held ${ms} ms (want ~1200)`);

  // Paused: the totals stay; stepping with → lands on them after a turn's last beat.
  await page.getByTestId("battle-replay").click();
  await page.getByTestId("battle-play").click();
  let stepped = 0;
  for (let k = 0; k < 60 && !(await page.getByTestId("battle-step").isDisabled()); k++) {
    await page.getByTestId("battle-step").click();
    const b = await page.evaluate(board);
    if (b.turnEnd) {
      stepped++;
      if (!b.totals.length) errors.push(`${label}: stepped onto T${b.turnEnd}'s summary, no totals`);
      if (stepped === 1) {
        await page.waitForTimeout(1600);
        if ((await page.evaluate(board)).turnEnd !== b.turnEnd) errors.push(`${label}: paused, the summary didn't stay`);
        await page.screenshot({ path: `${out}/${label}-stepped.png` });
        // ← from the summary shows the turn's last beat again, → the summary again.
        await page.getByTestId("battle-back").click();
        if ((await page.evaluate(board)).turnEnd) errors.push(`${label}: ← from a summary stayed on it`);
        await page.getByTestId("battle-step").click();
        if (!(await page.evaluate(board)).turnEnd) errors.push(`${label}: → after ← didn't come back to the summary`);
      }
    }
    if (await page.getByTestId("end-card").isVisible().catch(() => false)) break;
  }
  if (!stepped) errors.push(`${label}: stepping never landed on a summary`);
  console.log(`${label}: stepped onto ${stepped} summaries`);
  await page.close();
}

// At 4×: none. Speed is remembered per device, so a fresh page picks it on the battle.
{
  const page = await browser.newPage({ viewport: { width: 360, height: 640 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true });
  await toBattle(page, `Tf${TAG}`);
  while ((await page.getByTestId("battle-speed").textContent()) !== "4×") await page.getByTestId("battle-speed").click();
  let any = 0;
  const t0 = Date.now();
  while (Date.now() - t0 < 30_000 && !(await page.getByTestId("end-card").isVisible().catch(() => false))) {
    if ((await page.evaluate(board)).turnEnd) any++;
    await page.waitForTimeout(40);
  }
  if (any) errors.push(`4×: a summary showed (${any} samples)`);
  console.log(`4×: ${any} summary samples`);
  await page.close();
}

await browser.close();
if (errors.length) {
  console.error(errors.join("\n"));
  process.exit(1);
}
console.log("turn-end summary: ok");
