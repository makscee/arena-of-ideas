// The first screen as a composition (arena#31, round 4, mission #26 move 3): at
// 1920×1080, 1440×900, 1280×720, 1024×768, 800×450 and 390×844 it fills the
// stage like the duel's mockup: the title large top left, the EN/РУ toggle top
// right, the chain demo as two rows (today's champion's line above, a player's
// line below) at the table's piece size and spread across the width, one step's
// wire and caption at a time, the headline, PLAY with its note, and the quiet
// name field under PLAY, all on screen with no scroll and no text under 11 px
// as drawn. Measures boxes. Starts its own server (rules v2's pool).
//   npm run mvp:start-screen -- [--url http://127.0.0.1:8841/arena/] [--out e2e/.shots/start-screen]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/start-screen";
mkdirSync(out, { recursive: true });
let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", ARENA_CODEX: "1", MVP_POOL: "v2" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
  for (let i = 0; i < 300; i++) {
    try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
    await new Promise((r) => setTimeout(r, 200));
  }
}

const SIZES = [["1920x1080", 1920, 1080], ["1440x900", 1440, 900], ["1280x720", 1280, 720], ["1024x768", 1024, 768], ["800x450", 800, 450], ["390x844", 390, 844]];
const errors = [];
const browser = await launchChromium();
const day = await (await fetch(url + "api/v1/day")).json();
const champ = (day.champion?.line ?? []).map((u) => u.name);

/** What the page shows now, in the stage's own pixels where it can. */
const measure = () => {
  const $ = (id) => document.querySelector(`[data-testid="${id}"]`);
  const stage = $("stage");
  const sr = stage?.getBoundingClientRect();
  const k = parseFloat(stage?.style.getPropertyValue("--k") || "1");
  const box = (e) => { const r = e?.getBoundingClientRect(); return r ? { l: r.left, t: r.top, r: r.right, b: r.bottom, w: r.width, h: r.height } : null; };
  const pieces = (p) => [0, 1, 2, 3].map((i) => $(`${p}-${i}`)).map((e) => ({ name: e?.getAttribute("data-name") ?? null, ...(box(e) ?? {}) }));
  const drawn = (e) => {
    const f = parseFloat(getComputedStyle(e).fontSize);
    let b = e;
    while (b && b !== document.body && (getComputedStyle(b).display === "inline" || b.offsetWidth < 24 || b.offsetHeight < 24)) b = b.parentElement;
    const r = b?.getBoundingClientRect();
    const s = b && b.offsetWidth > 0 && r.width > 0 ? r.width / b.offsetWidth : 1;
    return f * (s > 0 ? s : 1);
  };
  const small = [...document.querySelectorAll("#app *")].filter((e) => e.getClientRects().length && getComputedStyle(e).visibility !== "hidden" && /^(SPAN|DIV|BUTTON|P|LABEL|H1|H2|H3|A|SMALL|B|I|INPUT)$/.test(e.tagName) && !e.closest("[aria-hidden='true']") && (e.tagName === "INPUT" || [...e.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim()))).map((e) => ({ t: (e.textContent || e.placeholder || "").trim().slice(0, 20), px: drawn(e) })).filter((x) => x.px < 10.9);
  const d = document.scrollingElement ?? document.documentElement;
  const all = [...stage.querySelectorAll("[data-testid]")].filter((e) => e.getClientRects().length).map((e) => e.getBoundingClientRect().bottom);
  return {
    orient: stage?.getAttribute("data-orient"), k, stage: box(stage), W: innerWidth, H: innerHeight,
    scroll: d.scrollHeight - d.clientHeight, hscroll: d.scrollWidth - d.clientWidth,
    mine: pieces("start-piece"), theirs: pieces("start-their"),
    lowest: Math.max(...all),
    play: box($("start-play")), name: box($("name-input")), nameGo: box($("name-submit")),
    title: box($("start-brand")), titleSize: $("start-brand") ? parseFloat(getComputedStyle($("start-brand")).fontSize) : 0,
    headline: $("start-title")?.textContent, headlineBox: box($("start-title")),
    toggle: box($("start-lang")), en: !!$("lang-en"), ru: !!$("lang-ru"),
    labels: [...document.querySelectorAll('[data-testid="start-side"]')].map((e) => e.textContent.trim()),
    step: $("start-chain")?.getAttribute("data-step"), caption: $("start-caption")?.textContent ?? "",
    discs: [...document.querySelectorAll('[data-testid="start-wire-n"]')].filter((e) => e.getAttribute("data-on") === "1").map((e) => e.textContent),
    wiresOn: [...document.querySelectorAll('[data-testid="start-wire"]')].filter((e) => e.getAttribute("data-on") === "1").length,
    small,
  };
};

