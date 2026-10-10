/**
 * The daily post's dry run (M4-8, mission #810): the messages the bot would
 * post for a day, against a copy of a server's SQLite file, and the share
 * card each would carry, written to a PNG. Sends nothing and writes nothing
 * to the world (opening a file the server never opened adds its migrations'
 * tables, as the server would).
 *
 *   npm run mvp:post -- --db <path> [--day <seq>] [--langs ru,en] [--out <dir>] --dry-run
 *
 * --day defaults to the current day (its post tells of the day before it);
 * --langs as ARENA_TELEGRAM_POST_LANGS; images go to --out (default .) as
 * daily-post-<seq>-<langs>.png. The orchestrator shows Maks a dry run before
 * turning ARENA_TELEGRAM_POST=1 on.
 */
import { writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { dailyMessages, postGroups } from "./daily-post.js";
import { poolContent } from "./pool.js";
import { mvpRuntime } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";

const args = process.argv.slice(2);
const opt = (name: string) => {
  const at = args.indexOf(name);
  return at >= 0 ? args[at + 1] : undefined;
};
const db = opt("--db");
if (!db || !args.includes("--dry-run")) {
  console.error("usage: npm run mvp:post -- --db <path> [--day <seq>] [--langs ru,en] [--out <dir>] --dry-run   (only a dry run: the server posts after the day end with ARENA_TELEGRAM_POST=1)");
  process.exit(1);
}
const store = new SqliteMvpStore(db);
try {
  // A clock stopped at the stored day's start, so reading the day never ends it.
  const cur = store.currentDay();
  const rt = mvpRuntime({ content: poolContent(store), store, now: () => new Date(cur?.startedAt ?? Date.now()) });
  const seq = Number(opt("--day") ?? rt.today().seq);
  if (!Number.isInteger(seq) || seq < 2) throw new Error(`--day must be a day after the first (got ${opt("--day") ?? seq})`);
  const publicUrl = (process.env.ARENA_PUBLIC_URL ?? "https://arena.makscee.ru/arena").replace(/\/$/, "");
  const messages = dailyMessages(rt, seq, postGroups(opt("--langs")), publicUrl);
  for (const m of messages) {
    console.log(`----- day ${seq}, ${m.key} (${m.text.length} characters) -----`);
    console.log(m.text);
    if (m.png) {
      const file = resolve(opt("--out") ?? ".", `daily-post-${seq}-${m.key}.png`);
      writeFileSync(file, m.png);
      console.log(`[photo: ${file}]`);
    } else console.log("[no photo: no champion]");
    console.log("");
  }
  console.log("dry run: nothing sent");
} catch (e) {
  console.error((e as Error).message);
  process.exitCode = 1;
} finally {
  store.close();
}
