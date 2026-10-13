// Tune the rules v2 pool's numbers (mission #1043, unit batch 1, #1046).
//
//   npm run mvp:tune-v2 [-- --rounds N]
//
// The v2 pool is new, so there are no live units to measure a candidate
// against (what scripts/tune-unit.ts does for the grid): the batch is tuned
// against itself. Each round runs the quick meta check on the pool
// (metaHealth: a breaker team per unit, then the equilibrium), reads every
// unit's best team's field score, and moves each unit outside the band
// around the pool's median one step back toward it: HP first (by 2 when far
// out), PWR when HP is at its bound or the unit is far out. Units with no
// strike of their own (PWR 0) keep PWR 0. It stops when every unit is inside
// the band or after --rounds rounds, then runs the full meta check once.
//
// Writes src/mvp/units-v2-numbers.ts (the numbers ROWS_V2 plays with) and
// docs/mvp/tune-v2.md and .json (each round's scores and moves, the final
// meta verdict).

import { mkdirSync, writeFileSync } from "node:fs";
import { DEFAULT_SEED, META_FULL, META_QUICK, metaHealth, type MetaResult } from "../src/mvp/meta.js";
import { ROWS_V2, type V2Row } from "../src/mvp/units-v2.js";
import { contentOf } from "../server/src/mvp/content.js";
import { measureTeam, slugOf, TUNE_DEFAULT } from "../src/mvp/tune.js";

const arg = (name: string, dflt: number) => {
  const i = process.argv.indexOf(name);
  return i < 0 ? dflt : Number(process.argv[i + 1]);
};
const ROUNDS = arg("--rounds", 6);
/** Inside the band: within this much of the pool's median field score. */
const BAND = 0.08;
/** Far out: HP moves by 2 and PWR moves too. */
const FAR = 0.16;
const PWR = { min: 1, max: 6 };
const HP = { min: 3, max: 16 };

const pct = (x: number) => `${Math.round(x * 100)}%`;
const median = (xs: number[]) => {
  const s = [...xs].sort((a, b) => a - b);
  return s.length % 2 ? s[(s.length - 1) / 2]! : (s[s.length / 2 - 1]! + s[s.length / 2]!) / 2;
};
const scoresOf = (r: MetaResult) => new Map(r.found.map((f) => [f.unit, r.fieldOf(f.team)]));

let rows: V2Row[] = ROWS_V2.map((r) => ({ ...r }));
const t0 = Date.now();
const rounds: { round: number; median: number; scores: Record<string, number>; moves: string[]; healthy: boolean }[] = [];

for (let round = 1; round <= ROUNDS; round++) {
  const r = metaHealth(contentOf(rows), DEFAULT_SEED, META_QUICK);
  const scores = scoresOf(r);
  const mid = median([...scores.values()]);
  const moves: string[] = [];
  rows = rows.map((row) => {
    const s = scores.get(slugOf(row.name)) ?? mid;
    const off = s - mid;
    if (Math.abs(off) <= BAND) return row;
    const d = off > 0 ? -1 : 1;
    const far = Math.abs(off) > FAR;
    const hp = Math.max(HP.min, Math.min(HP.max, row.hp + d * (far ? 2 : 1)));
    const pwrMoves = row.pwr > 0 && (far || hp === row.hp);
    const pwr = pwrMoves ? Math.max(PWR.min, Math.min(PWR.max, row.pwr + d)) : row.pwr;
    if (hp === row.hp && pwr === row.pwr) return row;
    moves.push(`${row.name} ${pct(s)}: ${row.pwr}/${row.hp}→${pwr}/${hp}`);
    return { ...row, pwr, hp };
  });
  rounds.push({ round, median: +mid.toFixed(3), scores: Object.fromEntries([...scores].map(([k, v]) => [k, +v.toFixed(3)])), moves, healthy: r.healthy });
  console.error(`round ${round}: median ${pct(mid)}, ${moves.length} moves, quick meta ${r.healthy ? "healthy" : "not healthy"}  ${((Date.now() - t0) / 1000).toFixed(0)}s`);
  for (const m of moves) console.error(`  ${m}`);
  if (!moves.length) break;
}

