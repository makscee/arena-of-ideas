// Play feedback (arena#31, fix round after the blind first-play audit): with
// the API slowed to 1500 ms, a press on PLAY (click or Enter) is acknowledged
// within 100 ms and never starts a second run; a refused or failing start says
// so and PLAY works again; the coach in round 1 says how to buy (a how-line:
// "Tap Medic, then slot 2 · or press 2") and lights only the slot behind
// Gambler and refuses another one in one line; and a slow first load shows the
// title and a loader at once. Starts the server itself (rules v2's pool).
//   npm run mvp:play-feedback -- [--url http://127.0.0.1:8841/arena/]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
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

const DELAY = 1500;
const ACK_MS = 100;
const SIZES = [["1280x800", { width: 1280, height: 800 }], ["390x844", { width: 390, height: 844 }]];
const browser = await launchChromium();
const errors = [];

/** A fresh page with every API call slowed by `delay` ms, counting the calls that matter. */
async function open(viewport, label, { delay = DELAY, refuse = {} } = {}) {
  const ctx = await browser.newContext({ viewport });
  const page = await ctx.newPage();
  const seen = { players: 0, runs: 0, buys: [] };
  const fail = (m) => errors.push(`${label}: ${m}`);
  page.on("pageerror", (e) => fail(`pageerror: ${e.message}`));
  await page.route("**/api/v1/**", async (route) => {
    const req = route.request();
    const path = new URL(req.url()).pathname.replace(/^.*\/api\/v1/, "");
    const post = req.method() === "POST";
    const key = post && path === "/players" ? "players" : post && path === "/runs" ? "runs" : "";
    if (key) seen[key]++;
    if (post && /\/decisions$/.test(path) && req.postData()?.includes('"buy"')) seen.buys.push(req.postData());
    await new Promise((r) => setTimeout(r, delay));
    if (key && refuse[key] > 0) return void (refuse[key]--, route.fulfill({ status: 500, contentType: "application/json", body: JSON.stringify({ error: "the server is busy" }) }));
    await route.continue();
  });
  return { ctx, page, seen, fail, tid: (id) => page.getByTestId(id) };
}

