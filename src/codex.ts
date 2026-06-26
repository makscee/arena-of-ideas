// Codex data — fully generated from registry + tunables + content.
// Zero hand-written game facts here; every number and every list entry is
// derived so the codex never drifts from the code that runs the game. Where a
// rule has a formula (fatigue, income), the codex CALLS it — it never
// re-writes the formula as prose-math that could rot independently.
//
// Consumers:
//   web/codex.ts  — renders this as the Codex screen
//   src/codex.test.ts — verifies coverage and tunable matching

import { FATIGUE_RAMP, FATIGUE_START, TURN_CAP, fatigueAmount } from "./battle.js";
import {
  BOSS_TEAMS,
  BOOTSTRAP_TEAMS,
  DEFAULT_RUN_POOL,
  INCOME_CAP,
  INCOME_PER_ROUND,
  LEVEL_HP_GROWTH,
  LEVEL_PWR_GROWTH,
  REROLL_COST,
  SHOP_SIZE_BASE,
  SHOP_SIZE_MAX,
  SHOP_SIZE_STEP,
  STACK_THRESHOLD,
  STARTING_LIVES,
  TOWER_HEIGHT,
  UNIT_COST,
  incomeForRound,
} from "./tunables.js";
import { describeAbility, describeStatus } from "./describe.js";
import type { StatusRegistry, UnitDef } from "./types.js";

// ---------------------------------------------------------------------------
// Shape types
// ---------------------------------------------------------------------------

export interface CodexStatusEntry {
  /** Status name — also the deep-link anchor fragment: #codex/status/<name> */
  name: string;
  description: string;
}

export interface CodexUnitEntry {
  /** Unit name — anchor fragment: #codex/unit/<name> */
  name: string;
  hp: number;
  pwr: number;
  /** One sentence per ability, or empty array for vanilla bodies. */
  abilities: string[];
  /** Starting statuses, e.g. "Strength ×3" — empty for most units. */
  statuses: string[];
  /** Authorship credit for an approved (creation-loop) unit; absent for shipped
   * units. The codex shows it as a "made by …" line — the minimal credit display
   * the brief calls for (PRD #013 slice 4). */
  creator?: string;
}

export interface CodexRuleEntry {
  /** Short key used as anchor fragment: #codex/rule/<key> */
  key: string;
  /** Display title */
  title: string;
  /** Full rule text, all numbers derived from code. */
  text: string;
}

export interface CodexData {
  statuses: CodexStatusEntry[];
  units: CodexUnitEntry[];
  rules: CodexRuleEntry[];
}

// ---------------------------------------------------------------------------
// Unit collection
// ---------------------------------------------------------------------------

/** Every unit a player can meet: the shop pool first (so the buyable variant
 * wins the name dedup), then bootstrap climb ghosts and the per-floor bosses
 * (Warden, Warlord, scaled vanillas), then anything those units summon (Imp).
 * The codex must cover what a player can FACE, not only what they can buy — and
 * with a boss seated on every floor (PRD 075 slice 3) that includes every team
 * in BOSS_TEAMS, the summit (old BOOTSTRAP_CHAMPION) among them. */
