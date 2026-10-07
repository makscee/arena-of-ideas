// R4-2: the viewer on a battle with "No room". Builds one from the shipped
// units (e2e/no-room-battle.ts), plays a round on a LOCAL server with that
// battle swapped in, ends it, and checks the Log's rows: "Mortwrought → No
// room for Imp" and "… No room to revive …", with the term's icon and tip, and
// that a row opens a Why trace starting at No room.
// Needs `npm run mvp:build` first.   node e2e/probe-no-room.mjs [out.png]
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { launchChromium } from "./browser.mjs";
const rec = join(mkdtempSync(join(tmpdir(), "no-room-")), "no-room.json");
execFileSync("node", ["--import", "tsx/esm", "e2e/no-room-battle.ts", rec], { stdio: ["ignore", "inherit", "inherit"] });
const record = JSON.parse(readFileSync(rec, "utf8"));
const out = process.argv[2] ?? "e2e/.shots/no-room.png";
mkdirSync(dirname(out), { recursive: true });
const port = 18000 + Math.floor(Math.random() * 1000);
const child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", STATIC_DIR: "mobile/dist" }, stdio: ["ignore", "ignore", "inherit"] });
const url = `http://127.0.0.1:${port}/arena/`;
try {
  for (let i = 0; i < 120; i++) { try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {} await new Promise((r) => setTimeout(r, 500)); }
  const browser = await launchChromium();
  /** A page on a fresh name, in the first fight, the battle swapped for the No room one. */
  async function fight(viewport, who) {
    const page = await browser.newPage({ viewport, deviceScaleFactor: 2 });
    await page.route((u) => /\/battles\/[^/]+$/.test(u.pathname), async (route) => {
      const res = await route.fetch();
      const body = await res.json();
      await route.fulfill({ response: res, json: { ...body, ...record, battleId: body.battleId, player: body.player, runId: body.runId, kind: body.kind, round: body.round } });
    });
    await page.goto(url);
    await page.getByTestId("name-input").fill(who);
    await page.getByTestId("name-submit").click();
    await page.getByTestId("play").click();
    await page.getByTestId("offer-0").click();
    await page.getByTestId("buy").click();
    await page.getByTestId("fight").click();
    await page.getByTestId("caption").waitFor();
    return page;
  }
  const whyText = (page) => page.evaluate(() => document.querySelector('[data-testid="why-chain"]')?.textContent?.replace(/\s+/g, " ").trim() ?? "");
  let ok = true;

  // Desktop: the Log's rows, their term, and a row's Why.
  const desk = await fight({ width: 1280, height: 800 }, "RoomDesk");
  await desk.waitForTimeout(1000);
  await desk.getByTestId("battle-end").click();
  await desk.waitForTimeout(800);
  const hide = desk.getByTestId("end-close");
  if (await hide.isVisible().catch(() => false)) await hide.click();
  await desk.getByTestId("tab-log").click();
  const rows = desk.locator("[data-testid=log-row]:visible").filter({ hasText: "No room" });
  await rows.first().waitFor({ timeout: 30_000 });
  const infos = await rows.evaluateAll((rs) => rs.map((r) => { const t = r.querySelector("[title]"); return { text: r.textContent?.replace(/\s+/g, " ").trim(), title: t?.getAttribute("title"), icon: !!t?.querySelector("svg") }; }));
  console.log(JSON.stringify(infos.slice(0, 2)));
  const imp = infos.some((i) => /Mortwrought → No room for Imp$/.test(i.text ?? ""));
  const revive = infos.some((i) => /Mortwrought → No room to revive \S+/.test(i.text ?? ""));
  const termed = infos.every((i) => i.icon && /line is full/.test(i.title ?? ""));
  if (!(imp && revive && termed)) { ok = false; console.log(`Log rows: imp ${imp}, revive ${revive}, term ${termed}`); }
  await rows.nth(1).scrollIntoViewIfNeeded();
  await rows.nth(1).click();
  await desk.waitForTimeout(800);
  const deskWhy = await whyText(desk);
  console.log(`desktop why: ${deskWhy.slice(0, 160)}`);
  if (!/No room/.test(deskWhy) || !/falls/.test(deskWhy)) ok = false;
  await desk.screenshot({ path: out.replace(/\.png$/, "-desktop.png") });

  // Phone: the caption says it, and the "no room" chip on Mortwrought opens its Why.
  const phone = await fight({ width: 360, height: 640 }, "RoomPhone");
  const chip = phone.locator('[data-testid="change"].noRoom').first();
  await chip.waitFor({ timeout: 30_000 });
  const caption = await phone.evaluate(() => document.querySelector('[data-testid="caption"]')?.textContent?.replace(/\s+/g, " ").trim() ?? "");
  console.log(`phone caption: ${caption}`);
  await phone.screenshot({ path: out.replace(/\.png$/, "-phone-caption.png") });
  await chip.click({ force: true });
  await phone.waitForTimeout(800);
  const phoneWhy = await whyText(phone);
  console.log(`phone why: ${phoneWhy.slice(0, 160)}`);
  if (!/No room/.test(phoneWhy)) ok = false;
  await phone.screenshot({ path: out.replace(/\.png$/, "-phone-why.png") });

  console.log(ok ? `no room probe: OK (${infos.length} Log rows)` : "no room probe: FAILED");
  process.exitCode = ok ? 0 : 1;
  await browser.close();
} finally { child.kill(); }
