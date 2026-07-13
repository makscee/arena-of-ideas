/**
 * The arena's canonical run content — the one pool and status registry the
 * server accepts submitted runs against. Built exactly the way the web shell
 * builds its draft pool (web/approved.ts): the shipped DEFAULT_RUN_POOL plus
 * the committed approved-units registry, gated by the same parser. A submitted
 * run carries its pool/statuses by value (serializeRun); the server pins them
 * to THIS content, or re-derivation would happily "verify" a run played
 * against a god-unit pool the client invented.
 *
 * Injectable through AppDeps so tests can pin a tiny deterministic pool; prod
 * (main.ts) uses the default.
 */
import { isDeepStrictEqual } from "node:util";
import {
  CONTENT_GRAMMAR_VERSION,
  DEFAULT_RUN_POOL,
  assertValidPool,
  mergePool,
  migrateContentEnvelope,
  parseApprovedRegistry,
  stressRegistry,
  stressAbilities,
  type AbilityRegistry,
  type ApprovedRegistry,
  type StatusRegistry,
  type UnitDef,
} from "../../src/index.js";
import approvedJson from "../../registry/approved-units.json";

/** The content a run must have been played with to be accepted. */
export interface ArenaContent {
  pool: UnitDef[];
  statuses: StatusRegistry;
  abilities: AbilityRegistry;
}

export interface ArenaContentSnapshot extends ArenaContent {
  approvedRegistry: ApprovedRegistry;
}

export interface PersistedContentFields {
  approvedRegistry: string;
  pool: string;
  statuses: string;
  abilities: string;
}

function parseJson(raw: string, label: string): unknown {
  try { return JSON.parse(raw); }
  catch (err) { throw new Error(`${label} is corrupt JSON: ${(err as Error).message}`); }
}

function requireRegistryShape(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${label}: expected an object`);
  }
  return value as Record<string, unknown>;
}

/** Parse one persisted content snapshot through the kernel's canonical grammar
 * and semantic validators. The registry, pool and named definitions are one
 * receipt: governed units/Abilities must be the same values the pool serves. */
export function parseArenaContentSnapshot(row: PersistedContentFields, label: string): ArenaContentSnapshot {
  const statuses = requireRegistryShape(parseJson(row.statuses, `${label}.statuses`), `${label}.statuses`) as StatusRegistry;
  const abilitiesRaw = requireRegistryShape(parseJson(row.abilities, `${label}.abilities`), `${label}.abilities`) as AbilityRegistry;
  const poolRaw = parseJson(row.pool, `${label}.pool`);
  const canonical = migrateContentEnvelope({ grammarVersion: CONTENT_GRAMMAR_VERSION, units: poolRaw, abilities: abilitiesRaw }, label);
  assertValidPool(canonical.units, statuses, canonical.abilities, `${label}.pool`);

  const approvedRaw = requireRegistryShape(parseJson(row.approvedRegistry, `${label}.approved_registry`), `${label}.approved_registry`);
  if (approvedRaw["grammarVersion"] !== CONTENT_GRAMMAR_VERSION) {
    throw new Error(`${label}.approved_registry.grammarVersion must be ${CONTENT_GRAMMAR_VERSION}`);
  }
  const approvedRegistry = parseApprovedRegistry(approvedRaw, statuses, canonical.abilities, `${label}.approved_registry`);
  const expectedPool = mergePool(DEFAULT_RUN_POOL, approvedRegistry.units);
  if (!isDeepStrictEqual(canonical.units, expectedPool)) {
    throw new Error(`${label}.pool does not match DEFAULT_RUN_POOL + approved registry units`);
  }
  for (const [name, ability] of Object.entries(approvedRegistry.abilities ?? {})) {
    if (!isDeepStrictEqual(canonical.abilities[name], ability)) {
      throw new Error(`${label}.abilities.${name} does not match the approved registry receipt`);
    }
  }
  return { pool: canonical.units, statuses, abilities: canonical.abilities, approvedRegistry };
}

export function defaultApprovedRegistry(): ApprovedRegistry {
  return parseApprovedRegistry(approvedJson, stressRegistry, stressAbilities, "registry/approved-units.json");
}

export function defaultArenaContent(): ArenaContent {
  const reg = defaultApprovedRegistry();
  // An approved unit travels with its Ability (#081); merge any onto the shipped
  // registry so a run drafting an approved unit resolves its ability ref.
  return {
    pool: mergePool(DEFAULT_RUN_POOL, reg.units),
    statuses: stressRegistry,
    abilities: { ...stressAbilities, ...(reg.abilities ?? {}) },
  };
}
