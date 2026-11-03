// R3-26: the viewer on a battle the turn cap stopped. Builds one
// (e2e/time-up-battle.ts), plays a round on a LOCAL server with that battle
// swapped in, skips to its end, and checks the caption: "Time's up: draw",
// the term with the hourglass and its rule with this battle's cap, and the
// end card's DRAW. Shots at 360x640, 1024x768 and 1440x900.
// Needs `npm run mvp:build` first.   node e2e/probe-time-up.mjs [out-dir]
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { launchChromium } from "./browser.mjs";
const rec = join(mkdtempSync(join(tmpdir(), "time-up-")), "time-up.json");
execFileSync("node", ["--import", "tsx/esm", "e2e/time-up-battle.ts", rec], { stdio: ["ignore", "inherit", "inherit"] });
const battle = JSON.parse(readFileSync(rec, "utf8"));
const cap = battle.log.at(-1).turns;
const outDir = process.argv[2] ?? "e2e/.shots/time-up";
mkdirSync(outDir, { recursive: true });
const port = 18000 + Math.floor(Math.random() * 1000);
const child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", STATIC_DIR: "mobile/dist" }, stdio: ["ignore", "ignore", "inherit"] });
const url = `http://127.0.0.1:${port}/arena/`;
const errors = [];
try {
  for (let i = 0; i < 120; i++) { try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {} await new Promise((r) => setTimeout(r, 500)); }
  const browser = await launchChromium();
  const sizes = [
    ["phone", { viewport: { width: 360, height: 640 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true }],
    ["d1024", { viewport: { width: 1024, height: 768 }, deviceScaleFactor: 1 }],
    ["d1440", { viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1 }],
  ];
  for (const [i, [label, opts]] of sizes.entries()) {
    const page = await browser.newPage(opts);
    await page.route((u) => /\/battles\/[^/]+$/.test(u.pathname), async (route) => {
      const res = await route.fetch();
      const body = await res.json();
      await route.fulfill({ response: res, json: { ...body, ...battle, battleId: body.battleId, player: body.player, runId: body.runId, kind: body.kind, round: body.round } });
    });
    await page.goto(url);
    await page.getByTestId("name-input").fill(`TimeUp${i}`);
    await page.getByTestId("name-submit").click();
    await page.getByTestId("play").click();
    await page.getByTestId("offer-0").click();
    await page.getByTestId("buy").click();
    await page.getByTestId("fight").click();
    await page.getByTestId("caption").waitFor();
    await page.waitForTimeout(800);
    await page.getByTestId("battle-end").click();
    await page.waitForTimeout(1000);
    const info = await page.evaluate(() => {
      const cap = document.querySelector('[data-testid="caption"]');
      const term = [...(cap?.querySelectorAll('[data-testid="caption-term"]') ?? [])].find((t) => /Time's up/.test(t.textContent ?? ""));
      return {
        caption: cap?.textContent?.trim() ?? "",
        title: term?.getAttribute("title") ?? null,
        hourglass: !!term?.querySelector('use[href="#i-hourglass"]'),
        word: document.querySelector('[data-testid="battle-word"]')?.textContent ?? "",
        hscroll: document.documentElement.scrollWidth > document.documentElement.clientWidth,
      };
    });
    console.log(label, JSON.stringify(info));
    if (!/Time's up: draw/.test(info.caption)) errors.push(`${label}: caption "${info.caption}"`);
    if (!info.title?.includes(`after turn ${cap}`)) errors.push(`${label}: Time's up tip "${info.title}"`);
    if (!info.hourglass) errors.push(`${label}: no hourglass on Time's up`);
    if (info.word !== "DRAW") errors.push(`${label}: end card says "${info.word}"`);
    if (info.hscroll) errors.push(`${label}: horizontal scroll`);
    await page.screenshot({ path: join(outDir, `time-up-${label}.png`) });
    // The end card hidden: the caption band alone.
    const hide = page.getByTestId("end-close");
    if (await hide.isVisible().catch(() => false)) { await hide.click(); await page.waitForTimeout(500); await page.screenshot({ path: join(outDir, `time-up-${label}-board.png`) }); }
    await page.close();
  }
  await browser.close();
} finally { child.kill(); }
for (const e of errors) console.log("  " + e);
console.log(errors.length ? `time-up probe: FAILED (${errors.length})` : `time-up probe: OK (cap ${cap})`);
process.exitCode = errors.length ? 1 : 0;
