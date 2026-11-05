// The battle's field and its result tray (arena#29 round 4, mission #26): at 1920×1080, 1440×900, 1280×720,
// 1024×768, 800×450 and 390×844 a line of 1, 2 or 3 units (real round-1 fights) and the scene's longer lines
// stand centred across the field, their line above yours, spaced like the shop's slots, their pieces between
// 0.9× and 1.4× the shop's; the labels (THEM @name, Report, YOU) never touch a piece; a chain's wire runs
// between the real pieces; the result tray shows its damage rows whole (or two of them with "All N") and
// its buttons whole, 44 px on a phone. Without --url it builds the client and starts a scenes server.
//   npm run mvp:battle-field -- [--url http://127.0.0.1:PORT/arena/] [--out e2e/.shots/battle-field] [--size 1280x720] [--units 1,2,3]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/battle-field";
const SIZES = (opt("size") ?? "1920x1080,1440x900,1280x720,1024x768,800x450,390x844").split(",");
const UNITS = (opt("units") ?? "1,2,3").split(",").map(Number);
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_TUNER: "instant", ARENA_CODEX: "1", MVP_POOL: "v2" }, stdio: ["ignore", "ignore", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) { try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {} await new Promise((r) => setTimeout(r, 200)); }

// The shop line's piece, per stage (ui/line-layout.ts SHOP_PX): a battle's piece is measured against it.
const SHOP_PX = { L: 70, P: 56 };

/** What the field looks like now, in stage pixels: each row's pieces, the labels, the stage's middle. */
const readField = (page) => page.evaluate(() => {
  const st = document.querySelector('[data-testid="stage"]');
  const sr = st.getBoundingClientRect();
  const k = sr.width / st.offsetWidth;
  const toStage = (r) => ({ l: (r.left - sr.left) / k, t: (r.top - sr.top) / k, r: (r.right - sr.left) / k, b: (r.bottom - sr.top) / k });
  const cards = (id) => [...document.querySelectorAll(`[data-testid="${id}"] .bv-card:not(.dead)`)].map((c) => ({ card: toStage(c.getBoundingClientRect()), face: toStage((c.querySelector(".pc") ?? c).getBoundingClientRect()), w: (c.querySelector(".pc") ?? c).offsetWidth }));
  const row = (id) => {
    const cs = cards(id);
    if (!cs.length) return null;
    const l = Math.min(...cs.map((c) => c.face.l)), r = Math.max(...cs.map((c) => c.face.r));
    return { n: cs.length, l, r, mid: (l + r) / 2, top: Math.min(...cs.map((c) => c.card.t)), bottom: Math.max(...cs.map((c) => c.card.b)), px: Math.min(...cs.map((c) => c.w)), cards: cs.map((c) => c.card) };
  };
  const label = (cls) => { const e = document.querySelector(`.tb-side.${cls}`); return e ? toStage(e.getBoundingClientRect()) : null; };
  const kids = (cls) => [...document.querySelectorAll(`.tb-side.${cls} *`)].filter((e) => e.getClientRects().length).map((e) => toStage(e.getBoundingClientRect()));
  return { orient: st.dataset.orient, stageW: st.offsetWidth, them: row("battle-them"), you: row("battle-you"), labels: { them: label("them"), you: label("you") }, labelKids: { them: kids("them"), you: kids("you") } };
});

const hit = (a, b, pad = 0) => a.l < b.r + pad && a.r > b.l - pad && a.t < b.b + pad && a.b > b.t - pad;

