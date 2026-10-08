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
// M3-5 (makscee/void-board#807): once the day end has sent a unit to the
// Library, a new player proposes a new version of it: Codex → Library → its
// sheet's "Propose a new version" (off, with the reason, until they hold an
// idea) → the write screen with its line and current rule → My ideas "new
// version of …" → the reading pick titled "A new version of …" with the
// current rule first → "being tested overnight"; at 360×740, and to the
// reading pick at 1280×800. M3-9 (makscee/void-board#800): then the overnight
// check, fake votes and the day end let the version in: the Codex shows it NEW
// as v2, "evolved by @Prop…", with both versions; the old one stays in the Library.
//   npm run mvp:ideas-phone -- [--url http://127.0.0.1:PORT/arena/] [--out e2e/.shots/mvp-ideas] [--lang ru]
// --lang ru walks it on Russian phones (the browser's locale) and reads every
// checked text from the Russian catalog.
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";
import { catalog, e2eLang, localeOf, pattern } from "./lang.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const lang = e2eLang(args);
const L = catalog(lang);
const out = opt("out") ?? `e2e/.shots/mvp-ideas${lang === "en" ? "" : `-${lang}`}`;
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
  const page = await browser.newPage({ viewport, deviceScaleFactor: 2, isMobile: viewport.width < 700, hasTouch: viewport.width < 700, locale: localeOf(lang) });
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
  await page.waitForFunction((want) => document.querySelector('[data-testid="ideas"]')?.textContent === want, L("home.ideasHeld", { n: 1 }), { timeout: 10_000 });
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
    if ((await page.getByTestId("idea-stage").textContent()) !== L("ideas.stage.pickArchetype")) errors.push(`my ideas: stage "${await page.getByTestId("idea-stage").textContent()}"`);
    await shot("ideas-ready");
    // Home: the quiet "Your idea is ready".
    await page.getByTestId("ideas-back").click();
    await page.getByTestId("play").waitFor();
    if ((await page.getByTestId("ideas").textContent()) !== L("home.ideasReady", { n: 1 })) errors.push(`home: ideas line "${await page.getByTestId("ideas").textContent()}"`);
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
    if ((await page.getByTestId("pick-none").textContent()) !== L("pick.noneReadAgain")) errors.push("none: the first one doesn't read again");
    await page.getByTestId("pick-none").click();
    await page.getByTestId("idea-sent").waitFor();
    await page.locator('[data-testid="idea-sent-row"][data-state="pick-archetype"]').waitFor({ timeout: 30_000 });
    await page.getByTestId("idea-pick").click();
    await page.getByTestId("pick-archetypes").waitFor();
    const again = await rows.allTextContents();
    if (again.some((t) => first.includes(t))) errors.push(`none: the same archetypes again (${again.join(" | ")})`);
    if ((await page.getByTestId("pick-none").textContent()) !== L("pick.noneTakeBack")) errors.push("none: the second one doesn't say it gives the idea back");
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
  if (!(await page.getByTestId("sheet-unset").textContent()).includes(L("card.setBySim"))) errors.push("reading sheet: no 'set by simulation'");
  if ((await page.locator('[data-testid="pick-detail"] .term, [data-testid="pick-detail"] [data-term]').count()) === 0) errors.push("reading sheet: no highlighted keywords");
  await shot(`${name}-pick-reading-tapped`);
  await noScroll(`${name} pick-reading`);
  await onScreen(`${name} pick-reading: Confirm`, page.getByTestId("pick-confirm"));
  await page.getByTestId("see-awoken").click();
  if ((await page.getByTestId("sheet-state").textContent()).includes(L("card.sleeping"))) errors.push("see awoken: the sheet still says Sleeping");
  await shot(`${name}-pick-reading-awoken`);
  await noScroll(`${name} pick-reading-awoken`);
  if (!full) return page.close();
  await page.getByTestId("pick-confirm").click();
  await page.locator('[data-testid="idea-sent-row"][data-state="simulating"]').waitFor();
  if (!(await page.getByTestId("idea-sent").textContent()).includes(L("pick.pickedReading"))) errors.push(`after the pick: "${await page.getByTestId("idea-sent").textContent()}"`);
  if ((await page.getByTestId("idea-stage").textContent()) !== L("ideas.stage.simulating")) errors.push(`my ideas after the pick: "${await page.getByTestId("idea-stage").textContent()}"`);
  await shot("ideas-being-tested");
  // M3-2: the idea is spent: New idea is off, muted, with the reason beside it.
  const newIdea = page.getByTestId("idea-new");
  if (!(await newIdea.isDisabled())) errors.push("my ideas with none held: New idea isn't disabled");
  const look = await newIdea.evaluate((el) => { const c = getComputedStyle(el); return { bg: c.backgroundColor, op: c.opacity, pe: c.pointerEvents }; });
  if (!/rgba\(0, 0, 0, 0\)|transparent/.test(look.bg) || look.pe !== "none") errors.push(`my ideas: New idea still looks live (${JSON.stringify(look)})`);
  const why = (await page.getByTestId("idea-new-why").textContent()) ?? "";
  if (!new RegExp(`^${pattern(L("ideas.why", { n: 1 }))}$|^${pattern(L("ideas.why", { n: 2 }))}$|^${pattern(L("ideas.why", { n: 5 }))}$`).test(why)) errors.push(`my ideas: New idea's reason "${why}"`);
  // M2-11: the dev "Run the overnight check now" (the server's instant
  // tuner), and My ideas shows the idea in the vote.
  await page.getByTestId("ideas-back").click();
  await page.locator("details.dev summary").click();
  await page.getByTestId("overnight-check").click();
  await page.waitForFunction((want) => document.querySelector('[data-testid="error"]')?.textContent === want, L("dev.checking", { n: 1 }), { timeout: 10_000 });
  await shot("overnight-check");
  for (let i = 0; ; i++) {
    await page.getByTestId("ideas").click();
    await page.getByTestId("idea-sent-row").first().waitFor();
    if ((await page.getByTestId("idea-sent-row").first().getAttribute("data-state")) === "voting") break;
    if (i === 20) throw new Error(`the overnight check left the idea "${await page.getByTestId("idea-stage").first().textContent()}"`);
    await page.getByTestId("ideas-back").click();
    await page.waitForTimeout(500);
  }
  if (!(await page.getByTestId("idea-stage").first().textContent()).includes(L("ideas.stage.voting"))) errors.push(`my ideas after the check: "${await page.getByTestId("idea-stage").first().textContent()}"`);
  await shot("ideas-in-the-vote");
  await page.close();
  return vote(viewport);
}

