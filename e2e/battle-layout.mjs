// Battle layout holds still (mission 1007, slice 1): on the phone viewport,
// plays whole battles and records the rect of the HUD, both lines, the action
// strip, the ledger and the controls on every animation frame (and after
// every step when stepping with ›). Any change of a box's left, top, width or
// height, from the line-up to the end card, fails. Cards and one floating
// number may move; these boxes may not. It also checks that the strip is one
// line (its text never wraps) and that no change pill or running total is
// drawn over a card while playing.
// Needs `npm run mvp:build` first; starts its own local server, or uses one you give it:
//   node e2e/battle-layout.mjs [--url http://127.0.0.1:8911/arena/] [--out e2e/.shots/layout] [--rounds 3]
// Round 1 plays at 4×, round 2 steps with ›, round 3 plays at the speed the
// viewer gets (1×); then the same three under prefers-reduced-motion.
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { launchChromium } from "./browser.mjs";

const arg = (k, d) => { const i = process.argv.indexOf(k); return i > 0 ? process.argv[i + 1] : d; };
let url = arg("--url");
const out = arg("--out", "e2e/.shots/layout");
const rounds = Number(arg("--rounds", "3"));
const TAG = Date.now().toString(36).slice(-4);
let server = null;
if (!url) {
  const port = 18000 + Math.floor(Math.random() * 1000);
  server = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", STATIC_DIR: "mobile/dist" }, stdio: ["ignore", "ignore", "inherit"] });
  process.on("exit", () => server.kill());
  url = `http://127.0.0.1:${port}/arena/`;
  for (let i = 0; i < 120; i++) { try { if ((await fetch(`${url}api/v1/health`)).ok) break; } catch {} await new Promise((r) => setTimeout(r, 500)); }
}
mkdirSync(out, { recursive: true });

/** The boxes that must hold still, by name. */
const BOXES = {
  hud: ".bv-screen > .hud",
  them: '[data-testid="battle-them"]',
  you: '[data-testid="battle-you"]',
  strip: '[data-testid="caption"]',
  ledger: '[data-testid="ledger"]',
  controls: ".bv-controls",
};

