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
