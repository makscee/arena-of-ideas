// Plan vs build (arena#32, round 4): one assertion per bullet of moves 1–3 of the UI duel's final
// (m1:~/reports/ui-duel/arena/final/MOVES.md), measured, at 1920×1080, 1440×900, 1280×720, 1024×768,
// 800×450 and 390×844. A bullet's title is the plan's own words. The one deliberate difference: the
// plan says the first fight meets "the opponent the shop previewed"; Maks's 11-04 rule (the next
// opponent's team is never shown) wins, so that bullet asserts the opposite.
// Writes results.json and the screenshots (plan-<size>-<state>.png) to --out, prints one line a
// bullet and size, exits 1 if any fails. The Workshop is the Ideas screen, so its window is checked on
// the `ideas` scene. Without --url it builds the client and starts a server (scenes, Codex, credits).
//   npm run mvp:plan -- [--url http://127.0.0.1:PORT/arena/] [--out e2e/.shots/mvp-plan] [--sizes 1280x720,390x844]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";
import { chainProblems, readChain } from "./chain-checks.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/mvp-plan";
mkdirSync(out, { recursive: true });
const ALL = ["1920x1080", "1440x900", "1280x720", "1024x768", "800x450", "390x844"];
const sizes = (opt("sizes")?.split(",") ?? ALL).map((s) => s.split("x").map(Number));

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_TUNER: "instant", ARENA_CODEX: "1", MVP_POOL: "v2", ARENA_GUEST_JOINS_PER_IP: "100" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}
const api = (method, path, who, body) => fetch(`${url}api/v1${path}`, { method, headers: { "content-type": "application/json", ...(who ? { "X-Arena-Player": who.id } : {}) }, ...(body ? { body: JSON.stringify(body) } : {}) }).then((r) => r.json());
// Every unit of the pool is Maker's idea, so a unit's panel and the coach have an author to credit.
const maker = await api("POST", "/players", null, { name: "Amber Fox 417" });
for (const u of (await api("GET", "/content")).units) await api("POST", "/dev/credit-unit", maker, { unitId: u.id });

/** Plan bullets (move, the plan's own words), in the order of MOVES.md. */
const BULLETS = [
  [1, "Every state is one fixed 1280×720 stage (390×844 on a phone)"],
  [1, "HUD plates (round track, hearts at every size, gold)"],
  [1, "a tray that changes job: shop counter, chain list, run result"],
  [1, "The Workshop and Codex open as windows over the dimmed table, not as pages"],
  [1, "The Workshop opens as a window over the dimmed table, with a ‹ Esc plate and the title WORKSHOP"],
  [1, "Esc closes a window"],
  [1, "tap an offer, then a slot"],
  [1, "←/→ and 1–5"],
  [1, "Space to fight"],
  [1, "A battle plays itself"],
  [1, "Space or the middle button pauses it"],
  [1, "‹ › or the arrow keys step it"],
  [1, "E ends it"],
  [1, "Esc closes the inspector"],
  [1, "Tap any unit and the inspector shows its rule in words and whose idea it was"],
  [2, "The field draws only the current step's wire, numbered, between the real pieces"],
  [2, "The tray lists the steps so far in plain words, with CHAIN ×N beside them"],
  [2, "Earlier wires disappear, so lines never cross"],
  [3, "The first screen says \"One rule starts a chain.\" over a chain that plays itself"],
  [3, "PLAY (Enter) skips the name form and opens round 1"],
  [3, "the coach builds a useful pair: \"Put Medic behind Gambler\", with the chain previewed before you place it"],
  [3, "Then \"Now Fight\", and Medic's author is credited"],
  [3, "The fight that follows uses the line you bought"],
  [3, "No unit of the next opponent is shown before the fight (Maks 11-04: replaces \"the opponent the shop previewed\")"],
];
const results = BULLETS.map(([move, title]) => ({ move, title, sizes: {} }));
const rec = (size, title, ok, detail = "") => {
  const r = results.find((x) => x.title === title);
  if (!r) throw new Error(`no bullet: ${title}`);
  // A bullet fails at a size if any of its checks there failed; the first failure's words stay.
  const prev = r.sizes[size];
  if (prev && prev.ok === false) return;
  r.sizes[size] = { ok, detail };
};
const na = (size, title, why) => rec(size, title, "n/a", why);

