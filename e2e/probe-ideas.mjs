// PRD #076 slice 3, evolved to directional votes in #082 — the ideas screen,
// against the LIVE app at desktop and 375×667. The acceptance, end to end:
//   1. Reachable from a title button; renders the ranked list.
//   2. Logged out: the public table is READABLE, but submit/vote route to login
//      (a nudge, never a silent error).
//   3. Logged in: submit free text → it appears; UP-vote → its rank rises and
//      the up arrow reads voted; switching to DOWN lowers it and the vote FLIPS
//      (switch-only — there is no remove, the vote never returns to neutral).
//   4. Geometry: usable at 375px — list, submit box, vote arrows all fit, are
//      visible at real (non-zero) size, ≥44px taps, nothing overflows sideways.
//   5. Votes only RANK — nothing is gated/admitted by voting.
//
// The server is fresh per `npm run e2e` (temp SQLite, MOCK_MODE), so each
// viewport pass uses a DISTINCT player email — the one-vote-per-player rule is
// exercised against a real second account, and a pass never depends on another.

import { BASE, DESKTOP, PHONE, armGuard, box, check, finish, launch, loginViaUi } from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();

const noHorizontalOverflow = (page) =>
  page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1);

// The login-start rate limiter keys on X-Forwarded-For (server/src/app.ts) —
// 5 starts per IP per 10 min, and Playwright sends no XFF, so EVERY probe's
// logins would otherwise share one `ip:unknown` budget. This probe logs in
// several times (a scenario + the funnel, per viewport); give each context its
// OWN forwarded IP so each gets a fresh 5-start budget, isolated from the other
// probes and from this probe's other passes.
let ipCounter = 0;
function freshContext(viewport) {
  ipCounter += 1;
  return browser.newContext({
    viewport,
    hasTouch: viewport.width < 700,
    extraHTTPHeaders: { "x-forwarded-for": `10.0.0.${ipCounter}` },
  });
}

/** Reveal the (collapsed) submit box — the footer "＋ Submit an idea" CTA opens
 * it. Logged in it shows the form; call once after each login (a login reload
 * re-collapses it). Logged out the CTA routes to login instead. */
async function revealSubmit(page) {
  await page.click("#ideas-reveal");
  await page.waitForSelector("#ideas-form:not([hidden])");
}

/** A fresh logged-OUT page, landed on the ideas screen. No stored run. */
async function openIdeas(viewport) {
  const ctx = await freshContext(viewport);
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.addInitScript(() => localStorage.removeItem("aoi.run.v1"));
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  return { ctx, page };
}

/** Seed ONE idea onto the shared table as a DISTINCT player, then close.
 * One idea per player per day (#082 slice 4) caps each account at a single
 * submission per run, so a scenario that needs a second idea on the table (to
 * test vote-driven ranking) seeds it from its own account here. */
async function seedIdea(viewport, email, displayName, text) {
  const ctx = await freshContext(viewport);
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.addInitScript(() => localStorage.removeItem("aoi.run.v1"));
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  await loginViaUi(page, email, displayName);
  await page.click("#title-ideas");
  await page.waitForSelector("#ideas-view:not([hidden])");
  await revealSubmit(page);
  await page.fill("#ideas-text", text);
  await page.click("#ideas-submit");
  await page.waitForFunction(
    (t) => [...document.querySelectorAll("#ideas-list .ideas-text")].some((e) => e.textContent === t),
    text,
  );
  await ctx.close();
}

const rowTexts = (page) =>
  page.$$eval("#ideas-list .ideas-row .ideas-text", (els) => els.map((e) => e.textContent));

const rowVoteCount = (page, text) =>
  page.evaluate((t) => {
    const rows = [...document.querySelectorAll("#ideas-list .ideas-row")];
    const row = rows.find((r) => r.querySelector(".ideas-text")?.textContent === t);
    return row ? Number(row.querySelector(".ideas-vote-count").textContent) : null;
  }, text);

