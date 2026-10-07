// R4-10: the viewer on a line of 8 (both sides: Mortwrought's Ghouls, Imps and
// revives fill each line from 5 to 8; e2e/line-of-8-battle.ts). Plays a round
// on a LOCAL server with that battle swapped in, seeks to the turn where both
// lines hold 8, and checks on a phone and a desktop viewport that the page
// doesn't scroll sideways and every card sits on screen without overlapping
// its neighbours. R4-11: a line of 8 draws compact cards; each shows its
// emoji clear of its change chip, PWR and HP apart, at most one status row,
// the caption and the controls stay on screen, and a tap on a card opens it.
// Screenshots each viewport.
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
  for (const [label, viewport] of [["phone", { width: 360, height: 640 }], ["phone-390", { width: 390, height: 844 }], ["phone-side", { width: 640, height: 360 }], ["desktop-1024", { width: 1024, height: 768 }], ["desktop", { width: 1280, height: 800 }], ["desktop-1440", { width: 1440, height: 900 }]]) {
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
      const vh = document.documentElement.clientHeight;
      const r = (el) => el?.getBoundingClientRect();
      const apart = (a, b) => !a || !b || a.right <= b.left + 0.5 || b.right <= a.left + 0.5 || a.bottom <= b.top + 0.5 || b.bottom <= a.top + 0.5;
      // The compact card (R4-11): what each card shows, and what overlaps.
      const faults = [];
      for (const id of ["battle-you", "battle-them"]) {
        const line = document.querySelector(`[data-testid=${id}]`);
        if (!line.classList.contains("bv-compact")) faults.push(`${id} not compact`);
        for (const card of line.querySelectorAll(".bv-card:not(.dead)")) {
          const who = card.dataset.unit;
          const emoji = r(card.querySelector(".emoji"));
          const pill = r(card.querySelector(".bv-pill"));
          const p = r(card.querySelector(".stats.big .p"));
          const hp = r(card.querySelector(".stats.big .h"));
          const sts = r(card.querySelector(".bv-sts"));
          const box = r(card);
          if (!emoji || emoji.height < 10) faults.push(`${who}: no emoji`);
          if (!apart(emoji, pill)) faults.push(`${who}: chip over emoji`);
          if (!apart(p, hp)) faults.push(`${who}: PWR touches HP`);
          if (p.right > box.right + 0.5 || hp.right > box.right + 0.5 || p.left < box.left - 0.5) faults.push(`${who}: stats out of the card`);
          if (sts && sts.height > 17) faults.push(`${who}: ${Math.round(sts.height)}px status rows`);
          if (getComputedStyle(card.querySelector(".name")).display !== "none") faults.push(`${who}: name shown`);
        }
      }
      const cap = r(document.querySelector("[data-testid=caption]"));
      const ctl = r(document.querySelector(".bv-controls"));
      if (!cap || cap.bottom > vh + 0.5 || cap.top < 0) faults.push("caption off screen");
      if (!ctl || ctl.bottom > vh + 0.5) faults.push("controls off screen");
      return { vw, sw: document.documentElement.scrollWidth, you: side("battle-you"), them: side("battle-them"), playing: document.querySelector("[data-testid=battle-turn]")?.textContent, faults };
    });
    await page.screenshot({ path: out.replace(/\.png$/, `-${label}.png`) });
    // A tap low on a compact card (its stats, clear of the chip) opens the unit's Now sheet.
    let tapped = true;
    for (const id of ["battle-you", "battle-them"]) {
      const card = page.locator(`[data-testid=${id}] .bv-card:not(.dead)`).last();
      await card.locator(".stats.big").click();
      tapped &&= await page.getByTestId("now-sheet").isVisible().catch(() => false);
      await page.keyboard.press("Escape");
      await page.getByTestId("now-sheet").waitFor({ state: "detached", timeout: 3000 }).catch(() => { tapped = false; });
    }
    if (!tapped) m.faults.push("a card tap didn't open its Now sheet");
    const check = (cards) => cards.length === 8 && cards.every((c) => c.l >= -0.5 && c.r <= m.vw + 0.5 && c.w >= 30)
      && [...cards].sort((a, b) => a.l - b.l).every((c, i, s) => i === 0 || c.l >= s[i - 1].r - 0.5);
    const good = m.sw <= m.vw && check(m.you) && check(m.them) && errors.length === 0 && m.faults.length === 0;
    const widths = (cs) => cs.map((c) => Math.round(c.w)).join(",");
    console.log(`${label}: ${m.playing}; page ${m.sw}/${m.vw}px; you ${m.you.length} cards [${widths(m.you)}], them ${m.them.length} [${widths(m.them)}]${errors.length ? `; errors: ${errors.join(" | ")}` : ""}${m.faults.length ? `; ${m.faults.slice(0, 6).join(" | ")}` : ""} → ${good ? "ok" : "BROKEN"}`);
    if (!good) ok = false;
    await page.close();
  }
  console.log(ok ? "line of 8 probe: OK" : "line of 8 probe: FAILED");
  process.exitCode = ok ? 0 : 1;
  await browser.close();
} finally { child.kill(); }
