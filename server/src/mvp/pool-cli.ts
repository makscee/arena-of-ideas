/**
 * The live pool on the host (M2-2, mission #735), against the server's SQLite
 * file (safe while the server runs: it reads the current pool on every use,
 * and a new snapshot ends no run).
 *
 *   npm run mvp:pool -- show
 *   npm run mvp:pool -- sync-seed [--dry-run]
 *   npm run mvp:pool -- swap <out-id> <in-id>     (MVP_DEV=1 only)
 *
 * `sync-seed` writes the seed units' rows from code (src/mvp/units.ts ROWS)
 * into the DB and adds a pool snapshot with them, printing which units
 * changed. Since M2-1 the DB is the pool's source of truth, so a balance fix
 * in ROWS reaches a seeded world only through it; the orchestrator runs it on
 * a deploy that changed ROWS. Runs in progress keep the pool they started on.
 * Safe to run twice: the second run finds nothing to change.
 * `swap` (dev only) takes one live unit out for a stored one (the library's
 * Rat, say), to try a pool change by hand. Env: MVP_DB (default data/arena-mvp.db).
 */
import { swapUnit, syncSeed } from "./pool.js";
import { SqliteMvpStore } from "./sqlite-store.js";

const [cmd, ...rest] = process.argv.slice(2);
const store = new SqliteMvpStore(process.env.MVP_DB ?? "data/arena-mvp.db");

function main(): number {
  const pool = store.currentPool();
  if (!pool) {
    console.error("no pool yet: start the server once to seed it");
    return 1;
  }
  if (cmd === "show") {
    console.log(`pool ${pool.version} (day ${pool.daySeq}, ${pool.unitIds.length} units): ${pool.unitIds.join(" ")}`);
    const library = store.units({ status: "library" }).map((u) => u.unitId);
    console.log(`library (${library.length}): ${library.join(" ")}`);
    return 0;
  }
  if (cmd === "sync-seed") {
    const dryRun = rest.includes("--dry-run");
    const r = syncSeed(store, new Date(), { dryRun });
    if (!r.changed.length && !r.added.length) {
      console.log(`nothing to change: the DB's seed units match the code (pool ${pool.version})`);
      return 0;
    }
    if (r.changed.length) console.log(`changed (${r.changed.length}): ${r.changed.join(" ")}`);
    if (r.added.length) console.log(`added (${r.added.length}): ${r.added.join(" ")}`);
    console.log(dryRun ? `dry run: would make pool ${r.version} (now ${pool.version}); nothing written` : `pool ${pool.version} → ${r.version}; runs in progress keep their pool`);
    return 0;
  }
  if (cmd === "swap") {
    if (process.env.MVP_DEV !== "1") {
      console.error("swap is a dev tool: set MVP_DEV=1 (never on the live game)");
      return 1;
    }
    const [out, into] = rest;
    if (!out || !into) {
      console.error("usage: npm run mvp:pool -- swap <out-id> <in-id>");
      return 1;
    }
    const version = swapUnit(store, out, into, new Date());
    console.log(`swapped ${out} → ${into}: pool ${pool.version} → ${version}; runs in progress keep their pool`);
    return 0;
  }
  console.error("usage: npm run mvp:pool -- show | sync-seed [--dry-run] | swap <out-id> <in-id>");
  return 1;
}

try {
  process.exitCode = main();
} catch (e) {
  console.error((e as Error).message);
  process.exitCode = 1;
} finally {
  store.close();
}
