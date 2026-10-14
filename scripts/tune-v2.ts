// Tune the rules v2 pool's numbers (mission #1043, unit batch 1, #1046).
//
//   npm run mvp:tune-v2 [-- --rounds N --seeds N --checks N]
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
// under MARGIN gains +2 HP when it awakens (up to +4), then +1 PWR (up to
// +1, units that strike). After the full meta check the same step runs again
// on the final field until every margin clears.
//
// Writes src/mvp/units-v2-numbers.ts (the numbers ROWS_V2 plays with) and
// docs/mvp/tune-v2.md and .json (each round's scores and moves, the final
// meta verdict).

import { spawn } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { DEFAULT_SEED, META_FULL, META_QUICK, metaHealth, type MetaResult } from "../src/mvp/meta.js";
import { ROWS_V2, type V2Row } from "../src/mvp/units-v2.js";
import { contentOf } from "../server/src/mvp/content.js";
import { measureTeam, slugOf, TUNE_DEFAULT } from "../src/mvp/tune.js";

/** The awoken knob's step and cap: HP first, then PWR for units that strike.
 * At most +4 HP / +1 PWR (#1047); an awoken form earns the rest by its upgraded clause. */
const AWOKEN = { hp: 4, pwr: 1 };

const arg = (name: string, dflt: number) => {
  const i = process.argv.indexOf(name);
  return i < 0 ? dflt : Number(process.argv[i + 1]);
};
const ROUNDS = arg("--rounds", 8);
/** Quick meta checks per round, each on its own seed, run in parallel and averaged. */
const SEEDS = arg("--seeds", 3);
/** Quick meta checks the final table averages: the gate reads 12 (the orchestrator, 10-14). */
const CHECKS = arg("--checks", 12);
/** Inside the band: within this much of the pool's median field score. */
const BAND = 0.08;
/** Far out: HP moves by 2. */
const FAR = 0.16;
/** Sleeping numbers stay in the old pool's range (main's units: HP 4-12,
 * PWR 1-4; Maks's note on batch 1, #1047: awoken Medic had 33 HP). */
const PWR = { min: 1, max: 5 };
const HP = { min: 4, max: 12 };

const pct = (x: number) => `${Math.round(x * 100)}%`;
const median = (xs: number[]) => {
  const s = [...xs].sort((a, b) => a - b);
  return s.length % 2 ? s[(s.length - 1) / 2]! : (s[s.length / 2 - 1]! + s[s.length / 2]!) / 2;
};
const scoresOf = (r: MetaResult) => new Map(r.found.map((f) => [f.unit, r.fieldOf(f.team)]));

type Measured = { scores: Record<string, number>; margins: Record<string, number>; healthy: boolean };
/** One quick meta check on `rs` and `seed`: each unit's best team's field
 * score and what Awoken adds to it. */
function measureOnce(rs: V2Row[], seed: number): Measured {
  const r = metaHealth(contentOf(rs), seed, META_QUICK);
  const m = marginsOf(rs, r);
  return { scores: Object.fromEntries(scoresOf(r)), margins: Object.fromEntries([...m].map(([k, v]) => [k, v.score - v.sleeping])), healthy: r.healthy };
}
/** measureOnce in a child process, so a round's seeds run in parallel. */
function measureChild(rs: V2Row[], seed: number): Promise<Measured> {
  return new Promise((resolve, reject) => {
    const c = spawn(process.execPath, [...process.execArgv, process.argv[1]!, "--measure", String(seed)], { stdio: ["pipe", "pipe", "inherit"] });
    let out = "";
    c.stdout.on("data", (b) => (out += b));
    c.on("error", reject);
    c.on("close", (code) => (code === 0 ? resolve(JSON.parse(out)) : reject(new Error(`measure ${seed} exited ${code}`))));
    c.stdin.end(JSON.stringify(rs));
  });
}

const clamp = (x: number, b: { min: number; max: number }) => Math.max(b.min, Math.min(b.max, x));
/** The pool's numbers, clamped into today's bounds and the awoken cap (a
 * unit with no strike keeps PWR 0). */
let rows: V2Row[] = ROWS_V2.map((r) => {
  const a = r.awoken ?? { pwr: 0, hp: 0 };
  return { ...r, pwr: r.pwr > 0 ? clamp(r.pwr, PWR) : 0, hp: clamp(r.hp, HP), awoken: { pwr: Math.min(a.pwr, AWOKEN.pwr), hp: Math.min(a.hp, AWOKEN.hp) } };
});
const t0 = Date.now();
const rounds: { round: number; median: number; inBand: number; scores: Record<string, number>; moves: string[]; healthy: boolean }[] = [];
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

