// Numbers by simulation (mission #735, M2-7): given one candidate unit (a Row
// with both forms, its numbers placeholders), find its PWR, HP and effect
// sizes, and decide whether it may enter the pool.
//
// A unit is measured the way scripts/meta-health.ts measures the pool: its
// best team (a breaker() hill climb against a gauntlet of today's breaker
// teams), then that team's field score, its mean score against every one of
// today's breaker teams. Awoken plays at 3 copies; the same team with the
// unit asleep at 3 copies measures what Awoken adds.
//
// It passes when
//   1. its field score is inside the band of the live units' scores (measured
//      the same way: strong enough to matter in some team, not dominant);
//   2. Awoken beats sleeping by a clear margin (the same team's field score
//      with the unit asleep is lower by at least awokenMargin);
//   3. the whole meta check with the unit added stays HEALTHY.
// Otherwise the reason is one line a player can read.

import { createHash } from "node:crypto";
import type { MvpContent, UnitContent } from "./contract.js";
import { DEFAULT_SEED, META_QUICK, metaHealth, metaSim, type MetaSettings, type Team } from "./meta.js";
import { allDoes, awokenKeeps, effectKinds, mvpPool, type Row } from "./units.js";

export interface TuneSettings {
  /** Field teams used as the breaker search's gauntlet. */
  gauntlet: number;
  searchSeeds: number;
  searchSteps: number;
  /** Seeds per side for a team against each field team. */
  fieldSeeds: number;
  /** Most measurements in one unit's search. */
  maxEvals: number;
  /** Breaker searches per measure; the best one counts. */
  restarts: number;
  /** The least margin Awoken must have over sleeping, whatever the live
   * units' margins (Band.margin). */
  awokenMarginFloor: number;
  seed: number;
}
export const TUNE_DEFAULT: TuneSettings = { gauntlet: 12, searchSeeds: 2, searchSteps: 15, fieldSeeds: 2, maxEvals: 40, restarts: 2, awokenMarginFloor: 0.01, seed: DEFAULT_SEED };
const COPIES = 3;

/** The band a unit's field score must land in, from the live units' spread. */
export interface Band {
  low: number;
  high: number;
  /** The live median, for reference. */
  target: number;
  /** The margin Awoken's field score must have over sleeping's: as much as
   * the live units' at BAND_LOW_Q, at least awokenMarginFloor. */
  margin: number;
  /** Each live unit's field score and Awoken's margin over sleeping. */
  live: { unit: string; score: number; margin: number }[];
}
/** The band's edges as quantiles of the live scores: not weaker than the
 * weakest fifth, not stronger than the strongest tenth. */
export const BAND_LOW_Q = 0.2;
export const BAND_HIGH_Q = 0.9;

export interface Measure {
  /** The best team's field score. */
  score: number;
  /** The same team's field score with the unit asleep at 3 copies. */
  sleeping: number;
  team: Team;
}

export interface TuneStep {
  row: Row;
  measure: Measure;
  kept: boolean;
  change: string;
}

export interface TuneResult {
  row: Row;
  pass: boolean;
  /** One line a player can read. */
  reason: string;
  measure: Measure;
  band: Band;
  steps: TuneStep[];
  meta?: { healthy: boolean; share: number; checks: Record<string, { value: number; pass: boolean; rule: string }> };
  seconds: number;
}

const SLEEP_SUFFIX = "~asleep";

export function slugOf(name: string): string {
  return name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
}

/** Content for a fight: the rows' pool, versioned by its data like mvpContent. */
export function contentOf(rows: Row[], extra: UnitContent[] = []): MvpContent {
  const pool = mvpPool(rows);
  const body = { ...pool, units: [...pool.units, ...extra] };
  const version = "mvp-" + createHash("sha256").update(JSON.stringify(body)).digest("hex").slice(0, 10);
  return { version, ...body };
}

/** A unit that stays asleep at any copies: its awoken form is its sleeping one. */
function asleep(u: UnitContent): UnitContent {
  return { ...u, id: u.id + SLEEP_SUFFIX, forms: { sleeping: u.forms.sleeping, awoken: u.forms.sleeping } };
}

/** Today's field: the distinct teams in a meta-health report (its breakers
 * and its equilibrium), keeping only teams of units in `units`. */
