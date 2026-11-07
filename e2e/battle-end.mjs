// The battle's end explains itself (arena#53, mission #45 move 5): in the lab scenes battle-win, battle-loss, battle-chain-once and battle-six, at
// 800×450, 360×640 and 390×844 (1× and 2×), End shows the result: the board stays, the verdict card sits on the loser's row above the names and stats
// (covering neither), the tray's four rows are the causes (44 px on a phone, each opening its moment, paused), one computed sentence names the same
// units and numbers, the controls stay in the cells they had (End is Why, the chain-skip cell is Next), Why opens the deciding chain paused, ▶ plays
// it, Replay and Next are their own cells, and nothing leaves the screen.
// Without --url it builds the client and starts a scenes server.
//   npm run mvp:battle-end -- [--url http://127.0.0.1:PORT/arena/] [--scenes battle-win,battle-loss] [--size 800x450,390x844] [--out e2e/.shots/battle-end]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/battle-end";
const SCENES = (opt("scenes") ?? "battle-win,battle-loss,battle-chain-once,battle-six").split(",");
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

const CELLS = ["battle-back", "battle-play", "battle-step", "battle-speed", "battle-end", "battle-replay", "chain-skip"];
const AFTER = { "battle-end": "end-why", "battle-replay": "end-replay", "chain-skip": "battle-done" };

/** The screen now, in viewport pixels. */
const read = (page) => page.evaluate(({ cells, after }) => {
  const box = (el) => { if (!el) return null; const r = el.getBoundingClientRect(); return { l: r.left, t: r.top, r: r.right, b: r.bottom, w: r.width, h: r.height }; };
  const q = (id) => document.querySelector(`[data-testid="${id}"]`);
  const end = q("end-card");
  const row = end?.dataset.row ?? "";
  const sentence = q("carry-sentence");
  const ruleEl = q("rule");
  const boxes = (sel) => [...document.querySelectorAll(sel)].map(box);
  return {
    vw: innerWidth, vh: innerHeight, shown: !!end?.checkVisibility?.(),
    end: box(end), row, word: q("battle-word")?.textContent ?? "", sub: q("end-sub")?.textContent ?? "",
    tray: box(q("tray")), ladder: box(document.querySelector(".bv-ladder")), head: document.querySelector(".bv-lturn")?.textContent ?? "",
    carry: [...document.querySelectorAll('[data-testid="carry-row"]')].map((r) => ({ ...box(r), text: r.textContent.trim(), kind: r.dataset.kind, event: r.dataset.event, cut: r.scrollHeight > r.clientHeight + 1 })),
    rows: [...document.querySelectorAll(".bv-ladder .bv-lrow")].filter((r) => r.checkVisibility()).map(box), sentence: (sentence?.textContent ?? "").trim(), ruleCut: ruleEl ? ruleEl.scrollHeight > ruleEl.clientHeight + 1 : false, rule: box(ruleEl),
    winnerLine: box(q(row === "theirs" ? "battle-you" : "battle-them")), loserLine: box(q(row === "theirs" ? "battle-them" : "battle-you")),
    texts: boxes(".bv-line .pc-nm, .bv-line .pc-b"),
    cells: Object.fromEntries(cells.map((id) => [id, box(q(id) ?? q(after[id]))])),
    ids: Object.fromEntries(cells.map((id) => [id, !!q(id)])), after: Object.fromEntries(Object.values(after).map((id) => [id, !!q(id)])),
    playText: q("battle-play")?.textContent ?? "", endText: (q("end-why") ?? q("battle-end"))?.textContent ?? "", endOff: (q("end-why") ?? q("battle-end"))?.disabled ?? null,
    nextText: (q("battle-done") ?? q("chain-skip"))?.textContent ?? "",
    page: { sx: document.documentElement.scrollWidth - innerWidth, sy: document.documentElement.scrollHeight - innerHeight },
  };
}, { cells: CELLS, after: AFTER });

const same = (a, b, tol = 0.8) => !!a && !!b && ["l", "t", "r", "b"].every((k) => Math.abs(a[k] - b[k]) <= tol);
const within = (a, w, h, tol = 1) => a.l >= -tol && a.t >= -tol && a.r <= w + tol && a.b <= h + tol;
const hit = (a, b, tol = 0.5) => a.l < b.r - tol && a.r > b.l + tol && a.t < b.b - tol && a.b > b.t + tol;

