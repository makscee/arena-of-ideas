// #921: a keyword's underline is as long as its word. For every underlined
// term (mobile/ui/term.ts: a status, target, stat, condition, a summoned
// unit) in the shop's unit panel and in each Codex unit's sheet (sleeping and
// awoken), on a phone (360) and a desktop (1280), EN and RU, it compares where
// the underline is really painted (its pixels, the text made transparent and
// the underline magenta) with its text's box, line by line. It fails when an
// underline runs under an icon, a number or a space, stops short of the word,
// or a wrapped term's line has none. Without --url it builds the client and
// starts a local server.
//   npm run mvp:underlines -- [--url http://127.0.0.1:<port>/arena/] [--out dir] [--limit n]
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";
import { localeOf } from "./lang.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
let url = opt("url");
const out = opt("out") ?? "e2e/.shots/underlines";
const limit = Number(opt("limit") ?? 1000);
mkdirSync(out, { recursive: true });
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", STATIC_DIR: "mobile/dist" }, stdio: ["ignore", "ignore", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
  for (let i = 0; i < 300; i++) {
    try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
    await new Promise((r) => setTimeout(r, 200));
  }
}

const DPR = 2;
const TOL = 3; // css px: a dotted line's last dot may stop a gap short
/** The text made transparent and the dotted line magenta: what is left under
 * a word is what reads as its underline. HIDE then takes the terms away, so
 * the difference is every mark a term paints under its text (its dotted line,
 * and a drop shadow or border that reads as a second underline). */
const PAINT = `
  button.t, button.t * { color: transparent !important; text-shadow: none !important; transition: none !important; animation: none !important; }
  button.t svg, button.t img, button.t .t-emoji { visibility: hidden !important; }
  button.t u.changed { text-decoration-color: transparent !important; }
  button.t { text-decoration-color: rgb(255, 0, 255) !important; }
`;
const HIDE = "button.t { visibility: hidden !important; }";

/** Every visible term inside `root`: its id, text, whether it is underlined,
 * and per line the text's box (trimmed of spaces, icons left out). */
async function terms(page, root) {
  return page.evaluate((sel) => {
    const rootEl = [...document.querySelectorAll(sel)].pop();
    if (!rootEl) return [];
    const res = [];
    for (const b of rootEl.querySelectorAll("button.t")) {
      const underlined = getComputedStyle(b).textDecorationLine.includes("underline");
      const lines = [];
      const walker = document.createTreeWalker(b, NodeFilter.SHOW_TEXT);
      for (let n = walker.nextNode(); n; n = walker.nextNode()) {
        if (n.parentElement.closest(".t-emoji, svg")) continue;
        const s = n.data;
        for (let k = 0; k < s.length; k++) {
          if (/\s/.test(s[k])) continue;
          const r = document.createRange();
          r.setStart(n, k); r.setEnd(n, k + 1);
          const rc = r.getBoundingClientRect();
          if (!rc.width) continue;
          let line = lines.find((l) => Math.abs(l.bottom - rc.bottom) < rc.height / 2);
          if (!line) lines.push((line = { left: rc.left, right: rc.right, top: rc.top, bottom: rc.bottom }));
          line.left = Math.min(line.left, rc.left); line.right = Math.max(line.right, rc.right);
          line.top = Math.min(line.top, rc.top); line.bottom = Math.max(line.bottom, rc.bottom);
        }
      }
      // The box the term itself paints in on each line (its own fragment), so
      // a neighbour's underline one space away doesn't count as this one's.
      const frag = [...b.getClientRects()].filter((r) => r.width > 0 && r.height > 0);
      for (const l of lines) {
        const f = frag.filter((r) => r.bottom >= l.top + 1 && r.top <= l.bottom - 1);
        l.fl = Math.min(l.left, ...f.map((r) => r.left)); l.fr = Math.max(l.right, ...f.map((r) => r.right));
      }
      if (lines.length) res.push({ id: b.dataset.term ?? b.dataset.unitRef ?? "?", text: b.textContent.replace(/\s+/g, " ").trim(), underlined, lines });
    }
    return res;
  }, root);
}

