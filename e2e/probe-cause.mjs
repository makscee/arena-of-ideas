// A cause on every action (round 3, R3-19): plays a few rounds at 360×640
// and 1440×900, pauses each battle, steps through it with › and checks that
// every step shows a cause: the caption opens with the cause icon, and a
// badge sits on the acting unit (or, for fatigue, on the clash mark on
// desktop; the phone has fatigue in the caption only). It also checks that
// a battle card's icon line doesn't overlap its emoji, status row or HP.
// Screenshots go to --out. Needs a running MVP server (never the live one):
//   node e2e/probe-cause.mjs --url http://127.0.0.1:8913/arena/ [--out e2e/.shots/cause] [--rounds 3] [--steps 40]
import { mkdirSync } from "node:fs";
import { launchChromium } from "./browser.mjs";

const arg = (k, d) => { const i = process.argv.indexOf(k); return i > 0 ? process.argv[i + 1] : d; };
const url = arg("--url");
const out = arg("--out", "e2e/.shots/cause");
const rounds = Number(arg("--rounds", "3"));
const maxSteps = Number(arg("--steps", "40"));
const TAG = Date.now().toString(36).slice(-4);
if (!url) throw new Error("--url is required");
mkdirSync(out, { recursive: true });
const browser = await launchChromium();
const errors = [];
const kinds = {};
let strikesOnStriker = 0;

/** In the page: the step on screen, as the probe checks it. */
const readStep = () => {
  const cap = document.querySelector('[data-testid="caption"]');
  const cause = cap?.querySelector('[data-testid="caption-cause"]');
  const badges = [...document.querySelectorAll('[data-testid="trigger-badge"]')];
  const bright = badges.filter((b) => !b.classList.contains("past"));
  const r = (e) => e.getBoundingClientRect();
  const overlap = (a, b) => a.width && b.width && a.left < b.right - 0.5 && b.left < a.right - 0.5 && a.top < b.bottom - 0.5 && b.top < a.bottom - 0.5;
  // The icon line against the card's emoji, status row and stats.
  const clashes = [];
  for (const card of document.querySelectorAll(".bv-card:not(.dead)")) {
    const icons = [...card.querySelectorAll(".icons .ci")].filter((c) => getComputedStyle(c).display !== "none");
    for (const part of [".emoji", ".bv-sts", ".stats"]) {
      const p = card.querySelector(part);
      if (p && icons.some((c) => overlap(r(c), r(p)))) clashes.push(`${card.dataset.unit} icons over ${part}`);
    }
    // The icon line stays inside its card.
    const cr = r(card);
    if (icons.some((c) => r(c).right > cr.right + 0.5)) clashes.push(`${card.dataset.unit} icons past the card's edge`);
  }
  return {
    text: cap?.textContent ?? "",
    cause: cause?.getAttribute("data-cause") ?? null,
    badges: badges.map((b) => ({ cause: b.getAttribute("data-cause"), kind: b.getAttribute("data-kind"), past: b.classList.contains("past"), clash: !!b.closest(".bv-clash"), visible: r(b).width > 0, on: b.closest(".bv-slot")?.querySelector(".bv-card")?.getAttribute("data-unit") ?? null })),
    /** The first unit the caption names: a strike's striker. */
    named: cap?.querySelector("[data-unit]")?.getAttribute("data-unit") ?? null,
    bright: bright.length,
    fired: document.querySelectorAll(".bv-card .icons .ci.bv-fired").length,
    iconsShown: Math.max(0, ...[...document.querySelectorAll(".bv-card:not(.dead)")].map((c) => [...c.querySelectorAll(".icons .ci")].filter((x) => getComputedStyle(x).display !== "none").length)),
    clashes,
  };
};

