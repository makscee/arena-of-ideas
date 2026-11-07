// The shop's fixed places (arena#51, mission #44 move 3 and 6): at 800×450 and 390×844 (and the compact phone window, 390×700),
// on scenes: five line frames and three bench frames, six offer wells, the opponent rail ("?" over the line and "Team hidden until
// battle", never a unit of the next opponent), labels with occupancy, Cards and the full round track (twelve rounds and the Crown) in the HUD;
// nothing above the tray moves on any tap (offers, units, Lock, Esc); no page scroll, no text under 11 px at 800×450, the words
// fit their boxes (RU too). Then Home to the first round: the champion's frames, your five slots and PLAY sit where the
// shop's line, bench and Fight are. Needs a scenes server (arena-lab.sh or ARENA_SCENES=1).
//   node e2e/places.mjs --url http://127.0.0.1:PORT/arena/ [--out dir] [--shots] [--scene shop-r3,shop-r8,shop-bench-full] [--lang en,ru]
import { launchChromium } from "./browser.mjs";
import { mkdirSync } from "node:fs";

const args = process.argv.slice(2);
const opt = (n, d) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : d; };
const url = opt("url");
if (!url) { console.error("usage: node e2e/places.mjs --url http://127.0.0.1:PORT/arena/"); process.exit(2); }
const scenes = opt("scene", "shop-r3,shop-r8,shop-bench-full").split(",");
const langs = opt("lang", "en,ru").split(",");
const out = opt("out", "e2e/.shots/places");
const shots = args.includes("--shots");
if (shots) mkdirSync(out, { recursive: true });
const SIZES = [
  { name: "800x450", viewport: { width: 800, height: 450 } },
  { name: "390x844", viewport: { width: 390, height: 844 } },
  { name: "390x700", viewport: { width: 390, height: 700 } },
];
const errors = [];
const r1 = (n) => Math.round(n * 10) / 10;

/** Every box that is a place, in stage pixels. */
const places = (page) =>
  page.evaluate(() => {
    const st = document.querySelector(".stage");
    const k = parseFloat(st.style.transform.replace(/[^0-9.]/g, "")) || 1;
    const s = st.getBoundingClientRect();
    const box = (e) => { const b = e.getBoundingClientRect(); return [(b.x - s.x) / k, (b.y - s.y) / k, b.width / k, b.height / k].map((n) => Math.round(n * 10) / 10); };
    const all = (sel) => [...document.querySelectorAll(sel)];
    const one = (id) => document.querySelector(`[data-testid="${id}"]`);
    return {
      k,
      line: all('[data-testid="line"] > .cell').map(box),
      bench: all('[data-testid="bench"] > .cell').map(box),
      wells: all('[data-testid="offers"] > .cell').map(box),
      marks: all('[data-testid^="their-"]').filter((e) => /^their-\d$/.test(e.dataset.testid)).map(box),
      still: Object.fromEntries(["tray", "reroll", "fight", "inspector", "line-label", "bench-label", "team-hidden", "next-opponent", "legend-open", "round", "gold"].map((id) => [id, one(id) ? box(one(id)) : null])),
      track: all(".track .track-dot").length,
      theirPieces: all('[data-testid="their-line"] .pc').length,
      hidden: one("team-hidden")?.textContent?.trim() ?? null,
      labels: { line: one("line-label")?.textContent?.replace(/\s+/g, " ").trim() ?? null, bench: one("bench-label")?.textContent?.replace(/\s+/g, " ").trim() ?? null },
      cards: one("legend-open")?.textContent?.trim() ?? null,
      scroll: [document.scrollingElement.scrollHeight - document.scrollingElement.clientHeight, document.scrollingElement.scrollWidth - document.scrollingElement.clientWidth],
      compact: st.classList.contains("compact"),
    };
  });

/** Two boxes, or two lists of boxes, are the same place: every number within `tol` (a tapped offer or the coach's pulse scales for a moment). */
const same = (a, b, what, at, tol = 0.6) => {
  if (!a || !b) return errors.push(`${at}: ${what} is missing`);
  const flat = (x) => (Array.isArray(x[0]) ? x.flat() : x);
  const [fa, fb] = [flat(a), flat(b)];
  if (fa.length !== fb.length || fa.some((n, i) => Math.abs(n - fb[i]) > tol)) errors.push(`${at}: ${what} moved (${JSON.stringify(a)} → ${JSON.stringify(b)})`);
};

