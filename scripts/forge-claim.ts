// Claims designed fusions in a world's Codex by hand (makscee/void-board#1065):
// each named pair's candidate becomes its canonical design, credited to one
// player ("the forge" by default), as a human's claim at the forge would.
//
//   node --import tsx/esm scripts/forge-claim.ts --db data/workshop.db [--by "the forge"] hourglass+mirror="Déjà Vu" …
//
// A pair is given as its two unit ids (either order) and the candidate's EN
// name as stored. A pair that is already discovered is left alone.
import { codexPair, pairState } from "../src/mvp/codex.js";
import type { PlayerRef } from "../src/mvp/contract.js";
import { SqliteMvpStore } from "../server/src/mvp/sqlite-store.js";

const args = process.argv.slice(2);
const opt = (name: string): string | undefined => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 ? args.splice(i, 2)[1] : undefined;
};
const db = opt("db") ?? process.env.MVP_DB;
const byName = opt("by") ?? "the forge";
if (!db || args.length === 0) {
  console.error('usage: forge-claim.ts --db <world.db> [--by "the forge"] a+b="Candidate name" …');
  process.exit(2);
}
const store = new SqliteMvpStore(db);
const id = byName.toLowerCase().replace(/[^a-z0-9]+/g, "-");
const by: PlayerRef = store.player(id) ?? { id, name: byName, bot: false };
if (!store.player(id)) store.addPlayer(by);
let failed = 0;
for (const a of args) {
  const m = /^([a-z0-9-]+)\+([a-z0-9-]+)=(.+)$/.exec(a);
  if (!m) {
    console.error(`skip ${a}: not a+b="name"`);
    failed++;
    continue;
  }
  const pair = codexPair(m[1]!, m[2]!);
  const stored = store.codexPair(pair.a, pair.b);
  const state = pairState(stored);
  if (state !== "ready") {
    console.error(`${pair.a}+${pair.b}: ${state}, left alone`);
    if (state === "unforged") failed++;
    continue;
  }
  const pick = stored!.candidates.findIndex((c) => c.name === m[3]);
  if (pick < 0) {
    console.error(`${pair.a}+${pair.b}: no candidate "${m[3]}" (has ${stored!.candidates.map((c) => c.name).join(", ")})`);
    failed++;
    continue;
  }
  store.claimCodexPair(pair.a, pair.b, { pick, by, at: new Date().toISOString() });
  console.log(`${pair.a}+${pair.b}: discovered as ${m[3]}, credited to ${by.name}`);
}
process.exit(failed ? 1 : 0);
