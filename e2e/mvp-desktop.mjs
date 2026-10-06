// MVP desktop e2e (round 2, R2-9): a whole run at 1440×900 in Chromium, with
// mouse and keyboard only (no taps), and a screenshot of every screen. Checks
// the desktop layout: the top bar, the wide board with 132×172 cards, the
// inspector on the right that reads the hovered or selected card (no pop-up
// sheet in the shop), drag to reorder and the keys (1–7 buy, R reroll, L lock, Space
// fight, ← → move, S sell, Esc back). Without --url it builds the mobile
// client and starts the MVP server on a free port.
//   npm run mvp:desktop -- [--url https://m1.twin-pogona.ts.net/arena/] [--out e2e/.shots/mvp-desktop]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";
import { escPass } from "./esc-keys.mjs";
import { nowSheetChecks } from "./now-sheet.mjs";

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

// A tag per pass keeps a second pass at one server apart (names may repeat).
const TAG = Date.now().toString(36).slice(-4);
const browser = await launchChromium();
const errors = [];
let shots = 0;
try {
  const page = await browser.newPage({ viewport: { width: W, height: H } });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && errors.push(`console: ${m.text()}`));
  /** Every card's When · Who · Does icon line fits its card (R3-4): no row
   * overflows, and a line with more icons than it shows ends in "+". A
   * battle card's line lies in its corner, over the card (R3-19): it is left
   * out here, and e2e/probe-cause.mjs checks it clears the card's parts. */
  let iconCards = 0;
  const iconsFit = async (name) => {
    const r = await page.evaluate(() => [...document.querySelectorAll('.card:not(.bv-card) [data-testid="card-icons"]')].filter((el) => el.getClientRects().length).map((el) => {
      const shown = [...el.querySelectorAll(".ci")].filter((c) => getComputedStyle(c).display !== "none").length;
      const more = [...el.querySelectorAll(".more")].some((m) => getComputedStyle(m).display !== "none");
      return { name: el.closest(".card")?.querySelector(".name")?.textContent ?? "?", over: el.scrollWidth > el.clientWidth, short: el.querySelectorAll(".ci").length > shown && !more };
    }));
    iconCards += r.length;
    for (const c of r) {
      if (c.over) errors.push(`${name}: ${c.name}'s icon line overflows its card`);
      if (c.short) errors.push(`${name}: ${c.name} hides icons without a "+"`);
    }
  };
  const shot = async (name) => { await page.screenshot({ path: `${out}/${String(++shots).padStart(2, "0")}-${name}.png` }); await iconsFit(name); };
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
  /** The last sound the client asked for (round 3, note 16: ui/sound.ts logs each to window.__sfx). */
  const lastSfx = () => page.evaluate(() => window.__sfx?.at(-1) ?? "");
  const wantSfx = async (what, re) => {
    const got = await lastSfx();
    if (!re.test(got)) errors.push(`sound: ${what} played "${got}"`);
  };

  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").waitFor({ timeout: 10_000 });
  await page.keyboard.type(`DeskTester${TAG}`);
  await page.keyboard.press("Enter");
  await page.getByTestId("play").waitFor();
  // The title menu's Sound row: on at 60% by default.
  if ((await page.getByTestId("home-actions").getByTestId("sound-toggle").textContent())?.includes("on") !== true) errors.push("sound: the title menu's toggle isn't on by default");
  if ((await page.getByTestId("home-actions").getByTestId("sound-volume").inputValue()) !== "60") errors.push("sound: the default volume isn't 60");
  await shot("home"); await noHScroll("home"); await wide("home", 1100); await onScreen("home: Play", page.getByTestId("play"));
  await cardSize("home champion", "champion");
  // Home's two columns: Play sits right of the champion panel.
  const [champBox, playBox] = [await page.getByTestId("champion").boundingBox(), await page.getByTestId("play").boundingBox()];
  if (!(playBox.x > champBox.x + champBox.width)) errors.push("home: Play isn't in the right column");
  {
    // At 1024px the champion's 5 cards stay on one line (R2-10).
    const size = page.viewportSize();
    await page.setViewportSize({ width: 1024, height: size.height });
    const tops = await page.getByTestId("champion").locator(".card").evaluateAll((els) => els.map((e) => Math.round(e.getBoundingClientRect().top)));
    if (new Set(tops).size > 1) errors.push(`home at 1024px: the champion's cards wrap (${tops.join(",")})`);
    await shot("home-1024");
    await page.setViewportSize(size);
  }

  await page.getByTestId("stats").click();
  await page.getByTestId("stats-back").waitFor();
  await shot("stats"); await noHScroll("stats"); await wide("stats", 900);
  await page.getByTestId("stats-back").click();
  // The Codex uses the width: its unit grid runs many cards a row; Esc goes back.
  await page.getByTestId("codex").click();
  await page.getByTestId("codex-units").waitFor();
  await shot("codex"); await noHScroll("codex"); await wide("codex", 1200);
  {
    // 7 a row beside the 380px inspector.
    const tops = await page.getByTestId("codex-unit").evaluateAll((els) => els.slice(0, 7).map((e) => Math.round(e.getBoundingClientRect().top)));
    if (new Set(tops).size > 1) errors.push(`codex: the first 7 cards wrap (${tops.join(",")})`);
  }
  // A unit opens in the inspector on the right (no overlay on desktop), which ends at the window's edge.
  await page.getByTestId("codex-unit").nth(2).click();
  await page.locator('[data-testid="inspector"] [data-testid="unit-sheet"]').waitFor({ timeout: 2_000 }).catch(() => errors.push("codex: a unit doesn't open in the inspector"));
  if (await page.getByTestId("overlay").count()) errors.push("codex: a unit opened in an overlay");
  {
    const box = await page.getByTestId("inspector").boundingBox();
    if (!box || box.x + box.width < W - 2) errors.push(`codex: the inspector ends at ${box ? Math.round(box.x + box.width) : "?"}px, not the window's edge`);
  }
  await page.locator('[data-testid="inspector"] [data-testid="see-awoken"]').click().catch(() => {});
  await shot("codex-inspector");
  // R3-5: the Summoned chip; a summon opens in the inspector with "Summoned by".
  await page.getByTestId("codex-tier-summoned").click();
  if ((await page.getByTestId("codex-summon").count()) !== 6) errors.push(`codex: Summoned shows ${await page.getByTestId("codex-summon").count()} cards, not 6`);
  await page.locator('[data-testid="codex-summon"][data-summon="wolf"]').click();
  await page.locator('[data-testid="inspector"] [data-testid="summon-sheet"]').waitFor({ timeout: 2_000 }).catch(() => errors.push("codex: the Wolf doesn't open in the inspector"));
  const wolfBy = (await page.getByTestId("summoned-by").textContent().catch(() => "")) ?? "";
  if (!wolfBy.includes("Summoner")) errors.push(`codex: the Wolf's "Summoned by" lacks Summoner ("${wolfBy}")`);
  await shot("codex-summoned");
  await page.getByTestId("codex-tier-all").click();
  await page.getByTestId("codex-sort-pick").click();
  await page.getByTestId("codex-rate").first().waitFor();
  await shot("codex-sort-pick");
  await page.getByTestId("codex-sort-tier").click();
  await page.getByTestId("codex-tab-keywords").click();
  await page.getByTestId("codex-keywords").waitFor();
  await shot("codex-keywords");
  // Esc clears the inspected unit first (R3-6), then goes back.
  await page.keyboard.press("Escape");
  if (await page.locator('[data-testid="inspector"] [data-testid="unit-sheet"]').count()) errors.push("codex: Esc doesn't clear the inspector");
  if (!(await page.getByTestId("codex-keywords").count())) errors.push("codex: Esc with a unit inspected left the Codex");
  await page.keyboard.press("Escape");
  await page.getByTestId("play").click();

  let round = 0;
  let dragged = false;
  let sold = false;
  let resultShot = false;
  for (let guard = 0; guard < 20; guard++) {
    await page.getByTestId("fight").waitFor({ timeout: 10_000 });
    round++;
    const crown = (await page.getByTestId("gold").count()) === 0;
    if (round === 2 && !crown && (await gold()) >= 1 && (await page.getByTestId("offers").locator(".card").count()) >= 2) {
      // Lock (R3-12): click offer 2, L locks it and it stays chosen; R keeps it
      // at the left, still locked; right-click unlocks it.
      const offer = (s) => page.getByTestId(`offer-${s}`);
      const locked = (s) => offer(s).evaluate((el) => el.classList.contains("locked"));
      const name = await offer(1).locator(".name").textContent();
      await offer(1).click();
      await page.getByTestId("inspector").getByTestId("lock").waitFor();
      if (!(await page.getByTestId("inspector").getByTestId("lock").textContent()).startsWith("Lock · L")) errors.push("lock: no Lock · L in the inspector");
      await page.keyboard.press("l");
      await settle();
      if (!(await locked(1))) errors.push("lock: L didn't lock the chosen offer");
      await page.getByTestId("inspector").getByTestId("lock").waitFor();
      if (!(await page.getByTestId("inspector").getByTestId("lock").textContent()).startsWith("Unlock")) errors.push("lock: the locked offer isn't still in the inspector with Unlock");
      if (!(await page.getByTestId("keys").textContent()).includes("L lock")) errors.push("lock: the keys line has no L");
      await shot("shop-locked");
      const g1 = await gold();
      await page.keyboard.press("r");
      await settle();
      if ((await gold()) !== g1 - 1) errors.push("lock: R didn't reroll with a locked offer");
      if ((await offer(0).locator(".name").textContent()) !== name || !(await locked(0))) errors.push(`lock: after R offer 1 is "${await offer(0).locator(".name").textContent()}", not the locked "${name}"`);
      await offer(0).click({ button: "right" });
      await settle();
      if (await locked(0)) errors.push("lock: right-click didn't unlock");
      if ((await page.locator(".overlay, [role=menu]").count()) > 0) errors.push("lock: right-click opened something");
    }
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
        await wantSfx("S (sell)", /^sell$/);
        sold = true;
        continue;
      }
      const g0 = g;
      await page.keyboard.press("1");
      await settle();
      if ((await gold()) === g0) { errors.push(`round ${round}: key 1 didn't buy`); break; }
      await wantSfx("key 1 (buy)", /^(coin|merge|level-up)$/);
    }
    if (round === 1 && !crown && (await gold()) >= 1) {
      // R rerolls.
      const g0 = await gold();
      await page.keyboard.press("r");
      await settle();
      if ((await gold()) !== g0 - 1) errors.push("R didn't reroll");
      await wantSfx("R", /^reroll$/);
      // M mutes: the next sound is logged muted; M again turns it back on.
      await page.keyboard.press("m");
      await page.keyboard.press("r");
      await settle();
      // (a reroll, or "wrong" when the gold ran out)
      await wantSfx("R after M", /^(reroll|wrong) \(muted\)$/);
      await page.keyboard.press("m");
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
      await wantSfx("← (move)", /^move-left$/);
      if (!(await page.getByTestId("line-0").evaluate((el) => el.classList.contains("selected")))) errors.push("the moved unit isn't selected anymore");
      // Esc deselects.
      await page.keyboard.press("Escape");
      if (await page.locator('[data-testid="line"] .card.selected').count()) errors.push("Esc didn't deselect");
      // With nothing left to step back from, Esc opens the ☰ run menu (R2-10); Esc again closes it.
      await page.keyboard.press("Escape");
      await page.getByTestId("run-menu").waitFor({ timeout: 2_000 }).catch(() => errors.push("Esc didn't open the run menu"));
      if (!(await page.getByTestId("run-menu").getByTestId("sound-toggle").count())) errors.push("sound: no Sound row in the run menu");
      await shot("run-menu");
      // ☰ Codex, then the window narrows below 1024px: Back finds the shop in its phone layout.
      await page.getByTestId("menu-codex").click();
      await page.getByTestId("codex-units").waitFor();
      await page.setViewportSize({ width: 900, height: H });
      await page.waitForTimeout(200);
      if (await page.getByTestId("inspector").count()) errors.push("codex at 900px: the inspector stays");
      await page.getByTestId("codex-back").click();
      await page.getByTestId("fight").waitFor();
      if (await page.getByTestId("inspector").count()) errors.push("codex → 900px → Back: the shop kept its desktop layout");
      await page.setViewportSize({ width: W, height: H });
      await page.waitForTimeout(200);
      if (!(await page.getByTestId("inspector").count())) errors.push("back at 1440px: the shop has no inspector");
      await page.keyboard.press("Escape");
      await page.getByTestId("run-menu").waitFor({ timeout: 2_000 }).catch(() => {});
      await page.keyboard.press("Escape");
      if (await page.getByTestId("run-menu").count()) errors.push("Esc didn't close the run menu");
    }
    if (crown) { await shot("crown-shop"); await noHScroll("crown-shop"); }
    // Space fights.
    await page.keyboard.press("Space");
    await page.getByTestId("battle-end").waitFor({ timeout: 10_000 });
    if (round === 1) await shot("battle");
    if (round === 1) await logFirst("battle opens");
    if (round === 1) await whyOnDesktop();
    if (round === 1) await desktopBattle();
    if (round === 1) await nowSheetChecks(page, errors, { shot, phone: false });
    await page.getByTestId("battle-end").click();
    // R2-17 batch E: the end card is the fight's one result: the round
    // fought, hearts and record, and its button goes straight on.
    await page.getByTestId("end-card").waitFor({ timeout: 10_000 });
    const runLine = (await page.getByTestId("end-run").textContent().catch(() => "")) ?? "";
    if (!(crown ? /CROWN/ : new RegExp(`R${round}/12`)).test(runLine) || !/[♥♡]/.test(runLine) || !/\d+W/.test(runLine)) errors.push(`end card: run line "${runLine}" after round ${round}`);
    const done = (await page.getByTestId("battle-done").textContent()) ?? "";
    if (!/^(Next round|To the Crown|See the run)$/i.test(done.trim())) errors.push(`end card: last button "${done}"`);
    if (await page.getByTestId("outcome").count()) errors.push("a result screen still follows the end card");
    if (!resultShot) {
      resultShot = true;
      await shot("result"); await noHScroll("result");
      await onScreen("end card: Next round", page.getByTestId("battle-done"));
    } else if ((await page.getByTestId("battle-word").textContent()) === "DEFEAT") {
      await shot(`result-loss-r${round}`);
    }
    // Enter moves on, from the end card.
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

  // Awaken and fuse with keys and mouse: a second player plays through the
  // API (as in mvp-phone) until it has one Awoken unit and the copy that
  // awakens a second in the shop; the page buys it with its number key,
  // selects the first, F, clicks the second, and fuses in the inspector.
  const call = async (method, path, body, pid) => {
    const res = await fetch(new URL(`api/v1${path}`, url), { method, headers: { "content-type": "application/json", ...(pid ? { "X-Arena-Player": pid } : {}) }, ...(body ? { body: JSON.stringify(body) } : {}) });
    const json = await res.json();
    if (!res.ok) throw new Error(`${method} ${path}: ${json.error}`);
    return json;
  };
  const fuser = await call("POST", "/players", { name: `DeskFuser${TAG}` });
  let run = await call("POST", "/runs", undefined, fuser.id);
  const ready = (r) => r.phase === "shop" && r.gold >= 3 && r.line.some((u) => u.kind === "unit" && u.form === "awoken") && r.line.some((u) => u.kind === "unit" && u.form === "sleeping" && u.copies === 2 && r.offers.some((o) => o.unitId === u.unitId));
  for (let steps = 0; steps < 3000 && !ready(run); steps++) {
    if (run.phase === "over") { run = await call("POST", "/runs", undefined, fuser.id); continue; }
    const dupe = run.offers.find((o) => run.line.some((u) => u.unitId === o.unitId && u.kind === "unit" && u.form === "sleeping"));
    const want = dupe ?? (run.line.length < 5 ? run.offers[0] : undefined);
    // An awakening's gift is skipped here (R3-15); the phone's chooser is R3-16's.
    const d = run.gift ? { kind: "gift", pick: null } : run.phase === "crown" ? { kind: "fight" } : want && run.gold >= want.cost ? { kind: "buy", slot: want.slot } : run.gold >= 1 ? { kind: "reroll" } : { kind: "fight" };
    run = (await call("POST", `/runs/${run.runId}/decisions`, d, fuser.id)).run;
  }
  // The awakening copy the page buys brings a gift (R3-15): the chooser
  // (R3-16) opens as a dialog. Esc sets it aside (a banner; the keys that
  // buy, reroll or fight do nothing meanwhile), Open gift brings it back, and
  // Skip lets it go with a click (the line stays as the fusion below needs it).
  const skipGift = async () => {
    const r = await call("GET", `/runs/${run.runId}`, undefined, fuser.id);
    if (!r.gift) return void errors.push("awakening: no gift offered");
    const shown = await page.getByTestId("gift-title").waitFor({ timeout: 5_000 }).then(() => true, () => false);
    if (!shown) return void errors.push("gift: the chooser doesn't open after the awakening copy");
    const cards = await page.getByTestId("gift-choices").locator(".card").count();
    if (cards !== 3) errors.push(`gift: ${cards} cards, want 3`);
    const width = await page.getByTestId("gift-card-0").evaluate((e) => Math.round(e.getBoundingClientRect().width));
    if (width < 100) errors.push(`gift: desktop cards are ${width}px wide`);
    await shot("gift-chooser");
    await page.keyboard.press("Escape");
    await page.getByTestId("gift-banner").waitFor({ timeout: 3_000 }).catch(() => errors.push("gift: Esc leaves no banner"));
    if (await page.getByTestId("gift-title").isVisible()) errors.push("gift: Esc doesn't set the chooser aside");
    const g = await gold();
    await page.keyboard.press("r");
    await page.keyboard.press(" ");
    await settle();
    if ((await gold()) !== g || !(await page.getByTestId("gift-banner").isVisible())) errors.push("gift: R or Space went through while the gift waits");
    await shot("gift-aside");
    await page.getByTestId("gift-open").click();
    await page.getByTestId("gift-skip").click();
    await page.getByTestId("gift-banner").waitFor({ state: "hidden", timeout: 5_000 }).catch(() => errors.push("gift: the banner stays after Skip"));
    await wantSfx("a skipped gift", /^click$/);
    if ((await call("GET", `/runs/${run.runId}`, undefined, fuser.id)).gift) errors.push("gift: still waiting after Skip");
  };
  if (!ready(run)) errors.push(`fusion setup: never reached two Awoken units (phase ${run.phase}, round ${run.round})`);
  else {
    await page.evaluate((p) => localStorage.setItem("arena.player", JSON.stringify(p)), fuser);
    await page.reload();
    await page.getByTestId("play").click();
    await page.getByTestId("fight").waitFor();
    const almost = run.line.find((u) => u.kind === "unit" && u.form === "sleeping" && u.copies === 2 && run.offers.some((o) => o.unitId === u.unitId));
    const key = run.offers.findIndex((o) => o.unitId === almost.unitId) + 1;
    // Hovering the offer shows "Awakens!" in the inspector.
    await page.getByTestId(`offer-${run.offers[key - 1].slot}`).hover();
    await page.getByTestId("inspector").getByTestId("buy-preview").waitFor();
    if (!/Awakens/.test(await page.getByTestId("inspector").getByTestId("buy-preview").textContent())) errors.push("awaken preview: no 'Awakens!' in the inspector");
    await shot("awaken-preview");
    await page.mouse.move(5, H - 5);
    await page.keyboard.press(String(key));
    await page.getByTestId("gift-title").waitFor();
    await wantSfx("the awakening copy", /^level-up$/);
    await skipGift();
    await page.getByTestId("hint").filter({ hasText: "fuse" }).waitFor();
    const awake = run.line.findIndex((u) => u.form === "awoken");
    const named = run.line[awake].name;
    // R3-14: B sends the first Awoken unit to the bench, and it fuses from there with a line unit.
    await page.getByTestId(`line-${awake}`).click();
    await page.keyboard.press("b");
    await page.getByTestId("bench-0").waitFor();
    await page.mouse.move(5, H - 5); // the inspector shows the hovered card first
    if ((await page.getByTestId("bench-0").locator(".name").textContent()) !== named) errors.push(`bench: B put "${await page.getByTestId("bench-0").locator(".name").textContent()}" on the bench, not "${named}"`);
    if (!/On your bench/.test(await page.getByTestId("inspector").textContent())) errors.push("bench: the inspector doesn't say the selected unit is on the bench");
    await shot("bench-b-key");
    const second = run.line.findIndex((u) => u.uid === almost.uid) - (run.line.findIndex((u) => u.uid === almost.uid) > awake ? 1 : 0);
    await page.getByTestId("inspector").getByTestId("fuse").waitFor();
    await page.keyboard.press("f");
    await page.locator(".card.fusable").first().waitFor();
    await shot("fuse-pick");
    await page.getByTestId(`line-${second}`).click();
    await page.getByTestId("inspector").getByTestId("fuse-preview").waitFor();
    await noOverlay("fusion preview");
    await shot("fusion-preview");
    // Swap tries the other order in place (R2-5).
    if (await page.getByTestId("inspector").getByTestId("preview-swap").count()) await page.getByTestId("preview-swap").click();
    await page.getByTestId("preview-confirm").click();
    await page.getByTestId("line").locator(".card.fused").waitFor();
    if (!/\bfuse\b/.test((await page.evaluate(() => window.__sfx ?? [])).slice(-2).join(" "))) errors.push("sound: the fuse made no sound");
    // A pair nobody had made reveals its name (R2-5); Esc closes it.
    if (await page.getByTestId("fusion-reveal").isVisible().catch(() => false)) {
      await wantSfx("the discovery reveal", /^discover$/);
      await shot("fusion-reveal");
      await page.keyboard.press("Escape");
      await page.getByTestId("fusion-reveal").waitFor({ state: "detached" });
    }
    await page.getByTestId("line").locator(".card.fused").hover();
    await page.getByTestId("inspector").getByTestId("unit-sheet").waitFor();
    if (!/discovered by (you|@\S+)/.test(await page.getByTestId("inspector").textContent())) errors.push("fused: no discovery credit in the inspector");
    if ((await page.getByTestId("bench").locator(".card.you").count()) !== 0) errors.push("bench: the fused bench unit is still on the bench");
    await shot("fused");
    // Drag between line and bench (R3-14): line-0 onto the bench's empty slot, then back onto the line's.
    const front = await page.getByTestId("line-0").locator(".name").textContent();
    await page.getByTestId("line-0").dragTo(page.getByTestId("bench-0-empty"));
    await page.getByTestId("bench-0").waitFor();
    if ((await page.getByTestId("bench-0").locator(".name").textContent()) !== front) errors.push(`bench: dragging line-0 to the bench put "${await page.getByTestId("bench-0").locator(".name").textContent()}" there, not "${front}"`);
    await shot("bench-dragged");
    await page.getByTestId("bench-0").dragTo(page.getByTestId("line").locator(".card.empty").first());
    await page.getByTestId("bench-0-empty").waitFor();
    const back = await page.getByTestId("line").locator(".card.you").last().locator(".name").textContent();
    if (back !== front) errors.push(`bench: dragging back put "${back}" at the line's end, not "${front}"`);
  }

  // 1024px is still desktop (120px cards, 340px inspector, nothing cut off); 1023px is the phone.
  // (The shop is on screen: the fusion run's, or a new run's when that setup failed.)
  await page.setViewportSize({ width: 1024, height: 768 });
  if (await page.getByTestId("play").count()) await page.getByTestId("play").click();
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
  /** R2-16: the desktop battle. The lines face each other on one row, fronts
   * in the middle; the whole fight fits the window (1440×900 and 1024×768)
   * with the control bar on screen; dragging the timeline to a turn shows
   * that turn; a Log row's click opens its Why. */
  async function desktopBattle() {
    if ((await page.getByTestId("battle-play").textContent()) === "❚❚") await page.getByTestId("battle-play").click();
    // R2-17: the end card keeps the timeline and the controls clear (as the mockup does), at both sizes.
    const endClear = async (name) => {
      if (!(await page.getByTestId("end-card").isVisible())) await page.getByTestId("battle-end").click();
      const endBox = await page.getByTestId("end-card").boundingBox();
      for (const id of ["timeline", "battle-back", "battle-play", "battle-replay"]) {
        const b = await page.getByTestId(id).boundingBox();
        if (endBox && b && endBox.y + endBox.height > b.y + 0.5 && endBox.y < b.y + b.height && endBox.x < b.x + b.width && endBox.x + endBox.width > b.x) errors.push(`${name}: the end card covers ${id}`);
      }
      const keys = await page.getByTestId("battle-keys").boundingBox();
      if (!keys) errors.push(`${name}: no key hints (Space / ←→ / R) by the controls`);
    };
    await endClear("end card 1440×900");
    await shot("battle-end-card");
    if (await page.getByTestId("end-card").isVisible()) await page.getByTestId("end-close").click();
    const facing = async (name, w, h) => {
      const [mine, theirs, panel] = [await page.getByTestId("battle-you").boundingBox(), await page.getByTestId("battle-them").boundingBox(), await page.getByTestId("trace").boundingBox()];
      if (!mine || !theirs || Math.abs(mine.y - theirs.y) > 2 || mine.x + mine.width > theirs.x + 0.5) errors.push(`${name}: the lines don't face each other on one row (${JSON.stringify({ mine, theirs })})`);
      if (!panel || theirs.x + theirs.width > panel.x + 0.5) errors.push(`${name}: the side panel covers their line`);
      // Fronts in the middle: your line runs right to left, theirs left to right.
      const xs = async (testid) => page.getByTestId(testid).locator(".bv-slot").evaluateAll((els) => els.map((e) => e.getBoundingClientRect().x));
      const [m, t] = [await xs("battle-you"), await xs("battle-them")];
      if (m.length > 1 && !(m[0] > m.at(-1))) errors.push(`${name}: your front isn't in the middle`);
      if (t.length > 1 && !(t[0] < t.at(-1))) errors.push(`${name}: their front isn't in the middle`);
      const doc = await page.evaluate(() => [document.documentElement.scrollWidth, document.documentElement.scrollHeight]);
      if (doc[0] > w || doc[1] > h) errors.push(`${name}: the fight scrolls (${doc.join("×")})`);
      for (const id of ["battle-back", "battle-play", "battle-step", "battle-speed", "battle-end", "battle-replay", "timeline"]) {
        const b = await page.getByTestId(id).boundingBox();
        if (!b || b.y < 0 || b.y + b.height > h + 0.5 || b.x + b.width > w + 0.5) errors.push(`${name}: ${id} off screen`);
      }
    };
    await facing("battle 1440×900", W, H);
    // Drag the timeline to the turn your front fell (or the last turn).
    const turns = page.getByTestId("timeline-turn");
    const n = await turns.count();
    if (!n) errors.push("timeline: no turns");
    else {
      let target = n - 1;
      for (let i = 0; i < n; i++) if (await turns.nth(i).locator('[data-kind="death"][data-side="you"]').count()) { target = i; break; }
      const turn = await turns.nth(target).getAttribute("data-turn");
      const [from, to] = [await turns.nth(0).boundingBox(), await turns.nth(target).boundingBox()];
      await page.mouse.move(from.x + 2, from.y + 10);
      await page.mouse.down();
      await page.mouse.move(to.x + to.width / 2, to.y + 10, { steps: 6 });
      await page.mouse.up();
      const hud = (await page.locator(".hud span").nth(2).textContent()) ?? "";
      const label = (t) => (Number(t) >= 1 ? `T${t}` : "Start");
      if (hud !== label(turn)) errors.push(`timeline: dragged to turn ${turn}, the board shows ${hud}`);
      if (!(await turns.nth(target).evaluate((e) => e.classList.contains("on")))) errors.push("timeline: the dragged-to turn isn't lit");
      if (await page.getByTestId("battle-play").textContent() !== "▶") errors.push("timeline: scrubbing didn't pause");
      console.log(`timeline: ${n} turns, dragged to T${turn} (${target === n - 1 ? "the last turn" : "your first loss"}), board at ${hud}`);
      await shot("battle-timeline");
      // The first turn's block, clicked at its start, is turn 1 (battle start has its own block, R2-17).
      for (const t of ["0", "1"]) {
        const block = page.locator(`[data-testid="timeline-turn"][data-turn="${t}"]`);
        if (!(await block.count())) continue;
        const bb = await block.boundingBox();
        await page.mouse.click(bb.x + 1, bb.y + bb.height - 4);
        const at = (await page.locator(".hud span").nth(2).textContent()) ?? "";
        if (at !== label(t)) errors.push(`timeline: clicked the start of ${label(t)}'s block, the HUD reads ${at}`);
      }
      // A mark's click jumps to its own beat.
      const mark = page.getByTestId("timeline-mark").first();
      if (await mark.count()) {
        await mark.click();
        const beat = Number(await mark.getAttribute("data-beat"));
        const lit = await page.getByTestId("timeline-turn").evaluateAll((els) => els.findIndex((e) => e.classList.contains("on")));
        const want = await page.getByTestId("timeline-turn").evaluateAll((els, b) => els.findIndex((e) => [...e.querySelectorAll("[data-beat]")].some((m) => Number(m.getAttribute("data-beat")) === b)), beat);
        if (lit !== want) errors.push(`timeline: a mark's click lit block ${lit}, not its own (${want})`);
      }
    }
    // Log: a row's click opens its Why.
    await page.getByTestId("tab-log").click();
    const rows = page.locator('[data-testid="log-row"]:visible');
    const k = await rows.count();
    if (!k) errors.push("log: no rows for the turns played");
    let opened = false;
    let hit = -1;
    for (let i = k - 1; i >= 0 && !opened; i--) {
      await page.getByTestId("tab-log").click();
      await rows.nth(i).click();
      opened = await page.getByTestId("trace-text").isVisible();
      if (opened) hit = i;
    }
    if (k && !opened) errors.push("log: no row opened its Why");
    if (opened && !(await page.getByTestId("tab-why").evaluate((e) => e.classList.contains("on")))) errors.push("log: a row's Why opened without the Why tab");
    if (opened && (await page.getByTestId("tab-why").isDisabled())) errors.push("log: a row's Why opened with the Why tab disabled");
    await shot("battle-log-why");
    // R3-17: ✕, Esc and ▶ each close Why back to Log, Why disabled again.
    if (opened) {
      await page.getByTestId("trace-close").click();
      await logFirst("Why ✕");
      await rows.nth(hit).click();
      await page.keyboard.press("Escape");
      await logFirst("Why Esc");
      await rows.nth(hit).click();
      await page.getByTestId("battle-play").click();
      await logFirst("Why ▶");
      if ((await page.getByTestId("battle-play").textContent()) === "❚❚") await page.getByTestId("battle-play").click();
    }
    // 1024×768: still whole, still facing.
    await page.setViewportSize({ width: 1024, height: 768 });
    await page.waitForTimeout(200);
    await facing("battle 1024×768", 1024, 768);
    await endClear("end card 1024×768");
    await shot("battle-end-card-1024");
    if (await page.getByTestId("end-card").isVisible()) await page.getByTestId("end-close").click();
    await shot("battle-1024");
    await page.setViewportSize({ width: W, height: H });
    await page.waitForTimeout(200);
    await page.getByTestId("trace-close").click().catch(() => {});
  }

  /** R3-17: with nothing traced, the side panel shows Log and Why is disabled. */
  async function logFirst(when) {
    if (!(await page.getByTestId("tab-log").evaluate((e) => e.classList.contains("on")))) errors.push(`${when}: the Log tab isn't on`);
    if (!(await page.getByTestId("tab-why").isDisabled())) errors.push(`${when}: Why is enabled with nothing traced`);
    if (!(await page.getByTestId("battle-log").isVisible())) errors.push(`${when}: the Log isn't showing`);
  }

  /** R2-15: Why opens as a panel right of the battle column, and its chain
   * runs back to a turn. Steps through the battle for a change that a firing
   * made in a turn (event ← firing ← … ← Turn N), and clicks its steps. */
  async function whyOnDesktop() {
    await page.getByTestId("battle-play").click(); // pause
    for (let i = 0; i < 80; i++) {
      for (const chip of await page.getByTestId("change").all()) {
        await chip.click();
        const steps = page.getByTestId("why-step");
        const kinds = await steps.evaluateAll((els) => els.map((e) => e.dataset.kind));
        const root = (await steps.last().textContent()) ?? "";
        if (kinds.includes("firing") && kinds.includes("event") && /Turn \d/.test(root)) {
          const panel = await page.getByTestId("trace").boundingBox();
          const column = await page.locator(".bv-screen").boundingBox();
          if (!panel || !column || panel.x < column.x + column.width) errors.push(`why: the panel ${JSON.stringify(panel)} covers the battle column ${JSON.stringify(column)}`);
          await shot("battle-why"); await noHScroll("battle-why");
          const hud = () => page.locator(".hud span").nth(2).textContent();
          const before = await hud();
          await steps.nth(kinds.indexOf("event")).click();
          const after = await hud();
          if (!(await steps.nth(kinds.indexOf("event")).evaluate((e) => e.classList.contains("on")))) errors.push("why: the clicked step isn't lit");
          await shot("battle-why-step");
          console.log(`why: ${kinds.join(" ← ")} (${root.replace(/caused by/i, "").trim()}); its event step moved the board ${before} → ${after}`);
          await page.getByTestId("trace-close").click();
          return;
        }
        await page.getByTestId("trace-close").click();
      }
      if (await page.getByTestId("battle-step").isDisabled()) break;
      await page.getByTestId("battle-step").click();
      if (await page.getByTestId("end-card").isVisible()) break;
    }
    console.log("why: no change in round 1 came from a firing in a turn; panel not checked");
  }

  console.log(`mvp desktop: ${round} fights, ${shots} screenshots in ${out}, ${iconCards} card icon lines fit`);
  // Esc everywhere (R3-6): a fresh player, keys only.
  await escPass(browser, url, { label: `desktop-${W}`, viewport: { width: W, height: H }, errors });
  await escPass(browser, url, { label: "narrow-390", viewport: { width: 390, height: 844 }, errors });
} finally {
  await browser.close();
  child?.kill();
}
if (errors.length) { console.error(errors.join("\n")); process.exit(1); }
