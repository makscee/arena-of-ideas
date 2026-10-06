// R2-4 (#594): the asks for the "S" habit bench, from the checkout's real
// namer messages. The same 40 pairs as pairs.json, 2 asks each, per variant:
//   t08:  the one2 prompt at 0.8 (the baseline)
//   t10:  the one2 prompt at 1.0
//   hint: the one2 prompt at 0.8 plus "Start with the letter X." (random X from NAMER_LETTERS)
// Usage: npx tsx docs/round2/namer/asks.mts asks.json
import { readFileSync, writeFileSync } from "node:fs";
import { mvpContent } from "../../../server/src/mvp/content.ts";
import { namerMessages, NAMER_LETTERS } from "../../../server/src/mvp/fusions.ts";

const units = new Map(mvpContent().units.map((u) => [u.id, u]));
const pairs = JSON.parse(readFileSync(new URL("./pairs.json", import.meta.url), "utf8")) as { first: string; second: string }[];
let seed = 594;
const rnd = () => ((seed = (seed * 1103515245 + 12345) % 2147483648) / 2147483648);
const asks = [];
for (const [variant, temp, hinted] of [["t08", 0.8, false], ["t10", 1.0, false], ["hint", 0.8, true]] as const)
  for (const p of pairs)
    for (let k = 0; k < 2; k++) {
      const a = units.get(p.first)!, b = units.get(p.second)!;
      const letter = hinted ? NAMER_LETTERS[Math.floor(rnd() * NAMER_LETTERS.length)] : undefined;
      asks.push({ variant, temp, first: a.id, second: b.id, a: a.name, b: b.name, k, letter, messages: namerMessages(a, b, letter) });
    }
writeFileSync(process.argv[2]!, JSON.stringify(asks));
console.log(asks.length, "asks");
