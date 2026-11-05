// The other screens' sigils (arena#39), in a real browser: cards, sheets, the Codex,
// the forge, ideas, vote, the champion pin and the battle's lists draw a unit's sigil
// where they used to print its emoji. For each scene: sigil canvases are on the page,
// none of the live units' emoji is left in its visible text, and there is no page error.
//   npm run mvp:sigil-screens -- [--url http://127.0.0.1:PORT/arena/] [--out e2e/.shots/sigil-screens]
// Without --url it builds the client and starts its own scenes server.
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/sigil-screens";
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_TUNER: "instant", ARENA_CODEX: "1", ARENA_IDEA_READER: "fake", MVP_POOL: "v2" }, stdio: ["ignore", "ignore", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

const failures = [];
const check = (ok, what) => { console.log(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failures.push(what); };

// The emoji a live unit or summon wears. A scene's text must not print any of them
// outside the parts that are not a unit's portrait (the when tab, headline badge, status chips: ui icons).
const content = await (await fetch(url + "api/v1/content")).json();
const EMOJI = [...new Set([...content.units, ...(content.summons ?? []), ...(content.left ?? [])].map((u) => u.emoji).filter(Boolean))];
const NOT_PORTRAIT = ".pc-tab, .pc-hb, .pc-mark, .pc-st, .pc-flag, .pc-lock, .sheet-now-chip, .icon-line, .iconline, .by-mark, .pc-num, .tier, .stat-row .emoji:not(.pair), [data-k=head], [data-k=ring], svg";

// [scene, view, canvases it must show at least, a step to take first]
const SCENES = [
  ["codex-units", "phone", 4], ["codex-fusions", "phone", 2, async (page) => { await page.selectOption('[data-testid="codex-fusions-of"]', { index: 3 }); await page.waitForTimeout(600); }], ["codex-library", "phone", 0], ["codex-keywords", "phone", 1], ["codex-champions", "phone", 1], ["codex-unit", "phone", 1],
  ["forge-unforged", "phone", 2], ["forge-ready", "phone", 2], ["forge-discovered", "phone", 2], ["claim", "phone", 3], ["claim-done", "phone", 2],
  ["vote", "phone", 2], ["ideas", "phone", 0], ["idea-archetype", "phone", 3], ["idea-implementation", "phone", 3],
  ["gift", "phone", 3], ["shop-unit", "wide", 2], ["fusion-now", "phone", 2], ["battle-ledger", "wide", 2], ["battle-result", "phone", 2],
];
const VIEWS = { phone: { width: 390, height: 844, mobile: true }, wide: { width: 800, height: 450, mobile: false } };

const browser = await launchChromium();
try {
  for (const [id, view, min, step] of SCENES) {
    const v = VIEWS[view];
    const ctx = await browser.newContext({ viewport: { width: v.width, height: v.height }, deviceScaleFactor: 2, isMobile: v.mobile, hasTouch: v.mobile });
    const page = await ctx.newPage();
    const errs = [];
    page.on("pageerror", (e) => errs.push(e.message));
    await page.goto(`${url}#scene=${id}&lang=en`, { timeout: 60_000 });
    await page.waitForFunction(() => document.querySelector("#app")?.textContent.trim().length > 20, null, { timeout: 60_000 });
    await page.waitForTimeout(1200);
    if (step) await step(page);
    const r = await page.evaluate(({ EMOJI, NOT_PORTRAIT }) => {
      const app = document.querySelector("#app").cloneNode(true);
      app.querySelectorAll(NOT_PORTRAIT).forEach((n) => n.remove());
      const text = app.textContent;
      const titles = [...document.querySelectorAll("#app [title],#app [aria-label]")].map((n) => (n.getAttribute("title") ?? "") + (n.getAttribute("aria-label") ?? "")).join(" ");
      return {
        left: EMOJI.filter((e) => text.includes(e)),
        canvases: document.querySelectorAll("#app canvas.sg-ic, #app canvas.pc-sg").length,
        inked: [...document.querySelectorAll("#app canvas.sg-ic")].filter((c) => { const d = c.getContext("2d")?.getImageData(0, 0, c.width, c.height).data; if (!d) return false; for (let i = 3; i < d.length; i += 4) if (d[i] > 0) return true; return false; }).length,
        icons: document.querySelectorAll("#app canvas.sg-ic").length,
        tiny: [...document.querySelectorAll("#app canvas.sg-ic")].filter((c) => c.getBoundingClientRect().width < 13 && c.getBoundingClientRect().width > 0).length,
        hidden: [...document.querySelectorAll("#app canvas.sg-ic, #app canvas.pc-sg")].filter((c) => !c.getBoundingClientRect().width).length,
        wide: document.documentElement.scrollWidth > window.innerWidth + 1,
        titleEmoji: EMOJI.filter((e) => titles.includes(e)),
      };
    }, { EMOJI, NOT_PORTRAIT });
    const name = `${id} ${view}`;
    check(r.left.length === 0, `${name}: no unit emoji in the text${r.left.length ? " (" + r.left.join(" ") + ")" : ""}`);
    check(r.canvases >= min, `${name}: ${r.canvases} sigils drawn (at least ${min})`);
    check(r.icons === r.inked, `${name}: every small sigil has drawn pixels (${r.inked}/${r.icons})`);
    check(r.tiny === 0, `${name}: no sigil under 13 px (22 px drawn at 800×450 is 13.75)`);
    check(!r.wide, `${name}: no sideways scroll`);
    check(errs.length === 0, `${name}: no page error${errs.length ? " (" + errs[0] + ")" : ""}`);
    await page.screenshot({ path: `${out}/${id}-${view}.png` });
    await ctx.close();
  }
} finally {
  await browser.close();
  child?.kill();
}
if (failures.length) { console.error(`\n${failures.length} check(s) failed`); process.exit(1); }
console.log("\nsigil screens ok");
