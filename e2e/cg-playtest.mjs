// The playtest frame (#1219, scripts/arena-cg-playtest.sh): opens the page,
// plays a guest run to the first battle in the 800×450 frame, presses every
// portal button and checks the SDK log order, the guest's storage and the
// login. Screenshots go to --shots and are listed in shots/index.json for the
// page to show.
//   npm run mvp:cg-playtest [-- --url https://m1.twin-pogona.ts.net/r/arena-cg/] [--shots DIR]
import { mkdirSync, writeFileSync } from "node:fs";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (name, d) => (args.includes(name) ? args[args.indexOf(name) + 1] : d);
// A new CrazyGames account each run: the page's one account keeps the player of the first login.
const url = `${opt("--url", "https://m1.twin-pogona.ts.net/r/arena-cg/")}?cguser=e2e-${Date.now()}`;
const shots = opt("--shots", "e2e/.shots/cg-playtest");
mkdirSync(shots, { recursive: true });
const errors = [];
const captions = [];
const browser = await launchChromium();
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  // The page's own shots/index.json is not there before the first publish; a track the game
  // stops streaming is aborted: neither is a failure.
  const known = (u) => u.endsWith("/shots/index.json") || /\/music\/[^/]+\.m4a$/.test(u);
  page.on("console", (m) => m.type() === "error" && !/status of 404/.test(m.text()) && errors.push(`console: ${m.text()}`));
  page.on("requestfailed", (r) => !known(r.url()) && errors.push(`request failed: ${r.url()} ${r.failure()?.errorText}`));
  page.on("response", (r) => r.status() >= 400 && !known(r.url()) && errors.push(`${r.status()}: ${r.url()}`));
  await page.goto(url, { timeout: 30_000 });
  const game = page.frameLocator("#game");
  const logText = () => page.locator("#log").innerText();
  const shot = async (file, caption) => { await page.screenshot({ path: `${shots}/${file}` }); captions.push({ file, caption }); };
  const inFrame = () => page.frames().find((f) => f !== page.mainFrame());

  // First open: a fresh guest at Home, in the 800×450 frame.
  await game.getByTestId("play").waitFor({ timeout: 60_000 });
  const box = await page.locator("#box").boundingBox();
  if (Math.round(box.width) !== 800 || Math.round(box.height) !== 450) errors.push(`frame is ${box.width}×${box.height}, not 800×450`);
  const player = () => inFrame().evaluate(() => JSON.parse(localStorage.getItem("arena.player") ?? "null"));
  const first = await player();
  if (!/^\p{L}+ \p{L}+ \d+$/u.test(first?.name ?? "")) errors.push(`first open isn't a fresh guest: ${first?.name}`);
  await shot("01-home.png", "First open at 800×450: a fresh guest at Home. The log shows init, loadingStart, loadingStop.");

  // Ad break, mute: logged and restored.
  await page.getByRole("button", { name: "Ad break" }).click();
  await page.waitForFunction(() => document.querySelector("#log").innerText.includes("adFinished"), null, { timeout: 10_000 }).catch(() => errors.push("the ad break never finished"));
  await page.getByRole("button", { name: "Mute site audio" }).click();
  await page.waitForFunction(() => document.querySelector("#log").innerText.includes("muteAudio=true"), null, { timeout: 5_000 }).catch(() => errors.push("mute wasn't logged"));
  await shot("02-muted.png", "Mute site audio pressed: the portal's settings change reaches the game.");
  await page.getByRole("button", { name: "Mute site audio" }).click();

  // A guest run to the first battle.
  await game.getByTestId("play").click();
  await game.getByTestId("fight").waitFor({ timeout: 20_000 });
  await game.getByTestId("offer-0").click();
  await game.getByTestId("buy").waitFor();
  await game.getByTestId("buy").click();
  await game.getByTestId("line").locator(".card.you").first().waitFor({ timeout: 10_000 });
  await shot("03-shop.png", "In the shop: gameplayStart is logged, line, bench and Fight fit at 800×450.");
  await game.getByTestId("fight").click();
  await game.getByTestId("battle-end").waitFor({ timeout: 20_000 });
  await shot("04-battle.png", "The first battle.");
  await game.getByTestId("battle-end").click();
  await game.getByTestId("battle-done").waitFor({ timeout: 40_000 });
  await game.getByTestId("battle-done").click();
  await game.getByTestId("fight").or(game.getByTestId("run-over")).first().waitFor({ timeout: 20_000 });

  // Menu → title: gameplayStop; then the CrazyGames login keeps the guest's player.
  await game.getByTestId("menu-open").click();
  await game.getByTestId("menu-title").click();
  await game.getByTestId("play").waitFor({ timeout: 15_000 });
  await page.getByRole("button", { name: "Log in as CrazyGames user" }).click();
  await inFrame().waitForFunction(() => /^PlaytestUser/.test(JSON.parse(localStorage.getItem("arena.player") ?? "{}").name ?? ""), null, { timeout: 20_000 }).catch(() => errors.push("the login didn't rename the guest to the CrazyGames username"));
  const linked = await player();
  if (linked?.id !== first?.id) errors.push(`login made another player: ${first?.id} → ${linked?.id}`);
  await game.getByTestId("play").waitFor({ timeout: 15_000 });
  await shot("05-logged-in.png", "After Log in as CrazyGames user: the same player, now named PlaytestUser, run kept.");

  // New guest: storage cleared, another player.
  await page.getByRole("button", { name: "New guest" }).click();
  await page.waitForFunction(() => document.querySelector("#log").innerText.includes("new guest"), null, { timeout: 5_000 });
  await game.getByTestId("play").waitFor({ timeout: 30_000 });
  await inFrame().waitForFunction((id) => JSON.parse(localStorage.getItem("arena.player") ?? "{}").id && JSON.parse(localStorage.getItem("arena.player")).id !== id, linked?.id, { timeout: 15_000 }).catch(() => errors.push("New guest kept the old player"));
  await shot("06-new-guest.png", "New guest: the frame's storage cleared, a new guest at Home.");

  // The log, in order.
  const log = await logText();
  console.log(log);
  const order = ["init", "game.loadingStart", "game.loadingStop", "ad.requestAd", "adStarted", "adFinished", "muteAudio=true", "game.gameplayStart", "game.gameplayStop", "user logged in", "new guest"];
  let at = 0;
  for (const want of order) {
    const i = log.indexOf(want, at);
    if (i < 0) errors.push(`log has no "${want}" after position ${at}`);
    else at = i + want.length;
  }

  // 821×462 and the full window.
  await page.getByRole("button", { name: "821×462" }).click();
  const b2 = await page.locator("#box").boundingBox();
  if (Math.round(b2.width) !== 821 || Math.round(b2.height) !== 462) errors.push(`frame is ${b2.width}×${b2.height}, not 821×462`);
  await shot("07-821x462.png", "The 821×462 size.");
  await page.getByRole("button", { name: "Full window" }).click();
  await page.waitForTimeout(500);
  await shot("08-full.png", "Full window.");
  writeFileSync(`${shots}/index.json`, JSON.stringify(captions, null, 1));
} catch (e) {
  errors.push(String(e?.stack ?? e));
} finally {
  await browser.close();
}
console.log(errors.length ? `cg-playtest FAILED\n${[...new Set(errors)].join("\n")}` : `cg-playtest ok (screenshots in ${shots})`);
process.exit(errors.length ? 1 : 0);
