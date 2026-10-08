/**
 * The rotation's dry run (M2-10, mission #735): what the coming day end would
 * enter and leave, and why, against a copy of a server's SQLite file. Writes
 * nothing to the world (opening a file the server never opened adds its
 * migrations' tables, as the server would).
 *
 *   npm run mvp:rotate -- --db <path> --dry-run [--json]
 *
 * The orchestrator runs it on a copy of the live DB before turning
 * MVP_ROTATION=1 on. Rules are the server's (MVP_RULES): at most 3 entrants a
 * day, leavers live at least 14 days, play read from the last 14 days.
 */
import { MVP_RULES } from "../../../src/mvp/contract.js";
import { describePlan, rotationPlan } from "./rotation.js";
import { SqliteMvpStore } from "./sqlite-store.js";

const args = process.argv.slice(2);
const at = args.indexOf("--db");
const db = at >= 0 ? args[at + 1] : undefined;
if (!db || !args.includes("--dry-run")) {
  console.error("usage: npm run mvp:rotate -- --db <path> --dry-run [--json]   (only a dry run: the server rotates at the day end with MVP_ROTATION=1)");
  process.exit(1);
}
const store = new SqliteMvpStore(db);
try {
  const plan = rotationPlan({ store, rules: MVP_RULES });
  console.log(args.includes("--json") ? JSON.stringify(plan, null, 2) : [...describePlan(plan, MVP_RULES), "dry run: nothing written"].join("\n"));
} catch (e) {
  console.error((e as Error).message);
  process.exitCode = 1;
} finally {
  store.close();
}
