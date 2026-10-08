// Credits and the library (M2-9, mission #735), read from M2-1's units as
// data (./pool.ts): who a unit's idea was, NEW for a unit that entered in the
// last NEW_DAYS days, a player's creator number (the days their units have
// been live), and the units that have left with how long they were live.
// M3-8 (mission #800) reads them across an archetype's versions (M3-3's
// lineage, ./lineage.ts): "idea by" the root's author, "evolved by" the
// version's, days live over every version, and the versions' history.
// Reads only; the content version is untouched (credits are not content).
import type { CreditsView, FusionDiscovery, LibraryView, PlayerRef, UnitCredit, UnitId, UnitVersion, VersionCredit } from "../../../src/mvp/contract.js";
import { slug } from "../../../src/mvp/units.js";
import { contentOf } from "./content.js";
import type { MvpRuntime } from "./runtime.js";
import type { PoolStint, StoredUnit } from "./store.js";

/** A unit is NEW on the day it entered and the 2 days after. */
export const NEW_DAYS = 3;

type CreditDeps = Pick<MvpRuntime, "store" | "content">;

const today = (deps: CreditDeps): number => deps.store.currentDay()?.seq ?? 1;

/** The days a unit was live over these stints, counting a day it was in the
 * pool at all: an open stint through today, a closed one up to the day it left. */
export function liveDays(stints: PoolStint[], day: number): number {
  return stints.reduce((n, s) => n + Math.max(0, (s.leftSeq ?? day + 1) - s.enteredSeq), 0);
}

/** One archetype (M3-8): its root's author, its versions (the root, then each
 * that entered the pool, in the order it entered) and its days live over all
 * of them (null: no stint was ever recorded). */
interface Archetype {
  rootAuthorId: string | null;
  versions: UnitVersion[];
  days: number | null;
}

/** Every stored unit's archetype and stints, read once (a lineage per unit
 * would read every unit again for each). */
function archetypes(deps: CreditDeps, day: number) {
  const units = deps.store.units();
  const stintsOf = new Map<UnitId, PoolStint[]>();
  for (const s of deps.store.stints()) stintsOf.set(s.unitId, [...(stintsOf.get(s.unitId) ?? []), s]);
  const players = new Map<string, PlayerRef | null>();
  const by = (id: string | null): PlayerRef | null => {
    if (!id) return null;
    if (!players.has(id)) players.set(id, deps.store.player(id) ?? null);
    return players.get(id)!;
  };
  const groups = new Map<UnitId, StoredUnit[]>();
  for (const u of units) {
    const root = u.rootId ?? u.unitId;
    groups.set(root, [...(groups.get(root) ?? []), u]);
  }
  const ofUnit = new Map<UnitId, Archetype>();
  for (const [root, all] of groups) {
    const first = (u: StoredUnit) => Math.min(...(stintsOf.get(u.unitId) ?? []).map((s) => s.enteredSeq));
    const entered = all.filter((u) => u.unitId !== root && stintsOf.has(u.unitId)).sort((x, y) => first(x) - first(y));
    const rootUnit = all.find((u) => u.unitId === root);
    const versions = [...(rootUnit ? [rootUnit] : []), ...entered].map((u, i): UnitVersion => {
      const st = stintsOf.get(u.unitId) ?? [];
      return { unitId: u.unitId, version: i + 1, by: by(u.authorId), liveDays: st.length ? liveDays(st, day) : null, live: u.status === "live" };
    });
    const known = versions.filter((v) => v.liveDays !== null);
    const a: Archetype = { rootAuthorId: rootUnit?.authorId ?? null, versions, days: known.length ? known.reduce((n, v) => n + v.liveDays!, 0) : null };
    for (const u of all) ofUnit.set(u.unitId, a);
  }
  /** A version's credit: "idea by" the root's author, "evolved by" its own
   * when that differs (a seed root has none: "evolved by" alone). */
  const credit = (u: StoredUnit): VersionCredit => {
    const a = ofUnit.get(u.unitId)!;
    const v = a.versions.find((x) => x.unitId === u.unitId);
    return {
      by: by(a.rootAuthorId),
      evolvedBy: u.authorId && u.authorId !== a.rootAuthorId ? by(u.authorId) : null,
      version: v?.version ?? a.versions.length + 1,
      versions: a.versions.length > 1 ? a.versions : [],
    };
  };
  return { units, stintsOf, ofUnit, credit };
}

/** A stored unit's credit ("idea by", "evolved by"), for any unit, live or
 * not (M4-8's daily post: the units that entered and left). Reads every unit
 * once; call the returned function per unit. */
