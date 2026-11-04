// Instant feedback on the table (arena#28 reopened, mission #26): the audit
// screenshots a few frames after a press, and a request slower than ~100 ms
// read as "nothing happened". Every decision API call here is delayed 1500 ms
// by a route handler; each press must show its result within 100 ms, before the
// answer: a buy draws the unit in its slot as bought (gold down, the offer gone),
// Fight starts its transition, Reroll, Lock and sell show their result or a
// pressed state, an offer's inspector reads its rule at once (never "…"), and
// Space / 1–5 keep working with an inspector open. Also: no text under 11 px
// as drawn at 800×450 with the optimistic states showing.
// Without --url it builds the client and starts the server (rules v2's pool).
//   npm run mvp:feedback -- [--url http://127.0.0.1:PORT/arena/] [--out e2e/.shots/mvp-feedback]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/mvp-feedback";
mkdirSync(out, { recursive: true });

const DELAY = 1500;
/** What "at once" means: the result is on screen this soon after the press. */
const WITHIN = 100;

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

// (label, viewport, how Fight is pressed): the desktop uses keys, the phone and the CG frame taps and clicks.
const SIZES = [
  ["1280x720", { width: 1280, height: 720 }, "key"],
  ["800x450", { width: 800, height: 450 }, "click"],
  ["390x844", { width: 390, height: 844 }, "click"],
];
const browser = await launchChromium();
const errors = [];
try {
  for (const [label, viewport, fightBy] of SIZES) {
    const ctx = await browser.newContext({ viewport, hasTouch: viewport.width < 800, isMobile: viewport.width < 800 });
    const page = await ctx.newPage();
    const fail = (m) => errors.push(`${label}: ${m}`);
    page.on("pageerror", (e) => fail(`pageerror: ${e.message}`));
    // The one 409 is the refused buy this run provokes on purpose.
    page.on("console", (m) => m.type() === "error" && !/status of 409/.test(m.text()) && fail(`console: ${m.text()}`));
    const tid = (id) => page.getByTestId(id);
    const desk = viewport.width >= 1024;
    let delaying = false;
    let out_ = 0; // requests held by the route handler right now
    let refuseNext = false; // the next decision is refused (409) after the delay
    await page.route(/\/runs\/[^/]+\/(decisions|preview)$/, async (route) => {
      if (!delaying) return route.continue();
      out_++;
      await new Promise((r) => setTimeout(r, DELAY));
      out_--;
      if (refuseNext && route.request().method() === "POST" && /decisions$/.test(route.request().url())) {
        refuseNext = false;
        return route.fulfill({ status: 409, contentType: "application/json", body: JSON.stringify({ error: "costs 3, have 0" }) }).catch(() => {});
      }
      await route.continue().catch(() => {});
    });
    /** The table as drawn now. */
    const snap = () =>
      page.evaluate(() => {
        const num = (s) => Number((s ?? "").replace(/\D/g, ""));
        const insp = document.querySelector('[data-testid="inspector"]');
        return {
          gold: num(document.querySelector('[data-testid="gold"]')?.textContent),
          line: [...document.querySelectorAll('[data-testid^="line-"].cell.you')].map((c) => `${c.dataset.testid}:${c.querySelector(".pc")?.dataset.name}`),
          offers: [...document.querySelectorAll('[data-testid^="offer-"].cell:not(.offer-gap)')].map((c) => c.dataset.name),
          locked: [...document.querySelectorAll('[data-testid^="offer-"].cell.locked')].map((c) => c.dataset.name),
          fighting: !!document.querySelector(".stage.fighting"),
          banner: !!document.querySelector('[data-testid="fight-banner"]'),
          fightPressed: !!document.querySelector('[data-testid="fight"].pressed'),
          rolling: !!document.querySelector('[data-testid="offers"].rolling'),
          rerollPressed: !!document.querySelector('[data-testid="reroll"].pressed'),
          inspector: !!insp && !insp.hidden,
          inspText: insp && !insp.hidden ? insp.textContent.trim() : "",
        };
      });
    /** Presses, then checks the table WITHIN ms later, while the delayed answer is still out. */
    const at = async (what, press, ok) => {
      await press();
      const t0 = Date.now();
      const s = await snap();
      const took = Date.now() - t0;
      if (!ok(s)) {
        fail(`${what}: not shown within ${WITHIN} ms (${JSON.stringify(s)})`);
        await page.screenshot({ path: `${out}/fail-${label}-${what.replace(/\W+/g, "-")}.png` });
      } else if (took > WITHIN) fail(`${what}: the read took ${took} ms`);
      if (delaying && out_ === 0 && !s.fighting) fail(`${what}: no request was out, so the check proved nothing`);
      return s;
    };
    /** The delayed answer arrives and the table settles. */
    const settled = async () => {
      await page.waitForFunction(() => !document.getElementById("app").classList.contains("busy"), null, { timeout: DELAY * 4 });
      await page.waitForTimeout(150);
    };
    const offer = (name) => page.locator(`[data-testid="offers"] [data-testid^="offer-"][data-name="${name}"]`);
    const placeAt = async (slot) => (desk ? page.keyboard.press(String(slot + 1)) : tid(`line-${slot}-empty`).click());

    await page.goto(url, { timeout: 20_000 });
    await tid("start-play").waitFor({ timeout: 10_000 });
    await page.keyboard.press("Enter").catch(() => {});
    if (!(await tid("fight").isVisible().catch(() => false))) await tid("start-play").click().catch(() => {});
    await tid("fight").waitFor({ timeout: 15_000 });
    await page.waitForTimeout(500);
    delaying = true;
    const g0 = (await snap()).gold;

    // 1. Tap an offer, then its slot: the unit is in the slot as bought, gold down, the offer gone.
    await offer("Gambler").click();
    await at("buy: Gambler into slot 1", () => placeAt(0), (s) => s.line.some((l) => l.startsWith("line-0:Gambler")) && s.gold === g0 - 3 && !s.offers.includes("Gambler"));
    await page.screenshot({ path: `${out}/buy-drawn-${label}.png` }); // the answer is still 1.5 s away
    await settled();

    // 5. An offer's inspector reads its rule at once, never "…" (its preview is still out).
    await at("inspector of an offer", () => offer("Medic").click(), (s) => s.inspector && /Medic/.test(s.inspText) && !/^…$/.test(s.inspText) && /\d/.test(s.inspText) && s.inspText.length > 40);
    await page.screenshot({ path: `${out}/offer-inspector-${label}.png` });
    // 4. 1–5 place the offer in hand (the inspector is open); the unit shows at once.
    const g1 = (await snap()).gold;
    await at("buy: Medic into slot 2", () => placeAt(1), (s) => s.line.some((l) => l.startsWith("line-1:Medic")) && s.gold === g1 - 3 && !s.offers.includes("Medic"));
    await settled();

    // 1. A buy the server refuses is taken back: the offer returns, the slot empties, the gold is back, with the reason.
    {
      const before = await snap();
      const name = before.offers[0];
      if (name) {
        await offer(name).click();
        refuseNext = true;
        await at("refused buy, drawn first", () => placeAt(2), (s) => s.line.some((l) => l.startsWith(`line-2:${name}`)) && s.gold === before.gold - 3);
        await settled();
        const back = await snap();
        const msg = await page.getByTestId("error").textContent().catch(() => "");
        if (back.gold !== before.gold || back.line.some((l) => l.startsWith("line-2:")) || !back.offers.includes(name)) fail(`a refused buy isn't taken back (${JSON.stringify(back)})`);
        if (!msg || !/\d/.test(msg)) fail(`a refused buy says nothing (error line: "${msg}")`);
      }
    }

    // 3. Lock: the mark shows at once (an offer taken in hand, Lock in its inspector).
    const rest = (await snap()).offers;
    if (rest.length) {
      await offer(rest[0]).click();
      await tid("lock").waitFor({ timeout: 3_000 });
      await at("lock", () => tid("lock").click(), (s) => s.locked.includes(rest[0]));
      await settled();
    } else fail("no offer left to lock");

    // 3. Reroll: gold down and the offers rolling (or the button pressed), before the answer.
    const g2 = (await snap()).gold;
    if (g2 >= 1) await at("reroll", () => tid("reroll").click(), (s) => s.gold === g2 - 1 && (s.rolling || s.rerollPressed));
    await page.screenshot({ path: `${out}/reroll-pressed-${label}.png` });
    await settled();

    // 3. Sell: the unit is gone and the gold is back, at once.
    const g3 = (await snap()).gold;
    await tid("line-1").click();
    await tid("sell").waitFor({ timeout: 3_000 });
    await at("sell", () => tid("sell").click(), (s) => !s.line.some((l) => l.startsWith("line-1:")) && s.gold > g3);
    await settled();
    if (label === "800x450") {
      const small = await page.evaluate(() => {
        const k = Number(getComputedStyle(document.querySelector(".stage")).getPropertyValue("--k")) || 1;
        return [...new Set([...document.querySelectorAll("#app *")].filter((e) => [...e.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim()) && e.getClientRects().length && parseFloat(getComputedStyle(e).fontSize) * k < 10.95).map((e) => `${e.className || e.tagName}:${(parseFloat(getComputedStyle(e).fontSize) * k).toFixed(1)}`))];
      });
      if (small.length) fail(`text under 11px at 800×450: ${small.slice(0, 6).join(", ")}`);
    }

    // 2. Fight: pressed, and the table starts its transition, before the answer.
    if (desk) await tid("line-0").click(); // a unit's inspector is open: Space still fights
    const press = fightBy === "key" ? () => page.keyboard.press(" ") : () => tid("fight").click();
    const s = await at(`fight by ${fightBy}`, press, (s) => s.fightPressed && s.fighting && s.banner);
    await page.screenshot({ path: `${out}/fight-pressed-${label}.png` });
    if (!s.fighting) fail("fight: no transition");
    // The answer comes: the battle (or its end card) draws, with nothing left stuck.
    await page.getByTestId("battle-end").or(tid("end-card")).first().waitFor({ timeout: DELAY * 4 }).catch(() => fail("fight: the battle never drew after the delayed answer"));
    if (!errors.some((e) => e.startsWith(`${label}:`))) console.log(`${label}: feedback within ${WITHIN} ms on buy (and a refused one), inspector, keys, lock, reroll, sell, fight`);
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
console.log("mvp feedback: ok");
