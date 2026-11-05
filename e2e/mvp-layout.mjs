// A layout lint (arena#32, round 4): every scene (the shop, an offer selected, a line unit selected, a
// battle, its result, the Codex and Workshop windows, and every other state the scenes table holds) at
// 1920×1080, 1440×900, 1280×720, 1024×768, 800×450 and 390×844, measured, not eyeballed:
//   (a) clipped: no element hides content taller than itself (overflow hidden/clip, scrollHeight >
//       clientHeight); a scroll area (overflow auto/scroll) must be marked data-scroll and show its cue
//       (the custom property --scroll-cue, set by [data-scroll] in style.css)
//   (b) overlap: no two text-bearing boxes overlap (pieces' own badges, a wire's number and an
//       aria-hidden decoration are the stage's layers, not text boxes)
//   (c) no text under 11 px as drawn (read through the block ancestor's scale)
//   (d) no horizontal page scroll (and no vertical one in a window whose stage is fixed)
//   (e) interactive elements (not a word in running text) ≥ 32 px at ≥ 1024 wide, ≥ 44 px on touch (390×844); at 800×450 (the
//       CrazyGames frame, a mouse) ≥ 24 px
//   (f) with ≥ 3 units shown, their bounding box covers ≥ 60% of the stage's width
// Prints one line a failing scene and size, groups the failures by who owns the screen, writes
// layout.json. Without --url it builds the client and starts a scenes server. The v2-unit-* battle
// fixtures (all one layout) are sampled, 4 of them, unless --all.
//   (--windows runs the Codex and Workshop window states even with --only)
//   npm run mvp:layout -- [--url http://127.0.0.1:PORT/arena/] [--out e2e/.shots/mvp-layout] [--only shop-r3,codex-units] [--sizes 1280x720,390x844] [--all] [--shots]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/mvp-layout";
const only = opt("only")?.split(",");
const all = args.includes("--all");
const shots = args.includes("--shots");
mkdirSync(out, { recursive: true });
const ALL = ["1920x1080", "1440x900", "1280x720", "1024x768", "800x450", "390x844"];
const sizes = (opt("sizes")?.split(",") ?? ALL).map((s) => s.split("x").map(Number));

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_TUNER: "instant", ARENA_CODEX: "1", ARENA_CG: "1" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

let scenes = await (await fetch(url + "api/v1/dev/scenes")).json();
if (only) scenes = scenes.filter((s) => only.includes(s.id));
else if (!all) { let n = 0; scenes = scenes.filter((s) => !/^v2-unit-/.test(s.id) || n++ < 4); }

/** Who owns a screen this round: B the shop and the unit panel, C the battle, E the start; the rest is F's. */
const ownerOf = (s) => (s.group === "Start" && /^(signin|holding)$/.test(s.id) ? "E" : /^(shop|awaken|gift|fuse|fusion)/.test(s.id) || s.group === "Shop" ? "B" : /^(battle|v2-|crown|slay|day-end)/.test(s.id) || s.group === "Battle" ? "C" : "F");

