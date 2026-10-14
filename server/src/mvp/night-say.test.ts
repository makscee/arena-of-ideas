// The fusion's written sentence (makscee/void-board#1122): the night designer
// writes one with the recipe, checked against the composer's text; the Codex,
// the claim sheet and the cards read it; the composer never says
// "otherwise, nothing".
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { Hourglass, Mirror, FIXTURE_ABILITIES, FIXTURE_STATUSES } from "../../../src/words/fixtures.js";
import { formText, sentenceText } from "../../../src/mvp/form-text.js";
import { forgeDesignOf } from "../../../src/mvp/forge-view.js";
import type { FusionDesign } from "../../../src/mvp/codex.js";
import type { UnitDef } from "../../../src/types.js";
import { type Candidate, type DesignDraft, type Designer, type Parent, type Say, sayIssues, writeSay } from "./night-batch.js";
import { designOf } from "./night-codex.js";
import { fakeDesigner, saySystem } from "./night-designer.js";

const parent = (def: UnitDef): Parent => ({ id: def.name, def, abilities: FIXTURE_ABILITIES, statuses: FIXTURE_STATUSES });
const draft: DesignDraft = { lean: "B", intent: "Charges when hit; at 3 shields everyone.", rule: {}, abilities: {} };

// Chambered Mender as the workshop stored it (#1122's example)
const mender = JSON.parse(readFileSync(join(import.meta.dirname, "fixtures/chambered-mender.json"), "utf8")) as FusionDesign;
const rule: Say = { en: formText(mender.recipe, mender.abilities!, undefined, mender.statuses), ru: formText(mender.recipe, mender.abilities!, "ru", mender.statuses) };
const good: Say = {
  en: "When it is hit, it gains 1 Charge. At 3 Charge, it spends them and every ally gains 2 Shield.",
  ru: "Когда его бьют, получает 1 Заряд. При 3 Зарядах тратит их, и все союзники получают 2 Щита.",
};

describe("the composer (#1122)", () => {
  it("never says 'otherwise, nothing', and the Charge check reads as on the units", () => {
    expect(rule.en).not.toMatch(/otherwise/);
    expect(rule.ru).not.toMatch(/иначе/);
    expect(rule.en).toBe("Hit: gain 1 Charge, and at 3 Charge, 2 Shield to all allies, and at 3 Charge, it spends them.");
  });
});

describe("sayIssues", () => {
  it("passes a written sentence that says the rule's numbers and statuses", () => {
    expect(sayIssues(good, rule)).toEqual([]);
  });
  it("refuses a number the rule doesn't have, a missing status, machine text and a missing language", () => {
    expect(sayIssues({ ...good, en: good.en.replace("2 Shield", "5 Shield") }, rule).join()).toMatch(/says 5/);
    expect(sayIssues({ ...good, en: "When it is hit, it gains 1 Charge. At 3, every ally gains 2 armour." }, rule).join()).toMatch(/must name it/);
    expect(sayIssues({ ...good, en: `${good.en} Otherwise, nothing.` }, rule).join()).toMatch(/otherwise/);
    expect(sayIssues({ ...good, en: "It casts ShellAllCharged at 3 Charge and 2 Shield." }, rule).join()).toMatch(/engine id/);
    expect(sayIssues({ en: good.en, ru: "" }, rule)).not.toEqual([]);
    expect(sayIssues({ en: good.en, ru: good.en }, rule).join()).toMatch(/isn't in Russian/);
  });
});

describe("writeSay", () => {
  const writer = (answers: (Say | Error)[]): Designer & { prompts: (string[] | undefined)[] } => {
    const prompts: (string[] | undefined)[] = [];
    return {
      ...fakeDesigner(),
      prompts,
      async say(_a, _b, _d, _rule, fix) {
        prompts.push(fix);
        const x = answers.shift()!;
        if (x instanceof Error) throw x;
        return x;
      },
    };
  };
  it("keeps a matching sentence", async () => {
    expect(await writeSay(writer([good]), parent(Hourglass), parent(Mirror), draft, rule)).toEqual(good);
  });
  it("retries once with the reasons, then gives up (the card reads the composer)", async () => {
    const w = writer([{ ...good, en: "It gains 9 Charge." }, good]);
    expect(await writeSay(w, parent(Hourglass), parent(Mirror), draft, rule)).toEqual(good);
    expect(w.prompts[1]!.join()).toMatch(/says 9/);
    const lines: string[] = [];
    expect(await writeSay(writer([new Error("timeout"), { ...good, ru: "" }]), parent(Hourglass), parent(Mirror), draft, rule, (l) => lines.push(l))).toBeUndefined();
    expect(lines.join()).toMatch(/no matching sentence/);
  });
  it("the writer's prompt carries the units' style and the Charge reading", () => {
    expect(saySystem()).toMatch(/At 3 Charge, it spends them/);
    expect(saySystem()).toMatch(/otherwise, nothing/);
  });
});

describe("the sentence reaches the Codex and the cards", () => {
  const c = { pair: "a+b", lean: "B", intent: "", name: { en: "Chambered Mender", ru: "Камерный целитель" }, flavour: { en: "", ru: "" }, text: rule, say: good, unit: { name: "Chambered Mender", base: { hp: 5, pwr: 2 }, triggers: mender.recipe.when, selectors: mender.recipe.who, abilities: mender.recipe.does, also: mender.recipe.also!.map((x) => ({ selectors: x.who, abilities: x.does })) }, abilities: mender.abilities!, statuses: mender.statuses!, check: null, state: "ready" } as Candidate;
  it("designOf stores it on the recipe; the Codex view and the card sentence read it", () => {
    const d = designOf(c);
    expect(d.recipe.say).toEqual(good);
    const view = forgeDesignOf(d, [undefined, undefined], { abilities: {}, statuses: {} });
    expect(view.rule).toEqual(good);
    expect(sentenceText(d.recipe, d.abilities!, undefined, d.statuses)).toBe(good.en);
    expect(sentenceText(d.recipe, d.abilities!, "ru", d.statuses)).toBe(good.ru);
  });
  it("without one, the Codex falls back to the composer", () => {
    const { say: _, ...bare } = c;
    const d = designOf(bare);
    expect(d.recipe.say).toBeUndefined();
    expect(forgeDesignOf(d, [undefined, undefined], { abilities: {}, statuses: {} }).rule.en).toBe(rule.en);
  });
});
