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
import { captionOf, setTraceLang } from "./trace.js";

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
    expect(content.units).toHaveLength(60);
    expect(new Set(content.units.map((u) => u.tier))).toEqual(new Set([1, 2, 3, 4]));
    expect(contentFormProblems(content)).toEqual([]);
    expect(Object.keys(content.statuses)).toEqual(expect.arrayContaining(["Charge", "Shield", "Strength", "Freeze", "Blessing"]));
    // #1063: one shared counter, none of the old private ones
    for (const old of ["Grains", "Sands", "Beats", "Drums", "Mana", "Arcana", "Memory", "Memories", "Chime"]) expect(content.statuses[old], old).toBeUndefined();
  });

  it("numbers stay on the old pool's scale (#1047): sleeping HP 2-12, PWR at most 6, awoken at most +4 HP / +1 PWR", () => {
    for (const u of content.units) {
      expect(u.base.hp, u.name).toBeGreaterThanOrEqual(2);
      expect(u.base.hp, u.name).toBeLessThanOrEqual(12);
      expect(u.base.pwr, u.name).toBeLessThanOrEqual(6);
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
    expect(content.units.find((u) => u.name === "Gambler")!.forms.sleeping.say!.ru).toContain("с вероятностью 50%");
  });
  it("no log row or caption names a coin either (#1063): Gambler's says 50% → double / nothing", () => {
    const unit = (name: string, copies = 1, uid = name) => lineUnitOf(content.units.find((u) => u.name === name)!, uid, copies);
    const p = { id: "bot", name: "bot", bot: true };
    const a = [unit("Gambler"), unit("Gambler", 3, "G3")];
    const b = [unit("Nautilus"), unit("Fortune Teller")];
    const rec = fightLines({ player: p, line: a }, { player: p, line: b }, { battleId: "x", seed: 1, kind: "round", round: 1, runId: null, at: "2026-10-13T00:00:00.000Z", content, rules: MVP_RULES });
    const input = { teamA: a.map(toBattleDef), teamB: b.map(toBattleDef), seed: 1, statuses: content.statuses, abilities: content.abilities };
    const lines = traceLines(input, rec.log);
    expect(lines.some((l) => /^Gambler: 50% → (double|nothing)$/.test(l))).toBe(true);
    for (const l of lines) expect(l).not.toMatch(COIN.en);
    for (const lang of ["en", "ru"] as const) {
      setTraceLang(lang);
      try {
        const rows = rec.log.filter((e) => e.type === "Flipped").map((e) => captionOf(rec.log, e.id));
        expect(rows.length, lang).toBeGreaterThan(0);
        // Fortune Teller still rolls until its rework (#1063, next attempt); then only Gambler flips
        for (const r of rows) expect(r, lang).toMatch(lang === "en" ? /^(Gambler|Fortune Teller): 50% → (double|nothing)$/ : /^(Gambler|Fortune Teller): 50% → (двойной урон|ничего)$/);
        for (const e of rec.log) expect(captionOf(rec.log, e.id), `${lang} ${e.type}`).not.toMatch(COIN[lang]);
      } finally {
        setTraceLang("en");
      }
    }
  });

  it("every period says its turns, so no rule reads two ways: no 'every second/third turn' or 'every Nth' on a card (#1047)", () => {
    const vague = { en: /\bevery (second|third|\d+(st|nd|rd|th))\b|\bunhurt\b/i, ru: /кажд(ый|ое|ую) \d+-[йм]|кажд(ый|ое) (второй|третий)|не ранен/i };
    for (const u of content.units)
      for (const f of ["sleeping", "awoken"] as const)
        for (const lang of ["en", "ru"] as const) {
          for (const said of [u.forms[f].say![lang], formText(u.forms[f], content.abilities, lang, content.statuses)]) expect(said, `${u.name} ${f} ${lang}`).not.toMatch(vague[lang]);
        }
  });

  it("the card sentence keeps the word chain's glossary terms it names word for word (Vulture's Strength, Drummer's Charge)", () => {
    const vulture = content.units.find((u) => u.name === "Vulture")!;
    const segs = sentenceSegments(vulture.forms.sleeping, content.abilities, "en", content.statuses);
    expect(segs.map((s) => s.text).join("")).toBe("When any unit falls, ally or enemy, it gains 1 Strength.");
    expect(segs.some((s) => s.term && s.text.includes("Strength"))).toBe(true);
    const drummer = content.units.find((u) => u.name === "Drummer")!;
    const dsegs = sentenceSegments(drummer.forms.sleeping, content.abilities, "en", content.statuses);
    expect(dsegs.map((s) => s.text).join("")).toBe("When an ally strikes, it gains 1 Charge. At 3 Charge, it spends them and every ally behind it gains 1 Strength.");
    expect(dsegs.some((s) => s.term === "status:Charge" && s.text === "Charge")).toBe(true);
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

  it("a v2 line fights the MVP way: Gambler flips, Raven carries no status", () => {
    const unit = (name: string, copies = 1, uid = name) => lineUnitOf(content.units.find((u) => u.name === name)!, uid, copies);
    const a = [unit("Gambler"), unit("Raven", 3)];
    const b = [unit("Nautilus"), unit("Hourglass")];
    expect(toBattleDef(a[1]!).statuses ?? []).toEqual([]);
    const p = { id: "bot", name: "bot", bot: true };
    const rec = fightLines({ player: p, line: a }, { player: p, line: b }, { battleId: "x", seed: 1, kind: "round", round: 1, runId: null, at: "2026-10-13T00:00:00.000Z", content, rules: MVP_RULES });
    const lines = traceLines({ teamA: a.map(toBattleDef), teamB: b.map(toBattleDef), seed: 1, statuses: content.statuses, abilities: content.abilities }, rec.log);
    expect(lines.some((l) => /^Gambler: 50% → (double|nothing)$/.test(l))).toBe(true);
  });

  it("awoken Raven uses a fallen ally's rule once, then keeps it as a second rule with its own When (#1063)", () => {
    const unit = (name: string, copies = 1, uid = name) => lineUnitOf(content.units.find((u) => u.name === name)!, uid, copies);
    // Sniper (shoots the back enemy at each turn start) falls first; Raven then shoots too, every turn, as Raven.
    const a = [unit("Sniper"), unit("Raven", 3)];
    const b = [unit("Berserker", 3), unit("Berserker", 3, "B2"), unit("Berserker", 3, "B3")];
    const p = { id: "bot", name: "bot", bot: true };
    const rec = fightLines({ player: p, line: a }, { player: p, line: b }, { battleId: "x", seed: 1, kind: "round", round: 1, runId: null, at: "2026-10-14T00:00:00.000Z", content, rules: MVP_RULES });
    const log = rec.log as Array<{ type: string; op?: string; unit?: string; other?: string; turn: number; source: "kernel" | { unit: string; kept?: number } }>;
    const is = (id: string | undefined, name: string) => id?.endsWith(`:${name}`) ?? false;
    const death = log.find((e) => e.type === "Death" && is(e.unit, "Sniper"));
    expect(death).toBeDefined();
    const mods = log.filter((e) => e.type === "RuleMod" && is(e.unit, "Raven") && is(e.other, "Sniper")).map((e) => e.op);
    expect(mods).toEqual(["remember", "performRemembered", "keepRule"]);
    // The kept rule fires on later turns, sourced to Raven's kept slot, and the memory list stays empty (no Memories status).
    const keptHits = log.filter((e) => e.type === "Hurt" && e.source !== "kernel" && is(e.source.unit, "Raven") && e.source.kept === 0);
    expect(keptHits.length).toBeGreaterThan(0);
    expect(keptHits.every((e) => e.turn > death!.turn)).toBe(true);
    expect(log.some((e) => e.type === "StatusApplied" && (e as { status?: string }).status === "Memories")).toBe(false);
    const lines = traceLines({ teamA: a.map(toBattleDef), teamB: b.map(toBattleDef), seed: 1, statuses: content.statuses, abilities: content.abilities }, rec.log);
    expect(lines.some((l) => /^Raven keeps the rule of Sniper$/.test(l))).toBe(true);
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
    expect(say("Hourglass", "sleeping", "ru")).toMatch(/^Конец хода: получить 1 Заряд/);
    expect(say("Chrysalis", "sleeping", "ru")).toContain("превратиться в Бабочку");
    expect(say("Chrysalis", "awoken", "ru")).toContain("превратиться в Монарха");
    expect(say("Raven", "sleeping", "ru")).toContain("запомнить его правило");
    expect(say("Frostling", "sleeping", "ru")).toContain("1 Заморозку");
    expect(say("Nautilus", "sleeping", "ru")).toContain("получить 1 Щит");
  });

  it("a unit's sentence says the rules on its counters, the statuses it starts with and the units it becomes", () => {
    // #1063: the Charge units gain in their Does and spend in their "and" clauses, on one shared counter
    expect(say("Hourglass", "sleeping", "en")).toMatch(/^Turn end: gain 1 Charge, and at 3 Charge, the ally behind acts again, and at 3 Charge, it spends them\.$/);
    expect(say("Hourglass", "sleeping", "ru")).toMatch(/^Конец хода: получить 1 Заряд, и если у него 3 Заряда или больше, /);
    expect(say("Drummer", "sleeping", "en")).toMatch(/^Ally strikes: gain 1 Charge, and at 3 Charge, 1 Strength to every ally behind, /);
    expect(say("Raven", "sleeping", "en")).toBe("Ally dies, once per battle: remember the rule of that unit, then perform the oldest remembered rule once and forget it.");
    expect(say("Raven", "awoken", "en")).toMatch(/, then keep the rule of that unit as a second rule of its own\.$/);
    expect(say("Chrysalis", "sleeping", "en")).toBe("Turn start, on turn 3: become Butterfly (2/3). Butterfly: Turn start: PWR damage to all enemies.");
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
    expect(ruStatusName("Charge")).toBe("Charge");
    setKnownStatusNames(content.statuses);
    try {
      expect(["Charge"].map(ruStatusName)).toEqual(["Заряд"]);
    } finally {
      setKnownStatusNames(undefined);
    }
  });
  it("batch 3 (#1048): each new rule fires in a battle the way its sentence says", () => {
    const unit = (name: string, uid: string, copies = 1) => lineUnitOf(content.units.find((u) => u.name === name)!, uid, copies);
    const p = { id: "bot", name: "bot", bot: true };
    const cases: [string, string[], string[], RegExp][] = [
      ["Plague Rat", ["Plague Rat", "Sniper"], ["Berserker", "Leech"], /^Plague Rat → all enemies \(Berserker, Leech\) → Poison 1$/],
      ["Martyr", ["Martyr", "Sniper"], ["Berserker", "Leech"], /^Martyr → all allies \(Sniper\) → Strength \d+$/],
      ["Ox", ["Ox"], ["Nautilus"], /^Ox's strike on Nautilus hits for \d+$/],
      ["Daredevil", ["Daredevil"], ["Seed", "Seed"], /^Daredevil's strike on Seed hits for \d+$/],
      ["Duelist", ["Duelist"], ["Berserker"], /^Duelist's strike on Berserker hits for \d+$/],
      ["Squire", ["Squire", "Paladin", "Lantern"], ["Nautilus"], /^Squire → self \(Squire\) → Strength 1$/],
      ["Dynamo", ["Dynamo"], ["Seed", "Seed"], /^Dynamo strikes again$/],
      ["Siphon", ["Siphon"], ["Leech", "Medic"], /^Siphon intercepts: the heal on Leech lands on Siphon$/],
      ["Comet", ["Comet"], ["Seed", "Seed"], /^Comet \(on turns 4, 8, 12…\) → all enemies \(Seed, Seed\) → 4 damage$/],
      ["Heckler", ["Heckler"], ["Leech", "Seed"], /^Heckler → it \(Leech\) → \d+ damage$/],
      ["Broodmother", ["Broodmother"], ["Nautilus", "Seed"], /^Broodmother \(on turns 2, 4, 6…\) → self \(Spiderling\)$/],
      ["Dragon", ["Dragon"], ["Nautilus", "Seed"], /^Dragon → all enemies \(Nautilus, Seed\) → \d+ damage/],
      ["Kraken", ["Kraken"], ["Leech", "Seed"], /^Kraken → self \(Kraken, Leech\) → Charge 1, 1 damage$/],
      ["Puppeteer", ["Seed", "Puppeteer"], ["Seed", "Seed"], /^Puppeteer intercepts: the strike on \w+ lands on Seed$/],
      ["Phoenix", ["Phoenix"], ["Berserker", "Leech"], /^Phoenix \(once per battle\) → self \(Phoenix\)$/],
      ["Volcano", ["Volcano", "Sniper"], ["Nautilus", "Seed"], /^Volcano \(on turns 2, 4, 6…\) → all enemies \(Nautilus, Seed, Volcano, Sniper\) → .*1 damage$/],
    ];
    for (const [name, A, B, want] of cases) {
      const a = A.map((n, i) => unit(n, `a${i}`));
      const b = B.map((n, i) => unit(n, `b${i}`));
      const rec = fightLines({ player: p, line: a }, { player: p, line: b }, { battleId: "x", seed: 2, kind: "round", round: 1, runId: null, at: "2026-10-14T00:00:00.000Z", content, rules: MVP_RULES });
      const lines = traceLines({ teamA: a.map(toBattleDef), teamB: b.map(toBattleDef), seed: 2, statuses: content.statuses, abilities: content.abilities }, rec.log);
      expect(lines.some((l) => want.test(l)), `${name}:\n${lines.join("\n")}`).toBe(true);
    }
  });
  it("batch 3 (#1048): each awoken upgrade fires too", () => {
    const p = { id: "bot", name: "bot", bot: true };
    const cases: [string, string[], string[], RegExp][] = [
      ["Puppeteer", ["Puppeteer"], ["Leech", "Seed"], /^Puppeteer \(on turns 2, 4, 6… and once per turn\) → self \(Puppeteer\) → Shield 1$/],
      ["Squire", ["Squire", "Paladin", "Lantern"], ["Nautilus"], /^Squire → self \(Squire, Paladin\) → Strength 1$/],
      ["Dynamo", ["Dynamo"], ["Seed", "Seed"], /^Dynamo → self \(Dynamo\) → Charge 1, Strength 1$/],
      ["Siphon", ["Siphon"], ["Leech", "Medic"], /^Siphon → self \(Siphon\) → Strength 1$/],
      ["Phoenix", ["Phoenix"], ["Berserker", "Leech"], /^Phoenix \(once per battle\) → .*→ \d+ damage$/],
      ["Volcano", ["Volcano", "Sniper"], ["Nautilus", "Seed"], /^Volcano \(on turns 2, 4, 6…\) → .* → .*1 damage, Shield 1$/],
      ["Witch Doctor", ["Hornet", "Frostling", "Witch Doctor"], ["Seed", "Seed", "Seed"], /^Witch Doctor → .*Shield 1$/],
      ["Heckler", ["Heckler"], ["Leech", "Seed"], /^Heckler → it \(Leech\) → \d+ damage, Curse 1$/],
      ["Comet", ["Comet"], ["Seed", "Seed"], /^Comet \(on turns 3, 6, 9…\) → all enemies \(Seed, Seed\) → 3 damage$/],
    ];
    for (const [name, A, B, want] of cases) {
      const a = A.map((n, i) => lineUnitOf(content.units.find((u) => u.name === n)!, `a${i}`, n === name ? 3 : 1));
      const b = B.map((n, i) => lineUnitOf(content.units.find((u) => u.name === n)!, `b${i}`));
      const rec = fightLines({ player: p, line: a }, { player: p, line: b }, { battleId: "x", seed: 2, kind: "round", round: 1, runId: null, at: "2026-10-14T00:00:00.000Z", content, rules: MVP_RULES });
      const lines = traceLines({ teamA: a.map(toBattleDef), teamB: b.map(toBattleDef), seed: 2, statuses: content.statuses, abilities: content.abilities }, rec.log);
      expect(lines.some((l) => want.test(l)), `${name} awoken:\n${lines.join("\n")}`).toBe(true);
    }
  });
});
