// Units as data (M2-1, mission #735). The live pool's source of truth is the
// DB (store units, pool snapshots, stints); main.ts seeds it on first start
// and builds rt.content from the current pool. The seed is today's code pool
// (ROWS, as `seed`/`live`) plus the 8 units cut in round 4 (as `library`), and
// one snapshot whose version is the code pool's, so nothing changes for runs
// in progress.
import type { MvpContent, UnitContent } from "../../../src/mvp/contract.js";
import { ROWS, type Row } from "../../../src/mvp/units.js";
import { contentOf } from "./content.js";
import type { MvpStore, PoolSnapshot, PoolStint, StoredUnit } from "./store.js";

/** The 8 units cut in round 4 (R4-15, b4ce175; docs/round4/units.md "Cuts and
 * changes"), as they stood just before the cut (f5b5dab). They seed the library. */
/* eslint-disable prettier/prettier */
export const CUT_IN_ROUND_4: Row[] = [
  { name: "Rat",          emoji: "🐀", tier: 1, pwr: 2, hp: 4,  when: "die",          who: "front",  does: "Poison 2",   archetype: "Its own death poisons whoever stands in front.",          awoken: { who: "enemies" } },
  { name: "Spike",        emoji: "🌵", tier: 1, pwr: 1, hp: 6,  when: "allyShield",   who: "front",  does: "Hit 1",      archetype: "Every Shield on an ally makes it lash out at the front.", awoken: { add: ["Poison 1"] } },
  { name: "Wither",       emoji: "🥀", tier: 1, pwr: 2, hp: 4,  when: "enemyDies",    who: "allies", does: "Strength 1", archetype: "Each enemy kill powers the whole team.",                  awoken: { add: ["Shield 1"] } },
  { name: "Rot",          emoji: "🦠", tier: 2, pwr: 1, hp: 6,  when: "turnEnd",      who: "front",  does: "Poison 1",   archetype: "Drips Poison on the front enemy every turn.",             awoken: { who: "enemies" } },
  { name: "Pediatrician", emoji: "🍼", tier: 2, pwr: 2, hp: 8,  when: "allySummoned", who: "it",     does: "Strength 1", archetype: "Strengthens every new summon.",                           awoken: { add: ["Bless 1"] } },
  { name: "Redirector",   emoji: "🪞", tier: 3, pwr: 2, hp: 8,  when: "hurt",         who: "random", does: "Hit 2",      archetype: "Reflects hits at a random enemy.",                        awoken: { add: ["Freeze 1"] } },
  { name: "Director",     emoji: "🎬", tier: 4, pwr: 3, hp: 9,  when: "allyDies",     who: "allies", does: "Strength 1", archetype: "Each ally death powers the team.",                        awoken: { add: ["Heal 2"] } },
  { name: "Doctor",       emoji: "🥼", tier: 4, pwr: 2, hp: 10, when: "hurt",         who: "allies", does: "Heal 1",     archetype: "Heals the team each time it's hit.",                      awoken: { add: ["Shield 1"] } },
];
/* eslint-enable prettier/prettier */

/** Seeds the units and the first pool, once: false when a pool exists
 * (nothing is written). The seed's ids are the code pool's (`slug(name)`),
 * so stored lines, ghosts and fusions keep resolving. */
export function seedUnits(store: MvpStore, now: Date): boolean {
  if (store.currentPool()) return false;
  const live = contentOf(ROWS);
  const libraryIds = contentOf(CUT_IN_ROUND_4).units.map((u) => u.id);
  const createdAt = now.toISOString();
  const daySeq = store.currentDay()?.seq ?? 1;
  const unit = (row: Row, unitId: string, status: StoredUnit["status"]): StoredUnit =>
    ({ unitId, status, row, authorId: null, origin: "seed", parentId: null, createdAt });
  const units = [
    ...ROWS.map((row, i) => unit(row, live.units[i]!.id, "live")),
    ...CUT_IN_ROUND_4.map((row, i) => unit(row, libraryIds[i]!, "library")),
  ];
  const ids = new Set(units.map((u) => u.unitId));
  if (ids.size !== units.length) throw new Error("seed unit ids collide");
  const stints: PoolStint[] = live.units.map((u) => ({ unitId: u.id, enteredSeq: daySeq, leftSeq: null, reason: "seed" }));
  return store.seedPool(units, { version: live.version, daySeq, unitIds: live.units.map((u) => u.id), createdAt }, stints);
}

