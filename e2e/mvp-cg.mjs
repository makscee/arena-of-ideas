// The CrazyGames build (mission #1214, slices 1 and 3): the zip's files served
// from a second origin play against an invite-only Arena server. Builds the cg
// zip pointed at a local server, serves dist-cg on another port, plays as a
// guest at once (no form), buys a unit and fights one battle, and fails on any
// console error, failed request, request outside those origins (and the SDK's),
// off-portal link or leftover Telegram/email/invite UI. Then (#1218), with a
// mocked CrazyGames SDK whose tokens a test key signs: logging in keeps the
// guest's run under the CrazyGames name, a second browser with the same user is
// the same player, and the 6th guest from one address in an hour is refused.
// And (#1219) the SDK's lifecycle: the mock records every call, and loading,
// gameplay (shop and battle on, Home and the run menu off), muteAudio and an
// ad break must come in the order the portal expects.
//   npm run mvp:cg [-- --shots e2e/.shots/mvp-cg]
import { spawn, execFileSync } from "node:child_process";
import { generateKeyPairSync, sign } from "node:crypto";
import { createReadStream, existsSync, mkdirSync, mkdtempSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { createServer as http } from "node:http";
import { createServer } from "node:net";
import { extname, join, normalize } from "node:path";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const shots = args.includes("--shots") ? args[args.indexOf("--shots") + 1] : "e2e/.shots/mvp-cg";
mkdirSync(shots, { recursive: true });
const free = () => new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
const [apiPort, pagePort] = [await free(), await free()];
const apiOrigin = `http://127.0.0.1:${apiPort}`;
const pageOrigin = `http://localhost:${pagePort}`;

execFileSync("npm", ["run", "-s", "mvp:build:cg"], { stdio: "inherit", env: { ...process.env, VITE_API_ORIGIN: `${apiOrigin}/arena` } });
const root = "mobile/dist-cg";

// A test key stands in for CrazyGames' (the server takes it with MVP_DEV=1).
const keys = generateKeyPairSync("rsa", { modulusLength: 2048, publicKeyEncoding: { type: "spki", format: "pem" }, privateKeyEncoding: { type: "pkcs8", format: "pem" } });
const keyDir = mkdtempSync(join(tmpdir(), "arena-cg-key-"));
writeFileSync(join(keyDir, "cg.pem"), keys.publicKey);
const b64 = (x) => Buffer.from(JSON.stringify(x)).toString("base64url");
const cgToken = (userId, username) => {
  const now = Math.floor(Date.now() / 1000);
  const head = b64({ alg: "RS256", typ: "JWT" });
  const body = b64({ userId, username, gameId: "arena", profilePictureUrl: "", iat: now, exp: now + 3600 });
  return `${head}.${body}.${sign("RSA-SHA256", Buffer.from(`${head}.${body}`), keys.privateKey).toString("base64url")}`;
};
const SDK_URL = "https://sdk.crazygames.com/crazygames-sdk-v3.js";
// The SDK as the portal runs it, its user logged out until window.__cgLogin().
const mockSdk = (token) => `(() => {
  let user = null; const listeners = [];
  window.__cgLogin = () => { user = { username: "NightOwl" }; listeners.forEach((f) => f(user)); };
  const calls = (window.__cgCalls = []);
  const settings = { muteAudio: location.search.includes("mute=1"), disableChat: false };
  window.CrazyGames = { SDK: { environment: "crazygames", init: async () => { calls.push("init"); },
    game: { settings, loadingStart: () => calls.push("loadingStart"), loadingStop: () => calls.push("loadingStop"), gameplayStart: () => calls.push("gameplayStart"), gameplayStop: () => calls.push("gameplayStop"), addSettingsChangeListener: (f) => (window.__cgSettings = f) },
    ad: { requestAd: (type, cb) => { calls.push("requestAd:" + type); window.__cgAd = cb; } },
    user: {
    isUserAccountAvailable: true,
    getUser: async () => user,
    getUserToken: async () => { if (!user) throw new Error("userNotAuthenticated"); return ${JSON.stringify(token)}; },
    addAuthListener: (f) => listeners.push(f),
  } } };
  if (${JSON.stringify(token)} && location.search.includes("cguser=1")) user = { username: "NightOwl" };
})();`;

const api = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], {
  env: { ...process.env, PORT: String(apiPort), MVP_DB: ":memory:", MVP_INVITES: "1", MVP_DEV: "1", ARENA_CG: "1", ARENA_CG_PUBLIC_KEY: join(keyDir, "cg.pem"), MVP_CORS_ORIGINS: pageOrigin },
  stdio: ["ignore", "inherit", "inherit"],
});
const MIME = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".woff2": "font/woff2", ".m4a": "audio/mp4", ".mp3": "audio/mpeg", ".ogg": "audio/ogg", ".wav": "audio/wav", ".svg": "image/svg+xml" };
const pages = http((req, res) => {
  const path = normalize(decodeURIComponent(new URL(req.url, pageOrigin).pathname)).replace(/^\/+/, "") || "index.html";
  const file = join(root, path);
  if (path.startsWith("..") || !existsSync(file) || !statSync(file).isFile()) return void res.writeHead(404).end("not found");
  res.writeHead(200, { "content-type": MIME[extname(file)] ?? "application/octet-stream" });
  createReadStream(file).pipe(res);
}).listen(pagePort, "127.0.0.1");