if (process.argv.includes("--health")) {
  const r = metaHealth(contentOf(JSON.parse(readFileSync(0, "utf8")) as V2Row[]), arg("--health", DEFAULT_SEED), META_FULL);
  process.stdout.write(JSON.stringify({ healthy: r.healthy, checks: r.checks }));
  process.exit(0);
}
if (process.argv.includes("--measure")) {
  const seed = arg("--measure", DEFAULT_SEED);
  const chunks: Buffer[] = [];
  for await (const b of process.stdin) chunks.push(b as Buffer);
  process.stdout.write(JSON.stringify(measureOnce(JSON.parse(Buffer.concat(chunks).toString()), seed)));
  process.exit(0);
}

// Steadier than one step per quick meta: a unit whose last move went the
// other way holds still one round (no oscillating), and the rows of the round
// closest to the gate (tune-v2.test.ts: 16 of each batch's 20 in band, none
// under 33% or over 67%) are kept, then the most in band, then the smallest
// worst miss.
const BATCH = 20;
const GATE = { inBand: 16, min: 0.33, max: 0.67 };
const gateMisses = (offs: number[], scores: number[]) => {
  let misses = scores.filter((s) => s < GATE.min || s > GATE.max).length;
  for (let b = 0; b < offs.length; b += BATCH) {
    const batch = offs.slice(b, b + BATCH);
    misses += Math.max(0, Math.min(GATE.inBand, batch.length) - batch.filter((o) => Math.abs(o) <= BAND).length);
  }
  return misses;
};
let best: { rows: V2Row[]; misses: number; inBand: number; worst: number; round: number } | null = null;
const lastDir = new Map<string, number>();
for (let round = 1; round <= ROUNDS; round++) {
  const ms = await Promise.all(Array.from({ length: SEEDS }, (_, k) => measureChild(rows, DEFAULT_SEED + (round - 1) * SEEDS + k)));
  const avgOf = (pick: (m: Measured) => Record<string, number>) => {
    const sum = new Map<string, number[]>();
    for (const m of ms) for (const [k, v] of Object.entries(pick(m))) sum.set(k, [...(sum.get(k) ?? []), v]);
    return new Map([...sum].map(([k, vs]) => [k, vs.reduce((a, b) => a + b, 0) / vs.length]));
  };
  const scores = avgOf((m) => m.scores);
  const r = { healthy: ms.filter((m) => m.healthy).length * 2 > ms.length };
  const mid = median([...scores.values()]);
  const offs = rows.map((row) => (scores.get(slugOf(row.name)) ?? mid) - mid);
  const inBand = offs.filter((o) => Math.abs(o) <= BAND).length;
  const worst = Math.max(...offs.map(Math.abs));
  const misses = gateMisses(offs, rows.map((row) => scores.get(slugOf(row.name)) ?? mid));
  const better = !best || misses < best.misses || (misses === best.misses && (inBand > best.inBand || (inBand === best.inBand && worst < best.worst)));
  if (better) best = { rows: rows.map((x) => ({ ...x })), misses, inBand, worst, round };
  // the awoken knob: units whose Awoken adds too little to their best team
  const avgMargin = avgOf((m) => m.margins);
  const margins = new Map([...avgMargin].map(([k, v]) => [k, { score: v, sleeping: 0 }]));
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
    // turns when HP is at its bound, or with HP when the unit is far out.
    const pwrMoves = row.pwr > 0 && (hp === row.hp || far);
    const pwr = pwrMoves ? Math.max(PWR.min, Math.min(PWR.max, row.pwr + d)) : row.pwr;
    if (hp === row.hp && pwr === row.pwr) return next;
    moves.push(`${row.name} ${pct(s)}: ${row.pwr}/${row.hp}→${pwr}/${hp}`);
    return { ...next, pwr, hp };
  });
  rounds.push({ round, median: +mid.toFixed(3), inBand, scores: Object.fromEntries([...scores].map(([k, v]) => [k, +v.toFixed(3)])), moves, healthy: r.healthy });
  console.error(`round ${round} (${SEEDS} seeds): median ${pct(mid)}, ${inBand}/${rows.length} in band, ${misses} short of the gate, worst ${pct(worst)}, ${moves.length} moves, quick meta ${r.healthy ? "healthy" : "not healthy"}  ${((Date.now() - t0) / 1000).toFixed(0)}s`);
  for (const m of moves) console.error(`  ${m}`);
  if (!moves.length) {
    best = { rows: rows.map((x) => ({ ...x })), misses, inBand, worst, round };
    break;
  }
}
// The best round's band numbers, with the awoken knob as the last round left it
// (it only grows, and the meta plays every unit awoken).
const lastAwoken = new Map(rows.map((r) => [r.name, r.awoken]));
if (best) rows = best.rows.map((r) => ({ ...r, awoken: lastAwoken.get(r.name)! }));
console.error(best ? `kept round ${best.round}'s numbers (${best.inBand}/${rows.length} in band, ${best.misses} short of the gate)` : "no rounds: the numbers as they are");