/** The content of the current pool: its units' rows, in the snapshot's order. */
export function poolContent(store: MvpStore): MvpContent {
  const pool = store.currentPool();
  if (!pool) throw new Error("no pool: seedUnits first");
  const rows = pool.unitIds.map((id) => {
    const u = store.unit(id);
    if (!u) throw new Error(`pool ${pool.version} names unknown unit ${id}`);
    return u.row;
  });
  const content = contentOf(rows);
  // The version is derived from the built content, so a code change to how a
  // row is built (an Ability's effects) shows up here as a new version.
  if (content.version !== pool.version) console.warn(`[pool] the current pool was stored as ${pool.version} but builds as ${content.version}`);
  return content;
}

/** A snapshot's rows: its own, else its units' stored rows (undefined when
 * one is missing). */
function rowsOf(store: MvpStore, pool: PoolSnapshot): Row[] | undefined {
  if (pool.rows) return pool.rows;
  const rows: Row[] = [];
  for (const id of pool.unitIds) {
    const u = store.unit(id);
    if (!u) return undefined;
    rows.push(u.row);
  }
  return rows;
}

/** The pools runs play on (M2-2). A run pins the version it started with and
 * buys, gifts, copies and fuses through `of(version)` until it ends; new runs
 * start on `current()`. A pool change ends nothing. */
export interface PoolBook {
  /** The live pool. */
  current(): MvpContent;
  /** A pinned pool: the snapshot with this version; the live pool (with a
   * logged warning) when there is none, so an old run never crashes. */
  of(version: string): MvpContent;
}

/** The store's pools, built once per version. `fixed` is the content when the
 * store has no pool (tests that pass content and a bare store). */
export function poolBook(store: MvpStore, fixed?: MvpContent): PoolBook {
  const built = new Map<string, MvpContent>();
  if (fixed) built.set(fixed.version, fixed);
  const warned = new Set<string>();
  const build = (pool: PoolSnapshot): MvpContent | undefined => {
    const have = built.get(pool.version);
    if (have) return have;
    const rows = rowsOf(store, pool);
    if (!rows) return undefined;
    const content = { ...contentOf(rows), version: pool.version };
    built.set(pool.version, content);
    return content;
  };
  const current = (): MvpContent => {
    const pool = store.currentPool();
    if (!pool) {
      if (!fixed) throw new Error("no pool: seedUnits first");
      return fixed;
    }
    const content = build(pool);
    if (!content) throw new Error(`the current pool ${pool.version} names a unit the store doesn't have`);
    return content;
  };
  return {
    current,
    of(version) {
      const have = built.get(version);
      if (have) return have;
      const pool = store.pool(version);
      const content = pool && build(pool);
      if (content) return content;
      const live = current();
      if (!warned.has(version)) {
        warned.add(version);
        console.warn(`[pool] no pool ${version} to pin; its runs go on with the live pool ${live.version}`);
      }
      return live;
    },
  };
}

/** What `syncSeed` found: the seed units whose code row differs from the
 * stored one, the ones the store lacked, and the new pool's version. */
export interface SeedSync {
  changed: string[];
  added: string[];
  version: string;
  /** False: nothing differed, nothing was written. */
  wrote: boolean;
}

/** Writes the seed units' rows from code (ROWS) into the store and adds a new
 * pool snapshot with them (the orchestrator, on a deploy that changed ROWS;
 * `npm run mvp:pool -- sync-seed`). The current pool keeps its units: a seed
 * unit is updated in place, a new code row is added to the pool, a unit that
 * left code stays. Before a row changes, the current snapshot is stored again
 * with its rows, so runs pinned to it keep buying the old row. Safe to run
 * twice: the second run finds nothing to change. With `dryRun` nothing is written. */