const browser = await launchChromium();
try {
  for (const v of SIZES) {
    // Reduced motion: a pulsing PLAY or a tapped offer scales for a moment, which moves its box without moving its place.
    const ctx = await browser.newContext({ viewport: v.viewport, isMobile: true, hasTouch: true, deviceScaleFactor: 2, reducedMotion: "reduce" });
    const page = await ctx.newPage();
    page.on("pageerror", (e) => errors.push(`${v.name}: page error ${e.message}`));
    for (const [scene, lang] of scenes.flatMap((sc) => langs.map((l) => [sc, l]))) {
      const at = `${v.name} ${scene} ${lang}`;
      await page.goto("about:blank");
      await page.goto(`${url}#scene=${scene}&lang=${lang}`, { timeout: 120_000 });
      await page.waitForSelector('[data-testid="fight"]', { timeout: 60_000 });
      await page.waitForTimeout(1200);
      const idle = await places(page);
      if (shots) await page.screenshot({ path: `${out}/${scene}-${lang}-${v.name}.png` });
      // The places: five and three frames, six wells, five marks, the rail's words, Cards, the whole track.
      if (idle.line.length !== 5) errors.push(`${at}: ${idle.line.length} line frames, not 5`);
      if (idle.bench.length < 3) errors.push(`${at}: ${idle.bench.length} bench frames, not 3`);
      if (idle.wells.length !== 6) errors.push(`${at}: ${idle.wells.length} offer wells, not 6`);
      if (idle.marks.length !== 5) errors.push(`${at}: ${idle.marks.length} "?" marks, not 5`);
      if (idle.theirPieces) errors.push(`${at}: the next opponent's team is drawn (${idle.theirPieces} pieces)`);
      if (!idle.hidden) errors.push(`${at}: no "Team hidden until battle" line`);
      if (lang === "en" && idle.hidden !== "Team hidden until battle") errors.push(`${at}: the rail says ${JSON.stringify(idle.hidden)}`);
      if (idle.track !== 13) errors.push(`${at}: the round track has ${idle.track} dots, not 13 (twelve rounds and the Crown)`);
      if (!idle.cards) errors.push(`${at}: Cards is not in the HUD`);
      if (idle.scroll.some((n) => n > 0)) errors.push(`${at}: the page scrolls (${idle.scroll})`);
      if (!/\d+\s*\/\s*5/.test(idle.labels.line ?? "")) errors.push(`${at}: the line label has no occupancy (${JSON.stringify(idle.labels.line)})`);
      if (!/\d\s*\/\s*3/.test(idle.labels.bench ?? "")) errors.push(`${at}: the bench label has no occupancy (${JSON.stringify(idle.labels.bench)})`);
      // The frames are one grid: the same width and height, the line's columns one pitch apart, the wells one row (landscape and the compact phone) or three by two.
      for (const [n, list] of [["line", idle.line], ["bench", idle.bench], ["wells", idle.wells]]) {
        if (list.length && list.some((b) => Math.abs(b[2] - list[0][2]) > 0.6 || Math.abs(b[3] - list[0][3]) > 0.6)) errors.push(`${at}: the ${n} frames differ in size (${JSON.stringify(list)})`);
      }
      for (let i = 1; i < idle.line.length; i++) if (Math.abs(idle.line[i][0] - idle.line[i - 1][0] - (idle.line[1][0] - idle.line[0][0])) > 0.6) errors.push(`${at}: the line frames are not one pitch apart`);
      // Over the tray: the rail over the line columns, the line over the bench, no frame under the tray.
      const tray = idle.still.tray;
      for (const [n, list] of [["line", idle.line], ["bench", idle.bench]]) for (const b of list) if (tray && b[1] + b[3] > tray[1] + 0.5) errors.push(`${at}: a ${n} frame reaches into the tray`);
      if (idle.line.length && idle.bench.length && idle.bench[0][1] < idle.line[0][1] + idle.line[0][3] - 0.5) errors.push(`${at}: the bench overlaps the line`);
      // Taps: nothing above the tray (nor the tray's other parts) moves.
      const targets = await page.evaluate(() => [...document.querySelectorAll('[data-testid^="offer-"]:not([data-testid^="offer-gap"]), [data-testid^="line-"]:not([data-testid$="-empty"]):not([data-testid="line-label"]), [data-testid^="bench-"]:not([data-testid$="-empty"]):not([data-testid="bench-label"])')].map((e) => e.getAttribute("data-testid")));
      for (const id of targets) {
        await page.getByTestId(id).click();
        await page.waitForTimeout(300);
        const now = await places(page);
        for (const key of ["line", "bench", "wells", "marks"]) same(idle[key], now[key], key, `${at}: tapping ${id}`, key === "wells" ? 1.5 : 0.6);
        for (const key of ["tray", "reroll", "fight", "inspector", "line-label", "bench-label", "team-hidden", "next-opponent", "legend-open", "round", "gold"]) same(idle.still[key], now.still[key], key, `${at}: tapping ${id}`);
      }
      await page.keyboard.press("Escape");
      // No text under 11 px as drawn at 800×450, and the words fit: a box whose text runs past it.
      const bad = await page.evaluate(() => {
        const k = parseFloat(document.querySelector(".stage").style.transform.replace(/[^0-9.]/g, "")) || 1;
        const out = [];
        for (const e of document.querySelectorAll(".stage *")) {
          if (e.children.length || !e.textContent.trim() || e.closest("[aria-hidden='true']") || e.closest("svg")) continue;
          const r = e.getBoundingClientRect();
          if (!r.width) continue;
          const px = parseFloat(getComputedStyle(e).fontSize) * k;
          if (innerWidth >= 700 && px < 10.95) out.push(`${e.textContent.trim().slice(0, 18)} (${px.toFixed(1)} px)`);
        }
        return out;
      });
      if (bad.length) errors.push(`${at}: text under 11 px: ${bad.slice(0, 6).join(", ")}`);
      // The words fit: nothing inside Reroll, Fight, Cards, the rail's or the labels' boxes leaves its box (RU too).
      const spill = await page.evaluate(() => {
        const out = [];
        for (const id of ["reroll", "fight", "legend-open", "line-label", "bench-label", "team-hidden", "next-opponent"]) {
          const root = document.querySelector(`[data-testid="${id}"]`);
          if (!root) continue;
          const r = root.getBoundingClientRect();
          const box = id === "team-hidden" || id === "next-opponent" ? root.parentElement.getBoundingClientRect() : r;
          for (const e of root.querySelectorAll("*")) {
            const b = e.getBoundingClientRect();
            if (b.width && (b.left < box.left - 1 || b.right > box.right + 1 || b.top < box.top - 1 || b.bottom > box.bottom + 1)) out.push(`${id}: ${e.textContent.trim().slice(0, 16)}`);
          }
        }
        return out;
      });
      if (spill.length) errors.push(`${at}: words leave their box: ${spill.join(", ")}`);
    }
    // Home to the first round: the same places.
    for (const lang of langs) {
      const at = `${v.name} home ${lang}`;
      await page.goto("about:blank");
      await page.goto(`${url}#scene=home&lang=${lang}`, { timeout: 120_000 });
      await page.waitForSelector('[data-testid="play"]', { timeout: 60_000 });
      await page.waitForTimeout(1200);
      const home = await page.evaluate(() => {
        const st = document.querySelector(".stage");
        const k = parseFloat(st.style.transform.replace(/[^0-9.]/g, "")) || 1;
        const s = st.getBoundingClientRect();
        const box = (e) => { const b = e.getBoundingClientRect(); return [(b.x - s.x) / k, (b.y - s.y) / k, b.width / k, b.height / k].map((n) => Math.round(n * 10) / 10); };
        const one = (id) => document.querySelector(`[data-testid="${id}"]`);
        const text = document.body.innerText;
        return {
          theirs: [...document.querySelectorAll('[data-testid="home-their"] > .hm-cell')].map(box),
          slots: [0, 1, 2, 3, 4].map((i) => (one(`home-slot-${i}`) ? box(one(`home-slot-${i}`)) : null)),
          tray: box(one("home-tray")),
          play: box(one("play")),
          playCss: getComputedStyle(one("play")).transform + " " + getComputedStyle(one("play")).animationName,
          fiction: /\b(R\d+\/12|Round \d|10 ?g)\b/.test(text) || !!document.querySelector(".heart-icons, .track, [data-testid='gold']"),
          scroll: [document.scrollingElement.scrollHeight - document.scrollingElement.clientHeight, document.scrollingElement.scrollWidth - document.scrollingElement.clientWidth],
        };
      });
      if (process.env.DEBUG_PLAY) console.log(at, JSON.stringify(home.play), home.playCss);
      if (shots) await page.screenshot({ path: `${out}/home-${lang}-${v.name}.png` });
      if (home.fiction) errors.push(`${at}: Home shows a round, hearts or gold before a run`);
      if (home.scroll.some((n) => n > 0)) errors.push(`${at}: Home scrolls (${home.scroll})`);
      await page.getByTestId("play").click();
      await page.waitForSelector('[data-testid="fight"]', { timeout: 60_000 });
      await page.waitForTimeout(1500);
      const shop = await places(page);
      const shopBox = await page.evaluate(() => {
        const st = document.querySelector(".stage");
        const k = parseFloat(st.style.transform.replace(/[^0-9.]/g, "")) || 1;
        const s = st.getBoundingClientRect();
        const b = document.querySelector('[data-testid="fight"]').getBoundingClientRect();
        return [(b.x - s.x) / k, (b.y - s.y) / k, b.width / k, b.height / k].map((n) => Math.round(n * 10) / 10);
      });
      // The champion's frames are the line's; your five slots start where the bench's three do; PLAY is Fight's box; the tray is the tray.
      home.theirs.forEach((b, i) => shop.line[i] && same(shop.line[i], b, `the champion's frame ${i + 1} against line slot ${i + 1}`, at));
      for (let i = 0; i < 3; i++) same(shop.bench[i], home.slots[i], `your slot ${i + 1} against bench slot ${i + 1}`, at);
      same(shop.still.tray, home.tray, "the tray", at);
      // Fight pulses on a first run (the coach points at it): its box is within the pulse of PLAY's.
      same(shopBox, home.play, "PLAY against Fight", at, 20);
    }
    await ctx.close();
  }
} finally {
  await browser.close();
}
if (errors.length) {
  console.error(`${errors.length} problem(s):\n${errors.join("\n")}`);
  process.exit(1);
}
console.log("places: fixed frames, six wells, the rail, Cards and the track stay put at 800×450, 390×844 and 390×700; Home to the first round keeps the same places");
