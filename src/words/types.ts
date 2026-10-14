// The word registry (rules v2, #1043 §3 and §7): one file per primitive,
// carrying how it applies, how it reads in English and Russian, and its trace
// line. A rule the renderer can't say is not a legal rule: validate.ts accepts
// a v2 kind only when its word is registered here.
//
// The engine hands a word a narrow view of the battle (WordCtx), so a word
// can't reach into kernel state it has no business with.

import type { BattleEvent, BattleInput, Side } from "../types.js";

/** A unit as a word sees it: the engine's own unit state, read-only. */
export interface WordUnit {
  readonly id: string;
  readonly name: string;
  readonly side: Side;
  readonly alive: boolean;
}

export interface WordCtx {
  /** The living line of a side, front to back. */
  line(side: Side): readonly WordUnit[];
  unit(id: string): WordUnit | undefined;
  /** One draw from the battle seed. */
  draw(): number;
  readonly turn: number;
  hp(u: WordUnit): number;
  stacks(u: WordUnit, status: string): number;
  stat(u: WordUnit, stat: import("../types.js").StatName): number;
  missingHp(u: WordUnit): number;
  /** A battle fact the kernel keeps, seen from `holder` (§3.3). */
  fact(fact: import("../types.js").BattleFact, holder: WordUnit): number;
  /** An event already in the log, by id: a word follows `causedBy` with it
   * (killedByHolder: Death → Hurt → Strike). */
  event(id: number): BattleEvent | undefined;
}

/** A sentence in both languages. RU selectors come in the two cases the
 * sentences need: dative ("союзнику позади") and accusative. */
export interface Says {
  en: string;
  ru: string;
}

/** Each word ships its own example: a small battle whose golden log
 * (src/words/goldens/<kind>.jsonl) shows the word at work, and the trace
 * line the battle must contain. */
export interface WordExample {
  input: BattleInput;
  /** A line `traceLines(log)` must produce for this battle. */
  trace: string;
}

export interface WordMeta {
  /** The primitive class of §3: 3.1 position … 3.6 timing. */
  class: "position" | "condition" | "counter" | "replacement" | "ruleMod" | "timing";
  example: WordExample;
}

export type EventOf = BattleEvent;

/** A position selector (§3.1): who a rule picks, by place in the line. */
export interface SelectorWord extends WordMeta {
  kind: string;
  /** Which side its picks are on, relative to the holder. */
  side?: "ally" | "enemy";
  select(ctx: WordCtx, holder: WordUnit, event: BattleEvent): WordUnit[];
  /** The noun phrase in a card sentence, the terse chip, and the trace
   * phrase ("the ally behind it"). */
  en: { phrase: string; chip: string; trace: string };
  ru: { dat: string; acc: string; chip: string; trace: string };
}

/** Where a condition is checked: the firing's holder and event, its identity
 * (for once-rules), and the marks and flips the check would make. The engine
 * commits marks and logs flips only once the whole condition is known. */
export interface CondAt {
  holder: WordUnit;
  event: BattleEvent;
  /** The reaction's identity, stable for the battle. */
  key: string;
  marks: string[];
  flips: boolean[];
  /** Whether a mark is already committed. */
  marked(mark: string): boolean;
  /** Check a nested condition (and / not). */
  check(c: import("../types.js").Condition): boolean;
}

/** A condition (§3.2). `C` is the condition's own shape. */
export interface ConditionWord<C = any> extends WordMeta {
  kind: string;
  check(ctx: WordCtx, c: C, at: CondAt): boolean;
  /** The clause in a card sentence: "on turn 3", "на 3-м ходу". */
  en(c: C, say: (c: import("../types.js").Condition) => string): string;
  ru(c: C, say: (c: import("../types.js").Condition) => string): string;
}

/** Where an amount is evaluated: the firing's holder, the effect's target
 * (absent in an interceptor) and the unit the triggering event is about. */
export interface AmountAt {
  holder: WordUnit;
  target?: WordUnit;
  eventUnit?: WordUnit;
  eval(a: import("../types.js").Amount): number;
}

/** A scaling amount (§3.3). `en`/`ru` read as the noun after "equal to" /
 * "равный" (Russian dative). */
export interface AmountWord<A = any> extends WordMeta {
  kind: string;
  value(ctx: WordCtx, a: A, at: AmountAt): number;
  en(a: A, say: (a: import("../types.js").Amount) => string): string;
  ru(a: A, say: (a: import("../types.js").Amount) => string): string;
}

/** What a word's sentence can ask the renderer to say. */
export interface Say {
  amount(a: import("../types.js").Amount): string;
  who(s: import("../types.js").Selector): string;
  /** A nested effect (delay's), in the sentence's language, against the same Who. */
  effect(e: import("../types.js").Effect): string;
}

/** Where an interceptor effect runs: the holder, the proposed event (the
 * word may change it) and the kernel moves a replacement may make. */
export interface InterceptAt {
  holder: WordUnit;
  draft: import("../types.js").EventBody;
  eval(a: import("../types.js").Amount): number;
  select(s: import("../types.js").Selector): WordUnit[];
  /** Move the proposed event onto `to`; false when it was already
   * redirected once, or has no unit to move. */
  redirect(to: WordUnit): boolean;
  /** After the cancelled Death: set the unit to `hp` and take `amount` from
   * its `stat`. */
  survive(unit: string, hp: number, stat: import("../types.js").StatName, amount: number): void;
  /** Once the intercepted status has landed: give `unit` the same status too
   * (reflectStatus). */
  alsoApply(unit: string, status: string, stacks: number): void;
}

/** An effect (§3.4–3.6). Interceptor words change or cancel the proposal. */
export interface EffectWord<E = any> extends WordMeta {
  kind: string;
  /** "either": a word that runs in both (branch). */
  context: "trigger" | "interceptor" | "either";
  intercept?(ctx: WordCtx, e: E, at: InterceptAt): "pass" | "cancel";
  /** The clause in a card sentence; `target` is the rule's Who already said. */
  en(e: E, say: Say, target: string): string;
  ru(e: E, say: Say, target: string): string;
}

/** "правило X" in Russian: a pronoun target goes before the noun ("его
 * правило"), a named one after it ("правило переднего врага"). */
export const ruRuleOf = (target: string): string => (/^(его|её|их)$/.test(target) ? `${target} правило` : `правило ${target}`);
