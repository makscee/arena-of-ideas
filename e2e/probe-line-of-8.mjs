// R4-10: the viewer on a line of 8 (both sides: Mortwrought's Ghouls, Imps and
// revives fill each line from 5 to 8; e2e/line-of-8-battle.ts). Plays a round
// on a LOCAL server with that battle swapped in, seeks to the turn where both
// lines hold 8, and checks on a phone and a desktop viewport that the page
// doesn't scroll sideways and every card sits on screen without overlapping
// its neighbours. Screenshots each viewport (the compact layout is R4-11).
// Needs `npm run mvp:build` first.   node e2e/probe-line-of-8.mjs [out.png]
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { launchChromium } from "./browser.mjs";
const rec = join(mkdtempSync(join(tmpdir(), "line-of-8-")), "line-of-8.json");
execFileSync("node", ["--import", "tsx/esm", "e2e/line-of-8-battle.ts", rec], { stdio: ["ignore", "inherit", "inherit"] });
const record = JSON.parse(readFileSync(rec, "utf8"));
const fullTurn = Number(readFileSync(`${rec}.turn`, "utf8"));
const out = process.argv[2] ?? "e2e/.shots/line-of-8.png";
mkdirSync(dirname(out), { recursive: true });
const port = 18000 + Math.floor(Math.random() * 1000);
const child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", STATIC_DIR: "mobile/dist" }, stdio: ["ignore", "ignore", "inherit"] });
const url = `http://127.0.0.1:${port}/arena/`;
try {
  for (let i = 0; i < 120; i++) { try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {} await new Promise((r) => setTimeout(r, 500)); }
  const browser = await launchChromium();
  let ok = true;
  for (const [label, viewport] of [["phone", { width: 360, height: 640 }], ["desktop", { width: 1280, height: 800 }], ["desktop-1024", { width: 1024, height: 768 }]]) {
    const page = await browser.newPage({ viewport, deviceScaleFactor: 2 });
    await page.route((u) => /\/battles\/[^/]+$/.test(u.pathname), async (route) => {
      const res = await route.fetch();
      const body = await res.json();
      await route.fulfill({ response: res, json: { ...body, ...record, battleId: body.battleId, player: body.player, runId: body.runId, kind: body.kind, round: body.round } });
    });
    const errors = [];
    page.on("pageerror", (e) => errors.push(String(e)));
    await page.goto(url);
    await page.getByTestId("name-input").fill(`Line8${label.replace(/\W/g, "")}`);
    await page.getByTestId("name-submit").click();
    await page.getByTestId("play").click();
    await page.getByTestId("offer-0").click();
    await page.getByTestId("buy").click();
    await page.getByTestId("fight").click();
    await page.getByTestId("caption").waitFor();
    // Pause, then step (→) until both lines hold 8 and the turn after they
    // filled has begun (the phone has no timeline to click).
    await page.keyboard.press(" ");
    const slots = () => page.evaluate(() => ["battle-you", "battle-them"].map((id) => document.querySelectorAll(`[data-testid=${id}] .bv-slot`).length));
    for (let i = 0; i < 2000; i++) {
      const [a, b] = await slots();
      const turn = Number((await page.getByTestId("battle-turn").textContent())?.match(/\d+/)?.[0] ?? 0);
      if (a === 8 && b === 8 && turn > fullTurn) break;
      await page.keyboard.press("ArrowRight");
    }
    await page.waitForTimeout(1500);
    const m = await page.evaluate(() => {
      const vw = document.documentElement.clientWidth;
      const side = (id) => [...document.querySelectorAll(`[data-testid=${id}] .bv-slot`)].map((s) => s.getBoundingClientRect()).map((r) => ({ l: r.left, r: r.right, w: r.width }));
      return { vw, sw: document.documentElement.scrollWidth, you: side("battle-you"), them: side("battle-them"), playing: document.querySelector("[data-testid=battle-turn]")?.textContent };
    });
    const check = (cards) => cards.length === 8 && cards.every((c) => c.l >= -0.5 && c.r <= m.vw + 0.5 && c.w >= 30)
      && [...cards].sort((a, b) => a.l - b.l).every((c, i, s) => i === 0 || c.l >= s[i - 1].r - 0.5);
    const good = m.sw <= m.vw && check(m.you) && check(m.them) && errors.length === 0;
    const widths = (cs) => cs.map((c) => Math.round(c.w)).join(",");
    console.log(`${label}: ${m.playing}; page ${m.sw}/${m.vw}px; you ${m.you.length} cards [${widths(m.you)}], them ${m.them.length} [${widths(m.them)}]${errors.length ? `; errors: ${errors.join(" | ")}` : ""} → ${good ? "ok" : "BROKEN"}`);
    if (!good) ok = false;
    await page.screenshot({ path: out.replace(/\.png$/, `-${label}.png`) });
    await page.close();
  }
  console.log(ok ? "line of 8 probe: OK" : "line of 8 probe: FAILED");
  process.exitCode = ok ? 0 : 1;
  await browser.close();
} finally { child.kill(); }