// Which direction the player currently holds on a row ("up"/"down"/null) — the
// arrow whose aria-pressed is true. Switch-only: at most one arrow is pressed.
const rowVoteDir = (page, text) =>
  page.evaluate((t) => {
    const rows = [...document.querySelectorAll("#ideas-list .ideas-row")];
    const row = rows.find((r) => r.querySelector(".ideas-text")?.textContent === t);
    if (!row) return null;
    if (row.querySelector(".ideas-vote-up").getAttribute("aria-pressed") === "true") return "up";
    if (row.querySelector(".ideas-vote-down").getAttribute("aria-pressed") === "true") return "down";
    return null;
  }, text);

// Click a row's up/down arrow.
const clickVote = (page, text, dir) =>
  page.evaluate(
    ({ t, d }) => {
      const rows = [...document.querySelectorAll("#ideas-list .ideas-row")];
      rows.find((r) => r.querySelector(".ideas-text")?.textContent === t).querySelector(`.ideas-vote-${d}`).click();
    },
    { t: text, d: dir },
  );

// Wait until a row's net-score count reaches `want`.
const waitForCount = (page, text, want) =>
  page.waitForFunction(
    ({ t, w }) => {
      const rows = [...document.querySelectorAll("#ideas-list .ideas-row")];
      const row = rows.find((r) => r.querySelector(".ideas-text")?.textContent === t);
      return row && Number(row.querySelector(".ideas-vote-count").textContent) === w;
    },
    { t: text, w: want },
  );

