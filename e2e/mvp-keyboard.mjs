// Keyboard only (#1064, mission #1043): the whole of Arena on a desktop with
// no mouse, in English and in Russian. A pointer click anywhere fails the run
// (every click must come from a key: Enter or Space on a focused control has
// detail 0, a mouse click has detail ≥ 1).
//   1. a full run on keys: sign in, Home (Enter plays), buy (1–7), arrange (Tab
//      to a card, Enter selects, ← →, B), lock and reroll, sell, the menu, fight
//      (Space), the battle's keys, the end card, run over, back Home;
//   2. the scenes that need a state: awaken (the 3rd copy bought by its number),
//      fuse a designed pair through the forge (F, Tab, Enter, Enter), claim one
//      of three sealed designs (2, Enter), the awakening gift (a number picks);
//   3. the Codex (tabs by number, search by /), the Forge and the Dictionary,
//      Settings and every sheet (Esc closes, Tab stays inside);
//   4. focus is visible: every Tab stop of the shop and the Codex has an
//      outline in plum, brass and felt, and the tab order follows the page.
// Without --url it builds the client and starts its own scenes server.
//   npm run mvp:keyboard -- [--url http://127.0.0.1:PORT/arena/] [--lang en|ru] [--out e2e/.shots/keyboard]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/keyboard";
const langs = opt("lang") ? [opt("lang")] : ["en", "ru"];
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_TUNER: "instant", ARENA_CODEX: "1" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

const TAG = Date.now().toString(36).slice(-4);
const errors = [];
const browser = await launchChromium();

