// The word registry (#1043 §3, §7): every v2 primitive, by kind. Adding a word
// is one file here plus its entry below; the registry test runs each word's
// example against its golden in goldens/.

import { alliesAlive } from "./alliesAlive.js";
import { alliesBehind } from "./alliesBehind.js";
import { allyAhead } from "./allyAhead.js";
import { allyBehind } from "./allyBehind.js";
import { and } from "./and.js";
import { enemiesAlive } from "./enemiesAlive.js";
import { eventTarget } from "./eventTarget.js";
import { flip } from "./flip.js";
import { lastEnemy } from "./lastEnemy.js";
import { neighbours } from "./neighbours.js";
import { not } from "./not.js";
import { onceHolder } from "./onceHolder.js";
import { oncePerTurn } from "./oncePerTurn.js";
import { randomAlly } from "./randomAlly.js";
import { stacksAtLeast } from "./stacksAtLeast.js";
import { turnAtLeast } from "./turnAtLeast.js";
import { turnEvery } from "./turnEvery.js";
import { turnIs } from "./turnIs.js";
import { count } from "./count.js";
import { half } from "./half.js";
import { missingHp } from "./missingHp.js";
import { plus } from "./plus.js";
import { stacksOf } from "./stacksOf.js";
import { statOf } from "./statOf.js";
import { twice } from "./twice.js";
import type { AmountWord, ConditionWord, SelectorWord } from "./types.js";
import type { AmountV2, ConditionV2, PositionSelector } from "../types.js";

export type { AmountAt, AmountWord, CondAt, ConditionWord, SelectorWord, WordCtx, WordUnit } from "./types.js";

export const SELECTOR_WORDS: Record<PositionSelector["kind"], SelectorWord> = {
  allyBehind, allyAhead, neighbours, alliesBehind, lastEnemy, randomAlly, eventTarget,
};

export const CONDITION_WORDS: Record<ConditionV2["kind"], ConditionWord> = {
  and, not, turnIs, turnAtLeast, turnEvery, stacksAtLeast, alliesAlive, enemiesAlive, flip, onceHolder, oncePerTurn,
};

export const AMOUNT_WORDS: Record<AmountV2["kind"], AmountWord> = {
  stacksOf, count, statOf, missingHp, half, twice, plus,
};

export const amountWord = (kind: string): AmountWord | undefined => (AMOUNT_WORDS as Record<string, AmountWord>)[kind];
export const selectorWord = (kind: string): SelectorWord | undefined => (SELECTOR_WORDS as Record<string, SelectorWord>)[kind];
export const conditionWord = (kind: string): ConditionWord | undefined => (CONDITION_WORDS as Record<string, ConditionWord>)[kind];

/** Every registered word, for the registry test. */
export const ALL_WORDS: Array<SelectorWord | ConditionWord | AmountWord> = [...Object.values(SELECTOR_WORDS), ...Object.values(CONDITION_WORDS), ...Object.values(AMOUNT_WORDS)];