/** The problems with a field reading, as sentences. */
function fieldProblems(f, what) {
  const bad = [];
  if (!f.them || !f.you) return [`${what}: a line is empty at the line-up (${JSON.stringify({ them: !!f.them, you: !!f.you })})`];
  const mid = f.stageW / 2;
  for (const [who, r] of [["their", f.them], ["your", f.you]]) {
    if (Math.abs(r.mid - mid) > f.stageW * 0.02) bad.push(`${what}: ${who} line is centred at ${Math.round(r.mid)}, the stage's middle is ${Math.round(mid)}`);
    const shop = SHOP_PX[f.orient];
    if (r.px < shop * 0.9 - 0.5) bad.push(`${what}: ${who} pieces are ${r.px} px, under the shop's ${shop} × 0.9`);
    if (r.px > shop * 1.4 + 0.5) bad.push(`${what}: ${who} pieces are ${r.px} px, over the shop's ${shop} × 1.4`);
  }
  if (!(f.them.bottom <= f.you.top + 1)) bad.push(`${what}: their line (${Math.round(f.them.bottom)}) is not above yours (${Math.round(f.you.top)})`);
  for (const side of ["them", "you"]) {
    const row = f[side];
    const l = f.labels[side];
    if (!l) { bad.push(`${what}: no ${side} label`); continue; }
    for (const [i, c] of [...f.them.cards, ...f.you.cards].entries()) if (f.labelKids[side].some((kid) => hit(kid, c, 0))) { bad.push(`${what}: the ${side} label touches piece ${i}`); break; }
    if (f.orient === "L" && l.r > row.l - 4) bad.push(`${what}: the ${side} label (right edge ${Math.round(l.r)}) is not in the left gutter (pieces start at ${Math.round(row.l)})`);
    if (f.orient === "P") {
      if (l.l > 30) bad.push(`${what}: the ${side} label starts at ${Math.round(l.l)}, not at its row's left edge`);
      if (l.b > row.top + 1) bad.push(`${what}: the ${side} label (bottom ${Math.round(l.b)}) is not a heading above its row (top ${Math.round(row.top)})`);
    }
  }
  return bad;
}

/** The result tray: every damage row whole, or two with "All N"; the buttons whole and 44 px on a phone. */
const readResult = (page) => page.evaluate(() => {
  const st = document.querySelector('[data-testid="stage"]');
  const k = st.getBoundingClientRect().width / st.offsetWidth;
  const card = document.querySelector('[data-testid="end-card"]');
  const cr = card.getBoundingClientRect();
  const inside = (r) => r.top >= cr.top - 0.5 && r.bottom <= cr.bottom + 0.5 && r.left >= cr.left - 0.5 && r.right <= cr.right + 0.5;
  const body = card.querySelector(".bv-end-body");
  const rows = [...card.querySelectorAll('[data-testid="damage-row"]')];
  // A row is whole when its box is inside the card and inside its scrolling parent's visible box.
  const clip = (e) => { let p = e.parentElement; const r = e.getBoundingClientRect(); while (p && p !== card.parentElement) { const s = getComputedStyle(p); if (/(auto|scroll|hidden)/.test(s.overflowY)) { const pr = p.getBoundingClientRect(); if (r.bottom > pr.bottom + 0.5 || r.top < pr.top - 0.5) return true; } p = p.parentElement; } return false; };
  const buttons = [...card.querySelectorAll('[data-testid="battle-done"], [data-testid="end-replay"], [data-testid="end-why"]')].map((b) => { const r = b.getBoundingClientRect(); return { id: b.dataset.testid, whole: inside(r) && !clip(b), h: r.height, w: r.width }; });
  const rowH = rows.length ? Math.min(...rows.map((r) => r.getBoundingClientRect().height)) : 0;
  return { wide: innerWidth, clipped: card.scrollHeight - card.clientHeight, rowH, rows: rows.length, whole: rows.filter((r) => inside(r.getBoundingClientRect()) && !clip(r)).length, all: card.querySelector('[data-testid="damage-more"]')?.textContent ?? "", scrolls: body ? body.scrollHeight - body.clientHeight : 0, buttons, k, phone: innerWidth < 600 };
});

function resultProblems(r, what) {
  const bad = [];
  const more = /\d/.test(r.all) && !/top|3/i.test(r.all);
  if (r.whole < r.rows) bad.push(`${what}: ${r.rows - r.whole} of ${r.rows} damage rows are cut off or need a scroll`);
  if (r.rows < 2) bad.push(`${what}: ${r.rows} damage rows shown, two at least`);
  if (r.clipped > 3) bad.push(`${what}: the result hides ${r.clipped} px of its content`);
  // A damage row is a target: 44 px on a phone, 32 px at 1024 and up, 24 px drawn at 800×450 (the lint's floors).
  const floor = r.phone ? 43.5 : r.wide >= 1024 ? 31.5 : 23.5;
  if (r.rowH < floor) bad.push(`${what}: a damage row is ${Math.round(r.rowH)} px tall, ${Math.round(floor)} at least`);
  if (r.rows < (more ? 2 : 1)) bad.push(`${what}: the damage list shows too few rows`);
  for (const b of r.buttons) {
    if (!b.whole) bad.push(`${what}: ${b.id} is not whole on screen`);
    if (r.phone && b.h * r.k < 43.5) bad.push(`${what}: ${b.id} is ${Math.round(b.h * r.k)} px tall on a phone`);
  }
  if (!r.buttons.some((b) => b.id === "battle-done")) bad.push(`${what}: no NEXT ROUND button`);
  return bad;
}

