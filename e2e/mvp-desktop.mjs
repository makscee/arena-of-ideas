// MVP desktop e2e (round 2, R2-9): a whole run at 1440×900 in Chromium, with
// mouse and keyboard only (no taps), and a screenshot of every screen. Checks
// the desktop layout: the top bar, the wide board with 132×172 cards, the
// inspector on the right that reads the hovered or selected card (no pop-up
// sheet in the shop), drag to reorder and the keys (1–7 buy, R reroll, Space
// fight, ← → move, S sell, Esc back). Without --url it builds the mobile
// client and starts the MVP server on a free port.
//   npm run mvp:desktop -- [--url https://m1.twin-pogona.ts.net/arena/] [--out e2e/.shots/mvp-desktop]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const W = 1440;
const H = 900;
const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/mvp-desktop";
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
  for (let i = 0; i < 50; i++) {
    try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
    await new Promise((r) => setTimeout(r, 200));
  }
}

const browser = await launchChromium();
const errors = [];
let shots = 0;
try {
  const page = await browser.newPage({ viewport: { width: W, height: H } });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && errors.push(`console: ${m.text()}`));
  const shot = (name) => page.screenshot({ path: `${out}/${String(++shots).padStart(2, "0")}-${name}.png` });
  const noHScroll = async (name) => {
    const w = await page.evaluate(() => document.documentElement.scrollWidth);
    if (w > W) errors.push(`${name}: horizontal scroll (${w}px)`);
  };
  /** The screen uses the width: its content spans at least `min` px. */
  const wide = async (name, min) => {
    const w = await page.evaluate(() => document.getElementById("app").getBoundingClientRect().width);
    if (w < min) errors.push(`${name}: the screen is ${Math.round(w)}px wide, want ≥ ${min}`);
  };
  /** Desktop cards are 132×172 at this width. */
  const cardSize = async (name, testid) => {
    const box = await page.getByTestId(testid).locator(".card").first().boundingBox();
    if (!box || Math.abs(box.width - 132) > 1 || Math.abs(box.height - 172) > 1) errors.push(`${name}: card ${box ? `${Math.round(box.width)}×${Math.round(box.height)}` : "missing"}, want 132×172`);
  };
  /** Whole on screen, no scrolling. */
  const onScreen = async (name, locator) => {
    const box = await locator.boundingBox();
    if (!box || box.y < 0 || box.y + box.height > H + 0.5 || box.x + box.width > W + 0.5) errors.push(`${name}: off screen`);
  };
  const noOverlay = async (name) => {
    if (await page.getByTestId("overlay").count()) errors.push(`${name}: a pop-up sheet in the shop (the inspector reads cards on desktop)`);
  };
  const gold = async () => Number(((await page.getByTestId("gold").textContent().catch(() => "0")) ?? "0").replace("g", ""));
  const lineCount = () => page.getByTestId("line").locator(".card.you").count();
  /** Waits until the shop is drawn again after a decision (the gold or the line changed). */
  const settle = async () => { await page.waitForTimeout(150); await page.waitForFunction(() => !document.getElementById("app").classList.contains("busy")); };

  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").waitFor({ timeout: 10_000 });
  await page.keyboard.type("DeskTester");
  await page.keyboard.press("Enter");
  await page.getByTestId("play").waitFor();
  await shot("home"); await noHScroll("home"); await wide("home", 1100); await onScreen("home: Play", page.getByTestId("play"));
  await cardSize("home champion", "champion");
  // Home's two columns: Play sits right of the champion panel.
  const [champBox, playBox] = [await page.getByTestId("champion").boundingBox(), await page.getByTestId("play").boundingBox()];
  if (!(playBox.x > champBox.x + champBox.width)) errors.push("home: Play isn't in the right column");

  await page.getByTestId("stats").click();
  await page.getByTestId("stats-back").waitFor();
  await shot("stats"); await noHScroll("stats"); await wide("stats", 900);
  await page.getByTestId("stats-back").click();
  await page.getByTestId("play").click();

  let round = 0;
  let dragged = false;
  let sold = false;
  let resultShot = false;
  for (let guard = 0; guard < 20; guard++) {
    await page.getByTestId("fight").waitFor({ timeout: 10_000 });
    round++;
    const crown = (await page.getByTestId("gold").count()) === 0;
    if (round === 1) {
      await shot("shop"); await noHScroll("shop"); await wide("shop", W - 1);
      await cardSize("shop offers", "offers");
      await onScreen("shop: Fight", page.getByTestId("fight"));
      await onScreen("shop: inspector", page.getByTestId("inspector"));
      await onScreen("shop: last offer", page.getByTestId("offers").locator(".card").last());
      // Hover reads an offer in the inspector, with its Buy.
      await page.getByTestId("offers").locator(".card").first().hover();
      await page.getByTestId("inspector").getByTestId("unit-sheet").waitFor();
      await page.getByTestId("inspector").getByTestId("buy").waitFor();
      await shot("shop-hover-offer"); await noOverlay("hover");
      // The inspector is right of the board.
      const [boardOffer, insp] = [await page.getByTestId("offers").boundingBox(), await page.getByTestId("inspector").boundingBox()];
      if (!(insp.x >= boardOffer.x + boardOffer.width)) errors.push("shop: the inspector isn't right of the board");
      if (Math.abs(insp.width - 380) > 2) errors.push(`shop: inspector ${Math.round(insp.width)}px wide, want 380`);
      // Double-click buys the first offer.
      const before = await lineCount();
      await page.getByTestId("offers").locator(".card").first().dblclick();
      await settle();
      if ((await lineCount()) !== before + 1) errors.push("shop: double-click didn't buy");
    }
    // Buy with the number keys while gold lasts (and the line has room).
    for (let k = 0; k < 4 && !crown; k++) {
      const g = await gold();
      const offers = await page.getByTestId("offers").locator(".card").count();
      if (g < 3 || offers === 0) break;
      const n = await lineCount();
      if (n >= 5) {
        // Line full: once, select the back unit and sell it with S.
        if (sold) break;
        await page.getByTestId("line-4").click();
        await page.getByTestId("inspector").getByTestId("sell").waitFor();
        await shot("shop-selected-full");
        await page.keyboard.press("s");
        await settle();
        if ((await lineCount()) !== 4) errors.push("S didn't sell the selected unit");
        sold = true;
        continue;
      }
      const g0 = g;
      await page.keyboard.press("1");
      await settle();
      if ((await gold()) === g0) { errors.push(`round ${round}: key 1 didn't buy`); break; }
    }
    if (round === 1 && !crown && (await gold()) >= 1) {
      // R rerolls.
      const g0 = await gold();
      await page.keyboard.press("r");
      await settle();
      if ((await gold()) !== g0 - 1) errors.push("R didn't reroll");
    }
    if (!crown && !dragged && (await lineCount()) >= 2) {
      dragged = true;
      // Drag the front unit onto the second slot: they swap places.
      const name0 = await page.getByTestId("line-0").locator(".name").textContent();
      await page.getByTestId("line-0").dragTo(page.getByTestId("line-1"));
      await settle();
      if ((await page.getByTestId("line-1").locator(".name").textContent()) !== name0) errors.push("drag didn't reorder the line");
      // The dragged unit stays selected; the inspector holds its sheet and actions; ← moves it back, still selected.
      if (!(await page.getByTestId("line-1").evaluate((el) => el.classList.contains("selected")))) {
        errors.push("the dragged unit isn't selected");
        await page.getByTestId("line-1").click();
      }
      await page.getByTestId("inspector").getByTestId("move-left").waitFor();
      await shot("shop-selected"); await noOverlay("selected");
      await page.keyboard.press("ArrowLeft");
      await settle();
      if ((await page.getByTestId("line-0").locator(".name").textContent()) !== name0) errors.push("← didn't move the selected unit");
      if (!(await page.getByTestId("line-0").evaluate((el) => el.classList.contains("selected")))) errors.push("the moved unit isn't selected anymore");
      // Esc deselects.
      await page.keyboard.press("Escape");
      if (await page.locator('[data-testid="line"] .card.selected').count()) errors.push("Esc didn't deselect");
    }
    if (crown) { await shot("crown-shop"); await noHScroll("crown-shop"); }
    // Space fights.
    await page.keyboard.press("Space");
    await page.getByTestId("battle-skip").waitFor({ timeout: 10_000 });
    if (round === 1) await shot("battle");
    await page.getByTestId("battle-skip").click();
    await page.getByTestId("outcome").waitFor({ timeout: 10_000 });
    if (!resultShot) {
      resultShot = true;
      await shot("result"); await noHScroll("result"); await wide("result", 1100);
      await onScreen("result: Next round", page.getByTestId("continue"));
    } else if (await page.getByTestId("why-lost").isVisible().catch(() => false)) {
      await shot(`result-loss-r${round}`);
    }
    // Enter moves on.
    await page.keyboard.press("Enter");
    await page.waitForFunction(() => document.querySelector('[data-testid="fight"]') || document.querySelector('[data-testid="run-over"]'));
    if (await page.getByTestId("run-over").isVisible().catch(() => false)) break;
  }
  await page.getByTestId("run-over").waitFor({ timeout: 10_000 });
  await shot("run-over"); await noHScroll("run-over"); await wide("run-over", 1100);
  await page.keyboard.press("Enter");
  await page.getByTestId("play").waitFor();
  await shot("home-after");
  if (!dragged) errors.push("never had two units to drag");
  if (round < 2) errors.push(`only ${round} fights`);

  // 1024px is still desktop (120px cards, 340px inspector, nothing cut off); 1023px is the phone.
  await page.setViewportSize({ width: 1024, height: 768 });
  await page.getByTestId("play").click();
  await page.getByTestId("fight").waitFor();
  await shot("shop-1024");
  if (!(await page.getByTestId("inspector").count())) errors.push("1024px: no inspector");
  const w1024 = await page.evaluate(() => document.documentElement.scrollWidth);
  if (w1024 > 1024) errors.push(`1024px: horizontal scroll (${w1024}px)`);
  const line1024 = await page.getByTestId("line").locator(".card").evaluateAll((els) => new Set(els.map((e) => Math.round(e.getBoundingClientRect().top))).size);
  if (line1024 !== 1) errors.push("1024px: the line wraps");
  await page.setViewportSize({ width: 1023, height: 768 });
  await page.waitForTimeout(200);
  if (await page.getByTestId("inspector").count()) errors.push("1023px: the inspector shows on the phone layout");
  await shot("shop-1023");
  console.log(`mvp desktop: ${round} fights, ${shots} screenshots in ${out}`);
} finally {
  await browser.close();
  child?.kill();
}
if (errors.length) { console.error(errors.join("\n")); process.exit(1); }
