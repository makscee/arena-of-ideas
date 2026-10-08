// Units as data (M2-1, mission #735). The live pool's source of truth is the
// DB (store units, pool snapshots, stints); main.ts seeds it on first start
// and builds rt.content from the current pool. The seed is today's code pool
// (ROWS, as `seed`/`live`) plus the 8 units cut in round 4 (as `library`), and
// one snapshot whose version is the code pool's, so nothing changes for runs
// in progress.
import type { MvpContent } from "../../../src/mvp/contract.js";
import { ROWS, type Row } from "../../../src/mvp/units.js";
import { contentOf } from "./content.js";
import type { MvpStore, PoolStint, StoredUnit } from "./store.js";

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
