// The shop's fusion marks under rules v2 (#1049 reopened, mission #1043):
// a fuse needs two Awoken units whose pair has a design, and the shop says
// only that. Opens the scenes fusion-none, fusion-now and fusion-shop on a
// phone (EN, RU) and a desktop, in every palette, and checks:
//   1. none: two Awoken of an unforged pair: no ⚗, no glow, no Fuse, no fuse hint;
//   2. now: two Awoken of a discovered pair: ⚗ on both, Fuse opens the forge;
//   3. shop: the 3rd copy that awakens into a ready pair wears its pill, one
//      that still needs awakening a quiet ⚗, one into an unforged pair nothing;
//   5. no splice preview or "Fusion preview" copy anywhere;
//   6. PWR/HP badges (#1124): in every badge of the line, bench, shop, forge
//      preview, claim sheet, Codex and unit sheet the number's ink has the same
//      gap above and below (within 1px, and left and right) at 1280 and 390, in
//      plum, brass and felt.
// Without --url it builds the client and starts its own scenes server.
//   npm run mvp:fusion-marks -- [--url http://127.0.0.1:PORT/arena/] [--out e2e/.shots/fusion-marks] [--only badges]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";
import { badgeGaps, badgeFailures } from "./badge-centre.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/fusion-marks";
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_TUNER: "instant", ARENA_CODEX: "1" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