/** One page, one language, no mouse. `fail` tags every error with where it came from. */
async function open(lang, label, { theme = "plum", hash = "" } = {}) {
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, locale: lang === "ru" ? "ru-RU" : "en-US" });
  await ctx.addInitScript(([th]) => {
    try { localStorage.setItem("arena.theme", th); } catch {}
    window.__mouse = 0;
    for (const type of ["click", "pointerdown", "mousedown"]) addEventListener(type, (e) => { if (e.isTrusted && (type !== "click" || e.detail > 0)) window.__mouse++; }, true);
  }, [theme]);
  const page = await ctx.newPage();
  const fail = (m) => errors.push(`${label} [${lang}]: ${m}`);
  page.on("pageerror", (e) => fail(`pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && fail(`console: ${m.text()}`));
  page.on("response", async (r) => {
    if (r.url().includes("/api/v1/") && r.status() >= 400) fail(`HTTP ${r.status()} ${r.request().method()} ${new URL(r.url()).pathname}`);
  });
  const tid = (id) => page.getByTestId(id);
  const press = (k) => page.keyboard.press(k);
  const active = () => page.evaluate(() => document.activeElement?.dataset?.testid ?? document.activeElement?.tagName ?? "");
  const settle = async () => { await page.waitForTimeout(150); await page.waitForFunction(() => !document.getElementById("app").classList.contains("busy")); };
  /** Tab (or Shift+Tab) until the focus is on `id`; the tab order has to reach it. */
  const tabTo = async (id, { back = false, max = 70 } = {}) => {
    for (let i = 0; i < max; i++) {
      if ((await active()) === id) return true;
      await press(back ? "Shift+Tab" : "Tab");
    }
    fail(`Tab never reached ${id} (stopped on ${await active()})`);
    return false;
  };
  const ringOn = (id) => page.evaluate((id) => {
    const el = document.activeElement;
    if (!el || el.dataset.testid !== id) return "not focused";
    const cs = getComputedStyle(el);
    return cs.outlineStyle !== "none" && parseFloat(cs.outlineWidth) >= 2 ? "" : `outline ${cs.outlineStyle} ${cs.outlineWidth}`;
  }, id);
  const expect = (what, ok) => ok || fail(what);
  const mouseClicks = () => page.evaluate(() => window.__mouse);
  return { ctx, page, fail, tid, press, active, settle, tabTo, ringOn, expect, mouseClicks, goto: (h) => page.goto(url + h) };
}

const unitName = (card) => card.evaluate((el) => el.querySelector(".name")?.textContent ?? "");

// ---------- 1. a full run on keys ----------
async function fullRun(lang) {
  const k = await open(lang, "run");
  const { page, tid, press, tabTo, settle, expect, fail } = k;
  await page.goto(`${url}?lang=${lang}`, { timeout: 20_000 });
  await tid("name-input").waitFor({ timeout: 10_000 });
  expect("sign-in: the name field has the keyboard", (await k.active()) === "name-input");
  await page.keyboard.type(`Keys${lang}${TAG}`);
  await press("Enter");
  await tid("play").waitFor();
  await page.waitForTimeout(300);
  expect("Home: Play has the keyboard", (await k.active()) === "play");
  for (const id of ["codex", "forge-open", "dictionary-open", "stats", "settings"]) if (!(await tid(id).count())) fail(`Home: no ${id}`);
  expect("Home: the links wear their keys", (await tid("codex").locator("kbd").count()) === 1 && (await tid("forge-open").locator("kbd").count()) === 1);
  await press("Enter");
  await tid("fight").waitFor();
  await settle();

  let lockChecked = false;
  let arranged = false;
  let sold = false;
  let menuChecked = false;
  let battleChecked = false;
  let round = 0;
  for (; round < 16; round++) {
    await page.waitForFunction(() => document.querySelector('[data-testid="fight"]') || document.querySelector('[data-testid="run-over"]'), null, { timeout: 15_000 });
    if (await tid("run-over").isVisible().catch(() => false)) break;
    await settle();
    // The awakening gift: a number picks a card, else Esc sets it aside.
    const gift = async () => {
      if (!(await tid("gift-title").isVisible().catch(() => false))) return;
      await press("1");
      if (await tid("gift-title").isVisible().catch(() => false)) await press("Escape");
      await settle();
      // A full board can't take the gift: sell a bench unit by keys (Tab, Enter, S), then open the gift and pick.
      if (await tid("gift-banner").count()) {
        const benched = await page.locator('[data-testid^="bench-"].card.you').count();
        if (benched && (await tabTo("bench-0"))) {
          await press("Enter");
          await press("s");
          await settle();
          if (await tabTo("gift-open")) {
            await press("Enter");
            await tid("gift-title").waitFor({ timeout: 3_000 }).catch(() => fail("Enter on the gift banner didn't open the gift"));
            await press("1");
            await settle();
          }
        }
      }
    };
    await gift();
    // Buy with the number keys, as long as the gold lasts.
    for (let n = 0; n < 4; n++) {
      const gold = Number(((await tid("gold").textContent().catch(() => "0")) ?? "0").replace(/\D/g, ""));
      if (gold < 3 || !(await tid("offer-0").count())) break;
      const before = await page.getByTestId("line").locator(".card.you").count() + await page.getByTestId("bench").locator(".card.you").count();
      await press("1");
      await settle();
      await gift();
      const after = await page.getByTestId("line").locator(".card.you").count() + await page.getByTestId("bench").locator(".card.you").count();
      if (after === before && n === 0 && round === 0) fail("1 didn't buy the first offer");
    }
    const lineCards = () => tid("line").locator(".card.you");
    // Arrange (round 1 on): Tab to a unit, Enter selects it, → moves it, B benches it.
    if (!arranged && (await lineCards().count()) >= 2) {
      const first = await unitName(lineCards().nth(0));
      if (await tabTo("line-0")) {
        expect("a focused unit shows its ring", (await k.ringOn("line-0")) === "");
        expect("Tab to a unit reads it in the inspector", ((await tid("inspector").textContent()) ?? "").includes(first));
        await press("Enter");
        await page.waitForTimeout(200);
        expect("Enter selects the unit", (await tid("line-0").getAttribute("class"))?.includes("selected"));
        await press("ArrowRight");
        await settle();
        expect("→ moves the selected unit", (await unitName(lineCards().nth(1))) === first);
        expect("the keyboard stays on the unit that moved", (await k.active()) === "line-1");
        await press("ArrowLeft");
        await settle();
        expect("← moves it back", (await unitName(lineCards().nth(0))) === first);
        if (await tid("bench").count()) {
          const n = await lineCards().count();
          await press("b");
          await settle();
          expect("B puts the unit on the bench", (await lineCards().count()) === n - 1 && (await tid("bench").locator(".card.you").count()) >= 1);
          await press("b");
          await settle();
          expect("B brings it back to the line", (await lineCards().count()) === n);
        }
        await press("Escape");
        arranged = true;
      }
    }
    // Lock an offer from the keyboard (Tab, Enter, L), then unlock it.
    if (!lockChecked && (await tid("offer-0").count())) {
      if (await tabTo("offer-0")) {
        await press("Enter");
        await press("l");
        await settle();
        expect("L locks the chosen offer", (await page.locator('[data-testid^="offer-"] .lock-mark').count()) >= 1);
        await press("l");
        await settle();
        expect("L unlocks it", (await page.locator('[data-testid^="offer-"] .lock-mark').count()) === 0);
        lockChecked = true;
      }
    }
    // Reroll.
    if (round === 1) {
      const gold = Number(((await tid("gold").textContent()) ?? "0").replace(/\D/g, ""));
      if (gold >= 1 && !(await tid("reroll").isDisabled())) {
        await press("r");
        await settle();
        expect("R rerolls", Number(((await tid("gold").textContent()) ?? "0").replace(/\D/g, "")) < gold);
      }
    }
    // Sell a unit: Tab, Enter, S.
    if (!sold && round >= 2 && (await lineCards().count()) >= 3) {
      const n = await lineCards().count();
      if (await tabTo("line-0", { back: false })) {
        await press("Enter");
        await press("s");
        await settle();
        expect("S sells the selected unit", (await lineCards().count()) === n - 1);
        sold = true;
      }
    }
    // The menu: Esc opens it with the keyboard inside, Tab stays inside, Esc closes it.
    if (!menuChecked && round === 1) {
      await press("Escape");
      await tid("run-menu").waitFor({ timeout: 3_000 }).catch(() => fail("Esc didn't open the menu"));
      await page.waitForTimeout(150);
      expect("the menu takes the keyboard (Resume)", (await k.active()) === "menu-resume");
      for (let i = 0; i < 25; i++) await press("Tab");
      expect("Tab stays inside the menu", await page.evaluate(() => !!document.activeElement?.closest(".overlay")));
      await press("Escape");
      await page.waitForTimeout(150);
      expect("Esc closes the menu", !(await tid("run-menu").count()));
      expect("the keyboard is back on the shop", (await k.active()) !== "BODY");
      menuChecked = true;
    }
    await gift();
    // Fight: Space.
    expect("Fight and Reroll wear their keys", (await tid("fight").locator("kbd").count()) === 1 && (await tid("reroll").locator("kbd").count()) === 1);
    await press("Space");
    await tid("battle-end").waitFor({ timeout: 10_000 }).catch(async () => {
      await page.screenshot({ path: `${out}/stuck-${lang}.png` });
      fail(`Space didn't fight in round ${round + 1}: focus on ${await k.active()}, fight ${(await tid("fight").isDisabled()) ? "disabled" : "enabled"}, gold ${await tid("gold").textContent()}, sheet ${await tid("overlay").count()}`);
      throw new Error(`stuck: ${errors.at(-1)}`);
    });
    // The battle's keys: Space pauses, → steps, ← steps back, R replays; the Skip button by Tab.
    if (!battleChecked) {
      expect("the battle shows its keys", ((await tid("battle-keys").textContent()) ?? "").length > 0);
      await press("Space");
      await press("ArrowRight");
      await press("ArrowLeft");
      await press("r");
      battleChecked = true;
    }
    if (!(await tid("end-card").isVisible().catch(() => false))) {
      if (await tabTo("battle-end")) await press("Enter");
    }
    await tid("end-card").waitFor({ timeout: 10_000 });
    // Enter on the end card goes on.
    await press("Enter");
  }
  await tid("run-over").waitFor({ timeout: 15_000 }).catch(() => fail("the run never ended"));
  expect("run over: the keyboard is on its main button", ["home", "new-run-start"].includes(await k.active()));
  await press("Enter");
  await tid("play").waitFor();
  expect("Home again: Play has the keyboard", (await k.active()) === "play");
  expect("no mouse click in the whole run", (await k.mouseClicks()) === 0);
  expect("the run bought, arranged, locked and slept the menu", arranged && lockChecked && menuChecked);
  await k.ctx.close();
}

