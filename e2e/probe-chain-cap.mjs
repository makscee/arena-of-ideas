// R3-23: the viewer on a battle cut at the chain cap. Finds a capped battle
// of the shipped units (e2e/chain-cap-battle.ts), plays a round on a LOCAL
// server with that battle swapped in, ends it, and checks the Log's capped
// row: "Chain stopped after <cap> steps", the breaking chain, its tip.
// Needs `npm run mvp:build` first.   node e2e/probe-chain-cap.mjs [out.png]
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { launchChromium } from "./browser.mjs";
const rec = join(mkdtempSync(join(tmpdir(), "chain-cap-")), "capped.json");
execFileSync("node", ["--import", "tsx/esm", "e2e/chain-cap-battle.ts", rec], { stdio: ["ignore", "inherit", "inherit"] });
const capped = JSON.parse(readFileSync(rec, "utf8"));
const out = process.argv[2] ?? "e2e/.shots/chain-cap.png";
mkdirSync(dirname(out), { recursive: true });
// chain-cap-battle.ts checks this is MVP_RULES.chainStepCap.
const cap = capped.log.find((e) => e.type === "ChainCapped").steps;
const port = 18000 + Math.floor(Math.random() * 1000);
const child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", STATIC_DIR: "mobile/dist" }, stdio: ["ignore", "ignore", "inherit"] });
const url = `http://127.0.0.1:${port}/arena/`;
try {
  for (let i = 0; i < 120; i++) { try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {} await new Promise((r) => setTimeout(r, 500)); }
  const browser = await launchChromium();
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 });
  await page.route((u) => /\/battles\/[^/]+$/.test(u.pathname), async (route) => {
    const res = await route.fetch();
    const body = await res.json();
    await route.fulfill({ response: res, json: { ...body, ...capped, battleId: body.battleId, player: body.player, runId: body.runId, kind: body.kind, round: body.round } });
  });
  await page.goto(url);
  await page.getByTestId("name-input").fill("CapProbe");
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
  const tab = page.getByTestId("tab-log");
  if (await tab.isVisible().catch(() => false)) await tab.click();
  const row = page.locator("[data-testid=log-row]:visible").filter({ hasText: "Chain stopped after" }).first();
  await row.waitFor({ state: "attached", timeout: 30_000 });
  await row.scrollIntoViewIfNeeded();
  const info = await row.evaluate((r) => { const t = r.querySelector("[title]"); return { text: r.textContent, title: t?.getAttribute("title"), icon: !!t?.querySelector("svg") }; });
  console.log(JSON.stringify(info));
  const ok = info.text.trim() === `Chain stopped after ${cap} steps` && info.icon && info.title?.includes(`ran ${cap} steps`);
  console.log(ok ? `chain cap probe: OK (${cap})` : "chain cap probe: FAILED");
  process.exitCode = ok ? 0 : 1;
  await row.click().catch(() => {});
  await page.waitForTimeout(800);
  await page.screenshot({ path: out });
  await browser.close();
} finally { child.kill(); }
