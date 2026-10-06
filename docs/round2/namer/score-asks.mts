// R2-4 (#594): scores bench-asks.py output the way the server takes an answer:
// cleanModelName, then refused when it is a unit's name or holds one
// (hasUnitName). Per variant: valid, one word, starting with S, the hint obeyed,
// and the odd names (score.mts's heuristic, read by eye after).
// Usage: npx tsx docs/round2/namer/score-asks.mts out-qwen3-4b-s.jsonl
import { readFileSync } from "node:fs";
import { mvpContent } from "../../../server/src/mvp/content.ts";
import { cleanModelName, hasUnitName, isOneWord } from "../../../server/src/mvp/fusions.ts";

const units = new Map(mvpContent().units.map((u) => [u.id, u]));
const unitNames = new Set([...units.values()].map((u) => u.name.toLowerCase()));
const dict = new Set(readFileSync("/usr/share/dict/words", "utf8").split("\n").map((w) => w.toLowerCase()).filter((w) => w.length >= 3));
const compound = (w: string, depth = 0): boolean => {
  if (dict.has(w) && depth > 0) return true;
  if (depth >= 3) return false;
  for (let i = depth === 0 ? 3 : 4; i <= w.length - 3; i++) if (dict.has(w.slice(0, i)) && (compound(w.slice(i), depth + 1) || dict.has(w.slice(i)))) return true;
  return false;
};
const oddness = (name: string) => {
  const w = name.toLowerCase().replace(/[^a-z]/g, "");
  if (!isOneWord(name)) return null;
  if (w.length > 14) return "long";
  if (dict.has(w)) return "plain word";
  if (!compound(w)) return "gibberish?";
  return null;
};
const pct = (x: number, of: number) => `${of ? Math.round((100 * x) / of) : 0}%`;
const rows = readFileSync(process.argv[2]!, "utf8").trim().split("\n").map((l) => JSON.parse(l));
for (const variant of [...new Set(rows.map((r) => r.variant))]) {
  const rs = rows.filter((r) => r.variant === variant);
  const names: string[] = [], refused: string[] = [], odd: string[] = [], multi: string[] = [];
  let s = 0, obeyed = 0, hinted = 0;
  const seen = new Map<string, number>();
  for (const r of rs) {
    const name = cleanModelName(r.raw, units.get(r.first), units.get(r.second));
    if (!name || unitNames.has(name.toLowerCase()) || hasUnitName(name, units.values())) { refused.push(`${r.a}+${r.b}: ${JSON.stringify(r.raw.trim())}`); continue; }
    names.push(`${r.a}+${r.b}=${name}`);
    seen.set(name.toLowerCase(), (seen.get(name.toLowerCase()) ?? 0) + 1);
    if (!isOneWord(name)) multi.push(name);
    if (name[0] === "S") s++;
    if (r.letter) { hinted++; if (name[0] === r.letter) obeyed++; }
    const o = oddness(name);
    if (o) odd.push(`${name} (${o})`);
  }
  const valid = names.length;
  const dupes = [...seen].filter(([, c]) => c > 1).map(([n, c]) => `${n}x${c}`);
  const lats = rs.map((r) => r.s).sort((a, b) => a - b);
  console.log(`\n### ${rows[0].model} | ${variant}`);
  console.log(`asks ${rs.length} | valid ${valid} (${pct(valid, rs.length)}) | one word ${valid - multi.length}/${valid} (${pct(valid - multi.length, valid)}) | starts with S ${s} (${pct(s, valid)})${hinted ? ` | letter obeyed ${pct(obeyed, hinted)}` : ""} | odd ${odd.length} | repeated ${dupes.join(", ") || "none"} | latency med ${lats[lats.length >> 1]} s`);
  console.log(`REFUSED: ${refused.join(" | ")}`);
  console.log(`MULTI: ${multi.join(", ")}`);
  console.log(`ODD: ${odd.join(", ")}`);
  console.log(`NAMES: ${names.join(", ")}`);
}
