// Shared e2e plumbing: a hard timeout guard, a tiny assertion collector, and
// kernel-built RunStates injected via localStorage. The states are hand-shaped
// (deserializeRun checks structure + pool validity, not log consistency), but
// every string the probes measure — fusion notice, full-line error — is
// produced by the LIVE kernel/run-screen at click time, never by the probe.
// Run with `node --import tsx/esm` so the kernel imports resolve.

import { chromium } from "playwright";
import {
  DEFAULT_RUN_POOL,
  STACK_THRESHOLD,
  UNIT_COST,
  initRun,
  serializeRun,
  stressRegistry,
} from "../src/index.js";

export const BASE = process.env.AOI_BASE_URL ?? "http://localhost:5280";
export const PHONE = { width: 375, height: 667 };
export const DESKTOP = { width: 1280, height: 800 };

/** Kill the probe outright if it wedges — no probe may outlive its budget. */
export function armGuard(ms = 100_000) {
  const t = setTimeout(() => {
    console.error(`PROBE TIMEOUT after ${ms}ms`);
    process.exit(2);
  }, ms);
  return () => clearTimeout(t);
}

// ---------- assertion collector ----------

const failures = [];

export function check(ok, label, detail = "") {
  const line = `${ok ? "ok  " : "FAIL"} ${label}${detail === "" ? "" : ` — ${detail}`}`;
  console.log(line);
  if (!ok) failures.push(line);
}

export function finish(name) {
  if (failures.length > 0) {
    console.error(`\n${name}: ${failures.length} failure(s)`);
    process.exit(1);
  }
  console.log(`\n${name}: all checks passed`);
}

// ---------- injected run states ----------

const byName = Object.fromEntries(DEFAULT_RUN_POOL.map((d) => [d.name, d]));

/** A line unit built from a pool def, the shape buy() itself appends. */
const unitOf = (def, stacks = 1, level = 1) => ({
  name: def.name,
  base: { ...def.base },
  level,
  stacks,
  def,
});

/** A real initRun state, then hand-shaped team/offers/gold for the scenario. */
function shaped(mutate, pool = DEFAULT_RUN_POOL) {
  const s = initRun({ seed: 7, pool, statuses: stressRegistry });
  mutate(s);
  return serializeRun(s);
}

/** Buying offer 0 (Necromancer, the pool's longest name) fuses: the team copy
 * sits one stack short of STACK_THRESHOLD. Three offers = the wrapped shop. */
export const fuseReadyRun = () =>
  shaped((s) => {
    s.team = [unitOf(byName.Necromancer, STACK_THRESHOLD - 1), unitOf(byName.Brawler), unitOf(byName.Squire)];
    s.offers = [byName.Necromancer, byName.Summoner, byName.Bulwark];
    s.gold = 10;
  });

/** Full line of five distinct units; offer 0 is a sixth distinct name, so
 * buying it raises the real "line is full" error, verbatim from the kernel. */
export const lineFullRun = () =>
  shaped((s) => {
    s.team = [
      unitOf(byName.Venomancer),
      unitOf(byName.Summoner),
      unitOf(byName.Silencer),
      unitOf(byName.Necromancer),
      unitOf(byName.Brawler),
    ];
    s.offers = [byName.Bulwark, byName.Squire, byName.Venomancer];
    s.gold = 10;
  });

/** Two distinct units + three distinct offers + gold for exactly three buys:
 * the slice-3 close line-growth scenario — each buy adds a NEW name, so the
 * line wraps rows 2→3 and 4→5; the reachable-count reserve must already hold
 * every row a buy can reach. */
export const lineGrowthRun = () =>
  shaped((s) => {
    s.team = [unitOf(byName.Brawler), unitOf(byName.Squire)];
    s.offers = [byName.Venomancer, byName.Summoner, byName.Bulwark];
    s.gold = 3 * UNIT_COST;
  });

/** Two units, gold below UNIT_COST (slice-3 close): the reachable count is
 * just the team — the reserve must not burn rows gold cannot fill. Lives are
 * topped up so the round-1 fight always continues into round 2's shop. */
export const poorGoldRun = () =>
  shaped((s) => {
    s.team = [unitOf(byName.Brawler), unitOf(byName.Squire)];
    s.offers = [byName.Venomancer, byName.Summoner, byName.Bulwark];
    s.gold = UNIT_COST - 1;
    s.lives = 5;
  });

/** Plain 3-offer shop, nothing fuses: the buy → collapse and reroll probes. */
export const plainShopRun = () =>
  shaped((s) => {
    s.team = [unitOf(byName.Brawler)];
    s.offers = [byName.Venomancer, byName.Summoner, byName.Bulwark];
    s.gold = 10;
  });

/** A finished run (#014): status "over", out of lives — the run-end screen,
 * where the menu must still appear and abandon must still work. A small team
 * so the end screen renders a couple of post-mortem cards. */
export const endedRun = () =>
  shaped((s) => {
    s.team = [unitOf(byName.Brawler), unitOf(byName.Squire)];
    s.offers = [];
    s.gold = 0;
    s.lives = 0;
    s.status = "over";
    s.endedBy = "out-of-lives";
  });

/** Three line units (middle card shows both arrows enabled, last card has ▸
 * disabled); the front card carries three adjacent chips for the chip sweep. */
