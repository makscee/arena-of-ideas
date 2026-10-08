// M2-6 pick screens e2e (makscee/void-board#794): on a 360×640 phone, a new
// player gets "+1 idea", writes one, waits for the fake reader, sees Home's
// "💡 Your idea is ready", turns the archetypes down once ("None of these"),
// picks an archetype, then a reading (opening "See Awoken"), and My ideas
// shows it as being tested. M3-2 (makscee/void-board#802): each reading's rule
// is text under its card, on screen untapped, and with the idea spent "New
// idea" is off (unfilled, unpressable) with "N more runs for an idea" beside it. M2-11 (makscee/void-board#796): the dev "Run the
// overnight check now" puts it in the vote, and a second player votes for it
// on Home's either/or card; "+5 fake votes" and the dev "End day now" with
// rotation on (M2-10) let it into the pool, and the Codex shows its card with
// NEW and its sheet with "idea by @Pick…". A screenshot of
// each screen; the pick screens never scroll the page. Then My ideas and the
// reading pick once at 1280×800, its sheet in the side panel. Without --url
// it builds the client and starts a dev server (fake reader, reading every
// 500 ms, the instant tuner, rotation on) on a free port; never point it at
// the live game. A --url server needs MVP_DEV=1, ARENA_TUNER=instant and
// MVP_ROTATION=1.
//   npm run mvp:ideas-phone -- [--url http://127.0.0.1:PORT/arena/] [--out e2e/.shots/mvp-ideas]
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/mvp-ideas";
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", ARENA_IDEA_READER: "fake", ARENA_IDEA_READ_MS: "500", ARENA_TUNER: "instant", MVP_ROTATION: "1" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
  for (let i = 0; i < 300; i++) {
    try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
    await new Promise((r) => setTimeout(r, 200));
  }
}
if (/makscee\.ru|twin-pogona/.test(url)) throw new Error("never against the live game");

const TAG = Date.now().toString(36).slice(-4);
const IDEA = "A hedgehog that punishes whoever hits it with its spines.";
const browser = await launchChromium();
const errors = [];
let shots = 0;

