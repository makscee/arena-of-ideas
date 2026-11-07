// Buying on a priced gold target (arena#49, MOVES 2 and 4 of the shop duel), at 800×450 and 390×844, EN and RU:
//  - no Buy button; the offer in hand lights ONE gold destination (the next free well, the twin) wearing its price
//    ("Buy 3g", "+1 copy", "Awaken 3g" over the numbers), and nothing else;
//  - a buy is one gold tap: by a tap, by a drop, by the slot's key and by Enter; gold and a receipt appear at once;
//  - a tap on your own unit only reads it (nothing is armed, nothing is bought); Move arms named teal destinations;
//  - the tag never covers the name, no text under 11 px at 800×450, no page scroll, the rows above the tray don't move.
// Needs a scenes server (arena-lab.sh or ARENA_SCENES=1).
//   node e2e/shop-buy.mjs --url http://127.0.0.1:PORT/arena/ [--out dir] [--shots]
import { launchChromium } from "./browser.mjs";
import { mkdirSync } from "node:fs";

const args = process.argv.slice(2);
const opt = (n, d) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : d; };
const url = opt("url");
if (!url) { console.error("usage: node e2e/shop-buy.mjs --url http://127.0.0.1:PORT/arena/"); process.exit(2); }
const out = opt("out", "e2e/.shots/shop-buy");
const shots = args.includes("--shots");
if (shots) mkdirSync(out, { recursive: true });
const SIZES = [
  { name: "800x450", viewport: { width: 800, height: 450 }, touch: true },
  { name: "390x844", viewport: { width: 390, height: 844 }, touch: true },
];
const LANGS = opt("lang", "en,ru").split(",");
const STILL = ["line", "bench", "offers", "reroll", "fight", "inspector"];
const errors = [];
const fail = (at, m) => errors.push(`${at}: ${m}`);

