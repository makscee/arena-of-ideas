// R3-18 probe: the battle Now sheet on a chosen fight. Starts a local server
// (a temp DB; its bots fill it), picks a stored battle where one unit holds
// Shield and Poison at once and a unit is summoned, shows it in place of the
// first fight, and opens the Now sheet on both units at 1440×900 and 360×640.
//   node e2e/probe-now-sheet.mjs [--out dir]
// Exit 1 with the problems listed. Never point it at the live instance.
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import Database from "better-sqlite3";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const out = args.includes("--out") ? args[args.indexOf("--out") + 1] : "e2e/.shots/now-sheet";
mkdirSync(out, { recursive: true });
execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
const db = join(mkdtempSync(join(tmpdir(), "now-sheet-")), "mvp.db");
const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
const child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: db, STATIC_DIR: "mobile/dist" }, stdio: ["ignore", "ignore", "inherit"] });
const url = `http://127.0.0.1:${port}/arena/`;
for (let i = 0; i < 100; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

/** A battle with a summon and a unit holding Shield and Poison at once. */
function pickBattle() {
  const rows = new Database(db, { readonly: true }).prepare("SELECT json FROM mvp_battles").all();
  let summonOnly = null;
  for (const { json } of rows) {
    const b = JSON.parse(json);
    if (!b.log.some((e) => e.type === "Summon" && !e.resurrected)) continue;
    summonOnly ??= b;
    const held = new Map();
    for (const e of b.log) {
      if (e.type !== "StatusApplied" && e.type !== "StatusRemoved") continue;
      const m = held.get(e.unit) ?? {};
      m[e.status] = e.type === "StatusApplied" ? e.total : e.remaining;
      held.set(e.unit, m);
      if (m.Shield > 0 && m.Poison > 0) return { battle: b, both: e.unit };
    }
  }
  return summonOnly ? { battle: summonOnly, both: null } : null;
}
let pick = null;
for (let i = 0; i < 60 && !pick; i++) {
  pick = pickBattle();
  if (!pick) await new Promise((r) => setTimeout(r, 1000));
}

const errors = [];
const browser = await launchChromium();
try {
  if (!pick) throw new Error("no stored battle has a summon");
  console.log(`battle ${pick.battle.player?.name} v ${pick.battle.opponent?.name}: ${pick.both ? `${pick.both} holds Shield and Poison` : "no unit holds Shield and Poison at once"}`);
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  await page.route("**/api/v1/battles/*", (r) => r.fulfill({ json: pick.battle }));
  await page.goto(url);
  await page.getByTestId("name-input").waitFor({ timeout: 10_000 });
  await page.keyboard.type(`NowProbe${Date.now().toString(36).slice(-4)}`);
  await page.keyboard.press("Enter");
  await page.getByTestId("play").click();
  await page.getByTestId("fight").waitFor({ timeout: 10_000 });
  await page.keyboard.press("1");
  await page.getByTestId("fight").click();
  await page.getByTestId("battle-play").waitFor({ timeout: 10_000 });
  if ((await page.getByTestId("battle-play").textContent()) === "❚❚") await page.getByTestId("battle-play").click();
  for (let i = 0; i < 400 && !(await page.getByTestId("battle-back").isDisabled()); i++) await page.getByTestId("battle-back").click();

  const has = (sel) => page.locator(sel).count();
  const stepTo = async (sel) => {
    for (let i = 0; i < 400; i++) {
      if (await has(sel)) return true;
      if (await page.getByTestId("battle-step").isDisabled()) return false;
      await page.getByTestId("battle-step").click();
    }
    return false;
  };
  const openAndShoot = async (sel, name, check) => {
    for (const [w, hgt] of [[1440, 900], [360, 640]]) {
      await page.setViewportSize({ width: w, height: hgt });
      await page.waitForTimeout(150);
      const box = await page.locator(sel).first().boundingBox();
      await page.mouse.click(box.x + box.width / 2, box.y + box.height - 8);
      await page.getByTestId("now-sheet").waitFor({ timeout: 2_000 }).catch(() => errors.push(`${name} ${w}: no Now sheet`));
      if (!(await page.getByTestId("now-sheet").count())) continue;
      const text = await page.getByTestId("now-sheet").innerText();
      console.log(`--- ${name} ${w}×${hgt}\n${text}`);
      await check(text, `${name} ${w}`);
      await page.screenshot({ path: `${out}/${name}-${w}.png` });
      await page.keyboard.press("Escape");
      await page.getByTestId("now-sheet").waitFor({ state: "detached", timeout: 2_000 }).catch(() => errors.push(`${name} ${w}: Esc didn't close it`));
      if ((await page.getByTestId("battle-play").textContent()) !== "▶") errors.push(`${name} ${w}: not paused after Esc`);
    }
    await page.setViewportSize({ width: 1440, height: 900 });
  };

  if (pick.both) {
    // Chips can fold into "+n": read the sheet on each beat the card holds two or more.
    const sel = `.bv-card:not(.dead)[data-unit="${pick.both}"]`;
    let found = false;
    for (let i = 0; i < 400 && !found; i++) {
      const card = page.locator(sel);
      if ((await card.count()) && Number(await card.getByTestId("card-statuses").getAttribute("data-count")) >= 2) {
        const box = await card.boundingBox();
        await page.mouse.click(box.x + box.width / 2, box.y + box.height - 8);
        await page.getByTestId("now-sheet").waitFor({ timeout: 2_000 });
        const st = await page.getByTestId("live-status").evaluateAll((els) => els.map((e) => e.dataset.status));
        found = st.includes("Shield") && st.includes("Poison");
        await page.keyboard.press("Escape");
      }
      if (found || (await page.getByTestId("battle-step").isDisabled())) break;
      await page.getByTestId("battle-step").click();
    }
    if (!found) errors.push(`never reached a beat where ${pick.both} holds Shield and Poison`);
    else await openAndShoot(sel, "shield-poison", (t, n) => {
      for (const s of ["Shield ×", "Poison ×"]) if (!t.includes(s)) errors.push(`${n}: no "${s}" in the sheet`);
      if (!/HP \d+ \/ \d+/.test(t)) errors.push(`${n}: no current HP`);
    });
  }
  for (let i = 0; i < 400 && !(await page.getByTestId("battle-back").isDisabled()); i++) await page.getByTestId("battle-back").click();
  const summon = '.bv-card:not(.dead)[data-unit*="+"]';
  if (!(await stepTo(summon))) errors.push("never reached a summoned card");
  else await openAndShoot(summon, "summon", (t, n) => {
    if (!/No ability: it fights with its PWR \/ HP\.|:/.test(t)) errors.push(`${n}: no ability line for the summon`);
    if (!/Entered as \d+ PWR \/ \d+ HP · summoned/.test(t)) errors.push(`${n}: no "Entered as … · summoned" line`);
    if (/Full card/.test(t)) errors.push(`${n}: a summon offers a full card it doesn't have`);
  });
} catch (e) {
  errors.push(String(e?.stack ?? e));
} finally {
  await browser.close();
  child.kill();
}
if (errors.length) {
  console.log(`now-sheet probe: ${errors.length} problems\n- ${errors.join("\n- ")}`);
  process.exit(1);
}
console.log(`now-sheet probe: ok, shots in ${out}`);
