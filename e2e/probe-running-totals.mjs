// Running totals above units, and the dead clearing at turn end (round 4,
// R4-22). Plays fights at 360×640 and 1440×900 at 1×, sampling the board
// every ~40 ms:
// - no totals label in a card's centre, no float;
// - above each unit one row per kind of change this turn; a row never moves
//   once shown (its index among the unit's rows stays), and its number
//   counts (an in-between value shows when a sum grows by more than 1);
// - a unit that dies stays greyed in its slot until the turn ends, never
//   vanishing and coming back; the next turn has it gone;
// - paused, ← and → show the running sums of the beat on screen;
// - reduced motion: rows still show, numbers change without counting.
// Screenshots of a multi-hit row and of a mid-turn death go to --out.
// Needs a running MVP server (never the live one):
//   node e2e/probe-running-totals.mjs --url http://127.0.0.1:8927/arena/ [--out e2e/.shots/running-totals]
import { mkdirSync } from "node:fs";
import { launchChromium } from "./browser.mjs";

const arg = (k, d) => { const i = process.argv.indexOf(k); return i > 0 ? process.argv[i + 1] : d; };
const url = arg("--url");
const out = arg("--out", "e2e/.shots/running-totals");
const TAG = Date.now().toString(36).slice(-4);
if (!url) throw new Error("--url is required");
mkdirSync(out, { recursive: true });
const browser = await launchChromium();
const errors = [];

