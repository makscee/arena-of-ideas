// The shop's field and the unit panel, measured (arena#28, round 4: "space is not used
// properly, in the shop when selecting a unit I have to scroll a lot on its card").
// Boxes, not eyeballs:
//  1. FIELD, panel closed, at 1920×1080, 1440×900, 1280×720, 1024×768, 800×450 and 390×844:
//     the line's 5 slots span ≥ 70% of the stage's width (landscape), a piece's edge is
//     ≥ 11% of the stage's height, their `?` wells and the bench stand in the same columns as
//     the line's slots, the labels (NEXT, YOUR LINE, BENCH) sit in a left gutter that no slot
//     enters, the field and the tray leave no band of blank stage (1024×768 uses its height).
//  2. PANEL, for every unit of the pool (an offer, a line unit, a bench unit; sleeping and
//     awoken): landscape (≥ 1024 wide): as tall as its content needs, nothing scrolls (scrollHeight
//     ≤ clientHeight), every box inside the panel, no two text or art boxes intersect (art vs text,
//     text vs buttons), the stat chips inside, the art a hero piece of 96–120 px at 1440 wide,
//     and docked: no line or bench slot is covered by it (also at 800×450). Portrait (390×844):
//     a bottom sheet of at most 55% of the height, Lock/Buy/Sell below the text, never over Reroll
//     or Fight, scrolling only when the text is taller, with a cue.
// Each unit is put on offer 0, in the line and on the bench by rewriting the run's answer.
// Without --url it builds the client and starts a server.
//   npm run mvp:field -- [--url http://127.0.0.1:PORT/arena/] [--only gambler,medic] [--sizes 1440x900,390x844] [--out e2e/.shots/mvp-field]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const only = opt("only")?.split(",");
const out = opt("out") ?? "e2e/.shots/mvp-field";
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", ARENA_CODEX: "1", MVP_POOL: "v2", ARENA_GUEST_JOINS_PER_IP: "500", ARENA_GUEST_JOINS_PER_HOUR: "500", ARENA_SCENES: "1" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

const content = await (await fetch(url + "api/v1/content")).json();
const units = content.units.filter((u) => !only || only.includes(u.id));
const ALL = [["1920x1080", 1920, 1080], ["1440x900", 1440, 900], ["1280x720", 1280, 720], ["1024x768", 1024, 768], ["800x450", 800, 450], ["390x844", 390, 844]];
const SIZES = ALL.filter(([l]) => !opt("sizes") || opt("sizes").split(",").includes(l));

/** In the page: the stage's drawn box and the boxes the field tests read. */
function field() {
  const R = (el) => { const r = el.getBoundingClientRect(); return { x: r.left, y: r.top, w: r.width, h: r.height, cx: r.left + r.width / 2, cy: r.top + r.height / 2, r: r.right, b: r.bottom }; };
  const st = document.querySelector('[data-testid="stage"]');
  if (!st) return { missing: "stage" };
  const stage = R(st);
  const cell = (id) => document.querySelector(`[data-testid="${id}"]`);
  const line = [0, 1, 2, 3, 4].map((i) => cell(`line-${i}`) ?? cell(`line-${i}-empty`)).map((e) => (e ? R(e) : null));
  const bench = [0, 1, 2].map((i) => cell(`bench-${i}`) ?? cell(`bench-${i}-empty`)).map((e) => (e ? R(e) : null));
  const wells = [0, 1, 2, 3, 4].map((i) => cell(`their-${i}`)).map((e) => (e ? R(e) : null));
  // A line piece's edge: the piece itself, or the empty slot's well (the same --s).
  const piece = [0, 1, 2, 3, 4].map((i) => { const c = cell(`line-${i}`) ?? cell(`line-${i}-empty`); const e = c?.querySelector(".pc") ?? c?.querySelector(".cell-well"); return e ? R(e).w : 0; });
  const labels = [...document.querySelectorAll(".tb-side, .bench-label")].map((e) => R(e));
  const tray = cell("tray") ? R(cell("tray")) : null;
  const hud = cell("hud") ? R(cell("hud")) : null;
  return { stage, line, bench, wells, piece, labels, tray, hud, vw: innerWidth, vh: innerHeight, scroll: document.documentElement.scrollHeight - document.documentElement.clientHeight };
}