export function versionCredits(deps: CreditDeps): (u: StoredUnit) => VersionCredit {
  return archetypes(deps, today(deps)).credit;
}

/** The live units' credits, NEW badges and days live, and the caller's
 * creator number. */
export function creditsView(deps: CreditDeps, playerId?: string): CreditsView {
  const day = today(deps);
  const arch = archetypes(deps, day);
  const units: UnitCredit[] = [];
  // The pool's content ids are its rows' slugs (contentOf), and a version
  // keeps its root's name: map a content id to the stored unit through the
  // pool snapshot (a version "hog-3" is served as "hog").
  const pool = deps.store.currentPool()?.unitIds ?? [];
  const storedOf = new Map<UnitId, UnitId>();
  for (const id of pool) {
    const s = deps.store.unit(id);
    if (s) storedOf.set(slug(s.row.name), id);
  }
  for (const u of deps.content.units) {
    const stored = deps.store.unit(pool.includes(u.id) ? u.id : storedOf.get(u.id) ?? u.id);
    if (!stored) continue;
    // The seed entered on the first day units were data: never NEW, unless
    // it comes back from the Library unchanged (M3-7's "return").
    const open = (arch.stintsOf.get(stored.unitId) ?? []).find((s) => s.leftSeq === null);
    const isNew = !!open && (stored.origin !== "seed" || open.reason === "return") && day - open.enteredSeq < NEW_DAYS;
    units.push({ unitId: u.id, ...arch.credit(stored), isNew, liveDays: arch.ofUnit.get(stored.unitId)!.days ?? 0 });
  }
  let you: CreditsView["you"] = null;
  if (playerId) {
    // An archetype's first author counts every version's days; an evolver
    // counts only their own versions'.
    let days = 0;
    for (const a of new Set(arch.ofUnit.values()))
      days += a.rootAuthorId === playerId ? (a.days ?? 0) : a.versions.reduce((n, v) => n + (v.by?.id === playerId ? (v.liveDays ?? 0) : 0), 0);
    you = { units: arch.units.filter((u) => u.authorId === playerId).length, days };
  }
  return { day, units, you };
}

/** The units that have left, the one that left last first (then by tier and
 * name), each built on its
 * own (its id kept) with the abilities and summons their sheets need. A
 * version that never entered the pool (a proposal that lost) isn't one. */
export function libraryView(deps: CreditDeps): LibraryView {
  const day = today(deps);
  const arch = archetypes(deps, day);
  const out: LibraryView = { units: [], abilities: {}, statuses: {}, summons: [] };
  const fusions = deps.store.fusions();
  const left = (id: UnitId) => Math.max(0, ...(arch.stintsOf.get(id) ?? []).map((s) => s.leftSeq ?? 0));
  const versionOf = (u: StoredUnit) => arch.ofUnit.get(u.unitId)!.versions.some((v) => v.unitId === u.unitId);
  const stored = arch.units
    .filter((u) => u.status === "library" && versionOf(u))
    .sort((a, b) => left(b.unitId) - left(a.unitId) || a.row.tier - b.row.tier || a.row.name.localeCompare(b.row.name));
  for (const s of stored) {
    const c = contentOf([s.row]);
    // M4-5: its Russian name and line ride along, as on GET /content.
    const unit = { ...c.units[0]!, id: s.unitId, ...(s.texts?.ru ? { texts: s.texts } : {}) };
    Object.assign(out.abilities, c.abilities);
    Object.assign(out.statuses, c.statuses);
    for (const x of c.summons ?? []) if (!out.summons.some((y) => y.id === x.id)) out.summons.push(x);
    const its: FusionDiscovery[] = fusions.filter((f) => f.first === s.unitId || f.second === s.unitId);
    out.units.push({ unit, ...arch.credit(s), liveDays: arch.ofUnit.get(s.unitId)!.days, fusions: its });
  }
  return out;
}

/** Dev (POST /dev/credit-unit): the live unit becomes the player's idea that
 * entered today, so its card shows the credit and NEW. Its row is untouched,
 * so the content version stays. False: no such live unit. */
export function creditUnit(deps: CreditDeps, unitId: UnitId, playerId: string): boolean {
  const u = deps.store.unit(unitId);
  if (!u || u.status !== "live") return false;
  const day = today(deps);
  for (const s of deps.store.stints(unitId)) if (s.leftSeq === null && s.enteredSeq !== day) deps.store.putStint({ ...s, leftSeq: day, reason: "dev: credited" });
  deps.store.putUnit({ ...u, authorId: playerId, origin: "idea" });
  deps.store.putStint({ unitId, enteredSeq: day, leftSeq: null, reason: "dev: credited" });
  return true;
}
