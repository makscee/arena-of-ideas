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
import { describeAbility, describeAbilityDef, describeStatus } from "./describe.js";
import { familyHex } from "./tunables.js";
import { partAtoms } from "./parts.js";
import type { Ability, AbilityRegistry, Family, StatusRegistry, UnitDef } from "./types.js";

/** A unit's resolved ability bodies (PRD #081): its one `ability` ref looked up
 * in the registry, with the legacy inline `abilities[]` as a back-compat read.
 * One place so the codex follows summons and describes abilities identically. */
function unitAbilityDefs(u: UnitDef, abilities: AbilityRegistry): Ability[] {
  const ab = abilities[u.ability];
  return ab ? [ab] : [];
}

// ---------------------------------------------------------------------------
// Shape types
// ---------------------------------------------------------------------------

export interface CodexStatusEntry {
  /** Status name — also the deep-link anchor fragment: #codex/status/<name> */
  name: string;
  description: string;
  /** Per-stack stat contributions, formatted for the card's framed stat cells
   * (e.g. "+2", "-1"); "·" when the status moves that stat by nothing. A Status
   * renders through the same card as a Unit (#078), framing its statMods where a
   * Unit frames base hp/pwr. */
  hp: string;
  pwr: string;
}

/** An entry in the Ability catalogue (PRD #081) — beside the Status catalogue.
 * Every shipped ability lists with its family (the colour axis), the derived
 * hex, and its DSL-derived description. */
export interface CodexAbilityEntry {
  /** Ability id/name — anchor fragment: #codex/ability/<name> */
  name: string;
  /** The colour family (one of 7). */
  family: Family;
  /** The family's hex — derived from the one palette (tunables FAMILY_HEX). */
  hex: string;
  description: string;
}

export interface CodexUnitEntry {
  /** Unit name — anchor fragment: #codex/unit/<name> */
  name: string;
  hp: number;
  pwr: number;
  /** The unit's single ability id (PRD #081) — the ref it carries. */
  ability: string;
  /** The unit's colour family, derived from its ability; absent only if the
   * ability isn't in the registry the codex was built with (shipped: always). */
  family?: Family;
  /** The unit's colour hex, derived from its family (tunables FAMILY_HEX). */
  hex?: string;
  /** The ability's description sentence(s) — a 1-element array for a unit's one
   * ability, empty if the ability didn't resolve. Kept as an array for the
   * search index and the card's ability line. */
  abilities: string[];
  /** Starting statuses, e.g. "Strength ×3" — empty for most units. */
  statuses: string[];
  /** Authorship credit for an approved (creation-loop) unit; absent for shipped
   * units. The codex shows it as a "made by …" line — the minimal credit display
   * the brief calls for (PRD #013 slice 4). */
  creator?: string;
}

