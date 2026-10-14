// The night batch (rules v2 §6, mission makscee/void-board#1043 slice 7,
// makscee/void-board#1050): a server job that designs fusions ahead of play.
//
//   tonight's list   pickPairs: requested pairs, then wished, then the pairs
//                    bots reach most, then a spread over the newest units;
//                    order dropped (A+B = B+A); a pair designed in the last
//                    week waits unless players passed on it.
//   design           a Designer (Opus via `claude -p`, or the fake) returns
//                    three candidates per pair: lean A, lean B, twist; a
//                    Namer (Sonnet) gives each a name and flavour EN/RU.
//   compile          composed words expanded, the rule validated; a design
//                    that asks for a word the registry lacks is stored as a
//                    word request, not bent.
//   sim check        it terminates (no chain cap hit, mostly not time-up).
//   quick tune       the rule's numbers scaled until the fusion's score
//                    against the day's ghosts (rounds 8-12) plus the meta's
//                    breakers sits in band; breakers only under 50 ghosts.
//   store            three candidates per pair, as `ready to claim`.
//
// No numbers come from a model: stats are the parents' sum (fusedBase), the rule's cost
// is what the quick tune measured.

import { battle } from "../../../src/battle.js";
import type { DescribeSegment, Lang } from "../../../src/describe.js";
import { formSegments } from "../../../src/mvp/form-text.js";
import type { AbilityDef, AbilityRegistry, Amount, BattleEvent, Effect, StatusDef, StatusRegistry, UnitDef } from "../../../src/types.js";
import { validateAbilityRegistry, validateTeam } from "../../../src/validate.js";
import { expandAbilities, expandStatuses } from "../../../src/words/macros.js";
import { namesMissing } from "../../../src/status-names.js";
import { traceLines } from "../../../src/words/trace.js";

// ---------- tonight's list ----------

export const BATCH_MIN = 50;
export const BATCH_MAX = 100;
export const REDESIGN_AFTER_MS = 7 * 24 * 3600 * 1000;

/** An unordered pair: "A+B" with the names sorted. */
export const pairKey = (a: string, b: string): string => (a < b ? `${a}+${b}` : `${b}+${a}`);

export interface PairHistory {
  /** When its candidates were last designed. */
  designedAt: number;
  claimed?: boolean;
  /** Players passed on its candidates: redesign it tonight. */
  passed?: boolean;
}

export interface ListInput {
  /** The live pool, each unit with when it arrived. */
  units: { id: string; addedAt: number }[];
  /** pairKey → how often bots reached it (both parents awoken on one line). */
  botReach?: Record<string, number>;
  /** pairKey → wishes. */
  wishes?: Record<string, number>;
  /** pairKeys players requested at the forge (a hard gate). */
  requests?: string[];
  /** pairKey → what happened to it before. */
  history?: Record<string, PairHistory>;
  now: number;
  /** How many pairs tonight; clamped to 50-100 unless `small` (a try run). */
  size?: number;
  small?: boolean;
}

export interface ListedPair {
  key: string;
  a: string;
  b: string;
  why: "requested" | "wished" | "bots" | "newest";
}

