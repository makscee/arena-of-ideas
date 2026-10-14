// Back and Forward step through the screens (#1146, mission #1043): every
// screen change is a history entry of its own; a sheet or a battle still
// closes first; Esc still works.
//   1. Home → the Forge → the shop → the Codex (over the shop): Back goes
//      Codex → shop → Forge → Home one screen at a time, Forward the way
//      back; the screens come back built (Play, the shop, the Codex);
//   2. a sheet: Back closes it and stays on the screen, Esc closes it and
//      leaves no extra entry behind (the next Back leaves the screen);
//   3. an in-app Back button (the Forge's, the Codex's) is the browser's Back:
//      one step, and Forward goes on from there;
//   4. a battle: Back is its Esc (the run menu opens, Back closes it), Title
//      menu from there reaches Home.
// Without --url it builds the client and starts its own scenes server.
//   npm run mvp:history -- [--url http://127.0.0.1:PORT/arena/] [--out e2e/.shots/history]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/history";
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

const errors = [];
const fail = (m) => errors.push(m);
const browser = await launchChromium();
let page;
let screen = async () => "-";
try {
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  page = await ctx.newPage();
  page.on("console", (m) => m.type() === "warning" && console.log("console:", m.text()));
  page.on("pageerror", (e) => fail(`pageerror: ${e.message}`));
  page.on("response", (r) => { if (r.url().includes("/api/v1/") && r.status() >= 400) fail(`HTTP ${r.status()} ${new URL(r.url()).pathname}`); });
  const tid = (id) => page.getByTestId(id);
  screen = () => page.evaluate(() => document.getElementById("app").dataset.screen ?? "-");
  const idle = async () => {
    for (let i = 0; i < 2; i++) {
      await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
      await page.waitForFunction(() => !document.getElementById("app").classList.contains("busy"), null, { timeout: 30_000 });
    }
  };
  /** The browser's Back (or Forward) and the screen it should land on. */
  const step = async (dir, want, what) => {
    await page.evaluate((d) => (d < 0 ? history.back() : history.forward()), dir);
    try {
      await page.waitForFunction((w) => document.getElementById("app").dataset.screen === w, want, { timeout: 8_000 });
    } catch {
      fail(`${what}: wanted the ${want} screen, got ${await screen()}`);
    }
    await idle();
  };

  await page.goto(`${url}?lang=en`, { timeout: 20_000 });
  await tid("name-input").waitFor({ timeout: 10_000 });
  await tid("name-input").fill(`Hist${Date.now().toString(36).slice(-4)}`);
  await page.keyboard.press("Enter");
  await tid("play").waitFor();
  await idle();

  // ---------- 1. four screens, Back and Forward ----------
  await tid("forge-open").click();
  await tid("forge").waitFor();
  await idle();
  if ((await screen()) !== "forge") fail(`the Forge opened on the ${await screen()} screen`);
  await step(-1, "home", "Back from the Forge");
  await tid("play").waitFor();
  await step(1, "forge", "Forward to the Forge");
  await tid("forge-back").click(); // the Forge's own Back button is the browser's Back
  await tid("play").waitFor();
  await idle();
  if ((await screen()) !== "home") fail("the Forge's Back button didn't return Home");
  await step(1, "forge", "Forward after the Forge's Back button");
  await step(-1, "home", "Back again");
  await tid("play").click();
  await tid("fight").waitFor();
  await idle();
  await tid("menu-open").click();
  await tid("menu-codex").click();
  await tid("codex-back").waitFor();
  await idle();
  if ((await screen()) !== "codex") fail(`the Codex opened on the ${await screen()} screen`);
  await step(-1, "shop", "Back from the Codex");
  await tid("fight").waitFor();
  await step(-1, "home", "Back from the shop");
  await tid("play").waitFor();
  await page.screenshot({ path: `${out}/home-after-back.png` });
  await step(1, "shop", "Forward to the shop");
  await tid("fight").waitFor();
  await step(1, "codex", "Forward to the Codex");
  await tid("codex-back").waitFor();
  await page.screenshot({ path: `${out}/codex-after-forward.png` });
  await tid("codex-back").click();
  await tid("fight").waitFor();
  await idle();
  await step(-1, "home", "Back from the shop after the Codex's Back button");

  // ---------- 2. a sheet closes first; Esc leaves no entry behind ----------
  const depth = () => page.evaluate(() => history.length);
  await tid("settings").click();
  await page.locator(".overlay").first().waitFor();
  await page.evaluate(() => history.back());
  await page.locator(".overlay").first().waitFor({ state: "detached", timeout: 5_000 }).catch(() => fail("Back didn't close the settings sheet"));
  await idle();
  if ((await screen()) !== "home") fail(`Back from a sheet left Home (${await screen()})`);
  await tid("settings").click();
  await page.locator(".overlay").first().waitFor();
  await page.keyboard.press("Escape");
  await page.locator(".overlay").first().waitFor({ state: "detached", timeout: 5_000 }).catch(() => fail("Esc didn't close the settings sheet"));
  await idle();
  const before = await depth();
  await tid("settings").click();
  await page.locator(".overlay").first().waitFor();
  await page.keyboard.press("Escape");
  await page.locator(".overlay").first().waitFor({ state: "detached" });
  await idle();
  if ((await depth()) !== before) fail(`a sheet opened and closed by Esc left ${(await depth()) - before} history entries`);
  // ---------- 3. a battle: Back is its Esc; the shop's entry stays one ----------
  await tid("play").click();
  await tid("fight").waitFor();
  await idle();
  await page.keyboard.press("1"); // a unit to fight with
  await idle();
  await tid("fight").click();
  await page.waitForFunction(() => document.getElementById("app").dataset.screen === "battle", null, { timeout: 30_000 }).catch(() => fail("the fight didn't open a battle"));
  if ((await screen()) === "battle") {
    // Back is the battle's Esc: the run menu opens over it, and Back again closes it.
    await page.evaluate(() => history.back());
    await tid("run-menu").waitFor({ timeout: 8_000 }).catch(() => fail("Back in a battle didn't open the run menu"));
    await page.evaluate(() => history.back());
    await tid("run-menu").waitFor({ state: "detached", timeout: 8_000 }).catch(() => fail("Back didn't close the run menu"));
    if ((await screen()) !== "battle") fail(`closing the run menu left the battle (${await screen()})`);
    // Title menu from there takes the run's entries with it: one Back-step was Home.
    await page.keyboard.press("Escape");
    await tid("menu-title").click({ timeout: 8_000 }).catch(() => fail("no Title menu in the battle's run menu"));
    await tid("play").waitFor({ timeout: 10_000 }).catch(() => fail("Title menu didn't reach Home"));
    await idle();
    if ((await screen()) !== "home") fail(`Title menu ended on the ${await screen()} screen`);
  }
} catch (e) {
  fail(`${e.message.split("\n")[0]} (${await screen().catch(() => "page gone")})`);
  await page.screenshot({ path: `${out}/stuck.png` }).catch(() => {});
} finally {
  await browser.close();
  child?.kill();
}
if (errors.length) {
  console.error(`\n${errors.length} history problem(s):\n- ${errors.join("\n- ")}`);
  process.exit(1);
}
console.log("history: Back and Forward step through the screens");