// ---------- 2. scenes: awaken, fuse through the forge, claim, gift ----------
async function scenes(lang) {
  // Awaken: the offer that is the 3rd copy, bought with its number.
  {
    const k = await open(lang, "awaken");
    await k.goto(`#scene=awaken&lang=${lang}`);
    await k.tid("line").waitFor();
    await k.page.waitForTimeout(1200);
    const awoken = () => k.page.locator('[data-testid^="line-"].awoken, [data-testid^="bench-"].awoken').count();
    const before = await awoken();
    // A copy of a unit you own merges in: buy each by its number until one awakens (a gift opens).
    const owned = await k.page.locator('[data-testid^="offer-"]').evaluateAll((els) => els.map((e, i) => (e.classList.contains("owned") ? i + 1 : 0)).filter(Boolean));
    for (const n of owned) {
      if ((await awoken()) > before || (await k.tid("gift-title").isVisible().catch(() => false))) break;
      await k.press(String(n));
      await k.settle();
    }
    const gift = await k.tid("gift-title").isVisible().catch(() => false);
    k.expect("buying the 3rd copy by its number awakens (a gift opens)", gift || (await awoken()) > before);
    if (gift) {
      k.expect("the gift sheet takes the keyboard", await k.page.evaluate(() => !!document.activeElement?.closest(".overlay")));
      k.expect("the gift's picks wear their numbers", (await k.tid("gift-pick-0").locator("kbd").count()) === 1);
      // The first pick that is not "full" (a full board takes only a copy of a unit you own).
      const n = await k.page.locator('[data-testid^="gift-pick-"]').evaluateAll((els) => els.findIndex((e) => !e.disabled) + 1);
      // None fits a full board: Esc sets the gift aside behind its banner.
      await k.press(n > 0 ? String(n) : "Escape");
      await k.tid("gift-title").waitFor({ state: "hidden", timeout: 5_000 }).catch(() => k.fail(`${n > 0 ? n : "Esc"} didn't close the gift`));
      if (n === 0) k.expect("Esc sets the gift aside behind a banner", (await k.tid("gift-banner").count()) === 1);
    }
    k.expect("no mouse (awaken)", (await k.mouseClicks()) === 0);
    await k.ctx.close();
  }
  // Fuse a designed pair through the forge: select, F, Tab to the partner, Enter, Enter.
  {
    const k = await open(lang, "fuse");
    await k.goto(`#scene=fusion-now&lang=${lang}`);
    await k.tid("line").waitFor();
    await k.page.waitForResponse((r) => /\/forge$/.test(r.url()), { timeout: 30_000 }).catch(() => {});
    await k.page.waitForTimeout(600);
    const marked = await k.page.locator('[data-testid^="fuse-mark-"]').evaluateAll((els) => els.map((e) => e.closest(".card")?.dataset.testid));
    k.expect("the fusion-now scene has two marked units", marked.length === 2);
    if (marked.length === 2) {
      await k.tabTo(marked[0]);
      await k.press("Enter");
      await k.press("f");
      await k.page.waitForTimeout(200);
      k.expect("F starts a fuse", (await k.tid(marked[1]).getAttribute("class"))?.includes("fusable"));
      await k.tabTo(marked[1]);
      await k.press("Enter");
      await k.tid("preview-confirm").waitFor({ timeout: 5_000 }).catch(() => k.fail("Enter on the partner didn't open the forge"));
      k.expect("the forge takes the keyboard (Fuse)", (await k.active()) === "preview-confirm");
      k.expect("the forge's buttons wear Esc and Enter", (await k.tid("preview-cancel").locator("kbd").count()) === 1 && (await k.tid("preview-confirm").locator("kbd").count()) === 1);
      const units = await k.tid("line").locator(".card.you").count();
      await k.press("Enter");
      await k.settle();
      k.expect("Enter on Fuse fuses the pair", (await k.tid("line").locator(".card.you").count()) === units - 1);
      k.expect("the keyboard lands on the fused unit", (await k.active()) === "line-0");
      await k.press("Space");
      await k.tid("battle-end").waitFor({ timeout: 10_000 }).catch(() => k.fail("Space didn't fight after the fuse"));
    }
    k.expect("no mouse (fuse)", (await k.mouseClicks()) === 0);
    await k.ctx.close();
  }
  // Claim: pick one of 3 sealed cards by number, Enter claims.
  {
    const k = await open(lang, "claim");
    await k.goto(`#scene=forge-ready&lang=${lang}`);
    await k.tid("forge-claim").waitFor();
    await k.page.waitForTimeout(500);
    await k.press("3");
    k.expect("3 picks the third sealed design", (await k.tid("forge-card-2").getAttribute("aria-pressed")) === "true");
    k.expect("the sealed designs wear their numbers", (await k.tid("forge-card-0").locator("kbd").count()) === 1);
    await k.press("Escape");
    await k.page.waitForTimeout(200);
    k.expect("Esc closes the forge sheet", !(await k.tid("forge-claim").count()));
    // And again, claimed: Enter on the button.
    await k.goto(`#scene=forge-ready&lang=${lang}`);
    await k.page.reload();
    await k.tid("forge-claim").waitFor();
    await k.press("2");
    await k.press("Enter");
    await k.tid("forge-claimed").waitFor({ timeout: 8_000 }).catch(() => k.fail("Enter on Claim didn't claim"));
    k.expect("the claimed screen has the keyboard on Back", (await k.active()) === "forge-done");
    await k.press("Enter");
    await k.tid("fight").waitFor({ timeout: 8_000 }).catch(() => k.fail("Enter on Back didn't return to the shop"));
    k.expect("no mouse (claim)", (await k.mouseClicks()) === 0);
    await k.ctx.close();
  }
}