/** Tonight's list, most wanted first. */
export function pickPairs(input: ListInput): ListedPair[] {
  const want = input.small ? Math.max(1, input.size ?? 5) : Math.min(BATCH_MAX, Math.max(BATCH_MIN, input.size ?? 75));
  const ids = new Set(input.units.map((u) => u.id));
  const history = input.history ?? {};
  const open = (key: string): boolean => {
    const [a, b] = key.split("+");
    if (!a || !b || a === b || !ids.has(a) || !ids.has(b)) return false;
    const h = history[key];
    if (!h) return true;
    if (h.claimed) return false;
    return h.passed === true || input.now - h.designedAt >= REDESIGN_AFTER_MS;
  };
  const out: ListedPair[] = [];
  const seen = new Set<string>();
  const add = (key: string, why: ListedPair["why"]): void => {
    if (out.length >= want || seen.has(key) || !open(key)) return;
    seen.add(key);
    const [a, b] = key.split("+") as [string, string];
    out.push({ key, a, b, why });
  };
  const norm = (k: string): string => {
    const [a, b] = k.split("+");
    return a && b ? pairKey(a, b) : k;
  };
  const ranked = (m: Record<string, number> = {}): string[] =>
    Object.entries(m)
      .map(([k, n]) => [norm(k), n] as const)
      .sort((x, y) => y[1] - x[1] || (x[0] < y[0] ? -1 : 1))
      .map(([k]) => k);
  for (const k of input.requests ?? []) add(norm(k), "requested");
  for (const k of ranked(input.wishes)) add(k, "wished");
  // bots get two thirds of what's left, the newest units the rest
  const botShare = out.length + Math.ceil(((want - out.length) * 2) / 3);
  for (const k of ranked(input.botReach)) {
    if (out.length >= botShare) break;
    add(k, "bots");
  }
  // a spread: each newest unit in turn takes its next partner
  const newest = [...input.units].sort((x, y) => y.addedAt - x.addedAt || (x.id < y.id ? -1 : 1));
  const n = newest.length;
  for (let gap = 1; out.length < want && gap < n; gap++) {
    for (let i = 0; i + gap < n && out.length < want; i++) add(pairKey(newest[i]!.id, newest[i + gap]!.id), "newest");
  }
  return out;
}

// ---------- the designer's answer ----------

export type Lean = "A" | "B" | "twist";
export const LEANS: Lean[] = ["A", "B", "twist"];

/** One parent as the designer sees it: its rule and the registry entries it uses. */
export interface Parent {
  id: string;
  emoji?: string;
  def: UnitDef;
  abilities: AbilityRegistry;
  statuses: StatusRegistry;
}

/** What the designer returns per candidate: a v2 rule, no stats. */
export interface DesignDraft {
  lean: Lean;
  /** One sentence the designer meant, to check the compiled rule against. */
  intent: string;
  rule: Pick<UnitDef, "triggers" | "condition" | "selectors" | "abilities" | "also" | "statuses">;
  abilities: Record<string, AbilityDef>;
  statuses?: Record<string, StatusDef>;
  /** A word the design needs and the registry lacks (§7): the pair waits for it. */
  wordRequest?: { word: string; meaning: string };
}

export interface Naming {
  name: { en: string; ru: string };
  flavour: { en: string; ru: string };
}

export interface Designer {
  readonly kind: string;
  design(a: Parent, b: Parent): Promise<DesignDraft[]>;
  /** `taken`: names the pair's earlier candidates got (each name is unique within the pair). */
  name(a: Parent, b: Parent, d: DesignDraft, taken: string[]): Promise<Naming>;
}

// ---------- compile, check, tune ----------

/** The day's opponents: ghosts are saved lines from rounds 8-12; breakers are
 * the meta's strongest teams, used only when there are under 50 ghosts. */
export interface Field {
  ghosts: UnitDef[][];
  breakers: UnitDef[][];
  abilities: AbilityRegistry;
  statuses: StatusRegistry;
}

export const BREAKERS_UNDER = 50;
/** In band: the fusion's line wins this share of its games against the field. */
export const BAND = { low: 0.35, high: 0.75 };
export const TUNE_SCALES = [1, 0.5, 1.5, 0.75, 2, 0.25, 3];

export interface Compiled {
  unit: UnitDef;
  abilities: AbilityRegistry;
  statuses: StatusRegistry;
}

export interface CheckResult {
  terminates: boolean;
  games: number;
  capped: number;
  timeUp: number;
  score: number;
  inBand: boolean;
  scale: number;
  trace?: string;
}

