// The stress set (SPEC §7) — all of it is DSL data. Nothing here is engine code;
// if one of these can't be expressed, the kernel grows consciously, never silently.

import type { AbilityDef, AbilityRegistry, StatusDef, StatusRegistry, UnitDef } from "../types.js";

export const Strength: StatusDef = { name: "Strength", statMods: { pwr: 1 }, abilities: [] };
export const Vitality: StatusDef = { name: "Vitality", statMods: { hp: 1 }, abilities: [] };
export const Curse: StatusDef = { name: "Curse", statMods: { pwr: -1 }, abilities: [] };

export const Poison: StatusDef = {
  name: "Poison",
  triggers: [{ kind: "trigger", on: { on: "TurnEnd" } }],
  selectors: [{ kind: "holder" }],
  abilities: [{ effects: [
    { kind: "damage", amount: { kind: "stacks" } },
    { kind: "consumeStacks", stacks: { kind: "const", value: 1 } },
  ] }],
};

export const Shield: StatusDef = {
  name: "Shield",
  triggers: [{ kind: "interceptor", on: { on: "Hurt", unit: "holder" } }],
  selectors: [{ kind: "holder" }],
  abilities: [{ effects: [{ kind: "absorbHurt" }] }],
};

export const Freeze: StatusDef = {
  name: "Freeze",
  triggers: [{ kind: "interceptor", on: { on: "Strike", striker: "holder" } }],
  selectors: [{ kind: "holder" }],
  abilities: [{ effects: [{ kind: "cancel", consumeSelf: 1 }] }],
};

export const Blessing: StatusDef = {
  name: "Blessing",
  triggers: [{ kind: "interceptor", on: { on: "Death", unit: "holder" } }],
  selectors: [{ kind: "holder" }],
  abilities: [{ effects: [{ kind: "preventDeathHeal", toHp: { kind: "stacks" }, removeSelf: true }] }],
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
// Each named Unit references exactly one of these actions by id, and the
// action's `family` supplies the Unit's color. Trigger/Selector context lives on
// the Unit; the effects here remain behavior-identical.

/** The vanilla basic-attack action. Its zero heal is mechanically inert; the
 * Unit recipe supplies the BattleStart/holder context. */
export const StrikeAbility: AbilityDef = {
  name: "Strike",
  family: "Strike",
  effects: [{ kind: "heal", amount: { kind: "const", value: 0 } }],
};

/** Venomancer's ability — apply 2 Poison to the front enemy after it strikes. */
export const Venom: AbilityDef = {
  name: "Venom",
  family: "Poison",
  effects: [{ kind: "applyStatus", status: "Poison", stacks: { kind: "const", value: 2 } }],
};

/** The summoned body — a vanilla Imp (Strike family). Referenced by `Conjure`. */
export const Imp: UnitDef = { name: "Imp", base: { hp: 2, pwr: 1 }, triggers: [{ kind: "trigger", on: { on: "BattleStart" } }], selectors: [{ kind: "holder" }], abilities: ["Strike"] };

/** Summoner's ability — spawn an Imp at the back of its team when it dies. */
export const Conjure: AbilityDef = {
  name: "Conjure",
  family: "Summon",
  effects: [{ kind: "summon", unit: Imp }],
};

/** Silencer's ability — silence the front enemy when the battle begins. */
export const Hush: AbilityDef = {
  name: "Hush",
  family: "Control",
  effects: [{ kind: "silence" }],
};

/** Necromancer's ability — return the most recently dead ally at 1 hp. */
export const Reanimate: AbilityDef = {
  name: "Reanimate",
  family: "Summon",
  effects: [{ kind: "resurrect", hp: { kind: "const", value: 1 } }],
};

export const stressAbilities: AbilityRegistry = {
  Strike: StrikeAbility,
  Venom,
  Conjure,
  Hush,
  Reanimate,
};

// ---- Units exercising the effect atoms (Summon, Silence, Resurrect) ----
// Each references exactly one ability by id (PRD #081); the ability's family is
// the unit's color.

export const Summoner: UnitDef = { name: "Summoner", base: { hp: 6, pwr: 1 }, triggers: [{ kind: "trigger", on: { on: "Death", unit: "holder" } }], selectors: [{ kind: "holder" }], abilities: ["Conjure"] };

export const Silencer: UnitDef = { name: "Silencer", base: { hp: 8, pwr: 2 }, triggers: [{ kind: "trigger", on: { on: "BattleStart" } }], selectors: [{ kind: "frontEnemy" }], abilities: ["Hush"] };

export const Necromancer: UnitDef = { name: "Necromancer", base: { hp: 7, pwr: 1 }, triggers: [{ kind: "trigger", on: { on: "Death", unit: "ally" } }], selectors: [{ kind: "lastDeadAlly" }], abilities: ["Reanimate"] };

export const Venomancer: UnitDef = { name: "Venomancer", base: { hp: 6, pwr: 1 }, triggers: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }], selectors: [{ kind: "frontEnemy" }], abilities: ["Venom"] };
