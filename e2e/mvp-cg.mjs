// The CrazyGames build (mission #1214, slice 1): the zip's files served from a
// second origin play against the Arena server. Builds the cg zip pointed at a
// local server, serves dist-cg on another port, joins by name, buys a unit and
// fights one battle, and fails on any console error, failed request, request
// outside those two origins, off-portal link or leftover Telegram/email/invite UI.
//   npm run mvp:cg [-- --shots e2e/.shots/mvp-cg]
import { spawn, execFileSync } from "node:child_process";
import { createReadStream, existsSync, mkdirSync, statSync } from "node:fs";
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

const api = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], {
  env: { ...process.env, PORT: String(apiPort), MVP_DB: ":memory:", MVP_INVITES: "1", MVP_OPEN: "1", MVP_CORS_ORIGINS: pageOrigin },
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
    const page = await browser.newPage({ viewport: { width: 800, height: 450 } });
    const origins = new Set();
    page.on("request", (r) => { if (!r.url().startsWith("data:")) origins.add(new URL(r.url()).origin); });
    page.on("requestfailed", (r) => errors.push(`request failed: ${r.url()} ${r.failure()?.errorText}`));
    page.on("response", (r) => r.status() >= 400 && errors.push(`${r.status()}: ${r.url()}`));
    page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
    page.on("console", (m) => m.type() === "error" && errors.push(`console: ${m.text()}`));

    await page.goto(`${pageOrigin}/index.html`, { timeout: 20_000 });
    await page.getByTestId("join-name").waitFor({ timeout: 15_000 });
    await page.screenshot({ path: `${shots}/01-join.png` });
    for (const id of ["tg-login", "email-signin-open", "own-link-row"]) if (await page.getByTestId(id).count()) errors.push(`the cg build shows ${id}`);
    await page.getByTestId("join-name").fill(`Cg${Date.now().toString(36).slice(-5)}`);
    await page.keyboard.press("Enter");
    await page.getByTestId("play").waitFor({ timeout: 15_000 });
    await page.screenshot({ path: `${shots}/02-home.png` });
    if (await page.locator(".dev").count()) errors.push("the cg build shows the dev tools");
    await page.getByTestId("play").click();
    await page.getByTestId("fight").waitFor({ timeout: 15_000 });
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
    const foreign = [...origins].filter((o) => o !== pageOrigin && o !== apiOrigin);
    if (foreign.length) errors.push(`requests outside the two origins: ${foreign}`);
    if (!origins.has(apiOrigin)) errors.push("the client never reached the API origin");
  } finally {
    await browser.close();
  }
} catch (e) {
  errors.push(String(e?.stack ?? e));
} finally {
  api.kill();
  pages.close();
}
failed = errors.length > 0;
console.log(failed ? `mvp:cg FAILED\n${[...new Set(errors)].join("\n")}` : `mvp:cg ok (screenshots in ${shots})`);
process.exit(failed ? 1 : 0);