let failed = false;
const errors = [];
try {
  for (let i = 0; i < 300; i++) {
    try { if ((await fetch(`${apiOrigin}/arena/api/v1/health`)).ok) break; } catch {}
    await new Promise((r) => setTimeout(r, 200));
  }
  const browser = await launchChromium();
  try {
    const token = cgToken("cg-e2e-1", "NightOwl");
    const newPage = async (opts = {}) => {
      const p = await browser.newPage({ viewport: { width: 800, height: 450 } });
      await p.route(SDK_URL, (r) => r.fulfill({ contentType: "text/javascript", body: mockSdk(token) }));
      return p;
    };
    const page = await newPage();
    const origins = new Set();
    page.on("request", (r) => { if (!r.url().startsWith("data:")) origins.add(new URL(r.url()).origin); });
    page.on("requestfailed", (r) => errors.push(`request failed: ${r.url()} ${r.failure()?.errorText}`));
    page.on("response", (r) => r.status() >= 400 && errors.push(`${r.status()}: ${r.url()}`));
    page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
    page.on("console", (m) => m.type() === "error" && errors.push(`console: ${m.text()}`));

    await page.goto(`${pageOrigin}/index.html?playtest=1`, { timeout: 20_000 });
    // A guest at once: Home with a generated name, no form.
    await page.getByTestId("play").waitFor({ timeout: 15_000 });
    await page.screenshot({ path: `${shots}/02-home.png` });
    if (await page.getByTestId("join-name").count()) errors.push("the cg build shows the name form");
    for (const id of ["tg-login", "email-signin-open", "own-link-row"]) if (await page.getByTestId(id).count()) errors.push(`the cg build shows ${id}`);
    const me = () => page.evaluate(() => JSON.parse(localStorage.getItem("arena.player") ?? "null"));
    const guest = await me();
    if (!/^\p{L}+ \p{L}+ \d+$/u.test(guest?.name ?? "")) errors.push(`guest name: ${guest?.name}`);
    if (await page.locator(".dev").count()) errors.push("the cg build shows the dev tools");
    const calls = () => page.evaluate(() => window.__cgCalls.join(" "));
    // #1219: SDK init, then loading start and stop around Home; no gameplay on Home.
    await page.waitForFunction(() => window.__cgCalls.includes("loadingStop"), null, { timeout: 5_000 }).catch(() => undefined);
    if ((await calls()) !== "init loadingStart loadingStop") errors.push(`lifecycle at Home: ${await calls()}`);
    await page.getByTestId("play").click();
    await page.getByTestId("fight").waitFor({ timeout: 15_000 });
    if (!(await calls()).endsWith("loadingStop gameplayStart")) errors.push(`lifecycle in the shop: ${await calls()}`);
    // The run menu is a pause: gameplayStop, and Resume starts it again.
    await page.getByTestId("menu-open").click();
    await page.getByTestId("run-menu").waitFor();
    await page.waitForFunction(() => window.__cgCalls.at(-1) === "gameplayStop", null, { timeout: 5_000 }).catch(async () => errors.push(`no gameplayStop with the run menu open: ${await calls()}`));
    await page.getByTestId("menu-resume").click();
    await page.waitForFunction(() => window.__cgCalls.at(-1) === "gameplayStart", null, { timeout: 5_000 }).catch(() => errors.push("no gameplayStart after Resume"));
    // muteAudio changed in the portal's menu: silent at once, and back when unmuted;
    // an ad break mutes while it plays and restores after, and can't undo muteAudio.
    const master = (want) => page.waitForFunction((w) => (w === 0 ? window.__music().master === 0 : window.__music().master > 0.1), want, { timeout: 5_000 });
    const gain = async (want, why) => master(want).catch(async () => errors.push(`${why}: master gain is ${await page.evaluate(() => window.__music().master)}`));
    await gain(1, "sound before muting");
    await page.evaluate(() => window.__cgSettings({ muteAudio: true, disableChat: false }));
    await gain(0, "muteAudio");
    await page.evaluate(() => window.__cgSettings({ muteAudio: false, disableChat: false }));
    await gain(1, "unmuting");
    // The playtest frame's message asks for an ad break through the SDK.
    await page.evaluate(() => window.postMessage({ type: "arena-playtest-ad" }, "*"));
    await page.waitForFunction(() => window.__cgCalls.includes("requestAd:midgame"), null, { timeout: 5_000 }).catch(() => errors.push("no ad request from the playtest message"));
    await page.evaluate(() => window.__cgAd.adStarted());
    await gain(0, "ad started");
    await page.evaluate(() => window.__cgAd.adFinished());
    await gain(1, "ad finished");
    await page.getByTestId("offer-0").click();
    await page.getByTestId("buy").waitFor();
    await page.getByTestId("buy").click();
    await page.getByTestId("line").locator(".card.you").first().waitFor({ timeout: 10_000 });
    await page.screenshot({ path: `${shots}/03-shop.png` });
    await page.getByTestId("fight").click();
    await page.getByTestId("battle-end").waitFor({ timeout: 15_000 });
    await page.getByTestId("battle-end").click();
    await page.getByTestId("battle-done").waitFor({ timeout: 30_000 });
    await page.screenshot({ path: `${shots}/04-battle.png` });

    // Credit links are plain text, and no link leaves the portal.
    await page.getByTestId("battle-done").click();
    await page.waitForFunction(() => document.querySelector('[data-testid="fight"], [data-testid="run-over"]'));
    const anchors = await page.evaluate(() => [...document.querySelectorAll("a[href]")].map((a) => a.getAttribute("href")));
    if (anchors.some((h) => /^https?:|^\/\//.test(h ?? ""))) errors.push(`outside links on screen: ${anchors}`);
    const foreign = [...origins].filter((o) => o !== pageOrigin && o !== apiOrigin && o !== new URL(SDK_URL).origin);
    if (foreign.length) errors.push(`requests outside the two origins: ${foreign}`);
    if (!origins.has(apiOrigin)) errors.push("the client never reached the API origin");

    // #1218: the CrazyGames user logs in mid-run: the same player, now NightOwl, the run kept.
    const runOf = (p) => p.evaluate(async (base) => {
      const r = await fetch(`${base}/arena/api/v1/home`, { headers: { "X-Arena-Token": localStorage.getItem("arena.token") } });
      return (await r.json()).activeRunId;
    }, apiOrigin);
    const run = await runOf(page);
    await page.evaluate(() => window.__cgLogin());
    await page.waitForFunction(() => JSON.parse(localStorage.getItem("arena.player") ?? "{}").name === "NightOwl", null, { timeout: 15_000 });
    // Home again, the CrazyGames name in the settings sheet.
    await page.getByTestId("settings").waitFor({ timeout: 15_000 });
    await page.getByTestId("settings").click();
    await page.getByTestId("settings-sheet").getByText("@NightOwl").waitFor({ timeout: 15_000 });
    await page.screenshot({ path: `${shots}/05-logged-in.png` });
    const linked = await me();
    if (linked.id !== guest.id) errors.push(`login made another player: ${guest.id} → ${linked.id}`);
    if (!run || (await runOf(page)) !== run) errors.push(`login lost the run ${run}`);

    // A second browser, the same CrazyGames user from the start: the same player and run.
    const other = await browser.newContext({ viewport: { width: 800, height: 450 } });
    const page2 = await other.newPage();
    await page2.route(SDK_URL, (r) => r.fulfill({ contentType: "text/javascript", body: mockSdk(token) }));
    await page2.goto(`${pageOrigin}/index.html?cguser=1`, { timeout: 20_000 });
    await page2.getByTestId("play").waitFor({ timeout: 15_000 });
    await page2.screenshot({ path: `${shots}/06-second-device.png` });
    const second = await page2.evaluate(() => JSON.parse(localStorage.getItem("arena.player") ?? "null"));
    if (second?.id !== guest.id) errors.push(`second device is another player: ${second?.id}`);
    if ((await runOf(page2)) !== run) errors.push("second device lost the run");
    await other.close();

    // Guests 2–5 from this address, then the 6th is refused politely.
    for (let i = 2; i <= 5; i++) {
      const r = await fetch(`${apiOrigin}/arena/api/v1/players/guest`, { method: "POST" });
      if (!r.ok) errors.push(`guest ${i}: ${r.status}`);
    }
    const sixth = await browser.newContext({ viewport: { width: 800, height: 450 } });
    const page6 = await sixth.newPage();
    await page6.route(SDK_URL, (r) => r.fulfill({ contentType: "text/javascript", body: mockSdk(token) }));
    await page6.goto(`${pageOrigin}/index.html`, { timeout: 20_000 });
    await page6.getByText("Too many new players from here this hour").waitFor({ timeout: 15_000 }).catch(() => errors.push("the 6th guest wasn't refused politely"));
    await page6.screenshot({ path: `${shots}/07-sixth-guest.png` });
    await sixth.close();

    // #1219: a portal that starts the game with muteAudio on: silent from the first sound.
    const quiet = await browser.newContext({ viewport: { width: 800, height: 450 } });
    const page7 = await quiet.newPage();
    await page7.route(SDK_URL, (r) => r.fulfill({ contentType: "text/javascript", body: mockSdk(token) }));
    await page7.goto(`${pageOrigin}/index.html?cguser=1&mute=1`, { timeout: 20_000 });
    await page7.getByTestId("play").waitFor({ timeout: 15_000 });
    await page7.getByTestId("play").click();
    await page7.getByTestId("fight").waitFor({ timeout: 15_000 });
    const silent = await page7.waitForFunction(() => window.__music?.().master === 0 && window.__sfx.length > 0, null, { timeout: 5_000 }).then(() => true, () => false);
    if (!silent) errors.push("muteAudio at start left the sound on");
    await quiet.close();
  } finally {
    await browser.close();
  }
} catch (e) {
  errors.push(String(e?.stack ?? e));
} finally {
  api.kill();
  pages.close();
  rmSync(keyDir, { recursive: true, force: true });
}
failed = errors.length > 0;
console.log(failed ? `mvp:cg FAILED\n${[...new Set(errors)].join("\n")}` : `mvp:cg ok (screenshots in ${shots})`);
process.exit(failed ? 1 : 0);