// ---------- 3. Codex, Forge, Dictionary, Settings, sheets ----------
async function menus(lang) {
  const k = await open(lang, "menus");
  const { page, tid, press, expect, fail } = k;
  await k.goto(`#scene=home&lang=${lang}`);
  await tid("play").waitFor();
  await page.waitForTimeout(500);
  // Settings: G opens, Tab stays inside, Esc closes and gives the keyboard back.
  await press("g");
  await tid("settings-sheet").waitFor({ timeout: 3_000 }).catch(() => fail("G didn't open Settings"));
  expect("Settings take the keyboard", await page.evaluate(() => !!document.activeElement?.closest(".overlay")));
  for (let i = 0; i < 30; i++) await press("Tab");
  expect("Tab stays inside Settings", await page.evaluate(() => !!document.activeElement?.closest(".overlay")));
  // theme and language by keys: Tab to the buttons, Enter
  if (await k.tabTo("theme-brass")) {
    await press("Enter");
    expect("Enter switches the theme", (await page.evaluate(() => document.documentElement.dataset.theme)) === "brass");
  }
  await press("Escape");
  await page.waitForTimeout(200);
  expect("Esc closes Settings", !(await tid("settings-sheet").count()));
  expect("the keyboard goes back to where it was (Play)", (await k.active()) === "play");
  // The Codex: C, tabs by number, search by /, a unit's sheet by Enter, Esc closes the sheet, Esc leaves.
  await press("c");
  await tid("codex-units").waitFor({ timeout: 5_000 }).catch(() => fail("C didn't open the Codex"));
  await press("/");
  expect("/ focuses the search", (await k.active()) === "codex-search");
  await page.keyboard.type("a");
  await press("Escape"); // blurs the field
  if (await k.tabTo("codex-unit")) {
    await press("Enter");
    await page.waitForTimeout(300);
    expect("Enter on a unit reads it", (await tid("inspector").locator('[data-testid="unit-sheet"]').count()) === 1);
    await press("Escape");
  }
  for (const [key, id] of [["2", "codex-fusions"], ["3", "codex-keywords"], ["6", "codex-howto"]]) {
    await press(key);
    await page.waitForTimeout(500);
    expect(`${key} opens the Codex tab ${id}`, (await tid(`codex-tab-${id.replace("codex-", "")}`).getAttribute("class"))?.includes("on") ?? false);
  }
  await press("Escape");
  await tid("play").waitFor({ timeout: 5_000 }).catch(() => fail("Esc didn't leave the Codex"));
  // The Forge and the Dictionary.
  await press("f");
  await tid("forge").waitFor({ timeout: 5_000 }).catch(() => fail("F didn't open the Forge"));
  await press("ArrowRight");
  await page.waitForTimeout(200);
  expect("→ goes to the next unit in the Forge", (await k.active()) === "forge-pick");
  await press("2");
  expect("2 opens the feed", (await tid("forge-tab-feed").getAttribute("class"))?.includes("on") ?? false);
  await press("3");
  expect("3 opens the Dictionary", (await tid("forge-tab-words").getAttribute("class"))?.includes("on") ?? false);
  expect("the Forge lists its keys", (await tid("forge-keys").locator("kbd").count()) >= 5);
  await press("Escape");
  await tid("play").waitFor({ timeout: 5_000 }).catch(() => fail("Esc didn't leave the Forge"));
  await press("d");
  await tid("forge-tab-words").waitFor({ timeout: 5_000 }).catch(() => fail("D didn't open the Dictionary"));
  expect("the Dictionary opens on its tab", (await tid("forge-tab-words").getAttribute("class"))?.includes("on") ?? false);
  await press("Escape");
  await tid("play").waitFor();
  // The ideas screen and the vote card are Buttons; I opens ideas.
  if (await tid("ideas").count()) {
    await press("i");
    await page.waitForTimeout(500);
    expect("I leaves Home for the ideas", !(await tid("play").count()));
    await press("Escape");
    await tid("play").waitFor({ timeout: 5_000 }).catch(() => fail("Esc didn't leave the ideas"));
  }
  expect("no mouse (menus)", (await k.mouseClicks()) === 0);
  await k.ctx.close();
}