/** The measurements, in the page. Returns the failures as {rule, what}. */
function lint({ touchSize, minTouch, minDesk, minFrame }) {
  const bad = [];
  const root = document.getElementById("app");
  const under = document.getElementById("under");
  // The dev tools (a `.dev` panel, only in a dev world) are not the product.
  // A window over the table (body.over-table) dims the table: only the window is measured, the table under it is a layer.
  const overTable = document.body.classList.contains("over-table");
  // A modal sheet (.overlay) over a page: the page under it is a layer, only the sheet is measured.
  const modal = document.querySelector("#app .overlay, body > .overlay");
  const els = [...document.querySelectorAll(modal ? `${modal.matches("#app .overlay") ? "#app .overlay" : "body > .overlay"} *` : overTable ? "#app *" : "#app *, #under *")].filter((e) => e.getClientRects().length && getComputedStyle(e).visibility !== "hidden" && getComputedStyle(e).display !== "contents" && !e.closest("details.dev, .dev"));
  const name = (e) => `${e.tagName.toLowerCase()}${e.dataset?.testid ? `[${e.dataset.testid}]` : e.className && typeof e.className === "string" ? "." + e.className.trim().split(/\s+/)[0] : ""}`;
  const add = (rule, what) => bad.push({ rule, what });

  // (a) clipped content, and unmarked scroll areas
  for (const e of els) {
    const cs = getComputedStyle(e);
    const vOver = e.scrollHeight - e.clientHeight;
    if (vOver > 3 && e.clientHeight > 0) { // 1–3 px is a glyph's overshoot, not hidden content
      if (/hidden|clip/.test(cs.overflowY) && !e.closest(".bv-fx, .stage > .bv-fx")) {
        // A box that is only a mask for its children's animation (zero-height gaps) is not text: it must hold text or a control to count.
        if ((e.textContent ?? "").trim() || e.querySelector("button,input")) add("a", `${name(e)} hides ${Math.round(vOver)} px of content (${e.clientHeight} px box)`);
      } else if (/auto|scroll/.test(cs.overflowY) && e !== document.scrollingElement) {
        if (!e.hasAttribute("data-scroll")) add("a", `${name(e)} scrolls ${Math.round(vOver)} px and is not marked data-scroll`);
        else if (cs.getPropertyValue("--scroll-cue").trim() === "") add("a", `${name(e)} is data-scroll with no visible cue`);
      }
    }
  }
  // (b) overlap of text-bearing boxes, by the text's own rects
  const texts = [];
  for (const e of els) {
    if (e.closest("[aria-hidden='true']") && !e.closest("button")) continue;
    if (e.closest(".pc, .bv-fx, .bv-beam, .coach-pair, .start-chain, .tpill, .track-dot")) continue;
    const own = [...e.childNodes].filter((n) => n.nodeType === 3 && n.textContent.trim());
    if (!own.length) continue;
    const range = document.createRange();
    let l = Infinity, t = Infinity, r = -Infinity, b = -Infinity;
    // Text the box's scroll ancestors cut away (scrolled out of view) is not on screen: only what shows counts.
    const clip = [];
    for (let n = e.parentElement; n && n !== document.body; n = n.parentElement) { const o = getComputedStyle(n); if (/(auto|scroll|hidden|clip)/.test(o.overflowX + o.overflowY)) clip.push(n.getBoundingClientRect()); }
    for (const n of own) { range.selectNodeContents(n); for (const q0 of range.getClientRects()) {
      let ql = q0.left, qt = q0.top, qr = q0.right, qb = q0.bottom;
      for (const c of clip) { ql = Math.max(ql, c.left); qt = Math.max(qt, c.top); qr = Math.min(qr, c.right); qb = Math.min(qb, c.bottom); }
      if (qr - ql < 1 || qb - qt < 1) continue;
      l = Math.min(l, ql); t = Math.min(t, qt); r = Math.max(r, qr); b = Math.max(b, qb); } }
    if (l === Infinity) continue;
    // Text another element covers (under a result card, a sheet) is not on screen either: look at what is on top at its centre.
    const top = document.elementFromPoint(Math.min(innerWidth - 1, Math.max(0, (l + r) / 2)), Math.min(innerHeight - 1, Math.max(0, (t + b) / 2)));
    if (top && !e.contains(top) && !top.contains(e) && !top.closest("[aria-hidden='true']")) continue;
    texts.push({ e, l, t, r, b });
  }
  const seen = new Set();
  for (let i = 0; i < texts.length; i++) for (let j = i + 1; j < texts.length; j++) {
    const a = texts[i], c = texts[j];
    if (a.e.contains(c.e) || c.e.contains(a.e)) continue;
    const w = Math.min(a.r, c.r) - Math.max(a.l, c.l), h = Math.min(a.b, c.b) - Math.max(a.t, c.t);
    if (w > 3 && h > 3 && w * h > 16) {
      // Two windows over each other (the table under a window) are layers: only boxes of the same layer count.
      if (!!a.e.closest("#under") !== !!c.e.closest("#under")) continue;
      // Content scrolls under a sticky bar (Back, Close): that is the scroll area's own layering.
      const sticky = (x) => { for (let n = x; n && n !== document.body; n = n.parentElement) { const p = getComputedStyle(n).position; if (p === "sticky" || p === "fixed") return n; } return null; };
      const sa = sticky(a.e), sc = sticky(c.e);
      if ((sa || sc) && sa !== sc) continue;
      const key = `${name(a.e)}|${name(c.e)}`;
      if (!seen.has(key)) { seen.add(key); add("b", `"${a.e.textContent.trim().slice(0, 14)}" (${name(a.e)}) overlaps "${c.e.textContent.trim().slice(0, 14)}" (${name(c.e)}) by ${Math.round(w)}×${Math.round(h)} px`); }
    }
  }
  // (c) text under 11 px as drawn
  const drawn = (e) => {
    let b = e;
    while (b && b !== document.body && (getComputedStyle(b).display === "inline" || b.offsetWidth < 24 || b.offsetHeight < 24)) b = b.parentElement;
    const r = b?.getBoundingClientRect();
    const k = b && b.offsetWidth > 0 && r.width > 0 ? r.width / b.offsetWidth : 1;
    return parseFloat(getComputedStyle(e).fontSize) * (k > 0 ? k : 1);
  };
  const smalls = new Map();
  for (const e of els) {
    if (![...e.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim())) continue;
    const px = drawn(e);
    if (px > 0 && px < 10.9) smalls.set(`${e.textContent.trim().slice(0, 16)}`, `${name(e)} ${px.toFixed(1)} px`);
  }
  for (const [t, d] of [...smalls].slice(0, 6)) add("c", `"${t}" ${d}`);
  // (d) page scroll
  const de = document.documentElement;
  if (de.scrollWidth > innerWidth + 1) add("d", `the page scrolls sideways (${de.scrollWidth} px in ${innerWidth})`);
  const stageEl = document.querySelector('[data-testid="stage"]');
  if (stageEl && de.scrollHeight > innerHeight + 1) add("d", `the page scrolls down (${de.scrollHeight} px in ${innerHeight})`);
  // (e) interactive elements
  const min = touchSize ? minTouch : innerWidth >= 1024 ? minDesk : minFrame;
  const small = [];
  for (const e of els) {
    if (!e.matches("button, a[href], input, select, textarea, [role='button'], [data-testid$='-empty']") || e.disabled || e.closest("[aria-hidden='true']")) continue;
    // A word in running text (a rule's term, a link in a sentence) is text, not a control of its own.
    if (getComputedStyle(e).display === "inline" || e.matches("button.t, .t-glue > button, .seal-rule button")) continue;
    const r = e.getBoundingClientRect();
    if (r.width < 1 || r.height < 1 || r.right < 0 || r.bottom < 0 || r.left > innerWidth || r.top > innerHeight) continue;
    if (Math.min(r.width, r.height) < min) small.push(`${name(e)} ${Math.round(r.width)}×${Math.round(r.height)}`);
  }
  for (const s of [...new Set(small)].slice(0, 6)) add("e", `${s} under ${min} px`);
  if (new Set(small).size > 6) add("e", `… and ${new Set(small).size - 6} more under ${min} px`);
  // (f) the units fill the field
  if (stageEl && !overTable) {
    const sr = stageEl.getBoundingClientRect();
    const pcs = [...stageEl.querySelectorAll(".pc:not(.pc-XS)")].filter((e) => !e.closest(".tb-insp, .inspector, .coach, .sheet") && e.getClientRects().length);
    if (pcs.length >= 3) {
      let l = Infinity, r = -Infinity;
      for (const p of pcs) { const q = p.getBoundingClientRect(); l = Math.min(l, q.left); r = Math.max(r, q.right); }
      const cover = (r - l) / sr.width;
      if (cover < 0.6) add("f", `${pcs.length} units span ${Math.round(cover * 100)}% of the stage's width (≥ 60%)`);
    }
  }
  return bad;
}