/** In the page: what the open panel does. */
function panel() {
  const insp = document.querySelector('[data-testid="inspector"]');
  if (!insp || insp.hidden) return { missing: true };
  const R = (el) => { const r = el.getBoundingClientRect(); return { x: r.left, y: r.top, r: r.right, b: r.bottom, w: r.width, h: r.height }; };
  const ir = R(insp);
  const body = insp.querySelector(".insp-body") ?? insp;
  const hero = insp.querySelector(".sheet-hero");
  const inside = (el, root) => root && root.contains(el);
  // Boxes: the art (the hero piece and everything hanging off it), the text (every line of every text node outside the art and the buttons), the buttons.
  const art = [];
  for (const e of insp.querySelectorAll(".pc, .pc *")) if (e.getClientRects().length) art.push({ n: `art:${e.className || e.tagName}`, ...R(e) });
  const buttons = [...insp.querySelectorAll(":scope > .row button, :scope > .row.actions button")].filter((b) => b.getClientRects().length).map((b) => ({ n: `btn:${b.dataset.testid || b.textContent.trim().slice(0, 8)}`, ...R(b) }));
  const text = [];
  const cut = [];
  const walker = document.createTreeWalker(insp, NodeFilter.SHOW_TEXT);
  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    if (!n.textContent.trim()) continue;
    const p = n.parentElement;
    if (inside(p, hero) || p.closest("button") || p.closest(".pc")) continue;
    const cs = getComputedStyle(p);
    if (cs.visibility === "hidden" || cs.display === "none") continue;
    const rg = document.createRange(); rg.selectNodeContents(n);
    for (const r of rg.getClientRects()) {
      if (r.width <= 1 || r.height <= 1) continue;
      // What a clipping ancestor (a scrolling body, an ellipsis) leaves visible of this line.
      let c = { x: r.left, y: r.top, r: r.right, b: r.bottom };
      for (let a = p; a && a !== insp.parentElement; a = a.parentElement) {
        const cs = getComputedStyle(a);
        if (cs.overflowX === "visible" && cs.overflowY === "visible") continue;
        const ar = a.getBoundingClientRect();
        c = { x: Math.max(c.x, ar.left), y: Math.max(c.y, ar.top), r: Math.min(c.r, ar.right), b: Math.min(c.b, ar.bottom) };
      }
      const name = `text:${n.textContent.trim().slice(0, 14)}`;
      if (c.r - c.x < r.width - 1.5 || c.b - c.y < r.height - 1.5) cut.push(name);
      if (c.r > c.x && c.b > c.y) text.push({ n: name, x: c.x, y: c.y, r: c.r, b: c.b, w: c.r - c.x, h: c.b - c.y });
    }
  }
  const hit = (a, b) => Math.min(a.r, b.r) - Math.max(a.x, b.x) > 1.5 && Math.min(a.b, b.b) - Math.max(a.y, b.y) > 1.5;
  const clash = [];
  for (const a of art) for (const t of text) if (hit(a, t)) clash.push(`${a.n} × ${t.n}`);
  for (const b of buttons) for (const t of text) if (hit(b, t)) clash.push(`${b.n} × ${t.n}`);
  for (const b of buttons) for (const a of art) if (hit(b, a)) clash.push(`${b.n} × ${a.n}`);
  for (let i = 0; i < text.length; i++) for (let j = i + 1; j < text.length; j++) if (hit(text[i], text[j])) clash.push(`${text[i].n} × ${text[j].n}`);
  // Boxes outside the panel's own box.
  const outside = [...art, ...text, ...buttons].filter((b) => b.x < ir.x - 1 || b.r > ir.r + 1 || b.y < ir.y - 1 || b.b > ir.b + 1).map((b) => b.n);
  const chips = [...insp.querySelectorAll(".pc-b")].filter((e) => e.getClientRects().length).map(R).filter((b) => b.x < ir.x - 1 || b.r > ir.r + 1 || b.y < ir.y - 1 || b.b > ir.b + 1).length;
  const st = document.querySelector('[data-testid="stage"]').getBoundingClientRect();
  const heroPc = hero?.querySelector(".pc");
  const slots = [...document.querySelectorAll('[data-testid^="line-"], [data-testid^="bench-"]')].filter((e) => e.getClientRects().length).map((e) => ({ n: e.dataset.testid, ...R(e) }));
  const covered = slots.filter((s) => hit(s, ir)).map((s) => s.n);
  const go = ["reroll", "fight"].map((id) => document.querySelector(`[data-testid="${id}"]`)).filter(Boolean).map((e) => ({ n: e.dataset.testid, ...R(e) }));
  const over = go.filter((g) => hit(g, ir)).map((g) => g.n);
  const tray = document.querySelector('[data-testid="tray"]')?.getBoundingClientRect();
  const hud = document.querySelector('[data-testid="hud"]')?.getBoundingClientRect();
  const scrolls = Math.max(body.scrollHeight - body.clientHeight, insp.scrollHeight - insp.clientHeight) > 1;
  const cue = !!insp.querySelector('[data-testid="insp-more"]');
  // The rule's own text sits above the buttons.
  const lastText = text.reduce((m, t) => Math.max(m, t.b), 0);
  const firstBtn = buttons.filter((b) => !b.n.includes("inspector-close")).reduce((m, b) => Math.min(m, b.y), 1e9);
  return { ir, scrolls, cue, cut: [...new Set(cut)].slice(0, 3), clash: [...new Set(clash)].slice(0, 4), outside: [...new Set(outside)].slice(0, 4), chips, heroEdge: heroPc ? heroPc.getBoundingClientRect().width / st.width : 0, lastText: Math.round(lastText), firstBtn: Math.round(firstBtn), covered, over, belowTray: tray ? ir.b - tray.top : 0, aboveHud: hud ? hud.bottom - ir.y : 0, textAboveButtons: lastText <= firstBtn + 1, stageH: st.height, vh: innerHeight, noButtons: buttons.length === 0 };
}

