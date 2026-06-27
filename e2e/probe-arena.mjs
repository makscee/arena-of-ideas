// #016 slice 3 — client login + remote ladder, against the REAL arena server
// (booted by run.mjs in MOCK_MODE with a throwaway DB; codes read back via
// /_mock/last-code, the void-mail _mock/last pattern). Three scenarios:
//
//  1. Logged out, ZERO network: a fresh context plays a full local round —
//     title, leaderboard, shop, fight, reload — and not one fetch/XHR (and
//     nothing to /v1 or /_mock) leaves the page. Logged-out play is local.
//  2. Two contexts share one ladder: A registers via the email→code→name
//     flow, plays a run to its end, and the server ACCEPTS it (re-derivation
//     passing is the probe's strongest claim — the client really fought the
//     served views). B registers separately and sees A's ghost in the round-1
//     pool count and on the leaderboard. Own-ghost exclusion is observable:
//     A's second run is offered ONE FEWER round-1 rival than B sees, because
//     A never fights A's own ghosts.
//  3. The login panel passes the shared occlusion sweep at phone width (the
//     probe-menu guard, extended to the new UI).

import {
  BASE,
  DESKTOP,
  PHONE,
  armGuard,
  check,
  finish,
  launch,
  sweepOcclusion,
} from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

// ---------- 1. logged out → zero network --------------------------------------

