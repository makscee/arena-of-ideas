// Report (#1271, mission #1214): a player's name (the champion, a battle's foe)
// and a player-made unit's sheet each have a small "Report"; sending one asks
// the server (POST /reports) and thanks the player; the report sits stored
// (the sheet's own 200 and the second try's "already" show it).
// The world's other player ("Maker") is a real one made over the API, whose
// units are credited to them through the dev route; only the champion's and the
// foe's seats are swapped in the page's replies, since the bot world fills them.
//   npm run mvp:report -- [--url http://127.0.0.1:PORT/arena/] [--out e2e/.shots/report]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/report";
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_TUNER: "instant", ARENA_CODEX: "1" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}
const api = (method, path, who, body) => fetch(`${url}api/v1${path}`, { method, headers: { "content-type": "application/json", ...(who ? { "X-Arena-Player": who.id } : {}) }, ...(body ? { body: JSON.stringify(body) } : {}) }).then((r) => r.json());
const maker = await api("POST", "/players", null, { name: "Amber Fox 417" });
// Every unit of the pool is Maker's idea, so any sheet in the Codex has a credit to report.
for (const u of (await api("GET", "/content")).units) await api("POST", "/dev/credit-unit", maker, { unitId: u.id });

const failures = [];
const check = (ok, what) => { console.log(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failures.push(what); };
const browser = await launchChromium();
try {
  const open = async (scene, viewport, patch) => {
    const ctx = await browser.newContext({ viewport, isMobile: viewport.width < 1024, hasTouch: viewport.width < 1024 });
    const page = await ctx.newPage();
    page.on("pageerror", (e) => failures.push(`pageerror: ${e.message}`));
    if (patch) await page.route(patch.glob, async (route) => {
      const res = await route.fetch();
      const json = await res.json();
      patch.edit(json);
      await route.fulfill({ response: res, json });
    });
    await page.goto(`${url}#scene=${scene}&lang=en`);
    return { ctx, page };
  };
  /** Press Report, see the sheet, send with a reason, see the thanks, and the server's answer. */
  const flow = async (page, testid, shot) => {
    await page.getByTestId(testid).click();
    await page.getByTestId("report-sheet").waitFor({ timeout: 10_000 });
    await page.getByTestId("report-reason").fill("pretend this is rude");
    await page.screenshot({ path: `${out}/${shot}-sheet.png` });
    const reply = page.waitForResponse((r) => r.url().endsWith("/api/v1/reports") && r.request().method() === "POST");
    await page.getByTestId("report-send").click();
    check((await reply).status() === 200, `${shot}: the server took the report (200)`);
    await page.getByText("Thanks, we'll look at it.").waitFor({ timeout: 5_000 });
    await page.screenshot({ path: `${out}/${shot}-thanks.png` });
    await page.getByTestId("report-close").click();
    await page.getByTestId("report-sheet").waitFor({ state: "hidden", timeout: 5_000 });
  };

  // The champion's name on Home.
  for (const [name, viewport] of [["phone", { width: 390, height: 844 }], ["800x450", { width: 800, height: 450 }]]) {
    const { ctx, page } = await open("home", viewport, { glob: "**/api/v1/home", edit: (j) => { if (j.day.champion) j.day.champion.player = maker; } });
    await page.getByTestId("report-name").waitFor({ timeout: 30_000 });
    await page.screenshot({ path: `${out}/home-${name}.png` });
    await flow(page, "report-name", `champion-${name}`);
    await ctx.close();
  }
  // The foe's name in a battle's header; with a bot foe there is no Report.
  {
    const { ctx, page } = await open("battle-win", { width: 390, height: 844 });
    await page.getByTestId("battle-you").waitFor({ timeout: 60_000 });
    check((await page.getByTestId("report-name").count()) === 0, "battle against a bot: no Report on its name");
    await ctx.close();
  }
  for (const [name, viewport] of [["phone", { width: 390, height: 844 }], ["800x450", { width: 800, height: 450 }], ["1280x720", { width: 1280, height: 720 }]]) {
    const { ctx, page } = await open("battle-win", viewport, { glob: "**/api/v1/battles/*", edit: (j) => { j.opponent = maker; } });
    await page.getByTestId("report-name").waitFor({ timeout: 60_000 });
    const box = await page.getByTestId("report-name").boundingBox();
    check(!!box && box.x >= 0 && box.x + box.width <= viewport.width && box.y >= 0 && box.y + box.height <= viewport.height, `battle ${name}: Report sits inside the screen`);
    await page.screenshot({ path: `${out}/battle-${name}.png` });
    if (name === "phone") await flow(page, "report-name", `battle-${name}`);
    await ctx.close();
  }
  // A player-made unit's sheet in the Codex; a second report of the same unit answers ok too.
  {
    const { ctx, page } = await open("codex-units", { width: 390, height: 844 });
    await page.getByTestId("codex-unit").first().click();
    await page.getByTestId("unit-sheet").waitFor({ timeout: 30_000 });
    await page.getByTestId("report-unit").waitFor({ timeout: 5_000 });
    await page.screenshot({ path: `${out}/unit-sheet.png` });
    await flow(page, "report-unit", "unit");
    await ctx.close();
  }
} finally {
  await browser.close();
  child?.kill();
}
console.log(failures.length ? `mvp:report FAILED\n${failures.join("\n")}` : `mvp:report ok (${out})`);
process.exit(failures.length ? 1 : 0);
