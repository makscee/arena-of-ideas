// The rules v2 pool (#1046): the eight units build into content that
// validates, and their lines fight through the MVP's own path.

import { describe, expect, it } from "vitest";
import { contentOf } from "../../server/src/mvp/content.js";
import { addCopy, contentFormProblems, lineUnitOf } from "./forms.js";
import { fightLines, toBattleDef } from "./fight.js";
import { MVP_RULES } from "./contract.js";
import { ROWS_V2 } from "./units-v2.js";
import { V2_NUMBERS } from "./units-v2-numbers.js";
import { traceLines } from "../words/trace.js";
import { formSegments, formText, sentenceSegments, sentenceText } from "./form-text.js";
import { ruStatusName, setKnownStatusNames } from "../status-names.js";

const content = contentOf(ROWS_V2);

describe("rules v2 pool", () => {
  it("a unit gains its awoken bonus (the tune's awoken knob) when it awakens, merged or bought awake", () => {
    const u = { ...content.units[0]!, base: { pwr: 1, hp: 4 }, awokenBonus: { pwr: 1, hp: 2 } };
    const c = { ...content, units: [u, ...content.units.slice(1)] };
    const g = MVP_RULES.copyGrowth;
    const n = MVP_RULES.copiesToAwaken;
    expect(lineUnitOf(u, "a", n - 1).stats).toEqual({ pwr: 1 + (n - 2) * g.pwr, hp: 4 + (n - 2) * g.hp });
    expect(lineUnitOf(u, "a", n).stats).toEqual({ pwr: 2 + (n - 1) * g.pwr, hp: 6 + (n - 1) * g.hp });
    expect(addCopy(lineUnitOf(u, "a", n - 1), c).stats).toEqual(lineUnitOf(u, "a", n).stats);
  });

  it("the eight §4 units build into content with no form problems", () => {
    expect(content.units.map((u) => u.name).slice(0, 8)).toEqual(["Hourglass", "Mirror", "Candle", "Gambler", "Nautilus", "Magnet", "Raven", "Seed"]);
    expect(content.units).toHaveLength(40);
    expect(new Set(content.units.map((u) => u.tier))).toEqual(new Set([1, 2, 3, 4]));
    expect(contentFormProblems(content)).toEqual([]);
    expect(Object.keys(content.statuses)).toEqual(expect.arrayContaining(["Grains", "Sands", "Memory", "Memories", "Shield", "Strength"]));
  });

  it("numbers stay on the old pool's scale (#1047): sleeping HP 4-12, PWR at most 5, awoken at most +4 HP / +1 PWR", () => {
    for (const u of content.units) {
      expect(u.base.hp, u.name).toBeGreaterThanOrEqual(4);
      expect(u.base.hp, u.name).toBeLessThanOrEqual(12);
      expect(u.base.pwr, u.name).toBeLessThanOrEqual(5);
      expect(u.awokenBonus?.hp ?? 0, u.name).toBeLessThanOrEqual(4);
      expect(u.awokenBonus?.pwr ?? 0, u.name).toBeLessThanOrEqual(1);
    }
  });

  it("every card sentence reads like a person wrote it (#1047): its own, EN and RU, never starting with a bare number or a status name", () => {
    const statusNames = Object.values(content.statuses).flatMap((d) => [d.name, ...(d.names ? [d.names.en, d.names.ru.name] : [])]);
    for (const u of content.units) {
      for (const f of ["sleeping", "awoken"] as const) {
        const form = u.forms[f];
        expect(form.say, `${u.name} ${f}`).toBeDefined();
        for (const lang of ["en", "ru"] as const) {
          const said = sentenceText(form, content.abilities, lang, content.statuses);
          expect(said, `${u.name} ${f} ${lang}`).toBe(form.say![lang]);
          for (const sentence of said.split(/(?<=[.;:])\s+/)) {
            expect(sentence, `${u.name} ${f} ${lang}: "${sentence}"`).not.toMatch(/^\d/);
            for (const n of statusNames) expect(sentence.startsWith(n), `${u.name} ${f} ${lang}: "${sentence}" starts with ${n}`).toBe(false);
          }
        }
      }
      // the awoken sentence says what awakening changes
      expect(u.forms.awoken.say!.en, u.name).not.toBe(u.forms.sleeping.say!.en);
      expect(u.forms.awoken.say!.ru, u.name).not.toBe(u.forms.sleeping.say!.ru);
    }
  });

  // Maks, 10-14: a card says the chance, never a coin; the battle log may keep the flavour.
  // capital "Heads" stays allowed: Hydra's Heads are units
  const COIN = { en: /\b(heads|tails|Tails|flips?|Flips?|flipped|coins?|Coins?)\b/, ru: /(орёл|орла|орле|решк|монет)/i };
  it("no card sentence or archetype line names a coin: heads/tails/flip/coin, орёл/решка/монета (#1047)", () => {
    for (const u of content.units) {
      expect(u.archetype, u.name).not.toMatch(COIN.en);
      for (const f of ["sleeping", "awoken"] as const)
        for (const lang of ["en", "ru"] as const) {
          for (const said of [u.forms[f].say![lang], formText(u.forms[f], content.abilities, lang, content.statuses)]) expect(said, `${u.name} ${f} ${lang}`).not.toMatch(COIN[lang]);
        }
    }
    expect(content.units.find((u) => u.name === "Gambler")!.forms.sleeping.say!.en).toContain("50% chance");
    expect(content.units.find((u) => u.name === "Fortune Teller")!.forms.sleeping.say!.ru).toContain("с вероятностью 50%");
  });

  it("every period says its turns, so no rule reads two ways: no 'every second/third turn' or 'every Nth' on a card (#1047)", () => {
    const vague = { en: /\bevery (second|third|\d+(st|nd|rd|th))\b|\bunhurt\b/i, ru: /кажд(ый|ое|ую) \d+-[йм]|кажд(ый|ое) (второй|третий)|не ранен/i };
    for (const u of content.units)
      for (const f of ["sleeping", "awoken"] as const)
        for (const lang of ["en", "ru"] as const) {
          for (const said of [u.forms[f].say![lang], formText(u.forms[f], content.abilities, lang, content.statuses)]) expect(said, `${u.name} ${f} ${lang}`).not.toMatch(vague[lang]);
        }
  });

  it("the card sentence keeps the word chain's glossary terms it names word for word (Drummer's Strength)", () => {
    const drummer = content.units.find((u) => u.name === "Drummer")!;
    const segs = sentenceSegments(drummer.forms.sleeping, content.abilities, "en", content.statuses);
    expect(segs.map((s) => s.text).join("")).toBe("Each time an ally strikes, it gains 1 beat; at a turn end with 3+ beats, it spends 3 and every ally behind it gains 1 Strength.");
    expect(segs.some((s) => s.term && s.text.includes("Strength"))).toBe(true);
  });

  it("every unit of the batch fights, sleeping and awoken, and says both forms in EN and RU", () => {
    const p = { id: "bot", name: "bot", bot: true };
    for (const [i, u] of content.units.entries()) {
      const mate = content.units[(i + 3) % content.units.length]!;
      const foes = [content.units[(i + 7) % content.units.length]!, content.units[(i + 11) % content.units.length]!, content.units[(i + 13) % content.units.length]!];
      for (const copies of [1, 3]) {
        const a = [lineUnitOf(u, "a1", copies), lineUnitOf(mate, "a2")];
        const b = foes.map((f, j) => lineUnitOf(f, `b${j}`));
        const rec = fightLines({ player: p, line: a }, { player: p, line: b }, { battleId: `${u.id}-${copies}`, seed: 3, kind: "round", round: 1, runId: null, at: "2026-10-13T00:00:00.000Z", content, rules: MVP_RULES });
        expect(rec.log.length).toBeGreaterThan(2);
      }
      for (const lang of ["en", "ru"] as const) for (const form of ["sleeping", "awoken"] as const) {
        const text = formSegments(u.forms[form], content.abilities, lang).map((s) => s.text).join("");
        expect(text, `${u.name} ${form} ${lang}`).toMatch(/\S/);
        expect(text, `${u.name} ${form} ${lang}`).not.toMatch(/undefined|\[object/);
      }
    }
  });

  it("a v2 line fights the MVP way: Gambler flips, Raven starts with its Memory", () => {
    const unit = (name: string, copies = 1, uid = name) => lineUnitOf(content.units.find((u) => u.name === name)!, uid, copies);
    const a = [unit("Gambler"), unit("Raven", 3)];
    const b = [unit("Nautilus"), unit("Hourglass")];
    expect(toBattleDef(a[1]!).statuses).toEqual([{ status: "Memories", stacks: 1 }]);
    const p = { id: "bot", name: "bot", bot: true };
    const rec = fightLines({ player: p, line: a }, { player: p, line: b }, { battleId: "x", seed: 1, kind: "round", round: 1, runId: null, at: "2026-10-13T00:00:00.000Z", content, rules: MVP_RULES });
    const lines = traceLines({ teamA: a.map(toBattleDef), teamB: b.map(toBattleDef), seed: 1, statuses: content.statuses, abilities: content.abilities }, rec.log);
    expect(lines.some((l) => /^Gambler flips: (heads|tails)$/.test(l))).toBe(true);
  });

  // Batch 1's gate (#1046, reopened 10-14): the card says the whole unit, in
  // Russian with no English left in it.
  const say = (name: string, form: "sleeping" | "awoken", lang: "en" | "ru") =>
    formText(content.units.find((u) => u.name === name)!.forms[form], content.abilities, lang, content.statuses);

  it("every RU sentence of all 20 units, both forms, is fully Russian", () => {
    for (const u of content.units) for (const form of ["sleeping", "awoken"] as const) {
      const ru = say(u.name, form, "ru");
      expect(ru.match(/[A-Za-z]+/g) ?? [], `${u.name} ${form}: ${ru}`).toEqual([]);
    }
    expect(say("Hourglass", "sleeping", "ru")).toMatch(/^Конец хода: получить 1 Песчинку\./);
    expect(say("Seed", "sleeping", "ru")).toContain("превратиться в Дерево (0/8)");
    expect(say("Chrysalis", "sleeping", "ru")).toContain("превратиться в Бабочку");
    expect(say("Chrysalis", "awoken", "ru")).toContain("превратиться в Монарха");
    expect(say("Raven", "sleeping", "ru")).toContain("запомнить его правило");
    expect(say("Nautilus", "sleeping", "ru")).toContain("получить 1 Щит");
  });

  it("a unit's sentence says the rules on its counters, the statuses it starts with and the units it becomes", () => {
    expect(say("Hourglass", "sleeping", "en")).toBe("Turn end: gain 1 Grains. Turn end, if it has 3+ Grains: the ally behind acts again, and spend 3 Grains.");
    expect(say("Hourglass", "sleeping", "ru")).toBe(
      "Конец хода: получить 1 Песчинку. Конец хода, если у него 3 Песчинки или больше: заставить союзника позади действовать ещё раз, и потратить 3 Песчинки.",
    );
    expect(say("Drummer", "sleeping", "en")).toBe("Ally strikes: gain 1 Beats. Turn end, if it has 3+ Beats: give every ally behind 1 Strength, and spend 3 Beats.");
    expect(say("Raven", "sleeping", "en")).toBe("Ally dies: remember the rule of that unit. Turn start: perform the oldest remembered rule once and forget it.");
    expect(say("Raven", "awoken", "en")).toMatch(/Turn start: perform every remembered rule\.$/);
    expect(say("Seed", "awoken", "en")).toBe(`Battle start: in 2 turns: become Tree (0/${8 + V2_NUMBERS["Seed"]!.awoken!.hp}). Tree: Turn start: give the ally behind 1 Strength, and give its neighbours 1 Shield.`);
    expect(say("Chrysalis", "sleeping", "en")).toBe("Turn start, on turn 3: become Butterfly (2/6). Butterfly: Turn start: PWR damage to all enemies.");
    // every form that gives or starts with a status carrying a rule, or
    // becomes a unit with a rule, says more than its own clause
    for (const u of content.units) for (const form of ["sleeping", "awoken"] as const) {
      const f = u.forms[form];
      const effects = [...f.does, ...(f.also ?? []).flatMap((c) => c.does)].flatMap((id) => content.abilities[id]?.effects ?? []);
      const inner = effects.flatMap((e) => (e.kind === "delay" ? [e.effect] : [e]));
      const ruled = (id: string) => (content.statuses[id]?.abilities.length ?? 0) > 0 && content.statuses[id]!.counter === true;
      for (const e of inner) if (e.kind === "transform" && (e.into.abilities?.length ?? 0) > 0) expect(say(u.name, form, "en"), `${u.name} ${form}`).toContain(` ${e.into.name}: `);
      const holds = inner.some((e) => e.kind === "applyStatus" && ruled(e.status)) || (f.statuses ?? []).some((s) => ruled(s.status));
      if (!holds) continue;
      const bare = formText({ ...f, statuses: [] }, content.abilities, "en");
      expect(say(u.name, form, "en").length, `${u.name} ${form}`).toBeGreaterThan(bare.length);
    }
  });

  it("a battle's status chip and log row say a coined status's RU name once the pack is loaded", () => {
    expect(ruStatusName("Grains")).toBe("Grains");
    setKnownStatusNames(content.statuses);
    try {
      expect(["Grains", "Sands", "Beats", "Drums", "Memory", "Memories"].map(ruStatusName)).toEqual(["Песчинки", "Пески", "Такты", "Барабаны", "Память", "Воспоминания"]);
    } finally {
      setKnownStatusNames(undefined);
    }
  });
});