export function fieldOfReport(report: { breakers: { team: Team }[]; equilibrium?: { team: Team }[] }, units: string[]): Team[] {
  const ok = new Set(units);
  const seen = new Set<string>();
  const out: Team[] = [];
  for (const t of [...(report.equilibrium ?? []).map((e) => e.team), ...report.breakers.map((b) => b.team)]) {
    const k = t.join(",");
    if (seen.has(k) || !t.every((id) => ok.has(id))) continue;
    seen.add(k);
    out.push(t);
  }
  return out;
}

/** A team's mean score against every field team: fixed seeds, so two
 * versions of one unit compare on the same fights. */
export function fieldScore(content: MvpContent, team: Team, field: Team[], s: TuneSettings): number {
  const sim = metaSim(content, s.seed, { copies: COPIES, searchSeeds: s.searchSeeds, searchSteps: 0 });
  return field.reduce((a, f) => a + sim.score(team, f, s.fieldSeeds, 2000), 0) / field.length;
}

/** The best team for `id` (breaker() from `restarts` starts, against a
 * gauntlet spread over the field), with its field score awake and asleep. */
export function measure(content: MvpContent, pool: UnitContent[], id: string, field: Team[], s: TuneSettings): Measure {
  const sim = metaSim(content, s.seed, { copies: COPIES, searchSeeds: s.searchSeeds, searchSteps: s.searchSteps, pool });
  const n = Math.min(s.gauntlet, field.length);
  const gauntlet = Array.from({ length: n }, (_, i) => field[Math.floor((i * field.length) / n)]!);
  let best = sim.breaker(id, gauntlet);
  for (let r = 1; r < s.restarts; r++) {
    const b = sim.breaker(id, gauntlet);
    if (b.score > best.score) best = b;
  }
  return measureTeam(content, best.team, id, field, s);
}

/** A fixed team's field score, awake and with `id` asleep. */
export function measureTeam(content: MvpContent, team: Team, id: string, field: Team[], s: TuneSettings): Measure {
  const asleepTeam = team.map((x) => (x === id ? id + SLEEP_SUFFIX : x));
  return { score: fieldScore(content, team, field, s), sleeping: fieldScore(content, asleepTeam, field, s), team };
}

function quantile(xs: number[], q: number): number {
  const s = [...xs].sort((a, b) => a - b);
  const pos = (s.length - 1) * q;
  const lo = Math.floor(pos);
  const hi = Math.ceil(pos);
  return s[lo]! + (s[hi]! - s[lo]!) * (pos - lo);
}

/** The band, measured on `units` (default every live unit) of the live pool. */
export function liveBand(liveRows: Row[], field: Team[], s: TuneSettings, units?: string[], log: (line: string) => void = () => {}): Band {
  const base = mvpPool(liveRows).units;
  const content = contentOf(liveRows, base.map(asleep));
  const ids = units ?? base.map((u) => u.id);
  const t0 = Date.now();
  const live = ids.map((id, i) => {
    const m = measure(content, base, id, field, s);
    if ((i + 1) % 10 === 0) log(`band ${i + 1}/${ids.length}  ${((Date.now() - t0) / 1000).toFixed(0)}s`);
    return { unit: id, score: +m.score.toFixed(3), margin: +(m.score - m.sleeping).toFixed(3) };
  });
  const scores = live.map((l) => l.score);
  const margin = Math.max(s.awokenMarginFloor, quantile(live.map((l) => l.margin), BAND_LOW_Q));
  return { low: quantile(scores, BAND_LOW_Q), high: quantile(scores, BAND_HIGH_Q), target: quantile(scores, 0.5), margin: +margin.toFixed(3), live };
}

// ---------- the numbers: PWR, HP and every "Word N" in the Does ----------

const NUM = /\b([A-Z][a-z]+) (\d+)\b/g;

/** A row's Does strings in a fixed order; the first is the sleeping Does. */
function doesStrings(row: Row): string[] {
  const a = row.awoken;
  return [row.does, a.more ?? "", ...(a.before ?? []), ...(a.add ?? []), a.also?.does ?? ""];
}

/** Each number in a row's Does, in doesStrings order: its word, value, and
 * whether only the Awoken form has it. */
export function numbersOf(row: Row): { word: string; value: number; awokenOnly: boolean }[] {
  return doesStrings(row).flatMap((d, i) => [...d.matchAll(NUM)].map((m) => ({ word: m[1]!, value: Number(m[2]), awokenOnly: i > 0 })));
}

