// The CrazyGames frame (#1219, #1220): Arena in the crazy kit's shared preview
// (makscee/crazy-kit `npm run cgpreview`, r/crazy/cg/arena/), which swaps the
// SDK for its mock. Plays a guest run to the first battle in the 800×450 frame,
// checks the SDK log order and the time to gameplay, then logs in as the mock's
// one CrazyGames user (PreviewTester) and checks that a cleared save (a second
// device) gets the same player back, and that a new guest gets another.
// Screenshots go to --shots and are listed in shots/index.json.
//   npm run mvp:cg-playtest [-- --url https://m1.twin-pogona.ts.net/r/crazy/cg/arena/] [--shots DIR]
import { mkdirSync, writeFileSync } from "node:fs";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (name, d) => (args.includes(name) ? args[args.indexOf(name) + 1] : d);
const url = opt("--url", "https://m1.twin-pogona.ts.net/r/crazy/cg/arena/");
const shots = opt("--shots", "e2e/.shots/cg-playtest");
mkdirSync(shots, { recursive: true });
const errors = [];
const captions = [];
const browser = await launchChromium();
try {
  // A fresh context: the frame's saved size (800×450) and account (guest) start at their defaults.
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 900 } });
  const page = await ctx.newPage();
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  // A track the game stops streaming is aborted, the browser's own favicon probe isn't the game's: neither is a failure.
  const known = (u) => /\/music\/[^/]+\.m4a$/.test(u) || u.endsWith("/favicon.ico");
  page.on("console", (m) => m.type() === "error" && !/status of 404/.test(m.text()) && errors.push(`console: ${m.text()}`));
  // The frame reloads on Log in and Clear save: requests in flight then end ERR_ABORTED.
  page.on("requestfailed", (r) => !known(r.url()) && r.failure()?.errorText !== "net::ERR_ABORTED" && errors.push(`request failed: ${r.url()} ${r.failure()?.errorText}`));
  page.on("response", (r) => r.status() >= 400 && !known(r.url()) && errors.push(`${r.status()}: ${r.url()}`));
  await page.goto(url, { timeout: 30_000 });
  const game = page.frameLocator("#game");
  const inFrame = () => page.frames().find((f) => f !== page.mainFrame());
  const player = () => inFrame().evaluate(() => JSON.parse(localStorage.getItem("arena.player") ?? "null"));
  const shot = async (file, caption) => { await page.screenshot({ path: `${shots}/${file}` }); captions.push({ file, caption }); };
  // Log in, Guest and Clear save reload the game: wait for the new load (cg_run counts loads) to reach Home.
  const reloads = async (click) => {
    const run = await page.locator("#game").getAttribute("src");
    await click();
    await page.waitForFunction((was) => document.getElementById("game").getAttribute("src") !== was, run, { timeout: 10_000 });
    await page.waitForFunction(() => /SDK mock loaded/.test(document.getElementById("ev").innerText), null, { timeout: 30_000 });
    await game.getByTestId("play").waitFor({ timeout: 30_000 });
  };
  const named = (re, what) => page.waitForFunction((src) => {
    const f = document.getElementById("game").contentWindow;
    try { return new RegExp(src, "u").test(JSON.parse(f.localStorage.getItem("arena.player") ?? "{}").name ?? ""); } catch { return false; }
  }, re.source, { timeout: 30_000 }).catch(() => errors.push(what));

  // First open: a fresh guest at Home, in the 800×450 frame.
  await game.getByTestId("play").waitFor({ timeout: 60_000 });
  const box = await page.locator("#frame").boundingBox();
  if (Math.round(box.width) !== 800 || Math.round(box.height) !== 450) errors.push(`frame is ${box.width}×${box.height}, not 800×450`);
  await named(/^\p{L}+ \p{L}+ \d+$/u, "first open isn't a fresh guest");
  const guest = await player();
  await shot("01-home.png", "First open at 800×450: a fresh guest at Home. The log shows init, loadingStart, loadingStop.");

  // A guest run to the first battle.
  await game.getByTestId("play").click();
  await game.getByTestId("fight").waitFor({ timeout: 20_000 });
  await game.getByTestId("offer-0").click(); await game.getByTestId("offer-0").click();
  await game.getByTestId("buy").waitFor();
  await game.getByTestId("buy").click();
  await game.getByTestId("line").locator(".card.you").first().waitFor({ timeout: 10_000 });
  await shot("02-shop.png", "In the shop: gameplayStart is logged; line, bench and Fight fit at 800×450.");
  await game.getByTestId("fight").click();
  await game.getByTestId("battle-end").waitFor({ timeout: 20_000 });
  await shot("03-battle.png", "The first battle.");
  await game.getByTestId("battle-end").click();
  await game.getByTestId("battle-done").waitFor({ timeout: 40_000 });
  await game.getByTestId("battle-done").click();
  await game.getByTestId("fight").or(game.getByTestId("run-over")).first().waitFor({ timeout: 20_000 });
  await game.getByTestId("menu-open").click();
  await game.getByTestId("menu-title").click();
  await game.getByTestId("play").waitFor({ timeout: 15_000 });

  // The SDK log, in order, and the frame's time to first gameplay.
  const log = await page.locator("#ev").innerText();
  console.log(log);
  let at = 0;
  for (const want of ["SDK mock loaded", "game.loadingStart", "game.loadingStop", "game.gameplayStart", "game.gameplayStop"]) {
    const i = log.indexOf(want, at);
    if (i < 0) errors.push(`log has no "${want}" after position ${at}`);
    else at = i + want.length;
  }
  const first = await page.locator("#first").innerText();
  if (!/\bok\b/.test(await page.locator("#first").getAttribute("class"))) errors.push(`first gameplay: ${first}`);

  // Logged in: the mock's CrazyGames user, named in the game.
  await reloads(() => page.locator('#acct button[data-user="1"]').click());
  await named(/^PreviewTester/u, "logging in didn't name the player after the CrazyGames user");
  await game.getByTestId("play").waitFor({ timeout: 30_000 });
  const linked = await player();
  await page.waitForTimeout(500);
  await shot("04-logged-in.png", "Logged in as PreviewTester: the game's player carries the CrazyGames username.");

  // A second device: the save cleared, still logged in, the same player comes back.
  await reloads(() => page.locator("#wipe").click());
  await named(/^PreviewTester/u, "after Clear save, the logged-in player didn't come back");
  const again = await player();
  await page.waitForTimeout(500);
  if (again?.id !== linked?.id) errors.push(`a cleared save logged in as another player: ${linked?.id} → ${again?.id}`);
  await shot("05-second-device.png", "Clear save, still logged in (a second device): the same PreviewTester player.");

  // Guest again on a cleared save: another player.
  await reloads(() => page.locator('#acct button[data-user="0"]').click());
  await reloads(() => page.locator("#wipe").click());
  await named(/^\p{L}+ \p{L}+ \d+$/u, "Clear save as a guest didn't make a new guest");
  const fresh = await player();
  await page.waitForTimeout(500);
  if (!fresh?.id || fresh.id === linked?.id || fresh.id === guest?.id) errors.push(`the new guest reused a player: ${fresh?.id}`);
  await shot("06-new-guest.png", "Guest and Clear save: a new guest at Home.");

  // Phone portrait.
  await page.locator('#dev button[data-w="390"]').click();
  const b2 = await page.locator("#frame").boundingBox();
  if (Math.round(b2.width) !== 390 || Math.round(b2.height) !== 844) errors.push(`frame is ${b2.width}×${b2.height}, not 390×844`);
  await page.waitForTimeout(500);
  await shot("07-phone.png", "Phone portrait.");
  writeFileSync(`${shots}/index.json`, JSON.stringify(captions, null, 1));
} catch (e) {
  errors.push(String(e?.stack ?? e));
} finally {
  await browser.close();
}
console.log(errors.length ? `cg-playtest FAILED\n${[...new Set(errors)].join("\n")}` : `cg-playtest ok (screenshots in ${shots})`);
process.exit(errors.length ? 1 : 0);
