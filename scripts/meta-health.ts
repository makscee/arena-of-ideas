// Meta-health report for the MVP pool (mission #574, slice 7).
//
//   npm run mvp:meta [-- --quick]
//
// 1. Breaker search: for every unit, a hill climb over whole teams of 5 that
//    contain it (members and order), scored against a gauntlet of teams. Units
//    play awoken at 3 copies, the shape a round-12 line has.
// 2. Equilibrium: every breaker team plays every other on both sides; the
//    symmetric zero-sum game's mixed equilibrium (multiplicative weights) is
//    the meta. Healthy: ≥5 teams in it, the largest share ≤40%, and every top
//    team beaten reliably (≥60%) by ≥3 others.
//
// Writes docs/mvp/meta-health.md and docs/mvp/meta-health.json.

import { mkdirSync, writeFileSync } from "node:fs";
import { battle, winnerOf } from "../src/battle.js";
import { MVP_RULES, type LineUnit, type MvpContent, type UnitContent } from "../src/mvp/contract.js";
import { lineUnitOf } from "../src/mvp/forms.js";
import { toBattleDef } from "../src/mvp/run.js";
import { mvpContent } from "../server/src/mvp/content.js";

const QUICK = process.argv.includes("--quick");
const COPIES = 3;
const GAUNTLET = QUICK ? 12 : 16;
const SEARCH_SEEDS = QUICK ? 2 : 3;
const SEARCH_STEPS = QUICK ? 15 : 30;
const MATRIX_SEEDS = QUICK ? 4 : 8;
const RELIABLE = 0.6;
const SHARE_FLOOR = 0.02;
const COUNTER_ROUNDS = QUICK ? 2 : 4;
const COUNTER_RESTARTS = 3;

const content: MvpContent = mvpContent();
const units = content.units;
const byId = new Map(units.map((u) => [u.id, u]));

let state = 0x5eed;
const rand = () => {
  state = (state + 0x6d2b79f5) | 0;
  let t = Math.imul(state ^ (state >>> 15), 1 | state);
  t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
};
const pick = <T>(xs: T[]): T => xs[Math.floor(rand() * xs.length)]!;

type Team = string[]; // unit ids, front first
const key = (t: Team) => t.join(",");

const defsCache = new Map<string, ReturnType<typeof toBattleDef>[]>();
function defs(t: Team) {
  const k = key(t);
  let d = defsCache.get(k);
  if (!d) {
    d = t.map((id, i) => toBattleDef(lineUnitOf(byId.get(id) as UnitContent, `u${i}`, COPIES) as LineUnit));
    defsCache.set(k, d);
  }
  return d;
}

let battles = 0;
let capped = 0;
/** Score of a vs b over `seeds` seeds on each side: wins 1, draws ½. */
function score(a: Team, b: Team, seeds: number, seed0 = 1): number {
  let s = 0;
  for (let i = 0; i < seeds; i++) {
    for (const aSide of ["A", "B"] as const) {
      const [teamA, teamB] = aSide === "A" ? [defs(a), defs(b)] : [defs(b), defs(a)];
      const log = battle({ teamA, teamB, seed: seed0 + i, statuses: content.statuses, abilities: content.abilities, chainStepCap: MVP_RULES.chainStepCap });
      battles++;
      if (log.some((e) => e.type === "ChainCapped")) capped++;
      const w = winnerOf(log);
      s += w === aSide ? 1 : w === "draw" ? 0.5 : 0;
    }
  }
  return s / (2 * seeds);
}

function randomTeam(withId?: string): Team {
  const ids = new Set<string>(withId ? [withId] : []);
  while (ids.size < MVP_RULES.lineSize) ids.add(pick(units).id);
  const t = [...ids];
  for (let i = t.length - 1; i > 0; i--) {
    const j = Math.floor(rand() * (i + 1));
    [t[i], t[j]] = [t[j]!, t[i]!];
  }
  return t;
}

const vsGauntlet = (t: Team, g: Team[]) => g.reduce((s, o) => s + score(t, o, SEARCH_SEEDS), 0) / g.length;

/** Best team and order containing `id`, by hill climb against the gauntlet. */
function breaker(id: string, gauntlet: Team[]): { team: Team; score: number } {
  let best = randomTeam(id);
  let bestScore = vsGauntlet(best, gauntlet);
  for (let step = 0; step < SEARCH_STEPS; step++) {
    const t = [...best];
    if (rand() < 0.6) {
      const slots = t.map((x, i) => (x === id ? -1 : i)).filter((i) => i >= 0);
      const slot = pick(slots);
      let n = pick(units).id;
      while (t.includes(n)) n = pick(units).id;
      t[slot] = n;
    } else {
      const i = Math.floor(rand() * t.length);
      const j = Math.floor(rand() * t.length);
      [t[i], t[j]] = [t[j]!, t[i]!];
    }
    const s = vsGauntlet(t, gauntlet);
    if (s >= bestScore) [best, bestScore] = [t, s];
  }
  return { team: best, score: bestScore };
}

