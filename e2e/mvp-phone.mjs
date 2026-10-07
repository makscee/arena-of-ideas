// MVP phone e2e (mission #574 verification ladder): a whole game at 360×640
// in Chromium, with a screenshot of every screen. Without --url it builds the
// mobile client and starts the MVP server on a free port.
//   npm run mvp:phone -- [--url https://m1.twin-pogona.ts.net/arena/] [--out e2e/.shots/mvp]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";
import { escPass } from "./esc-keys.mjs";
import { beamChecks } from "./beams.mjs";
import { nowSheetChecks } from "./now-sheet.mjs";

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

// A tag per pass keeps a second pass at one server apart (names may repeat).
const TAG = Date.now().toString(36).slice(-4);
const browser = await launchChromium();
const errors = [];
let shots = 0;
try {
  const page = await browser.newPage({ viewport: { width: 360, height: 640 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && errors.push(`console: ${m.text()}`));
  /** Every player name on screen (ui/dom.ts who()) sits on one line: never
   * "@bot-" / "3" across two (#587). */
  const namesOneLine = async (name) => {
    const split = await page.evaluate(() => [...document.querySelectorAll(".who")].filter((el) => el.getClientRects().length > 1 || el.getBoundingClientRect().height > parseFloat(getComputedStyle(el).fontSize) * 2).map((el) => el.textContent));
    for (const n of split) errors.push(`${name}: the name "${n}" splits across lines`);
  };
  /** Every card's When · Who · Does icon line fits its card (R3-4): no row
   * overflows, and a line with more icons than it shows ends in "+". A
   * battle card shows only its When, in the corner, so it is left out. */
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
  const shot = async (name) => { await page.screenshot({ path: `${out}/${String(++shots).padStart(2, "0")}-${name}.png` }); await namesOneLine(name); await iconsFit(name); };
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
  /** On every acting battle card, the name stays inside the gold ring (1px
   * border + 2px ring) with 1px of air. */
  const nameInRing = async (name) => {
    const bad = await page.evaluate(() => [...document.querySelectorAll(".bv-card.acting .name")].map((el) => {
      const card = el.closest(".bv-card").getBoundingClientRect();
      const t = el.getBoundingClientRect();
      return t.left < card.left + 4 - 0.5 || t.right > card.right - 4 + 0.5 ? `${el.textContent} at ${Math.round(t.left - card.left)}..${Math.round(card.right - t.right)}px from the card's edges` : null;
    }).filter(Boolean));
    for (const b of bad) errors.push(`${name}: acting card's name in the ring: ${b}`);
  };
  /** Rates are a hint (round 2, R2-7): no percentage on screen except a unit
   * sheet's last line (data-testid unit-rates). The Stats page isn't checked. */
  const noRates = async (name) => {
    const found = await page.evaluate(() => {
      const out = [];
      const walk = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
      for (let n = walk.nextNode(); n; n = walk.nextNode()) {
        const el = n.parentElement;
        if (!/\d\s*%/.test(n.textContent) || !el || el.closest('[data-testid="unit-rates"]') || !el.getClientRects().length) continue;
        out.push(n.textContent.trim());
      }
      return out;
    });
    for (const t of found) errors.push(`${name}: a rate outside the sheet's last line: "${t}"`);
  };
  /** A unit sheet shows one form; a sleeping unit's "See Awoken" swaps in the
   * awoken text and Back returns; rates, if any, are its last line. */
  const sheetChecks = async (name) => {
    const sheet = page.getByTestId("unit-sheet");
    if ((await sheet.getByTestId("sheet-form").count()) !== 1) errors.push(`${name}: not exactly one form block`);
    await noRates(name);
    const last = await sheet.evaluate((el) => el.lastElementChild?.dataset.testid ?? "");
    if ((await sheet.getByTestId("unit-rates").count()) && last !== "unit-rates") errors.push(`${name}: rates aren't the last line`);
    await termChecks(name, sheet.getByTestId("sheet-form"));
    if (await sheet.getByTestId("see-awoken").count()) {
      const before = await sheet.getByTestId("sheet-form").textContent();
      await sheet.getByTestId("see-awoken").click();
      await sheet.locator(".sheet-form.other").waitFor();
      const after = await sheet.getByTestId("sheet-form").textContent();
      if (!/^Awoken/i.test(after) || after.includes(before)) errors.push(`${name}: See Awoken shows "${after}"`);
      if (!(await sheet.locator(".sheet-form.other u.changed").count())) errors.push(`${name}: See Awoken underlines nothing`);
      await shot(`${name.replace(/ /g, "-")}-awoken`); await noHScroll(name);
      await sheet.getByTestId("see-sleeping").click();
      if ((await sheet.getByTestId("sheet-form").textContent()) !== before) errors.push(`${name}: Back doesn't restore the sleeping text`);
    }
  };
  /** Unit text is highlighted (R2-8): its terms are buttons, and tapping one
   * (Shield when the text has it) opens a sheet with its rule. */
  let tappedShield = false;
  const termChecks = async (name, form) => {
    const terms = form.getByTestId("term");
    if (!(await terms.count())) { errors.push(`${name}: no highlighted terms in the unit text`); return; }
    const shield = form.locator('[data-term="status:Shield"]');
    const pick = (await shield.count()) ? shield.first() : terms.first();
    const term = await pick.getAttribute("data-term");
    await pick.click();
    const tsheet = page.getByTestId("term-sheet");
    await tsheet.waitFor({ timeout: 3000 }).catch(() => errors.push(`${name}: tapping ${term} opens no rule sheet`));
    if (await tsheet.count()) {
      const tip = (await tsheet.getByTestId("term-tip").textContent()) ?? "";
      if (tip.length < 10) errors.push(`${name}: ${term}'s rule sheet has no rule ("${tip}")`);
      if (term === "status:Shield" && !tappedShield) { tappedShield = true; await shot("term-shield"); }
      await page.locator('.sheet:has([data-testid="term-sheet"]) [data-testid="sheet-close"]').click();
      if (await tsheet.count()) errors.push(`${name}: the rule sheet doesn't close`);
    }
  };
  /** The page doesn't scroll at 640 px. */
  const noVScroll = async (name) => {
    const hgt = await page.evaluate(() => document.documentElement.scrollHeight);
    if (hgt > 640 + 0.5) errors.push(`${name}: scrolls (${hgt}px tall)`);
  };
  /** The element is on screen without scrolling (the bottom of a 640 px phone). */
  const onScreen = async (name, locator) => {
    const box = await locator.boundingBox();
    if (!box || box.y + box.height > 640 + 0.5 || box.y < 0) errors.push(`${name}: off screen (${box ? Math.round(box.y + box.height) : "none"}px)`);
  };

  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").waitFor({ timeout: 10_000 });
  await shot("name"); await noHScroll("name");
  await page.getByTestId("name-input").fill(`PhoneTester${TAG}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").waitFor();
  await shot("home"); await noHScroll("home"); await noRates("home"); await onScreen("home: Play", page.getByTestId("play"));
  await tap44("dev summary", page.locator("details.dev summary"));
  await page.getByTestId("rules-open").click();
  await page.getByTestId("rules").waitFor();
  await shot("rules"); await noHScroll("rules");
  if (!/game-icons\.net, CC BY 3\.0/.test(await page.getByTestId("icon-credits").textContent().catch(() => ""))) errors.push("rules: no icon credits");
  // The phone's Back (R3-26) closes the sheet and stays in the game.
  const back = () => page.evaluate(() => history.back());
  await back();
  await page.getByTestId("rules").waitFor({ state: "detached", timeout: 3_000 }).catch(() => errors.push("back: Back didn't close the Rules sheet"));
  if (!(await page.getByTestId("play").isVisible().catch(() => false))) errors.push(`back: Back with a sheet open left the game (${page.url()})`);
  await page.getByTestId("rules-open").click();
  await page.getByTestId("rules").waitFor();
  await page.getByTestId("overlay").click({ position: { x: 180, y: 10 } });
  await page.getByTestId("rules").waitFor({ state: "detached" });
  await page.getByTestId("stats").click();
  await page.getByTestId("stats-back").waitFor();
  await shot("stats"); await noHScroll("stats");
  await page.getByTestId("stats-back").click();
  await page.getByTestId("play").waitFor();

  // The Codex (R2-11) from the title menu: tier 3 filtered by a trigger icon,
  // a unit's sheet with See Awoken and its dim rates line; Keywords lists
  // Shield's units; a term's "Open in Codex" lands on its row; Fusions counts
  // the pairs found. Back returns to the title menu.
  await page.getByTestId("codex").click();
  await page.getByTestId("codex-units").waitFor();
  if ((await page.getByTestId("codex-unit").count()) < 60) errors.push(`codex: only ${await page.getByTestId("codex-unit").count()} units`);
  await shot("codex-units"); await noHScroll("codex-units");
  await page.getByTestId("codex-tier-3").click();
  const tier3 = await page.getByTestId("codex-unit").count();
  const trig = page.locator('[data-testid^="codex-trigger-"]').first();
  await tap44("codex trigger", trig); await tap44("codex trigger", trig, "width");
  await trig.click();
  const both = await page.getByTestId("codex-unit").count();
  if (both === 0 || both > tier3) errors.push(`codex: tier 3 + a trigger shows ${both} of ${tier3}`);
  for (const t of await page.getByTestId("codex-unit").locator(".tier").allTextContents()) if (t !== "III") errors.push(`codex: a tier-${t} unit under tier III`);
  await shot("codex-filtered"); await noHScroll("codex-filtered");
  // A filter tap redraws in place: the window keeps its scroll.
  await page.evaluate(() => window.scrollTo(0, 120));
  const y0 = await page.evaluate(() => window.scrollY);
  await page.getByTestId("codex-tier-3").click();
  await page.waitForTimeout(100);
  const y1 = await page.evaluate(() => window.scrollY);
  if (y0 > 0 && Math.abs(y1 - y0) > 2) errors.push(`codex: a filter tap scrolled ${y0} → ${y1}`);
  // Sort by win rate: the rates show on the cards only then, highest first.
  await tap44("codex sort", page.getByTestId("codex-sort-win"));
  await page.getByTestId("codex-sort-win").click();
  await page.getByTestId("codex-rate").first().waitFor();
  {
    const rates = await page.getByTestId("codex-rate").allTextContents();
    if (rates.length !== (await page.getByTestId("codex-unit").count())) errors.push(`codex: ${rates.length} rates on ${await page.getByTestId("codex-unit").count()} cards`);
    const nums = rates.filter((r) => r !== "–").map((r) => Number(r.replace("%", "")));
    if (nums.some((x, i) => i > 0 && x > nums[i - 1])) errors.push(`codex: win-rate sort not highest first (${rates.join(",")})`);
    if (rates.slice(0, nums.length).includes("–")) errors.push("codex: an uncounted unit sorted before counted ones");
  }
  await shot("codex-sort-win"); await noHScroll("codex-sort-win");
  await page.getByTestId("codex-sort-tier").click();
  await page.waitForTimeout(100);
  if (await page.getByTestId("codex-rate").count()) errors.push("codex: rates stay on the cards after Sort: Tier");
  await page.getByTestId("codex-unit").first().click();
  await page.getByTestId("see-awoken").click();
  await page.getByTestId("see-sleeping").waitFor();
  await shot("codex-sheet");
  await page.getByTestId("sheet-close").click();
  // R3-5: Planter's sheet shows the Imp it summons; "Imp (1/2)" opens the Imp's card.
  // Summoned clears the When filter left on above; then every tier.
  await page.getByTestId("codex-tier-summoned").click();
  await page.getByTestId("codex-tier-all").click();
  await page.getByTestId("codex-search").fill("Planter");
  await page.getByTestId("codex-unit").first().click();
  const planterSummons = (await page.getByTestId("sheet-summons").textContent().catch(() => "")) ?? "";
  if (!planterSummons.includes("Imp")) errors.push(`codex: Planter's sheet has no Summons block with the Imp ("${planterSummons}")`);
  await page.locator('[data-testid="sheet-form"] [data-testid="unit-ref"]').first().click();
  await page.getByTestId("summon-sheet").waitFor({ timeout: 2_000 }).catch(() => errors.push("codex: Planter's \"Imp (1/2)\" didn't open the Imp's card"));
  const impHead = (await page.locator('[data-testid="summon-sheet"] h2').textContent().catch(() => "")) ?? "";
  if (impHead !== "👺 Imp") errors.push(`codex: the Imp's sheet is titled "${impHead}"`);
  await shot("codex-summon-sheet");
  while (await page.getByTestId("sheet-close").count()) await page.getByTestId("sheet-close").last().click();
  await page.getByTestId("codex-search").fill("");
  // The Summoned chip: the 5 bodies, tagged S; the Wolf names who summons it.
  await page.getByTestId("codex-tier-summoned").click();
  const summonCards = await page.getByTestId("codex-summon").count();
  if (summonCards !== 8) errors.push(`codex: Summoned shows ${summonCards} cards, not 8`);
  if (await page.getByTestId("codex-unit").count()) errors.push("codex: Summoned shows tiered units");
  await shot("codex-summoned"); await noHScroll("codex-summoned");
  await page.locator('[data-testid="codex-summon"][data-summon="wolf"]').click();
  const wolfBy = (await page.getByTestId("summoned-by").textContent().catch(() => "")) ?? "";
  for (const n of ["Summoner"]) if (!wolfBy.includes(n)) errors.push(`codex: the Wolf's "Summoned by" lacks ${n} ("${wolfBy}")`);
  await shot("codex-summon-wolf");
  await page.getByTestId("sheet-close").click();
  await page.getByTestId("codex-tier-all").click();
  await page.getByTestId("codex-tab-keywords").click();
  const shieldUsers = await page.locator('[data-term="status:Shield"] [data-testid="codex-term-unit"]').count();
  if (shieldUsers === 0) errors.push("codex: Shield lists no units");
  await page.locator('[data-term="status:Shield"]').scrollIntoViewIfNeeded();
  await shot("codex-keywords"); await noHScroll("codex-keywords");
  // R3-2: keywords stand alone, the groups read When / Who / Does, jargon no unit uses is hidden.
  {
    const damage = (await page.locator('[data-term="effect:damage"] .kw-tip').textContent()) ?? "";
    if (/Shield/.test(damage) || !damage) errors.push(`codex: Damage reads "${damage}"`);
    const groups = await page.locator(".kw-group").allTextContents();
    for (const g of ["When", "Who", "Does"]) if (!groups.includes(g)) errors.push(`codex: no "${g}" group (${groups.join(", ")})`);
    for (const t of ["effect:absorbHurt", "effect:preventDeathHeal", "effect:cancel", "effect:consumeStacks", "term:would"])
      if (await page.locator(`.kw-row[data-term="${t}"]`).count()) errors.push(`codex: the unused row ${t} shows`);
  }
  // Dies said of an enemy has its own line, its own rule and its own units.
  {
    if (await page.locator('.kw-row:not([data-term^="trigger:"]) .kw-scope').count()) errors.push("codex: a term that isn't a trigger has scope lines");
    const enemyDies = page.locator('[data-term="trigger:Death"] .kw-scope[data-scope="enemy"]');
    if (!(await enemyDies.count())) errors.push("codex: Dies has no \"an enemy dies\" line");
    else {
      if (!/When an enemy dies/.test(await enemyDies.locator(".kw-tip").textContent())) errors.push(`codex: the enemy-death line reads "${await enemyDies.locator(".kw-tip").textContent()}"`);
      // R3-2: the scoped line has its own label in the card's words.
      if ((await enemyDies.getByTestId("codex-term-scope-label").textContent()) !== "Enemy dies") errors.push(`codex: the enemy-death line's label reads "${await enemyDies.getByTestId("codex-term-scope-label").textContent()}"`);
      if (!(await enemyDies.getByTestId("codex-term-unit").count())) errors.push("codex: the enemy-death line lists no units");
      await enemyDies.scrollIntoViewIfNeeded();
      await shot("codex-dies-scoped");
      // An enemy-death unit's "Open in Codex" lands on that line, not on the self-death rule.
      await enemyDies.getByTestId("codex-term-unit").first().click();
      await page.locator('[data-testid="unit-sheet"] [data-term="trigger:Death"]').first().click();
      await page.getByTestId("term-codex").click();
      await page.locator(".kw-scope.landed").waitFor({ timeout: 2_000 }).catch(() => {});
      if ((await page.locator(".kw-scope.landed").getAttribute("data-scope").catch(() => null)) !== "enemy") errors.push("codex: an enemy-death term's Open in Codex didn't land on its line");
    }
  }
  await page.locator('[data-term="status:Shield"] [data-testid="codex-term-unit"]').first().click();
  await page.locator('[data-testid="unit-sheet"] [data-testid="term"]').first().click();
  const termId = await page.locator('[data-testid="unit-sheet"] [data-testid="term"]').first().getAttribute("data-term");
  await page.getByTestId("term-codex").click();
  await page.locator(".kw-row.landed").waitFor();
  if ((await page.locator(".kw-row.landed").getAttribute("data-term")) !== termId) errors.push(`codex: Open in Codex for ${termId} landed elsewhere`);
  await onScreen("codex: the landed row", page.locator(".kw-row.landed"));
  await shot("codex-landed");
  await page.getByTestId("codex-tab-fusions").click();
  await page.getByTestId("codex-fusions-found").waitFor();
  // Tabs switch without refetching: /fusions once while the Codex is open.
  {
    let fetches = 0;
    const count = (r) => /\/fusions(\?|$)/.test(new URL(r.url()).pathname + new URL(r.url()).search) && fetches++;
    page.on("request", count);
    await page.getByTestId("codex-tab-units").click();
    await page.getByTestId("codex-units").waitFor();
    await page.getByTestId("codex-tab-fusions").click();
    await page.getByTestId("codex-fusions-found").waitFor();
    page.off("request", count);
    if (fetches) errors.push(`codex: switching tabs fetched /fusions ${fetches} more time(s)`);
  }
  if (!/^[\d,]+ of 6,480 found$/.test(await page.getByTestId("codex-fusions-found").textContent())) errors.push(`codex: "${await page.getByTestId("codex-fusions-found").textContent()}"`);
  await shot("codex-fusions"); await noHScroll("codex-fusions");
  if ((await page.getByTestId("icon-credits").textContent()).indexOf("CC BY 3.0") < 0) errors.push("codex: no icon credits");
  await page.getByTestId("codex-back").click();
  await page.getByTestId("play").waitFor();
  await page.getByTestId("play").click();

  let round = 0;
  /** R3-12: the offer locked before a fight, checked in the next shop. */
  let lockedFromLast = null;
  let lockSurvived = false;
  /** Result screens whose ☰ named a round (R2-17). */
  let menuRounds = 0;
  let whyShot = false;
  for (let guard = 0; guard < 20; guard++) {
    await page.getByTestId("fight").waitFor({ timeout: 10_000 });
    if (round === 4) {
      // The ☰ run menu (R2-10): Title menu leaves the run waiting; Continue
      // on the title menu brings back the same round.
      const at = await page.getByTestId("round").textContent();
      await tap44("☰", page.getByTestId("menu-open")); await tap44("☰", page.getByTestId("menu-open"), "width");
      await page.getByTestId("menu-open").click();
      await page.getByTestId("run-menu").waitFor();
      await shot("run-menu"); await noHScroll("run-menu");
      // The Sound row (round 3, note 16): the toggle turns it off and on, with 44px taps.
      const toggle = page.getByTestId("run-menu").getByTestId("sound-toggle");
      await tap44("sound toggle", toggle);
      await toggle.click();
      if (!(await toggle.textContent()).includes("off")) errors.push("sound: the menu toggle didn't turn sound off");
      await toggle.click();
      if (!(await toggle.textContent()).includes("on")) errors.push("sound: the menu toggle didn't turn sound back on");
      const vbox = await page.getByTestId("run-menu").getByTestId("sound-volume").boundingBox();
      if (!vbox || vbox.width < 100) errors.push(`sound: the volume slider is ${vbox ? Math.round(vbox.width) : 0}px wide`);
      // ☰ Codex opens over the shop; Back returns to the same round.
      await page.getByTestId("menu-codex").click();
      await page.getByTestId("codex-units").waitFor();
      await page.getByTestId("codex-back").click();
      await page.getByTestId("fight").waitFor();
      if ((await page.getByTestId("round").textContent()) !== at) errors.push(`codex from ☰: back in ${await page.getByTestId("round").textContent()}, not ${at}`);
      // Back (R3-26) is Esc: in the shop it opens the run menu, again resumes;
      // over the Codex it returns to the shop.
      await back();
      await page.getByTestId("run-menu").waitFor({ timeout: 3_000 }).catch(() => errors.push("back: Back in the shop didn't open the run menu"));
      await back();
      await page.getByTestId("run-menu").waitFor({ state: "detached", timeout: 3_000 }).catch(() => errors.push("back: Back didn't close the run menu"));
      await page.getByTestId("menu-open").click();
      await page.getByTestId("menu-codex").click();
      await page.getByTestId("codex-units").waitFor();
      await back();
      await page.getByTestId("fight").waitFor({ timeout: 3_000 }).catch(() => errors.push(`back: Back in the Codex didn't return to the shop (${page.url()})`));
      await page.getByTestId("menu-open").click();
      await page.getByTestId("run-menu").waitFor();
      await page.getByTestId("menu-title").click();
      await page.getByTestId("play").waitFor();
      const cont = await page.getByTestId("play").textContent();
      if (!cont.includes(`Continue run · ${at.split("/")[0]}`)) errors.push(`title menu: "${cont}" doesn't continue ${at}`);
      if ((await page.getByTestId("new-run").count()) === 0) errors.push("title menu: no New run while a run waits");
      await shot("home-continue"); await noHScroll("home-continue"); await onScreen("home: Continue", page.getByTestId("play"));
      await page.getByTestId("play").click();
      await page.getByTestId("fight").waitFor();
      if ((await page.getByTestId("round").textContent()) !== at) errors.push(`continue: back in ${await page.getByTestId("round").textContent()}, not ${at}`);
    }
    // Lock (R3-12): in the first shop, lock offer 1 from its sheet and reroll:
    // it moves to the left, still locked; unlock it again. A locked offer from
    // the last shop is still there, locked, at the left.
    const offerName = (slot) => page.getByTestId(`offer-${slot}`).locator(".name").textContent();
    const isLocked = async (slot) => (await page.getByTestId(`offer-${slot}`).getAttribute("class")).includes("locked") && (await page.getByTestId(`offer-${slot}`).locator(".lock-mark").count()) === 1;
    const lockFromSheet = async (slot, want) => {
      await page.getByTestId(`offer-${slot}`).click();
      await page.getByTestId("lock").waitFor();
      if (!(await page.getByTestId("lock").textContent()).includes(want)) errors.push(`lock: offer ${slot}'s sheet says "${await page.getByTestId("lock").textContent()}", not ${want}`);
      if (want === "Lock" && round === 0) { await shot("offer-sheet-lock"); await tap44("Lock", page.getByTestId("lock")); }
      await page.getByTestId("lock").click();
      await page.waitForFunction(([s, on]) => document.querySelector(`[data-testid="offer-${s}"]`)?.classList.contains("locked") === on, [slot, want === "Lock"]);
    };
    if (lockedFromLast !== null) {
      if ((await offerName(0)) !== lockedFromLast || !(await isLocked(0))) errors.push(`lock: "${lockedFromLast}" locked last round isn't locked at offer 0 (${await offerName(0)})`);
      else lockSurvived = true;
      lockedFromLast = null;
    }
    if (round === 0 && (await page.getByTestId("offers").locator(".card").count()) >= 2) {
      const name = await offerName(1);
      await lockFromSheet(1, "Lock");
      if (!(await isLocked(1))) errors.push("lock: offer 1 has no padlock after Lock");
      await shot("shop-locked"); await noHScroll("shop-locked");
      const gold = await page.getByTestId("gold").textContent();
      await page.getByTestId("reroll").click();
      await page.waitForFunction((g) => document.querySelector('[data-testid="gold"]')?.textContent !== g, gold);
      if ((await offerName(0)) !== name || !(await isLocked(0))) errors.push(`lock: after a reroll offer 0 is "${await offerName(0)}", not the locked "${name}"`);
      await lockFromSheet(0, "Unlock");
      if (await isLocked(0)) errors.push("lock: offer 0 still locked after Unlock");
    }
    // Buy while the gold allows and the line has room.
    for (let k = 0; k < 4; k++) {
      if ((await page.getByTestId("gold").count()) === 0) break; // the Crown: no shop, no gold
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
    if (round === 1 && (await page.getByTestId("offers").locator(".card").count()) > 0) {
      lockedFromLast = await offerName(0);
      await lockFromSheet(0, "Lock");
    }
    round++;
    if (round === 1) { await shot("shop"); await noHScroll("shop"); await noRates("shop"); }
    if (round === 1) {
      // The shop's "?" explains a card's numbers, and Rules open during a run (#587).
      await page.getByTestId("legend-open").click();
      await page.getByTestId("legend").waitFor();
      await shot("legend"); await noHScroll("legend"); await noRates("legend");
      await page.getByTestId("legend-rules").click();
      await page.getByTestId("rules").waitFor();
      await page.getByTestId("sheet-close").click();
      await page.getByTestId("rules").waitFor({ state: "detached" });
      await tap44("shop ? and Rules", page.locator('[data-testid="legend-open"], [data-testid="shop-rules"]'));
      await tap44("shop ? and Rules", page.locator('[data-testid="legend-open"], [data-testid="shop-rules"]'), "width");
    }
    if (round === 2 && (await page.getByTestId("line").locator(".card.you").count()) > 1) {
      await page.getByTestId("line-1").click();
      await shot("shop-selected");
      await page.getByTestId("info").click();
      await page.getByTestId("unit-sheet").waitFor();
      await shot("unit-sheet"); await noHScroll("unit-sheet");
      await sheetChecks("unit sheet");
      await page.getByTestId("overlay").click({ position: { x: 180, y: 10 } });
      await page.getByTestId("champion-pin-open").click().catch(() => {});
      if (await page.getByTestId("overlay").isVisible().catch(() => false)) { await shot("champion-pin"); await page.getByTestId("overlay").click({ position: { x: 180, y: 10 } }); }
      await page.getByTestId("move-left").click();
      await page.getByTestId("fight").waitFor();
    }
    await page.getByTestId("fight").click();
    // The battle viewer (slice 9) comes first. In round 1, watch it play, then
    // tap a change and read its chain. End shows the end card, Continue goes
    // to the result, which shows "why I lost" after a loss (shot once).
    await page.getByTestId("battle-end").waitFor({ timeout: 10_000 });
    // R2-14: the speed chosen in round 1 (4×) holds in the next battle.
    if (round === 2 && (await page.getByTestId("battle-speed").textContent()) !== "4×") errors.push(`speed: round 2 plays at ${await page.getByTestId("battle-speed").textContent()}, not the 4× chosen in round 1`);
    if (round === 1) {
      // R2-14: the control bar stays on screen and the battle never scrolls, every frame.
      await page.evaluate(() => {
        const S = { frames: 0, off: 0, scroll: 0 };
        window.__ctl = S;
        const tick = () => {
          if (!document.querySelector(".bv-controls")) return;
          S.frames++;
          const r = document.querySelector(".bv-controls").getBoundingClientRect();
          if (r.bottom > innerHeight + 0.5) S.off++;
          S.scroll = Math.max(S.scroll, document.documentElement.scrollHeight - innerHeight);
          requestAnimationFrame(tick);
        };
        requestAnimationFrame(tick);
      });
      await page.getByTestId("change").first().waitFor({ timeout: 15_000 });
      await shot("battle"); await noHScroll("battle"); await nameInRing("battle"); await noRates("battle");
      // R2-13: every living battle card has an HP bar and a status row of icons
      // (no status words) that fits its two rows, nothing reads "−0" (a blocked
      // hit shows the Shield), and a caption colours each unit by its own line
      // (a mirror match shares names).
      const anatomy = await page.evaluate(() => {
        const cards = [...document.querySelectorAll(".bv-card:not(.dead)")];
        return {
          cards: cards.length,
          noBar: cards.filter((c) => !c.querySelector('[data-testid="hp-bar"]')).length,
          noRow: cards.filter((c) => !c.querySelector('[data-testid="card-statuses"]')).length,
          wordy: [...document.querySelectorAll('[data-testid="card-status"]')].filter((s) => !s.querySelector("svg")).length,
          zero: [...document.querySelectorAll('[data-testid="change"], [data-testid="caption"], .bv-past, .bv-still, .bv-float')].filter((c) => /[−-]0(?!\d)/.test(c.textContent)).length,
          hiddenStatus: [...document.querySelectorAll('[data-testid="card-statuses"]')].flatMap((row) => [...row.children].filter((c) => c.getBoundingClientRect().bottom > row.getBoundingClientRect().bottom + 0.5)).length,
          wrongSide: [...document.querySelectorAll(".bv-cn[data-unit]")].filter((n) => {
            const line = document.querySelector(`.bv-line [data-unit="${CSS.escape(n.dataset.unit)}"]`)?.closest(".bv-line");
            return line && line.classList.contains("mine") !== n.classList.contains("tone-ally");
          }).length,
        };
      });
      if (!anatomy.cards || anatomy.noBar || anatomy.noRow) errors.push(`battle cards: ${JSON.stringify(anatomy)} (each needs an HP bar and a status row)`);
      if (anatomy.wordy) errors.push(`battle cards: ${anatomy.wordy} statuses without an icon`);
      if (anatomy.zero) errors.push(`battle: ${anatomy.zero} chips, captions or floats read "−0"`);
      if (anatomy.hiddenStatus) errors.push(`battle cards: ${anatomy.hiddenStatus} statuses hidden below the status rows`);
      if (anatomy.wrongSide) errors.push(`battle caption: ${anatomy.wrongSide} unit names in the other side's colour`);
      await page.getByTestId("change").first().click();
      await page.getByTestId("trace-text").waitFor();
      const chain = await page.getByTestId("trace-text").textContent();
      if (!/←/.test(chain)) errors.push(`trace: no chain in "${chain}"`);
      await shot("battle-trace"); await noHScroll("battle-trace");
      // R2-15: Why lists the chain from the change back to the turn, each
      // step a 44 px button that moves the playhead to its moment, the
      // control bar still on screen.
      const kinds = await page.getByTestId("why-step").evaluateAll((els) => els.map((e) => e.dataset.kind));
      if (kinds[0] !== "change" || kinds.at(-1) !== "root") errors.push(`why: the chain runs ${kinds.join(" ← ")}, not change … root`);
      await tap44("why step", page.getByTestId("why-step"));
      const turnBefore = await page.locator(".hud span").nth(2).textContent();
      await page.getByTestId("why-step").last().click();
      if (!(await page.getByTestId("why-step").last().evaluate((e) => e.classList.contains("on")))) errors.push("why: the clicked step isn't lit");
      if ((await page.getByTestId("battle-play").textContent()) !== "▶") errors.push("why: a step's click didn't leave the battle paused");
      const turnAfter = await page.locator(".hud span").nth(2).textContent();
      // R2-17: "Turn N" lands on turn N's start, and the HUD says so.
      const root = (await page.getByTestId("why-step").last().textContent()) ?? "";
      const rootTurn = /Turn (\d+)/.exec(root)?.[1];
      if (rootTurn && turnAfter !== `T${rootTurn}`) errors.push(`why: "Turn ${rootTurn}" put the board at ${turnAfter}`);
      // R2-17: a tap on a step's words (a term, the trigger pill) seeks too; it opens no glossary.
      const worded = page.locator('[data-testid="why-step"]:has(.t)');
      if (await worded.count()) {
        await worded.first().locator(".t").first().click();
        if (await page.getByTestId("term-sheet").count().catch(() => 0)) errors.push("why: a tap on a step's word opened the glossary instead of seeking");
        if (!(await worded.first().evaluate((e) => e.classList.contains("on")))) errors.push("why: a tap on a step's word didn't light the step");
      }
      const bar = await page.getByTestId("battle-play").boundingBox();
      if (!bar || bar.y + bar.height > 640) errors.push(`why: the control bar is off screen with Why open (${JSON.stringify(bar)})`);
      await shot("battle-why-step"); await noHScroll("battle-why-step");
      console.log(`why: ${kinds.join(" ← ")}; the root's click moved the board ${turnBefore} → ${turnAfter}`);
      await tap44("change chip", page.getByTestId("change"));
      await tap44("trace close", page.getByTestId("trace-close"));
      await tap44("trace close", page.getByTestId("trace-close"), "width");
      await tap44("past step", page.locator("button.bv-past"));
      await tap44("trigger badge", page.getByTestId("trigger-badge"));
      if ((await page.getByTestId("caption-side").count()) === 0 && /→/.test(await page.getByTestId("caption").textContent())) errors.push("battle caption: no side tag on a unit's act");
      // A battle card (below its chip) opens its Now sheet (R3-18), whose
      // Full card opens the unit's sheet, which closes with Close.
      await page.getByTestId("trace-close").click();
      // (A summon has no full card yet, R3-5: take a unit that entered the battle.)
      const box = await page.getByTestId("battle-you").locator('.bv-card:not([data-unit*="+"])').first().boundingBox();
      await page.mouse.click(box.x + box.width / 2, box.y + box.height - 8);
      await page.getByTestId("now-sheet").waitFor();
      await shot("battle-now-sheet-end"); await noHScroll("battle-now-sheet-end");
      await tap44("now sheet: full card", page.getByTestId("now-full-card"));
      await page.getByTestId("now-full-card").click();
      await page.getByTestId("unit-sheet").waitFor();
      if (await page.getByTestId("now-sheet").count()) errors.push("now sheet: still up under the full card");
      await shot("battle-unit-sheet"); await noHScroll("battle-unit-sheet");
      await sheetChecks("battle unit sheet");
      // The Codex opened over a playing battle pauses it; Back finds it paused where it was.
      await page.evaluate(() => document.querySelector('[data-testid="battle-play"]').click());
      if ((await page.getByTestId("battle-play").textContent()) !== "❚❚") errors.push("battle: Play didn't resume");
      await page.locator('[data-testid="unit-sheet"] [data-testid="term"]').first().click();
      await page.getByTestId("term-codex").click();
      await page.getByTestId("codex-keywords").waitFor();
      await page.waitForTimeout(1500);
      await page.getByTestId("codex-back").click();
      await page.getByTestId("battle-end").waitFor();
      if ((await page.getByTestId("battle-play").textContent()) !== "▶") errors.push("battle: still playing behind the Codex");
      if (await page.getByTestId("battle-done").count()) errors.push("battle: it played out behind the Codex");
      if (await page.getByTestId("unit-sheet").count()) await page.getByTestId("sheet-close").click();
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
        await shot("battle-two-changes"); await noHScroll("battle-two-changes"); await nameInRing("battle-two-changes");
        await multi.click();
        await page.getByTestId("trace-group").waitFor();
        if ((await page.getByTestId("trace-group-change").count()) !== 2) errors.push("two changes: the trace doesn't list both");
        await tap44("trace group change", page.getByTestId("trace-group-change"));
        await shot("battle-two-changes-trace");
        await page.getByTestId("trace-close").click();
      } else console.log("mvp phone: round 1 had no unit with two changes in one step (no shot)");
      await beamChecks(page, errors, { shot, phone: true });
      // The beams again on a taller phone (390×844), where the caption band sits elsewhere.
      await page.setViewportSize({ width: 390, height: 844 });
      await beamChecks(page, errors, { shot: (n) => shot(`${n}-390`), phone: true, label: "phone 390" });
      await page.setViewportSize({ width: 360, height: 640 });
      await nowSheetChecks(page, errors, { shot, phone: true });
      // R2-14: controls, the end card, key moments and Replay.
      await tap44("battle controls", page.locator(".bv-controls button"));
      await tap44("battle controls", page.locator(".bv-controls button"), "width");
      await onScreen("battle controls", page.locator(".bv-controls"));
      for (let k = 0; k < 3 && (await page.getByTestId("battle-speed").textContent()) !== "4×"; k++) await page.getByTestId("battle-speed").click();
      if ((await page.getByTestId("battle-speed").textContent()) !== "4×") errors.push("speed: no 4×");
      // ▶ on the last beat shows the end card instead of starting over.
      if (await page.getByTestId("battle-play").textContent() === "❚❚") await page.getByTestId("battle-play").click();
      for (let k = 0; k < 400 && !(await page.getByTestId("end-card").isVisible()); k++) await page.getByTestId("battle-step").click();
      if (!(await page.getByTestId("end-card").isVisible())) errors.push("end card: › to the last beat never showed it");
      await page.getByTestId("end-close").click();
      await page.getByTestId("battle-play").click();
      if (!(await page.getByTestId("end-card").isVisible())) errors.push("end card: ▶ on the last beat didn't show it");
      const word = await page.getByTestId("battle-word").textContent();
      if (!/^(VICTORY|DEFEAT|DRAW)$/.test(word)) errors.push(`end card: word "${word}"`);
      if ((await page.getByTestId("damage-row").count()) < 2) errors.push("end card: no damage by unit");
      const moments = await page.getByTestId("key-moment").count();
      if (moments < 1 || moments > 3) errors.push(`end card: ${moments} key moments`);
      await onScreen("end card: Continue", page.getByTestId("battle-done"));
      // R2-17: the end card never hides the caption's result line ("They win").
      const [endBox, capBox] = [await page.getByTestId("end-card").boundingBox(), await page.locator(".bv-cap").boundingBox()];
      if (endBox && capBox && endBox.y < capBox.y + capBox.height - 0.5) errors.push(`end card: it covers the caption's result line (${Math.round(endBox.y)} < ${Math.round(capBox.y + capBox.height)})`);
      await tap44("end card buttons", page.locator('[data-testid="end-card"] button'));
      await shot("battle-end-card"); await noHScroll("battle-end-card");
      if (await page.getByTestId("end-why").count()) {
        // Why I lost, or Why I won (R2-17).
        const label = await page.getByTestId("end-why").textContent();
        await page.getByTestId("end-why").click();
        const panel = page.getByTestId(label === "Why I won" ? "why-won" : "why-lost");
        await panel.waitFor();
        await shot("battle-end-why");
        if (await panel.locator("button.bv-why").count()) {
          await panel.locator("button.bv-why").first().click();
          await page.getByTestId("trace-text").waitFor();
          // R2-17 batch E: over the finished battle, the way on stays in sight.
          await onScreen(`${label}'s trace: the way on`, page.getByTestId("trace-done"));
          await page.getByTestId("trace-close").click();
          if (!(await page.getByTestId("end-card").isVisible())) errors.push(`end card: not back after ${label}'s trace`);
        } else await page.getByTestId("sheet-close").click();
      }
      // A key moment replays from its beat.
      if (moments) {
        const beat = Number(await page.getByTestId("key-moment").first().getAttribute("data-beat"));
        await page.getByTestId("key-moment").first().click();
        if (await page.getByTestId("end-card").isVisible()) errors.push("key moment: the end card stayed up");
        if ((await page.getByTestId("battle-play").textContent()) !== "❚❚") errors.push("key moment: not playing");
        await page.getByTestId("battle-play").click();
        const turn = await page.locator(".hud span").nth(2).textContent();
        console.log(`mvp phone: key moment at beat ${beat} plays from ${turn}`);
        await page.getByTestId("battle-end").click();
      }
      // Replay plays from the start: the line-up first.
      await page.getByTestId("end-replay").click();
      if (await page.getByTestId("end-card").isVisible()) errors.push("replay: the end card stayed up");
      if (!/The lines face off/.test(await page.getByTestId("caption").textContent())) errors.push(`replay: didn't start from the line-up ("${await page.getByTestId("caption").textContent()}")`);
      const ctl = await page.evaluate(() => window.__ctl);
      if (ctl.off || ctl.scroll > 0) errors.push(`battle: controls off screen in ${ctl.off}/${ctl.frames} frames, page scrolls ${ctl.scroll}px`);
    }
    await page.getByTestId("battle-end").click();
    // R2-17 batch E: the end card is the fight's one result. No result
    // screen follows; the card carries its round, hearts, record and cost.
    await page.getByTestId("end-card").waitFor({ timeout: 10_000 });
    if (await page.getByTestId("outcome").count()) errors.push("a result screen still follows the end card");
    await onScreen("end card: the way on", page.getByTestId("battle-done"));
    if (!whyShot && (await page.getByTestId("end-why").textContent().catch(() => "")) === "Why I lost") {
      whyShot = true;
      await shot("result-after-loss"); await noHScroll("result-after-loss");
      await page.getByTestId("end-why").click();
      await page.getByTestId("why-lost").waitFor();
      const why = page.getByTestId("why-lost").locator("button.bv-why").first();
      if (await why.count()) {
        await why.click();
        await page.getByTestId("trace-text").waitFor();
        await shot("why-trace"); await noHScroll("why-trace");
        // The trace sits where the end card was; Result brings the card back.
        await onScreen("why I lost's trace: the way on", page.getByTestId("trace-done"));
        await page.getByTestId("trace-result").click();
        if (!(await page.getByTestId("end-card").isVisible())) errors.push("why I lost: Result didn't bring the end card back");
      } else await page.getByTestId("sheet-close").click();
    }
    if (round === 1) { await shot("result"); await noHScroll("result"); await noRates("result"); }
    // The end card's run line and the battle's ☰ name the round just fought,
    // not the next one the run has moved on to (or the Crown, or a run that is over).
    {
      const shown = (await page.getByTestId("result-round").textContent()) ?? "";
      if (!/[♥♡]/.test((await page.getByTestId("end-run").textContent()) ?? "")) errors.push("end card: no hearts");
      await page.getByTestId("menu-open").click();
      await page.getByTestId("run-menu").waitFor();
      const menu = (await page.getByTestId("run-menu").locator(".label").first().textContent()) ?? "";
      await page.getByTestId("menu-resume").click();
      await page.getByTestId("run-menu").waitFor({ state: "detached" });
      const n = /round (\d+) of/.exec(menu)?.[1];
      if (n !== undefined) {
        menuRounds++;
        if (Number(n) !== round || /^R(\d+)\//.exec(shown)?.[1] !== n) errors.push(`end card ☰: "${menu}" after round ${round} ("${shown}")`);
      } else if (!/^Run · (over|the Crown)$/.test(menu)) errors.push(`end card ☰: "${menu}" after round ${round}`);
    }
    const done = ((await page.getByTestId("battle-done").textContent()) ?? "").trim();
    if (!/^(Next round|To the Crown|See the run)$/i.test(done)) errors.push(`end card: last button "${done}"`);
    await page.getByTestId("battle-done").click();
    await page.waitForFunction(() => document.querySelector('[data-testid="fight"]') || document.querySelector('[data-testid="run-over"]'));
    if (await page.getByTestId("run-over").isVisible().catch(() => false)) break;
  }
  if (!menuRounds) errors.push("end card ☰: no fight's menu named a round");
  if (!lockSurvived) errors.push("lock: no locked offer was seen to survive a fight");
  await page.getByTestId("run-over").waitFor({ timeout: 10_000 });
  await shot("run-over"); await noHScroll("run-over"); await noRates("run-over");
  const over = await page.getByTestId("run-over").textContent();
  if (/No champion/.test(over) && /Reached the Crown/.test(over)) errors.push(`run over: "${over}" contradicts itself`);
  // A standalone count only: the rating line's "expected 3.1 wins" is no "1 wins".
  if (/(?<![\w.])1 (wins|draws|losses)\b|(?<![\w.])([02-9]|\d\d+) (win|draw|loss)\b/.test(over)) errors.push(`run over: plural wrong in "${over}"`);
  await page.getByTestId("home").click();
  await page.getByTestId("play").waitFor();
  await shot("home-after");

  // Stats (slice 11, R2-11): records and the champion history; units and
  // fusions moved to the Codex.
  await page.getByTestId("stats").click();
  await page.getByTestId("stats-champions").waitFor();
  if ((await page.getByTestId("stats-records").count()) === 0) errors.push("stats: no records");
  if ((await page.getByTestId("stats-tab-units").count()) > 0) errors.push("stats: still has a Units tab (the Codex has it)");
  await shot("stats"); await noHScroll("stats");
  {
    // Back stays on the first screen however long the lists grow.
    const box = await page.getByTestId("stats-back").boundingBox();
    if (!box || box.y + box.height > 640) throw new Error(`stats: Back is off screen (${box && Math.round(box.y)})`);
  }
  await page.getByTestId("stats-back").click();
  await page.getByTestId("play").waitFor();

  // Giving up (R2-10 on R2-2's abandon): New run on the title menu asks to
  // abandon the waiting run first; ☰ Abandon run confirms once and ends on
  // the run-over screen with the rating change.
  await page.getByTestId("play").click();
  await page.getByTestId("fight").waitFor();
  await page.getByTestId("menu-open").click();
  await page.getByTestId("menu-title").click();
  await page.getByTestId("new-run").click();
  await page.getByTestId("abandon-text").waitFor();
  await shot("new-run-confirm"); await noHScroll("new-run-confirm");
  if (!/Every heart left counts as a lost fight/.test(await page.getByTestId("abandon-text").textContent())) errors.push("new run: the confirm doesn't say what abandoning costs");
  await page.getByTestId("abandon-cancel").click();
  await page.getByTestId("play").click();
  await page.getByTestId("fight").waitFor();
  await page.getByTestId("menu-open").click();
  await page.getByTestId("menu-abandon").click();
  await page.getByTestId("abandon-confirm").waitFor();
  await shot("abandon-confirm"); await noHScroll("abandon-confirm");
  await page.getByTestId("abandon-confirm").click();
  await page.getByTestId("run-over").waitFor({ timeout: 10_000 });
  await shot("run-over-abandoned"); await noHScroll("run-over-abandoned");
  if (!/You gave up/.test(await page.getByTestId("run-why").textContent())) errors.push("abandon: run-over doesn't say the run was given up");
  if ((await page.getByTestId("rating-change").count()) === 0) errors.push("abandon: no rating change on the run-over screen");
  await page.getByTestId("home").click();
  await page.getByTestId("play").waitFor();
  if ((await page.getByTestId("play").textContent()) !== "Play") errors.push("abandon: the title menu still offers Continue");
  // New run on the title menu gives the waiting run up too, and shows its
  // end and rating change first; its New run starts the next run (R2-17).
  await page.getByTestId("play").click();
  await page.getByTestId("fight").waitFor();
  await page.getByTestId("menu-open").click();
  await page.getByTestId("menu-title").click();
  await page.getByTestId("new-run").click();
  await page.getByTestId("abandon-confirm").click();
  await page.getByTestId("run-over").waitFor({ timeout: 10_000 });
  if (!/You gave up/.test(await page.getByTestId("run-why").textContent())) errors.push("new run: run-over doesn't say the run was given up");
  if ((await page.getByTestId("rating-change").count()) === 0) errors.push("new run: no rating change for the given-up run");
  await shot("run-over-new-run"); await noHScroll("run-over-new-run");
  await page.getByTestId("new-run-start").click();
  await page.getByTestId("fight").waitFor();
  if ((await page.getByTestId("round").textContent()) !== "R1/12") errors.push(`new run: starts at ${await page.getByTestId("round").textContent()}`);

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
  const fuser = await call("POST", "/players", { name: `Fuser${TAG}` });
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
    // An awakening's gift is skipped here (R3-15); the phone's chooser is R3-16's.
    const d = run.gift ? { kind: "gift", pick: null } : run.phase === "crown" ? { kind: "fight" } : want && run.gold >= want.cost ? { kind: "buy", slot: want.slot } : run.gold >= 1 ? { kind: "reroll" } : { kind: "fight" };
    run = (await call("POST", `/runs/${run.runId}/decisions`, d, fuser.id)).run;
  }
  // The awakening copy the page buys brings a gift (R3-15): the chooser
  // (R3-16) shows 3 cards; a card opens its sheet; Esc sets the gift aside
  // behind a banner (nothing but sell and reorder goes meanwhile); Open gift
  // brings it back, and a pick joins the board (or merges in).
  const chooseGift = async () => {
    const before = await call("GET", `/runs/${run.runId}`, undefined, fuser.id);
    if (!before.gift) return void errors.push("awakening: no gift offered");
    const shown = await page.getByTestId("gift-title").waitFor({ timeout: 5_000 }).then(() => true, () => false);
    if (!shown) return void errors.push("gift: the chooser doesn't open after the awakening copy");
    if (!/Awakened! Pick a gift/.test(await page.getByTestId("gift-title").textContent())) errors.push("gift: no 'Awakened! Pick a gift'");
    const cards = await page.getByTestId("gift-choices").locator(".card").count();
    if (cards !== 3) errors.push(`gift: ${cards} cards, want 3`);
    await shot("gift-chooser"); await noHScroll("gift-chooser"); await noVScroll("gift-chooser");
    await tap44("gift Pick and Skip", page.locator('[data-testid^="gift-pick-"], [data-testid="gift-skip"]'));
    await onScreen("gift: Skip", page.getByTestId("gift-skip"));
    // A card's sheet is one tap away, and closes back to the chooser.
    await page.getByTestId("gift-card-0").click();
    await page.locator('.overlay [data-testid="unit-sheet"]').waitFor();
    await shot("gift-card-sheet");
    await page.getByTestId("sheet-close").click();
    if (!(await page.getByTestId("gift-title").isVisible())) errors.push("gift: closing a card's sheet closed the chooser too");
    // Esc sets it aside: a banner, the shop's Fight and Reroll off until it's picked or skipped.
    await page.keyboard.press("Escape");
    await page.getByTestId("gift-banner").waitFor({ timeout: 3_000 }).catch(() => errors.push("gift: Esc leaves no banner"));
    if (await page.getByTestId("gift-title").isVisible()) errors.push("gift: Esc doesn't set the chooser aside");
    if (!(await page.getByTestId("fight").isDisabled())) errors.push("gift: Fight is on while the gift waits");
    if (!(await page.getByTestId("reroll").isDisabled())) errors.push("gift: Reroll is on while the gift waits");
    await shot("gift-aside"); await noHScroll("gift-aside");
    await tap44("gift banner: Open gift", page.getByTestId("gift-open"));
    await page.getByTestId("gift-open").click();
    await page.getByTestId("gift-title").waitFor();
    // A tap outside the sheet sets it aside too, and Open gift brings it back.
    await page.mouse.click(180, 20);
    await page.getByTestId("gift-banner").waitFor();
    if (await page.getByTestId("gift-title").isVisible()) errors.push("gift: a tap outside doesn't set the chooser aside");
    await page.getByTestId("gift-open").click();
    await page.getByTestId("gift-title").waitFor();
    // Pick a unit the line doesn't hold (it goes to the line's end, else the
    // bench), so the line's indices stay as they were; a copy would do too.
    const owned = (id) => [...before.line, ...(before.bench ?? [])].some((u) => u.unitId === id || u.fusion?.first === id || u.fusion?.second === id);
    const i = Math.max(0, before.gift.findIndex((id) => !owned(id)));
    await page.getByTestId(`gift-pick-${i}`).click();
    await page.getByTestId("gift-title").waitFor({ state: "hidden" }).catch(() => {});
    await page.getByTestId("gift-banner").waitFor({ state: "hidden", timeout: 5_000 }).catch(() => {});
    let after = await call("GET", `/runs/${run.runId}`, undefined, fuser.id);
    // A pick that awakens a unit brings another gift: skip that one.
    if (after.gift) {
      await page.getByTestId("gift-skip").click();
      await page.getByTestId("gift-banner").waitFor({ state: "hidden", timeout: 5_000 }).catch(() => {});
      after = await call("GET", `/runs/${run.runId}`, undefined, fuser.id);
    }
    if (after.gift) errors.push("gift: still waiting after the pick");
    const has = [...after.line, ...after.bench].some((u) => u.unitId === before.gift[i] || u.fusion?.first === before.gift[i] || u.fusion?.second === before.gift[i]);
    if (!has) errors.push(`gift: the picked ${before.gift[i]} isn't in the line or on the bench`);
    await shot("gift-picked"); await noHScroll("gift-picked");
  };
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
    await chooseGift();
    await page.getByTestId("hint").filter({ hasText: "fuse" }).waitFor();
    const first = run.line.findIndex((u) => u.form === "awoken");
    const second = run.line.findIndex((u) => u.uid === almost.uid);
    await page.getByTestId(`line-${first}`).click();
    await shot("shop-fuse-actions"); await noHScroll("shop-fuse-actions");
    await tap44("move buttons beside Fuse", page.locator('[data-testid="move-left"], [data-testid="move-right"]'), "width");
    await page.getByTestId("fuse").click();
    await shot("fuse-pick"); await noHScroll("fuse-pick");
    // R2-5: both orders are previewed up front; a pair nobody has fused comes
    // with no name in the response and shows "??? New fusion".
    const previews = [];
    const onPreview = async (res) => { if (/\/preview$/.test(res.url())) previews.push({ req: res.request().postDataJSON(), body: await res.text() }); };
    page.on("response", onPreview);
    await page.getByTestId(`line-${second}`).click();
    await page.getByTestId("preview-confirm").waitFor();
    page.off("response", onPreview);
    const tapped = previews.find((p) => p.req.first === first && p.req.second === second);
    const fusedIn = (p) => JSON.parse(p.body).run.line.find((u) => u.kind === "fused");
    const isNew = tapped && fusedIn(tapped).name === "";
    if (previews.length !== 2 || !tapped) errors.push(`fusion preview: expected both orders previewed, got ${previews.length}`);
    const sheetText = await page.getByTestId("overlay").textContent();
    if (isNew && !/\?\?\? New fusion/.test(sheetText)) errors.push("fusion preview: a new pair doesn't say '??? New fusion'");
    if (tapped && !isNew && !sheetText.includes(fusedIn(tapped).name)) errors.push("fusion preview: a known pair doesn't show its name");
    await shot("fusion-preview"); await noHScroll("fusion-preview");
    const recipe = await page.getByTestId("fusion-recipe").textContent();
    await page.getByTestId("preview-swap").click();
    const swapped = await page.getByTestId("fusion-recipe").textContent();
    const names = [run.line[first].name, run.line[second].name];
    if (!(recipe.indexOf(names[0]) < recipe.indexOf(names[1]) && swapped.indexOf(names[1]) < swapped.indexOf(names[0]))) errors.push(`fusion preview: Swap doesn't flip the recipe (${recipe} / ${swapped})`);
    await shot("fusion-preview-swapped"); await noHScroll("fusion-preview-swapped");
    await page.getByTestId("preview-swap").click();
    await page.getByTestId("preview-confirm").click();
    // The reveal comes with the fused line (main.ts fuse): for a new pair, and
    // also for one only bots had made (its name was shown, the credit is now
    // yours). Either way it is closed before the line is used (R2-17).
    await page.getByTestId("line").locator(".card.fused").waitFor();
    const revealed = (await page.getByTestId("fusion-reveal").count()) > 0;
    if (isNew && !revealed) errors.push("fusion reveal: a new pair shows no reveal");
    if (revealed) {
      const reveal = await page.getByTestId("fusion-reveal").textContent();
      const named = reveal.replace(/^.*You discovered /, "");
      if (!/You discovered \S/.test(reveal)) errors.push(`fusion reveal: '${reveal}'`);
      if (!/discovered by you/.test(await page.getByTestId("overlay").textContent())) errors.push("fusion reveal: no 'discovered by you'");
      if (isNew && previews.some((p) => p.body.includes(named))) errors.push(`fusion preview: the response carried the name '${named}' before the fuse`);
      await shot("fusion-reveal"); await noHScroll("fusion-reveal");
      await page.getByTestId("sheet-close").click();
    }
    console.log(`mvp phone: the fusion was ${isNew ? "new" : "known"}${revealed ? ", revealed" : ""}`);
    await shot("fused"); await noHScroll("fused");
    await page.locator(".card.fused").screenshot({ path: `${out}/${String(++shots).padStart(2, "0")}-fused-card.png` });
    // The fused card's row lines up: every card in it is one height.
    const heights = await page.getByTestId("line").locator(".card.you").evaluateAll((els) => els.map((e) => Math.round(e.getBoundingClientRect().height)));
    if (new Set(heights).size > 1) errors.push(`fused row: uneven card heights ${heights.join(",")}`);
    await page.locator(".card.fused").click();
    await page.getByTestId("info").click();
    await page.getByTestId("unit-sheet").waitFor();
    // The pair may be someone else's discovery already (live, or a reused DB).
    if (!/discovered by (you|@\S+)/.test(await page.getByTestId("unit-sheet").textContent())) errors.push("fused sheet: no discovery credit");
    if (!(await page.getByTestId("sheet-close").isVisible())) errors.push("unit sheet from Info: no Close button");
    await shot("fused-sheet"); await noHScroll("fused-sheet");
    await sheetChecks("fused sheet");
    if (await page.getByTestId("sheet-parts").isVisible()) errors.push("fused sheet: the parts show before the tap");
    await page.getByTestId("sheet-parts-open").click();
    if ((await page.getByTestId("sheet-parts").locator(".sheet-form").count()) !== 2) errors.push("fused sheet: the parts don't open");
    await shot("fused-sheet-parts");
  }
  // The bench (R3-14): with the line full, a new unit goes to the bench (the
  // buy preview says so); a bench unit selected and a tap on a line unit swap
  // the two; To bench and To line move a unit across.
  {
    const bencher = await call("POST", "/players", { name: `Bench${TAG}` });
    let r = await call("POST", "/runs", undefined, bencher.id);
    const owned = (r, id) => [...r.line, ...r.bench].some((u) => u.unitId === id || u.fusion?.first === id || u.fusion?.second === id);
    const fresh = (r) => r.offers.find((o) => !owned(r, o.unitId) && o.cost <= r.gold);
    const benchReady = (r) => r.phase === "shop" && !r.gift && r.line.length === 5 && r.bench.length === 0 && fresh(r);
    for (let steps = 0; steps < 2000 && !benchReady(r); steps++) {
      if (r.phase === "over") { r = await call("POST", "/runs", undefined, bencher.id); continue; }
      const o = r.offers[0];
      const d = r.gift ? { kind: "gift", pick: null } : r.phase === "shop" && r.line.length < 5 && o && r.gold >= o.cost ? { kind: "buy", slot: o.slot } : { kind: "fight" };
      r = (await call("POST", `/runs/${r.runId}/decisions`, d, bencher.id)).run;
    }
    if (!benchReady(r)) errors.push(`bench: never reached a full line with a new unit on offer (phase ${r.phase}, round ${r.round})`);
    else {
      await page.evaluate((p) => localStorage.setItem("arena.player", JSON.stringify(p)), bencher);
      await page.reload();
      await page.getByTestId("play").click();
      await page.getByTestId("fight").waitFor();
      if ((await page.getByTestId("bench").locator(".card.empty").count()) !== 3) errors.push("bench: the empty bench doesn't show 3 dashed slots");
      const o = fresh(r);
      await page.getByTestId(`offer-${o.slot}`).click();
      await page.getByTestId("buy-preview").waitFor();
      if (!/Goes to your bench/.test(await page.getByTestId("buy-preview").textContent())) errors.push(`bench: the buy preview says "${await page.getByTestId("buy-preview").locator(".label").textContent()}", not "Goes to your bench"`);
      await shot("bench-buy-preview"); await noHScroll("bench-buy-preview");
      await page.getByTestId("buy").click();
      await page.getByTestId("bench-0").waitFor();
      const name = (id) => page.getByTestId(id).locator(".name").textContent();
      const benched = await name("bench-0");
      await shot("shop-bench"); await noHScroll("shop-bench"); await noVScroll("shop-bench");
      // A bench unit selected, the line full: a tap on a line unit swaps the two.
      const last = await name("line-4");
      await page.getByTestId("bench-0").click();
      await page.getByTestId("to-line").waitFor();
      if ((await page.getByTestId("hint").filter({ hasText: "swap" }).count()) === 0) errors.push("bench: no swap hint with a bench unit selected and the line full");
      await shot("bench-selected"); await noHScroll("bench-selected"); await noVScroll("bench-selected");
      await tap44("bench actions", page.getByTestId("actions").locator("button"));
      await page.getByTestId("line-4").click();
      await page.waitForFunction((n) => document.querySelector('[data-testid="line-4"] .name')?.textContent === n, benched, { timeout: 5_000 }).catch(() => {});
      if ((await name("line-4")) !== benched || (await name("bench-0")) !== last) errors.push(`bench: the tap-to-swap left line-4 "${await name("line-4")}" and bench-0 "${await name("bench-0")}"`);
      // To bench: line-0 goes to the bench's next slot; To line brings it back to the line's end.
      const front = await name("line-0");
      await page.getByTestId("line-0").click();
      await page.getByTestId("to-bench").click();
      await page.getByTestId("bench-1").waitFor();
      if ((await name("bench-1")) !== front) errors.push(`bench: To bench put "${await name("bench-1")}" on bench-1, not "${front}"`);
      if ((await page.getByTestId("line-4-empty").count()) !== 1) errors.push("bench: the line has no empty slot after To bench");
      await page.getByTestId("bench-1").click();
      await page.getByTestId("to-line").click();
      await page.getByTestId("line-4").waitFor();
      if ((await name("line-4")) !== front) errors.push(`bench: To line put "${await name("line-4")}" in line-4, not "${front}"`);
      await shot("bench-moved"); await noHScroll("bench-moved");
    }
  }
  // The gift's "make room" (R3-16): a gift none of whose units can merge in,
  // with line and bench full. Its picks are off, Make room sets it aside, a
  // sale reopens it, and the pick joins the line. A player gets there through the API.
  {
    const roomer = await call("POST", "/players", { name: `Room${TAG}` });
    let r = await call("POST", "/runs", undefined, roomer.id);
    const board = (r) => [...r.line, ...r.bench];
    const owned = (r, id) => board(r).some((u) => u.unitId === id || u.fusion?.first === id || u.fusion?.second === id);
    const isFull = (r) => r.line.length === 5 && r.bench.length === 3;
    const stuck = (r) => r.phase === "shop" && r.gift && isFull(r) && r.gift.every((id) => !owned(r, id));
    for (let steps = 0; steps < 6000 && !stuck(r); steps++) {
      if (r.phase === "over") { r = await call("POST", "/runs", undefined, roomer.id); continue; }
      let d;
      if (r.gift) {
        // Fill the board with gifts; a full board takes a copy, else skips.
        const fresh = r.gift.findIndex((id) => !owned(r, id));
        const copy = r.gift.findIndex((id) => owned(r, id));
        d = { kind: "gift", pick: !isFull(r) && fresh >= 0 ? fresh : copy >= 0 ? copy : null };
      } else if (r.phase === "crown") d = { kind: "fight" };
      else {
        // Copies of what the board holds first (they awaken, which brings gifts), then new units while there is room.
        const dupe = r.offers.find((o) => o.cost <= r.gold && board(r).some((u) => u.kind === "unit" && u.unitId === o.unitId));
        const fresh = !isFull(r) ? r.offers.find((o) => o.cost <= r.gold) : undefined;
        const o = dupe ?? fresh;
        d = o ? { kind: "buy", slot: o.slot } : r.gold >= 1 && isFull(r) ? { kind: "reroll" } : { kind: "fight" };
      }
      r = (await call("POST", `/runs/${r.runId}/decisions`, d, roomer.id)).run;
    }
    if (!stuck(r)) errors.push(`make room: never reached a gift with line and bench full (phase ${r.phase}, round ${r.round})`);
    else {
      await page.evaluate((p) => localStorage.setItem("arena.player", JSON.stringify(p)), roomer);
      await page.reload();
      await page.getByTestId("play").click();
      await page.getByTestId("gift-title").waitFor();
      if (!(await page.getByTestId("gift-full").isVisible())) errors.push("make room: no 'make room' line with line and bench full");
      for (let i = 0; i < 3; i++) if (!(await page.getByTestId(`gift-pick-${i}`).isDisabled())) errors.push(`make room: Pick ${i} is on with no room`);
      await shot("gift-make-room"); await noHScroll("gift-make-room"); await noVScroll("gift-make-room");
      await tap44("make room buttons", page.locator('[data-testid="gift-make-room"], [data-testid="gift-skip"]'));
      await page.getByTestId("gift-make-room").click();
      await page.getByTestId("gift-banner").waitFor();
      if (!/make room/.test(await page.getByTestId("gift-banner").textContent())) errors.push(`make room: the banner says "${await page.getByTestId("gift-banner").textContent()}"`);
      await shot("gift-make-room-aside"); await noHScroll("gift-make-room-aside"); await noVScroll("gift-make-room-aside");
      // Selling stays possible behind it: sell the front unit, and the chooser comes back.
      await page.getByTestId("line-0").click();
      await page.getByTestId("sell").click();
      const back = await page.getByTestId("gift-title").waitFor({ timeout: 5_000 }).then(() => true, () => false);
      if (!back) errors.push("make room: the chooser doesn't come back after a sale makes room");
      else {
        if (await page.getByTestId("gift-full").count()) errors.push("make room: still says 'make room' after the sale");
        await shot("gift-room-made");
        await page.getByTestId("gift-pick-1").click();
        await page.getByTestId("gift-banner").waitFor({ state: "hidden", timeout: 5_000 }).catch(() => {});
        const after = await call("GET", `/runs/${r.runId}`, undefined, roomer.id);
        if (after.gift) errors.push("make room: the gift still waits after the pick");
        if (after.line.length !== 5 || after.line.at(-1).unitId !== r.gift[1]) errors.push(`make room: the pick didn't join the line's end (${after.line.map((u) => u.unitId).join(",")})`);
        await shot("gift-room-picked"); await noHScroll("gift-room-picked");
      }
    }
  }
  // Compact cards (R2-7): at 360×640 in round 8, with 5 units in the line, a
  // full bench (R3-14) and the grown shop, nothing scrolls. A third player
  // gets there through the API.
  {
    const compact = await call("POST", "/players", { name: `Compact${TAG}` });
    let r = await call("POST", "/runs", undefined, compact.id);
    const full = (r) => r.phase === "shop" && r.round >= 8 && r.line.length === 5 && r.bench.length === 3;
    for (let steps = 0; steps < 4000 && !full(r); steps++) {
      if (r.phase === "over") { r = await call("POST", "/runs", undefined, compact.id); continue; }
      const o = r.offers[0];
      const room = r.line.length < 5 || r.bench.length < 3;
      const d = r.gift ? { kind: "gift", pick: null } : r.phase === "shop" && room && o && r.gold >= o.cost ? { kind: "buy", slot: o.slot } : { kind: "fight" };
      r = (await call("POST", `/runs/${r.runId}/decisions`, d, compact.id)).run;
    }
    if (!full(r)) errors.push(`compact: never reached round 8 with a full line and bench (phase ${r.phase}, round ${r.round})`);
    else {
      await page.evaluate((p) => localStorage.setItem("arena.player", JSON.stringify(p)), compact);
      await page.reload();
      await page.getByTestId("play").click();
      await page.getByTestId("fight").waitFor();
      const offers = await page.getByTestId("offers").locator(".card").count();
      if (offers < 6) errors.push(`compact: round ${r.round} shows ${offers} offers, not 6`);
      await shot("shop-round-8"); await noHScroll("shop-round-8"); await noVScroll("shop-round-8"); await noRates("shop-round-8");
      await onScreen("round 8: Fight", page.getByTestId("fight"));
      const sizes = await page.locator('[data-testid="line"] .card, [data-testid="offers"] .card').evaluateAll((els) => els.map((e) => `${Math.round(e.getBoundingClientRect().width)}x${Math.round(e.getBoundingClientRect().height)}`));
      if (sizes.some((x) => x !== "64x84")) errors.push(`compact: card sizes ${[...new Set(sizes)].join(", ")}, not 64x84`);
      await page.getByTestId("line-0").click();
      await page.getByTestId("info").click();
      await page.getByTestId("unit-sheet").waitFor();
      await shot("unit-sheet-round-8");
      await sheetChecks("round 8 sheet");
      await page.getByTestId("sheet-close").click();
    }
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
  if (!/^Day \d+ ended/.test((await page.getByTestId("day-ended").textContent().catch(() => "")) ?? "")) errors.push("day end: no 'Day N ended' notice");
  await shot("home-day-ended"); await noHScroll("home-day-ended");
  await onScreen("home after day end: Play", page.getByTestId("play"));

  // The reigning champion's own Crown (#587): "Reigning" slays and is crowned
  // through the API (e2e/mvp-own-crown.ts, local server only), then the phone
  // plays their run at the Crown against their own team: a win is a slay (#591).
  if (child) {
    const setup = execFileSync("node", ["--import", "tsx/esm", "e2e/mvp-own-crown.ts", "--url", url], { encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] });
    const { player: champ } = JSON.parse(setup.trim().split("\n").at(-1));
    await page.evaluate((p) => localStorage.setItem("arena.player", JSON.stringify(p)), champ);
    await page.reload();
    await page.getByTestId("play").waitFor();
    const homeText = await page.locator("#app").textContent();
    if (!/This is your team\. Others try to beat it today, and so can you/.test(homeText) || /Beat this team in the Crown/.test(homeText)) errors.push("champion's home: no 'your team' hint");
    await shot("home-champion"); await noHScroll("home-champion");
    await page.getByTestId("play").click();
    await page.getByTestId("fight").waitFor();
    const opp = await page.getByTestId("next-opponent").textContent();
    if (!/\(your champion team\)/.test(opp)) errors.push(`own crown: next opponent "${opp}"`);
    if (!/Beat it to be a slayer again/.test(await page.getByTestId("hint").textContent())) errors.push("own crown: the hint promises no slay");
    await shot("crown-own"); await noHScroll("crown-own");
    await page.getByTestId("fight").click();
    await page.getByTestId("battle-end").click();
    await page.getByTestId("end-card").waitFor({ timeout: 10_000 });
    const result = await page.getByTestId("end-card").textContent();
    const ownWon = (await page.getByTestId("battle-word").textContent()).includes("VICTORY");
    if (ownWon ? !/Slayer today/.test(result) : !/Your team holds/.test(result)) errors.push(`own crown result: "${result.slice(0, 200)}"`);
    await shot("result-own-crown"); await noHScroll("result-own-crown");
    console.log(`mvp phone: the champion's own Crown: ${await page.getByTestId("battle-word").textContent()}`);
    await page.getByTestId("battle-done").click();
    await page.getByTestId("run-over").waitFor({ timeout: 10_000 });
    const ownOver = await page.getByTestId("run-over").textContent();
    if (ownWon ? !/You beat your own champion team: you are a slayer today/.test(ownOver) : !/Your champion team held the Crown/.test(ownOver)) errors.push(`own crown run over: "${ownOver}"`);
    await shot("run-over-own-crown"); await noHScroll("run-over-own-crown");
  }
  console.log(`mvp phone: ${round} fights, ${shots} screenshots in ${out}, ${iconCards} card icon lines fit`);
  // Esc everywhere (R3-6): a fresh player, keys only.
  await escPass(browser, url, { label: "phone-390", viewport: { width: 390, height: 844 }, touch: true, errors });
} finally {
  await browser.close();
  child?.kill();
}
if (errors.length) { console.error(errors.join("\n")); process.exit(1); }