export interface Candidate {
  pair: string;
  lean: Lean;
  intent: string;
  name: { en: string; ru: string };
  flavour: { en: string; ru: string };
  /** The rule's sentence, rendered from the compiled rule, not from the model. */
  text: { en: string; ru: string };
  unit: UnitDef;
  abilities: AbilityRegistry;
  statuses: StatusRegistry;
  check: CheckResult | null;
  /** ready: claimable; word: waits for a word request; rejected: why. */
  state: "ready" | "word" | "rejected";
  why?: string;
  wordRequest?: { word: string; meaning: string };
}

/** A fusion's stats as the game's fuse gives them: the parents' sum. */
export function fusedBase(a: Parent, b: Parent): { hp: number; pwr: number } {
  return { hp: a.def.base.hp + b.def.base.hp, pwr: a.def.base.pwr + b.def.base.pwr };
}

/** The draft's own ids (name = key, as the validator wants). An id the
 * parents or the field already have (Shield, Strength) is the registry's: the
 * draft's redeclaration is dropped, so no "Shield2" reaches the card. Each
 * candidate fights with its own registry, so two pairs' "Echo" never meet. */
function owned(d: DesignDraft, taken: Set<string>): { rule: DesignDraft["rule"]; abilities: AbilityRegistry; statuses: StatusRegistry } {
  const aMap = new Map(Object.keys(d.abilities ?? {}).map((id) => [id, id]));
  const sMap = new Map(Object.keys(d.statuses ?? {}).map((id) => [id, id]));
  const walk = (x: unknown): unknown => {
    if (Array.isArray(x)) return x.map(walk);
    if (x && typeof x === "object") {
      const o: Record<string, unknown> = {};
      for (const [k, v] of Object.entries(x)) {
        if (k === "status" && typeof v === "string") o[k] = sMap.get(v) ?? v;
        else if ((k === "abilities" || k === "does") && Array.isArray(v) && v.every((s) => typeof s === "string")) o[k] = v.map((s) => aMap.get(s as string) ?? s);
        else o[k] = walk(v);
      }
      return o;
    }
    return x;
  };
  const abilities: AbilityRegistry = {};
  for (const [id, def] of Object.entries(d.abilities ?? {})) if (!taken.has(id)) abilities[id] = { ...(walk(def) as AbilityDef), name: id };
  const statuses: StatusRegistry = {};
  for (const [id, def] of Object.entries(d.statuses ?? {})) if (!taken.has(id)) statuses[id] = { ...(walk(def) as StatusDef), name: id };
  return { rule: walk(d.rule) as DesignDraft["rule"], abilities, statuses };
}

/** The entries of `k` the unit and its own entries reach, through abilities
 * and statuses (so a shared registry adds only the words the rule uses). */
function reachedOnly(unit: UnitDef, statuses: StatusRegistry, abilities: AbilityRegistry, own: { abilities: AbilityRegistry; statuses: StatusRegistry }, k: "abilities" | "statuses"): Record<string, never> {
  const seen = { abilities: new Set<string>(), statuses: new Set<string>() };
  const visit = (x: unknown): void => {
    if (Array.isArray(x)) return x.forEach(visit);
    if (!x || typeof x !== "object") return;
    for (const [key, v] of Object.entries(x)) {
      if (key === "status" && typeof v === "string") {
        if (!seen.statuses.has(v)) {
          seen.statuses.add(v);
          visit(statuses[v]);
        }
      } else if ((key === "abilities" || key === "does") && Array.isArray(v) && v.every((id) => typeof id === "string")) {
        for (const id of v as string[])
          if (!seen.abilities.has(id)) {
            seen.abilities.add(id);
            visit(abilities[id]);
          }
      } else visit(v);
    }
  };
  visit(unit);
  visit(own.abilities);
  visit(own.statuses);
  const reg = (k === "abilities" ? abilities : statuses) as Record<string, never>;
  // an unknown id stays unknown, so the validator still names it
  return Object.fromEntries([...seen[k], ...Object.keys(own[k])].filter((id) => reg[id]).map((id) => [id, reg[id]!]));
}