/** Milliseconds from the page's own first sight of the press (a click or a key) to the first frame where `probe` holds (null: never within 1 s). */
async function ackWithin(page, act, probe) {
  await page.evaluate((src) => {
    window.__ack = null;
    window.__t0 = null;
    const probeFn = new Function(`return (${src})()`);
    const seen = () => { if (window.__t0 === null) window.__t0 = performance.now(); };
    addEventListener("click", seen, true);
    addEventListener("keydown", seen, true);
    const tick = () => {
      if (window.__t0 !== null && probeFn()) window.__ack = performance.now() - window.__t0;
      else if (window.__t0 === null || performance.now() - window.__t0 < 1000) requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  }, probe);
  await act();
  await page.waitForTimeout(1200);
  return page.evaluate(() => window.__ack);
}

const acknowledged = `() => { const b = document.querySelector('[data-testid="start-play"]'); const s = document.querySelector('[data-testid="start"]'); return !!b && b.getAttribute("aria-busy") === "true" && !!s && s.classList.contains("starting"); }`;

try {
  for (const [label, viewport] of SIZES) {
    // 1. A slow API: PLAY by click, and by Enter, answers at once and starts one run.
    for (const how of ["click", "Enter"]) {
      const { ctx, page, seen, fail, tid } = await open(viewport, `${label} ${how}`);
      await page.goto(url, { timeout: 20_000 });
      await tid("start-play").waitFor({ timeout: 15_000 });
      const act = how === "click" ? () => tid("start-play").click() : () => page.keyboard.press("Enter");
      const ms = await ackWithin(page, act, acknowledged);
      if (ms === null || ms > ACK_MS) fail(`PLAY by ${how} was acknowledged ${ms === null ? "never" : `after ${Math.round(ms)} ms`} (limit ${ACK_MS} ms)`);
      // The chain demo stops while it starts, and a second press does nothing.
      const cap = await tid("start-caption").textContent();
      await page.keyboard.press("Enter");
      await tid("start-play").click({ force: true, timeout: 2000 }).catch(() => {});
      await page.waitForTimeout(200);
      if ((await tid("start-caption").textContent()) !== cap) fail("the chain demo keeps playing while the run starts");
      await tid("coach").waitFor({ timeout: 20_000 }).catch(() => fail("no table after a slow start"));
      if (seen.players !== 1 || seen.runs !== 1) fail(`${seen.players} registrations and ${seen.runs} run starts after pressing PLAY three times, not 1 and 1`);
      await ctx.close();
    }

    // 2. A refused registration, then a refused run start: a message each time, PLAY works again, one player in the end.
    {
      const { ctx, page, seen, fail, tid } = await open(viewport, `${label} refused`, { delay: 200, refuse: { players: 1, runs: 1 } });
      await page.goto(url, { timeout: 20_000 });
      await tid("start-play").waitFor({ timeout: 15_000 });
      await tid("start-play").click();
      await page.waitForFunction(() => document.querySelector('[data-testid="error"]')?.textContent?.trim(), null, { timeout: 5000 }).catch(() => fail("a refused registration says nothing"));
      if (!(await tid("start").count()) || (await tid("start-play").getAttribute("aria-busy")) === "true") fail("after a refused registration PLAY is not ready again");
      await tid("start-play").click();
      await page.waitForFunction(() => document.querySelector('[data-testid="error"]')?.textContent?.trim() && !document.querySelector('[data-testid="start-play"]')?.getAttribute("aria-busy"), null, { timeout: 5000 }).catch(() => fail("a refused run start says nothing"));
      await tid("start-play").click();
      await tid("coach").waitFor({ timeout: 10_000 }).catch(() => fail("PLAY does not work after two refusals"));
      if (seen.players !== 2) fail(`${seen.players} registrations: a retry after a registered player must not register again (expected 2: one refused, one made)`);
      await ctx.close();
    }

    // 3. The coach forces Medic into slot 2.
    {
      const { ctx, page, seen, fail, tid } = await open(viewport, `${label} coach`, { delay: 0 });
      await page.goto(url, { timeout: 20_000 });
      await tid("start-play").click();
      await tid("coach").waitFor({ timeout: 15_000 });
      const offer = (name) => page.locator(`[data-testid="offers"] [data-testid^="offer-"][data-name="${name}"]`);
      const keys = viewport.width >= 1024;
      const how = async (want, step) => {
        const got = (await page.locator('[data-testid="coach"] [data-testid="coach-how"]').innerText().catch(() => "")).replace(/\s+/g, " ").trim();
        if (got !== want) fail(`the coach's how-line at ${step} is ${JSON.stringify(got)}, expected ${JSON.stringify(want)}`);
      };
      await how(keys ? "Tap Gambler, then slot 1 · or press 1" : "Tap Gambler, then slot 1", "step 1");
      await offer("Gambler").click();
      await tid("line-0-empty").click();
      await page.waitForFunction(() => document.querySelector('[data-testid="coach"]')?.getAttribute("data-step") === "buy-second", null, { timeout: 5000 }).catch(() => fail("no step 2 after buying Gambler"));
      await how(keys ? "Tap Medic, then slot 2 · or press 2" : "Tap Medic, then slot 2", "step 2");
      await offer("Medic").click();
      await page.waitForTimeout(150);
      const lit = await page.evaluate(() => [...document.querySelectorAll('[data-testid="line"] .lit, [data-testid="bench"] .lit, [data-testid^="line-"].lit, [data-testid^="bench-"].lit')].map((e) => e.getAttribute("data-testid")));
      if (lit.length !== 1 || lit[0] !== "line-1-empty") fail(`slots lit for Medic: ${JSON.stringify(lit)}, expected only line-1-empty`);
      const want = await page.locator(".cell.want").count();
      if (want) fail(`${want} other slots glow "fill me" while the coach names one`);
      const before = seen.buys.length;
      await tid("line-0").click();
      await page.waitForTimeout(300);
      const said = (await tid("error").textContent())?.trim();
      if (said !== "Medic goes behind Gambler: slot 2") fail(`tapping slot 1 said ${JSON.stringify(said)}`);
      if (seen.buys.length !== before) fail("a tap on slot 1 bought Medic");
      await tid("line-1-empty").click();
      await page.waitForFunction(() => document.querySelector('[data-testid="coach"]')?.getAttribute("data-step") === "fight", null, { timeout: 5000 }).catch(() => fail("Medic in slot 2 does not lead to Now Fight"));
      await how(keys ? "Press Fight · or Space" : "Press Fight", "step 3");
      const line = await page.locator('[data-testid="line"] .pc[data-name]').evaluateAll((els) => els.map((e) => e.getAttribute("data-name")));
      if (line.join() !== "Gambler,Medic") fail(`the line is ${line.join()}`);
      await ctx.close();
    }
  }

  // 5. The coach with its how-line fits the small frames: no page scroll, nothing clipped, no text under 11 px as drawn.
  for (const [label, viewport] of [["800x450", { width: 800, height: 450 }], ["390x844", { width: 390, height: 844 }]]) {
    const { ctx, page, fail, tid } = await open(viewport, `${label} coach fit`, { delay: 0 });
    await page.goto(url, { timeout: 20_000 });
    await tid("start-play").click();
    await tid("coach").waitFor({ timeout: 15_000 });
    const offer = (name) => page.locator(`[data-testid="offers"] [data-testid^="offer-"][data-name="${name}"]`);
    const fit = (step) => page.evaluate(() => {
      const d = document.scrollingElement ?? document.documentElement;
      const how = document.querySelector('[data-testid="coach-how"]');
      const coach = document.querySelector('[data-testid="coach"]');
      const tray = coach?.closest(".tb-coach") ?? coach;
      const drawn = (e) => {
        const f = parseFloat(getComputedStyle(e).fontSize);
        let b = e;
        while (b && b !== document.body && (getComputedStyle(b).display === "inline" || b.offsetWidth < 24 || b.offsetHeight < 24)) b = b.parentElement;
        const r = b?.getBoundingClientRect();
        const k = b && b.offsetWidth > 0 && r.width > 0 ? r.width / b.offsetWidth : 1;
        return f * (k > 0 ? k : 1);
      };
      const small = [...(coach?.querySelectorAll("*") ?? [])].filter((e) => e.getClientRects().length && /^(SPAN|DIV|P|B|I|SMALL|STRONG)$/.test(e.tagName) && !e.closest("[aria-hidden='true']") && [...e.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim())).map((e) => `${e.textContent.trim().slice(0, 24)} (${drawn(e).toFixed(1)}px)`).filter((x) => parseFloat(x.slice(x.lastIndexOf("(") + 1)) < 10.9);
      const hr = how?.getBoundingClientRect(), tr = tray?.getBoundingClientRect();
      return { scroll: d.scrollHeight - d.clientHeight, hscroll: d.scrollWidth - d.clientWidth, small, how: !!hr && hr.width > 0 && hr.height > 0, clipped: !!hr && !!tr && (hr.bottom > tr.bottom + 1 || hr.top < tr.top - 1 || hr.bottom > innerHeight + 1 || hr.right > innerWidth + 1), coachScrolls: !!coach && coach.scrollHeight > coach.clientHeight + 1 };
    });
    const check = async (step) => {
      await mkdirSync("e2e/.shots/play-feedback", { recursive: true });
      await page.screenshot({ path: `e2e/.shots/play-feedback/coach-${label}-${step.replace(" ", "")}.png` });
      const m = await fit(step);
      if (m.scroll > 0 || m.hscroll > 0) fail(`${step}: the page scrolls (${m.scroll}px down, ${m.hscroll}px across)`);
      if (!m.how) fail(`${step}: no how-line on screen`);
      if (m.clipped || m.coachScrolls) fail(`${step}: the how-line is clipped`);
      if (m.small.length) fail(`${step}: text under 11 px in the coach: ${m.small.join(", ")}`);
    };
    await check("step 1");
    await offer("Gambler").click();
    await tid("line-0-empty").click();
    await page.waitForFunction(() => document.querySelector('[data-testid="coach"]')?.getAttribute("data-step") === "buy-second", null, { timeout: 5000 });
    await check("step 2");
    await offer("Medic").click();
    await tid("line-1-empty").click();
    await page.waitForFunction(() => document.querySelector('[data-testid="coach"]')?.getAttribute("data-step") === "fight", null, { timeout: 5000 });
    await check("step 3");
    await ctx.close();
  }

  // 4. A slow first load: the title and a loader are on screen before the script runs.
  {
    const { ctx, page, fail } = await open(SIZES[1][1], "slow load", { delay: 0 });
    await page.route(/\/assets\/.*\.js$/, async (route) => { await new Promise((r) => setTimeout(r, 4000)); await route.continue(); });
    await page.goto(url, { waitUntil: "commit", timeout: 20_000 });
    const t0 = Date.now();
    await page.getByTestId("boot-loader").waitFor({ timeout: 1000 }).catch(() => fail("no loader on screen while the page loads"));
    const box = await page.evaluate(() => { const e = document.querySelector('[data-testid="boot-loader"]'); const r = e?.getBoundingClientRect(); return { shown: !!r && r.width > 0 && r.height > 0, text: e?.textContent ?? "", bg: getComputedStyle(document.body).backgroundColor }; });
    if (!box.shown || !/ARENA OF IDEAS/i.test(box.text)) fail(`the loader is ${JSON.stringify(box)}`);
    if (box.bg === "rgba(0, 0, 0, 0)" || box.bg === "rgb(255, 255, 255)") fail(`a white page while loading (${box.bg})`);
    if (Date.now() - t0 > 1000) fail("the loader took over 1 s");
    await page.getByTestId("start").waitFor({ timeout: 20_000 }).catch(() => fail("the first screen never replaced the loader"));
    await ctx.close();
  }
} catch (e) {
  errors.push(`${e instanceof Error ? e.stack : e}`);
} finally {
  await browser.close();
  child?.kill();
}
if (errors.length) {
  console.error(errors.map((e) => `FAIL ${e}`).join("\n"));
  process.exit(1);
}
console.log("mvp play-feedback: ok");
