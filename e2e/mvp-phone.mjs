// MVP phone e2e (mission #574 verification ladder): a whole game at 360×640
// in Chromium, with a screenshot of every screen. Without --url it builds the
// mobile client and starts the MVP server on a free port.
//   npm run mvp:phone -- [--url https://m1.twin-pogona.ts.net/arena/] [--out e2e/.shots/mvp]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/mvp";
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1" }, stdio: ["ignore", "inherit", "inherit"] });
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
  const page = await browser.newPage({ viewport: { width: 360, height: 640 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && errors.push(`console: ${m.text()}`));
  const shot = async (name) => { await page.screenshot({ path: `${out}/${String(++shots).padStart(2, "0")}-${name}.png` }); };
  const noHScroll = async (name) => {
    const w = await page.evaluate(() => document.documentElement.scrollWidth);
    if (w > 360) errors.push(`${name}: horizontal scroll (${w}px)`);
  };

  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").waitFor({ timeout: 10_000 });
  await shot("name"); await noHScroll("name");
  await page.getByTestId("name-input").fill("PhoneTester");
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").waitFor();
  await shot("home"); await noHScroll("home");
  await page.getByTestId("play").click();

  let round = 0;
  for (let guard = 0; guard < 20; guard++) {
    await page.getByTestId("fight").waitFor({ timeout: 10_000 });
    // Buy while the gold allows and the line has room.
    for (let k = 0; k < 4; k++) {
      const gold = Number((await page.getByTestId("gold").textContent()).replace("g", ""));
      if (gold < 3 || (await page.getByTestId("offers").locator(".card").count()) === 0) break;
      const filled = await page.getByTestId("line").locator(".card.you").count();
      if (filled >= 5) break;
      await page.getByTestId("offer-0").click();
      await page.waitForFunction((g) => !document.querySelector('[data-testid="gold"]') || document.querySelector('[data-testid="gold"]').textContent !== `${g}g`, gold);
    }
    round++;
    if (round === 1) { await shot("shop"); await noHScroll("shop"); }
    if (round === 2 && (await page.getByTestId("line").locator(".card.you").count()) > 1) {
      await page.getByTestId("line-1").click();
      await shot("shop-selected");
      await page.getByTestId("move-left").click();
      await page.getByTestId("fight").waitFor();
    }
    await page.getByTestId("fight").click();
    // The battle screen (slice 9) comes first; its skip button leads to the result.
    await page.locator('[data-testid="outcome"], [data-testid="battle-skip"]').first().waitFor({ timeout: 10_000 });
    if (await page.getByTestId("battle-skip").isVisible()) await page.getByTestId("battle-skip").click();
    await page.getByTestId("outcome").waitFor({ timeout: 10_000 });
    if (round === 1) { await shot("result"); await noHScroll("result"); }
    await page.getByTestId("continue").click();
    if (await page.getByTestId("run-over").isVisible().catch(() => false)) break;
  }
  await page.getByTestId("run-over").waitFor({ timeout: 10_000 });
  await shot("run-over"); await noHScroll("run-over");
  await page.getByTestId("home").click();
  await page.getByTestId("play").waitFor();
  await shot("home-after");
  console.log(`mvp phone: ${round} fights, ${shots} screenshots in ${out}`);
} finally {
  await browser.close();
  child?.kill();
}
if (errors.length) { console.error(errors.join("\n")); process.exit(1); }
