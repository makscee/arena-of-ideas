// PRD #080 slice 4 — the B·Arena card proven on the shop surface. Pins, against
// the LIVE app, that the shop offers (FULL) and the team "Your line" (COMPACT)
// render the new family-coloured card at desktop + 375px:
//  1. Both variants render: an offer is `.unit-b.is-full` with the art area; a
//     line card is `.unit-b.is-compact` with the header sigil and no art area.
//  2. The card contract anchors survive (.uname / .unums .hp / .unums .pwr /
//     .chips) so the inspector + probes still key off them.
//  3. Family colour + chamfer: a polygon clip-path, a non-transparent family
//     border, HP cyan (#25e6d4) and PWR red (#ff5470).
//  4. No horizontal overflow at either width; the primary Buy action wears a
//     glow; touch targets are ≥44px at phone width.
//  5. Zero external (font CDN) requests at runtime.
// Screenshots → e2e/.shots/ for the LOOK against Arena.dc.html.

import { mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { BASE, DESKTOP, PHONE, armGuard, box, check, finish, launch, openRun, targetsRun } from "./lib.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const outDir = process.env.SHOTS_DIR ?? join(here, ".shots");
mkdirSync(outDir, { recursive: true });

const disarm = armGuard();
const browser = await launch();
const baseHost = new URL(BASE).host;
const isExternal = (url) => {
  try {
    const u = new URL(url);
    return u.protocol.startsWith("http") && u.host !== baseHost;
  } catch {
    return false;
  }
};

const OFFER = '#run-shop-row [data-offer="0"]';
const LINE = '#run-line [data-line="0"]';
const noHorizontalOverflow = (page) =>
  page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1);
const has = (page, sel) => page.$(sel).then((h) => h !== null);
const css = (page, sel, prop) => page.$eval(sel, (el, p) => getComputedStyle(el)[p], prop);

for (const [viewport, tag] of [
  [DESKTOP, "desktop"],
  [PHONE, "375px"],
]) {
  const external = [];
  const { ctx, page } = await (async () => {
    const c = await browser.newContext({ viewport, hasTouch: viewport.width < 700 });
    const p = await c.newPage();
    p.on("request", (req) => {
      if (isExternal(req.url())) external.push(req.url());
    });
    p.setDefaultTimeout(15_000);
    await p.addInitScript(([k, v]) => localStorage.setItem(k, v), ["aoi.run.v1", targetsRun()]);
    await p.goto(BASE, { waitUntil: "domcontentloaded" });
    await p.waitForSelector("#title-view:not([hidden])");
    await p.click("#title-play");
    await p.waitForSelector("#run-shop:not([hidden])");
    return { ctx: c, page: p };
  })();

  // ---- FULL offer card ----
  check(await has(page, `${OFFER}.unit-b.is-full`), `${tag} offer renders the FULL B·Arena card`);
  check(await has(page, `${OFFER} .ub-art`), `${tag} full card carries the 84px art area`);
  check(await has(page, `${OFFER} .ub-art .ub-sigil`), `${tag} full card draws the family sigil`);
  for (const anchor of [".uname", ".unums .hp", ".unums .pwr", ".chips"]) {
    check(await has(page, `${OFFER} ${anchor}`), `${tag} full card keeps the contract anchor ${anchor}`);
  }

  // ---- COMPACT line card ----
  check(await has(page, `${LINE}.unit-b.is-compact`), `${tag} line renders the COMPACT B·Arena card`);
  check(!(await has(page, `${LINE} .ub-art`)), `${tag} compact card has NO art area`);
  check(await has(page, `${LINE} .ub-mini .ub-sigil`), `${tag} compact card rides the header sigil`);
  for (const anchor of [".uname", ".unums .hp", ".unums .pwr", ".chips"]) {
    check(await has(page, `${LINE} ${anchor}`), `${tag} compact card keeps the contract anchor ${anchor}`);
  }

  // ---- family colour + chamfer + HP/PWR ----
  const clip = await css(page, OFFER, "clipPath");
  check(/polygon\(/.test(clip), `${tag} chamfered clip-path on the card`, clip);
  const border = await css(page, OFFER, "borderTopColor");
  check(border !== "rgba(0, 0, 0, 0)" && border !== "rgb(0, 0, 0)", `${tag} family border colour set`, border);
  check(
    (await css(page, `${OFFER} .unums .hp`, "color")) === "rgb(37, 230, 212)",
    `${tag} HP numeral is cyan #25e6d4`,
    await css(page, `${OFFER} .unums .hp`, "color"),
  );
  check(
    (await css(page, `${OFFER} .unums .pwr`, "color")) === "rgb(255, 84, 112)",
    `${tag} PWR numeral is red #ff5470`,
    await css(page, `${OFFER} .unums .pwr`, "color"),
  );
  const title = await css(page, `${OFFER} .uname`, "fontFamily");
  check(/chakra petch/i.test(title), `${tag} card name is Chakra Petch`, title);
  const numFont = await css(page, `${OFFER} .unums .hp`, "fontFamily");
  check(/ibm plex mono/i.test(numFont), `${tag} HP·PWR numerals are IBM Plex Mono`, numFont);

  // ---- primary action glow ----
  const buyShadow = await css(page, `${OFFER} .run-buy`, "boxShadow");
  check(buyShadow !== "none", `${tag} the Buy action wears a neon glow`, buyShadow);

  // ---- layout: no overflow; widths ~212px ----
  check(await noHorizontalOverflow(page), `${tag} shop has no horizontal overflow`);
  const offerW = (await box(page, OFFER)).width;
  const lineW = (await box(page, LINE)).width;
  check(Math.abs(offerW - 212) <= 2, `${tag} full card is 212px wide`, `${offerW.toFixed(1)}`);
  check(Math.abs(lineW - 212) <= 2, `${tag} compact card is 212px wide`, `${lineW.toFixed(1)}`);

  // ---- external requests ----
  check(external.length === 0, `${tag} zero external requests (no font CDN)`, external.slice(0, 3).join(", "));

  // ---- touch targets at phone ----
  if (viewport === PHONE) {
    const buy = await box(page, `${OFFER} .run-buy`);
    check(buy.height >= 44, `${tag} Buy tap is ≥44px tall`, `${buy.height.toFixed(1)}`);
    const card = await box(page, OFFER);
    check(card.height >= 44 && card.width >= 44, `${tag} card inspect tap is ≥44px`, `${card.width}x${card.height}`);
  }

  await page.screenshot({ path: join(outDir, `card-${tag}-shop.png`), fullPage: false });
  console.log(`shot card-${tag}-shop`);
  await ctx.close();
}

await browser.close();
disarm();
finish("probe-card");
