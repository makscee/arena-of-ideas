import { example, plain, rule } from "./examples.js";
import type { ConditionWord } from "./types.js";
import { statusRu } from "../status-names.js";
import type { ConditionV2 } from "../types.js";

type T = Extract<ConditionV2, { kind: "targetHas" }>;

/** True when the event's target (a Strike's defender, the unit a Hurt or a
 * status lands on) carries at least one of the named statuses (§3.2; #1063,
 * Bully). */
export const targetHas: ConditionWord<T> = {
  kind: "targetHas",
  class: "condition",
  check(ctx, c, at) {
    const e = at.event;
    const id = e.type === "Strike" ? e.defender : "unit" in e && typeof e.unit === "string" ? e.unit : undefined;
    const u = id !== undefined ? ctx.unit(id) : undefined;
    return u !== undefined && c.statuses.some((s) => ctx.stacks(u, s) > 0);
  },
  en: (c) => `if the target has ${listEn(c.statuses)}`,
  ru: (c) => `если у цели есть ${listRu(c.statuses)}`,
  example: {
    input: example([rule("Bully", 6, 2, { on: "Strike", striker: "holder" }, { kind: "holder" }, "Empower", { kind: "targetHas", statuses: ["Strength"] })], [rule("Brute", 9, 1, { on: "BattleStart" }, { kind: "holder" }, "Empower")]),
    trace: "Bully (if the target has Strength) → self (Bully) → Strength 1",
  },
};

const listEn = (xs: string[]): string => (xs.length <= 1 ? xs.join("") : `${xs.slice(0, -1).join(", ")} or ${xs[xs.length - 1]}`);
const listRu = (xs: string[]): string => {
  const names = xs.map((s) => statusRu(s).name);
  return names.length <= 1 ? names.join("") : `${names.slice(0, -1).join(", ")} или ${names[names.length - 1]}`;
};
