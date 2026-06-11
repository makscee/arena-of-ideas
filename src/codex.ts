// Codex data — fully generated from registry + tunables + content.
// Zero hand-written game facts here; everything is derived so the codex
// never drifts from the code that runs the game.
//
// Consumers:
//   web/codex.ts  — renders this as the Codex screen
//   src/codex.test.ts — verifies coverage and tunable matching

import { FATIGUE_START, FATIGUE_RAMP, TURN_CAP } from "./battle.js";
import {
  INCOME_BASE,
  STARTING_LIVES,
  STACK_THRESHOLD,
  SHOP_SIZE_BASE,
  SHOP_SIZE_MAX,
  SHOP_SIZE_STEP,
  UNIT_COST,
  REROLL_COST,
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
// Builder
// ---------------------------------------------------------------------------

/** Build the full codex from a live registry and the shipped unit pool.
 *
 * `units` should be the DEFAULT_RUN_POOL (or any canonical pool); the codex
 * describes every unit exactly once by name, deduplicated.
 */
export function buildCodex(registry: StatusRegistry, units: UnitDef[]): CodexData {
  // -- statuses: every entry in the registry --
  const statuses: CodexStatusEntry[] = Object.values(registry).map((def) => ({
    name: def.name,
    description: describeStatus(def),
  }));
  statuses.sort((a, b) => a.name.localeCompare(b.name));

  // -- units: every unique name in the pool --
  const seen = new Set<string>();
  const unitEntries: CodexUnitEntry[] = [];
  for (const u of units) {
    if (seen.has(u.name)) continue;
    seen.add(u.name);
    unitEntries.push({
      name: u.name,
      hp: u.base.hp,
      pwr: u.base.pwr,
      abilities: (u.abilities ?? []).map((ab) => describeAbility(ab)),
    });
  }
  unitEntries.sort((a, b) => a.name.localeCompare(b.name));

  // -- rules: templated from tunables/constants --
  const rules: CodexRuleEntry[] = [
    {
      key: "fatigue",
      title: "Fatigue",
      text:
        `From turn ${FATIGUE_START}, every living unit takes ` +
        `${FATIGUE_RAMP}, ${FATIGUE_RAMP + 1}, ${FATIGUE_RAMP + 2}… damage at the end of each turn. ` +
        `Damage grows every turn and has no ceiling—battles always end. ` +
        `(Hard cap: ${TURN_CAP} turns; reaching it is a draw.)`,
    },
    {
      key: "income",
      title: "Income",
      text: `You receive ${INCOME_BASE} gold at the start of each new round, on top of any gold you carry over.`,
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
        `Collect ${STACK_THRESHOLD} copies of the same unit and they fuse into a level‑2. ` +
        `A level‑2 unit has higher base stats. ` +
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
      title: "Ghosts & champion",
      text:
        `After each fight your team is frozen as a ghost in that round’s pool—future runs fight it. ` +
        `Beat every ghost in your round and the champion to crown. ` +
        `The champion spot holds the last team that earned it; ` +
        `a vacant spot crowns automatically (only possible on a brand-new ladder).`,
    },
    {
      key: "draws",
      title: "Draws",
      text: `If both teams are wiped on the same turn, or the turn cap is reached, the fight is a draw. A draw costs no life.`,
    },
  ];

  return { statuses, units: unitEntries, rules };
}
