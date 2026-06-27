// PRD #080 slice 1 — vendored fonts, no runtime CDN. Pins, against the LIVE app:
//  1. No request to fonts.googleapis.com / fonts.gstatic.com / any external host
//     at runtime — the 3 families are self-hosted under /fonts/*.woff2.
//  2. The title (h1.title-name) computes to "Chakra Petch" AND the face actually
//     loaded (document.fonts), so it's the vendored woff2, not a system fallback.
//  3. The woff2 are fetched same-origin (a /fonts/*.woff2 request is seen).
// Screens captured at desktop + 375px for the LOOK (bg #07080c + fonts flipped).

import { mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { BASE, DESKTOP, PHONE, armGuard, check, finish, launch } from "./lib.mjs";

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

for (const [viewport, tag] of [
  [DESKTOP, "desktop"],
  [PHONE, "375px"],
]) {
  const ctx = await browser.newContext({ viewport, hasTouch: viewport.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);

  const external = [];
  const fontReqs = [];
  page.on("request", (req) => {
    const url = req.url();
    if (isExternal(url)) external.push(url);
    if (/\.woff2(\?|$)/.test(url)) fontReqs.push(url);
  });

  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  // Give the title face a beat to fetch + load.
  await page.evaluate(() => document.fonts.ready);
  await page.waitForTimeout(300);

  check(
    external.length === 0,
    `${tag} zero external requests at runtime (no font CDN)`,
    external.slice(0, 4).join(", "),
  );

  const titleFamily = await page.$eval(".title-name", (el) => getComputedStyle(el).fontFamily);
  check(
    /chakra petch/i.test(titleFamily),
    `${tag} the title computes to Chakra Petch`,
    titleFamily,
  );

  const chakraLoaded = await page.evaluate(() =>
    document.fonts.check('700 16px "Chakra Petch"'),
  );
  check(chakraLoaded, `${tag} the Chakra Petch face actually loaded (vendored woff2)`);

  check(
    fontReqs.some((u) => /\/fonts\//.test(u) && !isExternal(u)),
    `${tag} a vendored /fonts/*.woff2 was fetched same-origin`,
    fontReqs.slice(0, 3).join(", "),
  );

  const bg = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
  check(bg === "rgb(7, 8, 12)", `${tag} body bg is the B·Arena #07080c`, bg);

  await page.screenshot({ path: join(outDir, `fonts-${tag}-title.png`), fullPage: false });
  console.log(`shot fonts-${tag}-title`);
  await ctx.close();
}

await browser.close();
disarm();
finish("probe-fonts");
