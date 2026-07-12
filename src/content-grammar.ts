import type { AbilityDef, AbilityRegistry, Selector, UnitDef, When } from "./types.js";

export const CONTENT_GRAMMAR_VERSION = 2 as const;
export const LEGACY_CONTENT_GRAMMAR_VERSION = 1 as const;

export interface ContentMigrationProvenance {
  grammarVersion: typeof CONTENT_GRAMMAR_VERSION;
  migratedFrom?: typeof LEGACY_CONTENT_GRAMMAR_VERSION;
}

type Obj = Record<string, unknown>;

export function isObject(value: unknown): value is Obj {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** Canonical v2 stores when/who on the Unit and what on named Abilities. */
export function canonicalUnit(
  name: string,
  base: UnitDef["base"],
  triggers: When[],
  selectors: Selector[],
  ability: string,
  extra: Pick<UnitDef, "level" | "statuses"> = {},
): UnitDef {
  return { name, base, triggers, selectors, abilities: [ability], ...extra };
}

export interface MigratedContent {
  units: UnitDef[];
  abilities: AbilityRegistry;
  provenance: ContentMigrationProvenance;
}

/**
 * Migrate a versioned persisted/content envelope. Version 1 used `unit.ability`
 * and put `whens`/`selectors` on AbilityDef. Version 2 separates the recipe:
 * Unit owns Trigger(s) and Selector(s), while Ability owns only Effect actions.
 * Unversioned data is treated as v1 solely for the checked compatibility path.
 */
export function migrateContentEnvelope(data: unknown, label = "content"): MigratedContent {
  if (!isObject(data) || !Array.isArray(data.units) || !isObject(data.abilities)) {
    throw new Error(`${label}: expected { grammarVersion, units, abilities }`);
  }
  const version = data.grammarVersion ?? LEGACY_CONTENT_GRAMMAR_VERSION;
  if (version !== LEGACY_CONTENT_GRAMMAR_VERSION && version !== CONTENT_GRAMMAR_VERSION) {
    throw new Error(`${label}.grammarVersion: unsupported content grammar version ${JSON.stringify(version)} (expected 1 or 2)`);
  }
  const rawAbilities = data.abilities as Obj;
  if (version === CONTENT_GRAMMAR_VERSION) {
    const units = data.units.map((u, i) => parseCanonicalUnit(u, `${label}.units[${i}]`));
    const abilities = parseCanonicalAbilities(rawAbilities, label);
    if (data.migratedFrom !== undefined && data.migratedFrom !== LEGACY_CONTENT_GRAMMAR_VERSION) {
      throw new Error(`${label}.migratedFrom: unsupported migration provenance ${JSON.stringify(data.migratedFrom)} (expected 1)`);
    }
    return {
      units,
      abilities,
      provenance: {
        grammarVersion: CONTENT_GRAMMAR_VERSION,
        ...(data.migratedFrom === LEGACY_CONTENT_GRAMMAR_VERSION ? { migratedFrom: LEGACY_CONTENT_GRAMMAR_VERSION } : {}),
      },
    };
  }

  const abilities: AbilityRegistry = {};
  const recipes = new Map<string, { triggers: When[]; selectors: Selector[]; condition?: UnitDef["condition"] }>();
  for (const [key, raw] of Object.entries(rawAbilities)) {
    if (!isObject(raw)) throw new Error(`${label}.abilities.${key}: legacy AbilityDef must be an object`);
    rejectMixedAbility(raw, `${label}.abilities.${key}`);
    if (!Array.isArray(raw.effects)) {
      throw new Error(`${label}.abilities.${key}: v1 AbilityDef needs an effects array`);
    }
    const fallback = legacyRecipe(key);
    const triggers = Array.isArray(raw.whens) ? raw.whens : fallback?.triggers;
    const selectors = Array.isArray(raw.selectors) ? raw.selectors : fallback?.selectors;
    if (!triggers || !selectors) {
      throw new Error(`${label}.abilities.${key}: cannot migrate v1 AbilityDef without whens/selectors; add them or convert its Units to grammar v2`);
    }
    recipes.set(key, {
      triggers: structuredClone(triggers) as When[],
      selectors: structuredClone(selectors) as Selector[],
      ...(raw.condition !== undefined ? { condition: structuredClone(raw.condition) as UnitDef["condition"] } : {}),
    });
    abilities[key] = {
      name: raw.name as string,
      family: raw.family as AbilityDef["family"],
      effects: structuredClone(raw.effects) as AbilityDef["effects"],
    };
  }
  const units = data.units.map((raw, i) => {
    const path = `${label}.units[${i}]`;
    if (!isObject(raw)) throw new Error(`${path}: unit must be an object`);
    rejectMixedUnit(raw, path);
    if (typeof raw.ability !== "string") throw new Error(`${path}.ability: v1 unit must reference exactly one Ability id`);
    const recipe = recipes.get(raw.ability) ?? legacyRecipe(raw.ability);
    if (!recipe) throw new Error(`${path}.ability: unknown ability ${JSON.stringify(raw.ability)} — cannot migrate without legacy recipe context`);
    const { ability: _old, ...rest } = raw;
    return { ...structuredClone(rest), ...structuredClone(recipe), abilities: [raw.ability] } as UnitDef;
  });
  return { units, abilities, provenance: { grammarVersion: CONTENT_GRAMMAR_VERSION, migratedFrom: LEGACY_CONTENT_GRAMMAR_VERSION } };
}

function parseCanonicalUnit(raw: unknown, path: string): UnitDef {
  if (!isObject(raw)) throw new Error(`${path}: unit must be an object`);
  rejectMixedUnit(raw, path);
  if (!Array.isArray(raw.triggers) || !Array.isArray(raw.selectors) || !Array.isArray(raw.abilities)) {
    throw new Error(`${path}: v2 unit needs triggers, selectors, and abilities arrays`);
  }
  return structuredClone(raw) as unknown as UnitDef;
}

function parseCanonicalAbilities(rawAbilities: Obj, label: string): AbilityRegistry {
  const out: AbilityRegistry = {};
  for (const [key, raw] of Object.entries(rawAbilities)) {
    if (!isObject(raw)) throw new Error(`${label}.abilities.${key}: AbilityDef must be an object`);
    if (raw.whens !== undefined || raw.selectors !== undefined || raw.condition !== undefined || raw.triggers !== undefined) {
      throw new Error(`${label}.abilities.${key}: Ability is only what happens in v2; move Trigger/Selector context to the Unit`);
    }
    if (!Array.isArray(raw.effects)) throw new Error(`${label}.abilities.${key}.effects: Ability needs an effects array describing what happens`);
    out[key] = structuredClone(raw) as unknown as AbilityDef;
  }
  return out;
}

export function rejectMixedUnit(unit: Obj, path: string): void {
  const old = unit.ability !== undefined;
  const canonical = unit.triggers !== undefined || unit.selectors !== undefined || unit.condition !== undefined || unit.abilities !== undefined;
  if (old && canonical) {
    throw new Error(`${path}: mixed v1/v2 unit payload (legacy ability with canonical triggers/selectors/condition/abilities); choose one grammar version`);
  }
}

/** Deterministic provenance for the shipped v1 actions. This is compatibility
 * data, not a default for new actions: unknown legacy actions fail actionably. */
export function legacyRecipe(id: string): { triggers: When[]; selectors: Selector[] } | undefined {
  switch (id) {
    case "Strike": return { triggers: [{ kind: "trigger", on: { on: "BattleStart" } }], selectors: [{ kind: "holder" }] };
    case "Venom": return { triggers: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }], selectors: [{ kind: "frontEnemy" }] };
    case "Conjure": return { triggers: [{ kind: "trigger", on: { on: "Death", unit: "holder" } }], selectors: [{ kind: "holder" }] };
    case "Hush": return { triggers: [{ kind: "trigger", on: { on: "BattleStart" } }], selectors: [{ kind: "frontEnemy" }] };
    case "Reanimate": return { triggers: [{ kind: "trigger", on: { on: "Death", unit: "ally" } }], selectors: [{ kind: "lastDeadAlly" }] };
    default: return undefined;
  }
}

function rejectMixedAbility(ability: Obj, path: string): void {
  if ((ability.whens !== undefined || ability.selectors !== undefined) && ability.triggers !== undefined) {
    throw new Error(`${path}: mixed legacy/canonical Ability payload; Trigger and Selector context belongs on the Unit in v2`);
  }
  if (ability.whens !== undefined && ability.effects !== undefined && ability.name !== undefined && ability.triggers === undefined) return;
  if (ability.whens !== undefined || ability.selectors !== undefined || ability.triggers !== undefined) {
    throw new Error(`${path}: Ability is only what happens in v2; move Trigger/Selector context to the Unit`);
  }
}
