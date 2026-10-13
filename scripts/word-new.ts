/**
 * The word card's scaffold (#1043 §7, #1052). A forged word is one file in
 * src/words/ plus its entry in index.ts and its kind in types.ts; this writes
 * all three from the template (src/words/scaffold.ts) and lists what is left.
 *
 *   npm run word:new -- <selector|condition|amount|effect> <kind> "<what it means>" [--word "<requested word>" --card <issue>]
 *
 * With --word and --card it also records the word as forged in
 * registry/forged-words.json (the Dictionary reads it).
 */
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { WORD_CLASSES, addToUnion, isWordKind, registerInIndex, wordSource, type WordClass } from "../src/words/scaffold.js";

const root = join(import.meta.dirname, "..");
const args = process.argv.slice(2);
const flag = (name: string): string | undefined => {
  const i = args.indexOf(`--${name}`);
  if (i < 0) return undefined;
  const [v] = args.splice(i, 2).slice(1);
  return v;
};
const word = flag("word");
const card = flag("card");
const [cls, kind, doc] = args as [WordClass, string, string];
if (!WORD_CLASSES.includes(cls) || !kind || !isWordKind(kind) || !doc) {
  console.error('usage: npm run word:new -- <selector|condition|amount|effect> <kind> "<what it means>" [--word "<requested word>" --card <issue>]');
  process.exit(2);
}
const file = join(root, "src/words", `${kind}.ts`);
if (existsSync(file)) {
  console.error(`src/words/${kind}.ts already exists`);
  process.exit(1);
}
const indexPath = join(root, "src/words/index.ts");
const typesPath = join(root, "src/types.ts");
const index = registerInIndex(readFileSync(indexPath, "utf8"), cls, kind);
const types = addToUnion(readFileSync(typesPath, "utf8"), cls, kind);
writeFileSync(file, wordSource(cls, kind, doc));
writeFileSync(indexPath, index);
writeFileSync(typesPath, types);
if (word && card) {
  const p = join(root, "registry/forged-words.json");
  const list = existsSync(p) ? JSON.parse(readFileSync(p, "utf8")) : [];
  list.push({ word, kind, card: Number(card) });
  writeFileSync(p, `${JSON.stringify(list, null, 2)}\n`);
}
console.log(`wrote src/words/${kind}.ts, registered it in src/words/index.ts, added { kind: "${kind}" } to src/types.ts.
Left to do (docs/words/CARD.md has the whole list):
  1. fill the TODOs in src/words/${kind}.ts (apply, EN and RU templates, example battle, trace line)
  2. if it has parameters: a case in validate.ts; an effect that lands in the kernel: a case in battle.ts
  3. UPDATE_WORD_GOLDENS=1 npx vitest run src/words/words.test.ts -t ${kind}   (writes the golden: read it)
  4. npm run mvp:meta on the base commit and on yours; say the difference in the PR`);
