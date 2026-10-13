// Status names in Russian: the shipped table, then the names a coined status
// carries (rules v2: a status a designer makes up says itself in EN and RU,
// #1045). Its own module, so the words in src/words/ can name a status without
// importing the renderer.

import type { StatusRegistry } from "./types.js";

/** A counted noun's three forms: 1 щит, 2 щита, 5 щитов. */
export type Forms = readonly [one: string, few: string, many: string];

/** A status in Russian: its name (nominative, as a label), the accusative
 * ("получает Силу") and its counted forms ("2 Силы"). */
export interface RuStatus {
  name: string;
  acc: string;
  count: Forms;
  /** Grammatical gender, when the singular's ending misleads (a masculine
   * noun in -ь). Otherwise read from the singular: -а/-я/-ь feminine,
   * -е/-о neuter, else masculine. */
  gender?: "m" | "f" | "n";
}

export const RU_STATUSES: Record<string, RuStatus> = {
  Strength: { name: "Сила", acc: "Силу", count: ["Сила", "Силы", "Сил"] },
  Vitality: { name: "Живучесть", acc: "Живучесть", count: ["Живучесть", "Живучести", "Живучести"] },
  Curse: { name: "Проклятие", acc: "Проклятие", count: ["Проклятие", "Проклятия", "Проклятий"] },
  Poison: { name: "Яд", acc: "Яд", count: ["Яд", "Яда", "Ядов"] },
  Shield: { name: "Щит", acc: "Щит", count: ["Щит", "Щита", "Щитов"] },
  Freeze: { name: "Заморозка", acc: "Заморозку", count: ["Заморозка", "Заморозки", "Заморозок"] },
  Blessing: { name: "Благословение", acc: "Благословение", count: ["Благословение", "Благословения", "Благословений"] },
};

// The registry whose coined names the sentence being rendered may use. Set
// for the length of one synchronous render by withStatusNames.
let scope: StatusRegistry | undefined;

/** Render with `statuses`' coined names in reach (nested renders keep the outer registry when given none). */
export function withStatusNames<T>(statuses: StatusRegistry | undefined, render: () => T): T {
  const outer = scope;
  scope = statuses ?? outer;
  try {
    return render();
  } finally {
    scope = outer;
  }
}

/** A status in Russian: the shipped table, then the status's own names. A
 * status with neither (player-made before v2) keeps its own name in every case. */
export function statusRu(s: string): RuStatus {
  return RU_STATUSES[s] ?? scope?.[s]?.names?.ru ?? known?.[s]?.names?.ru ?? { name: s, acc: s, count: [s, s, s] };
}

// The content pack's statuses, for the renders that run outside a
// withStatusNames scope (a battle's status chips and log rows, #1046).
let known: StatusRegistry | undefined;

/** Keep the content pack's coined status names in reach of every render. */
export function setKnownStatusNames(statuses: StatusRegistry | undefined): void {
  known = statuses;
}

/** A status's Russian name (its label), or its own name when it has none. */
export const ruStatusName = (s: string): string => statusRu(s).name;

/** Why a coined status can't be said, or undefined: it needs its EN name and
 * its RU name, accusative and three counted forms. */
export function namesMissing(def: StatusRegistry[string]): string | undefined {
  const n = def.names;
  const ok = (x: unknown): boolean => typeof x === "string" && x.trim() !== "";
  if (!n || !ok(n.en)) return "a coined status needs names.en";
  if (!n.ru || !ok(n.ru.name) || !ok(n.ru.acc) || !Array.isArray(n.ru.count) || n.ru.count.length !== 3 || !n.ru.count.every(ok))
    return "a coined status needs names.ru {name, acc, count: [one, few, many]}";
  return undefined;
}

const PLURAL = new Intl.PluralRules("ru");

/** A status counted in Russian: "3 Щита". */
export function ruStatusCount(n: number, s: string): string {
  const forms = statusRu(s).count;
  const cat = PLURAL.select(n);
  return `${n} ${cat === "one" ? forms[0] : cat === "few" ? forms[1] : forms[2]}`;
}

/** "равный" agreeing with the status ("Щит, равный …", "Сила, равная …",
 * "Проклятие, равное …"). */
export function ruEqualTo(s: string): string {
  const ru = statusRu(s);
  const g = ru.gender ?? (/[аяь]$/.test(ru.count[0]) ? "f" : /[еоё]$/.test(ru.count[0]) ? "n" : "m");
  return g === "f" ? "равная" : g === "n" ? "равное" : "равный";
}
