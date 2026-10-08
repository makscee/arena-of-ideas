// MVP content (mission #574). The pool is authored in src/mvp/units.ts
// (slice 7: ~80 units, both forms, tuned by `npm run mvp:meta`); this wraps it
// in the MvpContent shape with a version derived from the data, so ghosts and
// logs carry the content they were played with.
import { createHash } from "node:crypto";
import type { MvpContent } from "../../../src/mvp/contract.js";
import { mvpPool } from "../../../src/mvp/units.js";

export function mvpContent(): MvpContent {
  const body = mvpPool();
  const version = "mvp-" + createHash("sha256").update(JSON.stringify(body)).digest("hex").slice(0, 10);
  return { version, ...body };
}