async function scenario(viewport, tag, email) {
  const mine = `${tag} idea — make poison stack faster`;
  const other = `${tag} idea — add a draft phase`;
  // Seed `other` from a SEPARATE account first (one idea per player per day caps
  // this scenario's player at the single `mine` submission). `other` lands with
  // an earlier seq, so at equal votes it ranks above `mine` until `mine` is voted.
  await seedIdea(viewport, `seed-${tag.replace(/[^a-z0-9]/gi, "")}@probe.test`, `Seed ${tag}`, other);

  const { ctx, page } = await openIdeas(viewport);

  // ---- 1. reachable from the title, renders the screen --------------------
  await page.click("#title-ideas");
  await page.waitForSelector("#ideas-view:not([hidden])");
  check(await page.locator(".ideas-title").isVisible(), `${tag} the ideas screen carries its own title`);
  check(
    (await page.evaluate(() => localStorage.getItem("aoi.run.v1"))) === null,
    `${tag} opening ideas starts no run`,
  );

  // ---- 2. logged-out: table is READABLE, submit/vote route to login -------
  check(await page.locator("#ideas-login-note").isVisible(), `${tag} logged-out shows the read-only login note`);
  // refresh() fills the list async — wait for it to settle (rows, or the
  // empty-state paragraph on a fresh server) before asserting the table reads.
  await page.waitForFunction(() => document.querySelector("#ideas-list").children.length > 0);
  check(await page.locator("#ideas-list").isVisible(), `${tag} logged-out player can SEE the table`);
  // A logged-out submit attempt must route to the login panel, not error. The
  // footer "＋ Submit an idea" CTA is the door while logged out — tap it, expect
  // the title's login panel to open (a nudge, never a silent error).
  await page.click("#ideas-reveal");
  await page.waitForSelector("#login-panel:not([hidden])", { timeout: 5000 });
  check(await page.locator("#login-panel").isVisible(), `${tag} logged-out submit routes to login (a nudge, not a silent error)`);

  // ---- log in (real UI flow), back to the ideas screen --------------------
  await loginViaUi(page, email, `Player ${tag}`);
  await page.click("#title-ideas");
  await page.waitForSelector("#ideas-view:not([hidden])");
  check(await page.locator("#ideas-login-note").isHidden(), `${tag} logged-in hides the login note`);
  await revealSubmit(page); // open the submit box for the rest of the scenario
  check(await page.locator("#ideas-text").isEnabled(), `${tag} logged-in enables the submit box`);

  // ---- 3a. submit free text → it appears in the table ---------------------
  // The seeded `other` reads on the table — wait for the post-login refresh to
  // populate the list before asserting. Submit `mine` — the player's ONE idea.
  await page.waitForFunction(
    (t) => [...document.querySelectorAll("#ideas-list .ideas-text")].some((e) => e.textContent === t),
    other,
  );
  check((await rowTexts(page)).includes(other), `${tag} the seeded idea reads on the table`, other);
  await page.fill("#ideas-text", mine);
  await page.click("#ideas-submit");
  await page.waitForFunction(
    (t) => [...document.querySelectorAll("#ideas-list .ideas-text")].some((e) => e.textContent === t),
    mine,
  );
  let texts = await rowTexts(page);
  check(texts.includes(mine), `${tag} a submitted idea appears in the table`, mine);
  // Both at 0 votes: `other` (earlier seq) ranks above `mine`.
  check(
    texts.indexOf(other) < texts.indexOf(mine),
    `${tag} with equal votes the earlier idea ranks higher`,
    `${texts.indexOf(other)} < ${texts.indexOf(mine)}`,
  );

  // ---- one idea per player per day: a second submit is refused on the UI ---
  const extra = `${tag} idea — a second one, same day`;
  await page.fill("#ideas-text", extra);
  await page.click("#ideas-submit");
  await page.waitForFunction(
    () => /one idea per day/i.test(document.querySelector("#ideas-status")?.textContent ?? ""),
  );
  check(
    /one idea per day/i.test((await page.locator("#ideas-status").textContent()) ?? ""),
    `${tag} a same-day second submit is refused on the status line`,
    (await page.locator("#ideas-status").textContent()) ?? "",
  );
  check(!(await rowTexts(page)).includes(extra), `${tag} the refused second idea is NOT added to the table`);

  // ---- geometry at this viewport (real visibility + sane sizes) -----------
  check(await noHorizontalOverflow(page), `${tag} no horizontal overflow`);
  const submitBox = await box(page, "#ideas-submit");
  check(submitBox.height >= 44, `${tag} submit button is a ≥44px tap`, `${Math.round(submitBox.height)}px`);
  const inputBox = await box(page, "#ideas-text");
  check(inputBox.height >= 44 && inputBox.width > 100, `${tag} submit input is a real, sized field`, `${Math.round(inputBox.width)}×${Math.round(inputBox.height)}`);
  const firstVote = page.locator("#ideas-list .ideas-vote-arrow").first();
  check(await firstVote.isVisible(), `${tag} vote arrows are visible`);
  const voteBox = await box(page, "#ideas-list .ideas-vote-arrow >> nth=0");
  check(
    voteBox.height >= 44 && voteBox.width >= 44,
    `${tag} vote arrow is a ≥44px tap`,
    `${Math.round(voteBox.width)}×${Math.round(voteBox.height)}`,
  );
  // The vote arrow must sit inside the viewport, not clipped off the right edge.
  check(
    voteBox.x >= 0 && voteBox.x + voteBox.width <= viewport.width + 1,
    `${tag} vote arrow sits within the viewport`,
    `x=${Math.round(voteBox.x)} right=${Math.round(voteBox.x + voteBox.width)} vw=${viewport.width}`,
  );
  // Both arrows are present per row — the switch-only directional pair.
  check(
    (await page.locator("#ideas-list .ideas-row").first().locator(".ideas-vote-up").count()) === 1 &&
      (await page.locator("#ideas-list .ideas-row").first().locator(".ideas-vote-down").count()) === 1,
    `${tag} each row carries an up AND a down arrow`,
  );

  // ---- B·Arena redress (#082 s5 / #083 s5): @author + status pill per row ----
  const firstRow = page.locator("#ideas-list .ideas-row").first();
  check(
    (await firstRow.locator(".ideas-author").count()) === 1 &&
      /^@/.test((await firstRow.locator(".ideas-author").textContent()) ?? ""),
    `${tag} each row shows its @author`,
    (await firstRow.locator(".ideas-author").textContent()) ?? "",
  );
  // Every fresh idea is on-table → its pill reads "voting" (the display vocabulary
  // over the lifecycle status; nothing here computes eligibility).
  const pillText = (await firstRow.locator(".ideas-pill").textContent()) ?? "";
  check(
    (await firstRow.locator(".ideas-pill").count()) === 1 && pillText.trim() === "voting",
    `${tag} each row shows its lifecycle status pill (fresh = voting)`,
    pillText,
  );

  // ---- 3b. UP-vote → rank MOVES and the up arrow reflects voted -----------
  check((await rowVoteCount(page, mine)) === 0, `${tag} my idea starts at score 0`);
  check((await rowVoteDir(page, mine)) === null, `${tag} my idea starts not-voted`);
  // The vote-currency counter renders, at 0 before any cast (submitting is not voting).
  check(await page.locator("#ideas-currency").isVisible(), `${tag} the vote-currency counter renders when logged in`);
  check(
    /voted on 0 idea/.test((await page.locator("#ideas-currency").textContent()) ?? ""),
    `${tag} currency starts at 0 (submitting is not voting)`,
    (await page.locator("#ideas-currency").textContent()) ?? "",
  );
  await clickVote(page, mine, "up");
  await waitForCount(page, mine, 1);
  check((await rowVoteCount(page, mine)) === 1, `${tag} an up-vote raises the score to 1`);
  check((await rowVoteDir(page, mine)) === "up", `${tag} the up arrow now reads voted`);
  // The currency ticked up to 1 — voting on an idea accrues the footprint.
  await page.waitForFunction(() => /voted on 1 idea\b/.test(document.querySelector("#ideas-currency")?.textContent ?? ""));
  check(
    /voted on 1 idea\b/.test((await page.locator("#ideas-currency").textContent()) ?? ""),
    `${tag} currency updates to 1 after a cast`,
    (await page.locator("#ideas-currency").textContent()) ?? "",
  );
  texts = await rowTexts(page);
  check(
    texts.indexOf(mine) < texts.indexOf(other),
    `${tag} the up-vote moved my idea ABOVE the neutral one (rank moved)`,
    `${mine}@${texts.indexOf(mine)} vs ${other}@${texts.indexOf(other)}`,
  );

  // ---- switch, NEVER remove: flipping to down lowers it, the vote persists -
  await clickVote(page, mine, "down");
  await waitForCount(page, mine, -1);
  check((await rowVoteCount(page, mine)) === -1, `${tag} switching to down lowers the score to -1`);
  check((await rowVoteDir(page, mine)) === "down", `${tag} the vote FLIPPED to down (switch, not remove)`);
  // Switching direction does NOT change the currency — it's still one idea voted.
  check(
    /voted on 1 idea\b/.test((await page.locator("#ideas-currency").textContent()) ?? ""),
    `${tag} currency unchanged at 1 after a flip (non-farmable)`,
    (await page.locator("#ideas-currency").textContent()) ?? "",
  );
  // The vote is still held — there is no affordance that returns it to neutral.
  texts = await rowTexts(page);
  check(
    texts.indexOf(mine) > texts.indexOf(other),
    `${tag} the down-vote moved my idea BELOW the neutral one`,
    `${mine}@${texts.indexOf(mine)} vs ${other}@${texts.indexOf(other)}`,
  );
  // Re-tapping the SAME (down) arrow is a no-op — still down, never cleared.
  await clickVote(page, mine, "down");
  await waitForCount(page, mine, -1);
  check((await rowVoteDir(page, mine)) === "down", `${tag} re-tapping the held arrow is a no-op — the vote never clears`);

  // ---- 5. votes only RANK — nothing is gated/admitted ---------------------
  // Flip back up and confirm BOTH ideas are STILL in the table (not removed /
  // not "rejected") — voting reorders, it never admits or drops an idea.
  await clickVote(page, mine, "up");
  await waitForCount(page, mine, 1);
  texts = await rowTexts(page);
  check(
    texts.includes(other) && texts.includes(mine),
    `${tag} both ideas stay in the table — voting only reorders, never gates`,
    `${texts.length} rows`,
  );

  await ctx.close();
}

