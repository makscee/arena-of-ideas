// Stores a night batch's JSON in a world's Codex (makscee/void-board#1065):
// what `mvp:night --db` does after designing, for a batch designed elsewhere
// (on m4, say) or merged by hand. Each pair with ready candidates becomes
// ready to claim; a discovered pair is left alone.
//
//   node --import tsx/esm scripts/night-store.ts --json night-1065.json --db data/workshop.db [--only a+b,c+d]
import { readFileSync } from "node:fs";
import type { PairResult } from "../server/src/mvp/night-batch.js";
import { storeNight } from "../server/src/mvp/night-codex.js";
import { SqliteMvpStore } from "../server/src/mvp/sqlite-store.js";

const arg = (name: string): string | undefined => {
  const i = process.argv.indexOf(`--${name}`);
  return i > 0 ? process.argv[i + 1] : undefined;
};
const json = arg("json");
const db = arg("db") ?? process.env.MVP_DB;
if (!json || !db) {
  console.error("usage: night-store.ts --json <night.json> --db <world.db> [--only a+b,…]");
  process.exit(2);
}
const batch = JSON.parse(readFileSync(json, "utf8")) as { results: PairResult[]; parents: Record<string, { emoji?: string }> };
const only = arg("only")?.split(",");
const results = only ? batch.results.filter((r) => only.includes(r.pair.key)) : batch.results;
const stored = storeNight(new SqliteMvpStore(db), results, new Date(), (a, b) => `${batch.parents[a]?.emoji ?? ""}${batch.parents[b]?.emoji ?? ""}` || undefined);
console.log(`codex: ${stored.ready.length} pairs ready to claim, ${stored.claimed.length} already discovered and left alone, ${stored.words} word requests`);
