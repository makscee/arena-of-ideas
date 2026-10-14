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
import { actAgain } from "./actAgain.js";
import { copyRule } from "./copyRule.js";
import { delay } from "./delay.js";
import { summonWithRule } from "./summonWithRule.js";
import { transform } from "./transform.js";
import { performRemembered } from "./performRemembered.js";
import { preventDeath } from "./preventDeath.js";
import { remember } from "./remember.js";
import { swapRules } from "./swapRules.js";
import { redirect } from "./redirect.js";
import { replaceAmount } from "./replaceAmount.js";
import { branch } from "./branch.js";
import { strikeAgain } from "./strikeAgain.js";
import type { AmountWord, ConditionWord, EffectWord, SelectorWord } from "./types.js";
import { unhurt } from "./unhurt.js";
import { outnumbered } from "./outnumbered.js";
import { targetHas } from "./targetHas.js";
import { poisonedEnemies } from "./poisonedEnemies.js";
import { killedByHolder } from "./killedByHolder.js";
import type { AmountV2, ConditionV2, Effect, EffectV2, PositionSelector } from "../types.js";

export type { EffectWord, InterceptAt, Say, AmountAt, AmountWord, CondAt, ConditionWord, SelectorWord, WordCtx, WordUnit } from "./types.js";

export const SELECTOR_WORDS: Record<PositionSelector["kind"], SelectorWord> = {
  allyBehind, allyAhead, neighbours, alliesBehind, lastEnemy, randomAlly, eventTarget, poisonedEnemies,
};

export const CONDITION_WORDS: Record<ConditionV2["kind"], ConditionWord> = {
  and, not, turnIs, turnAtLeast, turnEvery, stacksAtLeast, alliesAlive, enemiesAlive, flip, onceHolder, oncePerTurn, unhurt, outnumbered, targetHas, killedByHolder,
};

export const AMOUNT_WORDS: Record<AmountV2["kind"], AmountWord> = {
  stacksOf, count, statOf, missingHp, half, twice, plus,
};

export const EFFECT_WORDS: Record<EffectV2["kind"], EffectWord> = {
  redirect, preventDeath, replaceAmount, copyRule, actAgain, remember, performRemembered, swapRules, delay, transform, summonWithRule, branch, strikeAgain,
};

export const effectWord = (kind: string): EffectWord | undefined => (EFFECT_WORDS as Record<string, EffectWord>)[kind];
export const isV2Effect = (e: Effect): e is EffectV2 => effectWord(e.kind) !== undefined;
export const amountWord = (kind: string): AmountWord | undefined => (AMOUNT_WORDS as Record<string, AmountWord>)[kind];
export const selectorWord = (kind: string): SelectorWord | undefined => (SELECTOR_WORDS as Record<string, SelectorWord>)[kind];
export const conditionWord = (kind: string): ConditionWord | undefined => (CONDITION_WORDS as Record<string, ConditionWord>)[kind];

/** Every registered word, for the registry test. */
export const ALL_WORDS: Array<SelectorWord | ConditionWord | AmountWord | EffectWord> = [...Object.values(SELECTOR_WORDS), ...Object.values(CONDITION_WORDS), ...Object.values(AMOUNT_WORDS), ...Object.values(EFFECT_WORDS)];
