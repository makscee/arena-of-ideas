// MVP content (mission #574). The pool is authored in src/mvp/units.ts
// (slice 7: ~80 units, both forms, tuned by `npm run mvp:meta`); this wraps it
// in the MvpContent shape with a version derived from the data, so ghosts and
// logs carry the content they were played with. Since M2-1 the server builds
// it from the DB's current pool (./pool.ts), through contentOf.
import { createHash } from "node:crypto";
import type { MvpContent } from "../../../src/mvp/contract.js";
import { mvpPool, ROWS, type Row } from "../../../src/mvp/units.js";

/** The content of these rows, in this order. */
export function contentOf(rows: Row[]): MvpContent {
  const body = mvpPool(rows);
  const version = "mvp-" + createHash("sha256").update(JSON.stringify(body)).digest("hex").slice(0, 10);
  return { version, ...body };
}

/** The pool in code (ROWS): what a fresh DB is seeded with, and what tests play. */
export function mvpContent(): MvpContent {
  return contentOf(ROWS);
}