/** In the page: the shop's controls and its scroll areas (round 5, from the layout lint). */
function controls() {
  const touch = innerWidth < innerHeight * 0.8;
  // As drawn (the stage is scaled): 44 on a phone, 32 from 1024 wide, 24 in the 800×450 frame (a mouse), the lint's own thresholds.
  const want = touch ? 44 : innerWidth >= 1024 ? 32 : 24;
  const small = [...document.querySelectorAll("#app button")]
    .filter((e) => e.getClientRects().length && !e.closest(".t, .sheet-form, .tb-bench-help") && e.getBoundingClientRect().height < want - 0.5)
    .map((e) => `${e.dataset.testid || e.className || e.textContent.trim().slice(0, 10)} ${Math.round(e.getBoundingClientRect().width)}×${Math.round(e.getBoundingClientRect().height)}`);
  // A scroll area that scrolls says so: marked, with its cue.
  const areas = [...document.querySelectorAll(".insp-body, .sheet")].filter((e) => e.getClientRects().length && e.scrollHeight > e.clientHeight + 1);
  const unmarked = areas.filter((e) => !e.hasAttribute("data-scroll")).map((e) => `${e.className} scrolls ${e.scrollHeight - e.clientHeight} px`);
  const noCue = areas.filter((e) => e.matches(".insp-body") && !e.parentElement.querySelector('[data-testid="insp-more"]')).length;
  // The close button never sits on text.
  const x = document.querySelector('[data-testid="inspector-close"]');
  const insp = document.querySelector('[data-testid="inspector"]');
  let covers = [];
  if (x && insp && !insp.hidden) {
    const xr = x.getBoundingClientRect();
    const walker = document.createTreeWalker(insp, NodeFilter.SHOW_TEXT);
    for (let n = walker.nextNode(); n; n = walker.nextNode()) {
      if (!n.textContent.trim() || n.parentElement.closest("button, .pc")) continue;
      const cs = getComputedStyle(n.parentElement);
      if (cs.display === "none" || cs.visibility === "hidden") continue;
      const rg = document.createRange(); rg.selectNodeContents(n);
      for (const r of rg.getClientRects()) if (Math.min(r.right, xr.right) - Math.max(r.left, xr.left) > 2 && Math.min(r.bottom, xr.bottom) - Math.max(r.top, xr.top) > 2) covers.push(n.textContent.trim().slice(0, 14));
    }
  }
  return { small: [...new Set(small)].slice(0, 4), unmarked: [...new Set(unmarked)].slice(0, 3), noCue, covers: [...new Set(covers)].slice(0, 2) };
}