/** The draft as a unit with the game's fused stats, or the issues. */
export function compile(d: DesignDraft, a: Parent, b: Parent, name: string, taken: Set<string>, shared?: Pick<Field, "abilities" | "statuses">): { ok: Compiled; own: Set<string> } | { issues: string[] } {
  const n = owned(d, taken);
  const unit: UnitDef = {
    name,
    base: fusedBase(a, b),
    ...(n.rule.triggers ? { triggers: n.rule.triggers } : {}),
    ...(n.rule.condition ? { condition: n.rule.condition } : {}),
    ...(n.rule.selectors ? { selectors: n.rule.selectors } : {}),
    ...(n.rule.abilities ? { abilities: n.rule.abilities } : {}),
    ...(n.rule.also?.length ? { also: n.rule.also } : {}),
    ...(n.rule.statuses?.length ? { statuses: n.rule.statuses } : {}),
  };
  // the pool's shared words (#1065: Strength, Curse, Charge …) under the parents' and its own
  const statuses = reachedOnly(unit, { ...shared?.statuses, ...a.statuses, ...b.statuses, ...n.statuses }, { ...shared?.abilities, ...a.abilities, ...b.abilities, ...n.abilities }, n, "statuses");
  const abilities = reachedOnly(unit, { ...shared?.statuses, ...a.statuses, ...b.statuses, ...n.statuses }, { ...shared?.abilities, ...a.abilities, ...b.abilities, ...n.abilities }, n, "abilities");
  // A coined status says itself in EN and RU, or the rule isn't legal (§3 guardrail).
  const unnamed = Object.entries(n.statuses).flatMap(([id, def]) => {
    const why = namesMissing(def);
    return why ? [`statuses.${id}: ${why}`] : [];
  });
  if (unnamed.length) return { issues: unnamed };
  let issues: string[];
  try {
    const xs = expandStatuses(statuses);
    const xa = expandAbilities(abilities);
    // Status rules are checked by the sim (the validator's status check
    // predates v2 counters with triggers of their own, as the fixtures use).
    issues = [...validateAbilityRegistry(xa, xs), ...validateTeam([unit], xs, xa)].map((i) => `${i.path}: ${i.message}`);
  } catch (e) {
    issues = [(e as Error).message];
  }
  return issues.length ? { issues } : { ok: { unit, abilities, statuses }, own: new Set([...Object.keys(n.abilities), ...Object.keys(n.statuses)]) };
}

/** Every `const` amount in the rule's own abilities and statuses, scaled
 * (at least 1). The quick tune's one knob. A counter's spend is left as
 * designed: it matches the counter's threshold ("at 3+ Glints … spend 3"),
 * which is rule, not power (#1047: Echo Sands spent 5 at 3+). */
export function scaled(c: Compiled, own: Set<string>, scale: number): Compiled {
  const walk = (x: unknown): unknown => {
    if (Array.isArray(x)) return x.map(walk);
    if (x && typeof x === "object") {
      const o = x as Record<string, unknown>;
      if (o.kind === "consumeStacks") return o;
      if (o.kind === "const" && typeof o.value === "number" && Object.keys(o).length === 2) return { kind: "const", value: Math.max(1, Math.round(o.value * scale)) } satisfies Amount;
      return Object.fromEntries(Object.entries(o).map(([k, v]) => [k, walk(v)]));
    }
    return x;
  };
  const mine = <T>(reg: Record<string, T>): Record<string, T> => Object.fromEntries(Object.entries(reg).map(([id, d]) => [id, own.has(id) ? (walk(d) as T) : d]));
  return { unit: c.unit, abilities: mine(c.abilities), statuses: mine(c.statuses) };
}

const ended = (log: readonly BattleEvent[]) => log.find((e) => e.type === "BattleEnd") as Extract<BattleEvent, { type: "BattleEnd" }> | undefined;

/** The fusion leads each ghost line (in place of its front unit) against every
 * other line of the field; the score is its line's share of wins (draws half). */