/** For each term line, the x-extent (css px) of what is painted in a band just
 * under its text, or null when nothing is. */
async function painted(page, list) {
  const frame = () => page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
  const xs = list.flatMap((t) => t.lines.flatMap((l) => [l.left, l.right]));
  const ys = list.flatMap((t) => t.lines.flatMap((l) => [l.top, l.bottom]));
  const vp = page.viewportSize();
  const clip = { x: Math.max(0, Math.min(...xs) - 40), y: Math.max(0, Math.min(...ys) - 10) };
  clip.width = Math.min(vp.width, Math.max(...xs) + 40) - clip.x;
  clip.height = Math.min(vp.height, Math.max(...ys) + 20) - clip.y;
  const paint = await page.addStyleTag({ content: PAINT });
  await frame();
  const a = await page.screenshot({ type: "png", clip });
  const hide = await page.addStyleTag({ content: HIDE });
  await frame();
  const b = await page.screenshot({ type: "png", clip });
  await hide.evaluate((s) => s.remove());
  await paint.evaluate((s) => s.remove());
  return page.evaluate(async ({ a, b, list, clip, DPR }) => {
    const load = async (b64) => {
      const img = new Image();
      img.src = `data:image/png;base64,${b64}`;
      await img.decode();
      const c = document.createElement("canvas");
      c.width = img.width; c.height = img.height;
      const ctx = c.getContext("2d");
      ctx.drawImage(img, 0, 0);
      return { w: c.width, h: c.height, d: ctx.getImageData(0, 0, c.width, c.height).data };
    };
    const [A, B] = [await load(a), await load(b)];
    return list.map((t) => t.lines.map((l) => {
      // The band from the text's foot to a few px under it: the dotted line
      // (offset 3px) and anything else that reads as a line under the word.
      const x0 = Math.max(0, Math.floor((l.fl - 1 - clip.x) * DPR)), x1 = Math.min(A.w, Math.ceil((l.fr + 1 - clip.x) * DPR));
      const y0 = Math.max(0, Math.floor((l.bottom - 2 - clip.y) * DPR)), y1 = Math.min(A.h, Math.ceil((l.bottom + 8 - clip.y) * DPR));
      let lo = Infinity, hi = -Infinity;
      for (let y = y0; y < y1; y++) for (let x = x0; x < x1; x++) {
        const i = (y * A.w + x) * 4;
        if (Math.abs(A.d[i] - B.d[i]) + Math.abs(A.d[i + 1] - B.d[i + 1]) + Math.abs(A.d[i + 2] - B.d[i + 2]) > 40) { lo = Math.min(lo, x); hi = Math.max(hi, x); }
      }
      return lo === Infinity ? null : { left: clip.x + lo / DPR, right: clip.x + (hi + 1) / DPR };
    }));
  }, { a: a.toString("base64"), b: b.toString("base64"), list, clip, DPR });
}

/** The terms in `root` whose underline doesn't match their text; one shot of the first. */
async function check(page, root, where, label, bad, shots) {
  const list = await terms(page, root);
  if (!list.length) return 0;
  const lines = await painted(page, list);
  list.forEach((t, i) => t.lines.forEach((l, j) => {
    const u = lines[i][j];
    const why = !t.underlined ? (u ? `a ${Math.round(u.right - u.left)}px line under a term with no underline` : null)
      : !u ? "no underline on this line"
      : u.left < l.left - TOL ? `starts ${Math.round(l.left - u.left)}px before the word`
      : u.right > l.right + TOL ? `runs ${Math.round(u.right - l.right)}px past the word`
      : u.left > l.left + TOL ? `starts ${Math.round(u.left - l.left)}px into the word`
      : u.right < l.right - TOL ? `stops ${Math.round(l.right - u.right)}px short of the word`
      : null;
    if (!why) return;
    const key = `${label} ${t.id} "${t.text}"${t.lines.length > 1 ? ` line ${j + 1}/${t.lines.length}` : ""}: ${why}`;
    if (!bad.has(key)) bad.set(key, where);
  }));
  const mine = [...bad.keys()].filter((k) => k.startsWith(`${label} `)).length;
  if (mine > shots.n && shots.k < 12) { shots.n = mine; await page.screenshot({ path: `${out}/${label}-${String(shots.k++).padStart(2, "0")}.png` }); }
  return list.length;
}