export const targetsRun = () => {
  const Mosaic = {
    name: "Mosaic",
    base: { hp: 9, pwr: 2 },
    statuses: [
      { status: "Poison", stacks: 2 },
      { status: "Shield", stacks: 1 },
      { status: "Vitality", stacks: 2 },
    ],
  };
  return shaped(
    (s) => {
      s.team = [unitOf(Mosaic), unitOf(byName.Brawler), unitOf(byName.Squire)];
      s.offers = [byName.Venomancer, byName.Summoner, byName.Bulwark];
      s.gold = 10;
    },
    [...DEFAULT_RUN_POOL, Mosaic],
  );
};

/** Constructed content (nothing shipped exercises these paths): a when clause
 * naming a status and an explicit-status consumeStacks — both must surface as
 * tappable refs in the live inspector. */
export const refsRun = () => {
  const Warden = {
    name: "Warden",
    base: { hp: 8, pwr: 2 },
    abilities: [
      {
        whens: [{ kind: "trigger", on: { on: "StatusApplied", unit: "ally", status: "Poison" } }],
        selectors: [{ kind: "holder" }],
        effects: [{ kind: "heal", amount: { kind: "const", value: 2 } }],
      },
      {
        whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
        selectors: [{ kind: "frontEnemy" }],
        effects: [
          { kind: "consumeStacks", status: "Shield", stacks: { kind: "const", value: 2 } },
          { kind: "damage", amount: { kind: "const", value: 3 } },
        ],
      },
    ],
  };
  return shaped(
    (s) => {
      s.team = [unitOf(Warden), unitOf(byName.Brawler)];
      s.offers = [byName.Venomancer, byName.Summoner, byName.Bulwark];
      s.gold = 10;
    },
    [...DEFAULT_RUN_POOL, Warden],
  );
};

// ---------- browser ----------

/** A fresh page with the run injected before any script runs. The app lands
 * on the title screen (#015 slice 3) where an active run reads "Continue run"
 * — this resumes through that entry, exactly like a returning player. `ready`
 * is the selector to wait on (the shop panel by default; pass
 * "#run-end:not([hidden])" for an injected finished run). */
export async function openRun(browser, serializedRun, viewport, ready = "#run-shop:not([hidden])") {
  const ctx = await browser.newContext({ viewport, hasTouch: viewport.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.addInitScript(
    ([key, val]) => localStorage.setItem(key, val),
    ["aoi.run.v1", serializedRun],
  );
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  await page.click("#title-play");
  await page.waitForSelector(ready);
  return { ctx, page };
}

export async function launch() {
  return chromium.launch();
}

/** Rounded bounding box of a selector (null-safe: fails the probe loudly). */
export async function box(page, selector) {
  const b = await page.locator(selector).boundingBox();
  if (b === null) throw new Error(`no box for ${selector}`);
  return b;
}

/** Sweep `targetSel`'s surface with a 5px grid of elementFromPoint probes and
 * count how many resolve to ANYTHING other than the target (or its own
 * descendants/ancestors). { stolen, total, thieves } — stolen > 0 means some
 * element occludes the target and will intercept taps.
 *
 * Cass #015-slice-4 carry, fixed here: the target is scrolled into view FIRST
 * and the grid uses fresh client-rect coords, clamped to the viewport —
 * elementFromPoint is viewport-relative, so a below-the-fold target used to
 * yield null at every point and count as "not stolen" (a vacuous pass). A
 * sweep that still lands zero in-viewport points throws rather than passing;
 * an in-viewport null (nothing hittable at a visible point) counts stolen.
 * Ancestors are allowed — a rounded corner inside the bounding box resolves
 * to the parent, which is layout, not occlusion. */
export async function sweepOcclusion(page, targetSel) {
  const target = page.locator(targetSel);
  if ((await target.count()) === 0 || !(await target.isVisible())) {
    return { stolen: 0, total: 0, thieves: [] }; // element not on screen — caller's call
  }
  await target.scrollIntoViewIfNeeded();
  return page.evaluate((sel) => {
    const el = document.querySelector(sel);
    const r = el.getBoundingClientRect();
    const x0 = Math.max(r.left + 2, 0);
    const x1 = Math.min(r.right - 2, window.innerWidth - 1);
    const y0 = Math.max(r.top + 2, 0);
    const y1 = Math.min(r.bottom - 2, window.innerHeight - 1);
    const step = 5;
    let stolen = 0;
    let total = 0;
    const thieves = new Set();
    for (let x = x0; x < x1; x += step) {
      for (let y = y0; y < y1; y += step) {
        total++;
        const hit = document.elementFromPoint(x, y);
        if (hit === null || (hit !== el && !el.contains(hit) && !hit.contains(el))) {
          stolen++;
          thieves.add(
            hit === null
              ? "(null)"
              : hit.id !== ""
                ? `#${hit.id}`
                : hit.tagName.toLowerCase() + (hit.className ? `.${hit.className}` : ""),
          );
        }
      }
    }
    if (total === 0) throw new Error(`occlusion sweep of ${sel} is vacuous — no in-viewport points`);
    return { stolen, total, thieves: [...thieves] };
  }, targetSel);
}