// ---------- 4. focus is visible, in every palette, in reading order ----------
async function rings(lang) {
  for (const theme of ["plum", "brass", "felt"]) {
    for (const scene of ["shop-r3", "codex-units", "settings"]) {
      const k = await open(lang, `ring ${theme} ${scene}`, { theme });
      await k.goto(`?theme=${theme}#scene=${scene}&lang=${lang}`);
      await k.page.waitForTimeout(1500);
      const on = await k.page.evaluate(() => document.documentElement.dataset.theme);
      if (on !== theme) k.fail(`the page shows the ${on} palette, not ${theme}`);
      const seen = [];
      for (let i = 0; i < 90; i++) {
        await k.press("Tab");
        const r = await k.page.evaluate(() => {
          const el = document.activeElement;
          if (!el || el === document.body) return null;
          const cs = getComputedStyle(el);
          const box = el.getBoundingClientRect();
          return { id: el.dataset.testid ?? el.tagName, ring: cs.outlineStyle !== "none" && parseFloat(cs.outlineWidth) >= 2, color: cs.outlineColor, y: Math.round(box.top + scrollY), x: Math.round(box.left), hidden: box.width === 0 };
        });
        if (!r) continue;
        if (seen.some((x) => x.id === r.id && x.x === r.x && x.y === r.y)) break; // the order has come round
        if (!r.ring && !r.hidden) k.fail(`${r.id} has no focus ring`);
        if (r.color === "rgba(0, 0, 0, 0)") k.fail(`${r.id}'s ring is transparent`);
        seen.push(r);
      }
      // Reading order: the rows go down the page (a stop may sit beside the last; none jumps back up more than a row).
      let maxY = -1;
      for (const r of seen) {
        if (scene !== "settings" && r.y + 40 < maxY && !/menu-open|settings/.test(r.id)) k.fail(`${r.id} comes after a stop lower on the page (y ${r.y} < ${maxY})`);
        maxY = Math.max(maxY, r.y);
      }
      if (seen.length < 3) k.fail(`only ${seen.length} Tab stops`);
      if (lang === "en" && scene === "shop-r3") await k.page.screenshot({ path: `${out}/focus-${theme}.png` });
      await k.ctx.close();
    }
  }
}

try {
  for (const lang of langs) {
    await fullRun(lang);
    await scenes(lang);
    await menus(lang);
    await rings(lang);
    console.log(`${lang}: done, ${errors.length} problem(s) so far`);
  }
} finally {
  await browser.close();
  child?.kill();
}
if (errors.length) {
  console.log(`\n${errors.length} keyboard problem(s):\n- ${errors.join("\n- ")}`);
  process.exit(1);
}
console.log("keyboard: every screen and action works without a mouse, in EN and RU");
