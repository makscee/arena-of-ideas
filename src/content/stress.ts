// The stress set (SPEC §7) — all of it is DSL data. Nothing here is engine code;
// if one of these can't be expressed, the kernel grows consciously, never silently.

import type { AbilityDef, AbilityRegistry, StatusDef, StatusRegistry, UnitDef } from "../types.js";

export const Strength: StatusDef = { name: "Strength", statMods: { pwr: 1 }, abilities: [] };
export const Vitality: StatusDef = { name: "Vitality", statMods: { hp: 1 }, abilities: [] };
export const Curse: StatusDef = { name: "Curse", statMods: { pwr: -1 }, abilities: [] };

export const Poison: StatusDef = {
  name: "Poison",
  abilities: [
    {
      whens: [{ kind: "trigger", on: { on: "TurnEnd" } }],
      selectors: [{ kind: "holder" }],
      effects: [
        { kind: "damage", amount: { kind: "stacks" } },
        { kind: "consumeStacks", stacks: { kind: "const", value: 1 } },
      ],
    },
  ],
};

export const Shield: StatusDef = {
  name: "Shield",
  abilities: [
    {
      whens: [{ kind: "interceptor", on: { on: "Hurt", unit: "holder" } }],
      selectors: [{ kind: "holder" }],
      effects: [{ kind: "absorbHurt" }],
    },
  ],
};

export const Freeze: StatusDef = {
  name: "Freeze",
  abilities: [
    {
      whens: [{ kind: "interceptor", on: { on: "Strike", striker: "holder" } }],
      selectors: [{ kind: "holder" }],
      effects: [{ kind: "cancel", consumeSelf: 1 }],
    },
  ],
};

export const Blessing: StatusDef = {
  name: "Blessing",
  abilities: [
    {
      whens: [{ kind: "interceptor", on: { on: "Death", unit: "holder" } }],
      selectors: [{ kind: "holder" }],
      effects: [{ kind: "preventDeathHeal", toHp: { kind: "stacks" }, removeSelf: true }],
    },
  ],
};

export const stressRegistry: StatusRegistry = {
  Strength,
  Vitality,
  Curse,
  Poison,
  Shield,
  Freeze,
  Blessing,
};

// ---- Abilities — named, referenceable bundles (PRD #081) ----
//
// Each named Unit references exactly one of these by id, and the ability's
// `family` is the unit's color. The bodies are the SAME whens/selectors/effects
// the units carried inline before #081, so resolving a ref produces a
// byte-identical firing list (the migration is behavior-preserving).

/** The vanilla "basic attacker" ability — a unit whose whole act is the kernel
 * strike. It carries the Strike family (its color) but no extra mechanic, so its
 * body is provably inert: it fires once at BattleStart and heals the holder for
 * 0, which `runEffect` drops before any event is emitted (heal's `amount <= 0`
 * guard). A vanilla body therefore contributes one reactor entry that never
 * appends to the log, consumes no RNG, and mutates nothing — byte-identical to
 * the old ability-less body, while still giving every unit the one-ability/one-
 * color identity #081 requires. */
export const StrikeAbility: AbilityDef = {
  name: "Strike",
  family: "Strike",
  whens: [{ kind: "trigger", on: { on: "BattleStart" } }],
  selectors: [{ kind: "holder" }],
  effects: [{ kind: "heal", amount: { kind: "const", value: 0 } }],
};

/** Venomancer's ability — apply 2 Poison to the front enemy after it strikes. */
export const Venom: AbilityDef = {
  name: "Venom",
  family: "Poison",
  whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
  selectors: [{ kind: "frontEnemy" }],
  effects: [{ kind: "applyStatus", status: "Poison", stacks: { kind: "const", value: 2 } }],
};

export const stressAbilities: AbilityRegistry = {
  Strike: StrikeAbility,
  Venom,
};

// ---- Units exercising the effect atoms (Summon, Silence, Resurrect) ----

export const Imp: UnitDef = { name: "Imp", base: { hp: 2, pwr: 1 } };

export const Summoner: UnitDef = {
  name: "Summoner",
  base: { hp: 6, pwr: 1 },
  abilities: [
    {
      whens: [{ kind: "trigger", on: { on: "Death", unit: "holder" } }],
      selectors: [{ kind: "holder" }],
      effects: [{ kind: "summon", unit: Imp }],
    },
  ],
};

export const Silencer: UnitDef = {
  name: "Silencer",
  base: { hp: 8, pwr: 2 },
  abilities: [
    {
      whens: [{ kind: "trigger", on: { on: "BattleStart" } }],
      selectors: [{ kind: "frontEnemy" }],
      effects: [{ kind: "silence" }],
    },
  ],
};

export const Necromancer: UnitDef = {
  name: "Necromancer",
  base: { hp: 7, pwr: 1 },
  abilities: [
    {
      whens: [{ kind: "trigger", on: { on: "Death", unit: "ally" } }],
      selectors: [{ kind: "lastDeadAlly" }],
      effects: [{ kind: "resurrect", hp: { kind: "const", value: 1 } }],
    },
  ],
};

export const Venomancer: UnitDef = {
  name: "Venomancer",
  base: { hp: 6, pwr: 1 },
  abilities: [
    {
      whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
      selectors: [{ kind: "frontEnemy" }],
      effects: [{ kind: "applyStatus", status: "Poison", stacks: { kind: "const", value: 2 } }],
    },
  ],
};
