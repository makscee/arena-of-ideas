// MVP content (mission #574). The pool is authored in src/mvp/units.ts
// (slice 7: ~80 units, both forms, tuned by `npm run mvp:meta`); this wraps it
// in the MvpContent shape with a version derived from the data, so ghosts and
// logs carry the content they were played with. Since M2-1 the server builds
// it from the DB's current pool (./pool.ts), through contentOf.
import { createHash } from "node:crypto";
import type { MvpContent } from "../../../src/mvp/contract.js";
import { mvpPool, ROWS, type AnyRow } from "../../../src/mvp/units.js";
import { ROWS_V2 } from "../../../src/mvp/units-v2.js";

/** The content of these rows, in this order. `ids` are the units' stored ids
 * (M3-7: a version keeps its root's name, so `slug(name)` isn't its id); absent,
 * each unit's id is its name's slug. Ids equal to the slugs build the same
 * content and version as none. */
export function contentOf(rows: AnyRow[], ids?: readonly string[]): MvpContent {
  const body = mvpPool(rows);
  if (ids) body.units = body.units.map((u, i) => ({ ...u, id: ids[i] ?? u.id }));
  const version = "mvp-" + createHash("sha256").update(JSON.stringify(body)).digest("hex").slice(0, 10);
  return { version, ...body };
}

/** The rows a fresh world is seeded with: the name grammar's ROWS, or with
 * `MVP_POOL=v2` the rules v2 pool (mission #1043: the workshop plays the unit
 * batches as they land; the live game switches at the reopen). */
export function seedRows(): AnyRow[] {
  return process.env.MVP_POOL === "v2" ? ROWS_V2 : ROWS;
}

/** The pool in code (the seed rows): what a fresh DB is seeded with, and what tests play. */
export function mvpContent(): MvpContent {
  return contentOf(seedRows());
}
