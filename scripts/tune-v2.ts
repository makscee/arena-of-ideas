// Tune the rules v2 pool's numbers (mission #1043, unit batch 1, #1046).
//
//   npm run mvp:tune-v2 [-- --rounds N]
//
// The v2 pool is new, so there are no live units to measure a candidate
// against (what scripts/tune-unit.ts does for the grid): the batch is tuned
// against itself. Each round runs the quick meta check on the pool
// (metaHealth: a breaker team per unit, then the equilibrium; a new seed each
// round), reads every unit's best team's field score, and moves each unit
// outside the band around the pool's median one step back toward it: HP (by
// 2 when far out), PWR only when HP is at its bound; units with no strike of
// their own (PWR 0) keep PWR 0. Steadier than one step per noisy measure: a
// unit whose last move went the other way holds still a round, and the
// numbers of the round with the most units in band are kept.
//
// The awoken knob: the meta plays every unit at 3 copies, awoken, so each
// round also measures what Awoken adds to the unit's best team (that team
// with the unit asleep, tune.ts's measureTeam); a unit whose Awoken adds
// under MARGIN gains +2 HP when it awakens (up to +8), then +1 PWR (up to
// +2, units that strike). After the full meta check the same step runs again
// on the final field until every margin clears.
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
const ROUNDS = arg("--rounds", 8);
/** Inside the band: within this much of the pool's median field score. */
const BAND = 0.08;
/** Far out: HP moves by 2. */
const FAR = 0.16;
const PWR = { min: 1, max: 6 };
const HP = { min: 3, max: 16 };

const pct = (x: number) => `${Math.round(x * 100)}%`;
const median = (xs: number[]) => {
  const s = [...xs].sort((a, b) => a - b);
  return s.length % 2 ? s[(s.length - 1) / 2]! : (s[s.length / 2 - 1]! + s[s.length / 2]!) / 2;
};
const scoresOf = (r: MetaResult) => new Map(r.found.map((f) => [f.unit, r.fieldOf(f.team)]));

let rows: V2Row[] = ROWS_V2.map((r) => ({ ...r, awoken: r.awoken ?? { pwr: 0, hp: 0 } }));
const t0 = Date.now();
const rounds: { round: number; median: number; inBand: number; scores: Record<string, number>; moves: string[]; healthy: boolean }[] = [];
/** The awoken knob's step and cap: HP first, then PWR for units that strike. */
const AWOKEN = { hp: 8, pwr: 2 };
/** Awoken must add at least this much to its team's field score. */
const MARGIN = 0.02;
const bonusUp = (row: V2Row): V2Row => {
  const a = row.awoken!;
  if (a.hp < AWOKEN.hp) return { ...row, awoken: { ...a, hp: a.hp + 2 } };
  if (row.pwr > 0 && a.pwr < AWOKEN.pwr) return { ...row, awoken: { ...a, pwr: a.pwr + 1 } };
  return row;
};
/** The pool with every unit's asleep twin (forms awoken = sleeping, no awoken bonus). */
const withAsleepOf = (rs: V2Row[]) => {
  const pool = contentOf(rs);
  const asleep = pool.units.map(({ awokenBonus: _, ...u }) => ({ ...u, id: u.id + "~asleep", forms: { sleeping: u.forms.sleeping, awoken: u.forms.sleeping } }));
  return { ...pool, units: [...pool.units, ...asleep] };
};
const marginsOf = (rs: V2Row[], r: MetaResult) => {
  const c = withAsleepOf(rs);
  return new Map(r.found.map((f) => [f.unit, measureTeam(c, f.team, f.unit, r.teams, TUNE_DEFAULT)]));
};