for (const [label, viewport, mobile] of [["phone", { width: 360, height: 640 }, true], ["desktop", { width: 1440, height: 900 }, false]]) {
  const page = await browser.newPage({ viewport, deviceScaleFactor: 2, isMobile: mobile, hasTouch: mobile });
  page.on("pageerror", (e) => errors.push(`${label}: page error ${e.message}`));
  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").fill(`Cause${label === "phone" ? "P" : "D"}${TAG}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").click();
  let shots = 0;
  let maxIcons = 0;
  for (let round = 1; round <= rounds; round++) {
    await page.getByTestId("fight").waitFor({ timeout: 10_000 });
    for (let k = 0; k < 3; k++) {
      if ((await page.getByTestId("gold").count()) === 0) break;
      const gold = Number((await page.getByTestId("gold").textContent()).replace("g", ""));
      if (gold < 3 || (await page.getByTestId("line").locator(".card.you").count()) >= 5) break;
      await page.getByTestId("offer-0").click();
      if (await page.getByTestId("buy").isDisabled()) { await page.getByTestId("offer-close").click(); break; }
      await page.getByTestId("buy").click();
      await page.waitForFunction((g) => document.querySelector('[data-testid="gold"]')?.textContent !== `${g}g`, gold);
    }
    await page.getByTestId("fight").click();
    await page.getByTestId("battle-step").waitFor({ timeout: 10_000 });
    await page.getByTestId("battle-play").click();
    for (let s = 0; s < maxSteps; s++) {
      if (await page.getByTestId("battle-step").isDisabled()) break;
      await page.getByTestId("battle-step").click();
      await page.waitForTimeout(60);
      const st = await page.evaluate(readStep);
      if (/\b(win|wins)\b|Draw|lines face off/.test(st.text) && !st.cause) continue;
      maxIcons = Math.max(maxIcons, st.iconsShown);
      const where = `${label} r${round} step ${s}: "${st.text.slice(0, 60)}"`;
      if (!st.cause) errors.push(`${where}: the caption has no cause icon`);
      const fatigue = st.cause === "battle:fatigue" || st.cause === "battle:chainCapped";
      const clashBadge = st.badges.some((b) => b.clash && b.visible);
      // On the phone the clash mark is hidden: fatigue's cause is the caption's icon.
      if (st.bright !== 1 && !(fatigue && label === "phone")) errors.push(`${where}: ${st.bright} bright badges (want 1)`);
      if (fatigue && label === "desktop" && !clashBadge) errors.push(`${where}: fatigue without its badge on the clash mark`);
      for (const c of st.clashes) errors.push(`${where}: ${c}`);
      // A strike's badge sits on the striker, the unit its caption names first.
      const lit = st.badges.find((b) => !b.past);
      if (st.cause === "trigger:Strike" && /strikes/.test(st.text) && lit?.on !== st.named) errors.push(`${where}: the strike's badge is on ${lit?.on}, not the striker ${st.named}`);
      if (st.cause === "trigger:Strike" && /strikes/.test(st.text)) strikesOnStriker++;
      const k = st.cause ?? "none";
      kinds[k] = (kinds[k] ?? 0) + 1;
      if (shots < 14) await page.screenshot({ path: `${out}/${label}-r${round}-s${String(s).padStart(2, "0")}.png` }), shots++;
    }
    if (!(await page.getByTestId("battle-end").isDisabled())) await page.getByTestId("battle-end").click();
    await page.getByTestId("battle-done").waitFor({ timeout: 30_000 });
    await page.getByTestId("battle-done").click();
    await page.waitForFunction(() => document.querySelector('[data-testid="fight"]') || document.querySelector('[data-testid="run-over"]'));
    if (await page.getByTestId("run-over").isVisible().catch(() => false)) break;
  }
  console.log(`${label}: up to ${maxIcons} icons on a battle card`);
  await page.close();
}
await browser.close();
console.log("causes seen:", JSON.stringify(kinds));
if (!strikesOnStriker) errors.push("no strike step was seen");
if (errors.length) {
  console.log(`${errors.length} problems:\n${[...new Set(errors)].slice(0, 40).join("\n")}`);
  process.exitCode = 1;
} else console.log("every step showed its cause");
