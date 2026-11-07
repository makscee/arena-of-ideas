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
import { beamChecks } from "./beams.mjs";
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
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", ARENA_CODEX: "1" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
  for (let i = 0; i < 300; i++) { // up to 60 s: m1 and m4 are shared, and under load the server starts slowly
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
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}\n${(e.stack ?? "").split("\n").slice(1, 6).join("\n")}`));
  page.on("console", (m) => m.type() === "error" && errors.push(`console: ${m.text()}`));
  // A refused decision (409) names what was sent and why, beside its console line.
  page.on("response", async (r) => {
    if (r.status() !== 409) return;
    const why = await r.text().catch(() => "");
    errors.push(`409 on ${r.request().method()} ${new URL(r.url()).pathname}: sent ${r.request().postData() ?? "nothing"}, got ${why.slice(0, 200)}`);
  });
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
    // M5-2: a card's name keeps its descenders (y, g, p): it clips sideways only (#889's "Bulwpper").
    const cut = await page.evaluate(() => [...document.querySelectorAll(".card:not(.bv-card) .name")].filter((el) => el.getClientRects().length).filter((el) => getComputedStyle(el).overflowY !== "visible").map((el) => el.textContent));
    for (const n of cut) errors.push(`${name}: ${n}'s name is clipped at the bottom`);
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
  /** Whole on screen, no scrolling. */
  const onScreen = async (name, locator) => {
    const box = await locator.boundingBox();
    if (!box || box.y < 0 || box.y + box.height > H + 0.5 || box.x + box.width > W + 0.5) errors.push(`${name}: off screen`);
  };
  const noOverlay = async (name) => {
    if (await page.getByTestId("overlay").count()) errors.push(`${name}: a pop-up sheet in the shop (the inspector reads cards on desktop)`);
  };
  const gold = async () => Number(((await page.getByTestId("gold").textContent().catch(() => "0")) ?? "0").replace("g", ""));
  const lineCount = () => page.getByTestId("line").locator(".cell.you").count();
  /** Waits until the shop is drawn again after a decision (the gold or the line changed). */
  const settle = async () => { await page.waitForTimeout(150); await page.waitForFunction(() => !document.getElementById("app").classList.contains("busy")); };
  /** A gift the run's own buys brought (an awakening copy, R3-15): skip it, so
   * the keys and Fight work again. The gift chooser has its own check below. */
  const dropOwnGift = async () => {
    if (!(await page.getByTestId("gift-title").isVisible()) && (await page.getByTestId("gift-open").isVisible())) await page.getByTestId("gift-open").click();
    if (!(await page.getByTestId("gift-title").isVisible())) return;
    await page.getByTestId("gift-skip").click();
    await page.getByTestId("gift-title").waitFor({ state: "hidden", timeout: 5_000 }).catch(() => {});
    await settle();
  };
  /** The last sound the client asked for (round 3, note 16: ui/sound.ts logs each to window.__sfx). */
  /** The end card: End jumps to it; once the fight was watched, End has given way to Next round
   * (arena#14), so › steps to the last beat, where the card comes up. */
  const showEnd = async () => {
    if (await page.getByTestId("battle-end").count()) return void (await page.getByTestId("battle-end").click());
    for (let k = 0; k < 400 && !(await page.getByTestId("end-card").isVisible()); k++) await page.getByTestId("battle-step").click();
  };
  const lastSfx = () => page.evaluate(() => window.__sfx?.at(-1) ?? "");
  const wantSfx = async (what, re) => {
    const got = await lastSfx();
    if (!re.test(got)) errors.push(`sound: ${what} played "${got}"`);
  };

  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").waitFor({ timeout: 10_000 });
  await page.getByTestId("name-input").fill(`DeskTester${TAG}`);
  await page.keyboard.press("Enter");
  await page.getByTestId("play").waitFor();
  // The gear's Sound row (M5-3: settings left the Throne): on at 60% by default.
  await page.getByTestId("settings").click();
  await page.getByTestId("settings-sheet").waitFor();
  if ((await page.getByTestId("settings-sheet").getByTestId("sound-toggle").textContent())?.includes("on") !== true) errors.push("sound: the title menu's toggle isn't on by default");
  if ((await page.getByTestId("settings-sheet").getByTestId("sound-volume").inputValue()) !== "60") errors.push("sound: the default volume isn't 60");
  // Round 4, note 5: Music under it, on at 40%; the first key started home's loop.
  if ((await page.getByTestId("settings-sheet").getByTestId("music-toggle").textContent())?.includes("on") !== true) errors.push("music: the title menu's toggle isn't on by default");
  if ((await page.getByTestId("settings-sheet").getByTestId("music-volume").inputValue()) !== "40") errors.push("music: the default volume isn't 40");
  if (!/^theme\d\.m4a$/.test((await page.evaluate(() => window.__music?.().track)) ?? "")) errors.push("music: home's theme isn't playing (void-board#922: home plays a run theme)");
  await shot("settings");
  await page.getByTestId("sheet-close").click();
  await shot("home"); await noHScroll("home"); await wide("home", 1100); await onScreen("home: Play", page.getByTestId("play"));
  {
    // The champion's pieces (void-board#1295): sized from the panel, so no two touch, each tab and name whole.
    const reach = await page.getByTestId("home-their").locator(".pc").evaluateAll((els) => els.map((pc) => {
      const rs = [pc, ...pc.querySelectorAll("*")].map((e) => e.getBoundingClientRect()).filter((r) => r.width > 0);
      return { l: Math.min(...rs.map((r) => r.left)), r: Math.max(...rs.map((r) => r.right)), t: Math.min(...rs.map((r) => r.top)), b: Math.max(...rs.map((r) => r.bottom)) };
    }));
    const touch = reach.some((a, i) => reach.some((b, j) => j > i && a.r > b.l && b.r > a.l && a.b > b.t && b.b > a.t));
    if (reach.length !== 5 || touch) errors.push(`home champion: ${reach.length} pieces${touch ? ", two touch" : ""}`);
  }
  // Home is a table state (arena#31, arena#51): PLAY sits in the tray, in Fight's place, under the champion's line.
  const [champBox, playBox] = [await page.getByTestId("home-their").boundingBox(), await page.getByTestId("play").boundingBox()];
  if (!(playBox.y > champBox.y + champBox.height * 0.8)) errors.push("home: Play isn't under the champion's line");
  {
    // At 1024px the champion's 5 pieces stay on one row.
    const size = page.viewportSize();
    await page.setViewportSize({ width: 1024, height: size.height });
    const tops = await page.getByTestId("home-their").locator(".pc").evaluateAll((els) => els.map((e) => Math.round(e.getBoundingClientRect().top)));
    if (tops.length !== 5 || new Set(tops).size > 1) errors.push(`home at 1024px: the champion's pieces wrap (${tops.join(",")})`);
    await shot("home-1024");
    await page.setViewportSize(size);
  }

  await page.getByTestId("stats").click();
  await page.getByTestId("stats-champions").waitFor();
  await shot("stats"); await noHScroll("stats"); await wide("stats", 900);
  await page.getByTestId("codex-back").click();
  // The Codex uses the width: its unit grid runs many cards a row; Esc goes back.
  await page.getByTestId("codex").click();
  await page.getByTestId("codex-units").waitFor();
  await shot("codex"); await noHScroll("codex"); await wide("codex", 1200);
  // The Codex is a window over the dimmed table (arena#28/#32; Home is a table state, arena#31): the kept Home
  // sits under it, the window is framed inside the viewport, and its unit grid fits the window.
  const win = await page.locator('#app[data-screen="codex"]').boundingBox();
  {
    const under = await page.evaluate(() => ({ cls: document.body.classList.contains("over-table"), home: !!document.querySelector('#under [data-testid="home"]'), dim: getComputedStyle(document.getElementById("under") ?? document.body).filter }));
    if (!under.cls || !under.home) errors.push(`codex: the window is not over the kept table (over-table ${under.cls}, Home under it ${under.home})`);
    else if (!/brightness/.test(under.dim)) errors.push(`codex: the table under the window is not dimmed (${under.dim})`);
    if (!win || win.x < 8 || win.y < 8 || win.x + win.width > W - 8 || win.x + win.width < W - 80 || win.y + win.height > H) errors.push(`codex: the window is not framed in the viewport (${JSON.stringify(win)})`);
    // The grid fits the window: no horizontal overflow, every card inside it, no two cards touching.
    const grid = await page.getByTestId("codex-units").evaluate((g) => {
      const win = document.querySelector('#app[data-screen="codex"]').getBoundingClientRect();
      const cards = [...g.querySelectorAll('[data-testid="codex-unit"]')].slice(0, 24).map((e) => e.getBoundingClientRect());
      const hit = (a, b) => a.left < b.right - 0.5 && b.left < a.right - 0.5 && a.top < b.bottom - 0.5 && b.top < a.bottom - 0.5;
      return {
        over: g.scrollWidth - g.clientWidth, n: cards.length,
        out: cards.filter((r) => r.left < win.left - 0.5 || r.right > win.right + 0.5).length,
        overlap: cards.flatMap((a, i) => cards.slice(i + 1).map((b, j) => (hit(a, b) ? `${i}/${i + 1 + j}` : null))).filter(Boolean),
        page: document.scrollingElement.scrollWidth - document.scrollingElement.clientWidth,
      };
    });
    if (grid.n < 7) errors.push(`codex: only ${grid.n} unit cards drawn`);
    if (grid.over > 0 || grid.page > 0) errors.push(`codex: the unit grid overflows sideways (${grid.over}px in the grid, ${grid.page}px on the page)`);
    if (grid.out) errors.push(`codex: ${grid.out} unit cards stick out of the window`);
    if (grid.overlap.length) errors.push(`codex: unit cards overlap (${grid.overlap.slice(0, 4).join(", ")})`);
  }
  // A unit opens in the inspector on the right (no overlay on desktop), which lies inside the window's right edge.
  await page.getByTestId("codex-unit").nth(2).click();
  await page.locator('[data-testid="inspector"] [data-testid="unit-sheet"]').waitFor({ timeout: 2_000 }).catch(() => errors.push("codex: a unit doesn't open in the inspector"));
  if (await page.getByTestId("overlay").count()) errors.push("codex: a unit opened in an overlay");
  {
    const box = await page.getByTestId("inspector").boundingBox();
    const right = win ? win.x + win.width : W;
    // Inside the window (its 4px border and padding take the last stretch), and not stopping short of it.
    if (!box || box.x + box.width > right + 0.5 || box.x + box.width < right - 48 || box.x < (win?.x ?? 0) || box.y < (win?.y ?? 0) - 0.5) errors.push(`codex: the inspector is not inside the window's right edge (${box ? `${Math.round(box.x)}–${Math.round(box.x + box.width)}px` : "?"}, the window ends at ${Math.round(right)}px)`);
  }
  await page.locator('[data-testid="inspector"] [data-testid="see-awoken"]').click().catch(() => {});
  await shot("codex-inspector");
  // R3-5: the Summoned chip; a summon opens in the inspector with "Summoned by".
  await page.getByTestId("codex-tier-summoned").click();
  if ((await page.getByTestId("codex-summon").count()) !== 8) errors.push(`codex: Summoned shows ${await page.getByTestId("codex-summon").count()} cards, not 8`);
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
  // A run's theme: muffled in the shop (its low-pass closes within a second).
  await page.getByTestId("fight").waitFor({ timeout: 10_000 });
  await page.waitForFunction(() => /^theme\d\.m4a$/.test(window.__music?.().track ?? "") && window.__music().filterHz < 1000, null, { timeout: 5_000 })
    .catch(async () => errors.push(`music: the shop doesn't play a muffled theme (${JSON.stringify(await page.evaluate(() => window.__music?.()))})`));
  const theme = await page.evaluate(() => window.__music?.().track);

  let round = 0;
  let dragged = false;
  let sold = false;
  let resultShot = false;
  for (let guard = 0; guard < 20; guard++) {
    await page.getByTestId("fight").waitFor({ timeout: 10_000 });
    round++;
    const crown = (await page.getByTestId("gold").count()) === 0;
    if (round === 2 && !crown && (await gold()) >= 1 && (await page.getByTestId("offers").locator(".cell").count()) >= 2) {
      // Lock (R3-12): click offer 2, L locks it and it stays chosen; R keeps it
      // at the left, still locked; right-click unlocks it.
      const offer = (s) => page.getByTestId(`offer-${s}`);
      const locked = (s) => offer(s).evaluate((el) => el.classList.contains("locked"));
      const name = await offer(1).locator(".name").textContent();
      await offer(1).click();
      await page.getByTestId("inspector").getByTestId("lock").waitFor();
      if (!(await page.getByTestId("inspector").getByTestId("lock").textContent()).startsWith("🔒 Lock · L")) errors.push("lock: no 🔒 Lock · L in the inspector");
      await page.keyboard.press("l");
      await settle();
      if (!(await locked(1))) errors.push("lock: L didn't lock the chosen offer");
      // Lock keeps the offer in hand (arena#47): the inspector stays on it and now offers Unlock.
      await page.getByTestId("inspector").getByTestId("lock").waitFor();
      if (!(await offer(1).evaluate((el) => el.classList.contains("selected")))) errors.push("lock: Lock put the offer down");
      if (!(await page.getByTestId("inspector").getByTestId("lock").textContent()).startsWith("🔓 Unlock")) errors.push("lock: the locked offer's inspector doesn't say Unlock");
      await page.keyboard.press("Escape");
      // M5-6: the keys line moved to Codex › How to play; the L sits on the inspector's Lock (checked above).
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
      // The offers are pieces in cells (#1293): one row, each at least 112 px wide.
      { const box = await page.getByTestId("offers").locator(".cell").first().boundingBox(); if (!box || box.width < 112) errors.push(`shop offers: cell ${box ? Math.round(box.width) : "missing"}px wide, want ≥ 112`); }
      await onScreen("shop: Fight", page.getByTestId("fight"));
      await onScreen("shop: last offer", page.getByTestId("offers").locator(".cell").last());
      // A tap on an offer reads it in the inspector, with its Buy (arena#47: the inspector is one fixed surface in the tray; selecting changes its words only).
      const still = ["line", "bench", "offers", "reroll", "fight", "inspector"];
      const boxes = async () => Object.fromEntries(await Promise.all(still.map(async (id) => [id, await page.getByTestId(id).boundingBox()])));
      const beforeTap = await boxes();
      // (A right-click Lock keeps its offer in hand: tap only when the first offer is not already the one read.)
      const firstOffer = page.getByTestId("offers").locator(".cell").first();
      if (!(await firstOffer.evaluate((el) => el.classList.contains("selected")))) await firstOffer.click();
      await page.getByTestId("inspector").getByTestId("unit-sheet").waitFor();
      // No Buy button (arena#49): the gold destination wears the price and is the purchase.
      await page.getByTestId("buy").waitFor();
      if (await page.getByTestId("inspector").locator("button", { hasText: /Buy|Купить/ }).count()) errors.push("shop: the inspector still has a Buy button");
      await shot("shop-offer-inspector"); await noOverlay("offer");
      await onScreen("shop: inspector", page.getByTestId("inspector"));
      const [lineBox, insp] = [await page.getByTestId("line").boundingBox(), await page.getByTestId("inspector").boundingBox()];
      if (!(insp.y >= lineBox.y + lineBox.height)) errors.push("shop: the inspector covers the line");
      const afterTap = await boxes();
      for (const id of still) { const [a, b] = [beforeTap[id], afterTap[id]]; if (!a || !b || ["x", "y", "width", "height"].some((k) => Math.abs(a[k] - b[k]) > 0.5)) errors.push(`shop: selecting an offer moved or resized ${id}`); }
      await page.keyboard.press("Escape");
      // A tap on the first offer, then on its gold destination, buys it: one gold tap, and the receipt says so.
      const before = await lineCount();
      await page.getByTestId("offers").locator(".cell").first().click();
      await page.getByTestId("buy").click();
      await settle();
      if ((await lineCount()) !== before + 1) errors.push("shop: a tap on the gold destination didn't buy");
      if (!(await page.getByTestId("receipt").count())) errors.push("shop: no receipt line after a buy");
      await dropOwnGift();
    }
    // Buy with the number keys while gold lasts (and the line has room).
    for (let k = 0; k < 4 && !crown; k++) {
      const g = await gold();
      const offers = await page.getByTestId("offers").locator(".cell").count();
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
      // →: the first offer in hand; the digit of the first empty slot buys it there.
      await page.keyboard.press("ArrowRight");
      // Its slot number commits the buy, on the gold destination only (a twin's slot when you own one).
      const digit = await page.getByTestId("buy").evaluate((el) => { const m = /^(line|bench)-(\d+)/.exec(el.closest(".cell").dataset.testid); return m[1] === "line" ? +m[2] + 1 : 5 + +m[2] + 1; }).catch(() => n + 1);
      await page.keyboard.press(String(digit));
      await settle();
      if ((await gold()) === g0) { errors.push(`round ${round}: → then ${digit} didn't buy`); break; }
      await wantSfx("key → 1–5 (buy)", /^(coin|merge|level-up)$/);
      // A copy that awakens a unit brings a gift; its chooser (checked below) blocks the keys: skip it.
      await dropOwnGift();
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
      await page.getByTestId("inspector").getByTestId("move").waitFor();
      await shot("shop-selected"); await noOverlay("selected");
      await page.keyboard.press("ArrowLeft");
      await settle();
      if ((await page.getByTestId("line-0").locator(".name").textContent()) !== name0) errors.push("← didn't move the selected unit");
      await wantSfx("← (move)", /^move-left$/);
      if (!(await page.getByTestId("line-0").evaluate((el) => el.classList.contains("selected")))) errors.push("the moved unit isn't selected anymore");
      // Esc deselects.
      await page.keyboard.press("Escape");
      if (await page.locator('[data-testid="line"] .cell.selected').count()) errors.push("Esc didn't deselect");
      // With nothing left to step back from, Esc opens the ☰ run menu (R2-10); Esc again closes it.
      await page.keyboard.press("Escape");
      await page.getByTestId("run-menu").waitFor({ timeout: 2_000 }).catch(() => errors.push("Esc didn't open the run menu"));
      if (!(await page.getByTestId("run-menu").getByTestId("sound-toggle").count())) errors.push("sound: no Sound row in the run menu");
      await shot("run-menu");
      // ☰ Codex opens as a window over the dimmed table; Back finds the table as it was, at any window size.
      await page.getByTestId("menu-codex").click();
      await page.getByTestId("codex-units").waitFor();
      if (!(await page.locator("#under [data-testid=stage]").count())) errors.push("codex: the table isn't kept dimmed under the window");
      await page.setViewportSize({ width: 900, height: H });
      await page.waitForTimeout(200);
      await page.getByTestId("codex-back").click();
      await page.getByTestId("fight").waitFor();
      if (!(await page.getByTestId("stage").count()) || (await page.locator("#under").count())) errors.push("codex → 900px → Back: the table isn't back");
      await page.setViewportSize({ width: W, height: H });
      await page.waitForTimeout(200);
      await page.keyboard.press("Escape");
      await page.getByTestId("run-menu").waitFor({ timeout: 2_000 }).catch(() => {});
      await page.keyboard.press("Escape");
      if (await page.getByTestId("run-menu").count()) errors.push("Esc didn't close the run menu");
    }
    if (crown) { await shot("crown-shop"); await noHScroll("crown-shop"); }
    // Space fights. Round 2 listens to it (round 3, note 16): the start
    // sting, wave sounds while it plays, nothing on a step, one end sound.
    if (round === 2) await page.evaluate(() => { window.__sfx = []; });
    await dropOwnGift();
    await page.keyboard.press("Space");
    await page.getByTestId("battle-end").waitFor({ timeout: 10_000 });
    if (round === 2) await battleSounds();
    if (round === 1) await shot("battle");
    await runBoardFight(round, crown);
    if (round === 1) await whyOnDesktop();
    if (round === 1) await desktopBattle();
    if (round === 1) await beamChecks(page, errors, { shot, phone: false });
    if (round === 1) await nowSheetChecks(page, errors, { shot, phone: false });
    if (!(await page.getByTestId("end-card").isVisible().catch(() => false))) await showEnd();
    // R2-17 batch E: the end card is the fight's one result: the round
    // fought, hearts and record, and its button goes straight on.
    await page.getByTestId("end-card").waitFor({ timeout: 10_000 });
    await runBoardResult(round, crown);
    const runLine = (await page.getByTestId("end-run").textContent().catch(() => "")) ?? "";
    if (!(crown ? /CROWN/ : new RegExp(`R${round}/12`)).test(runLine) || !/[♥♡]/.test(runLine) || !/\d+W/.test(runLine)) errors.push(`end card: run line "${runLine}" after round ${round}`);
    const done = (await page.getByTestId("battle-done").getAttribute("aria-label")) ?? "";
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
    // R4-5: a read pane beside the cards shows the first card's sheet, and
    // hovering another card reads that one there.
    const pane = page.getByTestId("gift-read");
    const nameOf = (i) => page.getByTestId(`gift-card-${i}`).locator(".name").textContent();
    const reads = async (i) => (await pane.getAttribute("data-card")) === String(i) && (await pane.getByTestId("unit-sheet").count()) === 1 && (await pane.textContent()).includes(await nameOf(i));
    if (!(await pane.isVisible())) errors.push("gift: no read pane on desktop");
    else if (!(await reads(0))) errors.push("gift: the read pane doesn't show the first card's sheet by default");
    await shot("gift-chooser");
    if (await pane.isVisible()) {
      await page.getByTestId("gift-card-2").hover();
      if (!(await reads(2))) errors.push("gift: hovering the third card doesn't read it in the pane");
      await shot("gift-hover-3");
      await page.getByTestId("gift-card-1").hover();
      if (!(await reads(1))) errors.push("gift: hovering the second card doesn't read it in the pane");
      if (await page.locator('.overlay [data-testid="sheet-close"]').count()) errors.push("gift: hovering opened a sheet over the chooser");
    }
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
    // Rules v2 (#1049): the pair fuses only with a design; give it one (as the
    // night batch would), claimed by this player.
    await call("POST", "/dev/codex-pair", { a: run.line.find((u) => u.form === "awoken").unitId, b: almost.unitId, state: "discovered" }, fuser.id);
    // A gift still waiting in this run opens over the shop: skip it first.
    await dropOwnGift();
    // Tapping the offer shows "Awakens!" in the inspector.
    await page.getByTestId(`offer-${run.offers[key - 1].slot}`).click();
    await page.getByTestId("inspector").getByTestId("buy-preview").waitFor();
    if (!/Awakens/.test(await page.getByTestId("inspector").getByTestId("buy-preview").textContent())) errors.push("awaken preview: no 'Awakens!' in the inspector");
    await shot("awaken-preview");
    // The slot digit of the twin it merges into.
    await page.keyboard.press(String(run.line.findIndex((u) => u.uid === almost.uid) + 1));
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
    await page.locator(".cell.fusable").first().waitFor();
    await shot("fuse-pick");
    await page.getByTestId(`line-${second}`).click();
    await page.getByTestId("inspector").getByTestId("forge-discovered").waitFor();
    await noOverlay("fusion forge");
    // The forge, never a splice preview (#1049 reopened, case 5).
    if ((await page.getByTestId("inspector").locator('[data-testid="preview-swap"], [data-testid="fusion-recipe"]').count()) > 0) errors.push("fusion: the forge shows a splice recipe or Swap");
    await shot("fusion-forge");
    await page.getByTestId("preview-confirm").click();
    await page.getByTestId("line").locator(".cell.fused").waitFor();
    if (!/\bfuse\b/.test((await page.evaluate(() => window.__sfx ?? [])).slice(-2).join(" "))) errors.push("sound: the fuse made no sound");
    // A pair nobody had made reveals its name (R2-5); Esc closes it.
    if (await page.getByTestId("fusion-reveal").isVisible().catch(() => false)) {
      await wantSfx("the discovery reveal", /^discover$/);
      await shot("fusion-reveal");
      await page.keyboard.press("Escape");
      await page.getByTestId("fusion-reveal").waitFor({ state: "detached" });
    }
    await page.getByTestId("line").locator(".cell.fused").click();
    await page.getByTestId("inspector").getByTestId("unit-sheet").waitFor();
    if (!/discovered by (you|@\S+)/.test(await page.getByTestId("inspector").textContent())) errors.push("fused: no discovery credit in the inspector");
    if ((await page.getByTestId("bench").locator(".cell.you").count()) !== 0) errors.push("bench: the fused bench unit is still on the bench");
    await shot("fused");
    // Drag between line and bench (R3-14): line-0 onto the bench's empty slot, then back onto the line's.
    const front = await page.getByTestId("line-0").locator(".name").textContent();
    await page.getByTestId("line-0").dragTo(page.getByTestId("bench-0-empty"));
    await page.getByTestId("bench-0").waitFor();
    if ((await page.getByTestId("bench-0").locator(".name").textContent()) !== front) errors.push(`bench: dragging line-0 to the bench put "${await page.getByTestId("bench-0").locator(".name").textContent()}" there, not "${front}"`);
    await shot("bench-dragged");
    await page.getByTestId("bench-0").dragTo(page.getByTestId("line").locator(".cell.empty").first());
    await page.getByTestId("bench-0-empty").waitFor();
    const back = await page.getByTestId("line").locator(".cell.you").last().locator(".name").textContent();
    if (back !== front) errors.push(`bench: dragging back put "${back}" at the line's end, not "${front}"`);
  }

  // 1024px is still desktop (120px cards, 340px inspector, nothing cut off); 1023px is the phone.
  // (The shop is on screen: the fusion run's, or a new run's when that setup failed.)
  await page.setViewportSize({ width: 1024, height: 768 });
  if (await page.getByTestId("play").count()) await page.getByTestId("play").click();
  await page.getByTestId("fight").waitFor();
  await shot("shop-1024");
  if (!(await page.getByTestId("stage").count())) errors.push("1024px: no table");
  const w1024 = await page.evaluate(() => document.documentElement.scrollWidth);
  if (w1024 > 1024) errors.push(`1024px: horizontal scroll (${w1024}px)`);
  const line1024 = await page.getByTestId("line").locator(".cell").evaluateAll((els) => new Set(els.map((e) => Math.round(e.getBoundingClientRect().top))).size);
  if (line1024 !== 1) errors.push("1024px: the line wraps");
  await page.setViewportSize({ width: 1023, height: 768 });
  await page.waitForTimeout(200);
  if (!(await page.getByTestId("stage").count())) errors.push("1023px: no table");
  await shot("shop-1023");
  /** M5-4 (#892): the fight plays on the run board. The run's track (its
   * dots and the Crown), hearts and gold stay in the top bar; the fight
   * fills the board's main area, left of the docked column (Why/Log), never
   * a full-screen swap. A fighting card's click shows that card in the
   * column. The fought round's dot stays "now" until the end card. */
  async function runBoardFight(round, crown) {
    const bar = page.getByTestId("run-bar");
    if (!(await bar.isVisible().catch(() => false))) return void errors.push(`run board: round ${round}'s fight has no run bar (track, hearts, gold)`);
    if (!(await bar.locator(".track .track-dot.crown").count())) errors.push("run board: the fight's top bar has no run track ending in the Crown");
    if (!(await bar.locator(".heart-icons").count())) errors.push("run board: the fight's top bar has no hearts");
    if (!crown && !(await bar.getByTestId("gold").count())) errors.push("run board: the fight's top bar has no gold");
    const now = await bar.locator(".track-dot.now").count();
    if (now !== 1) errors.push(`run board: ${now} dots are "now" during round ${round}'s fight`);
    const [b, board] = [await bar.boundingBox(), await page.getByTestId("battle-you").boundingBox()];
    if (b && board && board.y < b.y + b.height) errors.push("run board: the fight sits over the top bar");
    if (round !== 1) return;
    // A fighting card (arena#29) shows in the inspector over the table, with its rule and whose idea it was; Esc closes it first.
    const card = page.locator('[data-testid="battle-you"] .bv-card').first();
    const face = await card.locator(".pc-face").first().boundingBox();
    if (!face) return;
    await page.mouse.click(face.x + face.width / 2, face.y + face.height / 2);
    const insp = page.locator('[data-testid="inspector"] [data-testid="now-sheet"]');
    await insp.waitFor({ timeout: 2_000 }).catch(() => errors.push("run board: a fighting card's click didn't show it in the inspector"));
    if (await page.getByTestId("overlay").isVisible().catch(() => false)) errors.push("run board: a fighting card opened a pop-up, not the inspector");
    if (!(await page.locator('[data-testid="inspector"] [data-testid="now-ability"]').count())) errors.push("run board: the inspector doesn't read the unit's rule");
    await shot("battle-card-inspector");
    await page.keyboard.press("Escape");
    await insp.waitFor({ state: "detached", timeout: 2_000 }).catch(() => errors.push("run board: Esc didn't close the inspector"));
    if (await page.getByTestId("run-menu").count()) { errors.push("run board: Esc on the inspector opened the run menu"); await page.keyboard.press("Escape"); }
  }
  /** M5-4: the result in place: the bar stays, the fought round's dot turns won or lost, the end card sits on the board. */
  async function runBoardResult(round, crown) {
    const bar = page.getByTestId("run-bar");
    if (!(await bar.isVisible().catch(() => false))) return void errors.push(`run board: round ${round}'s result has no run bar`);
    if (!crown) {
      const cls = (await bar.locator(".track-dot").nth(round - 1).getAttribute("class").catch(() => "")) ?? "";
      if (!/\b(win|loss|draw)\b/.test(cls)) errors.push(`run board: round ${round}'s dot is "${cls}" on the end card, not won or lost`);
    }
    const [b, e] = [await bar.boundingBox(), await page.getByTestId("end-card").boundingBox()];
    if (b && e && e.y < b.y + b.height) errors.push("run board: the end card covers the top bar");
  }

  /** Round 3, note 16: a battle's sounds, read off window.__sfx. It opens
   * with "start", plays a hit, zap or block as waves land, makes none on ←
   * (a step) or a timeline jump, and plays its end sound once: not again
   * after a Replay played out to the end card. */
  async function battleSounds() {
    const sfx = () => page.evaluate(() => window.__sfx ?? []);
    // The shop's theme plays on in battle, opened up, not restarted.
    await page.waitForFunction((t) => window.__music?.().track === t && window.__music().filterHz > 5000, theme, { timeout: 5_000 })
      .catch(async () => errors.push(`music: the battle doesn't open the shop's theme (${JSON.stringify(await page.evaluate(() => window.__music?.()))})`));
    if ((await sfx())[0] !== "start") errors.push(`battle sound: it opened with "${(await sfx())[0] ?? "nothing"}", not start`);
    await page.waitForFunction(() => (window.__sfx ?? []).some((k) => /^(hit|zap|block)$/.test(k)), null, { timeout: 15_000 })
      .catch(async () => errors.push(`battle sound: no hit, zap or block while it played (${(await sfx()).join(", ")})`));
    await page.keyboard.press("ArrowLeft");
    const before = (await sfx()).length;
    await page.keyboard.press("ArrowLeft");
    await page.keyboard.press("ArrowRight");
    await page.waitForTimeout(1_500);
    const after = await sfx();
    if (after.length !== before) errors.push(`battle sound: ← / → (a step) played ${after.slice(before).join(", ")}`);
    const ends = (list) => list.filter((k) => /^(win|lose|draw)$/.test(k)).length;
    await page.getByTestId("battle-end").click();
    await page.getByTestId("end-card").waitFor({ timeout: 10_000 });
    await page.waitForTimeout(700); // a slay's discovery chime comes 0.5 s later
    const first = await sfx();
    if (ends(first) !== 1) errors.push(`battle sound: the end card played ${ends(first)} end sounds (${first.slice(-3).join(", ")})`);
    // Replay, then End again: the end sound doesn't play twice.
    await page.getByTestId("end-replay").click();
    await page.waitForTimeout(600);
    await page.getByTestId("battle-end").click();
    await page.getByTestId("end-card").waitFor({ timeout: 10_000 });
    await page.waitForTimeout(300);
    const again = await sfx();
    if (ends(again) !== 1) errors.push(`battle sound: the end sound played again after a Replay (${again.slice(-4).join(", ")})`);
    if (again.includes("start", 1)) errors.push("battle sound: Replay played the start sting again");
    console.log(`battle sound: ${first.length} sounds, ${[...new Set(first)].join(" ")}`);
  }

  /** The battle on the table (arena#29): the stage fits the window with the tray's controls on screen; the lines
   * stand one above the other, fronts at the left; the result covers the tray (its Next round on screen); Space
   * pauses, ← → step, E ends, Esc closes the inspector. */
  async function desktopBattle() {
    // A fight with no firing in it (whyOnDesktop stepped through it looking for one) ends before the keys are tried: replay it, or Space would go on to the shop.
    if (await page.getByTestId("end-card").isVisible().catch(() => false)) {
      await page.getByTestId("end-replay").click();
      await page.waitForTimeout(600);
    }
    if ((await page.getByTestId("battle-play").textContent()) === "❚❚") await page.getByTestId("battle-play").click();
    const facing = async (name, w, h) => {
      const [mine, theirs, tray] = [await page.getByTestId("battle-you").boundingBox(), await page.getByTestId("battle-them").boundingBox(), await page.getByTestId("tray").boundingBox()];
      if (!mine || !theirs || mine.y < theirs.y + theirs.height - 2 || Math.abs(mine.x - theirs.x) > 2) errors.push(`${name}: the lines don't stand one above the other (${JSON.stringify({ mine, theirs })})`);
      if (!tray || mine.y + mine.height > tray.y + 0.5) errors.push(`${name}: the tray covers your line`);
      const xs = async (testid) => page.getByTestId(testid).locator(".bv-slot").evaluateAll((els) => els.map((e) => e.getBoundingClientRect().x));
      const [m, t] = [await xs("battle-you"), await xs("battle-them")];
      if (m.length > 1 && !(m[0] < m.at(-1))) errors.push(`${name}: your front isn't at the left`);
      if (t.length > 1 && !(t[0] < t.at(-1))) errors.push(`${name}: their front isn't at the left`);
      const doc = await page.evaluate(() => [document.documentElement.scrollWidth, document.documentElement.scrollHeight]);
      if (doc[0] > w || doc[1] > h) errors.push(`${name}: the fight scrolls (${doc.join("×")})`);
      const endId = (await page.getByTestId("battle-end").count()) ? "battle-end" : "battle-next";
      for (const id of ["battle-back", "battle-play", "battle-step", "battle-speed", endId, "battle-replay"]) {
        const b = await page.getByTestId(id).boundingBox();
        if (!b || b.y < 0 || b.y + b.height > h + 0.5 || b.x + b.width > w + 0.5) errors.push(`${name}: ${id} off screen`);
      }
    };
    await facing("battle 1440×900", W, H);
    // Keys: ← → step, Space plays and pauses, E ends.
    const turn = async () => (await page.getByTestId("battle-turn").textContent()) ?? "";
    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("ArrowRight");
    const stepped = await turn();
    await page.keyboard.press("ArrowLeft");
    if ((await page.getByTestId("battle-play").textContent()) !== "▶") errors.push("keys: stepping with ← → left the battle playing");
    await page.keyboard.press("Space");
    if ((await page.getByTestId("battle-play").textContent()) !== "❚❚") errors.push("keys: Space didn't play the paused battle");
    await page.keyboard.press("Space");
    if ((await page.getByTestId("battle-play").textContent()) !== "▶") errors.push("keys: Space didn't pause the battle");
    console.log(`keys: stepped to ${stepped}, Space plays and pauses`);
    await page.keyboard.press("e");
    await page.getByTestId("end-card").waitFor({ timeout: 5_000 }).catch(() => errors.push("keys: E didn't end the battle"));
    // The verdict sits on the loser's row (arena#53); the tray keeps its controls, Next among them, and the winner's line stays clear.
    const endBox = await page.getByTestId("end-card").boundingBox();
    const next = await page.getByTestId("battle-done").boundingBox();
    const trayBox = await page.getByTestId("tray").boundingBox();
    if (!endBox || !next || !trayBox || next.y + next.height > H + 0.5 || next.x < trayBox.x - 0.5 || next.x + next.width > trayBox.x + trayBox.width + 0.5 || next.y < trayBox.y - 0.5) errors.push(`end card: Next isn't inside the tray on screen (${JSON.stringify({ trayBox, next })})`);
    if (endBox && trayBox && endBox.y + endBox.height > trayBox.y + 0.5) errors.push("end card: it covers the tray");
    const loserRow = await page.getByTestId("end-card").getAttribute("data-row");
    const winnerBox = await page.getByTestId(loserRow === "theirs" ? "battle-you" : "battle-them").boundingBox();
    if (endBox && winnerBox && endBox.y < winnerBox.y + winnerBox.height - 0.5 && endBox.y + endBox.height > winnerBox.y + 0.5) errors.push("end card: it covers the winner's line");
    await shot("battle-end-card");
    // The card's ✕ puts it away: the control bar is back, and its way on takes End's place.
    await page.getByTestId("end-close").click();
    await page.getByTestId("battle-back").waitFor();
    await facing("battle 1440×900 after the result", W, H);
    // 1024×768: still whole.
    await page.setViewportSize({ width: 1024, height: 768 });
    await page.waitForTimeout(200);
    await facing("battle 1024×768", 1024, 768);
    await shot("battle-1024");
    await page.setViewportSize({ width: W, height: H });
    await page.waitForTimeout(200);
    await page.getByTestId("trace-close").click().catch(() => {});
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
          if (!panel || panel.x < 0 || panel.y < 0 || panel.x + panel.width > W + 0.5 || panel.y + panel.height > H + 0.5) errors.push(`why: the window ${JSON.stringify(panel)} isn't on screen`);
          await shot("battle-why"); await noHScroll("battle-why");
          const hud = () => page.getByTestId("battle-turn").textContent();
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
