// The unit panel (the shop's inspector) reads whole (arena#32, third round): for
// every unit of the pool, sleeping and awoken, at 800×450 the rule text is
// either fully inside the panel or in an area that scrolls inside it, with Lock
// and Buy still in view; a rule's inline words never break mid-word ("heal" /
// "s"); no text under 11 px as drawn. The 10 units with the longest rule are
// also checked at 1280×720 and 390×844. Each unit is put on offer 0 by rewriting
// the run's answer, so no run has to be played to reach it. Without --url it
// builds the client and starts a server.
//   npm run mvp:panel -- [--url http://127.0.0.1:PORT/arena/] [--only gambler,medic] [--out e2e/.shots/mvp-panel]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const only = opt("only")?.split(",");
const out = opt("out") ?? "e2e/.shots/mvp-panel";
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", ARENA_CODEX: "1", MVP_POOL: "v2" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

const content = await (await fetch(url + "api/v1/content")).json();
const units = content.units.filter((u) => !only || only.includes(u.id));

/** What the panel shows now: does its text fit, and what breaks. Runs in the page. */
function measure() {
  const insp = document.querySelector('[data-testid="inspector"]');
  if (!insp) return { missing: true };
  const ir = insp.getBoundingClientRect();
  const body = insp.querySelector(".insp-body") ?? insp;
  const cs = getComputedStyle(body);
  const scrolls = body.scrollHeight > body.clientHeight + 1;
  const canScroll = cs.overflowY === "auto" || cs.overflowY === "scroll";
  // The foot's buttons (Lock, Buy, Sell, Move…): inside the panel and the window.
  const btns = [...insp.querySelectorAll(":scope > .row button, :scope > .row.actions button")];
  const lost = btns.filter((b) => { const r = b.getBoundingClientRect(); return r.bottom > innerHeight + 1 || r.bottom > ir.bottom + 1 || r.top < ir.top - 1; }).map((b) => b.textContent.trim().slice(0, 10));
  // Text cut by a box that does not scroll: any text block of the panel whose bottom is under the panel body's bottom edge.
  const br = body.getBoundingClientRect();
  const cut = scrolls && !canScroll;
  // A rule's inline word and the lowercase letters glued to it ("heal" + "s") on one line.
  const broken = [];
  for (const el of body.querySelectorAll("*")) {
    const next = el.nextSibling;
    if (!next || next.nodeType !== 3 || !/^[\p{Ll}]/u.test(next.textContent)) continue;
    const a = el.getClientRects(); const last = a[a.length - 1];
    if (!last) continue;
    const range = document.createRange(); range.setStart(next, 0); range.setEnd(next, 1);
    const c = range.getBoundingClientRect();
    if (c.height && Math.abs(c.top - last.top) > last.height * 0.6) broken.push(`${el.textContent.trim().slice(0, 12)}|${next.textContent.slice(0, 6)} el=${el.tagName}.${el.className} par=${el.parentElement.className} nextText=${JSON.stringify(next.textContent.slice(0, 12))}`);
  }
  // Text under 11 px as drawn, read through a block ancestor's scale.
  const drawn = (e) => {
    let b = e;
    while (b && b !== document.body && (getComputedStyle(b).display === "inline" || b.offsetWidth < 24 || b.offsetHeight < 24)) b = b.parentElement;
    const r = b?.getBoundingClientRect();
    const k = b && b.offsetWidth > 0 && r.width > 0 ? r.width / b.offsetWidth : 1;
    return parseFloat(getComputedStyle(e).fontSize) * (k > 0 ? k : 1);
  };
  const small = [...insp.querySelectorAll("*")].filter((e) => e.getClientRects().length && [...e.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim()) && drawn(e) > 0 && drawn(e) < 10.9).map((e) => `${e.textContent.trim().slice(0, 14)} ${drawn(e).toFixed(1)}`);
  return { scrolls, canScroll, cut, lost, broken, small: [...new Set(small)].slice(0, 5), h: Math.round(body.scrollHeight), box: Math.round(br.height), noBtns: btns.length === 0 };
}

const failures = [];
const browser = await launchChromium();
const SIZES = [["800x450", { width: 800, height: 450 }], ["1280x720", { width: 1280, height: 720 }], ["390x844", { width: 390, height: 844 }]];
const heights = new Map(); // unit id → scroll height at 800×450, to pick the longest rules for the other sizes
try {
  for (const [label, viewport] of SIZES) {
    const ctx = await browser.newContext({ viewport });
    const page = await ctx.newPage();
    let target = null;
    await page.route(/\/api\/v1\/runs(\/[^/]+)?$/, async (route) => {
      const resp = await route.fetch();
      if (!target || resp.status() >= 400 || !(resp.headers()["content-type"] ?? "").includes("json")) return route.fulfill({ response: resp });
      const j = await resp.json();
      const run = j.run ?? j;
      if (Array.isArray(run.offers) && run.offers[0]) run.offers[0] = { ...run.offers[0], unitId: target.id, tier: target.tier, cost: run.offers[0].cost };
      return route.fulfill({ response: resp, json: j });
    });
    await page.goto(url, { timeout: 60_000 });
    await page.getByTestId("start").waitFor({ timeout: 30_000 });
    await page.keyboard.press("Enter");
    await page.getByTestId("coach").waitFor({ timeout: 30_000 });
    const list = label === "800x450" ? units : [...units].sort((a, b) => (heights.get(b.id) ?? 0) - (heights.get(a.id) ?? 0)).slice(0, 10);
    for (const u of list) {
      target = u;
      await page.reload({ timeout: 60_000 });
      // A reload lands on Home; PLAY there goes back to the run in progress, whose answer is rewritten.
      await page.getByTestId("play").click({ timeout: 30_000 });
      await page.getByTestId("offers").waitFor({ timeout: 30_000 });
      const why = [];
      const offer = page.locator('[data-testid="offers"] [data-testid="offer-0"]');
      await offer.click({ timeout: 10_000 }).catch((e) => why.push(`offer: ${e.message.split("\n")[0]}`));
      await page.waitForTimeout(250);
      for (const form of ["sleeping", "awoken"]) {
        if (form === "awoken") {
          const aw = page.locator('[data-testid="inspector"] [data-testid="see-awoken"]');
          if (!(await aw.count())) continue;
          await aw.click({ timeout: 3_000 }).catch(() => {});
          await page.waitForTimeout(200);
        }
        const m = await page.evaluate(measure);
        if (m.missing) { why.push(`${form}: no panel`); continue; }
        if (form === "sleeping" && label === "800x450") heights.set(u.id, m.h);
        if (m.cut) why.push(`${form}: rule text clipped (${m.h} px of content in ${m.box}, no scroll)`);
        if (m.lost.length) why.push(`${form}: ${m.lost.join(", ")} out of view`);
        if (m.broken.length) why.push(`${form}: word broken ${m.broken.join(", ")}`);
        if (m.small.length) why.push(`${form}: text under 11 px: ${m.small.join(", ")}`);
        if (why.length && form === "sleeping") await page.screenshot({ path: `${out}/${u.id}-${label}.png` });
      }
      console.log(`${why.length ? "FAIL" : "ok  "} ${u.id} ${label}${why.length ? ": " + why.join("; ") : ""}`);
      if (why.length) failures.push(`${u.id} ${label}: ${why.join("; ")}`);
    }
    await ctx.close();
  }
} finally {
  await browser.close();
  child?.kill();
}
if (failures.length) { console.error(`\n${failures.length} panels failed`); process.exit(1); }
console.log("mvp panel: ok");