const views = [
  { name: "phone-en", viewport: { width: 360, height: 640 }, isMobile: true, hasTouch: true, lang: "en", theme: "plum" },
  { name: "phone-ru", viewport: { width: 360, height: 640 }, isMobile: true, hasTouch: true, lang: "ru", theme: "brass" },
  { name: "phone-felt", viewport: { width: 360, height: 640 }, isMobile: true, hasTouch: true, lang: "en", theme: "felt" },
  { name: "desktop", viewport: { width: 1280, height: 800 }, isMobile: false, hasTouch: false, lang: "en", theme: "plum" },
  { name: "desktop-ru", viewport: { width: 1280, height: 800 }, isMobile: false, hasTouch: false, lang: "ru", theme: "felt" },
];
const failures = [];
const browser = await launchChromium();
try {
  for (const v of opt("only") === "badges" ? [] : views) {
    const desk = !v.isMobile;
    const open = async (scene) => {
      const ctx = await browser.newContext({ viewport: v.viewport, isMobile: v.isMobile, hasTouch: v.hasTouch, deviceScaleFactor: 1 });
      await ctx.addInitScript((theme) => localStorage.setItem("arena.theme", theme), v.theme);
      const page = await ctx.newPage();
      const errs = [];
      page.on("pageerror", (e) => errs.push(`pageerror: ${e.message}`));
      page.on("response", (r) => r.url().includes("/api/v1/") && r.status() >= 400 && errs.push(`HTTP ${r.status()} ${new URL(r.url()).pathname}`));
      await page.goto(`${url}#scene=${scene}&lang=${v.lang}`, { timeout: 120_000 });
      await page.getByTestId("line").waitFor({ timeout: 120_000 });
      // The marks come with the run's forge, a moment after the shop.
      await page.waitForResponse((r) => /\/forge$/.test(r.url()), { timeout: 30_000 }).catch(() => {});
      await page.waitForTimeout(400);
      return { ctx, page, errs };
    };
    const fail = (scene, why) => failures.push(`${scene} ${v.name}: ${why}`);
    const noSplice = async (scene, page) => {
      if ((await page.getByTestId("fusion-recipe").count()) > 0) fail(scene, "a splice recipe (When · Who · Does) shows");
      if (/Fusion preview|Предпросмотр слияния/.test(await page.locator("#app").textContent())) fail(scene, "'Fusion preview' copy shows");
    };
    const shot = (page, name) => page.screenshot({ path: `${out}/${name}-${v.name}.png` });
    const select = async (page, id) => page.getByTestId(id).click();

    // 1. Two Awoken, no design.
    {
      const s = "fusion-none";
      const { ctx, page, errs } = await open(s);
      await shot(page, s);
      if ((await page.locator('[data-testid^="fuse-mark-"], [data-testid^="fuse-offer-"]').count()) > 0) fail(s, "a fusion mark shows");
      if ((await page.locator(".card.fuse-ready, .card.fusable").count()) > 0) fail(s, "a fusion glow shows");
      if (/fuse:|слить:|F\.|«Слить»/.test((await page.locator(".hint-slot").textContent()) ?? "")) fail(s, "the fuse hint shows");
      await select(page, "line-0");
      if ((await page.getByTestId("fuse").count()) > 0) fail(s, "a Fuse button on an Awoken unit of an unforged pair");
      if (desk) {
        await page.keyboard.press("f");
        await page.waitForTimeout(200);
        if ((await page.locator(".card.fusable").count()) > 0) fail(s, "F starts a fuse");
      }
      await noSplice(s, page);
      errs.forEach((e) => fail(s, e));
      await ctx.close();
    }
    // 2. A fusion you can make now.
    {
      const s = "fusion-now";
      const { ctx, page, errs } = await open(s);
      await shot(page, s);
      for (const i of [0, 1]) if ((await page.getByTestId(`fuse-mark-${i}`).count()) !== 1) fail(s, `no ⚗ on line-${i}`);
      if ((await page.getByTestId("fuse-mark-2").count()) > 0) fail(s, "⚗ on the sleeping unit");
      await select(page, "line-0");
      if ((await page.getByTestId("fuse").count()) !== 1) fail(s, "no Fuse button");
      else {
        await page.getByTestId("fuse").click();
        if ((await page.getByTestId("line-2").evaluate((e) => e.classList.contains("fusable"))) === true) fail(s, "the sleeping unit glows as a partner");
        await page.getByTestId("line-1").click();
        const opened = await page.getByTestId("forge-discovered").waitFor({ timeout: 15_000 }).then(() => true, () => false);
        if (!opened) fail(s, "Fuse didn't open the forge as discovered");
        await shot(page, `${s}-forge`);
      }
      await noSplice(s, page);
      errs.forEach((e) => fail(s, e));
      await ctx.close();
    }
    // 3 (and 4). A fusion one buy away; a close one; none for an unforged pair.
    {
      const s = "fusion-shop";
      const { ctx, page, errs } = await open(s);
      await shot(page, s);
      const pill = page.getByTestId("fuse-offer-0");
      if ((await pill.count()) !== 1) fail(s, "no mark on the offer that awakens into a ready pair");
      else if (!/claim|забрать/.test(await pill.textContent())) fail(s, `the pill says '${await pill.textContent()}', not 'to claim'`);
      if ((await page.getByTestId("fuse-offer-1").count()) !== 1) fail(s, "no quiet mark on the offer that still needs awakening");
      if ((await page.getByTestId("fuse-offer-2").count()) > 0) fail(s, "a mark on an offer into an unforged pair");
      if ((await page.locator('[data-testid^="fuse-mark-"]').count()) > 0) fail(s, "⚗ on the board with one Awoken unit");
      // The pill stays inside the screen and doesn't cover the price.
      const box = await pill.boundingBox().catch(() => null);
      if (box && (box.x < 0 || box.x + box.width > v.viewport.width)) fail(s, "the pill runs off the screen");
      // The offer's sheet (phone) or inspector (desktop) says what buying it brings.
      await page.getByTestId("offer-0").click();
      const said = await page.getByTestId("offer-fusions").waitFor({ timeout: 15_000 }).then(() => true, () => false);
      if (!said) fail(s, "the offer's sheet doesn't name the fusion");
      await shot(page, `${s}-offer`);
      await noSplice(s, page);
      errs.forEach((e) => fail(s, e));
      await ctx.close();
    }
    console.log(`${v.name}: done`);
  }

  // 6. The number sits in the middle of its badge, at both widths in every palette.
  for (const width of [1280, 390]) {
    const mobile = width < 1000;
    for (const theme of ["plum", "brass", "felt"]) {
      const ctx = await browser.newContext({ viewport: { width, height: mobile ? 844 : 800 }, isMobile: mobile, hasTouch: mobile, deviceScaleFactor: 1 });
      await ctx.addInitScript((t) => localStorage.setItem("arena.theme", t), theme);
      for (const scene of ["fusion-shop", "forge-ready", "codex-units"]) {
        const label = `badges ${scene} ${width} ${theme}`;
        const page = await ctx.newPage();
        await page.goto(`${url}#scene=${scene}&lang=en`, { timeout: 120_000 });
        await page.locator(".card .stats").first().waitFor({ state: "attached", timeout: 120_000 });
        await page.evaluate(() => document.fonts.ready);
        await page.waitForTimeout(600);
        let rows = await badgeGaps(page);
        if (scene === "codex-units") {
          await page.getByTestId("codex-unit").first().click();
          await page.locator(".sheet-stat").first().waitFor({ timeout: 15_000 });
          await page.evaluate(() => document.fonts.ready);
          rows = rows.concat((await badgeGaps(page)).filter((r) => r.card === "sheet"));
          if (!rows.some((r) => r.card === "sheet")) failures.push(`${label}: no unit-sheet badge measured`);
        }
        if (rows.length < 4) failures.push(`${label}: only ${rows.length} badges measured`);
        for (const f of badgeFailures(rows, 1)) failures.push(`${label}: ${f}`);
        await page.close();
      }
      await ctx.close();
    }
    console.log(`badges at ${width}: done`);
  }
} finally {
  await browser.close();
  child?.kill();
}
for (const f of failures) console.log(`FAIL ${f}`);
console.log(`fusion marks: ${failures.length} failing; screenshots in ${out}`);
if (failures.length) process.exit(1);
