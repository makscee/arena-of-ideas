// M3-3 (mission #800): an archetype's versions. A version is a new unit with
// `parentId` the version it was proposed from and `rootId` the archetype's
// first version; a unit without `rootId` is its own root, so every unit made
// before mission 3 is an archetype of one version and needed no write.
import type { UnitId } from "../../../src/mvp/contract.js";
import type { MvpStore, StoredUnit } from "./store.js";

/** The first version of `unitId`'s archetype (the unit itself when it has no
 * `rootId`, or isn't stored). */
export function rootOf(store: MvpStore, unitId: UnitId): UnitId {
  return store.unit(unitId)?.rootId ?? unitId;
}

/** Every version of `unitId`'s archetype in order: the root first, then the
 * others oldest first, whatever their status (a candidate or rejected version
 * included: callers filter). Empty when no such unit is stored. */
export function lineage(store: MvpStore, unitId: UnitId): StoredUnit[] {
  const root = rootOf(store, unitId);
  const all = store.units().filter((u) => (u.rootId ?? u.unitId) === root);
  return [...all.filter((u) => u.unitId === root), ...all.filter((u) => u.unitId !== root)];
}