export function measureFusion(c: Compiled, field: Field, seeds = 2): CheckResult {
  const opponents = field.ghosts.length < BREAKERS_UNDER ? [...field.ghosts, ...field.breakers] : field.ghosts;
  const statuses = { ...field.statuses, ...c.statuses };
  const abilities = { ...field.abilities, ...c.abilities };
  let games = 0;
  let capped = 0;
  let timeUp = 0;
  let points = 0;
  let trace: string | undefined;
  const lines = field.ghosts.length ? field.ghosts : [[]];
  for (let i = 0; i < lines.length; i++) {
    const mine = [c.unit, ...(lines[i] ?? []).slice(1)];
    for (let j = 0; j < opponents.length; j++) {
      if (opponents[j] === lines[i]) continue;
      for (let s = 1; s <= seeds; s++) {
        const input = { teamA: mine, teamB: opponents[j]!, seed: 1000 * i + 10 * j + s, statuses, abilities };
        let log: BattleEvent[];
        try {
          log = battle(input);
        } catch {
          capped++;
          games++;
          continue;
        }
        games++;
        if (log.some((e) => e.type === "ChainCapped")) capped++;
        const end = ended(log);
        if (!end) {
          capped++;
          continue;
        }
        if (end.timeUp) timeUp++;
        points += end.winner === "A" ? 1 : end.winner === "draw" ? 0.5 : 0;
        if (!trace) trace = traceLines(input, log).find((l) => l.startsWith(c.unit.name));
      }
    }
  }
  const score = games ? points / games : 0;
  const terminates = games > 0 && capped === 0 && timeUp <= games / 2;
  return { terminates, games, capped, timeUp, score, inBand: terminates && score >= BAND.low && score <= BAND.high, scale: 1, ...(trace ? { trace } : {}) };
}

/** Tries the scales in order and keeps the first in band, else the closest. */
export function quickTune(c: Compiled, own: Set<string>, field: Field): { compiled: Compiled; check: CheckResult } {
  let best: { compiled: Compiled; check: CheckResult; miss: number } | undefined;
  for (const scale of TUNE_SCALES) {
    const compiled = scaled(c, own, scale);
    const check = { ...measureFusion(compiled, field), scale };
    if (check.inBand) return { compiled, check };
    const miss = !check.terminates ? 9 : check.score < BAND.low ? BAND.low - check.score : check.score - BAND.high;
    if (!best || miss < best.miss) best = { compiled, check, miss };
  }
  return { compiled: best!.compiled, check: best!.check };
}

const sentence = (segs: DescribeSegment[]): string => segs.map((s) => s.text).join("");

/** The compiled rule as the card's sentence, EN or RU. */
export function ruleText(c: Compiled, lang: Lang): string {
  const u = c.unit;
  const abilities = expandAbilities(c.abilities);
  try {
    return sentence(
      formSegments(
        { when: u.triggers ?? [], ...(u.condition ? { condition: u.condition } : {}), who: u.selectors ?? [], does: u.abilities ?? [], also: (u.also ?? []).map((x) => ({ who: x.selectors, does: x.abilities })) },
        abilities,
        lang,
        c.statuses,
        (u.statuses ?? []).map((s) => s.status),
      ),
    );
  } catch (e) {
    return `(${(e as Error).message})`;
  }
}

// ---------- the batch ----------

export interface BatchOptions {
  designer: Designer;
  field: Field;
  /** id → the parent as the designer sees it. */
  parents: Record<string, Parent>;
  /** Pairs designed at once (each pair is one Opus call plus three Sonnet calls). */
  parallel?: number;
  log?: (line: string) => void;
}

export interface PairResult {
  pair: ListedPair;
  candidates: Candidate[];
  error?: string;
}

