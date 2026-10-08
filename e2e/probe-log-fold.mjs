// R4-13: repeated turns fold in the battle Log. Finds the fight with the most
// folded turns (e2e/log-rows-battle.ts … loop), plays a round on a LOCAL
// server with it swapped in, ends it, and checks the Log: one row per
// repeated turn (as many folds as foldTurnsOf gives), a fold's click shows
// its rows (and a row inside opens its Why), a second click hides them.
// Needs `npm run mvp:build` first.   node e2e/probe-log-fold.mjs [out.png]
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { launchChromium } from "./browser.mjs";
const rec = join(mkdtempSync(join(tmpdir(), "log-fold-")), "loop.json");
const found = JSON.parse(execFileSync("node", ["--import", "tsx/esm", "e2e/log-rows-battle.ts", rec, "200", "loop"], { encoding: "utf8" }).trim().split("\n").at(-1));
console.log(`looping battle: ${found.events} events, ${found.rows} Log rows → ${found.items} with ${found.turns} turns in ${found.folds} folds`);
const loop = JSON.parse(readFileSync(rec, "utf8"));
const out = process.argv[2] ?? "e2e/.shots/log-fold.png";
mkdirSync(dirname(out), { recursive: true });
const port = 18000 + Math.floor(Math.random() * 1000);
const child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", STATIC_DIR: "mobile/dist" }, stdio: ["ignore", "ignore", "inherit"] });
const url = `http://127.0.0.1:${port}/arena/`;
const errors = [];
try {
  for (let i = 0; i < 120; i++) { try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {} await new Promise((r) => setTimeout(r, 500)); }
  const browser = await launchChromium();
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 });
  await page.route((u) => /\/battles\/[^/]+$/.test(u.pathname), async (route) => {
    const res = await route.fetch();
    const body = await res.json();
    await route.fulfill({ response: res, json: { ...body, ...loop, battleId: body.battleId, player: body.player, runId: body.runId, kind: body.kind, round: body.round } });
  });
  await page.goto(url);
  await page.getByTestId("name-input").fill("FoldProbe");
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").click();
  await page.getByTestId("offer-0").click();
  await page.getByTestId("buy").click();
  await page.getByTestId("fight").click();
  await page.getByTestId("caption").waitFor();
  await page.waitForTimeout(1000);
  await page.getByTestId("battle-end").click();
  await page.waitForTimeout(800);
  const hide = page.getByTestId("end-close");
  if (await hide.isVisible().catch(() => false)) await hide.click();
  await page.getByTestId("tab-log").click();
  const rows = page.locator("[data-testid=log-row]:visible");
  const folds = page.locator("[data-testid=log-fold]:visible");
  await rows.first().waitFor({ timeout: 30_000 });
  const nFolds = await folds.count();
  const shown = (await rows.count()) + nFolds;
  console.log(`on screen: ${shown} Log lines, ${nFolds} folded turns`);
  if (nFolds !== found.folds) errors.push(`${nFolds} folded turns on screen, foldTurnsOf gives ${found.folds}`);
  if (shown !== found.items) errors.push(`${shown} Log lines on screen, foldTurnsOf gives ${found.items}`);
  if (nFolds) {
    // The longest run of repeated turns (T5–T9), else the middle fold.
    const spans = await folds.evaluateAll((els) => els.map((e) => Number(e.dataset.turnTo) - Number(e.dataset.turn)));
    const fold = folds.nth(spans.some((x) => x > 0) ? spans.indexOf(Math.max(...spans)) : Math.floor(nFolds / 2));
    const turn = await fold.getAttribute("data-turn");
    await fold.scrollIntoViewIfNeeded();
    console.log(`fold T${turn}: "${(await fold.textContent()).trim()}"`);
    await page.screenshot({ path: out });
    const before = await rows.count();
    await fold.click();
    const box = page.locator("[data-testid=log-fold-rows]:visible");
    const inside = box.locator("[data-testid=log-row]");
    const nInside = await inside.count();
    if ((await fold.getAttribute("aria-expanded")) !== "true") errors.push("a clicked fold is not open");
    if (nInside < 2) errors.push(`an open fold shows ${nInside} rows`);
    if ((await rows.count()) !== before + nInside) errors.push(`opening a fold showed ${(await rows.count()) - before} rows, it holds ${nInside}`);
    console.log(`open: ${nInside} rows: ${(await inside.allTextContents()).slice(0, 4).map((t) => t.trim()).join(" | ")} …`);
    await page.screenshot({ path: out.replace(/\.png$/, "-open.png") });
    // A row that changes nothing (a Freeze stopping a strike) has no Why: the first that hurts or heals.
    const texts = await inside.allTextContents();
    const k = Math.max(0, texts.findIndex((t) => /\+\d|→ −\d/.test(t)));
    await inside.nth(k).click();
    await page.getByTestId("trace-text").waitFor({ timeout: 5000 }).catch(() => errors.push(`a row inside a fold did not open its Why: "${texts[k].trim()}"`));
    await page.getByTestId("trace-close").click().catch(() => {});
    await page.getByTestId("tab-log").click();
    const again = page.locator(`[data-testid=log-fold][data-turn="${turn}"]`);
    if ((await again.getAttribute("aria-expanded")) !== "true") errors.push("the fold shut after a row's Why");
    await again.click();
    if ((await again.getAttribute("aria-expanded")) !== "false") errors.push("a second click did not shut the fold");
    if (await page.locator("[data-testid=log-fold-rows]:visible").count()) errors.push("a shut fold still shows its rows");
  }
  await browser.close();
} finally {
  child.kill();
}
console.log(errors.length ? `log fold probe: FAILED\n- ${errors.join("\n- ")}` : `log fold probe: OK, screenshots ${out}`);
process.exitCode = errors.length ? 1 : 0;
