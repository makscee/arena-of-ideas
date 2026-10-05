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
  /** Every match of `locator` is at least 44 px in `dim` (a tap target). */
  const tap44 = async (name, locator, dim = "height") => {
    for (const box of await locator.evaluateAll((els) => els.map((e) => e.getBoundingClientRect().toJSON()))) {
      if (box[dim] < 44 - 0.5) errors.push(`${name}: ${dim} ${Math.round(box[dim])}px < 44`);
    }
  };
  /** The element is on screen without scrolling (the bottom of a 640 px phone). */
  const onScreen = async (name, locator) => {
    const box = await locator.boundingBox();
    if (!box || box.y + box.height > 640 + 0.5 || box.y < 0) errors.push(`${name}: off screen (${box ? Math.round(box.y + box.height) : "none"}px)`);
  };

  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").waitFor({ timeout: 10_000 });
  await shot("name"); await noHScroll("name");
  await page.getByTestId("name-input").fill("PhoneTester");
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").waitFor();
  await shot("home"); await noHScroll("home");
  await tap44("dev summary", page.locator("details.dev summary"));
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
      await tap44("change chip", page.getByTestId("change"));
      await tap44("trace close", page.getByTestId("trace-close"));
      await tap44("trace close", page.getByTestId("trace-close"), "width");
      await tap44("past step", page.locator("button.bv-past"));
      if ((await page.getByTestId("caption-side").count()) === 0 && /→/.test(await page.getByTestId("caption").textContent())) errors.push("battle caption: no side tag on a unit's act");
      // A battle card (below its chip) opens the unit's sheet, which closes with Close.
      await page.getByTestId("trace-close").click();
      const box = await page.getByTestId("battle-you").locator(".bv-card").first().boundingBox();
      await page.mouse.click(box.x + box.width / 2, box.y + box.height - 8);
      await page.getByTestId("unit-sheet").waitFor();
      await shot("battle-unit-sheet"); await noHScroll("battle-unit-sheet");
      await page.getByTestId("sheet-close").click();
      await page.getByTestId("unit-sheet").waitFor({ state: "detached" });
      // A unit with two changes in one step (a status and the stat it moves)
      // shows both in its chip, and its trace lists both; shot when this
      // battle has one.
      if (await page.getByTestId("battle-play").textContent() === "❚❚") await page.getByTestId("battle-play").click();
      for (let k = 0; k < 80 && (await page.locator('[data-testid="change"][data-count="2"]').count()) === 0 && !(await page.getByTestId("battle-step").isDisabled()); k++) await page.getByTestId("battle-step").click();
      const multi = page.locator('[data-testid="change"][data-count="2"]').first();
      if (await multi.count()) {
        const lines = await multi.locator(".bv-l").allTextContents();
        if (lines.length !== 2 || lines.some((l) => !l.trim())) errors.push(`two changes: chip shows ${JSON.stringify(lines)}`);
        await shot("battle-two-changes"); await noHScroll("battle-two-changes");
        await multi.click();
        await page.getByTestId("trace-group").waitFor();
        if ((await page.getByTestId("trace-group-change").count()) !== 2) errors.push("two changes: the trace doesn't list both");
        await tap44("trace group change", page.getByTestId("trace-group-change"));
        await shot("battle-two-changes-trace");
        await page.getByTestId("trace-close").click();
      } else console.log("mvp phone: round 1 had no unit with two changes in one step (no shot)");
    }
    await page.getByTestId("battle-skip").click();
    await page.getByTestId("outcome").waitFor({ timeout: 10_000 });
    if (!whyShot && (await page.getByTestId("why-lost").isVisible())) {
      whyShot = true;
      await shot("result-after-loss"); await noHScroll("result-after-loss");
      // The main actions stay on screen however long "why I lost" runs.
      await onScreen("result after loss: Next round", page.getByTestId("continue"));
      await onScreen("result after loss: Replay", page.getByTestId("replay"));
      const why = page.getByTestId("why-lost").locator("button.bv-why").first();
      if (await why.count()) {
        await why.click();
        await page.getByTestId("why-sheet").waitFor();
        await shot("why-sheet"); await noHScroll("why-sheet");
        await page.getByTestId("sheet-close").click();
        await page.getByTestId("why-sheet").waitFor({ state: "detached" });
      }
    }
    await page.getByTestId("outcome").waitFor({ timeout: 10_000 });
    if (round === 1) { await shot("result"); await noHScroll("result"); }
    await page.getByTestId("continue").click();
    if (await page.getByTestId("run-over").isVisible().catch(() => false)) break;
  }
  await page.getByTestId("run-over").waitFor({ timeout: 10_000 });
  await shot("run-over"); await noHScroll("run-over");
  const over = await page.getByTestId("run-over").textContent();
  if (/No champion/.test(over) && /Reached the Crown/.test(over)) errors.push(`run over: "${over}" contradicts itself`);
  if (/\b1 (wins|draws|losses)\b|\b([02-9]|\d\d+) (win|draw|loss)\b/.test(over)) errors.push(`run over: plural wrong in "${over}"`);
  await page.getByTestId("home").click();
  await page.getByTestId("play").waitFor();
  await shot("home-after");

  // Stats (slice 11): the finished run counted its units' rates; every tab opens.
  await page.getByTestId("stats").click();
  await page.getByTestId("stats-units").waitFor();
  if ((await page.getByTestId("stats-unit").count()) === 0) errors.push("stats: no unit rates after a finished run");
  await shot("stats-units"); await noHScroll("stats-units");
  await page.getByTestId("stats-tab-champions").click();
  await page.getByTestId("stats-champions").waitFor();
  await shot("stats-champions"); await noHScroll("stats-champions");
  await page.getByTestId("stats-tab-fusions").click();
  await page.getByTestId("stats-fusions").waitFor();
  await shot("stats-fusions"); await noHScroll("stats-fusions");
  await page.getByTestId("stats-back").click();
  await page.getByTestId("play").waitFor();

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
    await shot("shop-fuse-actions"); await noHScroll("shop-fuse-actions");
    await tap44("move buttons beside Fuse", page.locator('[data-testid="move-left"], [data-testid="move-right"]'), "width");
    await page.getByTestId("fuse").click();
    await shot("fuse-pick"); await noHScroll("fuse-pick");
    await page.getByTestId(`line-${second}`).click();
    await page.getByTestId("preview-confirm").waitFor();
    await shot("fusion-preview"); await noHScroll("fusion-preview");
    await page.getByTestId("preview-confirm").click();
    await page.locator(".card.fused").waitFor();
    await shot("fused"); await noHScroll("fused");
    await page.locator(".card.fused").screenshot({ path: `${out}/${String(++shots).padStart(2, "0")}-fused-card.png` });
    // The fused card's row lines up: every card in it is one height.
    const heights = await page.getByTestId("line").locator(".card.you").evaluateAll((els) => els.map((e) => Math.round(e.getBoundingClientRect().height)));
    if (new Set(heights).size > 1) errors.push(`fused row: uneven card heights ${heights.join(",")}`);
    await page.locator(".card.fused").click();
    await page.getByTestId("info").click();
    await page.getByTestId("unit-sheet").waitFor();
    if (!/discovered by (you|@Fuser)/.test(await page.getByTestId("unit-sheet").textContent())) errors.push("fused sheet: no discovery credit");
    if (!(await page.getByTestId("sheet-close").isVisible())) errors.push("unit sheet from Info: no Close button");
    await shot("fused-sheet"); await noHScroll("fused-sheet");
  }
  // Dev "End day now": Home says plainly how the day ended; a table only for a real playoff.
  await page.reload();
  await page.getByTestId("play").waitFor();
  const endedSeq = (await call("GET", "/day")).seq;
  await page.locator("details.dev summary").click();
  await page.getByTestId("end-day").click();
  // Home re-renders with the day just ended ("Day N ended" / "Playoff · day
  // N"), not the panel of the day before, which is already on screen.
  await page.waitForFunction((n) => new RegExp(`\\b[Dd]ay ${n}\\b`).test(document.querySelector('[data-testid="playoff"] .label')?.textContent ?? ""), endedSeq, { timeout: 10_000 });
  const ended = await page.getByTestId("playoff-summary").textContent();
  const table = await page.getByTestId("playoff-standing").count();
  if (!/^No slayers|was the only slayer|won the playoff/.test(ended)) errors.push(`day end: "${ended}"`);
  if (/^No slayers|only slayer/.test(ended) && table > 0) errors.push(`day end: a table under "${ended}"`);
  if (/won the playoff/.test(ended) && table < 2) errors.push(`day end: playoff without its table`);
  await shot("home-day-ended"); await noHScroll("home-day-ended");

  // The reigning champion's own Crown (#587): "Reigning" slays and is crowned
  // through the API (e2e/mvp-own-crown.ts, local server only), then the phone
  // plays their run at the Crown against their own team: no slay is promised.
  if (child) {
    const setup = execFileSync("node", ["--import", "tsx/esm", "e2e/mvp-own-crown.ts", "--url", url], { encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] });
    const { player: champ } = JSON.parse(setup.trim().split("\n").at(-1));
    await page.evaluate((p) => localStorage.setItem("arena.player", JSON.stringify(p)), champ);
    await page.reload();
    await page.getByTestId("play").waitFor();
    const homeText = await page.locator("#app").textContent();
    if (!/This is your team: today the others try to beat it/.test(homeText) || /Beat this team in the Crown/.test(homeText)) errors.push("champion's home: no 'your team' hint");
    await shot("home-champion"); await noHScroll("home-champion");
    await page.getByTestId("play").click();
    await page.getByTestId("fight").waitFor();
    const opp = await page.getByTestId("next-opponent").textContent();
    if (!/\(your own team\)/.test(opp)) errors.push(`own crown: next opponent "${opp}"`);
    if (!/doesn't count as a slay/.test(await page.getByTestId("hint").textContent())) errors.push("own crown: the hint promises a slay");
    await shot("crown-own"); await noHScroll("crown-own");
    await page.getByTestId("fight").click();
    await page.getByTestId("battle-skip").click();
    await page.getByTestId("outcome").waitFor({ timeout: 10_000 });
    const result = await page.locator("#app").textContent();
    if (/slayer today/.test(result) || !/own champion team/.test(result)) errors.push(`own crown result: "${result.slice(0, 200)}"`);
    await shot("result-own-crown"); await noHScroll("result-own-crown");
    await page.getByTestId("continue").click();
    await page.getByTestId("run-over").waitFor({ timeout: 10_000 });
    const ownOver = await page.getByTestId("run-over").textContent();
    if (/slayer today/.test(ownOver) || !/own champion team/.test(ownOver)) errors.push(`own crown run over: "${ownOver}"`);
    await shot("run-over-own-crown"); await noHScroll("run-over-own-crown");
  }
  console.log(`mvp phone: ${round} fights, ${shots} screenshots in ${out}`);
} finally {
  await browser.close();
  child?.kill();
}
if (errors.length) { console.error(errors.join("\n")); process.exit(1); }
