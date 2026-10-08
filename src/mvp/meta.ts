// The meta simulation (mission #574 slice 7, split out for M2-7's tuner):
// the breaker search, the equilibrium and the health checks that
// `npm run mvp:meta` (scripts/meta-health.ts) reports, as a library the
// tuner (src/mvp/tune.ts) and later the server can call on any content.
//
// Every fight goes through fightLines (src/mvp/fight.ts), as the game's do.
// Deterministic: the same content, seed and settings give the same result.

import { MVP_RULES, type LineUnit, type MvpContent, type PlayerRef, type UnitContent } from "./contract.js";
import { fightLines } from "./fight.js";
import { lineUnitOf } from "./forms.js";

export interface MetaSettings {
  copies: number;
  gauntlet: number;
  searchSeeds: number;
  searchSteps: number;
  matrixSeeds: number;
  counterRounds: number;
  counterRestarts: number;
}
export const META_FULL: MetaSettings = { copies: 3, gauntlet: 12, searchSeeds: 3, searchSteps: 25, matrixSeeds: 8, counterRounds: 6, counterRestarts: 3 };
export const META_QUICK: MetaSettings = { copies: 3, gauntlet: 12, searchSeeds: 2, searchSteps: 15, matrixSeeds: 4, counterRounds: 2, counterRestarts: 3 };
export const DEFAULT_SEED = 0x5eed;
export const RELIABLE = 0.6;
export const SHARE_FLOOR = 0.02;

export type Team = string[]; // unit ids, front first
export const teamKey = (t: Team) => t.join(",");

const SIM: PlayerRef = { id: "meta-health", name: "meta-health", bot: true };
const AT = new Date(0).toISOString();

/** A seeded simulator over `content`: teams are drawn from `pool` (default:
 * every unit), each unit at `copies` copies. */
