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

/** A fresh page with the run injected before any script runs. */
export async function openRun(browser, serializedRun, viewport) {
  const ctx = await browser.newContext({ viewport, hasTouch: viewport.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(15_000);
  await page.addInitScript(
    ([key, val]) => localStorage.setItem(key, val),
    ["aoi.run.v1", serializedRun],
  );
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#run-shop:not([hidden])");
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
