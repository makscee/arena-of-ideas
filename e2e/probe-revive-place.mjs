// #918: the viewer on a battle where a revive returns mid-line. Builds one
// from the shipped units (e2e/revive-place-battle.ts: the front Squire falls
// and Necromancer raises it), plays it on a LOCAL server with that battle
// swapped in, and checks, on a phone (EN and RU) and on desktop, that the
// caption says it returns to its place and that the line on the board then
// matches the engine's: the Squire at the front, not behind Necromancer.
// Then the Squire falls again and stays down (a unit is revived once per
// battle): the caption says it can't return again, with a chip on Necromancer.
// Needs `npm run mvp:build` first.   node e2e/probe-revive-place.mjs [out.png]
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { launchChromium } from "./browser.mjs";
import { localeOf } from "./lang.mjs";
const rec = join(mkdtempSync(join(tmpdir(), "revive-place-")), "revive-place.json");
execFileSync("node", ["--import", "tsx/esm", "e2e/revive-place-battle.ts", rec], { stdio: ["ignore", "inherit", "inherit"] });
const { expect, ...record } = JSON.parse(readFileSync(rec, "utf8"));
const out = process.argv[2] ?? "e2e/.shots/revive-place.png";
mkdirSync(dirname(out), { recursive: true });
const port = 18000 + Math.floor(Math.random() * 1000);
const child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", STATIC_DIR: "mobile/dist" }, stdio: ["ignore", "ignore", "inherit"] });
const url = `http://127.0.0.1:${port}/arena/`;
const CAPTION = { en: /Squire returns to its place at \d+ HP/, ru: /Squire возвращается на своё место с \d+/ };
const AGAIN = { en: /Squire can't return again: once per battle/, ru: /Squire не вернётся: раз за бой/ };
const CHIP = { en: "can't return", ru: "лишь раз" };
try {
  for (let i = 0; i < 360; i++) { try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {} await new Promise((r) => setTimeout(r, 500)); }
  const browser = await launchChromium();
  /** A page on a fresh name, in the first fight, the battle swapped for the revive one. */
  async function fight(viewport, who, lang) {
    const page = await browser.newPage({ viewport, deviceScaleFactor: 2, locale: localeOf(lang) });
    await page.route((u) => /\/battles\/[^/]+$/.test(u.pathname), async (route) => {
      const res = await route.fetch();
      const body = await res.json();
      await route.fulfill({ response: res, json: { ...body, ...record, battleId: body.battleId, player: body.player, runId: body.runId, kind: body.kind, round: body.round } });
    });
    await page.goto(url);
    await page.getByTestId("name-input").fill(who);
    await page.getByTestId("name-submit").click();
    await page.getByTestId("play").click();
    await page.getByTestId("offer-0").click(); await page.getByTestId("offer-0").click();
    await page.getByTestId("buy").click();
    await page.getByTestId("fight").click();
    await page.getByTestId("caption").waitFor();
    return page;
  }
  /** Plays until the caption tells the revive, then reads side A's living line, front first. */
  async function atRevive(page, lang) {
    const caption = () => page.evaluate(() => document.querySelector('[data-testid="caption"]')?.textContent?.replace(/\s+/g, " ").trim() ?? "");
    let text = "";
    for (let i = 0; i < 200 && !CAPTION[lang].test(text); i++) { await page.waitForTimeout(100); text = await caption(); }
    await page.waitForTimeout(700); // the slide settles
    const line = await page.evaluate(() => [...document.querySelectorAll('[data-testid="battle-you"] .bv-card:not(.dead)')].map((c) => c.getAttribute("data-unit")));
    return { text, line };
  }
  /** Plays on until the caption tells the failed second revive; reads the chip on the reviver. */
  async function atAgain(page, lang) {
    const caption = () => page.evaluate(() => document.querySelector('[data-testid="caption"]')?.textContent?.replace(/\s+/g, " ").trim() ?? "");
    let text = "";
    for (let i = 0; i < 300 && !AGAIN[lang].test(text); i++) { await page.waitForTimeout(100); text = await caption(); }
    const chip = await page.evaluate(() => [...document.querySelectorAll('[data-testid="change"].noRoom')].map((c) => `${c.closest("[data-unit]")?.getAttribute("data-unit") ?? "?"}: ${c.textContent?.trim()}`));
    return { text, chip };
  }
  let ok = true;
  for (const [name, viewport, lang] of [["phone-en", { width: 360, height: 640 }, "en"], ["phone-ru", { width: 360, height: 640 }, "ru"], ["desktop-en", { width: 1280, height: 800 }, "en"]]) {
    const page = await fight(viewport, `Rev${name.replace(/\W/g, "")}`, lang);
    const { text, line } = await atRevive(page, lang);
    const good = CAPTION[lang].test(text) && JSON.stringify(line) === JSON.stringify(expect);
    console.log(`${name}: ${good ? "ok" : "FAILED"} · caption "${text}" · line ${line.join(", ")}${good ? "" : ` (want ${expect.join(", ")})`}`);
    await page.screenshot({ path: out.replace(/\.png$/, `-${name}.png`) });
    const again = await atAgain(page, lang);
    const goodAgain = AGAIN[lang].test(again.text) && again.chip.some((c) => c.includes("Necromancer") && c.includes(CHIP[lang]));
    console.log(`${name} again: ${goodAgain ? "ok" : "FAILED"} · caption "${again.text}" · chips ${again.chip.join(" | ") || "none"}`);
    ok &&= good && goodAgain;
    await page.screenshot({ path: out.replace(/\.png$/, `-${name}-again.png`) });
    await page.close();
  }
  console.log(ok ? "revive place probe: OK" : "revive place probe: FAILED");
  process.exitCode = ok ? 0 : 1;
  await browser.close();
} finally { child.kill(); }