export function metaSim(content: MvpContent, seed: number, opts: { copies: number; searchSeeds: number; searchSteps: number; pool?: UnitContent[] }) {
  const units = opts.pool ?? content.units;
  const byId = new Map(content.units.map((u) => [u.id, u]));
  let state = seed;
  const rand = () => {
    state = (state + 0x6d2b79f5) | 0;
    let t = Math.imul(state ^ (state >>> 15), 1 | state);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
  const pick = <T>(xs: T[]): T => xs[Math.floor(rand() * xs.length)]!;

  const linesCache = new Map<string, LineUnit[]>();
  function lineOf(t: Team): LineUnit[] {
    const k = teamKey(t);
    let line = linesCache.get(k);
    if (!line) {
      line = t.map((id, i) => {
        const u = byId.get(id);
        if (!u) throw new Error(`unknown unit "${id}"`);
        return lineUnitOf(u, `u${i}`, opts.copies);
      });
      linesCache.set(k, line);
    }
    return line;
  }

  const stats = { battles: 0, capped: 0 };
  /** Score of a vs b over `seeds` seeds on each side: wins 1, draws ½. */
  function score(a: Team, b: Team, seeds: number, seed0 = 1): number {
    let s = 0;
    for (let i = 0; i < seeds; i++) {
      for (const aSide of ["A", "B"] as const) {
        const [lineA, lineB] = aSide === "A" ? [lineOf(a), lineOf(b)] : [lineOf(b), lineOf(a)];
        const rec = fightLines(
          { player: SIM, line: lineA },
          { player: SIM, line: lineB },
          { battleId: "meta", seed: seed0 + i, kind: "round", round: MVP_RULES.rounds, runId: null, at: AT, content, rules: MVP_RULES },
        );
        stats.battles++;
        if (rec.log.some((e) => e.type === "ChainCapped")) stats.capped++;
        const w = rec.winner;
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

  const vsGauntlet = (t: Team, g: Team[]) => g.reduce((s, o) => s + score(t, o, opts.searchSeeds), 0) / g.length;

  /** Best team and order containing `id`, by hill climb against the gauntlet. */
  function breaker(id: string, gauntlet: Team[]): { team: Team; score: number } {
    let best = randomTeam(id);
    let bestScore = vsGauntlet(best, gauntlet);
    for (let step = 0; step < opts.searchSteps; step++) {
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

  /** Best free team against one target, by hill climb (the counter search). */
  function counter(target: Team): { team: Team; score: number } {
    let best = randomTeam();
    let bestScore = score(best, target, opts.searchSeeds * 2, 500);
    for (let step = 0; step < opts.searchSteps; step++) {
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
      const s = score(t, target, opts.searchSeeds * 2, 500);
      if (s >= bestScore) [best, bestScore] = [t, s];
    }
    return { team: best, score: bestScore };
  }

  return { rand, pick, units, byId, lineOf, score, randomTeam, vsGauntlet, breaker, counter, stats };
}
export type MetaSim = ReturnType<typeof metaSim>;

/** Mixed equilibrium of the symmetric zero-sum game with payoff m[i][j] − ½. */
export function equilibrium(m: number[][]): number[] {
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

export interface MetaResult {
  sim: MetaSim;
  found: { unit: string; team: Team; score: number }[];
  teams: Team[];
  /** m[i][j]: team i's score against team j. */
  m: number[][];
  eq: number[];
  counters: number;
  /** Teams with at least SHARE_FLOOR of the equilibrium, largest first. */
  top: number[];
  largest: number;
  beatenBy: (i: number) => number[];
  /** Mean score of each team against every candidate team. */
  fieldScore: number[];
  fieldOf: (t: Team) => number;
  unitShare: Map<string, number>;
  checks: {
    teamsInEquilibrium: { value: number; pass: boolean; rule: string };
    largestShare: { value: number; pass: boolean; rule: string };
    topTeamsBeaten: { value: number; pass: boolean; rule: string };
  };
  healthy: boolean;
}

/** The whole meta check on `content`: a breaker per unit, then the
 * equilibrium with counter search. Healthy: ≥5 teams in it, the largest
 * share ≤40%, and every top team beaten reliably (≥60%) by ≥3 others. */
export function metaHealth(content: MvpContent, seed: number, s: MetaSettings, log: (line: string) => void = () => {}): MetaResult {
  const t0 = Date.now();
  const sim = metaSim(content, seed, s);
  const { score, randomTeam, breaker, counter } = sim;
  const units = content.units;

  // 1. Breaker search, gauntlet grows from random teams into found breakers.
  const gauntlet: Team[] = Array.from({ length: s.gauntlet }, () => randomTeam());
  const found: { unit: string; team: Team; score: number }[] = [];
  for (const [i, u] of units.entries()) {
    const b = breaker(u.id, gauntlet);
    found.push({ unit: u.id, ...b });
    gauntlet[i % s.gauntlet] = b.team; // later units must beat earlier breakers
    if ((i + 1) % 10 === 0) log(`breaker ${i + 1}/${units.length}  ${((Date.now() - t0) / 1000).toFixed(0)}s`);
  }

  // 2. Matrix and equilibrium over distinct breaker teams, then a counter
  // search (double oracle): for each team in the equilibrium, hill-climb free
  // teams against it; reliable counters join the field and the meta is re-solved.
  const teams: Team[] = [];
  const m: number[][] = [];
  function addTeam(t: Team): boolean {
    if (teams.some((x) => teamKey(x) === teamKey(t))) return false;
    const row = teams.map((o) => score(t, o, s.matrixSeeds, 1000));
    teams.forEach((_, j) => m[j]!.push(1 - row[j]!));
    row.push(0.5);
    m.push(row);
    teams.push(t);
    return true;
  }
  for (const f of found) addTeam(f.team);

  let eq = equilibrium(m);
  let counters = 0;
  for (let round = 0; round < s.counterRounds; round++) {
    const targets = teams.map((_, i) => i).filter((i) => eq[i]! >= SHARE_FLOOR).sort((a, b) => eq[b]! - eq[a]!).slice(0, 8);
    for (const i of targets) {
      for (let r = 0; r < s.counterRestarts; r++) {
        const c = counter(teams[i]!);
        if (c.score >= RELIABLE && addTeam(c.team)) counters++;
      }
    }
    eq = equilibrium(m);
    log(`counter round ${round + 1}: ${teams.length} teams  ${((Date.now() - t0) / 1000).toFixed(0)}s`);
  }
  const order = teams.map((_, i) => i).sort((a, b) => eq[b]! - eq[a]!);
  const top = order.filter((i) => eq[i]! >= SHARE_FLOOR);
  const largest = eq[order[0]!]!;
  const beatenBy = (i: number) => teams.map((_, j) => j).filter((j) => j !== i && m[j]![i]! >= RELIABLE);

  const unitShare = new Map<string, number>();
  for (const i of top) for (const id of teams[i]!) unitShare.set(id, (unitShare.get(id) ?? 0) + eq[i]!);
  const fieldScore = teams.map((_, i) => m[i]!.reduce((a, x) => a + x, 0) / teams.length);
  const fieldOf = (t: Team) => fieldScore[teams.findIndex((x) => teamKey(x) === teamKey(t))]!;

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
  return { sim, found, teams, m, eq, counters, top, largest, beatenBy, fieldScore, fieldOf, unitShare, checks, healthy };
}
