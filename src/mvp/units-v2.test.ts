// The rules v2 pool (#1046): the eight units build into content that
// validates, and their lines fight through the MVP's own path.

import { describe, expect, it } from "vitest";
import { contentOf } from "../../server/src/mvp/content.js";
import { contentFormProblems, lineUnitOf } from "./forms.js";
import { fightLines, toBattleDef } from "./fight.js";
import { MVP_RULES } from "./contract.js";
import { ROWS_V2 } from "./units-v2.js";
import { traceLines } from "../words/trace.js";
import { formSegments, formText } from "./form-text.js";

const content = contentOf(ROWS_V2);

describe("rules v2 pool", () => {
  it("the eight §4 units build into content with no form problems", () => {
    expect(content.units.map((u) => u.name).slice(0, 8)).toEqual(["Hourglass", "Mirror", "Candle", "Gambler", "Nautilus", "Magnet", "Raven", "Seed"]);
    expect(content.units).toHaveLength(20);
    expect(new Set(content.units.map((u) => u.tier))).toEqual(new Set([1, 2, 3, 4]));
    expect(contentFormProblems(content)).toEqual([]);
    expect(Object.keys(content.statuses)).toEqual(expect.arrayContaining(["Grains", "Sands", "Memory", "Memories", "Shield", "Strength"]));
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
    expect(say("Hourglass", "sleeping", "ru")).toMatch(/^Конец хода: 1 Песчинка себе\./);
    expect(say("Seed", "sleeping", "ru")).toContain("превратиться в Дерево (0/8)");
    expect(say("Chrysalis", "sleeping", "ru")).toContain("превратиться в Бабочку");
    expect(say("Chrysalis", "awoken", "ru")).toContain("превратиться в Монарха");
    expect(say("Raven", "sleeping", "ru")).toContain("запомнить его правило");
    expect(say("Nautilus", "sleeping", "ru")).toContain("Щит, равный ");
  });

  it("a unit's sentence says the rules on its counters, the statuses it starts with and the units it becomes", () => {
    expect(say("Hourglass", "sleeping", "en")).toBe("Turn end: 1 Grains to self. Turn end, if it has 3+ Grains: the ally behind acts again, and spend 3 Grains.");
    expect(say("Hourglass", "sleeping", "ru")).toBe(
      "Конец хода: 1 Песчинка себе. Конец хода, если у него 3 Песчинки или больше: заставить союзника позади действовать ещё раз, и потратить 3 Песчинки.",
    );
    expect(say("Drummer", "sleeping", "en")).toBe("Ally strikes: 1 Beats to self. Turn end, if it has 3+ Beats: 1 Strength to every ally behind, and spend 3 Beats.");
    expect(say("Raven", "sleeping", "en")).toBe("Ally dies: remember the rule of that unit. Turn start: perform a remembered rule once.");
    expect(say("Raven", "awoken", "en")).toMatch(/Turn start: perform every remembered rule\.$/);
    expect(say("Seed", "awoken", "en")).toBe("Battle start: in 2 turns: become Tree (0/8). Tree: Turn start: 1 Strength to the ally behind, and 1 Shield to its neighbours.");
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
});