/** One player's walk at `viewport`; `full` walks every step, else only to the reading pick. */
async function walk(viewport, name, full) {
  const page = await browser.newPage({ viewport, deviceScaleFactor: 2, isMobile: viewport.width < 700, hasTouch: viewport.width < 700 });
  page.on("pageerror", (e) => errors.push(`${name}: pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && errors.push(`${name}: console: ${m.text()}`));
  const shot = (s) => page.screenshot({ path: `${out}/${String(++shots).padStart(2, "0")}-${s}.png` });
  /** The page doesn't scroll: everything fits the viewport (the sheet's own box may scroll). */
  const noScroll = async (s) => {
    const { w, h } = await page.evaluate(() => ({ w: document.documentElement.scrollWidth, h: document.documentElement.scrollHeight }));
    if (w > viewport.width) errors.push(`${s}: horizontal scroll (${w}px)`);
    if (h > viewport.height + 1) errors.push(`${s}: the page scrolls (${h}px tall)`);
  };
  const onScreen = async (s, loc) => {
    const b = await loc.boundingBox();
    if (!b || b.y < 0 || b.y + b.height > viewport.height + 0.5) errors.push(`${s}: off screen (${b ? Math.round(b.y) : "none"})`);
  };
  await page.goto(url);
  await page.getByTestId("name-input").fill(`${name}${TAG}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").waitFor();
  // Dev "+1 idea", then My ideas → New idea → Send.
  await page.locator("details.dev summary").click();
  await page.getByTestId("grant-idea").click();
  await page.waitForFunction(() => /^💡 1 idea$/.test(document.querySelector('[data-testid="ideas"]')?.textContent ?? ""), null, { timeout: 10_000 });
  await page.getByTestId("ideas").click();
  await page.getByTestId("idea-new").click();
  await page.getByTestId("idea-text").fill(IDEA);
  if (full) await shot("write");
  await page.getByTestId("idea-send").click();
  await page.getByTestId("idea-sent").waitFor();
  if (full) await shot("sent-being-read");
  // My ideas looks again on its own while the idea is read (10 s).
  await page.locator('[data-testid="idea-sent-row"][data-state="pick-archetype"]').waitFor({ timeout: 30_000 });
  if (!full) await shot(`${name}-my-ideas`);
  if (full) {
    if ((await page.getByTestId("idea-stage").textContent()) !== "Ready: pick its archetype") errors.push(`my ideas: stage "${await page.getByTestId("idea-stage").textContent()}"`);
    await shot("ideas-ready");
    // Home: the quiet "Your idea is ready".
    await page.getByTestId("ideas-back").click();
    await page.getByTestId("play").waitFor();
    if ((await page.getByTestId("ideas").textContent()) !== "💡 Your idea is ready") errors.push(`home: ideas line "${await page.getByTestId("ideas").textContent()}"`);
    await shot("home-idea-ready");
    await page.getByTestId("ideas").click();
  }
  // The archetype pick: 3 big rows, Confirm only after a tap.
  await page.getByTestId("idea-pick").click();
  await page.getByTestId("pick-archetypes").waitFor();
  const rows = page.getByTestId("pick-archetype");
  if ((await rows.count()) !== 3) errors.push(`archetypes: ${await rows.count()} rows`);
  if (!(await page.getByTestId("pick-confirm").isDisabled())) errors.push("archetypes: Confirm before a pick");
  if (full) {
    await shot("pick-archetype");
    await noScroll("pick-archetype");
    await onScreen("pick-archetype: Confirm", page.getByTestId("pick-confirm"));
    // None of these, once: read again for other archetypes.
    const first = await rows.allTextContents();
    if (!/read it again/.test(await page.getByTestId("pick-none").textContent())) errors.push("none: the first one doesn't read again");
    await page.getByTestId("pick-none").click();
    await page.getByTestId("idea-sent").waitFor();
    await page.locator('[data-testid="idea-sent-row"][data-state="pick-archetype"]').waitFor({ timeout: 30_000 });
    await page.getByTestId("idea-pick").click();
    await page.getByTestId("pick-archetypes").waitFor();
    const again = await rows.allTextContents();
    if (again.some((t) => first.includes(t))) errors.push(`none: the same archetypes again (${again.join(" | ")})`);
    if (!/take my idea back/.test(await page.getByTestId("pick-none").textContent())) errors.push("none: the second one doesn't say it gives the idea back");
  }
  await rows.nth(1).click();
  if (await page.getByTestId("pick-confirm").isDisabled()) errors.push("archetypes: Confirm stays off after a tap");
  if (full) await shot("pick-archetype-tapped");
  await page.getByTestId("pick-confirm").click();
  await page.getByTestId("idea-sent").waitFor();
  await page.locator('[data-testid="idea-sent-row"][data-state="pick-reading"]').waitFor({ timeout: 30_000 });
  // The reading pick: 3 cards like the shop's, numbers set by simulation.
  await page.getByTestId("idea-pick").click();
  await page.getByTestId("pick-readings").waitFor();
  const cards = page.getByTestId("pick-reading");
  if ((await cards.count()) !== 3) errors.push(`readings: ${await cards.count()} cards`);
  // M3-2: each card's sleeping rule as text under it, readable without a tap, inside the screen.
  const rules = page.getByTestId("pick-rule");
  if ((await rules.count()) !== 3) errors.push(`readings: ${await rules.count()} rule lines`);
  for (let i = 0; i < (await rules.count()); i++) {
    const rule = rules.nth(i);
    if (((await rule.textContent()) ?? "").trim().length < 10) errors.push(`reading ${i}: rule "${await rule.textContent()}"`);
    await onScreen(`${name} reading ${i}'s rule`, rule);
    const over = await rule.evaluate((el) => el.scrollWidth > el.clientWidth + 0.5);
    if (over) errors.push(`${name} reading ${i}'s rule overflows its column`);
  }
  await shot(full ? "pick-reading" : `${name}-pick-reading`);
  await noScroll(`${name} pick-reading (untapped)`);
  // Tap low on the card (its top opens nothing here, but stay clear of the icon line).
  await cards.nth(0).click({ position: { x: 30, y: 60 } });
  await page.getByTestId("unit-sheet").waitFor();
  if (!/set by simulation/.test(await page.getByTestId("sheet-unset").textContent())) errors.push("reading sheet: no 'set by simulation'");
  if ((await page.locator('[data-testid="pick-detail"] .term, [data-testid="pick-detail"] [data-term]').count()) === 0) errors.push("reading sheet: no highlighted keywords");
  await shot(`${name}-pick-reading-tapped`);
  await noScroll(`${name} pick-reading`);
  await onScreen(`${name} pick-reading: Confirm`, page.getByTestId("pick-confirm"));
  await page.getByTestId("see-awoken").click();
  if (!/Awoken/.test(await page.getByTestId("sheet-state").textContent())) errors.push("see awoken: the sheet still says Sleeping");
  await shot(`${name}-pick-reading-awoken`);
  await noScroll(`${name} pick-reading-awoken`);
  if (!full) return page.close();
  await page.getByTestId("pick-confirm").click();
  await page.locator('[data-testid="idea-sent-row"][data-state="simulating"]').waitFor();
  if (!/test it overnight/.test(await page.getByTestId("idea-sent").textContent())) errors.push(`after the pick: "${await page.getByTestId("idea-sent").textContent()}"`);
  if (!/being tested/.test(await page.getByTestId("idea-stage").textContent())) errors.push(`my ideas after the pick: "${await page.getByTestId("idea-stage").textContent()}"`);
  await shot("ideas-being-tested");
  // M3-2: the idea is spent: New idea is off, muted, with the reason beside it.
  const newIdea = page.getByTestId("idea-new");
  if (!(await newIdea.isDisabled())) errors.push("my ideas with none held: New idea isn't disabled");
  const look = await newIdea.evaluate((el) => { const c = getComputedStyle(el); return { bg: c.backgroundColor, op: c.opacity, pe: c.pointerEvents }; });
  if (!/rgba\(0, 0, 0, 0\)|transparent/.test(look.bg) || look.pe !== "none") errors.push(`my ideas: New idea still looks live (${JSON.stringify(look)})`);
  const why = (await page.getByTestId("idea-new-why").textContent()) ?? "";
  if (!/^\d+ more runs? for an idea$/.test(why)) errors.push(`my ideas: New idea's reason "${why}"`);
  // M2-11: the dev "Run the overnight check now" (the server's instant
  // tuner), and My ideas shows the idea in the vote.
  await page.getByTestId("ideas-back").click();
  await page.locator("details.dev summary").click();
  await page.getByTestId("overnight-check").click();
  await page.waitForFunction(() => /Checking 1 idea/.test(document.querySelector('[data-testid="error"]')?.textContent ?? ""), null, { timeout: 10_000 });
  await shot("overnight-check");
  for (let i = 0; ; i++) {
    await page.getByTestId("ideas").click();
    await page.getByTestId("idea-sent-row").first().waitFor();
    if ((await page.getByTestId("idea-sent-row").first().getAttribute("data-state")) === "voting") break;
    if (i === 20) throw new Error(`the overnight check left the idea "${await page.getByTestId("idea-stage").first().textContent()}"`);
    await page.getByTestId("ideas-back").click();
    await page.waitForTimeout(500);
  }
  if (!/in the vote/.test(await page.getByTestId("idea-stage").first().textContent())) errors.push(`my ideas after the check: "${await page.getByTestId("idea-stage").first().textContent()}"`);
  await shot("ideas-in-the-vote");
  await page.close();
  return vote(viewport);
}

/** A second player votes for the idea's unit on Home's either/or card. */
async function vote(viewport) {
  const [cand] = await (await fetch(url + "api/v1/dev/candidates")).json();
  if (!cand) return void errors.push("vote: no candidate after the check");
  const page = await browser.newPage({ viewport, deviceScaleFactor: 2, isMobile: true, hasTouch: true });
  page.on("pageerror", (e) => errors.push(`Voter: pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && errors.push(`Voter: console: ${m.text()}`));
  const shot = (s) => page.screenshot({ path: `${out}/${String(++shots).padStart(2, "0")}-${s}.png` });
  await page.goto(url);
  await page.getByTestId("name-input").fill(`Vote${TAG}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("vote-card").waitFor();
  const mine = page.locator(`[data-testid="vote-pick"][data-unit="${cand.unitId}"]`);
  if ((await mine.count()) !== 1) errors.push(`vote: the card doesn't show ${cand.name}`);
  await page.getByTestId("vote-card").scrollIntoViewIfNeeded();
  await shot("vote-card");
  await mine.locator(".vote-text").first().click(); // the text, clear of the card itself
  await page.waitForFunction(() => !!document.querySelector('[data-testid="vote-thanks"]') || !!document.querySelector('[data-testid="vote-card"]'));
  await page.getByTestId("vote-box").scrollIntoViewIfNeeded();
  await shot("voted");
  const after = (await (await fetch(url + "api/v1/dev/candidates")).json()).find((c) => c.unitId === cand.unitId);
  if (after?.votes !== 1) errors.push(`vote: ${cand.name} has ${after?.votes} votes, not 1`);
  // Rotation (M2-10): "+5 fake votes" lifts it over the vote bar, "End day
  // now" (MVP_ROTATION=1) lets it in, and the Codex shows it NEW, by its author.
  await page.locator("details.dev summary").click();
  await Promise.all([page.waitForResponse((r) => r.url().endsWith("/dev/fake-votes")), page.getByTestId("fake-votes").click()]);
  const scored = (await (await fetch(url + "api/v1/dev/candidates")).json()).find((c) => c.unitId === cand.unitId);
  if (!scored?.qualified) errors.push(`rotation: ${cand.name} doesn't qualify (${JSON.stringify(scored)})`);
  await page.goto(url);
  await page.locator("details.dev summary").click();
  await page.getByTestId("end-day").click();
  await page.getByTestId("day-ended").waitFor({ timeout: 30_000 });
  await shot("day-ended");
  await page.reload(); // a fresh load takes the new pool at once (otherwise within 15 s)
  await page.getByTestId("codex").click();
  await page.getByTestId("codex-search").fill(cand.name);
  const card = page.locator('[data-testid="codex-unit"]').filter({ has: page.getByTestId("card-new") });
  await card.first().waitFor({ timeout: 10_000 });
  if ((await card.count()) !== 1) errors.push(`rotation: ${await card.count()} NEW cards named ${cand.name}`);
  if ((await card.first().getByTestId("card-by").count()) !== 1) errors.push(`rotation: ${cand.name}'s card has no 💡`);
  await shot("codex-new-unit");
  await card.first().click({ position: { x: 32, y: 60 } });
  const credit = page.getByTestId("sheet-credit");
  await credit.waitFor();
  const text = (await credit.textContent()) ?? "";
  if (!/NEW/.test(text) || !text.includes(`idea by @Pick${TAG}`)) errors.push(`rotation: the sheet says "${text}"`);
  await shot("new-unit-sheet");
  await page.close();
}

try {
  await walk({ width: 360, height: 640 }, "Pick", true);
  await walk({ width: 360, height: 740 }, "Tall", false); // M3-2: the three rules side by side on a common phone
  await walk({ width: 1280, height: 800 }, "Desk", false);
} catch (e) {
  errors.push(`threw: ${e.message}`);
} finally {
  await browser.close();
  child?.kill();
}
console.log(`${shots} screenshots in ${out}`);
if (errors.length) {
  console.error(errors.map((e) => `✗ ${e}`).join("\n"));
  process.exit(1);
}
console.log("✓ write → archetype → reading → overnight check → a second player's vote → the day end lets it in, NEW and by its author, on a phone, and My ideas and the reading pick on desktop");