/** Designs, compiles, checks and tunes every listed pair. */
export async function runBatch(list: ListedPair[], o: BatchOptions): Promise<PairResult[]> {
  const log = o.log ?? (() => {});
  const out: PairResult[] = new Array(list.length);
  let next = 0;
  const work = async (): Promise<void> => {
    for (let i = next++; i < list.length; i = next++) {
      const pair = list[i]!;
      const a = o.parents[pair.a];
      const b = o.parents[pair.b];
      if (!a || !b) {
        out[i] = { pair, candidates: [], error: "parent missing" };
        continue;
      }
      try {
        out[i] = { pair, candidates: await designPair(pair, a, b, o) };
        log(`${pair.key}: ${out[i]!.candidates.map((c) => `${c.lean} ${c.state}${c.check ? ` ${c.check.score.toFixed(2)}` : ""}`).join(", ")}`);
      } catch (e) {
        out[i] = { pair, candidates: [], error: (e as Error).message.slice(0, 300) };
        log(`${pair.key}: ${out[i]!.error}`);
      }
    }
  };
  await Promise.all(Array.from({ length: Math.max(1, o.parallel ?? 3) }, work));
  return out;
}

async function designPair(pair: ListedPair, a: Parent, b: Parent, o: BatchOptions): Promise<Candidate[]> {
  const drafts = (await o.designer.design(a, b)).slice(0, 3);
  // named one after another, so the namer sees the names the pair already has
  const namings: Naming[] = [];
  for (const d of drafts) {
    const taken = namings.map((n) => n.name.en);
    let naming = await o.designer.name(a, b, d, taken).catch((): Naming => ({ name: { en: `${pair.a}·${pair.b} ${d.lean}`, ru: `${pair.a}·${pair.b} ${d.lean}` }, flavour: { en: "", ru: "" } }));
    if (taken.includes(naming.name.en)) naming = { ...naming, name: { en: `${naming.name.en} ${d.lean}`, ru: `${naming.name.ru} ${d.lean}` } };
    namings.push(naming);
  }
  return Promise.all(
    drafts.map(async (d, i): Promise<Candidate> => {
      const naming = namings[i]!;
      const base = { pair: pair.key, lean: d.lean, intent: d.intent ?? "", name: naming.name, flavour: naming.flavour };
      const empty = { unit: { name: naming.name.en, base: fusedBase(a, b) }, abilities: {}, statuses: {}, text: { en: "", ru: "" }, check: null };
      if (d.wordRequest?.word) return { ...base, ...empty, state: "word", wordRequest: d.wordRequest, why: `waiting for a new word: ${d.wordRequest.word}` };
      const taken = new Set([...Object.keys(a.abilities), ...Object.keys(b.abilities), ...Object.keys(a.statuses), ...Object.keys(b.statuses), ...Object.keys(o.field.abilities), ...Object.keys(o.field.statuses)]);
      const c = compile(d, a, b, naming.name.en, taken, o.field);
      if ("issues" in c) return { ...base, ...empty, state: "rejected", why: c.issues.slice(0, 3).join("; ") };
      const tuned = quickTune(c.ok, c.own, o.field);
      const text = { en: ruleText(tuned.compiled, "en"), ru: ruleText(tuned.compiled, "ru") };
      // in band, or as close as the tune's scales get: out of band is stored with its why, not claimable
      const state = tuned.check.inBand ? "ready" : "rejected";
      const why = !tuned.check.terminates ? `doesn't terminate (${tuned.check.capped} capped, ${tuned.check.timeUp} time-up of ${tuned.check.games})` : `out of band: wins ${Math.round(tuned.check.score * 100)}% at best (band ${BAND.low * 100}-${BAND.high * 100}%)`;
      return {
        ...base,
        unit: tuned.compiled.unit,
        // its own entries plus the parents' it reuses: everything it needs to fight
        abilities: tuned.compiled.abilities,
        statuses: tuned.compiled.statuses,
        text,
        check: tuned.check,
        state,
        ...(state === "rejected" ? { why } : {}),
      };
    }),
  );
}
