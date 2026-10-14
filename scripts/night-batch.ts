// One night batch run (makscee/void-board#1050): tonight's list, the designs,
// the checks, the stored candidates and a report page.
//
//   npm run mvp:night -- --pairs 5 --out /tmp/night [--report ~/reports/arena-night.html] [--db data/workshop.db]
//   npm run mvp:night -- --pool v2 --list hourglass+gambler,mirror+trickster --out /tmp/night [--db …]
//
// --pool v2 (#1065): the parents are the rules v2 pool's Awoken units
// (src/mvp/units-v2.ts) and the field is seeded lines of them; --list names
// the pairs outright instead of tonight's pick.
//
// With --db (or MVP_DB): the world's wishes weight tonight's list, its Codex
// pairs are the history, and the candidates land in the Codex tables (ready
// to claim) and the word queue (server/src/mvp/night-codex.ts). Pairs are
// keyed by the pool's unit ids (slug: "hourglass+mirror").
//
// ARENA_NIGHT_DESIGNER=claude uses Opus and Sonnet (`claude -p`); the default
// fake designer never calls a model. Until unit batch 1 (#1046) is in the
// pool, the parents are the eight §4 units and the field is lines of them
// (src/words/fixtures.ts); the job reads the live pool and the day's ghosts
// once the pool is v2.

import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import type { UnitDef } from "../src/types.js";
import { Candle, FIXTURE_ABILITIES, FIXTURE_STATUSES, Gambler, Hourglass, Magnet, Mirror, Nautilus, Raven, Seed } from "../src/words/fixtures.js";
import { plain, rule } from "../src/words/examples.js";
import { type Field, type Parent, pickPairs, runBatch } from "../server/src/mvp/night-batch.js";
import { designerFromEnv } from "../server/src/mvp/night-designer.js";
import { nightReport } from "../server/src/mvp/night-report.js";
import { listInputFrom, storeNight } from "../server/src/mvp/night-codex.js";
import { SqliteMvpStore } from "../server/src/mvp/sqlite-store.js";
import { slug } from "../src/mvp/units.js";
import { ROWS_V2 } from "../src/mvp/units-v2.js";
import { lineUnitOf } from "../src/mvp/forms.js";
import { toBattleDef } from "../src/mvp/fight.js";
import { contentOf } from "../server/src/mvp/content.js";
import type { AbilityRegistry, StatusRegistry } from "../src/types.js";

const arg = (name: string, dflt?: string): string | undefined => {
  const i = process.argv.indexOf(`--${name}`);
  return i > 0 ? process.argv[i + 1] : dflt;
};

const EMOJI: Record<string, string> = { Hourglass: "⏳", Mirror: "🪞", Candle: "🕯️", Gambler: "🎲", Nautilus: "🐚", Magnet: "🧲", Raven: "🐦‍⬛", Seed: "🌱" };
const UNITS: UnitDef[] = [Hourglass, Mirror, Candle, Gambler, Nautilus, Magnet, Raven, Seed];

/** The registry entries a unit's rule reaches, statuses through abilities included. */
function parentOf(def: UnitDef, ABILITIES: AbilityRegistry = FIXTURE_ABILITIES, STATUSES: StatusRegistry = FIXTURE_STATUSES, id = def.name, emoji = EMOJI[def.name]): Parent {
  const abilities: Parent["abilities"] = {};
  const statuses: Parent["statuses"] = {};
  const seen = new Set<string>();
  const visit = (x: unknown): void => {
    if (Array.isArray(x)) return x.forEach(visit);
    if (!x || typeof x !== "object") return;
    for (const [k, v] of Object.entries(x)) {
      if (k === "status" && typeof v === "string" && STATUSES[v] && !seen.has(`s:${v}`)) {
        seen.add(`s:${v}`);
        statuses[v] = STATUSES[v]!;
        visit(STATUSES[v]);
      } else if (k === "abilities" && Array.isArray(v)) {
        for (const id of v) {
          if (typeof id === "string" && ABILITIES[id] && !seen.has(`a:${id}`)) {
            seen.add(`a:${id}`);
            abilities[id] = ABILITIES[id]!;
            visit(ABILITIES[id]);
          }
        }
      } else visit(v);
    }
  };
  visit(def);
  return { id, ...(emoji ? { emoji } : {}), def, abilities, statuses };
}

