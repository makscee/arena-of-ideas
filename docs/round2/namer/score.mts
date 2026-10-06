// Scores bench answers with the real filter (cleanModelName from the checkout)
// plus the clash rule (a base unit's name is refused) and example echoes.
import { readFileSync } from "node:fs";
import { mvpContent } from "../../../server/src/mvp/content.ts";
import { cleanModelName, NAMER_EXAMPLES } from "../../../server/src/mvp/fusions.ts";

const units = new Map(mvpContent().units.map((u) => [u.id, u]));
const unitNames = new Set([...units.values()].map((u) => u.name.toLowerCase()));
const echoes = new Set([...NAMER_EXAMPLES.map((e) => e.name), "Fangwarden", "Stormmender", "Gravewing", "Almscutter"].map((s) => s.toLowerCase()));
const files = process.argv.slice(2);
// Odd-name heuristics (a hint, eyeballed after): CamelCase kept as one word,
// a whole unit name (5+ letters) inside, over 14 letters, a plain dictionary
// word (generic, not a new creature), or a single word that is neither a
// dictionary word nor 2-3 dictionary words of 4+ letters glued (gibberish).
const dict = new Set(readFileSync("/usr/share/dict/words", "utf8").split("\n").map((w) => w.toLowerCase()).filter((w) => w.length >= 3));
const longUnitNames = [...units.values()].map((u) => u.name.toLowerCase().replace(/[^a-z]/g, "")).filter((n) => n.length >= 5);
const compound = (w: string, depth = 0): boolean => {
  if (dict.has(w) && depth > 0) return true;
  if (depth >= 3) return false;
  for (let i = depth === 0 ? 3 : 4; i <= w.length - 3; i++) if (dict.has(w.slice(0, i)) && (compound(w.slice(i), depth + 1) || dict.has(w.slice(i)))) return true;
  return false;
};
function oddness(name: string): string | null {
  const w = name.toLowerCase().replace(/[^a-z]/g, "");
  if (/[a-z][A-Z]/.test(name)) return "camel";
  const u = longUnitNames.find((n) => w.includes(n));
  if (u) return "has unit name " + u;
  if (/[ -]/.test(name)) return null;
  if (w.length > 14) return "long";
  if (dict.has(w)) return "plain word";
  if (!compound(w)) return "gibberish?";
  return null;
}
for (const file of files) {
  const rows = readFileSync(file, "utf8").trim().split("\n").map((l) => JSON.parse(l));
  for (const variant of ["real", "one", "one2"]) {
    const rs = rows.filter((r) => r.variant === variant);
    if (!rs.length) continue;
    let valid = 0, single = 0, echo = 0;
    const odd: string[] = [];
    const refused: string[] = [], names: string[] = [], multi: string[] = [], echoed: string[] = [];
    const byPair = new Map<string, boolean>();
    const seen = new Map<string, number>();
    for (const r of rs) {
      const name = cleanModelName(r.raw, units.get(r.first), units.get(r.second));
      const clash = name !== null && unitNames.has(name.toLowerCase());
      const ok = name !== null && !clash;
      const key = r.first + ">" + r.second;
      byPair.set(key, (byPair.get(key) ?? false) || ok);
      if (!ok) { refused.push(`${r.a}+${r.b}: ${JSON.stringify(r.raw)}${clash ? " (unit name)" : ""}`); continue; }
      valid++;
      if (echoes.has(name!.toLowerCase())) { echo++; echoed.push(`${r.a}+${r.b}: ${name}`); }
      seen.set(name!.toLowerCase(), (seen.get(name!.toLowerCase()) ?? 0) + 1);
      const o = oddness(name!);
      if (o) odd.push(`${name} (${o})`);
      if (!/[ -]/.test(name!)) single++; else multi.push(name!);
      names.push(`${r.a}+${r.b}=${name}`);
    }
    const dupes = [...seen].filter(([, c]) => c > 1).map(([n, c]) => `${n}x${c}`);
    const lats = rs.map((r) => r.s).sort((a, b) => a - b);
    const pairsOk = [...byPair.values()].filter(Boolean).length;
    console.log(`\n### ${rows[0].model} | ${variant}`);
    console.log(`asks ${rs.length} | valid ${valid} (${Math.round((100 * valid) / rs.length)}%) | pairs with a valid name in ${rs.length / byPair.size} asks: ${pairsOk}/${byPair.size} | single-word ${single}/${valid} (${valid ? Math.round((100 * single) / valid) : 0}%) | odd ${odd.length} | example echoes ${echo} | repeated names ${dupes.join(", ") || "none"} | latency med ${lats[lats.length >> 1]} s, p90 ${lats[Math.floor(lats.length * 0.9)]} s`);
    console.log(`REFUSED: ${refused.join(" | ")}`);
    console.log(`ODD: ${odd.join(", ")}`);
    console.log(`MULTI: ${multi.join(", ")}`);
    console.log(`ECHO: ${echoed.join(", ")}`);
    console.log(`NAMES: ${names.join(", ")}`);
  }
}
