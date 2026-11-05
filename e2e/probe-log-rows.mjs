// R4-3: the battle Log on a long late-game fight. Finds one
// (e2e/log-rows-battle.ts), plays a round on a LOCAL server with it swapped
// in, ends it, and checks the Log: grouped rows (as many as logRowsOf
// gives, less those inside a folded turn), no line repeating the one above in a beat, and a merged row's Why
// reaching each change it holds, each chip naming its unit.
// Needs `npm run mvp:build` first.   node e2e/probe-log-rows.mjs [out.png]
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { launchChromium } from "./browser.mjs";
const rec = join(mkdtempSync(join(tmpdir(), "log-rows-")), "long.json");
const found = JSON.parse(execFileSync("node", ["--import", "tsx/esm", "e2e/log-rows-battle.ts", rec], { encoding: "utf8" }).trim().split("\n").at(-1));
console.log(`long battle: ${found.events} events, ${found.waves} waves → ${found.rows} Log rows, ${found.folds} turns folded`);
const long = JSON.parse(readFileSync(rec, "utf8"));
const out = process.argv[2] ?? "e2e/.shots/log-rows.png";
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
    await route.fulfill({ response: res, json: { ...body, ...long, battleId: body.battleId, player: body.player, runId: body.runId, kind: body.kind, round: body.round } });
  });
  await page.goto(url);
  await page.getByTestId("name-input").fill("LogProbe");
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
  await rows.first().waitFor({ timeout: 30_000 });
  const texts = await rows.evaluateAll((els) => els.map((e) => ({ beat: e.getAttribute("data-beat"), text: e.textContent.trim() })));
  // Repeated turns fold (R4-13): a shut fold stands for its rows.
  const shown = texts.length + (await page.locator("[data-testid=log-fold]:visible").count());
  if (shown !== found.items) errors.push(`${shown} Log rows and folds on screen, foldTurnsOf gives ${found.items}`);
  const runs = texts.filter((t, i) => i && t.beat === texts[i - 1].beat && t.text === texts[i - 1].text);
  if (runs.length) errors.push(`${runs.length} rows repeat the one above, e.g. "${runs[0].text}"`);
  const merged = rows.filter({ hasText: /all allies|all enemies|\(×\d+\)|ticks \d+ units/ });
  const nMerged = await merged.count();
  if (!nMerged) errors.push("no grouped row (all allies / all enemies / ×N / ticks N units)");
  else {
    const row = merged.nth(Math.floor(nMerged / 2));
    await row.scrollIntoViewIfNeeded();
    console.log(`grouped rows: ${nMerged}; clicking "${(await row.textContent()).trim()}"`);
    await row.click();
    await page.getByTestId("trace-text").waitFor();
    const chips = await page.getByTestId("trace-group-change").allTextContents();
    console.log(`its Why: ${chips.length} changes: ${chips.slice(0, 6).join(" | ")}${chips.length > 6 ? " | …" : ""}`);
    if (chips.length < 2) errors.push(`a grouped row's Why lists ${chips.length} change(s)`);
    else {
      await page.getByTestId("trace-group-change").last().click();
      await page.getByTestId("trace-text").waitFor();
    }
    await page.screenshot({ path: out.replace(/\.png$/, "-why.png") });
    await page.getByTestId("trace-close").click();
    await page.getByTestId("tab-log").click();
    await row.scrollIntoViewIfNeeded();
  }
  await page.screenshot({ path: out });
  await browser.close();
} finally {
  child.kill();
}
console.log(errors.length ? `log rows probe: FAILED\n- ${errors.join("\n- ")}` : `log rows probe: OK, screenshots ${out}`);
process.exitCode = errors.length ? 1 : 0;