const Archer = rule("Archer", 6, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Shoot");
const Brute = plain("Brute", 14, 3);
/** Lines like a round 8-12 ghost: four units, the §4 units with fillers. */
function trialField(): Field {
  const ghosts: UnitDef[][] = [];
  for (let i = 0; i < UNITS.length; i++) ghosts.push([Brute, UNITS[i]!, Archer, UNITS[(i + 3) % UNITS.length]!]);
  const breakers = [[Brute, Brute, Archer, Archer], [Nautilus, Magnet, Archer, Raven]];
  return { ghosts, breakers, abilities: FIXTURE_ABILITIES, statuses: FIXTURE_STATUSES };
}

const out = arg("out", "night-out")!;
const day = arg("day", new Date().toISOString().slice(0, 10))!;
const pairs = Number(arg("pairs", "5"));
const v2 = arg("pool") === "v2";
const v2content = v2 ? contentOf(ROWS_V2) : undefined;
const designer = designerFromEnv(process.env, v2content ? { abilities: v2content.abilities, statuses: v2content.statuses } : undefined);
const now = Date.now();
const dbPath = arg("db", process.env.MVP_DB);
const store = dbPath ? new SqliteMvpStore(dbPath) : undefined;
const requests = arg("list")?.split(",").filter(Boolean);
const units = v2content ? v2content.units.map((u, i) => ({ id: u.id, addedAt: now - i * 3600_000 })) : UNITS.map((u, i) => ({ id: slug(u.name), addedAt: now - i * 3600_000 }));
const size = requests ? requests.length : pairs;
const list = pickPairs({ units, now, size, small: requests !== undefined || size < 50, ...(store ? listInputFrom(store, units, now) : {}), ...(requests ? { requests } : {}) }).filter((p) => !requests || p.why === "requested");
/** Each v2 unit as it stands Awoken: three copies, its awoken form and stats. */
const awokenDef = (u: NonNullable<typeof v2content>["units"][number]): UnitDef => toBattleDef(lineUnitOf(u, u.id, 3));
const parents: Record<string, Parent> = v2content
  ? Object.fromEntries(v2content.units.map((u) => [u.id, parentOf(awokenDef(u), v2content.abilities, v2content.statuses, u.id, u.emoji)]))
  : Object.fromEntries(UNITS.map((u) => [slug(u.name), parentOf(u)]));
/** The v2 field: lines of four Awoken units like a round 8-12 ghost, from a fixed shuffle, plus two breakers. */
function v2Field(): Field {
  const defs = v2content!.units.map(awokenDef);
  let seed = 1065;
  const rnd = () => ((seed = (seed * 1103515245 + 12345) % 2147483648) / 2147483648);
  const ghosts: UnitDef[][] = [];
  for (let g = 0; g < 16; g++) {
    const pick = new Set<number>();
    while (pick.size < 4) pick.add(Math.floor(rnd() * defs.length));
    ghosts.push([...pick].map((i) => defs[i]!));
  }
  const by = (id: string) => defs[v2content!.units.findIndex((u) => u.id === id)]!;
  const breakers = [["nautilus", "medic", "archmage", "sniper"].map(by), ["bodyguard", "hornet", "alchemist", "drummer"].map(by)];
  return { ghosts, breakers, abilities: v2content!.abilities, statuses: v2content!.statuses };
}
console.log(`night batch ${day}: ${list.length} pairs (${list.map((p) => `${p.key} ${p.why}`).join(", ")}), designer ${designer.kind}${dbPath ? `, world ${dbPath}` : ""}`);
const started = Date.now();
const results = await runBatch(list, { designer, field: v2 ? v2Field() : trialField(), parents, parallel: Number(arg("parallel", "5")), log: (l) => console.log(`  ${l}`) });
const batch = { day, designer: designer.kind, ranAt: new Date().toISOString(), seconds: Math.round((Date.now() - started) / 1000), parents: Object.fromEntries(Object.entries(parents).map(([k, p]) => [k, p.emoji ? { emoji: p.emoji } : {}])), results };
mkdirSync(out, { recursive: true });
writeFileSync(join(out, `night-${day}.json`), JSON.stringify(batch, null, 1));
const report = arg("report", join(out, `night-${day}.html`))!;
writeFileSync(report, nightReport(batch));
if (store) {
  const stored = storeNight(store, results, new Date(), (a, b) => `${parents[a]?.emoji ?? ""}${parents[b]?.emoji ?? ""}` || undefined);
  console.log(`codex: ${stored.ready.length} pairs ready to claim (${stored.ready.join(", ")}), ${stored.claimed.length} already claimed and left alone, ${stored.words} word requests queued`);
}
const all = results.flatMap((r) => r.candidates);
console.log(`stored ${all.filter((c) => c.state === "ready").length} ready, ${all.filter((c) => c.state === "word").length} waiting for a word, ${all.filter((c) => c.state === "rejected").length} rejected, ${results.filter((r) => r.error).length} pairs failed; ${batch.seconds}s`);
console.log(`json ${join(out, `night-${day}.json`)}\nreport ${report}`);
