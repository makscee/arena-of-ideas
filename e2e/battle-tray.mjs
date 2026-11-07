// The battle tray's ladder (arena#52, mission #45 move 3): in the lab scenes battle-win, battle-chain-once and battle-six, at 800×450,
// 360×640 and 390×844 (1× and 2×), stepping every beat: the ladder keeps its four rows and its size, the controls stay in their cells
// (never moving, 44 px on a phone), the foot row is whole, the rule under it is whole, and nothing leaves the tray.
// Without --url it builds the client and starts a scenes server.
//   npm run mvp:battle-tray -- [--url http://127.0.0.1:PORT/arena/] [--scenes battle-win,battle-six] [--size 800x450,390x844] [--out e2e/.shots/battle-tray]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/battle-tray";
const SCENES = (opt("scenes") ?? "battle-win,battle-chain-once,battle-six").split(",");
const SIZES = (opt("size") ?? "800x450,360x640,390x844").split(",");
const SCALES = (opt("scales") ?? "1,2").split(",").map(Number);
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_TUNER: "instant", ARENA_CODEX: "1", MVP_POOL: "v2" }, stdio: ["ignore", "ignore", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) { try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {} await new Promise((r) => setTimeout(r, 200)); }

const CONTROLS = ["battle-back", "battle-play", "battle-step", "battle-speed", "battle-end", "battle-replay", "chain-skip"];

/** What the tray looks like now, in viewport pixels. */
const read = (page) => page.evaluate((controls) => {
  const box = (el) => { if (!el) return null; const r = el.getBoundingClientRect(); return { l: r.left, t: r.top, r: r.right, b: r.bottom, w: r.width, h: r.height }; };
  const st = document.querySelector('[data-testid="stage"]');
  const k = st.getBoundingClientRect().width / st.offsetWidth;
  const tray = document.querySelector('[data-testid="tray"]');
  const rows = [...document.querySelectorAll(".bv-ladder .bv-lrow")];
  const foot = document.querySelector('[data-testid="caption"]');
  const rule = document.querySelector('[data-testid="rule"]');
  const txt = foot?.querySelector(".bv-ltxt");
  return {
    k, tray: box(tray), ladder: box(document.querySelector(".bv-ladder")),
    rows: rows.map(box), rowText: rows.map((r) => r.textContent.trim()),
    controls: Object.fromEntries(controls.map((id) => [id, box(document.querySelector(`[data-testid="${id}"]`))])),
    foot: box(foot), footText: (foot?.textContent ?? "").trim(), footCut: txt ? txt.scrollWidth > txt.clientWidth + 1 || txt.scrollHeight > txt.clientHeight + 1 : false,
    rule: box(rule), ruleText: (rule?.textContent ?? "").trim(), ruleCut: rule ? rule.scrollHeight > rule.clientHeight + 1 : false,
    head: document.querySelector('[data-testid="ledger-turn"]')?.textContent ?? "", ended: !!document.querySelector('[data-testid="end-card"]')?.checkVisibility?.(),
  };
}, CONTROLS);

const same = (a, b, tol = 0.6) => !!a && !!b && ["l", "t", "r", "b"].every((k) => Math.abs(a[k] - b[k]) <= tol);
const inside = (a, tray, tol = 1) => a.l >= tray.l - tol && a.r <= tray.r + tol && a.t >= tray.t - tol && a.b <= tray.b + tol;

const bad = [];
const browser = await launchChromium();
try {
  for (const scene of SCENES) for (const size of SIZES) for (const dsf of SCALES) {
    const [W, H] = size.split("x").map(Number);
    const what = `${scene} ${size}@${dsf}×`;
    const ctx = await browser.newContext({ viewport: { width: W, height: H }, deviceScaleFactor: dsf, isMobile: W < 500, hasTouch: W < 500 });
    const page = await ctx.newPage();
    page.on("pageerror", (e) => bad.push(`${what}: pageerror ${e.message}`));
    await page.goto(`${url}#scene=${scene}&lang=en`);
    await page.getByTestId("battle-play").waitFor({ timeout: 60_000 });
    await page.waitForTimeout(500);
    if ((await page.getByTestId("battle-play").textContent()).includes("❚")) await page.getByTestId("battle-play").click();
    await page.waitForTimeout(250);
    let first = null, beats = 0, ruled = 0, chained = 0;
    for (let i = 0; i < 60; i++) {
      const s = await read(page);
      if (s.ended) break;
      beats++;
      const tag = `${what} beat ${i}`;
      first ??= s;
      if (s.rows.length !== 4) bad.push(`${tag}: ${s.rows.length} ladder rows, four expected`);
      if (!same(s.ladder, first.ladder)) bad.push(`${tag}: the ladder moved or changed size`);
      s.rows.forEach((r, j) => { if (Math.abs(r.h - first.rows[j].h) > 0.6 || Math.abs(r.t - first.rows[j].t) > 0.6) bad.push(`${tag}: ladder row ${j} moved or changed height`); });
      for (const id of CONTROLS) {
        if (!same(s.controls[id], first.controls[id])) bad.push(`${tag}: ${id} moved`);
        const c = s.controls[id];
        if (c && W < 500 && (c.w < 43.5 || c.h < 43.5)) bad.push(`${tag}: ${id} is ${Math.round(c.w)}×${Math.round(c.h)} px on a phone`);
      }
      if (!s.footText) bad.push(`${tag}: the foot row is empty`);
      if (s.footCut) bad.push(`${tag}: the foot row is cut ("${s.footText.slice(0, 50)}")`);
      if (s.ruleCut) bad.push(`${tag}: the rule is cut ("${s.ruleText.slice(0, 50)}")`);
      if (s.ruleText) ruled++;
      if (/CHAIN/i.test(await page.getByTestId("chain-count").textContent())) chained++;
      for (const [name, b] of [["ladder", s.ladder], ["foot row", s.foot], ["rule", s.rule], ...CONTROLS.map((id) => [id, s.controls[id]])]) if (b && !inside(b, s.tray)) bad.push(`${tag}: ${name} leaves the tray`);
      // Landscape: the controls stand right of the ladder. Portrait: below it.
      const back = s.controls["battle-back"];
      if (s.ladder && back && (W < 500 ? s.ladder.b > back.t + 0.5 : s.ladder.r > back.l + 0.5)) bad.push(`${tag}: the ladder runs into the controls`);
      if (i % 5 === 2) await page.screenshot({ path: `${out}/${scene}-${size}@${dsf}-${String(i).padStart(2, "0")}.png` });
      await page.getByTestId("battle-step").click();
      await page.waitForTimeout(120);
    }
    if (beats < 4) bad.push(`${what}: only ${beats} beats stepped`);
    if (!ruled) bad.push(`${what}: no step showed its rule`);
    if (scene !== "battle-win" && !chained && scene === "battle-chain-once") bad.push(`${what}: no chain count shown`);
    console.log(`${what}: ${beats} beats, ${ruled} with a rule, ${chained} in a chain`);
    await ctx.close();
  }
} finally {
  await browser.close();
  child?.kill();
}
if (bad.length) { console.log(bad.slice(0, 40).join("\n")); console.log(`FAIL: ${bad.length} problem(s)`); process.exit(1); }
console.log("battle-tray: ok");