/** Mixed equilibrium of the symmetric zero-sum game with payoff m[i][j] − ½. */
function equilibrium(m: number[][]): number[] {
  const n = m.length;
  let w = new Array(n).fill(1 / n);
  const avg = new Array(n).fill(0);
  const iters = 20000;
  const eta = 0.05;
  for (let it = 0; it < iters; it++) {
    const pay = m.map((row) => row.reduce((s, x, j) => s + (x - 0.5) * w[j], 0));
    w = w.map((x, i) => x * Math.exp(eta * pay[i]!));
    const z = w.reduce((a, b) => a + b, 0);
    w = w.map((x) => x / z);
    for (let i = 0; i < n; i++) avg[i] += w[i] / iters;
  }
  return avg;
}

const t0 = Date.now();
const name = (id: string) => `${byId.get(id)!.emoji} ${byId.get(id)!.name}`;
const teamText = (t: Team) => t.map(name).join(" · ");

// 1. Breaker search, gauntlet grows from random teams into found breakers.
const gauntlet: Team[] = Array.from({ length: GAUNTLET }, () => randomTeam());
const found: { unit: string; team: Team; score: number }[] = [];
for (const [i, u] of units.entries()) {
  const b = breaker(u.id, gauntlet);
  found.push({ unit: u.id, ...b });
  gauntlet[i % GAUNTLET] = b.team; // later units must beat earlier breakers
  if ((i + 1) % 10 === 0) console.error(`breaker ${i + 1}/${units.length}  ${((Date.now() - t0) / 1000).toFixed(0)}s`);
}

// 2. Matrix and equilibrium over distinct breaker teams, then a counter
// search (double oracle): for each team in the equilibrium, hill-climb free
// teams against it; reliable counters join the field and the meta is re-solved.
const teams: Team[] = [];
const m: number[][] = [];
function addTeam(t: Team): boolean {
  if (teams.some((x) => key(x) === key(t))) return false;
  const row = teams.map((o) => score(t, o, MATRIX_SEEDS, 1000));
  teams.forEach((_, j) => m[j]!.push(1 - row[j]!));
  row.push(0.5);
  m.push(row);
  teams.push(t);
  return true;
}
for (const f of found) addTeam(f.team);

function counter(target: Team): { team: Team; score: number } {
  let best = randomTeam();
  let bestScore = score(best, target, SEARCH_SEEDS * 2, 500);
  for (let step = 0; step < SEARCH_STEPS; step++) {
    const t = [...best];
    if (rand() < 0.6) {
      let n = pick(units).id;
      while (t.includes(n)) n = pick(units).id;
      t[Math.floor(rand() * t.length)] = n;
    } else {
      const i = Math.floor(rand() * t.length);
      const j = Math.floor(rand() * t.length);
      [t[i], t[j]] = [t[j]!, t[i]!];
    }
    const s = score(t, target, SEARCH_SEEDS * 2, 500);
    if (s >= bestScore) [best, bestScore] = [t, s];
  }
  return { team: best, score: bestScore };
}

let eq = equilibrium(m);
let counters = 0;
for (let round = 0; round < COUNTER_ROUNDS; round++) {
  const targets = teams.map((_, i) => i).filter((i) => eq[i]! >= SHARE_FLOOR).sort((a, b) => eq[b]! - eq[a]!).slice(0, 8);
  for (const i of targets) {
    for (let r = 0; r < COUNTER_RESTARTS; r++) {
      const c = counter(teams[i]!);
      if (c.score >= RELIABLE && addTeam(c.team)) counters++;
    }
  }
  eq = equilibrium(m);
  console.error(`counter round ${round + 1}: ${teams.length} teams  ${((Date.now() - t0) / 1000).toFixed(0)}s`);
}
const order = teams.map((_, i) => i).sort((a, b) => eq[b]! - eq[a]!);
const top = order.filter((i) => eq[i]! >= SHARE_FLOOR);
const largest = eq[order[0]!]!;
const beatenBy = (i: number) => teams.map((_, j) => j).filter((j) => j !== i && m[j]![i]! >= RELIABLE);

const unitShare = new Map<string, number>();
for (const i of top) for (const id of teams[i]!) unitShare.set(id, (unitShare.get(id) ?? 0) + eq[i]!);
const fieldScore = teams.map((_, i) => m[i]!.reduce((s, x) => s + x, 0) / teams.length);

const fieldOf = (t: Team) => fieldScore[teams.findIndex((x) => key(x) === key(t))]!;

