// Builds ~40 ordered fusion pairs from the real unit list, with the real
// namer messages and a one-word variant. Read-only on the checkout.
import { writeFileSync } from "node:fs";
import { mvpContent } from "../../../server/src/mvp/content.ts";
import { namerMessages, NAMER_SYSTEM, NAMER_EXAMPLES } from "../../../server/src/mvp/fusions.ts";

const units = mvpContent().units;
let seed = 574;
const rnd = () => ((seed = (seed * 1103515245 + 12345) % 2147483648) / 2147483648);
const pairs: [number, number][] = [];
const seen = new Set<string>();
while (pairs.length < 40) {
  const i = Math.floor(rnd() * units.length), j = Math.floor(rnd() * units.length);
  if (i === j || seen.has(`${i},${j}`)) continue;
  seen.add(`${i},${j}`);
  pairs.push([i, j]);
}

export const ONE_SYSTEM =
  "You name creatures in a fantasy auto-battler. Two fighters merge into one new creature. Invent a fresh, evocative English name for it: a single word, at most 14 letters, letters only. Do not just join or repeat the two fighters' names, use no emoji, no quotes, no explanation, and never a name from an existing game, film, book or comic.";
export const ONE_EXAMPLES = [
  { first: "Knight", second: "Wolf", name: "Fangwarden" },
  { first: "Spark", second: "Healer", name: "Stormmender" },
  { first: "Golem", second: "Raven", name: "Gravewing" },
  { first: "Thief", second: "Monk", name: "Almscutter" },
];
const ask = (a: string, b: string) => `${a} merges with ${b}. Name:`;
const oneMessages = (a: string, b: string) => [
  { role: "system", content: ONE_SYSTEM },
  ...ONE_EXAMPLES.flatMap((e) => [{ role: "user", content: ask(e.first, e.second) }, { role: "assistant", content: e.name }]),
  { role: "user", content: ask(a, b) },
];
// The examples must not use unit names (the model echoes example words).
const unitWords = new Set(units.flatMap((u) => u.name.toLowerCase().split(/\s+/)));
const clash = ONE_EXAMPLES.flatMap((e) => [e.first, e.second, e.name]).filter((w) => unitWords.has(w.toLowerCase()));
if (clash.length) throw new Error("example clash " + clash);

const out = pairs.map(([i, j]) => ({
  first: units[i]!.id, second: units[j]!.id, a: units[i]!.name, b: units[j]!.name,
  real: namerMessages(units[i]!, units[j]!),
  one: oneMessages(units[i]!.name, units[j]!.name),
}));
writeFileSync(process.argv[2]!, JSON.stringify(out, null, 1));
console.log(out.length, "pairs;", out.map((p) => `${p.a}+${p.b}`).join(", "));
console.log(NAMER_SYSTEM.length, NAMER_EXAMPLES.length);
