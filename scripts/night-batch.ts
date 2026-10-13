// One night batch run (makscee/void-board#1050): tonight's list, the designs,
// the checks, the stored candidates and a report page.
//
//   npm run mvp:night -- --pairs 5 --out /tmp/night [--report ~/reports/arena-night.html]
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

const arg = (name: string, dflt?: string): string | undefined => {
  const i = process.argv.indexOf(`--${name}`);
  return i > 0 ? process.argv[i + 1] : dflt;
};

const EMOJI: Record<string, string> = { Hourglass: "⏳", Mirror: "🪞", Candle: "🕯️", Gambler: "🎲", Nautilus: "🐚", Magnet: "🧲", Raven: "🐦‍⬛", Seed: "🌱" };
const UNITS: UnitDef[] = [Hourglass, Mirror, Candle, Gambler, Nautilus, Magnet, Raven, Seed];

/** The registry entries a unit's rule reaches, statuses through abilities included. */
function parentOf(def: UnitDef): Parent {
  const abilities: Parent["abilities"] = {};
  const statuses: Parent["statuses"] = {};
  const seen = new Set<string>();
  const visit = (x: unknown): void => {
    if (Array.isArray(x)) return x.forEach(visit);
    if (!x || typeof x !== "object") return;
    for (const [k, v] of Object.entries(x)) {
      if (k === "status" && typeof v === "string" && FIXTURE_STATUSES[v] && !seen.has(`s:${v}`)) {
        seen.add(`s:${v}`);
        statuses[v] = FIXTURE_STATUSES[v]!;
        visit(FIXTURE_STATUSES[v]);
      } else if (k === "abilities" && Array.isArray(v)) {
        for (const id of v) {
          if (typeof id === "string" && FIXTURE_ABILITIES[id] && !seen.has(`a:${id}`)) {
            seen.add(`a:${id}`);
            abilities[id] = FIXTURE_ABILITIES[id]!;
            visit(FIXTURE_ABILITIES[id]);
          }
        }
      } else visit(v);
    }
  };
  visit(def);
  return { id: def.name, emoji: EMOJI[def.name], def, abilities, statuses };
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
const designer = designerFromEnv();
const now = Date.now();
const list = pickPairs({ units: UNITS.map((u, i) => ({ id: u.name, addedAt: now - i * 3600_000 })), now, size: pairs, small: pairs < 50 });
const parents = Object.fromEntries(UNITS.map((u) => [u.name, parentOf(u)]));
console.log(`night batch ${day}: ${list.length} pairs, designer ${designer.kind}`);
const started = Date.now();
const results = await runBatch(list, { designer, field: trialField(), parents, parallel: Number(arg("parallel", "5")), log: (l) => console.log(`  ${l}`) });
const batch = { day, designer: designer.kind, ranAt: new Date().toISOString(), seconds: Math.round((Date.now() - started) / 1000), parents: Object.fromEntries(Object.entries(parents).map(([k, p]) => [k, { emoji: p.emoji }])), results };
mkdirSync(out, { recursive: true });
writeFileSync(join(out, `night-${day}.json`), JSON.stringify(batch, null, 1));
const report = arg("report", join(out, `night-${day}.html`))!;
writeFileSync(report, nightReport(batch));
const all = results.flatMap((r) => r.candidates);
console.log(`stored ${all.filter((c) => c.state === "ready").length} ready, ${all.filter((c) => c.state === "word").length} waiting for a word, ${all.filter((c) => c.state === "rejected").length} rejected, ${results.filter((r) => r.error).length} pairs failed; ${batch.seconds}s`);
console.log(`json ${join(out, `night-${day}.json`)}\nreport ${report}`);