const bad = [];
const browser = await launchChromium();
try {
  for (const scene of SCENES) for (const size of SIZES) for (const dsf of SCALES) {
    const [W, H] = size.split("x").map(Number);
    const what = `${scene} ${size}@${dsf}×`;
    const phone = W < 500;
    const ctx = await browser.newContext({ viewport: { width: W, height: H }, deviceScaleFactor: dsf, isMobile: phone, hasTouch: phone });
    const page = await ctx.newPage();
    page.on("pageerror", (e) => bad.push(`${what}: pageerror ${e.message}`));
    await page.goto(`${url}#scene=${scene}&lang=en`);
    await page.getByTestId("battle-play").waitFor({ timeout: 60_000 });
    await page.waitForTimeout(500);
    if ((await page.getByTestId("battle-play").textContent()).includes("❚")) await page.getByTestId("battle-play").click();
    await page.waitForTimeout(250);
    const before = await read(page);
    await page.keyboard.press("e");
    await page.getByTestId("end-card").waitFor({ timeout: 10_000 });
    await page.waitForTimeout(400);
    const s = await read(page);
    await page.screenshot({ path: `${out}/${scene}-${size}@${dsf}.png` });
    // The verdict: on screen, on the loser's row, clear of the tray, the winner's line, and every name and stat.
    if (!s.shown || !s.end) { bad.push(`${what}: no verdict card`); await ctx.close(); continue; }
    if (!within(s.end, s.vw, s.vh)) bad.push(`${what}: the verdict leaves the screen`);
    if (!s.word.trim() || !s.sub.trim()) bad.push(`${what}: the verdict has no word or no line under it`);
    if (hit(s.end, s.tray)) bad.push(`${what}: the verdict covers the tray`);
    if (s.winnerLine && hit(s.end, s.winnerLine)) bad.push(`${what}: the verdict covers the winner's line`);
    if (s.loserLine && !((s.end.t + s.end.b) / 2 >= s.loserLine.t - 20 && (s.end.t + s.end.b) / 2 <= s.loserLine.b)) bad.push(`${what}: the verdict isn't on the loser's row`);
    const covered = s.texts.filter((t) => hit(s.end, t));
    if (covered.length) bad.push(`${what}: the verdict covers ${covered.length} name or stat pill(s)`);
    // The tray: the same ladder and cells as before the end; the controls where they were.
    if (!same(s.ladder, before.ladder)) bad.push(`${what}: the ladder moved or changed size at the end`);
    s.rows.forEach((r, j) => { if (before.rows[j] && (Math.abs(r.h - before.rows[j].h) > 0.8 || Math.abs(r.t - before.rows[j].t) > 0.8)) bad.push(`${what}: ladder row ${j} moved or changed height at the end`); });
    if (s.rows.length !== 4) bad.push(`${what}: ${s.rows.length} ladder rows, four expected`);
    for (const id of CELLS) {
      if (!same(s.cells[id], before.cells[id])) bad.push(`${what}: ${id}'s cell moved at the end`);
      const c = s.cells[id];
      if (c && phone && (c.w < 43.5 || c.h < 43.5)) bad.push(`${what}: ${id}'s cell is ${Math.round(c.w)}×${Math.round(c.h)} px`);
      if (c && s.tray && !within({ l: c.l - s.tray.l, t: c.t - s.tray.t, r: c.r - s.tray.l, b: c.b - s.tray.t }, s.tray.w, s.tray.h)) bad.push(`${what}: ${id} leaves the tray`);
    }
    if (!s.after["end-why"] || !s.after["end-replay"] || !s.after["battle-done"]) bad.push(`${what}: Why, Replay and Next aren't all there (${JSON.stringify(s.after)})`);
    if (!s.ids["battle-play"] || s.playText.includes("❚")) bad.push(`${what}: Play isn't a Play button at the end`);
    if (!/next|дальше/i.test(s.nextText)) bad.push(`${what}: the Next cell reads "${s.nextText}"`);
    // Say why without opening anything: the causes as rows, and one sentence that names the same units and numbers.
    if (!s.sentence) bad.push(`${what}: no carry sentence`);
    if (s.ruleCut) bad.push(`${what}: the carry sentence is cut ("${s.sentence.slice(0, 60)}")`);
    if (s.sentence && s.rule && s.tray && !within({ l: s.rule.l - s.tray.l, t: s.rule.t - s.tray.t, r: s.rule.r - s.tray.l, b: s.rule.b - s.tray.t }, s.tray.w, s.tray.h)) bad.push(`${what}: the carry sentence leaves the tray`);
    if (!/(win|won|lost|ended|How it)/i.test(s.head)) bad.push(`${what}: the heading "${s.head}" doesn't say what the rows are`);
    for (const r of s.carry) {
      // The compact phone (a 360×640 window) keeps the ladder's 32 px rows (arena#52): its tray is 269 px tall.
      if (phone && r.h < (H < 700 ? 29.5 : 43.5)) bad.push(`${what}: a causal row is ${Math.round(r.h)} px tall`);
      if (r.cut) bad.push(`${what}: a causal row is cut ("${r.text}")`);
      const num = /\d+/.exec(r.text)?.[0];
      if (!num || !s.sentence.includes(num)) bad.push(`${what}: row "${r.text}" has a number the sentence doesn't ("${s.sentence}")`);
    }
    if (s.carry.length > 4) bad.push(`${what}: ${s.carry.length} causal rows`);
    if (!s.carry.length && !/fatigue|neither|Lost:|none/i.test(s.sentence)) bad.push(`${what}: no causal rows and no reason in the sentence ("${s.sentence}")`);
    if (s.page.sx > 1 || s.page.sy > 1) bad.push(`${what}: the page scrolls (${JSON.stringify(s.page)})`);

    // A causal row opens its moment: the chain, paused; Result brings the result back.
    if (s.carry.length) {
      await page.getByTestId("carry-row").first().click();
      await page.getByTestId("trace-text").waitFor({ timeout: 5_000 }).catch(() => bad.push(`${what}: a causal row opened no chain`));
      if ((await page.getByTestId("battle-play").textContent()) !== "▶") bad.push(`${what}: a causal row didn't pause`);
      await page.getByTestId("trace-result").click();
      await page.getByTestId("end-card").waitFor({ timeout: 5_000 }).catch(() => bad.push(`${what}: Result didn't bring the verdict back`));
    }
    // Why opens the chain that decided it, paused (a draw has none: Why is off).
    const why = page.getByTestId("end-why");
    if (await why.isEnabled()) {
      await why.click();
      await page.getByTestId("why-chain").waitFor({ timeout: 5_000 }).catch(() => bad.push(`${what}: Why opened no chain`));
      if ((await page.getByTestId("battle-play").textContent()) !== "▶") bad.push(`${what}: Why didn't land paused`);
      const steps = await page.getByTestId("why-step").count();
      if (!steps) bad.push(`${what}: Why's chain has no steps`);
      await page.getByTestId("trace-result").click();
      await page.getByTestId("end-card").waitFor({ timeout: 5_000 }).catch(() => bad.push(`${what}: Result didn't bring the verdict back after Why`));
      // Play on the result plays the deciding chain; End comes back to the result.
      await page.getByTestId("battle-play").click();
      if ((await page.getByTestId("battle-play").textContent()) !== "❚❚") bad.push(`${what}: Play on the result didn't play`);
      if (await page.getByTestId("end-card").isVisible()) bad.push(`${what}: Play left the verdict up`);
      await page.getByTestId("battle-play").click();
    } else if (scene !== "battle-chain-once") bad.push(`${what}: Why is off on a decided battle`);
    // Replay starts over from the line-up.
    await page.keyboard.press("e").catch(() => {});
    if (!(await page.getByTestId("end-card").isVisible().catch(() => false))) await page.keyboard.press("e");
    await page.getByTestId("end-card").waitFor({ timeout: 5_000 }).catch(() => bad.push(`${what}: End didn't come back to the result`));
    await page.getByTestId("end-replay").click();
    if (await page.getByTestId("end-card").isVisible()) bad.push(`${what}: Replay left the verdict up`);
    console.log(`${what}: ${s.carry.length} rows, "${s.sentence.slice(0, 70)}"`);
    await ctx.close();
  }
} finally {
  await browser.close();
  child?.kill();
}
if (bad.length) { console.log(bad.slice(0, 40).join("\n")); console.log(`FAIL: ${bad.length} problem(s)`); process.exit(1); }
console.log("battle-end: ok");
