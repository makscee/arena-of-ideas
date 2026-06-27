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
import {
  DEFAULT_RUN_POOL,
  mergePool,
  parseApprovedRegistry,
  stressRegistry,
  stressAbilities,
  type AbilityRegistry,
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

export function defaultArenaContent(): ArenaContent {
  const reg = parseApprovedRegistry(approvedJson, stressRegistry, stressAbilities, "registry/approved-units.json");
  // An approved unit travels with its Ability (#081); merge any onto the shipped
  // registry so a run drafting an approved unit resolves its ability ref.
  return {
    pool: mergePool(DEFAULT_RUN_POOL, reg.units),
    statuses: stressRegistry,
    abilities: { ...stressAbilities, ...(reg.abilities ?? {}) },
  };
}