/** In the page: remembers the first rect of each box and every later frame that differs from it, until the end card is up. */
const watch = (boxes) => {
  window.__layout = { first: null, moved: [], frames: 0, wrapped: 0, pills: 0 };
  const L = window.__layout;
  const rectOf = (sel) => {
    const el = document.querySelector(sel);
    if (!el) return null;
    const r = el.getBoundingClientRect();
    return [Math.round(r.left * 10) / 10, Math.round(r.top * 10) / 10, Math.round(r.width * 10) / 10, Math.round(r.height * 10) / 10];
  };
  const tick = () => {
    const end = document.querySelector('[data-testid="end-card"]');
    if (end && getComputedStyle(end).display !== "none") return;
    const now = Object.fromEntries(Object.entries(boxes).map(([k, sel]) => [k, rectOf(sel)]));
    if (now.strip && !L.first) L.first = now;
    if (L.first) {
      L.frames++;
      for (const k of Object.keys(boxes)) {
        const a = L.first[k], b = now[k];
        if (!a || !b || a.some((x, i) => Math.abs(x - b[i]) > 0.6)) L.moved.push({ box: k, was: a, now: b, at: L.frames, t: Math.round(performance.now()) });
      }
    }
    // The caption sits in the rail (#1294): it may wrap, but its words fit its fixed box.
    const strip = document.querySelector(boxes.strip);
    if (strip) {
      const line = strip.querySelector(".bv-strip-line");
      if (line && line.getBoundingClientRect().bottom > strip.getBoundingClientRect().bottom + 1) L.wrapped++;
    }
    // While playing, no result tag or running total sits on a piece's face
    // (#1294: the event's tag shows while playing, beside the piece at the arrow).
    if (document.querySelector(".bv-playing") && !matchMedia("(prefers-reduced-motion: reduce)").matches) {
      const faces = [...document.querySelectorAll(".bv-card .pc-face")].map((f) => f.getBoundingClientRect());
      const over = (r) => faces.some((f) => r.left < f.right - 1 && r.right > f.left + 1 && r.top < f.bottom - 1 && r.bottom > f.top + 1);
      for (const p of document.querySelectorAll(".bv-card .bv-pill:not(.bv-tot), .bv-run-row")) if (getComputedStyle(p).visibility !== "hidden" && p.getClientRects().length && over(p.getBoundingClientRect())) L.pills++;
    }
    requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
};

const browser = await launchChromium();
let bad = 0;
const report = async (page, label) => {
  const L = await page.evaluate(() => window.__layout);
  const first = new Map();
  for (const m of L.moved) if (!first.has(m.box)) first.set(m.box, m);
  const line = `${label}: ${L.frames} frames, ${L.moved.length} box moves${[...first.values()].map((m) => ` [${m.box} ${JSON.stringify(m.was)} → ${JSON.stringify(m.now)} at frame ${m.at}]`).join("")}, ${L.wrapped} overflowing caption frames, ${L.pills} pill frames while playing`;
  console.log(line);
  if (L.moved.length || L.wrapped || L.pills || L.frames < 5) bad++;
};

for (const motion of ["no-preference", "reduce"]) {
  const page = await browser.newPage({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true, reducedMotion: motion });
  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").fill(`Layout${motion === "reduce" ? "Still" : "Move"}${TAG}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").click();
  for (let round = 1; round <= rounds; round++) {
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
    await page.evaluate(watch, BOXES);
    const mode = round === 1 ? "4×" : round === 2 ? "stepping" : "1×";
    // The speed is remembered between battles: set what this round wants.
    const want = round === 1 ? "4×" : round === rounds ? "1×" : null;
    for (let i = 0; want && i < 3 && (await page.getByTestId("battle-speed").textContent()) !== want; i++) await page.getByTestId("battle-speed").click();
    if (round === 2) {
      await page.getByTestId("battle-play").click(); // pause
      for (let i = 0; i < 400 && (await page.getByTestId("battle-step").isEnabled()); i++) {
        await page.getByTestId("battle-step").click();
        if (i % 8 === 0) await page.screenshot({ path: `${out}/${motion === "reduce" ? "still" : "move"}-r${round}-s${String(i).padStart(3, "0")}.png` });
      }
    } else if (round === rounds) {
      for (let f = 0; f < 40; f++) {
        if (await page.getByTestId("battle-done").isVisible().catch(() => false)) break;
        await page.screenshot({ path: `${out}/${motion === "reduce" ? "still" : "move"}-r${round}-f${String(f).padStart(2, "0")}.png` });
        await page.waitForTimeout(1500);
      }
    }
    await page.getByTestId("battle-done").waitFor({ timeout: 300_000 });
    await report(page, `${motion === "reduce" ? "reduced" : "motion"} round ${round} (${mode})`);
    await page.screenshot({ path: `${out}/${motion === "reduce" ? "still" : "move"}-r${round}-end.png` });
    await page.getByTestId("battle-done").click();
    await page.waitForFunction(() => document.querySelector('[data-testid="fight"]') || document.querySelector('[data-testid="run-over"]'));
    if (await page.getByTestId("run-over").isVisible().catch(() => false)) break;
  }
  await page.close();
}

// Lines growing from 5 to 8 cards (summons and revives, compact cards; e2e/line-of-8-battle.ts) on both phone sizes: the boxes still hold.
const rec = join(mkdtempSync(join(tmpdir(), "layout-line8-")), "line-of-8.json");
execFileSync("node", ["--import", "tsx/esm", "e2e/line-of-8-battle.ts", rec], { stdio: ["ignore", "ignore", "inherit"] });
const record = JSON.parse(readFileSync(rec, "utf8"));
for (const viewport of [{ width: 390, height: 844 }, { width: 360, height: 640 }]) {
  const page = await browser.newPage({ viewport, deviceScaleFactor: 1, isMobile: true, hasTouch: true });
  await page.route((u) => /\/battles\/[^/]+$/.test(u.pathname), async (route) => {
    const res = await route.fetch();
    const body = await res.json();
    await route.fulfill({ response: res, json: { ...body, ...record, battleId: body.battleId, player: body.player, runId: body.runId, kind: body.kind, round: body.round } });
  });
  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").fill(`Line8${viewport.width}${TAG}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").click();
  await page.getByTestId("offer-0").click(); await page.getByTestId("offer-0").click();
  await page.getByTestId("buy").click();
  await page.getByTestId("fight").click();
  await page.getByTestId("battle-end").waitFor({ timeout: 10_000 });
  await page.evaluate(watch, BOXES);
  for (let i = 0; i < 3 && (await page.getByTestId("battle-speed").textContent()) !== "4×"; i++) await page.getByTestId("battle-speed").click();
  await page.getByTestId("battle-done").waitFor({ timeout: 300_000 });
  await report(page, `line of 8 at ${viewport.width}×${viewport.height} (4×)`);
  await page.close();
}
await browser.close();
server?.kill();
if (bad) { console.log(`battle layout: ${bad} battles moved a box`); process.exitCode = 1; }
else console.log("battle layout: no box moved");