const failures = [];
const browser = await launchChromium();
const fail = (label, what, why) => { failures.push(`${label} ${what}: ${why}`); console.log(`FAIL ${label} ${what}: ${why}`); };

async function runSize([label, w, h]) {
  const portrait = w < h * 0.8;
  const ctx = await browser.newContext({ viewport: { width: w, height: h }, hasTouch: portrait, isMobile: portrait });
  const page = await ctx.newPage();
  let target = null;
  await page.route(/\/api\/v1\/runs(\/[^/]+)?$/, async (route) => {
    const resp = await route.fetch();
    if (!target || resp.status() >= 400 || !(resp.headers()["content-type"] ?? "").includes("json")) return route.fulfill({ response: resp });
    const j = await resp.json();
    const run = j.run ?? j;
    if (Array.isArray(run.offers) && run.offers[0]) run.offers[0] = { ...run.offers[0], unitId: target.id, tier: target.tier };
    const lu = (form, uid) => ({ uid, kind: "unit", unitId: target.id, name: target.name, emoji: target.emoji, copies: form === "awoken" ? 3 : 1, form, stats: target.base, recipe: target.forms[form] });
    run.line = [lu("sleeping", "t1")];
    run.bench = [lu("awoken", "t2")];
    return route.fulfill({ response: resp, json: j });
  });
  page.on("pageerror", (e) => fail(label, "page", e.message));
  await page.goto(url, { timeout: 60_000 });
  await page.getByTestId("start").waitFor({ timeout: 30_000 });
  await page.keyboard.press("Enter");
  await page.getByTestId("fight").waitFor({ timeout: 30_000 });
  await page.waitForTimeout(500);

  // 1. The field, panel closed (their line and the bench in their columns).
  const f = await page.evaluate(field);
  if (f.missing) fail(label, "field", `no ${f.missing}`);
  else {
    const sw = f.stage.w, sh = f.stage.h;
    const cs = f.line.filter(Boolean).map((r) => r.cx);
    if (cs.length === 5 && !portrait) {
      const span = (cs[4] - cs[0]) / sw;
      if (span < 0.7) fail(label, "field", `the 5 slot centres span ${(span * 100).toFixed(0)}% of the stage's width, want ≥ 70%`);
      const pcs = f.piece.length ? Math.min(...f.piece) / sh : 0;
      if (pcs < 0.11) fail(label, "field", `a piece's edge is ${(pcs * 100).toFixed(1)}% of the stage's height, want ≥ 11%`);
      const wc = f.wells.filter(Boolean).map((r) => r.cx);
      if (wc.length !== 5 || wc.some((x, i) => Math.abs(x - cs[i]) > 2)) fail(label, "field", `their wells are not in the slots' columns (${wc.map(Math.round)} vs ${cs.map(Math.round)})`);
      const bc = f.bench.filter(Boolean).map((r) => r.cx);
      if (bc.length !== 3 || bc.some((x, i) => Math.abs(x - cs[i]) > 2)) fail(label, "field", `the bench is not in the slots' first columns (${bc.map(Math.round)} vs ${cs.slice(0, 3).map(Math.round)})`);
      const gutter = f.stage.x + sw * 0.115;
      const bad = f.labels.filter((l) => l.r > gutter + 1 || l.x < f.stage.x - 1);
      if (bad.length) fail(label, "field", `${bad.length} label(s) leave the left gutter (right edge ${Math.round(bad[0].r - f.stage.x)} of the stage's ${Math.round(sw)})`);
      const first = Math.min(...f.line.filter(Boolean).map((r) => r.x), ...f.wells.filter(Boolean).map((r) => r.x));
      if (f.labels.some((l) => l.r > first + 1)) fail(label, "field", "a label runs into the first slot");
      // No band of blank stage: the field's last row and the tray leave less than 14% of the stage's height between them.
      const lastRow = Math.max(...f.bench.filter(Boolean).map((r) => r.b), ...f.line.filter(Boolean).map((r) => r.b));
      if (f.tray && (f.tray.y - lastRow) / sh > 0.14) fail(label, "field", `${Math.round(f.tray.y - lastRow)} px of blank stage between the field and the tray`);
    }
    if (f.scroll > 0) fail(label, "field", `the page scrolls by ${f.scroll}px`);
    await page.screenshot({ path: `${out}/field-${label}.png` });
  }

  // 2. The panel, for every unit.
  const heights = [];
  for (const u of units) {
    target = u;
    await page.reload({ timeout: 60_000 });
    await page.getByTestId("play").click({ timeout: 30_000 });
    await page.getByTestId("offers").waitFor({ timeout: 30_000 });
    const why = [];
    const check = async (what) => {
      await page.waitForTimeout(120);
      const m = await page.evaluate(panel);
      if (m.missing) return why.push(`${what}: no panel`);
      heights.push(m.ir.b - m.ir.y);
      if (m.noButtons) why.push(`${what}: no buttons`);
      if (m.cut.length && (portrait ? !m.scrolls : true)) why.push(`${what}: text is cut off (${m.cut.join(", ")})`);
      if (m.clash.length) why.push(`${what}: boxes intersect (${m.clash.join("; ")})`);
      if (m.outside.length) why.push(`${what}: boxes outside the panel (${m.outside.join(", ")})`);
      if (m.chips) why.push(`${what}: ${m.chips} stat chip(s) outside the panel`);
      if (m.covered.length && !portrait) why.push(`${what}: the panel covers ${m.covered.join(", ")}`);
      if (m.over.length) why.push(`${what}: the panel covers ${m.over.join(", ")}`);
      if (m.belowTray > 1 && !portrait) why.push(`${what}: the panel runs ${Math.round(m.belowTray)} px into the tray`);
      if (m.aboveHud > 1 && !portrait) why.push(`${what}: the panel sits ${Math.round(m.aboveHud)} px over the HUD`);
      if (!m.textAboveButtons) why.push(`${what}: text runs under the buttons (text ends ${m.lastText}, buttons start ${m.firstBtn})`);
      if (portrait) {
        if (m.ir.h > m.vh * 0.55 + 1) why.push(`${what}: the sheet is ${Math.round((m.ir.h / m.vh) * 100)}% of the height, want ≤ 55%`);
        if (m.scrolls && !m.cue) why.push(`${what}: the text scrolls with no cue`);
      } else if (w >= 1024) {
        if (m.scrolls) why.push(`${what}: the panel scrolls`);
        if (m.heroEdge < 0.066 || m.heroEdge > 0.09) why.push(`${what}: the hero piece is ${(m.heroEdge * 100).toFixed(1)}% of the stage's width, want 6.6%–9% (96–120 px at 1440 wide)`);
      }
      return m;
    };
    await page.locator('[data-testid="offers"] [data-testid="offer-0"]').click({ timeout: 10_000 }).catch((e) => why.push(`offer: ${e.message.split("\n")[0]}`));
    await check("offer, sleeping");
    const aw = page.locator('[data-testid="inspector"] [data-testid="see-awoken"]');
    if (await aw.count()) { await aw.click({ timeout: 3_000 }).catch(() => {}); await check("offer, awoken"); }
    await page.keyboard.press("Escape");
    await page.locator('[data-testid="line-0"]').click({ timeout: 10_000 }).catch((e) => why.push(`line: ${e.message.split("\n")[0]}`));
    await check("line unit");
    await page.keyboard.press("Escape");
    await page.locator('[data-testid="bench-0"]').click({ timeout: 10_000 }).catch((e) => why.push(`bench: ${e.message.split("\n")[0]}`));
    await check("bench unit, awoken");
    if (why.length) await page.screenshot({ path: `${out}/${u.id}-${label}.png` });
    if (why.length) fail(label, u.id, why.slice(0, 3).join("; "));
  }
  console.log(`${label}: ${units.length} units, panel heights ${Math.min(...heights).toFixed(0)}–${Math.max(...heights).toFixed(0)} px`);
  await ctx.close();
}