const checks = {
  teamsInEquilibrium: { value: top.length, pass: top.length >= 5, rule: "≥5 teams with ≥2% share" },
  largestShare: { value: +largest.toFixed(3), pass: largest <= 0.4, rule: "largest share ≤40%" },
  topTeamsBeaten: {
    value: Math.min(...top.map((i) => beatenBy(i).length)),
    pass: top.every((i) => beatenBy(i).length >= 3),
    rule: `every top team beaten ≥${RELIABLE * 100}% by ≥3 others`,
  },
};
const healthy = Object.values(checks).every((c) => c.pass);
const capRate = capped / battles;

const report = {
  contentVersion: content.version,
  units: units.length,
  quick: QUICK,
  setup: { copies: COPIES, gauntlet: GAUNTLET, searchSeeds: SEARCH_SEEDS, searchSteps: SEARCH_STEPS, matrixSeeds: MATRIX_SEEDS, counterRounds: COUNTER_ROUNDS, counterRestarts: COUNTER_RESTARTS, candidateTeams: teams.length, countersFound: counters },
  battles,
  chainCappedRate: +capRate.toFixed(4),
  seconds: Math.round((Date.now() - t0) / 1000),
  healthy,
  checks,
  equilibrium: top.map((i) => ({ share: +eq[i]!.toFixed(3), team: teams[i], fieldScore: +fieldScore[i]!.toFixed(3), beatenReliablyBy: beatenBy(i).length })),
  breakers: found.map((f) => ({ unit: f.unit, team: f.team, gauntletScore: +f.score.toFixed(3), fieldScore: +fieldOf(f.team).toFixed(3) })),
};

const pct = (x: number) => `${(x * 100).toFixed(0)}%`;
const md: string[] = [
  "# MVP meta-health report",
  "",
  `Generated by \`npm run mvp:meta\` (scripts/meta-health.ts). Content \`${content.version}\`, ${units.length} units, awoken at ${COPIES} copies.`,
  "",
  `**${healthy ? "Healthy" : "Not healthy"}.**`,
  "",
  "| Check | Rule | Value | |",
  "|---|---|---|---|",
  ...Object.values(checks).map((c) => `| ${c.rule === checks.teamsInEquilibrium.rule ? "Teams in the equilibrium" : c.rule === checks.largestShare.rule ? "Largest share" : "Top teams have answers"} | ${c.rule} | ${typeof c.value === "number" && c.value < 1 && c.value > 0 ? pct(c.value) : c.value} | ${c.pass ? "✅" : "❌"} |`),
  "",
  `${battles.toLocaleString("en")} battles in ${report.seconds}s; ${pct(capRate)} hit the chain cap (${(capRate * 100).toFixed(2)}%).`,
  "",
  "## The equilibrium",
  "",
  `Mixed equilibrium of ${teams.length} candidate teams playing each other (both sides, ${MATRIX_SEEDS} seeds each): the ${found.length} breaker teams plus ${counters} counters found by ${COUNTER_ROUNDS} rounds of counter search against the equilibrium's teams. Field score = mean score against every candidate team.`,
  "",
  "| Share | Team (front first) | Field score | Beaten ≥60% by |",
  "|---|---|---|---|",
  ...top.map((i) => `| ${pct(eq[i]!)} | ${teamText(teams[i]!)} | ${pct(fieldScore[i]!)} | ${beatenBy(i).length} |`),
  "",
  "## Units in the meta",
  "",
  [...unitShare.entries()].sort((a, b) => b[1] - a[1]).map(([id, s]) => `${name(id)} ${pct(s)}`).join(" · "),
  "",
  "## Breaker search: the best team found for each unit",
  "",
  `Hill climb over members and order (${SEARCH_STEPS} steps), scored against a gauntlet of ${GAUNTLET} teams that fills with earlier breakers. Field score = the team's mean score against every candidate team: the unit's strength at its best, used to tune numbers.`,
  "",
  "| Unit | Tier | Best team | Field score | Gauntlet score |",
  "|---|---|---|---|---|",
  ...[...found].sort((a, b) => fieldOf(b.team) - fieldOf(a.team)).map((f) => `| ${name(f.unit)} | ${byId.get(f.unit)!.tier} | ${teamText(f.team)} | ${pct(fieldOf(f.team))} | ${pct(f.score)} |`),
  "",
];

mkdirSync("docs/mvp", { recursive: true });
const out = QUICK ? "docs/mvp/meta-health.quick" : "docs/mvp/meta-health";
writeFileSync(`${out}.md`, md.join("\n"));
writeFileSync(`${out}.json`, JSON.stringify(report, null, 2) + "\n");
console.log(`${healthy ? "HEALTHY" : "NOT HEALTHY"}: ${top.length} teams, largest ${pct(largest)}, min answers ${checks.topTeamsBeaten.value}, cap ${(capRate * 100).toFixed(2)}%, ${battles} battles, ${report.seconds}s → ${out}.md`);