/** The row with its Does numbers replaced, in numbersOf order. */
export function withNumbers(row: Row, values: number[]): Row {
  let k = 0;
  const sub = (d: string) => d.replace(NUM, (_, w: string) => `${w} ${values[k++]}`);
  const a = row.awoken;
  const awoken = { ...a };
  const does = sub(row.does);
  if (a.more !== undefined) awoken.more = sub(a.more);
  if (a.before) awoken.before = a.before.map(sub);
  if (a.add) awoken.add = a.add.map(sub);
  if (a.also) awoken.also = { ...a.also, does: sub(a.also.does) };
  return { ...row, does, awoken };
}

interface Knob {
  name: string;
  min: number;
  max: number;
  awokenOnly: boolean;
  get: (r: Row) => number;
  set: (r: Row, v: number) => Row;
}

/** The knobs the search turns, each bounded by what the live units use. */
export function knobsOf(row: Row, liveRows: Row[]): Knob[] {
  const pwrs = liveRows.map((r) => r.pwr);
  const hps = liveRows.map((r) => r.hp);
  const wordMax = new Map<string, number>();
  for (const r of liveRows) for (const n of numbersOf(r)) wordMax.set(n.word, Math.max(wordMax.get(n.word) ?? 1, n.value));
  const knobs: Knob[] = [
    { name: "PWR", min: Math.min(...pwrs), max: Math.max(...pwrs), awokenOnly: false, get: (r) => r.pwr, set: (r, v) => ({ ...r, pwr: v }) },
    { name: "HP", min: Math.min(...hps), max: Math.max(...hps), awokenOnly: false, get: (r) => r.hp, set: (r, v) => ({ ...r, hp: v }) },
  ];
  numbersOf(row).forEach((n, i) => {
    knobs.push({
      name: `${n.awokenOnly ? "Awoken " : ""}${n.word}`,
      min: 1,
      max: wordMax.get(n.word) ?? 3,
      awokenOnly: n.awokenOnly,
      get: (r) => numbersOf(r)[i]!.value,
      set: (r, v) => withNumbers(r, numbersOf(r).map((x, j) => (j === i ? v : x.value))),
    });
  });
  return knobs;
}

/** Why the row's Awoken form drops something its sleeping one has, or null. */
function keepsProblem(row: Row): string | null {
  const u = mvpPool([row]).units[0]!;
  return awokenKeeps(u.forms.sleeping, u.forms.awoken);
}

/** The row with every number moved into its knob's bounds, Awoken numbers
 * raised to at least the sleeping ones they enhance. */
function clampRow(row: Row, knobs: Knob[]): Row {
  let r = row;
  for (const k of knobs) r = k.set(r, Math.min(k.max, Math.max(k.min, k.get(r))));
  for (let guard = 0; keepsProblem(r) && guard < 20; guard++) for (const k of knobs) if (k.awokenOnly && k.get(r) < k.max) r = k.set(r, k.get(r) + 1);
  return r;
}

/** The row's numbers set at random inside their bounds (for tests and
 * --scramble), seeded. */
export function scrambleRow(row: Row, liveRows: Row[], seed = 7): Row {
  let state = seed;
  const rand = () => {
    state = (state * 1103515245 + 12345) & 0x7fffffff;
    return state / 0x7fffffff;
  };
  const knobs = knobsOf(row, liveRows);
  let r = row;
  for (const k of knobs) {
    // Away from the original: the far half of the range on a random side.
    const v = k.get(row);
    const up = rand() < 0.5 ? v < k.max : v <= k.min;
    const lo = up ? Math.ceil((v + k.max) / 2) : k.min;
    const hi = up ? k.max : Math.floor((v + k.min) / 2);
    r = k.set(r, Math.max(k.min, Math.min(k.max, lo + Math.floor(rand() * (hi - lo + 1)))));
  }
  return clampRow(r, knobs);
}

const THEME: Record<string, string> = {
  Hit: "damage", Smite: "damage", Poison: "Poison", Curse: "Curse", Freeze: "Freeze", Silence: "Silence", Shield: "Shield",
  Heal: "healing", Mend: "healing", Bless: "Blessing", Strength: "Strength", Vitality: "Vitality", Call: "summon", Revive: "summon",
};