// Steadier than one step per quick meta: a unit whose last move went the
// other way holds still one round (no oscillating), and the rows of the
// round with the most units in band (then the smallest worst miss) are kept.
let best: { rows: V2Row[]; inBand: number; worst: number; round: number } | null = null;
const lastDir = new Map<string, number>();
for (let round = 1; round <= ROUNDS; round++) {
  const r = metaHealth(contentOf(rows), DEFAULT_SEED + round - 1, META_QUICK);
  const scores = scoresOf(r);
  const mid = median([...scores.values()]);
  const offs = rows.map((row) => (scores.get(slugOf(row.name)) ?? mid) - mid);
  const inBand = offs.filter((o) => Math.abs(o) <= BAND).length;
  const worst = Math.max(...offs.map(Math.abs));
  if (!best || inBand > best.inBand || (inBand === best.inBand && worst < best.worst)) best = { rows: rows.map((x) => ({ ...x })), inBand, worst, round };
  // the awoken knob: units whose Awoken adds too little to their best team
  const margins = marginsOf(rows, r);
  const moves: string[] = [];
  rows = rows.map((row, i) => {
    const id = slugOf(row.name);
    const s = scores.get(id) ?? mid;
    const off = offs[i]!;
    const m = margins.get(id);
    let next = row;
    if (m && m.score - m.sleeping < MARGIN) {
      next = bonusUp(row);
      if (next !== row) moves.push(`${row.name} awoken adds ${pct(m.score - m.sleeping)}: +${row.awoken!.pwr}/+${row.awoken!.hp}→+${next.awoken!.pwr}/+${next.awoken!.hp}`);
    }
    if (Math.abs(off) <= BAND) return next;
    const d = off > 0 ? -1 : 1;
    if (lastDir.get(id) === -d) {
      lastDir.delete(id);
      return next;
    }
    lastDir.set(id, d);
    const far = Math.abs(off) > FAR;
    const hp = Math.max(HP.min, Math.min(HP.max, row.hp + d * (far ? 2 : 1)));
    // PWR is the coarse knob (one point moves a unit's every strike): it
    // turns only when HP is at its bound.
    const pwrMoves = row.pwr > 0 && hp === row.hp;
    const pwr = pwrMoves ? Math.max(PWR.min, Math.min(PWR.max, row.pwr + d)) : row.pwr;
    if (hp === row.hp && pwr === row.pwr) return next;
    moves.push(`${row.name} ${pct(s)}: ${row.pwr}/${row.hp}→${pwr}/${hp}`);
    return { ...next, pwr, hp };
  });
  rounds.push({ round, median: +mid.toFixed(3), inBand, scores: Object.fromEntries([...scores].map(([k, v]) => [k, +v.toFixed(3)])), moves, healthy: r.healthy });
  console.error(`round ${round}: median ${pct(mid)}, ${inBand}/20 in band, worst ${pct(worst)}, ${moves.length} moves, quick meta ${r.healthy ? "healthy" : "not healthy"}  ${((Date.now() - t0) / 1000).toFixed(0)}s`);
  for (const m of moves) console.error(`  ${m}`);
  if (!moves.length) {
    best = { rows: rows.map((x) => ({ ...x })), inBand, worst, round };
    break;
  }
}
// The best round's band numbers, with the awoken knob as the last round left it
// (it only grows, and the meta plays every unit awoken).
const lastAwoken = new Map(rows.map((r) => [r.name, r.awoken]));
rows = best!.rows.map((r) => ({ ...r, awoken: lastAwoken.get(r.name)! }));
console.error(`kept round ${best!.round}'s numbers (${best!.inBand}/20 in band)`);

const final = metaHealth(contentOf(rows), DEFAULT_SEED, process.argv.includes("--quick") ? META_QUICK : META_FULL, (l) => console.error(l));
const scores = scoresOf(final);
const mid = median([...scores.values()]);
const share = (id: string) => final.unitShare.get(id) ?? 0;