// ---- the first-time-contributor funnel (Cass round-2 regression) ----------
// Reach login the OTHER way: via #home-button before any authed submit (the
// nudge path scenario() uses skips the home round-trip). Then submit ideas in a
// TIGHT sequence — fill+click with no settle wait between, exactly as a real
// user mashing the form. The defect this guards: a phantom EMPTY submit fired
// between two real ones (an async re-entrancy race — onSubmit cleared the input
// inside its await window, so a racing second submit read ""), surfacing as a
// spurious "Type an idea before sending it." and a swallowed idea. After the
// fix NO empty submit fires and no spurious error shows. With one-per-day
// (#082 slice 4) only the FIRST of the mashed ideas can land — the rest are
// refused by the day limit (never a phantom-empty "Type an idea" error).
async function funnelScenario(viewport, tag, email) {
  const ctx = await freshContext(viewport);
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.addInitScript(() => localStorage.removeItem("aoi.run.v1"));
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");

  // logged-out → ideas → home → login (the home-button route to auth).
  await page.click("#title-ideas");
  await page.waitForSelector("#ideas-view:not([hidden])");
  await page.click("#home-button");
  await page.waitForSelector("#title-view:not([hidden])");
  await loginViaUi(page, email, `Funnel ${tag}`);
  await page.click("#title-ideas");
  await page.waitForSelector("#ideas-view:not([hidden])");
  await revealSubmit(page);

  // Spy on the actual submit events: their count and the input value at dispatch
  // is the ground truth for "a phantom empty submit fired".
  await page.evaluate(() => {
    window.__ideaSubmits = [];
    document
      .querySelector("#ideas-form")
      .addEventListener("submit", () => window.__ideaSubmits.push(document.querySelector("#ideas-text").value), true);
  });

  const ideas = [`${tag}-Alpha`, `${tag}-Beta`, `${tag}-Gamma`];
  for (const text of ideas) {
    await page.fill("#ideas-text", text);
    await page.click("#ideas-submit"); // no settle wait — mash the form
  }
  // Let the in-flight submits drain — wait for the FIRST idea to land (one-per-
  // day caps the rest), then assert no phantom fired.
  await page.waitForFunction(
    (first) => [...document.querySelectorAll("#ideas-list .ideas-text")].some((e) => e.textContent === first),
    ideas[0],
    { timeout: 10_000 },
  ).catch(() => {}); // a miss is the defect — asserted explicitly below, not thrown

  const fired = await page.evaluate(() => window.__ideaSubmits);
  check(
    !fired.some((v) => v.trim() === ""),
    `${tag} funnel: no phantom EMPTY submit fires`,
    `fired ${JSON.stringify(fired)}`,
  );
  const status = (await page.locator("#ideas-status").textContent().catch(() => "")) ?? "";
  check(
    !status.includes("Type an idea"),
    `${tag} funnel: no spurious "type an idea" error after real submits`,
    `status ${JSON.stringify(status)}`,
  );
  const rows = await rowTexts(page);
  check(rows.includes(ideas[0]), `${tag} funnel: the first idea landed in the table`, `rows ${rows.length}`);
  // One idea per player per day: exactly ONE of the mashed ideas reaches the
  // table — the rest are refused by the day limit, never silently swallowed and
  // never phantom-duplicated.
  const landed = rows.filter((t) => ideas.includes(t));
  check(landed.length === 1, `${tag} funnel: exactly one idea lands (one-per-day), no dup/drop`, `${landed.length} of ${ideas.length}`);

  await ctx.close();
}

for (const [viewport, tag] of [
  [DESKTOP, "desktop"],
  [PHONE, "375px"],
]) {
  // Distinct email per pass: a fresh account so the one-vote rule is real.
  await scenario(viewport, tag, `ideas-${tag.replace(/[^a-z0-9]/gi, "")}@probe.test`);
  await funnelScenario(viewport, tag, `funnel-${tag.replace(/[^a-z0-9]/gi, "")}@probe.test`);
}

await browser.close();
disarm();
finish("probe-ideas");