try {
  for (const [label, W, H] of SIZES) {
    const ctx = await browser.newContext({ viewport: { width: W, height: H } });
    const page = await ctx.newPage();
    const fail = (m) => errors.push(`${label}: ${m}`);
    page.on("pageerror", (e) => fail(`pageerror: ${e.message}`));
    await page.goto(url, { timeout: 20_000 });
    await page.getByTestId("start").waitFor({ timeout: 15_000 });
    await page.waitForTimeout(400);
    await page.screenshot({ path: `${out}/start-${label}.png` });
    const m = await page.evaluate(measure);
    const L = m.orient === "L";
    // 1. A composition that fills the stage.
    if (m.scroll > 0 || m.hscroll > 0) fail(`the first screen scrolls (${m.scroll}px down, ${m.hscroll}px across)`);
    const piecePx = m.mine[0].w / m.k;
    if (piecePx < (L ? 70 : 56) - 0.5) fail(`the demo's pieces are ${piecePx.toFixed(0)} px on the stage, under the table's ${L ? 70 : 56}`);
    if (m.theirs.some((p) => !p.w) || m.mine.some((p) => !p.w)) fail("the demo does not have 4 + 4 pieces");
    else {
      const cx = (p) => (p.l + p.r) / 2;
      const spread = (Math.max(...m.mine.map(cx), ...m.theirs.map(cx)) - Math.min(...m.mine.map(cx), ...m.theirs.map(cx))) / m.stage.w;
      if (spread < (L ? 0.45 : 0.7)) fail(`the rows span ${(spread * 100).toFixed(0)}% of the stage's width, not ${L ? 45 : 70}%`);
      if (!(Math.max(...m.theirs.map((p) => p.b)) < Math.min(...m.mine.map((p) => p.t) ) + 1)) fail("their line is not above a player's line");
      if (champ.length >= 4 && m.theirs.map((p) => p.name).join() !== champ.slice(0, 4).join()) fail(`the top row is ${m.theirs.map((p) => p.name)}, not today's champion's ${champ.slice(0, 4)}`);
    }
    const gap = (m.stage.b - m.lowest) / m.stage.h;
    if (gap > 0.12) fail(`the lowest element ends ${(gap * 100).toFixed(0)}% above the stage's bottom (limit 12%)`);
    if (!m.play || m.play.t < 0 || m.play.b > m.H + 0.5 || m.play.l < 0 || m.play.r > m.W + 0.5) fail(`PLAY is not on screen: ${JSON.stringify(m.play)}`);
    if (m.small.length) fail(`text under 11 px as drawn: ${m.small.map((s) => `${s.t} (${s.px.toFixed(1)})`).join(", ")}`);
    // 4. The name field stays under PLAY, on screen.
    if (!m.name || m.name.t < m.play.b - 1 || m.name.b > m.H + 0.5) fail(`the name field is not under PLAY on screen: ${JSON.stringify(m.name)}`);
    // The title and the toggle.
    if (!m.title || m.titleSize < (L ? 40 : 22)) fail(`the title is ${m.titleSize}px`);
    if (L && m.title && m.title.l > m.stage.l + m.stage.w * 0.15) fail("the title is not at the top left");
    if (!m.en || !m.ru || !m.toggle || m.toggle.r < m.stage.r - m.stage.w * 0.2) fail("the EN/РУ toggle is not at the top right");
    if (L && (m.labels.length !== 2)) fail(`side labels: ${JSON.stringify(m.labels)}`);

    // 2. The demo: three steps in order, one wire and caption at a time, looping.
    const seen = [];
    for (let i = 0; i < 36; i++) {
      const s = await page.evaluate(measure);
      if (seen.at(-1) !== s.step) seen.push(s.step);
      if (s.discs.length !== 1 || s.discs[0] !== s.step) fail(`step ${s.step}: wire numbers on screen ${JSON.stringify(s.discs)}`);
      if (!s.caption.startsWith(`${s.step} ·`)) fail(`step ${s.step}: caption "${s.caption}"`);
      if (s.wiresOn < 1 || s.wiresOn > 2) fail(`step ${s.step}: ${s.wiresOn} wires on`);
      await page.waitForTimeout(200);
    }
    const order = seen.join("");
    if (!/123123|231231|312312/.test(order.length >= 6 ? order : "") && !/^(1231|2312|3123)/.test(order)) fail(`the steps run ${order}, not 1 → 2 → 3 → 1`);
    // 5. PLAY and its Starting… state are mvp:play-feedback's.
    await ctx.close();
  }

  // Reduced motion: the last step, still.
  {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 }, reducedMotion: "reduce" });
    const page = await ctx.newPage();
    await page.goto(url, { timeout: 20_000 });
    await page.getByTestId("start").waitFor({ timeout: 15_000 });
    const a = await page.evaluate(measure);
    await page.waitForTimeout(2200);
    const b = await page.evaluate(measure);
    if (a.step !== "3" || b.step !== "3") fail_("reduced motion", `shows step ${a.step} then ${b.step}, not 3 and still`);
    await ctx.close();
  }

  // 3. The EN/РУ toggle works both ways.
  {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const page = await ctx.newPage();
    await page.goto(url, { timeout: 20_000 });
    await page.getByTestId("start").waitFor({ timeout: 15_000 });
    await page.getByTestId("lang-ru").click();
    await page.waitForFunction(() => document.documentElement.lang === "ru" && document.querySelector('[data-testid="start-title"]'), null, { timeout: 15_000 }).catch(() => fail_("toggle", "РУ does not switch the page"));
    const ru = await page.getByTestId("start-title").textContent().catch(() => "");
    if (ru !== "Одно правило запускает цепочку.") fail_("toggle", `Russian headline is ${JSON.stringify(ru)}`);
    await page.getByTestId("lang-en").click();
    await page.waitForFunction(() => document.documentElement.lang === "en" && document.querySelector('[data-testid="start-title"]'), null, { timeout: 15_000 }).catch(() => fail_("toggle", "EN does not switch back"));
    if ((await page.getByTestId("start-title").textContent().catch(() => "")) !== "One rule starts a chain.") fail_("toggle", "English headline did not come back");
    await ctx.close();
  }
} catch (e) {
  errors.push(`${e instanceof Error ? e.stack : e}`);
} finally {
  await browser.close();
  child?.kill();
}
function fail_(where, m) { errors.push(`${where}: ${m}`); }
if (errors.length) {
  console.error(errors.map((e) => `FAIL ${e}`).join("\n"));
  process.exit(1);
}
console.log("mvp start-screen: ok");