export function codexUnits(approved: readonly UnitDef[] = []): UnitDef[] {
  const queue: UnitDef[] = [...DEFAULT_RUN_POOL, ...approved, ...BOOTSTRAP_TEAMS.flat(2), ...BOSS_TEAMS.flat()];
  const seen = new Set<string>();
  const out: UnitDef[] = [];
  while (queue.length > 0) {
    const u = queue.shift()!;
    if (seen.has(u.name)) continue;
    seen.add(u.name);
    out.push(u);
    for (const ab of u.abilities ?? []) {
      for (const e of ab.effects) if (e.kind === "summon") queue.push(e.unit);
    }
  }
  return out;
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/** Build the full codex from a live registry and a unit list (codexUnits()
 * for the shipped game). Units are described once per name, first occurrence
 * winning the dedup. */
export function buildCodex(registry: StatusRegistry, units: UnitDef[]): CodexData {
  // -- statuses: every entry in the registry --
  const statuses: CodexStatusEntry[] = Object.values(registry).map((def) => ({
    name: def.name,
    description: describeStatus(def),
  }));
  statuses.sort((a, b) => a.name.localeCompare(b.name));

  // -- units: every unique name in the list --
  const seen = new Set<string>();
  const unitEntries: CodexUnitEntry[] = [];
  for (const u of units) {
    if (seen.has(u.name)) continue;
    seen.add(u.name);
    // Creator credit rides on approved units as a non-DSL `_creator` field
    // (src/registry.ts) — the kernel ignores it, the codex shows it.
    const creator = (u as { _creator?: unknown })._creator;
    unitEntries.push({
      name: u.name,
      hp: u.base.hp,
      pwr: u.base.pwr,
      abilities: (u.abilities ?? []).map((ab) => describeAbility(ab)),
      statuses: (u.statuses ?? []).map((s) => `${s.status} ×${s.stacks}`),
      ...(typeof creator === "string" && creator.length > 0 ? { creator } : {}),
    });
  }
  unitEntries.sort((a, b) => a.name.localeCompare(b.name));

  // -- rules: templated sentences; every number computed, never typed --

  // Income: the curve speaks for itself — flat and growing produce different
  // sentences, each true against incomeForRound().
  const incomeText =
    INCOME_PER_ROUND === 0
      ? `You receive ${incomeForRound(1)} gold at the start of each new round, on top of any gold you carry over.`
      : `Round income starts at ${incomeForRound(1)} gold and grows by ${INCOME_PER_ROUND} each round, ` +
        `capped at ${INCOME_CAP}; unspent gold carries over.`;

  // Fatigue growth: phrased off FATIGUE_RAMP — "grows without limit" is only
  // true while the ramp ramps; at RAMP=0 the damage holds steady instead.
  const fatigueGrowth =
    FATIGUE_RAMP > 0
      ? "Damage grows every turn without limit — battles always end."
      : "The damage holds steady each turn.";

  const rules: CodexRuleEntry[] = [
    {
      key: "fatigue",
      title: "Fatigue",
      // The ramp values come from the kernel's own fatigueAmount() — the codex
      // never re-states the formula.
      text:
        `From turn ${FATIGUE_START}, every living unit takes ` +
        `${fatigueAmount(FATIGUE_START)}, ${fatigueAmount(FATIGUE_START + 1)}, ${fatigueAmount(FATIGUE_START + 2)}… ` +
        `damage at the end of each turn. ${fatigueGrowth} ` +
        `(Hard cap: ${TURN_CAP} turns; reaching it is a draw.)`,
    },
    {
      key: "income",
      title: "Income",
      text: incomeText,
    },
    {
      key: "lives",
      title: "Lives",
      text:
        `You start with ${STARTING_LIVES} lives. ` +
        `Losing a fight costs 1 life. ` +
        `A draw costs no life. ` +
        `Reach 0 lives and the run ends.`,
    },
    {
      key: "fusion",
      title: "Fusion",
      text:
        `Collect ${STACK_THRESHOLD} copies of the same unit and they fuse: the unit gains a level, ` +
        `+${LEVEL_HP_GROWTH} base hp and +${LEVEL_PWR_GROWTH} base pwr. ` +
        `The copy count resets after each fuse, so ${STACK_THRESHOLD - 1} more copies reach the next level — ` +
        `there is no level cap. ` +
        `Each unit costs ${UNIT_COST}g; rerolling the shop costs ${REROLL_COST}g.`,
    },
    {
      key: "shop",
      title: "Shop growth",
      text:
        `The shop starts with ${SHOP_SIZE_BASE} offers and gains one more every ${SHOP_SIZE_STEP} rounds, ` +
        `up to ${SHOP_SIZE_MAX} offers.`,
    },
    {
      key: "strike-order",
      title: "Strike order",
      text:
        `Front units fight first. When two units meet for the first time, a seeded coin decides who strikes first ` +
        `(“PairFaced”). That result sticks for the rest of the battle—same pair, same first-striker.`,
    },
    {
      key: "ghosts",
      title: "Ghosts & bosses",
      // Mirrors ladderFight + challengeBoss (run.ts) and openLadder (ladder.ts):
      // one random ghost per climb; the boss challenge is the terminal move;
      // the tower is a fixed TOWER_HEIGHT and climbing past the top overshoots.
      text:
        `Before every fight your team is frozen as a ghost into your floor's pool — future runs fight it. ` +
        `Each floor you climb fights one ghost drawn at random from that floor's pool (never your own); ` +
        `win or lose, the run moves up a floor. ` +
        `Each floor also has a seated boss: challenging it is your run's terminal move — beat the boss to take its ` +
        `seat and end the run crowned; lose or draw and the run ends without the seat. ` +
        `A fresh ladder opens pre-seeded as a fixed ${TOWER_HEIGHT}-floor tower — every floor has shipped ghost teams ` +
        `and a shipped boss, the top floor's boss being the champion — so a crown is always earned by beating someone. ` +
        `Climb past the top and you overshoot onto an empty floor: no boss, no crown, so a winning run challenges a boss ` +
        `rather than climbing forever.`,
    },
    {
      key: "draws",
      title: "Draws",
      text: `If both teams are wiped on the same turn, or the turn cap is reached, the fight is a draw. A draw costs no life.`,
    },
  ];

  return { statuses, units: unitEntries, rules };
}