/** A second player votes for the idea's unit on Home's either/or card. */
async function vote(viewport) {
  const [cand] = await (await fetch(url + "api/v1/dev/candidates")).json();
  if (!cand) return void errors.push("vote: no candidate after the check");
  const page = await browser.newPage({ viewport, deviceScaleFactor: 2, isMobile: true, hasTouch: true, locale: localeOf(lang) });
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
  if (!text.includes(L("card.new")) || !text.includes(`${L("card.ideaBy")}@Pick${TAG}`)) errors.push(`rotation: the sheet says "${text}"`);
  await shot("new-unit-sheet");
  await page.close();
}

/** M3-5: a new player proposes a new version of a Library unit, to the reading pick (`full`: through Confirm). */
async function propose(viewport, name, full) {
  const page = await browser.newPage({ viewport, deviceScaleFactor: 2, isMobile: viewport.width < 700, hasTouch: viewport.width < 700, locale: localeOf(lang) });
  page.on("pageerror", (e) => errors.push(`${name}: pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && errors.push(`${name}: console: ${m.text()}`));
  const shot = (s) => page.screenshot({ path: `${out}/${String(++shots).padStart(2, "0")}-${name}-${s}.png` });
  const noScroll = async (s) => {
    const { w, h } = await page.evaluate(() => ({ w: document.documentElement.scrollWidth, h: document.documentElement.scrollHeight }));
    if (w > viewport.width) errors.push(`${name} ${s}: horizontal scroll (${w}px)`);
    if (h > viewport.height + 1) errors.push(`${name} ${s}: the page scrolls (${h}px tall)`);
  };
  await page.goto(url);
  await page.getByTestId("name-input").fill(`${name}${TAG}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").waitFor();
  const openLibraryUnit = async () => {
    await page.getByTestId("codex").click();
    await page.getByTestId("codex-tab-library").click();
    const unit = page.getByTestId("library-unit").first();
    await unit.waitFor({ timeout: 10_000 });
    const id = await unit.getAttribute("data-unit");
    await unit.click({ position: { x: 30, y: 60 } });
    await page.getByTestId("library-propose").waitFor();
    return id;
  };
  // No idea held: the button is off, with the reason beside it.
  await openLibraryUnit();
  const btn = page.getByTestId("library-propose");
  if (!(await btn.isDisabled())) errors.push(`${name}: Propose is on with no idea held`);
  const why = (await page.getByTestId("library-propose-why").textContent()) ?? "";
  if (!new RegExp(`^${pattern(L("ideas.why", { n: 1 }))}$|^${pattern(L("ideas.why", { n: 2 }))}$|^${pattern(L("ideas.why", { n: 5 }))}$`).test(why)) errors.push(`${name}: Propose's reason "${why}"`);
  await shot("library-propose-off");
  // +1 idea (dev on Home), then the same sheet: on.
  await page.goto(url);
  await page.locator("details.dev summary").click();
  await page.getByTestId("grant-idea").click();
  await page.waitForFunction((want) => document.querySelector('[data-testid="ideas"]')?.textContent === want, L("home.ideasHeld", { n: 1 }), { timeout: 10_000 });
  const target = await openLibraryUnit();
  if (await btn.isDisabled()) errors.push(`${name}: Propose is off with an idea held`);
  if ((await page.getByTestId("library-propose-why").count()) !== 0) errors.push(`${name}: Propose says why it's off while on`);
  await shot("library-propose-on");
  await btn.click();
  // The write screen: its title, line and current rule over the box.
  const title = page.getByTestId("write-title");
  await title.waitFor();
  const unitName = await (await fetch(url + "api/v1/library")).json().then((l) => l.units.find((x) => x.unit.id === target)?.unit);
  const named = `A new version of ${unitName.emoji} ${unitName.name}`;
  if ((await title.textContent()) !== named) errors.push(`${name}: write title "${await title.textContent()}"`);
  if ((await page.getByTestId("propose-line").textContent()) !== unitName.archetype) errors.push(`${name}: write line "${await page.getByTestId("propose-line").textContent()}"`);
  if (!new RegExp(`^${pattern(L("ideas.now"))}.{10,}`).test((await page.getByTestId("propose-rule").textContent()) ?? "")) errors.push(`${name}: write rule "${await page.getByTestId("propose-rule").textContent()}"`);
  if (!(await page.locator('label[for="idea-box"]').textContent()).includes(L("ideas.proposeLabel"))) errors.push(`${name}: the box isn't "What should change?"`);
  await page.getByTestId("idea-text").fill(WANT);
  await shot("propose-write");
  await noScroll("propose-write");
  await page.getByTestId("idea-send").click();
  await page.getByTestId("idea-sent").waitFor();
  const stage = page.getByTestId("idea-stage").first();
  // The fake reader may have finished already: being read, or ready to pick.
  const stageText = (await stage.textContent()) ?? "";
  const stageHead = L("ideas.namedVersion", { unit: `${unitName.emoji} ${unitName.name}` });
  if (!stageText.startsWith(stageHead) || ![L("ideas.stage.reading"), L("ideas.stage.pickReading")].includes(stageText.slice(stageHead.length))) errors.push(`${name}: my ideas while read "${stageText}"`);
  await shot("propose-sent");
  // The reader skips the archetype: straight to the readings.
  await page.locator('[data-testid="idea-sent-row"][data-state="pick-reading"]').waitFor({ timeout: 30_000 });
  await page.getByTestId("idea-pick").click();
  await page.getByTestId("pick-readings").waitFor();
  if ((await page.getByTestId("pick-title").textContent()) !== named) errors.push(`${name}: reading title "${await page.getByTestId("pick-title").textContent()}"`);
  const now = page.getByTestId("pick-current");
  if (!new RegExp(`^${pattern(L("pick.now"))}.{10,}`).test((await now.textContent()) ?? "")) errors.push(`${name}: reading pick's current rule "${await now.textContent()}"`);
  const nowBox = await now.boundingBox();
  const firstCard = await page.getByTestId("pick-reading").first().boundingBox();
  if (!nowBox || !firstCard || nowBox.y >= firstCard.y) errors.push(`${name}: the current version isn't above the readings`);
  if ((await page.getByTestId("pick-reading").count()) !== 3) errors.push(`${name}: ${await page.getByTestId("pick-reading").count()} readings`);
  await shot("propose-reading-pick");
  await noScroll("propose-reading-pick");
  // Its current version's sheet, then a reading's.
  await now.click({ position: { x: 12, y: 10 } });
  await page.locator('[data-testid="pick-detail"] [data-testid="unit-sheet"]').waitFor();
  if (!(await page.getByTestId("pick-confirm").isDisabled())) errors.push(`${name}: the current version can be confirmed`);
  await shot("propose-reading-current");
  await page.getByTestId("pick-reading").nth(0).click({ position: { x: 30, y: 60 } });
  await page.getByTestId("sheet-unset").waitFor();
  await shot("propose-reading-tapped");
  await noScroll("propose-reading-tapped");
  if (!full) return page.close();
  await page.getByTestId("pick-confirm").click();
  await page.locator('[data-testid="idea-sent-row"][data-state="simulating"]').waitFor();
  const after = (await page.getByTestId("idea-stage").first().textContent()) ?? "";
  if (after !== `${L("ideas.namedVersion", { unit: `${unitName.emoji} ${unitName.name}` })}${L("ideas.stage.simulating")}`) errors.push(`${name}: my ideas after the pick "${after}"`);
  await shot("propose-being-tested");
  await page.close();
  return { target, unit: unitName };
}

/** M3-9: the version goes through the overnight check and the vote, and the
 * day end lets it in: the Codex shows it NEW as v2, "evolved by @Prop…", with
 * both versions in its history; the old version stays in the Library. */
async function entered(viewport, { target, unit }) {
  const page = await browser.newPage({ viewport, deviceScaleFactor: 2, isMobile: true, hasTouch: true, locale: localeOf(lang) });
  page.on("pageerror", (e) => errors.push(`Enter: pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && errors.push(`Enter: console: ${m.text()}`));
  const shot = (s) => page.screenshot({ path: `${out}/${String(++shots).padStart(2, "0")}-Enter-${s}.png` });
  await page.goto(url);
  await page.getByTestId("name-input").fill(`Enter${TAG}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").waitFor();
  await page.locator("details.dev summary").click();
  await page.getByTestId("overnight-check").click();
  let version;
  for (let i = 0; !version && i < 40; i++) {
    version = (await (await fetch(url + "api/v1/dev/candidates")).json()).find((c) => c.kind === "version" && c.rootId === target);
    if (!version) await page.waitForTimeout(500);
  }
  if (!version) return void errors.push(`enter: no version of ${unit.name} in the vote after the check`);
  // The vote card offers it as "new version of …", or "… unchanged".
  await page.goto(url);
  const tag = page.getByTestId("vote-tag");
  if (await page.getByTestId("vote-card").count()) {
    await page.getByTestId("vote-card").scrollIntoViewIfNeeded();
    await shot("vote-card");
    const tags = (await tag.allTextContents()).join(" | ");
    if (tags && !tags.includes(unit.name)) errors.push(`enter: the vote card's tags "${tags}" don't name ${unit.name}`);
  }
  await page.locator("details.dev summary").click();
  await Promise.all([page.waitForResponse((r) => r.url().endsWith("/dev/fake-votes")), page.getByTestId("fake-votes").click()]);
  // Fake votes are even (4 of 5 for every candidate), so "unchanged" would win
  // on novelty: a real voter prefers the version on every card it's on, and
  // the typical unit over "unchanged".
  const api = url + "api/v1";
  const judge = (await (await fetch(api + "/players", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ name: `Judge${TAG}` }) })).json()).id;
  const as = { "content-type": "application/json", "X-Arena-Player": judge };
  let next = (await (await fetch(api + "/votes/next", { headers: as })).json()).card;
  for (let i = 0; next && i < 30; i++) {
    const pick = next.candidateId === version.unitId ? version.unitId : next.otherId;
    next = (await (await fetch(api + "/votes", { method: "POST", headers: as, body: JSON.stringify({ candidateId: next.candidateId, otherId: next.otherId, pick }) })).json()).card;
  }
  const scored = (await (await fetch(url + "api/v1/dev/candidates")).json()).find((c) => c.unitId === version.unitId);
  if (!scored?.entry) errors.push(`enter: the version isn't its archetype's entry (${JSON.stringify(scored)})`);
  await page.goto(url);
  await page.locator("details.dev summary").click();
  await page.getByTestId("end-day").click();
  await page.getByTestId("day-ended").waitFor({ timeout: 30_000 });
  await page.reload();
  await page.getByTestId("codex").click();
  await page.getByTestId("codex-search").fill(unit.name);
  const card = page.locator('[data-testid="codex-unit"]').filter({ has: page.getByTestId("card-version") });
  await card.first().waitFor({ timeout: 10_000 });
  if (!/v2/.test((await card.first().getByTestId("card-version").textContent()) ?? "")) errors.push(`enter: ${unit.name}'s card isn't v2`);
  if ((await card.first().getByTestId("card-new").count()) !== 1) errors.push(`enter: ${unit.name} v2 isn't NEW`);
  await shot("codex-v2");
  await card.first().click({ position: { x: 32, y: 60 } });
  const credit = page.getByTestId("sheet-credit");
  await credit.waitFor();
  const text = (await credit.textContent()) ?? "";
  if (!text.includes(`${L("card.evolvedBy")}@Prop${TAG}`)) errors.push(`enter: the sheet says "${text}"`);
  const history = (await page.getByTestId("sheet-history").textContent()) ?? "";
  if (!history.includes(L("codex.versions", { n: 2 }))) errors.push(`enter: the history says "${history}"`);
  await shot("v2-sheet");
  const live = await (await fetch(url + "api/v1/content")).json();
  if (!live.units.some((u) => u.id === version.unitId)) errors.push(`enter: ${version.unitId} isn't served under its own id`);
  if (live.units.some((u) => u.id === target)) errors.push(`enter: the old version ${target} is live too`);
  const lib = await (await fetch(url + "api/v1/library")).json();
  if (!lib.units.some((l) => l.unit.id === target)) errors.push(`enter: the old version ${target} left the Library`);
  await page.close();
}
const WANT = "Make it hit every enemy instead of only the front one.";

try {
  await walk({ width: 360, height: 640 }, "Pick", true);
  await walk({ width: 360, height: 740 }, "Tall", false); // M3-2: the three rules side by side on a common phone
  await walk({ width: 1280, height: 800 }, "Desk", false);
  // M3-5: the day end above sent a unit to the Library.
  const proposed = await propose({ width: 360, height: 740 }, "Prop", true);
  await entered({ width: 360, height: 740 }, proposed);
  await propose({ width: 1280, height: 800 }, "PropDesk", false);
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
console.log("✓ write → archetype → reading → overnight check → a second player's vote → the day end lets it in, NEW and by its author, on a phone, and My ideas and the reading pick on desktop; a new version of a Library unit: Propose (off, then on) → write → reading pick with the current rule first → being tested → the vote → the day end lets it in as v2, evolved by its author, on a phone, and to the reading pick on desktop");