const browser = await launchChromium();
const errors = [];
const note = (list) => list.forEach((e) => { errors.push(e); console.error("FAIL " + e); });
const toLineUp = async (page) => {
  await page.keyboard.press("Space");
  for (let i = 0; i < 60 && !(await page.getByTestId("battle-back").isDisabled()); i++) await page.keyboard.press("ArrowLeft");
};
try {
  for (const label of SIZES) {
    const [w, h] = label.split("x").map(Number);
    const phone = w < 500;
    for (const n of UNITS) {
      const ctx = await browser.newContext({ viewport: { width: w, height: h }, isMobile: phone, hasTouch: phone });
      const page = await ctx.newPage();
      const what = `${label} ${n} unit${n > 1 ? "s" : ""}`;
      page.on("pageerror", (e) => note([`${what}: pageerror ${e.message}`]));
      const tid = (id) => page.getByTestId(id);
      await page.goto(url); await tid("start").waitFor(); await page.keyboard.press("Enter"); await tid("coach").waitFor();
      for (let i = 0; i < n; i++) {
        const cell = page.locator('[data-testid="offers"] [data-testid^="offer-"][data-name]').first();
        for (let a = 0; a < 3; a++) { await cell.click(); if (await page.locator(`[data-testid="line-${i}-empty"].lit`).waitFor({ timeout: 800 }).then(() => true, () => false)) break; }
        if (!phone) await page.keyboard.press(String(i + 1)); else await tid(`line-${i}-empty`).click();
        await page.waitForTimeout(350);
      }
      await tid("fight").click();
      await tid("battle-play").waitFor();
      await toLineUp(page);
      const f = await readField(page);
      if (f.you && f.you.n !== n) note([`${what}: ${f.you.n} units on your line, bought ${n}`]);
      note(fieldProblems(f, what));
      await page.screenshot({ path: `${out}/battle-${n}u-${label}.png` });
      if (n === 2) {
        await page.keyboard.press("e");
        await tid("end-card").waitFor();
        await page.waitForTimeout(400);
        note(resultProblems(await readResult(page), `${what} result`));
        await page.screenshot({ path: `${out}/battle-result-${label}.png` });
      }
      await ctx.close();
    }
    // Longer lines and a chain, on the scenes: the lines as the scene has them (five a side or more).
    for (const scene of ["battle-win", "battle-six", "battle-chain-once"]) {
      const ctx = await browser.newContext({ viewport: { width: w, height: h }, isMobile: phone, hasTouch: phone });
      const page = await ctx.newPage();
      const what = `${label} ${scene}`;
      page.on("pageerror", (e) => note([`${what}: pageerror ${e.message}`]));
      await page.goto(`${url}#scene=${scene}&lang=en`);
      await page.getByTestId("battle-play").waitFor({ timeout: 30_000 });
      await toLineUp(page);
      const f = await readField(page);
      // A line past five shares the field: its pieces may be under the shop's size, the rest holds.
      const long = (f.you?.n ?? 0) > 5 || (f.them?.n ?? 0) > 5;
      const bad = fieldProblems(f, what).filter((e) => !(long && /pieces are/.test(e)));
      note(bad);
      await page.screenshot({ path: `${out}/${scene}-${label}.png` });
      if (scene === "battle-chain-once") {
        // Step to a chain of three links; its wire joins the real pieces, its number sits on the wire.
        let got = false;
        for (let i = 0; i < 200 && !got; i++) {
          await page.keyboard.press("ArrowRight");
          const wire = await page.evaluate(() => {
            const g = document.querySelector(".bv-fx .bv-beam.now:not(.ring)");
            const dot = document.querySelector(".bv-fx .bv-beam .bv-bn");
            const n = Number(document.querySelector('[data-testid="chain-count"]')?.dataset.n) || 0;
            if (!g || !dot || n < 3) return null;
            const nums = (g.querySelector("path")?.getAttribute("d") ?? "").match(/-?\d+(\.\d+)?/g)?.map(Number) ?? [];
            const box = (id) => { const c = document.querySelector(`.bv-card[data-unit="${CSS.escape(id)}"]`); const r = c?.getBoundingClientRect(); const f = c?.querySelector(".pc")?.getBoundingClientRect() ?? r; return r ? { l: r.left, r: r.right, t: r.top, b: r.bottom, cx: (f.left + f.right) / 2, hw: (f.right - f.left) / 2 } : null; };
            const d = dot.querySelector("circle")?.getBoundingClientRect() ?? dot.getBoundingClientRect();
            return { n, p0: nums.length >= 6 ? { x: nums[0], y: nums[1] } : null, p1: nums.length >= 6 ? { x: nums[4], y: nums[5] } : null, from: g.dataset.from === "clash" ? null : box(g.dataset.from), to: box(g.dataset.to), dot: { x: d.left + d.width / 2, y: d.top + d.height / 2 } };
          });
          if (!wire) continue;
          got = true;
          const bad = [];
          // The wire meets the piece at its middle column (within half the face) and at its box (tab to name), a few px of slack.
          const near = (p, b) => b && Math.abs(p.x - b.cx) <= b.hw && p.y >= b.t - 8 && p.y <= b.b + 8;
          if (!wire.p0 || !wire.p1) bad.push("the wire has no ends");
          else {
            if (wire.from && !near(wire.p0, wire.from)) bad.push(`its start (${Math.round(wire.p0.x)},${Math.round(wire.p0.y)}) is off its piece`);
            if (!near(wire.p1, wire.to)) bad.push(`its end (${Math.round(wire.p1.x)},${Math.round(wire.p1.y)}) is off its piece`);
            const lo = { x: Math.min(wire.p0.x, wire.p1.x) - 60, y: Math.min(wire.p0.y, wire.p1.y) - 60 }, hi = { x: Math.max(wire.p0.x, wire.p1.x) + 60, y: Math.max(wire.p0.y, wire.p1.y) + 60 };
            if (wire.dot.x < lo.x || wire.dot.x > hi.x || wire.dot.y < lo.y || wire.dot.y > hi.y) bad.push(`its step number sits at ${Math.round(wire.dot.x)},${Math.round(wire.dot.y)}, away from the wire`);
          }
          await page.waitForTimeout(350);
          const small = await page.evaluate(() => {
            const st = document.querySelector('[data-testid="stage"]');
            const k = st.getBoundingClientRect().width / st.offsetWidth;
            const chips = [...document.querySelectorAll('[data-testid="chain-chip"]')].map((c) => `chip ${(parseFloat(getComputedStyle(c).fontSize) * k).toFixed(1)}`).filter((t) => parseFloat(t.slice(5)) < 10.9);
            const lines = [...document.querySelectorAll(".bv-pill.two .bv-l")].filter((l) => l.scrollHeight > l.clientHeight + 3).map(() => "a result chip's line is clipped");
            const skip = document.querySelector('[data-testid="chain-skip"]');
            const skipH = skip && innerWidth < 600 ? skip.getBoundingClientRect().height : 99;
            return [...chips, ...lines, ...(skipH < 43.5 ? [`Skip chain is ${Math.round(skipH)} px tall on a phone`] : [])];
          });
          note(small.map((t) => `${what}: ${t}`));
          note(bad.map((b) => `${what}: chain of ${wire.n}: ${b}`));
          await page.screenshot({ path: `${out}/chain-${label}.png` });
        }
        if (!got) console.log(`${what}: no chain of three links in this fight (wire not checked)`);
      }
      await ctx.close();
    }
  }
} finally {
  await browser.close();
  child?.kill();
}
if (errors.length) { console.error(`${errors.length} problem(s)`); process.exit(1); }
console.log(`battle-field: ok (${SIZES.join(", ")}; screenshots in ${out})`);