// What Awoken adds: each unit's best team against the final field, as it is
// and with the unit asleep at 3 copies (tune.ts's measureTeam). A unit whose
// margin is still too small gets its awoken knob raised and is measured again.
let margins = marginsOf(rows, final);
for (let pass = 0; pass < 4; pass++) {
  const low = rows.filter((r) => { const m = margins.get(slugOf(r.name))!; return m.score - m.sleeping < MARGIN; });
  if (!low.length) break;
  rows = rows.map((r) => (low.includes(r) ? bonusUp(r) : r));
  console.error(`awoken fix ${pass + 1}: ${low.map((r) => r.name).join(", ")}`);
  margins = marginsOf(rows, final);
}
writeFileSync(
  "src/mvp/units-v2-numbers.ts",
  [
    "// PWR and HP of the rules v2 pool, written by `npm run mvp:tune-v2`",
    "// (scripts/tune-v2.ts): the numbers come from the tune, never from the",
    "// designer. A unit missing here keeps its row's placeholder numbers.",
    "// awoken: PWR and HP the unit gains when it awakens (the awoken knob).",
    "export const V2_NUMBERS: Record<string, { pwr: number; hp: number; awoken?: { pwr: number; hp: number } }> = {",
    ...rows.map((r) => `  ${JSON.stringify(r.name)}: { pwr: ${r.pwr}, hp: ${r.hp}, awoken: { pwr: ${r.awoken!.pwr}, hp: ${r.awoken!.hp} } },`),
    "};",
    "",
  ].join("\n"),
);

const units = rows.map((r) => {
  const id = slugOf(r.name);
  const s = scores.get(id) ?? 0;
  const m = margins.get(id)!;
  return { name: r.name, emoji: r.emoji, tier: r.tier, pwr: r.pwr, hp: r.hp, awoken: r.awoken!, fieldScore: +s.toFixed(3), awokenMargin: +(m.score - m.sleeping).toFixed(3), share: +share(id).toFixed(3), inBand: Math.abs(s - mid) <= BAND, team: m.team };
});
const report = { seconds: Math.round((Date.now() - t0) / 1000), band: BAND, median: +mid.toFixed(3), healthy: final.healthy, checks: final.checks, units, rounds };
mkdirSync("docs/mvp", { recursive: true });
writeFileSync("docs/mvp/tune-v2.json", JSON.stringify(report, null, 2) + "\n");
writeFileSync(
  "docs/mvp/tune-v2.md",
  [
    "# Rules v2 pool tune",
    "",
    `Generated by \`npm run mvp:tune-v2\` (scripts/tune-v2.ts): ${rounds.length} rounds of the quick meta check (kept round ${best!.round}), then the ${process.argv.includes("--quick") ? "quick" : "full"} one. ${report.seconds}s.`,
    "",
    `**Meta ${final.healthy ? "healthy" : "not healthy"}**: ${Object.values(final.checks).map((c) => `${c.rule} → ${c.value} ${c.pass ? "✅" : "❌"}`).join(" · ")}.`,
    "",
    `Each unit's best team's field score; the band is the median ${pct(mid)} ± ${pct(BAND)}. Awoken adds: that team's field score awoken minus with the unit asleep. Share: the unit's share of play in the equilibrium.`,
    "",
    "| Unit | Tier | PWR/HP | Awoken +PWR/+HP | Field score | Awoken adds | Share | |",
    "|---|---|---|---|---|---|---|---|",
    ...units.map((u) => `| ${u.emoji} ${u.name} | ${u.tier} | ${u.pwr}/${u.hp} | +${u.awoken.pwr}/+${u.awoken.hp} | ${pct(u.fieldScore)} | ${u.awokenMargin >= 0 ? "+" : ""}${pct(u.awokenMargin)} | ${pct(u.share)} | ${u.inBand ? "in band" : u.fieldScore > mid ? "strong" : "weak"} |`),
    "",
    "## Rounds",
    "",
    ...rounds.flatMap((r) => [`- Round ${r.round} (median ${pct(r.median)}): ${r.moves.length ? r.moves.join("; ") : "no moves"}`]),
    "",
  ].join("\n"),
);
console.error(`final: ${final.healthy ? "healthy" : "not healthy"}, ${units.filter((u) => u.inBand).length}/${units.length} in band; wrote src/mvp/units-v2-numbers.ts, docs/mvp/tune-v2.md`);
