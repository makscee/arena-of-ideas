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

// ---------- signal-safe browser teardown ----------
//
// Any script that opens a browser (every probe, shots.mjs) must close it on
// EVERY exit path, not just the happy one. A timed-out or Ctrl-C'd probe used
// to `process.exit()` straight past `browser.close()`, stranding a chromium
// (and its renderer children) that reparents to init and pins CPU. We track
// every launched browser and tear them all down from a single guarded path,
// wired to the timeout guard AND to process signals/crashes.

const browsers = new Set();
let tearingDown = false;

/** Best-effort synchronous-ish close of every tracked browser, then exit.
 * Guarded so re-entry (e.g. a signal during teardown) is a no-op. `code` is
 * the conventional exit code for the path that called us. */
async function teardownBrowsers(code) {
  if (tearingDown) return;
  tearingDown = true;
  // Race each close against a short grace — a wedged browser must not hang the
  // exit; the process leaving will orphan it, but `kill()` below also fires.
  await Promise.all(
    [...browsers].map((b) =>
      Promise.race([
        b.close().catch(() => {}),
        new Promise((r) => setTimeout(r, 3000)),
      ]).then(() => {
        // If close() stalled, kill the browser process group outright.
        try {
          const proc = b.process?.();
          if (proc?.pid) process.kill(proc.pid, "SIGKILL");
        } catch {}
      }),
    ),
  );
  process.exit(code);
}

let signalsArmed = false;
/** Install once-per-process handlers so a Ctrl-C/SIGTERM/crash still reaps the
 * browser. Idempotent — armGuard and launch both call it; only the first wins. */
function armSignalTeardown() {
  if (signalsArmed) return;
  signalsArmed = true;
  for (const sig of ["SIGINT", "SIGTERM"]) {
    process.on(sig, () => {
      const n = { SIGINT: 2, SIGTERM: 15 }[sig];
      teardownBrowsers(128 + n);
    });
  }
  process.on("uncaughtException", (err) => {
    console.error("uncaughtException:", err);
    teardownBrowsers(1);
  });
  process.on("unhandledRejection", (err) => {
    console.error("unhandledRejection:", err);
    teardownBrowsers(1);
  });
}

/** Kill the probe outright if it wedges — no probe may outlive its budget.
 * Closes any open browser first so a timed-out probe leaks nothing. */
export function armGuard(ms = 100_000) {
  armSignalTeardown();
  const t = setTimeout(() => {
    console.error(`PROBE TIMEOUT after ${ms}ms`);
    teardownBrowsers(2);
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

/** A FULL five-unit line vs the round-1 bootstrap opponent (#065 slice-1
 * regression): an ASYMMETRIC, near-max matchup so the desktop "equal halves"
 * crush reproduces. The injected run fields all TEAM_SIZE units; the ladder
 * draw seats the bootstrap round-1 enemy (two/three bodies) on side B, so the
 * battle opens 5 (side A) vs a smaller side B — the case that crushed B to a
 * sliver under `flex: 1 1 0`. The names are five distinct pool units so each
 * card renders its own art + stats (no stacking). */
export const bigBattleRun = () =>
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
    s.lives = 5;
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

/** Constructed content (#065 slice 2): a Duelist whose Strike trigger ALSO
 * deals direct damage to the front enemy — so a single Strike beat lands TWO
 * Hurts on the same defender. The injected fight reaches that beat, and the
 * hero-overlay probe asserts the two hits SUM into one red −N badge (the
 * "summed on repeated hits within the beat" clause). A tanky defender survives
 * both hits so the badge shows the live-incrementing total, not a death. */
export const duelistRun = () => {
  // pwr 1 + a +1 bonus to the front enemy: both hits are small, so the struck
  // enemy survives BOTH within the one Strike beat — its two Hurt lines stay on
  // the board and its red badge increments 1 → 2 (the live-incrementing sum).
  // A tanky body sits IN FRONT of the Duelist so the bootstrap Silencer's
  // BattleStart silence lands on the (ability-less) tank, never on the Duelist
  // — a silenced Duelist would lose its bonus-damage ability and the beat would
  // carry a single hit. The tank trades down, the Duelist reaches the front
  // with its ability intact, and its strikes then land the double hit.
  const Duelist = {
    name: "Duelist",
    base: { hp: 8, pwr: 1 },
    abilities: [
      {
        whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
        selectors: [{ kind: "frontEnemy" }],
        effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }],
      },
    ],
  };
  const Bodyguard = { name: "Bodyguard", base: { hp: 20, pwr: 1 } };
  return shaped(
    (s) => {
      s.team = [unitOf(Bodyguard), unitOf(Duelist)];
      s.offers = [byName.Venomancer, byName.Summoner, byName.Bulwark];
      s.gold = 10;
      s.lives = 5;
    },
    [...DEFAULT_RUN_POOL, Duelist, Bodyguard],
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
  armSignalTeardown();
  const browser = await chromium.launch();
  browsers.add(browser);
  browser.on("disconnected", () => browsers.delete(browser));
  return browser;
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