/** A phone's sheet closes with Esc; a desktop's panel stays. */
async function closeSheet(page) {
  if (!(await page.getByTestId("overlay").count())) return;
  await page.keyboard.press("Escape");
  await page.getByTestId("overlay").waitFor({ state: "detached", timeout: 3000 }).catch(() => {});
}

const T0 = Date.now();
const say = (m) => process.env.DEBUG && console.log(`${((Date.now() - T0) / 1000).toFixed(1)}s ${m}`);
const browser = await launchChromium();
const bad = new Map();
let seen = 0;
try {
  for (const lang of ["en", "ru"]) for (const viewport of [{ width: 360, height: 640 }, { width: 1280, height: 800 }]) {
    const label = `${lang}-${viewport.width}`;
    const phone = viewport.width < 700;
    const page = await browser.newPage({ viewport, deviceScaleFactor: DPR, locale: localeOf(lang), isMobile: phone, hasTouch: phone });
    page.setDefaultTimeout(15_000);
    const shots = { n: 0, k: 0 };
    await page.goto(url);
    await page.getByTestId("name-input").fill(`Line${lang}${viewport.width}${Date.now() % 10000}`);
    await page.getByTestId("name-submit").click();
    // The shop's unit panel: each offer.
    await page.getByTestId("play").click();
    await page.getByTestId("offer-0").waitFor();
    for (let k = 0; k < 5; k++) {
      const offer = page.getByTestId(`offer-${k}`);
      if (!(await offer.count())) break;
      await offer.click();
      const form = page.getByTestId("sheet-form");
      await form.waitFor();
      say(`${label} offer ${k}`);
      seen += await check(page, "[data-testid=sheet-form]", `shop offer ${k}`, label, bad, shots);
      await closeSheet(page);
    }
    // Each Codex unit: its sheet, sleeping and awoken.
    await page.keyboard.press("Escape").catch(() => {});
    await page.goto(url);
    await page.getByTestId("codex").click();
    const units = page.getByTestId("codex-unit");
    await units.first().waitFor();
    const n = Math.min(limit, await units.count());
    for (let k = 0; k < n; k++) {
      await units.nth(k).scrollIntoViewIfNeeded();
      await units.nth(k).click({ position: { x: 20, y: 60 } });
      const form = page.getByTestId("sheet-form").last();
      await form.waitFor();
      say(`${label} codex ${k}`);
      const name = await units.nth(k).evaluate((c) => c.querySelector(".name")?.getAttribute("title") ?? "");
      seen += await check(page, "[data-testid=sheet-form]", `codex ${name || k}`, label, bad, shots);
      const awoken = form.getByTestId("see-awoken");
      if (await awoken.count()) {
        await awoken.click();
        seen += await check(page, "[data-testid=sheet-form]", `codex ${name || k} awoken`, label, bad, shots);
      }
      await closeSheet(page);
    }
    console.log(`${label}: ${[...bad.keys()].filter((k) => k.startsWith(label)).length} mismatches`);
    await page.close();
  }
  await browser.close();
} finally {
  child?.kill();
}
for (const [k, where] of bad) console.log(`  ${k}  (${where})`);
console.log(`${seen} underlined terms checked, ${bad.size} mismatches`);
process.exit(bad.size || !seen ? 1 : 0);
