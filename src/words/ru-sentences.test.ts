// Rules v2 sentences in Russian (#1045, reopened by the night batch): every
// word nested under delay, a condition (and / not / flip) or a macro says
// itself in Russian, not in English; a rule living on a counter status reads
// on the unit's card; a coined status says its own Russian name. The lines are
// pinned in goldens/ru-nested.txt so a reviewer can read them.

import { describe, expect, it } from "vitest";
import { describeAbility } from "../describe.js";
import { formText } from "../mvp/form-text.js";
import { withStatusNames } from "../status-names.js";
import type { Ability, Condition, Effect, StatusRegistry } from "../types.js";
import { CONDITION_WORDS, EFFECT_WORDS } from "./index.js";
import { MACROS } from "./macros.js";
import { FIXTURE_ABILITIES, FIXTURE_STATUSES, Hourglass, Raven, RavenAwoken } from "./fixtures.js";

/** The first object of `kind` anywhere in a word's example battle. */
function find(x: unknown, kind: string): any {
  if (Array.isArray(x)) {
    for (const y of x) {
      const f = find(y, kind);
      if (f) return f;
    }
  } else if (x && typeof x === "object") {
    if ((x as { kind?: unknown }).kind === kind) return x;
    for (const y of Object.values(x)) {
      const f = find(y, kind);
      if (f) return f;
    }
  }
  return undefined;
}

/** Names a sentence may keep as they are: units (translated elsewhere, M4-4). */
function names(x: unknown, out: string[] = []): string[] {
  if (Array.isArray(x)) x.forEach((y) => names(y, out));
  else if (x && typeof x === "object") for (const [k, v] of Object.entries(x)) k === "name" && typeof v === "string" ? out.push(v) : names(v, out);
  return out;
}

/** The Latin words left in a Russian sentence once its names are taken out. */
function english(ru: string, keep: string[]): string[] {
  let s = ru;
  for (const n of keep) s = s.split(n).join("");
  return s.match(/[A-Za-z]{2,}/g) ?? [];
}

const ability = (effects: Effect[], condition?: Condition): Ability => ({
  whens: [{ kind: "trigger", on: { on: "TurnStart" } }],
  selectors: [{ kind: "frontEnemy" }],
  effects,
  ...(condition ? { condition } : {}),
});
const say = (ab: Ability) => withStatusNames(FIXTURE_STATUSES, () => ({ en: describeAbility(ab), ru: describeAbility(ab, { lang: "ru" }) }));

const V1_EFFECTS: Effect[] = [
  { kind: "damage", amount: { kind: "const", value: 3 } },
  { kind: "damage", amount: { kind: "stat", stat: "pwr", of: "holder" } },
  { kind: "heal", amount: { kind: "const", value: 2 } },
  { kind: "applyStatus", status: "Grains", stacks: { kind: "const", value: 1 } },
  { kind: "applyStatus", status: "Shield", stacks: { kind: "stacksOf", status: "Grains", of: "holder" } },
];
const nestedEffects = (): Array<[string, Effect]> => [
  ...V1_EFFECTS.map((e) => [e.kind, e] as [string, Effect]),
  ...Object.values(EFFECT_WORDS)
    .filter((w) => w.kind !== "delay")
    .map((w) => [w.kind, find(w.example.input, w.kind) as Effect] as [string, Effect]),
];

const lines: string[] = [];

describe("every word nested under delay says itself in Russian", () => {
  for (const [kind, e] of nestedEffects()) {
    it(kind, () => {
      expect(e, `${kind}'s example has the effect`).toBeDefined();
      const s = say(ability([{ kind: "delay", turns: 2, effect: e }]));
      lines.push(`delay ${kind}\n  en: ${s.en}\n  ru: ${s.ru}`);
      expect(s.ru).toMatch(/через 2 хода: /);
      expect(english(s.ru, names(e))).toEqual([]);
    });
  }
});

