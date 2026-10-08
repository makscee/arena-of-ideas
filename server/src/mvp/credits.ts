// Credits and the library (M2-9, mission #735), read from M2-1's units as
// data (./pool.ts): who a unit's idea was, NEW for a unit that entered in the
// last NEW_DAYS days, a player's creator number (the days their units have
// been live), and the units that have left with how long they were live.
// Reads only; the content version is untouched (credits are not content).
import type { CreditsView, FusionDiscovery, LibraryView, PlayerRef, UnitCredit, UnitId } from "../../../src/mvp/contract.js";
import { contentOf } from "./content.js";
import type { MvpRuntime } from "./runtime.js";
import type { PoolStint } from "./store.js";

/** A unit is NEW on the day it entered and the 2 days after. */
export const NEW_DAYS = 3;

type CreditDeps = Pick<MvpRuntime, "store" | "content">;

const today = (deps: CreditDeps): number => deps.store.currentDay()?.seq ?? 1;

/** The days a unit was live over these stints, counting a day it was in the
 * pool at all: an open stint through today, a closed one up to the day it left. */
export function liveDays(stints: PoolStint[], day: number): number {
  return stints.reduce((n, s) => n + Math.max(0, (s.leftSeq ?? day + 1) - s.enteredSeq), 0);
}

function author(deps: CreditDeps, id: string | null): PlayerRef | null {
  return id ? deps.store.player(id) ?? null : null;
}

/** The live units' authors and NEW badges, and the caller's creator number. */
export function creditsView(deps: CreditDeps, playerId?: string): CreditsView {
  const day = today(deps);
  const units: UnitCredit[] = [];
  for (const u of deps.content.units) {
    const stored = deps.store.unit(u.id);
    if (!stored) continue;
    // The seed entered on the first day units were data: never NEW.
    const open = deps.store.stints(u.id).find((s) => s.leftSeq === null);
    const isNew = stored.origin !== "seed" && !!open && day - open.enteredSeq < NEW_DAYS;
    const by = author(deps, stored.authorId);
    if (by || isNew) units.push({ unitId: u.id, by, isNew });
  }
  let you: CreditsView["you"] = null;
  if (playerId) {
    const mine = deps.store.units().filter((u) => u.authorId === playerId);
    you = { units: mine.length, days: mine.reduce((n, u) => n + liveDays(deps.store.stints(u.unitId), day), 0) };
  }
  return { day, units, you };
}

/** The units that have left, the one that left last first (then by tier and
 * name), each built on its
 * own (its id kept) with the abilities and summons their sheets need. */
export function libraryView(deps: CreditDeps): LibraryView {
  const day = today(deps);
  const out: LibraryView = { units: [], abilities: {}, statuses: {}, summons: [] };
  const fusions = deps.store.fusions();
  const left = (id: UnitId) => Math.max(0, ...deps.store.stints(id).map((s) => s.leftSeq ?? 0));
  const stored = deps.store.units({ status: "library" }).sort((a, b) => left(b.unitId) - left(a.unitId) || a.row.tier - b.row.tier || a.row.name.localeCompare(b.row.name));
  for (const s of stored) {
    const c = contentOf([s.row]);
    const unit = { ...c.units[0]!, id: s.unitId };
    Object.assign(out.abilities, c.abilities);
    Object.assign(out.statuses, c.statuses);
    for (const x of c.summons ?? []) if (!out.summons.some((y) => y.id === x.id)) out.summons.push(x);
    const stints = deps.store.stints(s.unitId);
    const its: FusionDiscovery[] = fusions.filter((f) => f.first === s.unitId || f.second === s.unitId);
    out.units.push({ unit, by: author(deps, s.authorId), liveDays: stints.length ? liveDays(stints, day) : null, fusions: its });
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
