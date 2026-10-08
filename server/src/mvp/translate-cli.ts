/**
 * Russian names and lines for the stored units (M4-4, mission #810).
 *
 *   npm run mvp:translate -- --db <path> [--dry-run] [--fake] [--out <page.md>]
 *
 * Asks Claude (`claude -p`, as the idea reader does; ARENA_IDEA_MODEL, default
 * sonnet; ARENA_CLAUDE_BIN) for each unit without one, checks each answer
 * (./translate.ts ruTextProblem) and asks again once for those refused. Writes
 * the review page (default docs/mission4/names-ru.md) and, without --dry-run,
 * each unit's `texts.ru` beside its row: the content version stays. `--fake`
 * spells the English in Cyrillic, for tests and trying the script. Safe to run
 * twice: units that have a Russian text keep it.
 */
import { writeFileSync } from "node:fs";
import { SqliteMvpStore } from "./sqlite-store.js";
import { claudeTranslator, fakeTranslator, reviewPage, translateUnits } from "./translate.js";

const args = process.argv.slice(2);
const flag = (name: string) => args.includes(name);
const value = (name: string) => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
};

async function main(): Promise<number> {
  const db = value("--db");
  if (!db) {
    console.error("usage: npm run mvp:translate -- --db <path> [--dry-run] [--fake] [--out <page.md>]");
    return 2;
  }
  const dryRun = flag("--dry-run");
  const out = value("--out") ?? "docs/mission4/names-ru.md";
  const store = new SqliteMvpStore(db);
  const before = store.currentPool()?.version;
  const tr = flag("--fake")
    ? fakeTranslator()
    : claudeTranslator({ model: process.env.ARENA_IDEA_MODEL || "sonnet", bin: process.env.ARENA_CLAUDE_BIN || "claude", timeoutMs: Number(process.env.ARENA_IDEA_TIMEOUT_MS) || 300_000 });
  const r = await translateUnits(store, tr, { dryRun, log: (l) => console.log(l) });
  writeFileSync(out, reviewPage(r, { dryRun, at: new Date().toISOString().slice(0, 10) }));
  const after = store.currentPool()?.version;
  console.log(`${tr.kind}: ${r.done.length} translated${dryRun ? " (dry run: nothing written)" : ""}, ${r.kept.length} kept, ${r.failed.length} failed; review page ${out}`);
  for (const f of r.failed) console.log(`  failed ${f.unit.unitId}: ${f.why}`);
  console.log(`content version ${before} → ${after}${before === after ? " (unchanged)" : " CHANGED"}`);
  return r.failed.length || before !== after ? 1 : 0;
}

main().then((code) => process.exit(code), (e) => {
  console.error(e);
  process.exit(1);
});