describe("every condition word under and / not / flip says itself in Russian", () => {
  for (const w of Object.values(CONDITION_WORDS)) {
    if (w.kind === "and" || w.kind === "not") continue;
    it(w.kind, () => {
      const c = find(w.example.input, w.kind) as Condition;
      expect(c, `${w.kind}'s example has the condition`).toBeDefined();
      const nested: Condition = { kind: "and", all: [{ kind: "flip", p: 0.5 }, { kind: "not", cond: c }] };
      const s = say(ability([{ kind: "delay", turns: 1, effect: { kind: "damage", amount: { kind: "const", value: 1 } } }], nested));
      lines.push(`and(flip, not ${w.kind})\n  en: ${s.en}\n  ru: ${s.ru}`);
      expect(english(s.ru, [])).toEqual([]);
    });
  }
});

describe("every macro says itself in Russian, whole and expanded", () => {
  for (const m of Object.values(MACROS)) {
    it(m.name, () => {
      const whole = say(ability([{ kind: "macro", name: m.name }]));
      const expanded = say(ability(m.effects));
      lines.push(`macro ${m.name}\n  en: ${whole.en}\n  ru: ${whole.ru}\n  expanded ru: ${expanded.ru}`);
      expect(english(whole.ru, [])).toEqual([]);
      expect(english(expanded.ru, [])).toEqual([]);
    });
  }
});

describe("a rule on a counter status reads on the unit's card", () => {
  const form = (u: typeof Hourglass) => ({ when: u.triggers ?? [], who: u.selectors ?? [], does: u.abilities ?? [] });
  it("Hourglass: the grain and, at 3 grains, the ally behind acts again", () => {
    const carries = (Hourglass.statuses ?? []).map((s) => s.status);
    expect(formText(form(Hourglass), FIXTURE_ABILITIES, "en", FIXTURE_STATUSES, carries)).toBe(
      "Turn end: gain 1 Grains. Turn end, if it has 3+ Grains: the ally behind acts again, and spend 3 Grains.",
    );
    expect(formText(form(Hourglass), FIXTURE_ABILITIES, "ru", FIXTURE_STATUSES, carries)).toBe(
      "Конец хода: получить 1 Песчинку. Конец хода, если у него 3 Песчинки или больше: заставить союзника позади действовать ещё раз, и потратить 3 Песчинки.",
    );
  });
  it("Raven: no memory list (#1063), the awoken form keeps the rule as a second rule", () => {
    expect(formText(form(Raven), FIXTURE_ABILITIES, "ru", FIXTURE_STATUSES)).toMatch(/^Смерть союзника: запомнить его правило, затем исполнить самое старое запомненное правило и забыть его\.$/);
    expect(formText(form(RavenAwoken), FIXTURE_ABILITIES, "ru", FIXTURE_STATUSES)).toMatch(/, затем оставить себе его правило как второе правило\.$/);
  });
  it("a card without the registry reads as before", () => {
    expect(formText(form(Hourglass), FIXTURE_ABILITIES, "en")).toBe("Turn end: 1 Grains to self.");
  });
});

describe("a coined status says its own Russian name", () => {
  const coined: StatusRegistry = { Wax: { name: "Wax", names: { en: "Wax", ru: { name: "Воск", acc: "Воск", count: ["Воск", "Воска", "Воска"] } }, counter: true, abilities: [] } };
  it("in an effect, a condition and an amount", () => {
    const ab = ability([{ kind: "applyStatus", status: "Wax", stacks: { kind: "const", value: 2 } }, { kind: "damage", amount: { kind: "stacksOf", status: "Wax", of: "holder" } }], { kind: "stacksAtLeast", status: "Wax", n: 5, of: "holder" });
    const ru = withStatusNames(coined, () => describeAbility(ab, { lang: "ru" }));
    expect(ru).toContain("2 Воска");
    expect(ru).toContain("5 Воска или больше");
    expect(ru).toContain("числу Воска у себя");
    expect(english(ru, [])).toEqual([]);
  });
});

describe("the nested sentences", () => {
  it("match the golden", async () => {
    await expect(lines.join("\n") + "\n").toMatchFileSnapshot("goldens/ru-nested.txt");
  });
});
