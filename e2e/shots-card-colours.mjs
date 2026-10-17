// Toybox R2 (makscee/void-board#914): the unit card in all 3 palettes. Plays
// one round on a LOCAL server per palette and viewport (360 and 1280) and
// shoots the shop, the battle a few beats in, a card's detail panel and the
// Codex; then measures the text on every card face (contrast ≥ 4.5:1).
// Needs `npm run mvp:build` first.
//   node e2e/shots-card-colours.mjs [outdir] [--lang ru]
// A loaded machine can take minutes to start the server: it waits up to 3.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { launchChromium } from "./browser.mjs";
import { localeOf } from "./lang.mjs";
const args = process.argv.slice(2);
const lang = args.includes("--lang") ? args[args.indexOf("--lang") + 1] : "en";
const out = args.find((a, i) => !a.startsWith("--") && args[i - 1] !== "--lang") ?? "e2e/.shots/card-colours";
mkdirSync(out, { recursive: true });
const port = 18000 + Math.floor(Math.random() * 1000);
// ARENA_URL=http://127.0.0.1:PORT/arena/ reuses a running server instead.
const child = process.env.ARENA_URL ? null : spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", STATIC_DIR: "mobile/dist" }, stdio: ["ignore", "ignore", "inherit"] });
const url = process.env.ARENA_URL ?? `http://127.0.0.1:${port}/arena/`;
const low = [];
// Every text on a card face against the face behind it (WCAG contrast).
const contrast = (page) => page.evaluate(() => {
  // rgb(a)(…) is 0–255; a color-mix() computes to color(srgb …) in 0–1.
  const rgb = (s) => { const k = s.startsWith("color(") ? 255 : 1; const m = s.replace(/^color\(srgb/, "").match(/[\d.]+/g)?.map(Number) ?? [0, 0, 0, 0]; return { r: m[0] * k, g: m[1] * k, b: m[2] * k, a: m[3] ?? 1 }; };
  const lum = ({ r, g, b }) => { const f = (c) => { c /= 255; return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4; }; return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b); };
  const ratio = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05); };
  const bad = new Set();
  for (const card of document.querySelectorAll(".card:not(.empty)")) {
    if (!card.offsetParent || card.closest(".dead, .muted, .poor")) continue;
    const bg = rgb(getComputedStyle(card).backgroundColor);
    if (bg.a < 1) continue;
    for (const el of card.querySelectorAll(".name, .stats span, .stats, .tier, .cost, .copies, .discovered, .ver")) {
      if (!el.offsetParent || !el.textContent.trim()) continue;
      const own = rgb(getComputedStyle(el).backgroundColor);
      const r = ratio(rgb(getComputedStyle(el).color), own.a === 1 ? own : bg);
      if (r < 4.5) bad.add(`${el.className || el.tagName} "${el.textContent.trim().slice(0, 12)}" ${r.toFixed(2)}`);
    }
  }
  return [...bad];
});
try {
  for (let i = 0; i < 360; i++) { try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {} await new Promise((r) => setTimeout(r, 500)); }
  const browser = await launchChromium();
  for (const theme of ["plum", "brass", "felt"]) for (const [label, viewport] of [["360", { width: 360, height: 640 }], ["1280", { width: 1280, height: 800 }]]) {
    const phone = viewport.width < 1024;
    const page = await browser.newPage({ viewport, deviceScaleFactor: 2, locale: localeOf(lang), ...(phone ? { isMobile: true, hasTouch: true } : {}) });
    page.setDefaultTimeout(120_000);
    page.setDefaultNavigationTimeout(180_000);
    const shot = async (what) => {
      await page.waitForTimeout(300);
      await page.screenshot({ path: `${out}/${theme}-${label}-${what}.png` });
      for (const b of await contrast(page)) low.push(`${theme} ${label} ${what}: ${b}`);
    };
    await page.goto(`${url}?theme=${theme}`);
    await page.getByTestId("name-input").fill(`Col${theme}${label}`);
    await page.getByTestId("name-submit").click();
    await page.getByTestId("play").click();
    await page.getByTestId("offer-0").click(); await page.getByTestId("offer-0").click();
    await page.getByTestId("buy").click();
    await page.getByTestId("offer-0").click().catch(() => {});
    await page.getByTestId("buy").click().catch(() => {});
    await page.getByTestId("offer-1").click().catch(() => {});
    await shot("shop");
    await page.getByTestId("fight").click();
    await page.getByTestId("caption").waitFor();
    await page.keyboard.press(" ");
    if (!phone) await page.keyboard.press(" ").catch(() => {});
    for (let i = 0; i < 4; i++) await page.getByTestId("battle-step").click().catch(() => page.keyboard.press("ArrowRight"));
    for (let i = 0; i < 6 && !(await page.locator(".bv-line .bv-card.acting").count()); i++) await page.getByTestId("battle-step").click();
    await page.waitForTimeout(600);
    await shot("battle");
    // The Codex and a card's detail panel, from a fresh load (the run stays saved).
    await page.goto(`${url}?theme=${theme}`);
    await page.getByTestId("codex").click();
    await page.getByTestId("codex-units").waitFor();
    await shot("codex");
    await page.getByTestId("codex-unit").nth(2).click();
    await page.getByTestId("unit-sheet").waitFor({ timeout: 3000 }).catch(() => {});
    await shot("detail");
    await page.close();
  }
  await browser.close();
} finally {
  child?.kill();
}
console.log(low.length ? `low contrast on the card face (${low.length}):\n${low.join("\n")}` : "card faces: every text ≥ 4.5:1");
process.exit(low.length ? 1 : 0);
