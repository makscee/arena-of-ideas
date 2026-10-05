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
  await page.getByTestId("rules-open").click();
  await page.getByTestId("rules").waitFor();
  await shot("rules"); await noHScroll("rules");
  await page.getByTestId("overlay").click({ position: { x: 180, y: 10 } });
  await page.getByTestId("rules").waitFor({ state: "detached" });
  await page.getByTestId("stats").click();
  await page.getByTestId("stats-back").waitFor();
  await shot("stats"); await noHScroll("stats");
  await page.getByTestId("stats-back").click();
  await page.getByTestId("play").waitFor();
  await page.getByTestId("play").click();

  let round = 0;
  let whyShot = false;
  for (let guard = 0; guard < 20; guard++) {
    await page.getByTestId("fight").waitFor({ timeout: 10_000 });
    // Buy while the gold allows and the line has room.
    for (let k = 0; k < 4; k++) {
      const gold = Number((await page.getByTestId("gold").textContent()).replace("g", ""));
      if (gold < 3 || (await page.getByTestId("offers").locator(".card").count()) === 0) break;
      const filled = await page.getByTestId("line").locator(".card.you").count();
      if (filled >= 5) break;
      // An offer opens its sheet (both forms, what buying does); Buy buys.
      await page.getByTestId("offer-0").click();
      await page.getByTestId("buy").waitFor();
      if (round === 0 && k === 0) { await shot("offer-sheet"); await noHScroll("offer-sheet"); }
      if (await page.getByTestId("buy").isDisabled()) { await page.getByTestId("offer-close").click(); break; }
      await page.getByTestId("buy").click();
      await page.waitForFunction((g) => !document.querySelector('[data-testid="gold"]') || document.querySelector('[data-testid="gold"]').textContent !== `${g}g`, gold);
    }
    round++;
    if (round === 1) { await shot("shop"); await noHScroll("shop"); }
    if (round === 2 && (await page.getByTestId("line").locator(".card.you").count()) > 1) {
      await page.getByTestId("line-1").click();
      await shot("shop-selected");
      await page.getByTestId("info").click();
      await page.getByTestId("unit-sheet").waitFor();
      await shot("unit-sheet"); await noHScroll("unit-sheet");
      await page.getByTestId("overlay").click({ position: { x: 180, y: 10 } });
      await page.getByTestId("champion-pin-open").click().catch(() => {});
      if (await page.getByTestId("overlay").isVisible().catch(() => false)) { await shot("champion-pin"); await page.getByTestId("overlay").click({ position: { x: 180, y: 10 } }); }
      await page.getByTestId("move-left").click();
      await page.getByTestId("fight").waitFor();
    }
    await page.getByTestId("fight").click();
    // The battle viewer (slice 9) comes first. In round 1, watch it play, then
    // tap a change and read its chain. Skip goes to the result, which shows
    // "why I lost" after a loss (shot once).
    await page.getByTestId("battle-skip").waitFor({ timeout: 10_000 });
    if (round === 1) {
      await page.getByTestId("change").first().waitFor({ timeout: 15_000 });
      await shot("battle"); await noHScroll("battle");
      await page.getByTestId("change").first().click();
      await page.getByTestId("trace-text").waitFor();
      const chain = await page.getByTestId("trace-text").textContent();
      if (!/←/.test(chain)) errors.push(`trace: no chain in "${chain}"`);
      await shot("battle-trace"); await noHScroll("battle-trace");
    }
    await page.getByTestId("battle-skip").click();
    await page.getByTestId("outcome").waitFor({ timeout: 10_000 });
    if (!whyShot && (await page.getByTestId("why-lost").isVisible())) { whyShot = true; await shot("why-lost"); await noHScroll("why-lost"); }
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

  // Awakening and fusion (slice 8): a second player plays through the API
  // until it has one Awoken unit, a second unit one copy short and that copy
  // in the shop, then the phone buys it (the awaken preview) and fuses the two
  // (the fusion preview and the fused card).
  const call = async (method, path, body, pid) => {
    const res = await fetch(new URL(`api/v1${path}`, url), { method, headers: { "content-type": "application/json", ...(pid ? { "X-Arena-Player": pid } : {}) }, ...(body ? { body: JSON.stringify(body) } : {}) });
    const json = await res.json();
    if (!res.ok) throw new Error(`${method} ${path}: ${json.error}`);
    return json;
  };
  const fuser = await call("POST", "/players", { name: "Fuser" });
  let run = await call("POST", "/runs", undefined, fuser.id);
  const ready = (r) => {
    const awake = r.line.filter((u) => u.kind === "unit" && u.form === "awoken");
    const almost = r.line.find((u) => u.kind === "unit" && u.form === "sleeping" && u.copies === 2 && r.offers.some((o) => o.unitId === u.unitId));
    return r.phase === "shop" && awake.length >= 1 && almost && r.gold >= 3;
  };
  // Fill the line, then buy only copies of what it holds, reroll for them,
  // fight when the gold is gone; a run that ends starts the next.
  for (let steps = 0; steps < 3000 && !ready(run); steps++) {
    if (run.phase === "over") { run = await call("POST", "/runs", undefined, fuser.id); continue; }
    const dupe = run.offers.find((o) => run.line.some((u) => u.unitId === o.unitId && u.kind === "unit" && u.form === "sleeping"));
    const want = dupe ?? (run.line.length < 5 ? run.offers[0] : undefined);
    const d = run.phase === "crown" ? { kind: "fight" } : want && run.gold >= want.cost ? { kind: "buy", slot: want.slot } : run.gold >= 1 ? { kind: "reroll" } : { kind: "fight" };
    run = (await call("POST", `/runs/${run.runId}/decisions`, d, fuser.id)).run;
  }
  if (!ready(run)) errors.push(`fusion setup: never reached two Awoken units (phase ${run.phase}, round ${run.round})`);
  else {
    await page.evaluate((p) => localStorage.setItem("arena.player", JSON.stringify(p)), fuser);
    await page.reload();
    await page.getByTestId("play").click();
    await page.getByTestId("fight").waitFor();
    const almost = run.line.find((u) => u.kind === "unit" && u.form === "sleeping" && u.copies === 2 && run.offers.some((o) => o.unitId === u.unitId));
    const slot = run.offers.find((o) => o.unitId === almost.unitId).slot;
    await shot("shop-almost"); await noHScroll("shop-almost");
    await page.getByTestId(`offer-${slot}`).click();
    await page.getByTestId("buy-preview").waitFor();
    if (!/Awakens/.test(await page.getByTestId("buy-preview").textContent())) errors.push("awaken preview: no 'Awakens!'");
    await shot("awaken-preview"); await noHScroll("awaken-preview");
    await page.getByTestId("buy").click();
    await page.getByTestId("hint").filter({ hasText: "fuse" }).waitFor();
    const first = run.line.findIndex((u) => u.form === "awoken");
    const second = run.line.findIndex((u) => u.uid === almost.uid);
    await page.getByTestId(`line-${first}`).click();
    await page.getByTestId("fuse").click();
    await shot("fuse-pick"); await noHScroll("fuse-pick");
    await page.getByTestId(`line-${second}`).click();
    await page.getByTestId("preview-confirm").waitFor();
    await shot("fusion-preview"); await noHScroll("fusion-preview");
    await page.getByTestId("preview-confirm").click();
    await page.locator(".card.fused").waitFor();
    await shot("fused"); await noHScroll("fused");
    await page.locator(".card.fused").click();
    await page.getByTestId("info").click();
    await page.getByTestId("unit-sheet").waitFor();
    if (!/discovered by @Fuser|not yet claimed/.test(await page.getByTestId("unit-sheet").textContent())) errors.push("fused sheet: no discovery credit");
    await shot("fused-sheet"); await noHScroll("fused-sheet");
  }
  console.log(`mvp phone: ${round} fights, ${shots} screenshots in ${out}`);
} finally {
  await browser.close();
  child?.kill();
}
if (errors.length) { console.error(errors.join("\n")); process.exit(1); }
