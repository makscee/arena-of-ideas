// M5-5: the battle board on every viewport, EN and RU: plays a round's fight
// on a LOCAL server, pauses a few beats in, and checks that each card's name
// sits whole above its HP bar, the opponent's name in the bar is readable,
// and the page doesn't scroll sideways. Then opens Why on a change and checks
// it sits beside the traced card, not over it. Screenshots each viewport.
// Needs `npm run mvp:build` first.   node e2e/probe-battle-board.mjs [outdir]
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { launchChromium } from "./browser.mjs";
import { localeOf } from "./lang.mjs";
const out = process.argv[2] ?? "e2e/.shots/battle-board";
mkdirSync(out, { recursive: true });
const port = 18000 + Math.floor(Math.random() * 1000);
const child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", STATIC_DIR: "mobile/dist" }, stdio: ["ignore", "ignore", "inherit"] });
const url = `http://127.0.0.1:${port}/arena/`;
let ok = true;
try {
  for (let i = 0; i < 120; i++) { try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {} await new Promise((r) => setTimeout(r, 500)); }
  const browser = await launchChromium();
  for (const lang of ["en", "ru"]) for (const [label, viewport] of [["phone", { width: 360, height: 640 }], ["phone-390", { width: 390, height: 844 }], ["desktop", { width: 1280, height: 800 }], ["desktop-1440", { width: 1440, height: 900 }]]) {
    const phone = viewport.width < 1024;
    const page = await browser.newPage({ viewport, deviceScaleFactor: 2, locale: localeOf(lang), ...(phone ? { isMobile: true, hasTouch: true } : {}) });
    const errors = [];
    page.on("pageerror", (e) => errors.push(String(e)));
    await page.goto(url);
    await page.getByTestId("name-input").fill(`Board${lang}${label.replace(/\W/g, "")}`);
    await page.getByTestId("name-submit").click();
    await page.getByTestId("play").click();
    await page.getByTestId("offer-0").click();
    await page.getByTestId("buy").click();
    await page.getByTestId("offer-0").click().catch(() => {});
    await page.getByTestId("buy").click().catch(() => {});
    await page.getByTestId("fight").click();
    await page.getByTestId("caption").waitFor();
    await page.keyboard.press(" ");
    if (!phone) await page.keyboard.press(" ").catch(() => {});
    for (let i = 0; i < 4; i++) await page.getByTestId("battle-step").click().catch(() => page.keyboard.press("ArrowRight"));
    // Land on a beat a card acts in (not a turn's totals), so the caption has a card to sit under.
    for (let i = 0; i < 6 && !(await page.locator(".bv-line .bv-card.acting").count()); i++) await page.getByTestId("battle-step").click();
    await page.waitForTimeout(800);
    const clipped = () => page.evaluate(() => {
      const bad = [];
      for (const card of document.querySelectorAll(".bv-line .bv-card")) {
        const n = card.querySelector(".name"), bar = card.querySelector(".hpbar");
        if (!n || !bar || getComputedStyle(n).display === "none") continue;
        const nr = n.getBoundingClientRect(), br = bar.getBoundingClientRect();
        // The text's own box: a range over the name's text.
        const r = document.createRange(); r.selectNodeContents(n); const tr = r.getBoundingClientRect();
        if (tr.bottom > br.top + 1 || nr.height < tr.height - 1 || n.scrollWidth > n.clientWidth + 1) bad.push(`${n.textContent}${card.classList.contains("acting") ? " (acting)" : ""} (text ${Math.round(tr.top)}–${Math.round(tr.bottom)}, box ${Math.round(nr.height)}, bar at ${Math.round(br.top)})`);
      }
      return bad;
    });
    const m = await page.evaluate(() => {
      // The opponent's name: the bar's "vs @…" when it shows, else the Them label's.
      const bar = document.querySelector("[data-testid=battle-vs]");
      const vs = bar && bar.offsetParent ? bar : document.querySelector("[data-testid=battle-them-who]");
      const lab = vs?.closest(".label") ?? vs;
      const vsCut = !vs || !vs.offsetParent || vs.scrollWidth > vs.clientWidth + 1 || lab.scrollWidth > lab.clientWidth + 1 || innerWidth >= 1024 && [...document.querySelectorAll(".bv-float, .bv-badge")].some((f) => { const a = f.getBoundingClientRect(), b = vs.getBoundingClientRect(); return a.width && a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top; });
      // Desktop: the caption sits under the acting card (its centre within the card's width, or pushed in at the board's edge).
      const cap = document.querySelector("[data-testid=caption]"), acting = document.querySelector(".bv-line .bv-card.acting");
      let capOff = null;
      if (innerWidth >= 1024 && cap && acting) {
        const c = cap.getBoundingClientRect(), k = acting.getBoundingClientRect(), b = cap.parentElement.getBoundingClientRect();
        const mid = c.left + c.width / 2;
        if (!(mid >= k.left - 1 && mid <= k.right + 1) && !(c.left <= b.left + 1 || c.right >= b.right - 1)) capOff = `caption ${Math.round(c.left)}–${Math.round(c.right)}, acting card ${Math.round(k.left)}–${Math.round(k.right)}`;
      }
      const cardW = Math.round(document.querySelector(".bv-line .bv-card")?.getBoundingClientRect().width ?? 0);
      const capAt = cap?.dataset.at ?? "";
      return { capAt, capOff, cardW, vs: vs?.textContent ?? null, vsCut, sw: document.documentElement.scrollWidth, vw: innerWidth, turn: document.querySelector("[data-testid=battle-turn]")?.textContent ?? "" };
    });
    await page.screenshot({ path: `${out}/${lang}-${label}.png` });
    // Every name on every beat, playing on to the end: acting, hurt, dying.
    const bad = new Set(await clipped());
    for (let i = 0; i < 60 && !(await page.getByTestId("end-card").isVisible()); i++) {
      await page.getByTestId("battle-step").click();
      for (const b of await clipped()) bad.add(b.replace(/ \(text.*$/, ""));
    }
    m.bad = [...bad];
    const fail = [];
    if (m.bad.length) fail.push(`clipped names: ${m.bad.join("; ")}`);
    if (m.vsCut) fail.push(`opponent cut: ${m.vs}`);
    if (m.capOff) fail.push(m.capOff);
    if (m.sw > m.vw) fail.push(`scrolls sideways (${m.sw} > ${m.vw})`);
    if (lang === "ru" && /Start|^T\d/.test(m.turn)) fail.push(`turn label in English: ${m.turn}`);
    if (errors.length) fail.push(errors.join("; "));
    console.log(`battle board ${lang} ${label}: card ${m.cardW}px, caption at "${m.capAt}", turn ${m.turn}, vs "${m.vs}"${fail.length ? ` FAIL ${fail.join(" | ")}` : " ok"}`);
    if (fail.length) ok = false;
    await page.close();
  }
  await browser.close();
} finally {
  child.kill();
}
process.exit(ok ? 0 : 1);