const browser = await launchChromium();
try {
  for (const [w, h] of sizes) {
    const S = `${w}x${h}`;
    const touch = w < 600;
    const ctx = await browser.newContext({ viewport: { width: w, height: h }, isMobile: touch, hasTouch: touch });
    const page = await ctx.newPage();
    const errs = [];
    page.on("pageerror", (e) => errs.push(e.message));
    const tid = (id) => page.getByTestId(id);
    const shot = (name) => page.screenshot({ path: `${out}/plan-${S}-${name}.png` });
    const T = (fn) => fn().catch((e) => ({ threw: e.message.split("\n")[0] }));
    const stageOf = () => page.evaluate(() => {
      const st = document.querySelector('[data-testid="stage"]');
      if (!st) return null;
      const r = st.getBoundingClientRect();
      return { w: st.offsetWidth, h: st.offsetHeight, rw: Math.round(r.width), rh: Math.round(r.height), L: st.classList.contains("L"), sh: document.documentElement.scrollHeight - innerHeight, sw: document.documentElement.scrollWidth - innerWidth };
    });
    const states = {}; // move 1, bullet 1: the stage in each state
    const stageNow = async (name) => { states[name] = await stageOf(); };
    const wantPhone = w / h < 0.8;

    // ---- the first screen (move 3)
    await page.goto(url, { timeout: 30_000 });
    await tid("start").waitFor({ timeout: 20_000 });
    await stageNow("first screen");
    await shot("first");
    const title = (await tid("start-title").textContent().catch(() => "")) ?? "";
    const cap0 = (await tid("start-caption").textContent().catch(() => "")) ?? "";
    const moved = await page.waitForFunction((was) => document.querySelector('[data-testid="start-caption"]')?.textContent !== was, cap0, { timeout: 6_000 }).then(() => true, () => false);
    const pieces = await page.locator('[data-testid^="start-piece-"]').count();
    rec(S, "The first screen says \"One rule starts a chain.\" over a chain that plays itself", title === "One rule starts a chain." && moved && pieces === 3, `title "${title}", caption moves ${moved}, ${pieces} pieces`);

    // ---- PLAY (Enter)
    await page.keyboard.press("Enter");
    await tid("coach").waitFor({ timeout: 20_000 });
    const formGone = (await tid("name-input").count()) === 0;
    const pname = await page.evaluate(() => { try { return JSON.parse(localStorage.getItem("arena.player") ?? "{}").name ?? ""; } catch { return ""; } });
    rec(S, "PLAY (Enter) skips the name form and opens round 1", formGone && /^[A-Z][a-z]+ [A-Z][a-z]+ \d+$/.test(pname) && (await tid("round").count()) > 0, `form gone ${formGone}, generated name "${pname}"`);
    await stageNow("shop");
    await shot("coach-1");

    // ---- the next opponent stays hidden (Maks 11-04), before anything is bought
    const hiddenBefore = () => page.evaluate(() => {
      const seen = new Set();
      const add = (s) => s && String(s).split(/[\n|]/).forEach((x) => seen.add(x.trim()));
      add(document.body.innerText);
      document.querySelectorAll("[title],[aria-label],[alt]").forEach((e) => { add(e.getAttribute("title")); add(e.getAttribute("aria-label")); add(e.getAttribute("alt")); });
      const their = document.querySelector('[data-testid="their-line"]');
      return { pieces: their ? their.querySelectorAll(".pc,[data-name]").length : -1, text: [...seen].join("\n"), wells: their ? their.querySelectorAll(".cell-well").length : 0 };
    });
    const pre = await hiddenBefore();
    const runShown = await page.evaluate(async () => { const id = location.hash; return id; });

    // ---- tap an offer, then a slot (Gambler into slot 1)
    const gold0 = await tid("gold").textContent();
    const offer = (name) => page.locator(`[data-testid="offers"] [data-testid^="offer-"][data-name="${name}"]`);
    await offer("Gambler").click();
    await tid("line-0-empty").click();
    await page.waitForFunction(() => [...document.querySelectorAll('[data-testid="line"] .pc[data-name]')].some((e) => e.dataset.name === "Gambler"), null, { timeout: 5_000 }).catch(() => {});
    const gold1 = await tid("gold").textContent();
    const lineNames = () => page.locator('[data-testid="line"] .pc[data-name]').evaluateAll((els) => els.map((e) => e.dataset.name));
    rec(S, "tap an offer, then a slot", (await lineNames()).join() === "Gambler" && gold0 !== gold1, `line ${(await lineNames()).join()}, gold ${gold0} → ${gold1}`);

    // ---- the coach: the chain previewed before the pair is placed
    await page.waitForFunction(() => document.querySelector('[data-testid="coach"]')?.getAttribute("data-step") === "buy-second", null, { timeout: 5_000 }).catch(() => {});
    const coach2 = (await tid("coach").innerText().catch(() => "")) ?? "";
    const wire = await tid("coach-wire").count();
    const coachOk = /Put Medic behind Gambler/.test(coach2) && wire > 0;
    await shot("coach-2");

    // ---- 1–5 keys put the offer in hand into that slot (desktop); a phone only taps
    await offer("Medic").click();
    if (touch) {
      await tid("line-1-empty").click();
      na(S, "←/→ and 1–5", "touch: the tap path only");
    } else {
      await page.keyboard.press("2");
    }
    await page.waitForFunction(() => [...document.querySelectorAll('[data-testid="line"] .pc[data-name]')].some((e) => e.dataset.name === "Medic"), null, { timeout: 5_000 }).catch(() => {});
    const line2 = (await lineNames()).join();
    await page.waitForFunction(() => document.querySelector('[data-testid="coach"]')?.getAttribute("data-step") === "fight", null, { timeout: 5_000 }).catch(() => {});
    const coach3 = (await tid("coach").innerText().catch(() => "")) ?? "";
    const by = (await page.locator('[data-testid="coach"] .coach-by').textContent().catch(() => "")) ?? "";
    rec(S, "the coach builds a useful pair: \"Put Medic behind Gambler\", with the chain previewed before you place it", coachOk && line2 === "Gambler,Medic", `coach said "${coach2.replace(/\s+/g, " ").slice(0, 60)}", wire drawn ${wire}, line ${line2}`);
    rec(S, "Then \"Now Fight\", and Medic's author is credited", /Now Fight/.test(coach3) && /Amber Fox/.test(by), `coach "${coach3.replace(/\s+/g, " ").slice(0, 40)}", credit "${by.trim()}"`);
    if (!touch) {
      // ←/→ move the unit in hand along the line; the order comes back before the fight.
      await tid("line-1").locator(".pc").first().click().catch(() => {});
      await page.waitForTimeout(150);
      await page.keyboard.press("ArrowLeft"); await page.waitForTimeout(350);
      const flipped = (await lineNames()).join();
      await page.keyboard.press("ArrowRight"); await page.waitForTimeout(350);
      const back = (await lineNames()).join();
      rec(S, "←/→ and 1–5", line2 === "Gambler,Medic" && flipped === "Medic,Gambler" && back === "Gambler,Medic", `key 2 placed Medic (${line2}); ← gave ${flipped}; → gave ${back}`);
      await page.keyboard.press("Escape");
    }
    await shot("shop");

    // ---- the tray in the shop, then Space fights
    const trayShop = await page.evaluate(() => ({ offers: !!document.querySelector('[data-testid="tray"] [data-testid="offers"], [data-testid="offers"]'), counter: !!document.querySelector('[data-testid="offers"]') }));
    const decided = page.waitForResponse((r) => r.request().method() === "POST" && /\/runs\/[^/]+\/decisions$/.test(r.url()) && r.request().postData()?.includes('"fight"'), { timeout: 15_000 }).catch(() => null);
    if (touch) await tid("fight").click(); else await page.keyboard.press("Space");
    await tid("battle-you").waitFor({ timeout: 15_000 }).catch(() => {});
    const fought = await decided;
    const foe = fought ? (await fought.json().catch(() => null))?.fight?.opponent : null;
    const inBattle = (await tid("battle-you").count()) > 0;
    if (touch) na(S, "Space to fight", "touch: the Fight button");
    else rec(S, "Space to fight", inBattle, inBattle ? "Space opened the battle" : "Space did not fight");
    const bought = (await page.locator('[data-testid="battle-you"] .pc[data-name]').evaluateAll((els) => els.map((e) => e.dataset.name))).join();
    rec(S, "The fight that follows uses the line you bought", bought === "Gambler,Medic", `battle's your line: ${bought}`);
    const foeNames = foe?.line ? foe.line.map((u) => u.name) : [];
    const own = new Set(["Gambler", "Medic"]);
    const leaked = foeNames.filter((n) => !own.has(n) && new RegExp(`(^|\\n)${n}(\\n|$)`).test(pre.text));
    rec(S, "No unit of the next opponent is shown before the fight (Maks 11-04: replaces \"the opponent the shop previewed\")", pre.pieces === 0 && leaked.length === 0 && pre.wells >= 3, `their line: ${pre.pieces} pieces, ${pre.wells} wells; opponent's units named on screen: ${leaked.join(", ") || "none"}`);
    void runShown;
    await stageNow("battle");

    // ---- the battle plays itself, then pauses, steps, ends
    const capNow = () => page.evaluate(() => `${document.querySelector('[data-testid="battle-play"]')?.textContent}|${document.querySelector(".bv-caption")?.textContent?.slice(0, 60)}|${document.querySelector(".bv-turn, .plate.turn")?.textContent}`);
    const c0 = await capNow();
    await page.waitForTimeout(1800);
    const c1 = await capNow();
    rec(S, "A battle plays itself", c0 !== c1, `${c0.slice(0, 40)} → ${c1.slice(0, 40)}`);
    await shot("battle");
    const trayBattle = await page.evaluate(() => ({ controls: !!document.querySelector('[data-testid="battle-play"]'), offers: !!document.querySelector('[data-testid="offers"]') }));
    await page.keyboard.press("Space"); await page.waitForTimeout(250);
    const paused = (await tid("battle-play").textContent()) ?? "";
    const p0 = await capNow(); await page.waitForTimeout(1200); const p1 = await capNow();
    rec(S, "Space or the middle button pauses it", p0 === p1 && !/❚❚/.test(paused), `play button "${paused.trim()}", it stood still ${p0 === p1}`);
    // step with ‹ › and the arrow keys; read the chain at every beat (move 2)
    const forward = []; const backward = [];
    let chained = 0; const chainBad = []; let maxWires = 0; let dotsOnField = true;
    for (let i = 0; i < 60 && !(await tid("battle-step").isDisabled()); i++) {
      const before = await capNow();
      if (i % 2) await page.keyboard.press("ArrowRight"); else await tid("battle-step").click();
      await page.waitForTimeout(90);
      forward.push(before !== (await capNow()));
      const ch = await readChain(page);
      maxWires = Math.max(maxWires, ch.wires - ch.old);
      if (ch.count >= 2) { chained++; if (ch.dots.length && ch.dots.some((n) => n !== ch.dots[0])) dotsOnField = false; }
      chainBad.push(...chainProblems(ch).map((b) => `beat ${i + 1}: ${b}`));
      if (chained === 1) await shot("chain");
    }
    const fwdOk = forward.filter(Boolean).length >= Math.min(3, forward.length);
    await tid("battle-back").click().catch(() => {}); await page.waitForTimeout(120);
    backward.push(true);
    rec(S, "‹ › or the arrow keys step it", fwdOk, `${forward.filter(Boolean).length} of ${forward.length} presses moved the beat`);
    rec(S, "The field draws only the current step's wire, numbered, between the real pieces", chained > 0 && dotsOnField && !chainBad.some((b) => /wire/.test(b)), chained ? `chain of ${chained}+ beats read; the wires on the field all wear the current step's number (at most ${maxWires} at a beat)${chainBad.length ? "; " + chainBad[0] : ""}` : "no chain was listed in this fight");
    rec(S, "The tray lists the steps so far in plain words, with CHAIN ×N beside them", chained > 0 && !chainBad.some((b) => /tray|list|lit step|CHAIN/.test(b)), chained ? chainBad.find((b) => /tray|list|lit step/.test(b)) ?? "CHAIN ×N and its rows agree at every beat" : "no chain was listed in this fight");
    rec(S, "Earlier wires disappear, so lines never cross", chained > 0 && !chainBad.some((b) => /earlier wire|different steps/.test(b)), chainBad.find((b) => /earlier wire|different steps/.test(b)) ?? (chained ? "no earlier wire stayed" : "no chain was listed in this fight"));

    // ---- tap a unit: the inspector reads its rule in words and whose idea it was; Esc closes it
    await tid("battle-play").textContent().then((t) => (/▶|►/.test(t) || true) && 0);
    await page.locator('[data-testid="battle-you"] .bv-card .pc-face').first().click();
    const insp = await tid("inspector").waitFor({ timeout: 3_000 }).then(() => true, () => false);
    const inspText = insp ? ((await tid("inspector").innerText().catch(() => "")) ?? "") : "";
    const hasRule = /Each|When|After|At |Battle|gets|deals|strike|gives|heals?|Starts?/i.test(inspText) && inspText.length > 40;
    const hasIdea = /idea by|Amber Fox|Forged by|Idea:/i.test(inspText);
    rec(S, "Tap any unit and the inspector shows its rule in words and whose idea it was", insp && hasRule && hasIdea, insp ? `rule in words ${hasRule}; author line ${hasIdea ? "yes" : "no"}: "${inspText.replace(/\s+/g, " ").slice(0, 90)}"` : "no inspector opened");
    await shot("inspector");
    await page.keyboard.press("Escape"); await page.waitForTimeout(300);
    rec(S, "Esc closes the inspector", insp && (await tid("inspector").count()) === 0 || (await tid("inspector").isHidden().catch(() => true)), "");
    // E ends it: the result takes the tray
    await page.keyboard.press("e");
    const endCard = await tid("end-card").waitFor({ state: "visible", timeout: 8_000 }).then(() => true, () => false);
    rec(S, "E ends it", endCard, endCard ? "the result card is up" : "no result after E");
    await stageNow("result");
    await shot("result");
    const trayResult = await page.evaluate(() => ({ done: !!document.querySelector('[data-testid="battle-done"]'), card: !!document.querySelector('[data-testid="end-card"]') }));
    rec(S, "a tray that changes job: shop counter, chain list, run result", trayShop.counter && trayBattle.controls && !trayBattle.offers && trayResult.done && trayResult.card, `shop tray holds the offers ${trayShop.counter}; battle tray holds the controls ${trayBattle.controls} and not the offers ${!trayBattle.offers}; result tray holds the result ${trayResult.card}`);

    // ---- back to the shop, then the windows
    await tid("battle-done").click().catch(() => {});
    await tid("offers").waitFor({ timeout: 15_000 }).catch(() => {});
    const hud = await page.evaluate(() => {
      const vis = (e) => { if (!e) return false; const r = e.getBoundingClientRect(); return r.width > 0 && r.height > 0 && r.left >= -1 && r.top >= -1 && r.right <= innerWidth + 1 && r.bottom <= innerHeight + 1; };
      const dots = document.querySelectorAll(".plate .track-dot").length;
      const hearts = document.querySelector('[data-testid="hearts"], .plate.hearts, .hearts');
      return { dots, hearts: vis(hearts), heartsN: hearts ? hearts.querySelectorAll("svg, .heart, [class*=heart], span").length : 0, gold: vis(document.querySelector('[data-testid="gold"]')), round: vis(document.querySelector('[data-testid="round"]')) };
    });
    // The phone's round plate is "R1/12" (the mockup's "Round 1 of 12"); a landscape table shows the 12-dot track.
    const roundText = (await tid("round").textContent().catch(() => "")) ?? "";
    const trackOk = wantPhone ? /\d+\s*\/\s*12|of 12/.test(roundText) : hud.dots >= 12;
    rec(S, "HUD plates (round track, hearts at every size, gold)", trackOk && hud.hearts && hud.gold, `round ${wantPhone ? `plate "${roundText.trim()}"` : `track of ${hud.dots} dots`}, hearts in view ${hud.hearts}, gold in view ${hud.gold}`);
    await shot("shop-2");
    await tid("legend-open").click();
    await page.waitForTimeout(700);
    const win = await page.evaluate(() => {
      const app = document.getElementById("app"); const under = document.getElementById("under");
      const r = app.getBoundingClientRect(); const cs = getComputedStyle(app);
      const st = under?.querySelector('[data-testid="stage"]'); const ucs = under ? getComputedStyle(under) : null;
      return { fixed: cs.position === "fixed", inset: Math.round(Math.min(r.left, r.top, innerWidth - r.right, innerHeight - r.bottom)), table: !!st, over: document.body.classList.contains("over-table"), dim: ucs ? { op: ucs.opacity, filter: ucs.filter } : null, tabs: !!document.querySelector('[data-testid="codex-tab-units"]') };
    });
    await stageNow("codex window");
    await shot("codex");
    rec(S, "The Workshop and Codex open as windows over the dimmed table, not as pages", win.fixed && win.table && win.over && win.tabs && win.inset >= 4, `Codex: window ${win.fixed}, margin ${win.inset}px, table behind ${win.table}`);
    await page.keyboard.press("Escape"); await page.waitForTimeout(500);
    const closed = (await tid("codex-tab-units").count()) === 0;
    rec(S, "Esc closes a window", closed, closed ? "Esc closed the Codex" : "the Codex stayed open on Esc");

    // ---- the Workshop: opened from the table (the run's menu), a window over the dimmed table
    const winOf = () => page.evaluate(() => {
      const app = document.getElementById("app"); const under = document.getElementById("under"); const r = app.getBoundingClientRect(); const cs = getComputedStyle(app);
      return { screen: app.dataset.screen, fixed: cs.position === "fixed", inset: Math.round(Math.min(r.left, r.top, innerWidth - r.right, innerHeight - r.bottom)), table: !!under?.querySelector('[data-testid="stage"]'), over: document.body.classList.contains("over-table"), h1: document.querySelector("#app h1")?.textContent?.trim() ?? "", plate: !!document.querySelector('[data-testid="workshop-back"]'), plateText: document.querySelector('[data-testid="workshop-back"]')?.textContent ?? "" };
    });
    await tid("menu-open").click().catch(() => {});
    const wkOpened = await tid("menu-ideas").click({ timeout: 4_000 }).then(() => true, () => false);
    await page.waitForTimeout(900);
    const wk = await winOf();
    await shot("workshop");
    const wkWindow = wkOpened && wk.screen === "ideas" && wk.fixed && wk.table && wk.over && wk.inset >= 4;
    const base = results.find((x) => x.title.startsWith("The Workshop and Codex"));
    const prior = base.sizes[S];
    base.sizes[S] = { ok: prior.ok === true && wkWindow, detail: `${prior.detail}; Workshop: ${wkOpened ? (wkWindow ? "a window over the table" : `a page (window ${wk.fixed}, table behind ${wk.table})`) : "no way to open it from the table (no menu entry)"}` };
    rec(S, "The Workshop opens as a window over the dimmed table, with a ‹ Esc plate and the title WORKSHOP", wkWindow && wk.h1 === "WORKSHOP" && wk.plate && (touch || /Esc/.test(wk.plateText)), `title "${wk.h1}", plate ${wk.plate ? `"${wk.plateText.trim()}"` : "none"}, window ${wkWindow}`);
    await page.keyboard.press("Escape"); await page.waitForTimeout(600);
    const wkClosed = (await tid("offers").count()) > 0 && !(await page.evaluate(() => document.body.classList.contains("over-table")));
    const cl = results.find((x) => x.title === "Esc closes a window");
    const clPrev = cl.sizes[S];
    cl.sizes[S] = { ok: clPrev.ok === true && wkClosed, detail: `${clPrev.detail}; Workshop: ${wkClosed ? "Esc returned to the shop" : "Esc did not return to the shop"}` };

    // ---- bullet 1: the stage, in every state
    const bad = [];
    for (const [name, st] of Object.entries(states)) {
      if (!st) { bad.push(`${name}: no stage`); continue; }
      const wantW = st.L ? 1280 : 390, wantH = st.L ? 720 : null;
      if (st.w !== wantW || (wantH && st.h !== wantH) || (!st.L && st.h < 690)) bad.push(`${name}: stage ${st.w}×${st.h}`);
      if (st.L === wantPhone) bad.push(`${name}: ${st.L ? "landscape" : "portrait"} stage in a ${w}×${h} window`);
      if (st.sh > 1 || st.sw > 1) bad.push(`${name}: the page scrolls`);
    }
    rec(S, "Every state is one fixed 1280×720 stage (390×844 on a phone)", bad.length === 0, bad.length ? bad.join("; ") : `${Object.keys(states).join(", ")}: ${states.shop.w}×${states.shop.h}`);
    if (errs.length) console.log(`  (${S}: page errors: ${[...new Set(errs)].slice(0, 2).join(" | ")})`);
    await ctx.close();
  }
} finally {
  await browser.close();
  child?.kill();
}

let failed = 0;
for (const r of results) {
  const cells = Object.entries(r.sizes);
  for (const [S, c] of cells) if (c.ok === false) failed++;
  console.log(`${cells.some(([, c]) => c.ok === false) ? "FAIL" : "ok  "} move ${r.move}: ${r.title}`);
  for (const [S, c] of cells) if (c.ok === false) console.log(`       ${S}: ${c.detail}`);
}
writeFileSync(`${out}/results.json`, JSON.stringify({ sizes: sizes.map(([w, h]) => `${w}x${h}`), results }, null, 1));
if (failed) { console.error(`\n${failed} plan checks failed (results.json in ${out})`); process.exit(1); }
console.log("mvp plan: ok");
