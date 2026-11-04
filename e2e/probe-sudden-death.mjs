// R4-1: the viewer on a battle sudden death ends. Builds one
// (e2e/sudden-death-battle.ts), plays a round on a LOCAL server with that
// battle swapped in, skips to its end, opens the Log and checks it says why:
// "Sudden death: Fatigue → everyone takes 20, nothing blocks it", a revive
// that "can't return: sudden death", the term with its skull and rule, and
// no Time's up. Shots at 360x640, 1024x768 and 1440x900.
// Needs `npm run mvp:build` first.   node e2e/probe-sudden-death.mjs [out-dir]
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { launchChromium } from "./browser.mjs";
const rec = join(mkdtempSync(join(tmpdir(), "sudden-death-")), "sudden-death.json");
execFileSync("node", ["--import", "tsx/esm", "e2e/sudden-death-battle.ts", rec], { stdio: ["ignore", "inherit", "inherit"] });
const battle = JSON.parse(readFileSync(rec, "utf8"));
const outDir = process.argv[2] ?? "e2e/.shots/sudden-death";
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
    await page.getByTestId("name-input").fill(`Sudden${i}`);
    await page.getByTestId("name-submit").click();
    await page.getByTestId("play").click();
    await page.getByTestId("offer-0").click();
    await page.getByTestId("buy").click();
    await page.getByTestId("fight").click();
    await page.getByTestId("caption").waitFor();
    await page.waitForTimeout(800);
    await page.getByTestId("battle-end").click();
    await page.waitForTimeout(1000);
    const tab = page.getByTestId("tab-log");
    if (await tab.isVisible().catch(() => false)) { await tab.click(); await page.waitForTimeout(500); }
    const info = await page.evaluate(() => {
      const rows = [...document.querySelectorAll('[data-testid="log-row"]')].map((r) => r.textContent?.replace(/\s+/g, " ").trim() ?? "");
      const term = [...document.querySelectorAll('[data-testid="caption-term"]')].find((t) => /sudden death/i.test(t.textContent ?? ""));
      return {
        rows: rows.length,
        fatigue: rows.find((r) => /Sudden death: Fatigue/.test(r)) ?? null,
        failed: rows.find((r) => /can't return: sudden death/.test(r)) ?? null,
        title: term?.getAttribute("title") ?? null,
        skull: !!term?.querySelector('use[href="#i-death-skull"]'),
        timeUp: /Time's up/.test(document.body.textContent ?? ""),
        word: document.querySelector('[data-testid="battle-word"]')?.textContent ?? "",
        hscroll: document.documentElement.scrollWidth > document.documentElement.clientWidth,
      };
    });
    console.log(label, JSON.stringify(info));
    if (!info.fatigue?.includes("everyone takes 20")) errors.push(`${label}: no sudden-death fatigue row`);
    if (!info.failed) errors.push(`${label}: no "can't return: sudden death" row`);
    if (!info.title?.startsWith("From turn 20, the turn-end damage doubles")) errors.push(`${label}: Sudden death tip "${info.title}"`);
    if (!info.skull) errors.push(`${label}: no skull on Sudden death`);
    if (info.timeUp) errors.push(`${label}: says Time's up`);
    if (info.hscroll) errors.push(`${label}: horizontal scroll`);
    await page.screenshot({ path: join(outDir, `sudden-death-${label}.png`) });
    await page.close();
  }
  await browser.close();
} finally { child.kill(); }
for (const e of errors) console.log("  " + e);
console.log(errors.length ? `sudden-death probe: FAILED (${errors.length})` : "sudden-death probe: OK");
process.exitCode = errors.length ? 1 : 0;
