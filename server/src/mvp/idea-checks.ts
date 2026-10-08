// The checks a read idea passes (mission 2, M2-5, makscee/void-board#792).
// The reader (./idea-reader.ts) only drafts; nothing it writes reaches a
// player unless it passes here. An archetype is a name, an emoji and a line in
// the game's words; a reading is a `Row` built only from WHEN, WHO and the
// Does words of src/mvp/units.ts, with numbers as placeholders (M2-7 tunes
// them), shown only as formText renders it.
import type { IdeaArchetype, IdeaReading } from "../../../src/mvp/contract.js";
import { formText } from "../../../src/mvp/form-text.js";
import { formProblems } from "../../../src/mvp/forms.js";
import { archetypeProblems, awokenKeeps, linkLoops, mvpPool, sig, WHEN, WHO, type AwokenRow, type Row, type WhenKey, type WhoKey } from "../../../src/mvp/units.js";
import { hasCrudeStem, isCrudeName } from "./crude.js";

/** A candidate's numbers until M2-7's tuner sets them. */
export const PLACEHOLDER_STATS: Pick<Row, "tier" | "pwr" | "hp"> = { tier: 2, pwr: 2, hp: 5 };

/** An archetype as the reader drafted it: unchecked strings. */
export interface ArchetypeDraft {
  name: string;
  emoji: string;
  line: string;
}

/** A reading as the reader drafted it: unchecked strings. */
export interface ReadingDraft {
  when: string;
  who: string;
  does: string;
  awoken: { who?: string; more?: string; before?: string[]; add?: string[]; also?: { who: string; does: string } };
}