async function zeroNetworkScenario() {
  const ctx = await browser.newContext({ viewport: DESKTOP });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  const offenders = [];
  page.on("request", (req) => {
    const url = req.url();
    const type = req.resourceType();
    if (type === "fetch" || type === "xhr" || /\/v1\/|\/_mock\//.test(url)) offenders.push(`${type} ${url}`);
  });

  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  // #066 slice 6: account/login is back on the title (not in Settings).
  check((await page.locator("#title-login").textContent()) === "Login", "logged out: title offers Login");
  check(await page.locator("#title-id").isHidden(), "logged out: no identity strip");

  // Leaderboard (local backing), then a full local round: start → fight →
  // skip → continue — the whole loop must run without a server.
  await page.click("#title-leaderboard");
  await page.waitForSelector("#leaderboard-view:not([hidden])");
  check((await page.locator(".tower-floor").count()) > 0, "logged out: leaderboard tower shows local bootstrap floors");
  await page.click("#home-button");
  await page.click("#title-play");
  await page.waitForSelector("#run-new:not([hidden])");
  await page.fill("#run-seed", "3");
  await page.click("#run-start");
  await page.waitForSelector("#run-shop:not([hidden])");
  await page.locator("#run-shop-row .run-buy").first().click();
  await page.click("#run-fight");
  await page.waitForSelector("#run-battle:not([hidden])");
  await page.click("#run-skip");
  await page.waitForSelector("#run-continue:not([hidden])");
  await page.click("#run-continue");
  await page.waitForSelector("#run-shop:not([hidden]), #run-end:not([hidden])");

  // Reload mid-run: boot with no token must read nothing remote either.
  await page.reload({ waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  check(
    offenders.length === 0,
    "logged out: ZERO fetch/XHR and zero /v1 traffic across play + reload",
    offenders.slice(0, 5).join(", "),
  );
  await ctx.close();
}

// ---------- 2. two contexts, one shared ladder ---------------------------------

/** The OTP code the mock mailer last sent to `email` (run.mjs boots the
 * server in MOCK_MODE; vite proxies /_mock through to it). */
async function lastCode(email) {
  const res = await fetch(`${BASE}/_mock/last-code?email=${encodeURIComponent(email)}`);
  if (!res.ok) throw new Error(`no code for ${email}: HTTP ${res.status}`);
  return (await res.json()).code;
}

/** Register a fresh user through the real title-screen flow and land logged
 * in (the app reloads itself after the first-login name pick). #066 slice 6:
 * login is back on the title — no Settings detour. */
async function register(ctx, email, name) {
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  await page.click("#title-login");
  await page.waitForSelector("#login-email-row:not([hidden])");
  await page.fill("#login-email", email);
  await page.click("#login-submit");
  await page.waitForSelector("#login-code-row:not([hidden])");
  await page.fill("#login-code", await lastCode(email));
  await page.click("#login-submit");
  // First login: the name step follows the verify, no reload in between.
  await page.waitForSelector("#login-name-row:not([hidden])");
  await page.fill("#login-name", name);
  await page.click("#login-submit"); // → reload, boot reads the session
  await page.waitForSelector("#title-view:not([hidden])");
  // The identity strip is on the title now (#066 slice 6) — the picked display
  // name shows right there, no Settings open needed.
  await page.waitForSelector("#title-id:not([hidden])");
  check(
    (await page.locator("#title-name").textContent()) === name,
    `${name}: logged-in title shows the picked display name`,
  );
  return page;
}

/** Play the active run to its end through the UI. The post-#075 tower has two
 * forward moves: a CLIMB (#run-fight) up a floor, and — at the champion's floor,
 * where a climb would overshoot — a terminal CHALLENGE (#run-challenge, a
 * two-tap confirm) for the crown. So: buy until the line can field, then climb
 * while #run-fight is enabled, and challenge the champion when it disables at
 * the top. Either fight skips its replay and continues; the challenge is
 * terminal and lands straight on the end screen. */
async function playRunToEnd(page, tag) {
  let runId = null;
  for (let i = 0; i < 30; i++) {
    await page.waitForSelector("#run-shop:not([hidden]), #run-end:not([hidden])");
    if (await page.locator("#run-end").isVisible()) break;
    if (runId === null) runId = (await page.locator("#run-head .run-id").textContent()).trim();
    const lineEmpty = (await page.locator("#run-line [data-move]").count()) === 0;
    if (lineEmpty) await page.locator("#run-shop-row .run-buy").first().click();
    if (!(await page.locator("#run-fight").isDisabled())) {
      // A floor below the champion: climb.
      await page.click("#run-fight");
      await page.waitForSelector("#run-battle:not([hidden])");
      await page.click("#run-skip");
      await page.waitForSelector("#run-continue:not([hidden])");
      await page.click("#run-continue");
    } else {
      // The climb is disabled with a fielded line — the champion's floor (#075):
      // challenge the champion for the crown. Two taps: arm, then fire.
      await page.click("#run-challenge"); // arm
      await page.click("#run-challenge"); // fire → battle, then the end screen
      await page.waitForSelector("#run-battle:not([hidden]), #run-end:not([hidden])");
      if (await page.locator("#run-battle").isVisible()) {
        await page.click("#run-skip");
        await page.waitForSelector("#run-continue:not([hidden])");
        await page.click("#run-continue");
      }
    }
  }
  check(await page.locator("#run-end").isVisible(), `${tag}: the run reached its end screen`);
  return runId;
}

/** The round-1 rivals count as the shop's next-fight line reports it. */
async function rivalsAtRoundOne(page, expected, tag) {
  await page.waitForSelector("#run-shop:not([hidden])");
  // The line settles once the served (own-ghost-excluded) view lands.
  const want = `— ${expected} waiting`;
  const ok = await page
    .waitForFunction(
      (text) => document.querySelector("#run-next")?.textContent?.includes(text),
      want,
      { timeout: 15_000 },
    )
    .then(() => true)
    .catch(() => false);
  const got = await page.locator("#run-next").textContent();
  check(ok, `${tag} round-1 rivals read "${want}"`, `got "${got}"`);
}

async function sharedLadderScenario() {
  // The bootstrap baseline, read once before anyone plays: round 1's public
  // pool size. Everything below is asserted relative to it.
  const k0 = (await (await fetch(`${BASE}/v1/ladder/pool/1`)).json()).pool.length;
  check(k0 > 0, "bootstrap round-1 pool is non-empty before any runs");

  // --- A registers and plays a full run that the server ACCEPTS ---
  const ctxA = await browser.newContext({ viewport: DESKTOP });
  const pageA = await register(ctxA, "alice@e2e.test", "Alice");
  await pageA.click("#title-play");
  await pageA.waitForSelector("#run-new:not([hidden])");
  await pageA.fill("#run-seed", "11");
  await pageA.click("#run-start");
  const runIdA = await playRunToEnd(pageA, "Alice");
  check(runIdA !== null && runIdA.startsWith("web-") && runIdA.length > 20, "Alice's runId is minted globally unique", runIdA);

  // The submit verdict: the server re-derived the run and took it. A refusal
  // here means the client fought views the server never served — the slice's
  // core contract — so the probe demands the accepted wording.
  const verdictOk = await pageA
    .waitForFunction(() => {
      const t = document.querySelector("#run-end-status")?.textContent ?? "";
      return t.includes("shared ladder") && !t.includes("submitting");
    }, undefined, { timeout: 15_000 })
    .then(() => true)
    .catch(() => false);
  const verdict = await pageA.locator("#run-end-status").textContent();
  check(verdictOk && /joined the shared ladder|crown is yours/.test(verdict), "Alice's run was accepted onto the shared ladder", verdict);
  check(!verdict.includes("refused"), "Alice's submission was not refused", verdict);

  // The server's public pool grew by exactly Alice's round-1 ghost.
  const k1 = (await (await fetch(`${BASE}/v1/ladder/pool/1`)).json()).pool.length;
  check(k1 === k0 + 1, "server round-1 pool gained Alice's re-derived ghost", `${k0} → ${k1}`);

  // --- own-ghost exclusion, through the UI: A's SECOND run sees k0 rivals ---
  await pageA.click("#run-new-run"); // back to title
  await pageA.waitForSelector("#title-view:not([hidden])");
  await pageA.click("#title-play");
  await pageA.waitForSelector("#run-new:not([hidden])");
  await pageA.fill("#run-seed", "12");
  await pageA.click("#run-start");
  await rivalsAtRoundOne(pageA, k0, "Alice (own ghost excluded):");

  // --- B registers separately: sees k0 + 1 rivals and Alice on the board ---
  const ctxB = await browser.newContext({ viewport: DESKTOP });
  const pageB = await register(ctxB, "bob@e2e.test", "Bob");
  await pageB.click("#title-play");
  await pageB.waitForSelector("#run-new:not([hidden])");
  await pageB.fill("#run-seed", "22");
  await pageB.click("#run-start");
  await rivalsAtRoundOne(pageB, k0 + 1, "Bob (sees Alice's ghost):");

  // B's leaderboard: round 1 lists a ghost labelled with Alice's runId prefix.
  await pageB.click("#home-button");
  await pageB.click("#title-leaderboard");
  await pageB.waitForSelector("#leaderboard-view:not([hidden])");
  const labelA = `${runIdA.slice(0, 12)}…`;
  const seen = await pageB
    .waitForFunction(
      (label) => [...document.querySelectorAll(".tower-handle")].some((el) => el.textContent.includes(label)),
      labelA,
      { timeout: 15_000 },
    )
    .then(() => true)
    .catch(() => false);
  check(seen, `Bob's leaderboard tower lists Alice's ghost (${labelA})`);

  // When Alice's crown really landed, the shared leaderboard names her as the
  // holder (the display-name → champion `holder` path, end to end) — the
  // champion rung's handle reads her name.
  if (verdict.includes("crown is yours")) {
    const champLine = await pageB.locator("#leaderboard-body .tower-floor.is-champ .tower-handle").textContent();
    check(champLine.includes("Alice"), "Bob's leaderboard tower names Alice as the crown holder", champLine);
  }

  await ctxA.close();
  await ctxB.close();
}

// ---------- 3. login panel occlusion (the probe-menu guard, extended) ---------

async function loginOcclusionScenario() {
  const ctx = await browser.newContext({ viewport: PHONE, hasTouch: true });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  await page.click("#title-login"); // #066 slice 6: login is on the title
  await page.waitForSelector("#login-email-row:not([hidden])");
  for (const target of ["#login-email", "#login-submit", "#login-cancel"]) {
    const { stolen, total, thieves } = await sweepOcclusion(page, target);
    check(
      stolen === 0 && total > 0,
      `375px login: ${target} not occluded`,
      `${stolen}/${total} surface points stolen${stolen > 0 ? ` by ${thieves.join(", ")}` : ""}`,
    );
  }
  await ctx.close();
}

/** #066 slice 6: the logged-in identity strip + logout, both on the title.
 * register() already proves the account NAME shows on the title after login;
 * here we prove Logout works — it swaps the identity strip back to Login and
 * clears the session so a reload boots logged out. */
async function titleLogoutScenario() {
  const ctx = await browser.newContext();
  const page = await register(ctx, "carol@e2e.test", "Carol");
  // Logged in: the identity strip (name + log out) shows on the title, Login hidden.
  check(await page.locator("#title-id").isVisible(), "logged in: identity strip on the title");
  check(await page.locator("#title-login").isHidden(), "logged in: Login entry hidden on the title");
  check((await page.locator("#title-name").textContent()) === "Carol", "logged in: title shows the account name");
  await page.click("#title-logout");
  await page.waitForSelector("#title-login:not([hidden])");
  check(await page.locator("#title-login").isVisible(), "after logout: Login is back on the title");
  check(await page.locator("#title-id").isHidden(), "after logout: identity strip gone");
  check(
    (await page.evaluate(() => localStorage.getItem("aoi.session.v1"))) === null,
    "after logout: the session token is cleared",
  );
  await ctx.close();
}

await zeroNetworkScenario();
await sharedLadderScenario();
await loginOcclusionScenario();
await titleLogoutScenario();

await browser.close();
disarm();
finish("probe-arena");