/** In the page: the line's When tabs. Words are the plan at every size; with a panel open the columns are narrow, so a word stays wherever it fits its column (≤ 10 characters) and never overlaps a neighbour. */
function tabs() {
  const R = (e) => { const r = e.getBoundingClientRect(); return { x: r.left, y: r.top, r: r.right, b: r.bottom }; };
  const list = [...document.querySelectorAll('.tb-line .pc-tab')].filter((e) => e.getClientRects().length);
  const boxes = list.map((e) => ({ label: e.title, word: !!e.querySelector(".tw") && getComputedStyle(e.querySelector(".tw")).display !== "none", ...R(e) }));
  const clash = [];
  for (let i = 0; i < boxes.length; i++) for (let j = i + 1; j < boxes.length; j++) {
    const a = boxes[i], b = boxes[j];
    if (Math.min(a.r, b.r) - Math.max(a.x, b.x) > 1 && Math.min(a.b, b.b) - Math.max(a.y, b.y) > 1) clash.push(`${a.label} × ${b.label}`);
  }
  return { n: boxes.length, noWord: boxes.filter((b) => !b.word && b.label.length <= 10).map((b) => b.label), clash: clash.slice(0, 3) };
}

/** The shop's scenes: controls at least 32 px (44 on a phone), every scroll area marked with a cue, the close button off the text. */
async function runScenes([label, w, h]) {
  const portrait = w < h * 0.8;
  const ctx = await browser.newContext({ viewport: { width: w, height: h }, hasTouch: portrait, isMobile: portrait });
  for (const scene of ["shop-r1", "shop-r3", "shop-offer", "shop-unit", "gift", "fuse", "forge-unforged", "forge-ready", "forge-claim", "forge-discovered", "fusion-now"]) {
    const page = await ctx.newPage();
    await page.goto(`${url}#scene=${scene}&lang=en`);
    await page.waitForSelector('[data-testid="fight"]', { timeout: 60_000 }).catch(() => {});
    await page.waitForTimeout(1200);
    const m = await page.evaluate(controls);
    const why = [];
    // The line's tabs, with the panel closed and open (landscape).
    if (scene === "shop-unit" && !portrait) {
      for (const open of [false, true]) {
        if (!open) { await page.keyboard.press("Escape"); await page.waitForTimeout(250); }
        else { await page.locator('[data-testid="line-0"]').click(); await page.waitForTimeout(300); }
        const t = await page.evaluate(tabs);
        if (t.n < 3) why.push(`tabs: only ${t.n} on the line`);
        if (t.noWord.length) why.push(`${open ? "panel open" : "panel closed"}: no word on the When tab of ${t.noWord.join(", ")}`);
        if (t.clash.length) why.push(`${open ? "panel open" : "panel closed"}: tabs overlap (${t.clash.join("; ")})`);
      }
    }
    if (m.small.length) why.push(`controls under ${portrait ? 44 : w >= 1024 ? 32 : 24} px drawn: ${m.small.join(", ")}`);
    if (m.unmarked.length) why.push(`unmarked scroll: ${m.unmarked.join(", ")}`);
    if (m.noCue) why.push("a scrolling panel with no cue");
    if (m.covers.length) why.push(`the ✕ covers text: ${m.covers.join(", ")}`);
    if (why.length) fail(label, scene, why.join("; "));
    await page.close();
  }
  await ctx.close();
  console.log(`${label}: scenes checked`);
}

try {
  await Promise.all([...SIZES.map(runSize), ...SIZES.map(runScenes)]);
} finally {
  await browser.close();
  child?.kill();
}
if (failures.length) { console.error(`\n${failures.length} failures`); process.exit(1); }
console.log("mvp field: ok");