const NAME = /^[A-Z][A-Za-z]*(?:[ '-][A-Za-z]+){0,2}$/;
const NAME_MAX = 20;
const LINE_MAX = 120;
/** One emoji: pictographs, joiners, variation selectors and skin tones; no ASCII. */
const EMOJI = /^(?:\p{Extended_Pictographic}|\p{Emoji_Modifier}|\p{Regional_Indicator}|‍|️)+$/u;

/** What's wrong with a drafted archetype; empty when it can be shown. `taken`
 * is every unit and candidate so far (any status) plus the options already
 * accepted: the name and the line must differ from all of them. */
export function archetypeDraftProblems(d: ArchetypeDraft, taken: Pick<Row, "name" | "archetype">[]): string[] {
  const out: string[] = [];
  const name = typeof d.name === "string" ? d.name.trim() : "";
  const emoji = typeof d.emoji === "string" ? d.emoji.trim() : "";
  const line = typeof d.line === "string" ? d.line.trim() : "";
  const at = `"${name}"`;
  if (!NAME.test(name) || name.length > NAME_MAX) out.push(`${at}: a name is 1–3 words of letters, ${NAME_MAX} characters at most, starting with a capital`);
  if (isCrudeName(name) || hasCrudeStem(name)) out.push(`${at}: the name reads crude`);
  if (taken.some((u) => u.name.toLowerCase() === name.toLowerCase())) out.push(`${at}: a unit with this name exists`);
  if (!EMOJI.test(emoji) || [...emoji].length > 8) out.push(`${at}: the emoji must be one emoji and nothing else`);
  if (line.length > LINE_MAX) out.push(`${at}: the line is over ${LINE_MAX} characters`);
  // Word by word: isCrudeName joins a name's words, which a sentence's
  // neighbours would trip ("numbs the").
  if (line.split(/[^A-Za-z'’-]+/).some((w) => w && isCrudeName(w))) out.push(`${at}: the line reads crude`);
  // archetypeProblems over the taken lines plus this one: only what this one adds.
  const before = new Set(archetypeProblems(taken));
  const self = "\u0000candidate";
  for (const p of archetypeProblems([...taken, { name: self, archetype: line }])) {
    if (!before.has(p)) out.push(p.replace(self, at));
  }
  return out;
}

/** What `units` already take: the shapes of both forms (R1, R2) and the
 * listen → emit loops (R4). Built once per batch; `accept` adds a reading's
 * row so the next options must differ from it too. */
export interface TakenShapes {
  rows: Row[];
  sleeping: Set<string>;
  awoken: Set<string>;
  loops: Set<string>;
  accept(row: Row): void;
}

export function takenShapes(rows: Row[]): TakenShapes {
  const t: TakenShapes = {
    rows: [...rows],
    sleeping: new Set(),
    awoken: new Set(),
    loops: new Set(linkLoops(mvpPool(rows).units)),
    accept(row) {
      const u = mvpPool([row]).units[0]!;
      t.rows.push(row);
      t.sleeping.add(sig(u.forms.sleeping));
      t.awoken.add(sig(u.forms.awoken));
    },
  };
  for (const u of mvpPool(rows).units) {
    t.sleeping.add(sig(u.forms.sleeping));
    t.awoken.add(sig(u.forms.awoken));
  }
  return t;
}

const isWhen = (k: unknown): k is WhenKey => typeof k === "string" && Object.hasOwn(WHEN, k);
const isWho = (k: unknown): k is WhoKey => typeof k === "string" && Object.hasOwn(WHO, k);
const isDoes = (d: unknown): d is string => typeof d === "string" && d.length > 0 && d.length <= 60;
const isDoesList = (l: unknown): l is string[] => Array.isArray(l) && l.length <= 3 && l.every(isDoes);

/** The candidate row a reading of `archetype` makes, numbers as placeholders. */
export function rowOf(archetype: IdeaArchetype, r: Pick<IdeaReading, "when" | "who" | "does" | "awoken">): Row {
  return { name: archetype.name, emoji: archetype.emoji, ...PLACEHOLDER_STATS, archetype: archetype.line, when: r.when, who: r.who, does: r.does, awoken: r.awoken };
}

/** Checks a drafted reading of `archetype` against what `taken` holds: the
 * words (WHEN, WHO, the Does words), formProblems, awokenKeeps, a sleeping
 * and an Awoken shape no unit has (R1, R2) and no new listen → emit loop
 * (R4). The reading, rendered by formText, when it passes. */
export function checkReading(d: ReadingDraft, archetype: IdeaArchetype, taken: TakenShapes): { reading: IdeaReading; row: Row } | { problems: string[] } {
  const bad = (...problems: string[]) => ({ problems });
  const a = (d?.awoken ?? {}) as ReadingDraft["awoken"];
  if (!isWhen(d?.when)) return bad(`When "${String(d?.when)}" is not one of ${Object.keys(WHEN).join(", ")}`);
  if (!isWho(d.who)) return bad(`Who "${String(d.who)}" is not one of ${Object.keys(WHO).join(", ")}`);
  if (!isDoes(d.does)) return bad("Does is missing");
  const awoken: AwokenRow = {};
  if (a.who !== undefined) {
    if (!isWho(a.who)) return bad(`awoken Who "${String(a.who)}" is not one of ${Object.keys(WHO).join(", ")}`);
    awoken.who = a.who;
  }
  if (a.more !== undefined) {
    if (!isDoes(a.more)) return bad("awoken more must be one Does");
    awoken.more = a.more;
  }
  for (const k of ["before", "add"] as const) {
    if (a[k] === undefined) continue;
    if (!isDoesList(a[k])) return bad(`awoken ${k} must be a list of at most 3 Does`);
    if (a[k]!.length) awoken[k] = [...a[k]!];
  }
  if (a.also !== undefined) {
    if (!isWho(a.also?.who) || !isDoes(a.also.does)) return bad("awoken also must be { who, does } in the game's words");
    awoken.also = { who: a.also.who, does: a.also.does };
  }
  const row = rowOf(archetype, { when: d.when, who: d.who, does: d.does, awoken });
  let pool: ReturnType<typeof mvpPool>;
  try {
    pool = mvpPool([row]);
  } catch (err) {
    return bad(`a Does word the game lacks: ${(err as Error).message}`);
  }
  const unit = pool.units[0]!;
  const out = formProblems(unit, pool);
  const keeps = awokenKeeps(unit.forms.sleeping, unit.forms.awoken);
  if (keeps) out.push(`the awoken form ${keeps}`);
  const s = sig(unit.forms.sleeping);
  const w = sig(unit.forms.awoken);
  if (taken.sleeping.has(s)) out.push(`the sleeping shape "${s}" is taken by another unit`);
  if (taken.awoken.has(w)) out.push(`the awoken shape "${w}" is taken by another unit`);
  if (out.length) return { problems: out };
  const loops = linkLoops(mvpPool([...taken.rows, row]).units).filter((l) => !taken.loops.has(l));
  if (loops.length) return bad(`it makes a loop only the chain cap stops: ${loops[0]}`);
  const text = { sleeping: formText(unit.forms.sleeping, pool.abilities), awoken: formText(unit.forms.awoken, pool.abilities) };
  return { reading: { when: row.when, who: row.who, does: row.does, awoken, text }, row };
}

/** M3-4: whether a reading's `row` has `current`'s shape in both forms (a new
 * version must change When, Who or Does somewhere; numbers are tuned later). */
export function sameShape(row: Row, current: Row): boolean {
  const a = mvpPool([row]).units[0];
  const b = mvpPool([current]).units[0];
  return sig(a!.forms.sleeping) === sig(b!.forms.sleeping) && sig(a!.forms.awoken) === sig(b!.forms.awoken);
}

/** M3-4: a unit's rule as formText renders both forms. */
export function ruleText(row: Row): { sleeping: string; awoken: string } {
  const pool = mvpPool([row]);
  const u = pool.units[0]!;
  return { sleeping: formText(u.forms.sleeping, pool.abilities), awoken: formText(u.forms.awoken, pool.abilities) };
}