export interface CodexPartEntry {
  /** Atom family — the codex section + first deep-link segment:
   * #codex/part/<family>/<kind>. */
  family: "trigger" | "interceptor" | "condition" | "selector" | "effect";
  /** The atom's union discriminant — unique within its family. */
  kind: string;
  /** Display label, e.g. "Apply status", "Front enemy". */
  name: string;
  /** One-line meaning, derived from describe.ts (presentation, kernel
   * untouched) — what this atom does when slotted into an ability. */
  meaning: string;
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
  /** The Ability catalogue (PRD #081) — every shipped ability with its family,
   * colour hex, and description, beside the Status catalogue. */
  abilities: CodexAbilityEntry[];
  units: CodexUnitEntry[];
  /** Every creator atom (Part) the type space defines — one entry per Trigger,
   * Interceptor, Condition, Selector, Effect (#078). Derived from the type
   * space (src/parts.ts), not a hand-maintained list. */
  parts: CodexPartEntry[];
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
export function codexUnits(approved: readonly UnitDef[] = [], abilities: AbilityRegistry = {}): UnitDef[] {
  const queue: UnitDef[] = [...DEFAULT_RUN_POOL, ...approved, ...BOOTSTRAP_TEAMS.flat(2), ...BOSS_TEAMS.flat()];
  const seen = new Set<string>();
  const out: UnitDef[] = [];
  while (queue.length > 0) {
    const u = queue.shift()!;
    if (seen.has(u.name)) continue;
    seen.add(u.name);
    out.push(u);
    for (const ab of unitAbilityDefs(u, abilities)) {
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
export function buildCodex(registry: StatusRegistry, units: UnitDef[], abilities: AbilityRegistry = {}): CodexData {
  // -- statuses: every entry in the registry --
  // A status frames its per-stack statMods in the card's stat cells (#078); a
  // stat the status doesn't move shows "·". No number is typed — it is the
  // statMod the kernel already carries.
  const fmtMod = (v: number | undefined): string => (v === undefined || v === 0 ? "·" : `${v > 0 ? "+" : ""}${v}`);
  const statuses: CodexStatusEntry[] = Object.values(registry).map((def) => ({
    name: def.name,
    description: describeStatus(def),
    hp: fmtMod(def.statMods?.hp),
    pwr: fmtMod(def.statMods?.pwr),
  }));
  statuses.sort((a, b) => a.name.localeCompare(b.name));

  // -- abilities: every entry in the ability registry (PRD #081) --
  // The Ability catalogue beside the Status catalogue: each shipped ability with
  // its family (the colour axis), the derived hex from the one palette, and its
  // DSL-derived description. #080's card and #082/#083's displays render colour +
  // ability-line from here.
  const abilityEntries: CodexAbilityEntry[] = Object.values(abilities).map((def) => ({
    name: def.name,
    family: def.family,
    hex: familyHex(def.family),
    description: describeAbilityDef(def),
  }));
  abilityEntries.sort((a, b) => a.name.localeCompare(b.name));

  // -- units: every unique name in the list --
  const seen = new Set<string>();
  const unitEntries: CodexUnitEntry[] = [];
  for (const u of units) {
    if (seen.has(u.name)) continue;
    seen.add(u.name);
    // Creator credit rides on approved units as a non-DSL `_creator` field
    // (src/registry.ts) — the kernel ignores it, the codex shows it.
    const creator = (u as { _creator?: unknown })._creator;
    // The unit's colour is its ability's family, derived here, never stored on
    // the unit (PRD #081). Resolves for shipped content; absent only if the
    // codex was built without the ability in its registry.
    const family: Family | undefined = abilities[u.ability]?.family;
    unitEntries.push({
      name: u.name,
      hp: u.base.hp,
      pwr: u.base.pwr,
      ability: u.ability,
      ...(family !== undefined ? { family, hex: familyHex(family) } : {}),
      abilities: unitAbilityDefs(u, abilities).map((ab) => describeAbility(ab)),
      statuses: (u.statuses ?? []).map((s) => `${s.status} ×${s.stacks}`),
      ...(typeof creator === "string" && creator.length > 0 ? { creator } : {}),
    });
  }
  unitEntries.sort((a, b) => a.name.localeCompare(b.name));

  // -- parts: every creator atom the type space defines (#078) --
  // One card per Trigger / Interceptor / Condition / Selector / Effect kind,
  // enumerated from the discriminated unions in types.ts (src/parts.ts) so a
  // newly-added Part kind gets a card automatically. Each meaning is derived
  // from the describe.ts helpers — presentation only, kernel untouched.
  const parts: CodexPartEntry[] = partAtoms().map((a) => ({
    family: a.family,
    kind: a.kind,
    name: a.name,
    meaning: a.meaning,
  }));

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
      // Mirrors ladderFight + challengeBoss (run.ts) and seedBootstrapTower (ladder.ts):
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

  return { statuses, abilities: abilityEntries, units: unitEntries, parts, rules };
}
