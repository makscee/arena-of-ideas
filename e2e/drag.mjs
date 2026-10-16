// Drag and drop at every size (void-board#1285): pointer events, mouse and touch.
// Drag an offer onto a line slot (buys it there), a line card onto the bench
// and another line slot, an offer onto its owned twin (merges), a unit onto the
// sell zone. At 800×450 (the CrazyGames frame) and 390×844 with real touch
// events (CDP), and 1280×720 and 821×462 with a mouse. A tap still opens/selects.
// Without --url it builds the client and starts its own scenes server.
//   npm run mvp:drag -- [--url http://127.0.0.1:PORT/arena/]
import { spawn, execFileSync } from "node:child_process";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_TUNER: "instant", ARENA_CODEX: "1", ARENA_CG: "1" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

const SIZES = [
  { name: "800x450 touch", viewport: { width: 800, height: 450 }, touch: true },
  { name: "390x844 touch", viewport: { width: 390, height: 844 }, touch: true },
  { name: "821x462 mouse", viewport: { width: 821, height: 462 }, touch: false },
  { name: "1280x720 mouse", viewport: { width: 1280, height: 720 }, touch: false },
];
const failures = [];
const browser = await launchChromium();
try {
  for (const v of SIZES) {
    const ctx = await browser.newContext({ viewport: v.viewport, isMobile: v.touch, hasTouch: v.touch, deviceScaleFactor: 1 });
    const page = await ctx.newPage();
    const cdp = v.touch ? await ctx.newCDPSession(page) : null;
    const why = [];
    const fail = (m) => why.push(m);
    page.on("pageerror", (e) => fail(`pageerror: ${e.message}`));
    const centre = async (sel) => {
      const el = page.locator(sel).first();
      await el.scrollIntoViewIfNeeded();
      const b = await el.boundingBox();
      if (!b) throw new Error(`${sel} not on screen`);
      return { x: b.x + b.width / 2, y: b.y + b.height / 2 };
    };
    /** Press on `from`, move in steps to `to`, release: touch events on a phone, mouse on desktop. */
    /** What the shop shows (board text + gold): a drop is done when it changes. */
    const sig = () => page.evaluate(() => document.querySelector('[data-testid="line"]')?.parentElement?.textContent + document.querySelector('[data-testid="gold"]')?.textContent);
    const drag = async (fromSel, toSel) => {
      const before = await sig();
      // Scroll the target in, then the source, then measure both (a scroll moves them).
      if (toSel !== "sell") await centre(toSel);
      await centre(fromSel);
      const a = await centre(fromSel);
      const dest = toSel === "sell" ? { x: v.viewport.width / 2, y: v.viewport.height - 20 } : await centre(toSel);
      const steps = 12;
      const pt = (x, y) => [{ x, y, id: 1 }];
      if (cdp) {
        await cdp.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: pt(a.x, a.y) });
        for (let i = 1; i <= steps; i++) { await cdp.send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: pt(a.x + (dest.x - a.x) * i / steps, a.y + (dest.y - a.y) * i / steps) }); await page.waitForTimeout(15); }
        await cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
      } else {
        await page.mouse.move(a.x, a.y);
        await page.mouse.down();
        await page.mouse.move(a.x + 10, a.y + 10, { steps: 2 });
        await page.mouse.move(dest.x, dest.y, { steps });
        await page.mouse.up();
      }
      // A drop may take a moment (a buy waits for its preview, then reorders).
      for (let i = 0; i < 40 && (await sig()) === before; i++) await page.waitForTimeout(100);
      await page.waitForTimeout(400);
    };
    const tap = async (sel) => {
      const a = await centre(sel);
      if (cdp) {
        await cdp.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x: a.x, y: a.y, id: 1 }] });
        await cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
      } else await page.mouse.click(a.x, a.y);
      await page.waitForTimeout(400);
    };
    const count = (z) => page.locator(`[data-testid^="${z}-"]:not([data-testid$="-empty"])`).count();
    const gold = async () => Number(((await page.locator('[data-testid="gold"]').first().textContent()) ?? "").replace(/\D/g, ""));
    const open = async (scene) => {
      await page.goto("about:blank");
      await page.goto(`${url}#scene=${scene}&lang=en`, { timeout: 120_000 });
      await page.waitForSelector('[data-testid="fight"]', { timeout: 60_000 });
      await page.waitForTimeout(2500);
    };
    const expect = (what, got, want) => { if (got !== want) fail(`${what}: got ${got}, want ${want}`); };
    try {
      // A tap still selects a line card (no drag started by a still press).
      await open("shop-r3");
      if ((await count("line")) === 0) fail("shop-r3 has no line unit");
      await tap('[data-testid="line-0"]');
      if (!(await page.locator('[data-testid="line-0"].selected').count())) fail("tap on line-0 did not select it");
      if (await page.locator(".drag-ghost").count()) fail("a tap left a drag ghost");

      // Offer → an empty slot on the line (buys it there).
      await open("shop-r3");
      const lineN = await count("line"), benchN = await count("bench"), g0 = await gold();
      const free = lineN < 5 ? `[data-testid="line-${lineN}-empty"]` : `[data-testid="bench-${benchN}-empty"]`;
      const aff = page.locator('[data-testid^="offer-"]:not(.poor)').first();
      if (!(await aff.count())) fail("no affordable offer in shop-r3");
      else {
        await drag(`[data-testid="${await aff.getAttribute("data-testid")}"]`, free);
        expect("offer → free slot: units", (await count("line")) + (await count("bench")), lineN + benchN + 1);
        if ((await gold()) >= g0) fail("offer → free slot: gold did not drop");
      }

      // Line card → bench (empty slot), then line slot 0 → another line slot (a reorder).
      await open("shop-r3");
      const l1 = await count("line"), b1 = await count("bench");
      await drag('[data-testid="line-0"]', `[data-testid="bench-0-empty"], [data-testid="bench-0"]`);
      expect("line → bench: bench", await count("bench"), Math.min(b1 + 1, b1 + (b1 ? 0 : 1)));
      if (b1 === 0) expect("line → bench: line", await count("line"), l1 - 1);
      if ((await count("line")) >= 2) {
        const name0 = await page.locator('[data-testid="line-0"]').first().textContent();
        await drag('[data-testid="line-0"]', '[data-testid="line-1"]');
        const name1 = await page.locator('[data-testid="line-1"]').first().textContent();
        if (name0 !== name1) fail("line-0 → line-1: the unit did not move to slot 2");
      }

      // Offer → its owned twin merges (no new unit, gold paid).
      await open("shop-merge-hint");
      const units = (await count("line")) + (await count("bench")), g2 = await gold();
      const twin = page.locator('[data-testid^="offer-"].owned').first();
      if (!(await twin.count())) fail("shop-merge-hint has no owned offer");
      else {
        const offerName = (await twin.locator(".name").first().textContent())?.trim();
        const target = page.locator(`[data-testid^="line-"]:not([data-testid$="-empty"]), [data-testid^="bench-"]:not([data-testid$="-empty"])`).filter({ hasText: offerName ?? "" }).first();
        const tid = await target.getAttribute("data-testid");
        await drag(`[data-testid="${await twin.getAttribute("data-testid")}"]`, `[data-testid="${tid}"]`);
        expect("offer → twin: units", (await count("line")) + (await count("bench")), units);
        if ((await gold()) >= g2) fail("offer → twin: gold did not drop");
      }

      // A unit → the sell zone.
      await open("shop-r3");
      const u3 = (await count("line")) + (await count("bench")), g3 = await gold();
      await drag('[data-testid="line-0"]', "sell");
      expect("sell: units", (await count("line")) + (await count("bench")), u3 - 1);
      if ((await gold()) <= g3) fail("sell: gold did not rise");
      if (await page.locator(".sell-zone, .drag-ghost").count()) fail("sell zone/ghost left on screen after the drop");
    } catch (e) {
      fail(`threw: ${String(e.message).split("\n")[0]}`);
    }
    console.log(`${why.length ? "FAIL" : "ok  "} ${v.name}${why.length ? ": " + why.join("; ") : ""}`);
    failures.push(...why.map((w) => `${v.name}: ${w}`));
    await ctx.close();
  }
} finally {
  await browser.close();
  child?.kill();
}
process.exit(failures.length ? 1 : 0);
