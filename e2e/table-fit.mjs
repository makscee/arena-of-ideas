// The table at CrazyGames' 800×450 frame (arena#32, mission #26): every state
// opens with no page scroll, no text under 11 px as drawn (the stage is scaled,
// so a font size is read through the element's own transform), and the next
// opponent's team is nowhere before the fight: their line is wells, and no unit
// of theirs is named in the shop (the Crown shows the champion's team, as before).
// A battle is checked playing, paused, and with its result in the tray.
// Without --url it builds the client and starts its own scenes server.
//   npm run mvp:table-fit -- [--url http://127.0.0.1:PORT/arena/] [--out e2e/.shots/table-fit] [--only shop-r3,gift] [--size 800x450] [--cg]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/table-fit";
const only = opt("only")?.split(",");
const cg = args.includes("--cg");
const [W, H] = (opt("size") ?? "800x450").split("x").map(Number);
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit", env: { ...process.env, ...(cg ? { VITE_CG: "1" } : {}) } });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_TUNER: "instant", ARENA_CODEX: "1", ARENA_CG: "1" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

const scenes = (await (await fetch(url + "api/v1/dev/scenes")).json()).filter((s) => (!only || only.includes(s.id)) && s.desktop !== false);

/** What a screen looks like now: scroll, text under 11 px as drawn, and what of the next opponent shows. */
function measure() {
  const d = document.scrollingElement ?? document.documentElement;
  const drawn = (e) => {
    const f = parseFloat(getComputedStyle(e).fontSize);
    // The scale the element is drawn at: a block ancestor's rect over its layout box (a transformed stage).
    // An inline element's offsetWidth is rounded, so it reads the scale off a wide enough box above it.
    let b = e;
    while (b && b !== document.body && (getComputedStyle(b).display === "inline" || b.offsetWidth < 24 || b.offsetHeight < 24)) b = b.parentElement;
    const r = b?.getBoundingClientRect();
    const k = b && b.offsetWidth > 0 && r.width > 0 ? r.width / b.offsetWidth : 1;
    return f * (k > 0 ? k : 1);
  };
  const small = [...document.querySelectorAll("#app *, body > *")].filter((e) => {
    if (!e.getClientRects().length || getComputedStyle(e).visibility === "hidden") return false;
    if (!/^(SPAN|DIV|BUTTON|P|LABEL|H1|H2|H3|H4|A|LI|TD|TH|SMALL|B|I|EM|STRONG|KBD|U|SUMMARY)$/.test(e.tagName)) return false;
    if (e.closest("[aria-hidden='true']") && !e.closest("button")) return false;
    return [...e.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim());
  }).map((e) => ({ t: e.textContent.trim().slice(0, 22), px: drawn(e), cls: ((e.className?.toString?.() ?? "") + " < " + (e.parentElement?.className?.toString?.() ?? "") + " < " + (e.parentElement?.parentElement?.className?.toString?.() ?? "")).slice(0, 90) })).filter((x) => x.px > 0 && x.px < 10.9);
  const their = document.querySelector('[data-testid="their-line"]');
  const crown = !!document.querySelector('[data-testid="crown-foe"]');
  const theirUnits = their && !crown ? [...their.querySelectorAll("[data-name], .pc")].length : 0;
  const theirText = their && !crown ? (their.textContent ?? "").replace(/[?\s]/g, "") : "";
  const title = crown ? [] : [...document.querySelectorAll(".tb-their [title]")].map((e) => e.getAttribute("title"));
  const label = crown ? [] : [...document.querySelectorAll(".tb-their [aria-label]:not([aria-hidden='true'])")].map((e) => e.getAttribute("aria-label"));
  return {
    sh: d.scrollHeight, sw: d.scrollWidth, H: innerHeight, W: innerWidth,
    small: [...new Map(small.map((x) => [x.t + x.px.toFixed(1), x])).values()].slice(0, 6),
    theirUnits, theirText, title, label,
    hasTheirLine: !!their,
  };
}

const failures = [];
const browser = await launchChromium();
try {
  for (const s of scenes) {
    const ctx = await browser.newContext({ viewport: { width: W, height: H }, isMobile: W < 900, hasTouch: W < 900, deviceScaleFactor: 1 });
    const page = await ctx.newPage();
    const why = [];
    page.on("pageerror", (e) => why.push(`pageerror: ${e.message}`));
    const check = async (what) => {
      const m = await page.evaluate(measure);
      const w = [];
      if (m.sh > m.H + 1) w.push(`scrolls ${m.sh - m.H}px down`);
      if (m.sw > m.W + 1) w.push(`scrolls ${m.sw - m.W}px sideways`);
      if (m.small.length) w.push(`text under 11px: ${m.small.map((x) => `"${x.t}" ${x.px.toFixed(2)} [${x.cls}]`).join(", ")}`);
      if (m.theirUnits) w.push(`the next opponent's line shows ${m.theirUnits} pieces`);
      if (m.theirText) w.push(`text in their line: ${m.theirText.slice(0, 30)}`);
      if (m.title.length) w.push(`a tooltip on their side: ${m.title.join("|")}`);
      for (const x of w) why.push(`${what}: ${x}`);
      return m;
    };
    await page.goto(`${url}#scene=${s.id}&lang=en`, { timeout: 120_000 }).catch((e) => why.push(`load: ${e.message.split("\n")[0]}`));
    await page.waitForFunction(() => (document.querySelector("#app")?.textContent ?? "").trim().length > 20, null, { timeout: 60_000 }).catch(() => why.push("blank"));
    await page.waitForTimeout(1500);
    await check("shown");
    await page.screenshot({ path: `${out}/${s.id}-${W}x${H}.png` });
    if (s.group === "Battle" || s.group === "Crown & day" || /^v2-/.test(s.id)) {
      if (await page.locator('[data-testid="battle-play"]').count()) {
        await page.keyboard.press("Space");
        await page.waitForTimeout(300);
        for (let i = 0; i < 6; i++) { await page.keyboard.press("ArrowRight"); await page.waitForTimeout(120); }
        await check("paused mid-battle");
        await page.screenshot({ path: `${out}/${s.id}-paused-${W}x${H}.png` });
        await page.keyboard.press("e");
        await page.waitForTimeout(900);
        await check("result");
        await page.screenshot({ path: `${out}/${s.id}-result-${W}x${H}.png` });
      }
    }
    console.log(`${why.length ? "FAIL" : "ok  "} ${s.id} ${W}x${H}${why.length ? ": " + why.join("; ") : ""}`);
    if (why.length) failures.push(`${s.id}: ${why.join("; ")}`);
    await ctx.close();
  }
} finally {
  await browser.close();
  child?.kill();
}
if (failures.length) { console.error(`\n${failures.length} of ${scenes.length} states failed`); process.exit(1); }
console.log(`\n${scenes.length} states fit ${W}x${H}`);