/** What a team is about, in a word: its most common effect kind. */
function themeOf(team: Team, content: MvpContent): string {
  const count = new Map<string, number>();
  for (const id of team) {
    const u = content.units.find((x) => x.id === id);
    if (!u) continue;
    for (const k of effectKinds(allDoes(u.forms.awoken))) {
      const t = THEME[k];
      if (t) count.set(t, (count.get(t) ?? 0) + 1);
    }
  }
  return [...count.entries()].sort((a, b) => b[1] - a[1])[0]?.[0] ?? "mixed";
}

const pct = (x: number) => `${Math.round(x * 100)}%`;

export interface TuneOptions {
  liveRows: Row[];
  field: Team[];
  band: Band;
  settings?: TuneSettings;
  /** Live units never drawn as its teammates (a copy's original). */
  notWith?: string[];
  /** The whole meta check with the unit added; null skips it. */
  meta?: MetaSettings | null;
  log?: (line: string) => void;
}

/** Tune one candidate's numbers, then judge it. */
export function tuneUnit(candidate: Row, o: TuneOptions): TuneResult {
  const s = o.settings ?? TUNE_DEFAULT;
  const log = o.log ?? (() => {});
  const t0 = Date.now();
  const { band } = o;
  const id = slugOf(candidate.name);
  if (o.liveRows.some((r) => slugOf(r.name) === id)) throw new Error(`candidate id "${id}" is already a live unit`);
  const livePool = mvpPool(o.liveRows).units.filter((u) => !(o.notWith ?? []).includes(u.id));
  const knobs = knobsOf(candidate, o.liveRows);

  const contentFor = (row: Row) => {
    const u = mvpPool([row]).units[0]!;
    return { u, content: contentOf([...o.liveRows, row], [asleep(u)]) };
  };
  // The unit at its best: a breaker search for its team (slow, and the team
  // found moves with luck), and its numbers measured on one team (fast, and
  // two versions of the unit meet the same fights).
  const search = (row: Row): Measure => {
    const { u, content } = contentFor(row);
    return measure(content, [...livePool, u], id, o.field, s);
  };
  const onTeam = (row: Row, team: Team): Measure => measureTeam(contentFor(row).content, team, id, o.field, s);
  const margin = (m: Measure) => m.score - m.sleeping;
  // The search aims just inside the band, so numbers that already fit stay
  // as they are and a unit out of it moves the least it must.
  const w = band.high - band.low;
  const [lo, hi] = [band.low + w / 8, band.high - w / 8];
  const off = (m: Measure) => (m.score < lo ? lo - m.score : m.score > hi ? m.score - hi : 0);
  const cost = (m: Measure) => off(m) + 2 * Math.max(0, band.margin - margin(m));
  const done = (m: Measure) => off(m) === 0 && margin(m) >= band.margin;
  const show = (r: Row) => `${r.pwr}/${r.hp} ${doesStrings(r).filter(Boolean).join(" · ")}`;
  const line = (m: Measure) => `score ${pct(m.score)}, asleep ${pct(m.sleeping)} (band ${pct(band.low)}–${pct(band.high)})`;

  let row = clampRow(candidate, knobs);
  if (keepsProblem(row)) throw new Error(`${candidate.name}: ${keepsProblem(row)}`);
  let cur = search(row);
  const steps: TuneStep[] = [{ row, measure: cur, kept: true, change: "start" }];
  log(`start   ${show(row)}  ${line(cur)}`);
  log(`        team ${cur.team.join(" · ")}`);

  for (let round = 0; round < 3 && steps.length < s.maxEvals; round++) {
    const tried = new Set<string>();
    const tries = new Map<string, number>();
    while (steps.length < s.maxEvals && !done(cur)) {
      // Weaker or stronger: the numbers all push the same way. Awoken's own
      // numbers widen its margin over sleeping.
      const dir = cur.score > (lo + hi) / 2 ? -1 : 1;
      const wantMargin = margin(cur) < band.margin;
      const far = off(cur) > w / 4;
      const moves: { knob: Knob; to: number }[] = [];
      for (const k of knobs) {
        const v = k.get(row);
        const d = wantMargin && k.awokenOnly ? 1 : dir;
        const by = far && k.name === "HP" ? 2 : 1;
        const to = Math.max(k.min, Math.min(k.max, v + d * by));
        if (to !== v && !tried.has(`${show(row)}|${k.name}|${to}`)) moves.push({ knob: k, to });
      }
      if (!moves.length) break;
      // Awoken-only knobs first when its margin is short; otherwise the knob
      // turned least so far.
      const turned = (k: Knob) => tries.get(k.name) ?? 0;
      moves.sort((x, y) => (wantMargin ? Number(y.knob.awokenOnly) - Number(x.knob.awokenOnly) : 0) || turned(x.knob) - turned(y.knob));
      const mv = moves[0]!;
      tries.set(mv.knob.name, turned(mv.knob) + 1);
      tried.add(`${show(row)}|${mv.knob.name}|${mv.to}`);
      const next = clampRow(mv.knob.set(row, mv.to), knobs);
      if (keepsProblem(next)) continue;
      const m = onTeam(next, cur.team);
      // Ties walk on, so a unit no number tames reaches its bounds.
      const kept = cost(m) <= cost(cur);
      const change = `${mv.knob.name} ${mv.knob.get(row)}→${mv.to}`;
      steps.push({ row: next, measure: m, kept, change });
      log(`${kept ? "keep" : "drop"}    ${change.padEnd(20)} ${line(m)}`);
      if (kept) [row, cur] = [next, m];
    }
    // The best team for the new numbers may be another one.
    const again = search(row);
    steps.push({ row, measure: again, kept: true, change: "new team search" });
    log(`search  ${show(row)}  ${line(again)}`);
    const same = again.team.join(",") === cur.team.join(",");
    if (!same) log(`        team ${again.team.join(" · ")}`);
    cur = again;
    if (same || done(cur)) break;
  }

  const atBounds = (d: 1 | -1) => knobs.every((k) => k.awokenOnly || k.get(row) === (d > 0 ? k.max : k.min));
  const finish = (pass: boolean, reason: string, meta?: TuneResult["meta"]): TuneResult => {
    log(`${pass ? "PASS" : "FAIL"}: ${reason}`);
    return { row, pass, reason, measure: cur, band, steps, ...(meta ? { meta } : {}), seconds: Math.round((Date.now() - t0) / 1000) };
  };
  const content = contentOf([...o.liveRows, row]);
  if (cur.score > band.high)
    return finish(false, `too strong in a ${themeOf(cur.team, content)} team${atBounds(-1) ? ", even at its lowest numbers" : ""}`);
  if (cur.score < band.low) return finish(false, `doesn't matter in any team${atBounds(1) ? ", even at its highest numbers" : ""}`);
  if (margin(cur) < band.margin) return finish(false, `Awoken is barely stronger than sleeping (+${pct(Math.max(0, margin(cur)))}, needs +${pct(band.margin)})`);

  if (o.meta === null) return finish(true, `fits: ${line(cur)}`);
  log("meta check with the unit added…");
  const r = metaHealth(content, DEFAULT_SEED, o.meta ?? META_QUICK, log);
  const meta = { healthy: r.healthy, share: +(r.unitShare.get(id) ?? 0).toFixed(3), checks: r.checks };
  if (!r.healthy) {
    const c = r.checks;
    const why = !c.largestShare.pass
      ? `one team takes ${pct(c.largestShare.value)} of the meta`
      : !c.teamsInEquilibrium.pass
        ? `only ${c.teamsInEquilibrium.value} teams stay worth playing`
        : `a top team has only ${c.topTeamsBeaten.value} answers`;
    return finish(false, `breaks the meta: ${why}`, meta);
  }
  return finish(true, `fits: ${line(cur)}; meta healthy with it in ${pct(meta.share)} of play`, meta);
}

/** Tune every candidate waiting, one at a time. */
export function tuneBatch(candidates: Row[], o: TuneOptions, onResult: (i: number, r: TuneResult) => void = () => {}): TuneResult[] {
  const log = o.log ?? (() => {});
  return candidates.map((c, i) => {
    log(`— ${i + 1}/${candidates.length}: ${c.emoji} ${c.name}`);
    const r = tuneUnit(c, o);
    onResult(i, r);
    return r;
  });
}

/** Rows for trying the tuner: one no numbers can tame. */
export const TEST_ROWS: Row[] = [
  {
    name: "Tyrant", emoji: "🦖", tier: 4, pwr: 4, hp: 12, when: "turnStart", who: "enemies", does: "Freeze 1 + Curse 1",
    archetype: "Freezes and saps the whole enemy line every turn.", awoken: { before: ["Silence"], add: ["Poison 1"] },
  },
];