// The table: each unit's field score and what Awoken adds, averaged over
// CHECKS quick checks on seeds no round used. One meta check's per-unit
// score swings with its search's luck, and the full check plays a stronger
// field (its median sits higher), so the band is read on the same steady
// measure the rounds tune by. A unit whose Awoken still adds too little gets
// its awoken knob raised and the table is measured again.
const avgOfMs = (ms: Measured[], pick: (m: Measured) => Record<string, number>) => {
  const sum = new Map<string, number[]>();
  for (const m of ms) for (const [k, v] of Object.entries(pick(m))) sum.set(k, [...(sum.get(k) ?? []), v]);
  return new Map([...sum].map(([k, vs]) => [k, vs.reduce((a, b) => a + b, 0) / vs.length]));
};
const steady = async (rs: V2Row[]) => {
  const ms = await Promise.all(Array.from({ length: CHECKS }, (_, k) => measureChild(rs, DEFAULT_SEED + 1000 + k)));
  return { scores: avgOfMs(ms, (m) => m.scores), margins: avgOfMs(ms, (m) => m.margins) };
};
let st = await steady(rows);
for (let pass = 0; pass < 3; pass++) {
  const low = rows.filter((r) => (st.margins.get(slugOf(r.name)) ?? 0) < MARGIN);
  const next = rows.map((r) => (low.includes(r) ? bonusUp(r) : r));
  if (next.every((r, i) => r === rows[i])) break;
  rows = next;
  console.error(`awoken fix ${pass + 1}: ${low.map((r) => r.name).join(", ")}`);
  st = await steady(rows);
}
const scores = st.scores;
const mid = median([...scores.values()]);

// The meta verdict and each unit's share: the full check on the final numbers.
// One full check's verdict flips with its seed too (#1046: the same numbers
// were healthy on two seeds and not on the third), so the verdict is the
// majority of HEALTH_SEEDS full checks, run in parallel; shares come from the
// first.
const HEALTH_SEEDS = process.argv.includes("--quick") ? 1 : 3;
const healthChild = (seed: number) =>
  new Promise<{ healthy: boolean; checks: MetaResult["checks"] }>((resolve, reject) => {
    const c = spawn(process.execPath, [...process.execArgv, process.argv[1]!, "--health", String(seed)], { stdio: ["pipe", "pipe", "inherit"] });
    let out = "";
    c.stdout.on("data", (b) => (out += b));
    c.on("error", reject);
    c.on("close", (code) => (code === 0 ? resolve(JSON.parse(out)) : reject(new Error(`health ${seed} exited ${code}`))));
    c.stdin.end(JSON.stringify(rows));
  });
const others = Array.from({ length: HEALTH_SEEDS - 1 }, (_, k) => healthChild(DEFAULT_SEED + 1 + k));
const first = metaHealth(contentOf(rows), DEFAULT_SEED, process.argv.includes("--quick") ? META_QUICK : META_FULL, (l) => console.error(l));
const healthChecks = [{ seed: DEFAULT_SEED, healthy: first.healthy, checks: first.checks }, ...(await Promise.all(others)).map((h, k) => ({ seed: DEFAULT_SEED + 1 + k, ...h }))];
const final = { ...first, healthy: healthChecks.filter((h) => h.healthy).length * 2 > healthChecks.length };
const share = (id: string) => final.unitShare.get(id) ?? 0;
const margins = new Map(rows.map((r) => {
  const id = slugOf(r.name);
  return [id, { score: st.margins.get(id) ?? 0, sleeping: 0, team: final.found.find((f) => f.unit === id)?.team ?? [] }];
}));
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
const report = { seconds: Math.round((Date.now() - t0) / 1000), band: BAND, median: +mid.toFixed(3), healthy: final.healthy, checks: final.checks, healthChecks: healthChecks.map((h) => ({ seed: h.seed, healthy: h.healthy })), units, rounds };
mkdirSync("docs/mvp", { recursive: true });
writeFileSync("docs/mvp/tune-v2.json", JSON.stringify(report, null, 2) + "\n");
writeFileSync(
  "docs/mvp/tune-v2.md",
  [
    "# Rules v2 pool tune",
    "",
    `Generated by \`npm run mvp:tune-v2\` (scripts/tune-v2.ts): ${rounds.length} rounds of ${SEEDS} quick meta checks each, averaged (kept round ${best?.round ?? "none"}), then the table from ${CHECKS} more (field score, Awoken adds) and the ${process.argv.includes("--quick") ? "quick" : "full"} meta check (health, share). ${report.seconds}s.`,
    "",
    `**Meta ${final.healthy ? "healthy" : "not healthy"}** (${healthChecks.filter((h) => h.healthy).length} of ${healthChecks.length} full checks healthy, seeds ${healthChecks.map((h) => `${h.seed} ${h.healthy ? "✅" : "❌"}`).join(", ")}). The first check: ${Object.values(final.checks).map((c) => `${c.rule} → ${c.value} ${c.pass ? "✅" : "❌"}`).join(" · ")}.`,
    "",
    `Each unit's best team's field score, averaged over ${CHECKS} quick meta checks; the band is the median ${pct(mid)} ± ${pct(BAND)}. Awoken adds: that team's field score awoken minus with the unit asleep. Share: the unit's share of play in the equilibrium.`,
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
