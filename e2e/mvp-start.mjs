// The first minute (arena#31, mission #26 move 3): a fresh browser opens on a
// chain, PLAY (Enter) skips the name form, round 1's coach builds Medic behind
// Gambler, and the first fight ends in under 20 s of PLAY, against the
// opponent the shop previewed. At 1280×800, 800×450 and 390×844; the 800×450
// frame has no scroll and no text under 11 px. Without --url it builds the
// client and starts the server (rules v2's pool) on a free port.
//   npm run mvp:start -- [--url http://127.0.0.1:8841/arena/] [--out e2e/.shots/mvp-start]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";
import { replayBattle } from "./battle-ctl.mjs";
import { chainProblems, readChain } from "./chain-checks.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/mvp-start";
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", ARENA_CODEX: "1", MVP_POOL: "v2" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
  for (let i = 0; i < 300; i++) {
    try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
    await new Promise((r) => setTimeout(r, 200));
  }
}

const SIZES = [["1280x800", { width: 1280, height: 800 }], ["800x450", { width: 800, height: 450 }], ["390x844", { width: 390, height: 844 }]];
const browser = await launchChromium();
const errors = [];
try {
  for (const [label, viewport] of SIZES) {
    const ctx = await browser.newContext({ viewport });
    const page = await ctx.newPage();
    const fail = (m) => errors.push(`${label}: ${m}`);
    page.on("pageerror", (e) => fail(`pageerror: ${e.message}`));
    page.on("console", (m) => m.type() === "error" && fail(`console: ${m.text()}`));
    const tid = (id) => page.getByTestId(id);
    const desk = viewport.width >= 1024;
    /** Buys the offer the coach points at, as a player does: a key, or tap the offer then its slot. */
    const buy = async (name, slot) => {
      const cell = page.locator(`[data-testid="offers"] [data-testid^="offer-"][data-name="${name}"]`);
            // The table's way: the offer in hand (a tap), then its slot (a key or a tap).
      await cell.click();
      if (desk) await page.keyboard.press(String(slot + 1));
      else await tid(`line-${slot}-empty`).click();
    };

    await page.goto(url, { timeout: 20_000 });
    await tid("start").waitFor({ timeout: 10_000 });
    if ((await tid("start-title").textContent()) !== "One rule starts a chain.") fail("the first screen's line is wrong");
    if ((await tid("start-play").count()) !== 1) fail("no PLAY");
    // The chain plays itself: the caption moves on.
    const first = await tid("start-caption").textContent();
    await page.waitForFunction((was) => document.querySelector('[data-testid="start-caption"]')?.textContent !== was, first, { timeout: 5_000 }).catch(() => fail("the chain does not play itself"));
    await page.screenshot({ path: `${out}/start-${label}.png` });
    const fit = await page.evaluate(() => ({ scroll: document.documentElement.scrollHeight - document.documentElement.clientHeight, small: [...document.querySelectorAll("#app *")].filter((e) => [...e.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim()) && parseFloat(getComputedStyle(e).fontSize) < 11).map((e) => e.textContent.trim().slice(0, 24)) }));
    if (fit.scroll > 0) fail(`the first screen scrolls by ${fit.scroll}px`);
    if (fit.small.length) fail(`text under 11px on the first screen: ${fit.small.join(", ")}`);

    const t0 = Date.now();
    await page.keyboard.press("Enter");
    await tid("coach").waitFor({ timeout: 10_000 });
    if ((await tid("name-input").count()) !== 0) fail("the name form is still there after PLAY");
    const coach = async () => (await tid("coach").getAttribute("data-step"));
    if ((await coach()) !== "buy-first") fail(`the coach starts at ${await coach()}`);
    await page.screenshot({ path: `${out}/coach-1-${label}.png` });
    const shopFit = await page.evaluate(() => document.documentElement.scrollHeight - document.documentElement.clientHeight);
    if (label === "800x450" && shopFit > 0) fail(`the shop with the coach scrolls by ${shopFit}px`);

    await buy("Gambler", 0);
    await page.waitForFunction(() => document.querySelector('[data-testid="coach"]')?.getAttribute("data-step") === "buy-second", null, { timeout: 5_000 }).catch(async () => fail(`after Gambler the coach says ${await coach().catch(() => "nothing")}`));
    if (!/Put Medic behind Gambler/.test(await tid("coach").innerText())) fail("the coach does not say Put Medic behind Gambler");
    await page.screenshot({ path: `${out}/coach-2-${label}.png` });
    await buy("Medic", 1);
    await page.waitForFunction(() => document.querySelector('[data-testid="coach"]')?.getAttribute("data-step") === "fight", null, { timeout: 5_000 }).catch(async () => fail(`after Medic the coach says ${await coach().catch(() => "nothing")}`));
    if (!/Now Fight/.test(await tid("coach").innerText())) fail("the coach does not say Now Fight");
    if ((await tid("fight-warn").count()) !== 0) fail("Fight warns about empty slots while the coach says Fight");
    const line = await page.locator('[data-testid="line"] .pc[data-name]').evaluateAll((els) => els.map((e) => e.getAttribute("data-name")));
    if (line.join() !== "Gambler,Medic") fail(`the line is ${line.join()}`);
    const next = (await tid("next-opponent").textContent()) ?? "";
    const foe = next.replace(/^.*?@/, "@").trim().split(/\s+/).slice(0, 1).join("");
    await page.screenshot({ path: `${out}/coach-3-${label}.png` });

    const decided = page.waitForResponse((r) => r.request().method() === "POST" && /\/runs\/[^/]+\/decisions$/.test(r.url()) && r.request().postData()?.includes('"fight"'));
    await tid("fight").click();
    const fought = await (await decided).json();
    // The fight uses the bought line against the opponent the shop previewed.
    if (foe.length > 1 && `@${fought.fight.opponent.player.name}` !== foe) fail(`the first fight is against @${fought.fight.opponent.player.name}, the shop showed ${foe}`);
    await page.getByTestId("battle-end").or(tid("end-card")).first().waitFor({ timeout: 15_000 });
    // The battle on the table (arena#29): no scroll, no text under 11 px as drawn (the stage is scaled), playing, with a unit's inspector open, and the result.
    const drawnFit = () => page.evaluate(() => {
      const k = Number(document.querySelector('[data-testid="stage"]')?.style.getPropertyValue("--k") || 1);
      const small = [...document.querySelectorAll("#app .stage *")].filter((e) => e.getClientRects().length && getComputedStyle(e).visibility !== "hidden" && [...e.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim()) && parseFloat(getComputedStyle(e).fontSize) * k < 10.99).map((e) => `${e.textContent.trim().slice(0, 20)} ${(parseFloat(getComputedStyle(e).fontSize) * k).toFixed(1)}px`);
      return { scroll: document.documentElement.scrollHeight - document.documentElement.clientHeight, small: [...new Set(small)] };
    });
    const checkBattle = async (what) => {
      const f = await drawnFit();
      if (f.scroll > 0) fail(`the battle (${what}) scrolls by ${f.scroll}px`);
      if (f.small.length) fail(`text under 11 px in the battle (${what}): ${f.small.join(", ")}`);
    };
    if (await tid("battle-play").count()) {
      await checkBattle("playing");
      await page.keyboard.press("Space");
      // Beat by beat to the end: statuses, result tags and chain steps appear along the way, none under 11 px.
      let chained = 0;
      for (let i = 0; i < 40 && !(await tid("battle-step").isDisabled()); i++) {
        await page.keyboard.press("ArrowRight");
        if (i % 2 === 0) await checkBattle(`beat ${i + 1}`);
        // The chain on the field (arena#30): one step's numbered wire, the tray lists the steps so far with CHAIN ×N.
        const chain = await readChain(page);
        if (chain.count >= 2) { chained++; if (chained === 1) await page.screenshot({ path: `${out}/battle-chain-${label}.png` }); }
        for (const bad of chainProblems(chain)) fail(`chain, beat ${i + 1}: ${bad}`);
      }
      if (!chained) fail("no chain was listed in the tray in the first fight (Gambler hit, Medic answers)");
      await replayBattle(page);
      await page.waitForTimeout(600);
      if ((await tid("battle-play").textContent()) === "❚❚") await page.keyboard.press("Space");
      await page.locator('[data-testid="battle-you"] .bv-card .pc-face').first().click();
      await tid("inspector").waitFor({ timeout: 3_000 }).catch(() => fail("a unit's tap didn't open the inspector"));
      await page.screenshot({ path: `${out}/battle-inspector-${label}.png` });
      await checkBattle("inspector");
      await page.keyboard.press("Escape");
      await page.screenshot({ path: `${out}/battle-paused-${label}.png` });
    }
    if (!(await tid("end-card").isVisible())) await page.keyboard.press("e"); // E ends the battle (the End button gives way to Next round once a result was seen)
    await tid("end-card").waitFor({ timeout: 10_000 });
    const took = Date.now() - t0;
    if (took > 20_000) fail(`PLAY to the end of the first fight took ${Math.round(took / 1000)} s`);
    await checkBattle("result");
    console.log(`${label}: PLAY to the end of the first fight in ${(took / 1000).toFixed(1)} s, against ${foe}`);
    await page.screenshot({ path: `${out}/fight-end-${label}.png` });
    await ctx.close();
  }
} catch (e) {
  errors.push(`${e instanceof Error ? e.message : e}`);
} finally {
  await browser.close();
  child?.kill();
}
if (errors.length) {
  console.error(errors.map((e) => `FAIL ${e}`).join("\n"));
  process.exit(1);
}
console.log("mvp start: ok");