const final = metaHealth(contentOf(rows), DEFAULT_SEED, process.argv.includes("--quick") ? META_QUICK : META_FULL, (l) => console.error(l));
const scores = scoresOf(final);
const mid = median([...scores.values()]);
const share = (id: string) => final.unitShare.get(id) ?? 0;

writeFileSync(
  "src/mvp/units-v2-numbers.ts",
  [
    "// PWR and HP of the rules v2 pool, written by `npm run mvp:tune-v2`",
    "// (scripts/tune-v2.ts): the numbers come from the tune, never from the",
    "// designer. A unit missing here keeps its row's placeholder numbers.",
    "export const V2_NUMBERS: Record<string, { pwr: number; hp: number }> = {",
    ...rows.map((r) => `  ${JSON.stringify(r.name)}: { pwr: ${r.pwr}, hp: ${r.hp} },`),
    "};",
    "",
  ].join("\n"),
);

// What Awoken adds: each unit's best team against the final field, as it is
// and with the unit asleep at 3 copies (tune.ts's measureTeam).
const pool = contentOf(rows);
const asleep = pool.units.map((u) => ({ ...u, id: u.id + "~asleep", forms: { sleeping: u.forms.sleeping, awoken: u.forms.sleeping } }));
const withAsleep = { ...pool, units: [...pool.units, ...asleep] };
const margins = new Map(final.found.map((f) => [f.unit, measureTeam(withAsleep, f.team, f.unit, final.teams, TUNE_DEFAULT)]));

const units = rows.map((r) => {
  const id = slugOf(r.name);
  const s = scores.get(id) ?? 0;
  const m = margins.get(id)!;
  return { name: r.name, emoji: r.emoji, tier: r.tier, pwr: r.pwr, hp: r.hp, fieldScore: +s.toFixed(3), awokenMargin: +(m.score - m.sleeping).toFixed(3), share: +share(id).toFixed(3), inBand: Math.abs(s - mid) <= BAND, team: m.team };
});
const report = { seconds: Math.round((Date.now() - t0) / 1000), band: BAND, median: +mid.toFixed(3), healthy: final.healthy, checks: final.checks, units, rounds };
mkdirSync("docs/mvp", { recursive: true });
writeFileSync("docs/mvp/tune-v2.json", JSON.stringify(report, null, 2) + "\n");
writeFileSync(
  "docs/mvp/tune-v2.md",
  [
    "# Rules v2 pool tune",
    "",
    `Generated by \`npm run mvp:tune-v2\` (scripts/tune-v2.ts): ${rounds.length} rounds of the quick meta check, then the ${process.argv.includes("--quick") ? "quick" : "full"} one. ${report.seconds}s.`,
    "",
    `**Meta ${final.healthy ? "healthy" : "not healthy"}**: ${Object.values(final.checks).map((c) => `${c.rule} → ${c.value} ${c.pass ? "✅" : "❌"}`).join(" · ")}.`,
    "",
    `Each unit's best team's field score; the band is the median ${pct(mid)} ± ${pct(BAND)}. Awoken adds: that team's field score awoken minus with the unit asleep. Share: the unit's share of play in the equilibrium.`,
    "",
    "| Unit | Tier | PWR/HP | Field score | Awoken adds | Share | |",
    "|---|---|---|---|---|---|---|",
    ...units.map((u) => `| ${u.emoji} ${u.name} | ${u.tier} | ${u.pwr}/${u.hp} | ${pct(u.fieldScore)} | ${u.awokenMargin >= 0 ? "+" : ""}${pct(u.awokenMargin)} | ${pct(u.share)} | ${u.inBand ? "in band" : u.fieldScore > mid ? "strong" : "weak"} |`),
    "",
    "## Rounds",
    "",
    ...rounds.flatMap((r) => [`- Round ${r.round} (median ${pct(r.median)}): ${r.moves.length ? r.moves.join("; ") : "no moves"}`]),
    "",
  ].join("\n"),
);
console.error(`final: ${final.healthy ? "healthy" : "not healthy"}, ${units.filter((u) => u.inBand).length}/${units.length} in band; wrote src/mvp/units-v2-numbers.ts, docs/mvp/tune-v2.md`);
