/**
 * The open reports of players' names and player-made units (#1271), for the
 * admin: who reported what, with the reason. Reads the world's SQLite file
 * and changes nothing.
 *
 *   npm run mvp:reports -- --db <path> [--all]
 */
import { SqliteMvpStore } from "./sqlite-store.js";

const args = process.argv.slice(2);
const at = args.indexOf("--db");
const db = at >= 0 ? args[at + 1] : process.env.MVP_DB;
if (!db || db === ":memory:") {
  console.error("usage: npm run mvp:reports -- --db <path> [--all]   (or MVP_DB=<path>)");
  process.exit(1);
}
const store = new SqliteMvpStore(db);
try {
  const list = store.reports({ open: !args.includes("--all") });
  if (!list.length) console.log("no open reports");
  for (const r of list) {
    const reporter = store.player(r.reporterId)?.name ?? r.reporterId;
    const target =
      r.kind === "name"
        ? `name of @${store.player(r.targetId)?.name ?? "?"} (${r.targetId})`
        : (() => {
            const u = store.unit(r.targetId);
            const by = u?.authorId ? store.player(u.authorId)?.name : undefined;
            return `unit ${u?.row.name ?? r.targetId}${by ? ` by @${by}` : ""} (${r.targetId})`;
          })();
    console.log(`${r.at}  ${r.kind}  ${target}  reported by @${reporter}${r.reason ? `: ${r.reason}` : ""}`);
  }
} finally {
  store.close();
}