/** In the page: what the board draws now. */
const board = () => {
  const vis = (el) => { for (let x = el; x; x = x.parentElement) { const s = getComputedStyle(x); if (s.display === "none" || s.visibility === "hidden" || Number(s.opacity) < 0.05) return false; } return true; };
  return {
    turn: `T${document.querySelector('[data-testid="running-totals"]')?.dataset.turn ?? ""}`,
    turnEnd: document.querySelector(".bv-controls")?.dataset.turnEnd ?? "",
    centre: document.querySelectorAll('[data-testid="turn-total"]').length,
    floats: [...document.querySelectorAll(".bv-float")].filter(vis).length,
    runs: [...document.querySelectorAll('[data-testid="running-totals"] .bv-run:not(.out)')].map((b) => ({
      unit: b.dataset.unit,
      rows: [...b.querySelectorAll('[data-testid="run-row"]')].map((r) => ({ key: r.dataset.key, value: Number(r.dataset.value), text: r.querySelector(".n")?.textContent ?? "", counting: r.dataset.counting === "1" })),
    })),
    dead: [...document.querySelectorAll('[data-testid="battle-them"] .bv-card.dead, [data-testid="battle-you"] .bv-card.dead')].map((c) => c.dataset.unit),
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

/** Plays the battle on screen at 1× to its end card, checking every sample. */
async function watch(page, label, still = false) {
  /** Per unit, its row keys in order this turn; the turn they belong to. */
  let order = new Map(), turnOf = "", lastTurn = "";
  /** The dead seen this turn, and the dead of turns gone. */
  let deadNow = new Set(), gone = new Set();
  const last = new Map();
  let between = 0, multi = false, midDeath = false, samples = 0;
  const t0 = Date.now();
  while (Date.now() - t0 < 90_000) {
    if (await page.getByTestId("end-card").isVisible().catch(() => false)) break;
    const b = await page.evaluate(board);
    samples++;
    if (b.centre) errors.push(`${label} ${b.turn}: ${b.centre} totals in a card's centre`);
    if (b.floats) errors.push(`${label} ${b.turn}: ${b.floats} floats`);
    // A new turn: rows start over; the last turn's dead are gone.
    if (b.turn !== turnOf) {
      if (turnOf) for (const u of deadNow) gone.add(u);
      order = new Map(); deadNow = new Set(); turnOf = b.turn; last.clear();
    }
    for (const r of b.runs) {
      const keys = r.rows.map((x) => x.key);
      const was = order.get(r.unit) ?? [];
      if (was.some((k, i) => keys.includes(k) && keys.indexOf(k) !== i)) errors.push(`${label} ${b.turn}: ${r.unit}'s rows reordered ${was.join(",")} → ${keys.join(",")}`);
      if (keys.length >= was.length) order.set(r.unit, keys);
      for (const x of r.rows) {
        const id = `${r.unit}/${x.key}`;
        const shown = Number(x.text.replace(/[^\d]/g, "") || 0);
        const prev = last.get(id);
        if (prev && prev.value !== x.value) multi = true;
        if (Math.abs(x.value) !== shown || x.counting) {
          between++;
          if (still) errors.push(`${label} ${b.turn}: reduced motion counts ${id} (${x.text}, sum ${x.value})`);
        }
        if (!still && prev && prev.value !== x.value && !page.__multiShot) {
          page.__multiShot = true;
          await page.screenshot({ path: `${out}/${label}-multi-hit-row.png` });
          console.log(`${label} ${b.turn}: ${id} ${prev.value} → ${x.value} (shows ${x.text})`);
        }
        last.set(id, { value: x.value });
      }
    }
    for (const u of b.dead) {
      if (gone.has(u)) errors.push(`${label} ${b.turn}: ${u} died in an earlier turn and is back`);
      deadNow.add(u);
    }
    for (const u of deadNow) if (!b.dead.includes(u)) errors.push(`${label} ${b.turn}: ${u} fell this turn and vanished before its end`);
    // A death shown while a later beat plays (the caption is about something else): it waits for the turn end.
    if (!still && b.dead.length && !/falls|fall\b|dies/.test(b.caption) && !b.turnEnd && !midDeath) {
      midDeath = true;
      await page.screenshot({ path: `${out}/${label}-mid-turn-death.png` });
      console.log(`${label} ${b.turn}: dead standing mid-turn: ${b.dead.join(", ")} | ${b.caption}`);
    }
    lastTurn = b.turn;
    await page.waitForTimeout(40);
  }
  console.log(`${label}: ${samples} samples to ${lastTurn}, a growing row: ${multi}, counting samples: ${between}, a mid-turn death: ${midDeath}`);
  return { multi, between, midDeath };
}

const seen = { multi: false, between: 0, midDeath: false };
for (const [label, viewport, mobile] of [["phone", { width: 360, height: 640 }, true], ["desktop", { width: 1440, height: 900 }, false]]) {
  const page = await browser.newPage({ viewport, deviceScaleFactor: 2, isMobile: mobile, hasTouch: mobile });
  page.on("pageerror", (e) => errors.push(`${label}: page error ${e.message}`));
  // Short round-1 fights may have no multi-hit: play new runs (a fresh name each) until one shows.
  const mine = { multi: false, between: 0, midDeath: false };
  for (let k = 0; k < 4 && !(mine.multi && mine.between && mine.midDeath); k++) {
    if (k) { await page.evaluate(() => localStorage.clear()); }
    await toBattle(page, `Rt${label[0]}${k}${TAG}`);
    const w = await watch(page, label);
    mine.multi ||= w.multi; mine.between += w.between; mine.midDeath ||= w.midDeath;
  }
  seen.multi ||= mine.multi; seen.between += mine.between; seen.midDeath ||= mine.midDeath;

  // Paused: stepping shows the running sums of the beat on screen, and ← undoes a step's sums.
  await page.getByTestId("battle-replay").click();
  await page.getByTestId("battle-play").click();
  let checked = 0;
  for (let k = 0; k < 80 && !(await page.getByTestId("battle-step").isDisabled()); k++) {
    const before = JSON.stringify((await page.evaluate(board)).runs);
    await page.getByTestId("battle-step").click();
    await page.waitForTimeout(30);
    const after = await page.evaluate(board);
    for (const r of after.runs) for (const x of r.rows) if (Math.abs(x.value) !== Number(x.text.replace(/[^\d]/g, "") || 0)) errors.push(`${label}: stepped, ${r.unit}/${x.key} shows ${x.text} for ${x.value}`);
    if (JSON.stringify(after.runs) !== before && after.turnEnd === "" && checked < 3) {
      await page.getByTestId("battle-back").click();
      await page.waitForTimeout(30);
      if (JSON.stringify((await page.evaluate(board)).runs) !== before) errors.push(`${label}: ← didn't bring back the earlier sums`);
      await page.getByTestId("battle-step").click();
      checked++;
    }
    if (await page.getByTestId("end-card").isVisible().catch(() => false)) break;
  }
  if (!checked) errors.push(`${label}: stepping never changed a row`);
  await page.close();
}
if (!seen.multi) errors.push("no row grew in a turn (no multi-hit seen)");
if (!seen.between) errors.push("no row ever counted (no in-between value sampled)");
if (!seen.midDeath) errors.push("no mid-turn death seen standing");

// Reduced motion: rows show, numbers jump straight to their sums.
{
  const page = await browser.newPage({ viewport: { width: 360, height: 640 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true, reducedMotion: "reduce" });
  page.on("pageerror", (e) => errors.push(`reduced: page error ${e.message}`));
  await toBattle(page, `Rr${TAG}`);
  await watch(page, "reduced", true);
  await page.close();
}

await browser.close();
if (errors.length) {
  console.error([...new Set(errors)].join("\n"));
  process.exit(1);
}
console.log("running totals: ok");
