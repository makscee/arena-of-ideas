// A press must never read a state that is not there yet (arena#28, round 7: `mvp:desktop` threw
// "Cannot read properties of undefined (reading 'unitId')" in two runs of three). The cause: a
// predicted draw (the table drawn before the server answers) asks the server for the run's forge, and
// the answer describes the board BEFORE the press (the decision is still on its way), so its pairs
// point at slots the predicted board no longer has. This test sells the unit of a pair with every
// decision and forge call delayed, so the stale forge arrives onto the predicted board, and fails on
// any page error (with its stack). The scene `fusion-now` holds two Awoken units of a designed pair.
// Without --url it builds the client and starts a scenes server.
//   npm run mvp:race -- [--url http://127.0.0.1:PORT/arena/] [--runs 5]
import { spawn, execFileSync } from "node:child_process";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const RUNS = Number(opt("runs") ?? 5);
let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_CODEX: "1", MVP_POOL: "v2", ARENA_GUEST_JOINS_PER_IP: "500", ARENA_GUEST_JOINS_PER_HOUR: "500" }, stdio: ["ignore", "ignore", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

const errors = [];
const browser = await launchChromium();
try {
  for (let run = 1; run <= RUNS; run++) {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const page = await ctx.newPage();
    const fail = (m) => errors.push(`run ${run}: ${m}`);
    page.on("pageerror", (e) => fail(`pageerror: ${e.message}\n${(e.stack ?? "").split("\n").slice(1, 6).join("\n")}`));
    let delaying = false;
    await page.route(/\/runs\/[^/]+\/(decisions|preview|forge)$/, async (route) => {
      // Only the decisions wait: the forge is asked at once and answers with the board as it still is.
      if (delaying && /decisions$/.test(route.request().url())) await new Promise((r) => setTimeout(r, 1200));
      await route.continue().catch(() => {});
    });
    await page.goto(`${url}#scene=fusion-now&lang=en`);
    await page.getByTestId("fight").waitFor({ timeout: 60_000 });
    await page.waitForResponse((r) => /\/forge$/.test(r.url()), { timeout: 15_000 }).catch(() => {});
    await page.waitForTimeout(600);
    // The sleeping third unit goes first (settled), leaving the pair alone: line-0 and line-1.
    if (await page.getByTestId("line-2").count()) {
      await page.getByTestId("line-2").click();
      await page.getByTestId("sell").click();
      await page.getByTestId("line-2-empty").waitFor({ timeout: 10_000 });
      await page.waitForTimeout(600);
    }
    // Now sell the pair's second unit with the decision on its way: the board shows one unit while the forge still says two.
    delaying = true;
    await page.getByTestId("line-1").click();
    await page.getByTestId("sell").click();
    await page.getByTestId("line-1-empty").waitFor({ timeout: 2_000 }).catch(() => fail("the sold unit isn't gone at once"));
    await page.waitForTimeout(1800); // the stale forge and the decision's answer both land
    if (!(await page.getByTestId("line-0").count())) fail("the table lost the remaining unit");
    await ctx.close();
  }
} finally {
  await browser.close();
  child?.kill();
}
if (errors.length) { console.error(errors.join("\n")); process.exit(1); }
console.log(`mvp race: ok (${RUNS} runs)`);
