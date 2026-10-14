import { EXAMPLE_ABILITIES, example, plain } from "./examples.js";
import type { EffectWord, Say } from "./types.js";
import type { Condition, Effect, EffectV2, UnitDef } from "../types.js";
import { describeCondition, describeEffect } from "../describe.js";
import { conditionWord } from "./index.js";
import { ALL_OR_NOTHING } from "./all-or-nothing.js";

type E = Extract<EffectV2, { kind: "branch" }>;

const Gambler: UnitDef = {
  name: "Gambler",
  base: { hp: 9, pwr: 2 },
  triggers: [{ kind: "interceptor", on: { on: "Strike", striker: "holder" } }],
  selectors: [{ kind: "holder" }],
  abilities: ["AllOrNothing"],
};

export { ALL_OR_NOTHING };

/** If / else: check the condition (a flip logs its coin) and run one side.
 * Runs in either context; Gambler's "double damage or none" (§4). */
export const branch: EffectWord<E> = {
  kind: "branch",
  class: "condition",
  context: "either",
  // #1122: an empty side is never said ("otherwise, nothing"); a check on
  // the holder's own counter reads as the units do ("at 3 Charge, …")
  en: (e, say, target) => `${condEn(e.if)}, ${side(e.then, say, target, "en", e.if)}${e.else.length ? `; otherwise, ${side(e.else, say, target, "en")}` : ""}`,
  ru: (e, say, target) => `${condRu(e.if)}, ${side(e.then, say, target, "ru", e.if)}${e.else.length ? `; иначе ${side(e.else, say, target, "ru")}` : ""}`,
  example: {
    // seed 2: the first coin is tails, so the strike deals none
    input: { ...example([Gambler], [plain("Brute", 30, 1)], 2), abilities: { ...EXAMPLE_ABILITIES, AllOrNothing: { name: "AllOrNothing", family: "Strike", effects: [ALL_OR_NOTHING] } } },
    trace: "Gambler: 50% → nothing",
  },
};

function side(effects: Effect[], say: Say, target: string, lang: "en" | "ru", cond?: Condition): string {
  if (effects.length === 0) return lang === "en" ? "nothing" : "ничего";
  // "at 3 Charge, spend 3 Charge" → "at 3 Charge, it spends them"
  const only = effects.length === 1 ? effects[0] : undefined;
  if (cond?.kind === "stacksAtLeast" && cond.of === "holder" && only?.kind === "consumeStacks" && only.status === cond.status && only.stacks.kind === "const" && only.stacks.value === cond.n) {
    return lang === "en" ? "it spends them" : "тратит их";
  }
  // RU: through say, so a v2 word on a side (Gambler's replaceAmount) says itself in Russian
  return effects.map((x) => (lang === "en" ? describeEffect(x, target) : say.effect(x))).join(lang === "en" ? " and " : " и ");
}

function condEn(c: Condition): string {
  return c.kind === "stacksAtLeast" && c.of === "holder" ? `at ${c.n} ${c.status}` : describeCondition(c);
}

function condRu(c: Condition): string {
  const w = c.kind === "holderHpAtMost" ? undefined : conditionWord(c.kind);
  return w ? w.ru(c, condRu) : describeCondition(c);
}
