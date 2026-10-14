import { EXAMPLE_ABILITIES, example, plain, rule } from "./examples.js";
import type { ConditionWord } from "./types.js";
import type { ConditionV2 } from "../types.js";

type T = Extract<ConditionV2, { kind: "killedByHolder" }>;

/** True when the triggering Death was dealt by the holder's own strike: the
 * Death's cause is a Hurt, the Hurt's cause a Strike by the holder (§3.2;
 * #1063, Executioner: "the killer is this unit"). */
export const killedByHolder: ConditionWord<T> = {
  kind: "killedByHolder",
  class: "condition",
  check(ctx, _c, at) {
    const e = at.event;
    if (e.type !== "Death" || e.causedBy === null || e.causedBy === undefined) return false;
    const hurt = ctx.event(e.causedBy);
    if (!hurt || hurt.type !== "Hurt" || hurt.causedBy === null || hurt.causedBy === undefined) return false;
    const strike = ctx.event(hurt.causedBy);
    return strike !== undefined && strike.type === "Strike" && strike.striker === at.holder.id && strike.defender === e.unit;
  },
  en: () => "if its strike made the kill",
  ru: () => "если убила его атака",
  example: {
    input: { ...example([rule("Headsman", 8, 3, { on: "Death", unit: "enemy" }, { kind: "frontEnemy" }, "Again", { kind: "killedByHolder" })], [plain("Thug", 3, 1), plain("Brute", 9, 1)]), abilities: { ...EXAMPLE_ABILITIES, Again: { name: "Again", family: "Strike", effects: [{ kind: "strikeAgain" }] } } },
    trace: "Headsman strikes again",
  },
};