export function syncSeed(store: MvpStore, now: Date, opts: { dryRun?: boolean } = {}): SeedSync {
  const pool = store.currentPool();
  if (!pool) throw new Error("no pool: start the server once to seed it");
  const code = contentOf(ROWS);
  const changed: string[] = [];
  const added: string[] = [];
  ROWS.forEach((row, i) => {
    const id = code.units[i]!.id;
    const stored = store.unit(id);
    if (!stored) added.push(id);
    else if (JSON.stringify(stored.row) !== JSON.stringify(row)) changed.push(id);
  });
  const unitIds = [...pool.unitIds, ...added.filter((id) => !pool.unitIds.includes(id))];
  const rowById = new Map(ROWS.map((row, i) => [code.units[i]!.id, row]));
  const oldRows = rowsOf(store, pool);
  if (!oldRows) throw new Error(`the current pool ${pool.version} names a unit the store doesn't have`);
  const rows = unitIds.map((id, i) => rowById.get(id) ?? oldRows[i]!);
  const version = contentOf(rows).version;
  const wrote = (changed.length > 0 || added.length > 0) && !opts.dryRun;
  if (wrote) {
    const at = now.toISOString();
    const daySeq = store.currentDay()?.seq ?? pool.daySeq;
    // Pin the old rows first: a later read of this version builds from them.
    if (!pool.rows) store.putPool({ ...pool, rows: oldRows });
    for (const id of changed) store.putUnit({ ...store.unit(id)!, row: rowById.get(id)! });
    for (const id of added) {
      store.putUnit({ unitId: id, status: "live", row: rowById.get(id)!, authorId: null, origin: "seed", parentId: null, createdAt: at });
      store.putStint({ unitId: id, enteredSeq: daySeq, leftSeq: null, reason: "seed" });
    }
    store.putPool({ version, daySeq, unitIds, createdAt: at, rows });
  }
  return { changed, added, version, wrote };
}

/** Every stored unit outside the live pool (the library, units that left, the
 * cut seed), as content: what a stored line, the champion's, a replay or the
 * Codex can still show (M2-2). Built once per live version. */
export function leftUnits(store: MvpStore, live: MvpContent): UnitContent[] {
  return leftContent(store, live).units;
}

type Left = Pick<MvpContent, "units" | "abilities" | "summons">;

function leftContent(store: MvpStore, live: MvpContent): Left {
  const have = leftCache.get(store);
  if (have?.version === live.version) return have.left;
  const inPool = new Set(live.units.map((u) => u.id));
  const out = store.units().filter((u) => !inPool.has(u.unitId) && u.status !== "candidate" && u.status !== "rejected");
  let left: Left = { units: [], abilities: {}, summons: [] };
  if (out.length) {
    try {
      const c = contentOf(out.map((u) => u.row));
      left = { units: c.units.map((u, i) => ({ ...u, id: out[i]!.unitId })), abilities: c.abilities, summons: c.summons ?? [] };
    } catch (e) {
      console.warn(`[pool] the units outside the pool don't build: ${(e as Error).message}`);
    }
  }
  leftCache.set(store, { version: live.version, left });
  return left;
}
const leftCache = new WeakMap<MvpStore, { version: string; left: Left }>();

/** GET /content (M2-2): the live pool, plus the units that left in `left`,
 * with the abilities and summoned bodies both need. The version stays the
 * live pool's. */
export function servedContent(rt: { store: MvpStore; content: MvpContent }): MvpContent {
  const live = rt.content;
  const left = leftContent(rt.store, live);
  if (!left.units.length) return live;
  const bodies = new Set((live.summons ?? []).map((b) => b.id));
  return {
    ...live,
    left: left.units,
    abilities: { ...left.abilities, ...live.abilities },
    summons: [...(live.summons ?? []), ...(left.summons ?? []).filter((b) => !bodies.has(b.id))],
  };
}
