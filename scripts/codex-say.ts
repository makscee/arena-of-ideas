// Writes the card sentence of every stored fusion design that has none
// (makscee/void-board#1122): the night designer's writer, checked against the
// composer's text as tonight's batch does (writeSay). Designs whose sentence
// doesn't match keep reading the composer, and are listed.
//
//   ARENA_NIGHT_DESIGNER=claude node --import tsx/esm scripts/codex-say.ts --db <world.db> [--dump says.json] [--parallel 6]
//   node --import tsx/esm scripts/codex-say.ts --db <world.db> --load says.json      (no model: apply a dump)
//
// A dump maps "a+b" → { "<design name>": {en, ru} }; --load writes those onto
// the designs of the same pair and name that still have no sentence.
import { readFileSync, writeFileSync } from "node:fs";
import type { FusionDesign } from "../src/mvp/codex.js";
import { formText } from "../src/mvp/form-text.js";
import { V2_ABILITIES, V2_STATUSES } from "../src/mvp/units-v2.js";
import { type Parent, type Say, writeSay } from "../server/src/mvp/night-batch.js";
import { designerFromEnv } from "../server/src/mvp/night-designer.js";
import { SqliteMvpStore } from "../server/src/mvp/sqlite-store.js";

const args = process.argv.slice(2);
const opt = (name: string): string | undefined => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 ? args[i + 1] : undefined;
};
const db = opt("db") ?? process.env.MVP_DB;
if (!db) {
  console.error("usage: codex-say.ts --db <world.db> [--dump says.json] [--load says.json] [--parallel 6]");
  process.exit(2);
}
const store = new SqliteMvpStore(db);
const pairs = store.codexPairs();
const dump: Record<string, Record<string, Say>> = {};
const load = opt("load") ? (JSON.parse(readFileSync(opt("load")!, "utf8")) as typeof dump) : undefined;

const ruleOf = (d: FusionDesign): Say => {
  const abilities = { ...V2_ABILITIES, ...d.abilities };
  const statuses = { ...V2_STATUSES, ...d.statuses };
  return { en: formText(d.recipe, abilities, undefined, statuses), ru: formText(d.recipe, abilities, "ru", statuses) };
};
const parent = (id: string): Parent => ({ id, def: { name: id, base: { hp: 1, pwr: 1 } }, abilities: {}, statuses: {} });

const jobs = pairs.flatMap((p) => p.candidates.map((d, i) => ({ p, d, i }))).filter((j) => !j.d.recipe.say);
const designer = designerFromEnv();
let wrote = 0;
const missed: string[] = [];
let next = 0;
const work = async (): Promise<void> => {
  for (let k = next++; k < jobs.length; k = next++) {
    const { p, d } = jobs[k]!;
    const key = `${p.a}+${p.b}`;
    const say = load
      ? load[key]?.[d.name]
      : await writeSay(designer, parent(p.a), parent(p.b), { lean: d.lean === "a" ? "A" : d.lean === "b" ? "B" : "twist", intent: d.flavour ?? d.name, rule: {}, abilities: {} }, ruleOf(d), (l) => console.error(`${key} ${d.name}: ${l}`));
    if (say) {
      d.recipe.say = say;
      (dump[key] ??= {})[d.name] = say;
      wrote++;
      console.log(`${key} ${d.name}: ${say.en}`);
    } else missed.push(`${key} ${d.name}`);
  }
};
await Promise.all(Array.from({ length: Math.max(1, Number(opt("parallel")) || (load ? 1 : 6)) }, work));
// write back pair by pair, from a fresh read: a claim made meanwhile is kept
for (const p of pairs) {
  const now = store.codexPair(p.a, p.b);
  if (!now) continue;
  let changed = false;
  now.candidates.forEach((d, i) => {
    const say = p.candidates[i]?.name === d.name ? p.candidates[i]!.recipe.say : undefined;
    if (say && !d.recipe.say) {
      d.recipe.say = say;
      changed = true;
    }
  });
  if (changed) store.putCodexPair(now);
}
if (opt("dump")) writeFileSync(opt("dump")!, JSON.stringify(dump, null, 1));
console.log(`${wrote} sentences written, ${missed.length} left on the composer${missed.length ? `: ${missed.join("; ")}` : ""}`);