const results = [];
const browser = await launchChromium();
try {
  for (const s of scenes) {
    for (const [w, h] of sizes) {
      const S = `${w}x${h}`;
      const touchSize = w < 600;
      const ctx = await browser.newContext({ viewport: { width: w, height: h }, isMobile: touchSize, hasTouch: touchSize, deviceScaleFactor: 1 });
      const page = await ctx.newPage();
      const why = [];
      page.on("pageerror", (e) => why.push({ rule: "x", what: `pageerror: ${e.message.slice(0, 80)}` }));
      await page.goto(`${url}#scene=${s.id}&lang=en`, { timeout: 120_000 }).catch((e) => why.push({ rule: "x", what: `load: ${e.message.split("\n")[0]}` }));
      await page.waitForFunction(() => (document.querySelector("#app")?.textContent ?? "").trim().length > 20, null, { timeout: 60_000 }).catch(() => why.push({ rule: "x", what: "blank" }));
      await page.waitForTimeout(1300);
      const bad = await page.evaluate(lint, { touchSize, minTouch: 44, minDesk: 32, minFrame: 24 }).catch((e) => [{ rule: "x", what: `lint: ${e.message.split("\n")[0]}` }]);
      why.push(...bad);
      if (shots || why.length) await page.screenshot({ path: `${out}/${s.id}-${S}.png` }).catch(() => {});
      const owner = ownerOf(s);
      results.push({ scene: s.id, group: s.group, size: S, owner, bad: why });
      console.log(`${why.length ? "FAIL" : "ok  "} ${s.id} ${S} [${owner}]${why.length ? ": " + why.map((x) => `(${x.rule}) ${x.what}`).join("; ").slice(0, 600) : ""}`);
      await ctx.close();
    }
  }
  // The windows over the table: the Codex and the Workshop, opened from the shop (not a scene of their own).
  if (!only || args.includes("--windows")) for (const [w, h] of sizes) {
    const S = `${w}x${h}`;
    const touchSize = w < 600;
    for (const [id, open] of [
      ["window-codex", async (page) => { await page.getByTestId("legend-open").click(); await page.waitForSelector('[data-testid="codex-tab-units"]', { timeout: 10_000 }); }],
      ["window-workshop", async (page) => { await page.getByTestId("menu-open").click(); await page.getByTestId("menu-ideas").click(); await page.waitForSelector('[data-testid="workshop-back"]', { timeout: 10_000 }); }],
    ]) {
      const ctx = await browser.newContext({ viewport: { width: w, height: h }, isMobile: touchSize, hasTouch: touchSize, deviceScaleFactor: 1 });
      const page = await ctx.newPage();
      const why = [];
      await page.goto(`${url}#scene=shop-r3&lang=en`, { timeout: 120_000 });
      await page.waitForSelector('[data-testid="fight"]', { timeout: 60_000 }).catch(() => why.push({ rule: "x", what: "no shop" }));
      await page.waitForTimeout(800);
      await open(page).catch((e) => why.push({ rule: "x", what: `open: ${e.message.split("\n")[0]}` }));
      await page.waitForTimeout(900);
      why.push(...(await page.evaluate(lint, { touchSize, minTouch: 44, minDesk: 32, minFrame: 24 }).catch((e) => [{ rule: "x", what: `lint: ${e.message.split("\n")[0]}` }])));
      if (shots || why.length) await page.screenshot({ path: `${out}/${id}-${S}.png` }).catch(() => {});
      results.push({ scene: id, group: "Windows", size: S, owner: "F", bad: why });
      console.log(`${why.length ? "FAIL" : "ok  "} ${id} ${S} [F]${why.length ? ": " + why.map((x) => `(${x.rule}) ${x.what}`).join("; ").slice(0, 600) : ""}`);
      await ctx.close();
    }
  }
} finally {
  await browser.close();
  child?.kill();
}
writeFileSync(`${out}/layout.json`, JSON.stringify({ sizes: sizes.map(([w, h]) => `${w}x${h}`), results }, null, 1));
const failing = results.filter((r) => r.bad.length);
const byRule = {}, byOwner = {};
for (const r of failing) { for (const b of r.bad) byRule[b.rule] = (byRule[b.rule] ?? 0) + 1; byOwner[r.owner] = (byOwner[r.owner] ?? 0) + 1; }
console.log(`\n${failing.length} of ${results.length} (scene, size) pairs fail. By rule: ${JSON.stringify(byRule)}. By owner: ${JSON.stringify(byOwner)}`);
if (failing.length) process.exit(1);
console.log("mvp layout: ok");