const browser = await launchChromium();
try {
  for (const v of SIZES) {
    for (const lang of LANGS) {
      const ctx = await browser.newContext({ viewport: v.viewport, isMobile: v.touch, hasTouch: v.touch, deviceScaleFactor: 2 });
      let page = await ctx.newPage();
      page.on("pageerror", (e) => fail(`${v.name} ${lang}`, `page error ${e.message}`));
      const open = async (scene) => {
        await page.goto("about:blank");
        await page.goto(`${url}#scene=${scene}&lang=${lang}`, { timeout: 120_000 });
        await page.waitForSelector('[data-testid="fight"]', { timeout: 60_000 });
        await page.waitForTimeout(900);
      };
      const gold = async () => Number((await page.getByTestId("gold").innerText()).replace(/\D/g, ""));
      const units = () => page.evaluate(() => document.querySelectorAll('[data-testid="line"] .cell.you, [data-testid="bench"] .cell.you').length);
      const boxes = () => page.evaluate((ids) => Object.fromEntries(ids.map((id) => { const r = document.querySelector(`[data-testid="${id}"]`)?.getBoundingClientRect(); return [id, r ? [r.x, r.y, r.width, r.height].map((n) => Math.round(n * 10) / 10) : null]; })), STILL);
      const lit = () => page.evaluate(() => [...document.querySelectorAll(".cell.lit")].map((e) => e.dataset.testid));
      const offerIds = () => page.evaluate(() => [...document.querySelectorAll('[data-testid^="offer-"]:not([data-testid^="offer-gap"])')].map((e) => e.dataset.testid));
      const shot = async (n) => { if (shots) await page.screenshot({ path: `${out}/${n}-${lang}-${v.name}.png` }); };
      const noScroll = async (at) => { if (await page.evaluate(() => document.documentElement.scrollWidth > innerWidth + 1 || document.documentElement.scrollHeight > innerHeight + 1)) fail(at, "the page scrolls"); };
      /** A tag's effective size (stage scale × font), its box inside the screen, and not over its cell's name. */
      const tagChecks = async (at) => {
        const bad = await page.evaluate((landscape) => {
          const k = parseFloat(document.querySelector(".stage").style.transform.replace(/[^0-9.]/g, "")) || 1;
          const out = [];
          for (const tag of document.querySelectorAll(".lit-tag")) {
            const cell = tag.closest(".cell");
            const r = tag.getBoundingClientRect();
            if (r.left < -0.5 || r.right > innerWidth + 0.5 || r.top < -0.5 || r.bottom > innerHeight + 0.5) out.push(`tag outside the screen (${Math.round(r.left)},${Math.round(r.top)},${Math.round(r.right)},${Math.round(r.bottom)})`);
            for (const e of [tag, ...tag.querySelectorAll("*")]) {
              if (!e.textContent.trim() || e.children.length) continue;
              const px = parseFloat(getComputedStyle(e).fontSize) * (e === tag ? 1 : 1) * k;
              if (landscape && px < 10.99) out.push(`tag text ${e.textContent.trim()} is ${px.toFixed(1)} px`);
            }
            const name = cell?.querySelector(".pc-nm");
            if (name && getComputedStyle(name).visibility !== "hidden") { const n = name.getBoundingClientRect(); if (!(r.bottom <= n.top + 0.5 || r.top >= n.bottom - 0.5 || r.right <= n.left || r.left >= n.right)) out.push("tag covers the name"); }
            if (tag.scrollWidth > tag.clientWidth + 1) out.push("tag text is cut");
          }
          return out;
        }, v.viewport.width > v.viewport.height);
        for (const b of bad) fail(at, b);
      };
      /** Taps the gold target of the offer in hand, and waits for the table to change. */
      const buyByTap = async () => { await page.getByTestId("buy").click(); await page.waitForFunction(() => !document.getElementById("app").classList.contains("busy"), null, { timeout: 15_000 }); await page.waitForTimeout(250); };

      // ---- 1. A room on the line: an offer in hand lights one gold well; a buy is one tap; the table never moves above the tray.
      {
        const at = `${v.name} ${lang} shop-r3`;
        await open("shop-r3");
        const idle = await boxes();
        const u0 = await units(), g0 = await gold();
        const free = await page.evaluate(() => document.querySelectorAll('[data-testid="line"] .cell.you').length);
        const o = (await offerIds())[0];
        const wasOwned = await page.getByTestId(o).evaluate((el) => el.classList.contains("owned"));
        await page.getByTestId(o).click();
        await page.waitForTimeout(300);
        if (await page.getByTestId("inspector").locator("button", { hasText: /Buy|Купить/ }).count()) fail(at, "the inspector has a Buy button");
        if ((await page.getByTestId("buy").count()) !== 1) fail(at, `expected one price tag, found ${await page.getByTestId("buy").count()}`);
        else {
          const cell = await page.getByTestId("buy").evaluate((el) => el.closest(".cell").dataset.testid);
          const benchN = await page.evaluate(() => document.querySelectorAll('[data-testid="bench"] .cell.you').length);
          const wants = (await page.getByTestId(o).evaluate((el) => el.classList.contains("owned"))) ? "a twin" : free < 5 ? `line-${free}-empty` : `bench-${benchN}-empty`;
          if (wants !== "a twin" && cell !== wants) fail(at, `the gold destination is ${cell}, not the next free well ${wants}`);
          const l = await lit();
          if (l.length !== 1) fail(at, `${l.length} slots are lit with an offer in hand (${l.join(", ")}): only the destination may be`);
          if (!/\d/.test(await page.getByTestId("buy").innerText())) fail(at, "the tag has no price");
        }
        await tagChecks(at);
        await noScroll(at);
        const now = await boxes();
        for (const k of STILL) if (!idle[k] || !now[k] || idle[k].some((x, i) => Math.abs(x - now[k][i]) > 0.6)) fail(at, `selecting the offer moved ${k}`);
        await shot("offer");
        // Tapping your own unit while an offer is in hand only reads it: nothing is bought, nothing is lit.
        // (a unit that is not the gold destination: a twin is the destination, and a tap on it buys)
        const mine = await page.evaluate(() => [...document.querySelectorAll('[data-testid="line"] .cell.you:not(.lit)')][0]?.dataset.testid);
        await page.getByTestId(mine).click();
        await page.waitForTimeout(300);
        if ((await units()) !== u0 || (await gold()) !== g0) fail(at, "a tap on your own unit bought or spent");
        if ((await lit()).length) fail(at, "a tap on your own unit armed destinations");
        if (!(await page.getByTestId("move").count())) fail(at, "no Move button for the unit in hand");
        // Move arms the teal destinations, each named; Move again (or Esc) lets go.
        await page.getByTestId("move").click();
        await page.waitForTimeout(250);
        const tags = await page.getByTestId("move-tag").allInnerTexts();
        if (!tags.length) fail(at, "Move lit no teal destination");
        if (tags.some((t) => !t.trim())) fail(at, "a teal destination is unnamed");
        await tagChecks(at);
        await shot("move");
        await page.keyboard.press("Escape");
        await page.waitForTimeout(200);
        if ((await lit()).length) fail(at, "Esc left the teal destinations lit");
        // The buy: tap the offer, then its gold well. Gold and the receipt show at once; the answer settles it.
        await page.getByTestId(o).click();
        await page.waitForTimeout(250);
        await page.getByTestId("buy").click();
        await page.waitForFunction((g) => Number(document.querySelector('[data-testid="gold"]').textContent.replace(/\D/g, "")) < g, g0, { timeout: 2000 }).catch(() => fail(at, "gold did not drop within 2 s of the tap"));
        const gift = () => page.getByTestId("gift-title").isVisible().catch(() => false);
        if (!(await page.getByTestId("receipt").count()) && !(await gift())) fail(at, "no receipt line after the buy");
        await page.waitForFunction(() => !document.getElementById("app").classList.contains("busy"), null, { timeout: 15_000 });
        await page.waitForTimeout(300);
        if ((await units()) !== u0 + (wasOwned ? 0 : 1)) fail(at, `a buy did not add a unit (${u0} → ${await units()})`);
        if (!(await page.getByTestId("receipt").count()) && !(await gift())) fail(at, "the receipt is gone after the server's answer");
        const after = await boxes();
        for (const k of STILL) if (!idle[k] || !after[k] || idle[k].some((x, i) => Math.abs(x - after[k][i]) > 0.6)) fail(at, `the buy moved ${k}`);
        if ((await page.getByTestId("offers").locator('[data-testid^="offer-gap"]').count()) < 1) fail(at, "the bought offer left no hole");
        await shot("bought");
        await noScroll(at);
      }

      // ---- 2. The same buy by a key (the destination's number) and by Enter. Keys exist only where there is a keyboard:
      // this runs in a non-touch context of the same size, and only where the keys are (landscape, a mouse).
      if (v.viewport.width > v.viewport.height) {
        const at = `${v.name} ${lang} keys`;
        const kctx = await browser.newContext({ viewport: v.viewport, deviceScaleFactor: 2 });
        const keep = page;
        page = await kctx.newPage();
        try {
          await open("shop-r3");
          const g0 = await gold(), u0 = await units();
          // → walks the offers; stop on one that can be bought (it lights its gold destination).
          for (let i = 0; i < 6; i++) {
            await page.keyboard.press("ArrowRight");
            await page.waitForTimeout(250);
            if ((await lit()).length) break;
          }
          if (!(await lit()).length) fail(at, "no offer of shop-r3 can be bought");
          else {
            const dest = await page.evaluate(() => { const m = /^(line|bench)-(\d+)/.exec(document.querySelector(".cell.lit").dataset.testid); return m[1] === "line" ? +m[2] + 1 : 6 + +m[2]; });
            // A key for a slot that is not the destination buys nothing.
            const wrong = dest === 1 ? 2 : 1;
            await page.keyboard.press(String(wrong));
            await page.waitForTimeout(250);
            if ((await gold()) !== g0) fail(at, `key ${wrong} (not the gold slot) spent gold`);
            await page.keyboard.press(String(dest));
            await page.waitForFunction((g) => Number(document.querySelector('[data-testid="gold"]').textContent.replace(/\D/g, "")) < g, g0, { timeout: 3000 }).catch(() => fail(at, `key ${dest} did not buy`));
            await page.waitForFunction(() => !document.getElementById("app").classList.contains("busy"), null, { timeout: 15_000 });
            await page.waitForTimeout(300);
            const g1 = await gold();
            // Enter takes the first legal target for the next offer in hand that can be bought.
            for (let i = 0; i < 6 && !(await page.getByTestId("buy").count()); i++) {
              await page.keyboard.press("ArrowRight");
              await page.waitForTimeout(250);
            }
            if ((await page.getByTestId("buy").count()) === 1) {
              await page.keyboard.press("Enter");
              await page.waitForFunction((g) => Number(document.querySelector('[data-testid="gold"]').textContent.replace(/\D/g, "")) < g, g1, { timeout: 3000 }).catch(() => fail(at, "Enter did not buy"));
              await page.waitForFunction(() => !document.getElementById("app").classList.contains("busy"), null, { timeout: 15_000 });
            }
            if ((await units()) < u0) fail(at, "the keys lost a unit");
          }
        } finally {
          page = keep;
          await kctx.close();
        }
      }
      // ---- 2b. The same buy by a drop (a real mouse drag, in a non-touch context of the same size): only the gold destination takes it.
      {
        const at = `${v.name} ${lang} drop`;
        const dctx = await browser.newContext({ viewport: v.viewport, deviceScaleFactor: 2 });
        const keep = page;
        page = await dctx.newPage();
        try {
          await open("shop-r3");
          const g0 = await gold(), u0 = await units();
          const free = await page.evaluate(() => document.querySelectorAll('[data-testid="line"] .cell.you').length);
          const o = (await offerIds())[0];
          const owned = await page.getByTestId(o).evaluate((el) => el.classList.contains("owned"));
          const poor = await page.getByTestId(o).evaluate((el) => el.classList.contains("poor"));
          if (!owned && !poor && free < 4) {
            const centre = async (id) => { const b = await page.getByTestId(id).boundingBox(); return [b.x + b.width / 2, b.y + b.height / 2]; };
            const drag = async (from, to) => {
              const [x0, y0] = await centre(from), [x1, y1] = await centre(to);
              await page.mouse.move(x0, y0);
              await page.mouse.down();
              await page.mouse.move(x0 + 12, y0 + 12, { steps: 3 });
              await page.mouse.move(x1, y1, { steps: 10 });
              await page.mouse.up();
              await page.waitForTimeout(700);
            };
            // A drop on a well that is not the destination buys nothing.
            await drag(o, `line-${free + 1}-empty`);
            if ((await gold()) !== g0) fail(at, "a drop on a slot that is not the gold destination bought");
            await drag(o, `line-${free}-empty`);
            await page.waitForFunction((g) => Number(document.querySelector('[data-testid="gold"]').textContent.replace(/\D/g, "")) < g, g0, { timeout: 3000 }).catch(() => fail(at, "a drop on the gold destination did not buy"));
            await page.waitForFunction(() => !document.getElementById("app").classList.contains("busy"), null, { timeout: 15_000 });
            await page.waitForTimeout(300);
            if ((await units()) !== u0 + 1) fail(at, `the drop added no unit (${u0} → ${await units()})`);
          }
        } finally {
          page = keep;
          await dctx.close();
        }
      }

      // ---- 3. A full line: the bench well is the destination; a full table has no gold target for a new unit.
      {
        const at = `${v.name} ${lang} bench`;
        await open("shop-bench-full");
        const o = (await offerIds())[0];
        await page.getByTestId(o).click();
        await page.waitForTimeout(300);
        const owned = await page.getByTestId(o).evaluate((el) => el.classList.contains("owned"));
        const n = await page.getByTestId("buy").count();
        if (!owned && n !== 0) fail(at, "a full line and bench show a gold target for a new unit");
        if (!owned && !(await page.getByTestId("buy-say").innerText()).trim()) fail(at, "no word on why it can't be bought");
        await tagChecks(at);
        await shot("full");
      }

      // ---- 4. A twin: "+1 copy" on the twin; its tag replaces the footer and never covers the name.
      {
        const at = `${v.name} ${lang} twin`;
        await open("shop-merge-hint");
        let seen = false;
        for (const id of await offerIds()) {
          await page.getByTestId(id).click();
          await page.waitForTimeout(300);
          if (await page.locator(".cell.lit-twin").count()) {
            seen = true;
            const txt = (await page.getByTestId("buy").innerText()).replace(/\s+/g, " ");
            if (!/\d/.test(txt)) fail(at, `the twin tag has no price: ${txt}`);
            await tagChecks(at);
            await shot("twin");
            break;
          }
        }
        if (!seen) fail(at, "no offer of this scene lit a twin");
      }

      // ---- 5. An awakening: the tag says the verb and the price on one line inside its frame (the numbers 2/13→4/18 stay
      // while they fit); the tray always reads the "from → to" line, whole, not under the scroll cue.
      {
        const at = `${v.name} ${lang} awaken`;
        await open("awaken");
        let seen = false;
        for (const id of await offerIds()) {
          await page.getByTestId(id).click();
          await page.waitForTimeout(400);
          const tag = page.getByTestId("buy");
          const txt = (await tag.innerText().catch(() => "")).replace(/\s+/g, " ");
          const kind = await page.evaluate(() => (document.querySelector(".cell.lit-twin .lit-tag") ? "twin" : ""));
          if (kind && !/^\+1|copy|копи/i.test(txt)) {
            seen = true;
            if (!/\d/.test(txt)) fail(at, `the awaken tag has no price: ${txt}`);
            const h = await tag.evaluate((el) => el.getBoundingClientRect().height);
            if (h > 26) fail(at, `the awaken tag is ${Math.round(h)} px tall: one line only`);
            const read = await page.evaluate(() => {
              const body = document.querySelector(".insp-body"), st = document.querySelector('[data-testid="sheet-stats"]');
              if (!body || !st) return null;
              const [b, r] = [body.getBoundingClientRect(), st.getBoundingClientRect()];
              return { text: st.innerText, inside: r.top >= b.top - 0.5 && r.bottom <= b.bottom + 0.5, cue: !!document.querySelector('[data-testid="insp-more"]') && getComputedStyle(document.querySelector('[data-testid="insp-more"]')).backgroundImage !== "none" && r.bottom > document.querySelector('[data-testid="insp-more"]').getBoundingClientRect().top + 0.5 };
            });
            if (!read || !/→/.test(read.text)) fail(at, "the tray has no from → to line for the awakening");
            else if (!read.inside) fail(at, "the from → to line is cut by the detail's box");
            else if (read.cue) fail(at, "the from → to line is under the scroll cue's fade");
            await tagChecks(at);
            await shot("awaken");
            break;
          }
        }
        if (!seen) fail(at, "no offer of this scene showed an awaken tag");
      }
      await ctx.close();
    }
  }
} finally {
  await browser.close();
}
if (errors.length) {
  console.error(errors.map((e) => `FAIL ${e}`).join("\n"));
  process.exit(1);
}
console.log("ok   shop buying: one gold destination, tap / key / Enter / drop, reading arms nothing, Move arms teal");
