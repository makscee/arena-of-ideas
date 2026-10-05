import { describe, expect, it } from "vitest";
import { MVP_RULES, type MvpContent } from "./contract.js";
import { fightLines } from "./fight.js";
import { contentFormProblems, lineUnitOf } from "./forms.js";
import { ROWS, mvpPool } from "./units.js";

describe("MVP pool (slice 7)", () => {
  const pool = mvpPool();
  const content = { version: "test", ...pool };

  it("has about 80 units, every one shippable in both forms", () => {
    expect(pool.units.length).toBeGreaterThanOrEqual(75);
    expect(contentFormProblems(content)).toEqual([]);
  });

  it("gives every unit its own emoji and a tier", () => {
    const emoji = pool.units.map((u) => u.emoji);
    expect(new Set(emoji).size).toBe(emoji.length);
    // One codepoint (plus an emoji presentation selector): a ZWJ sequence
    // splits into two pictures on older phones.
    expect(pool.units.filter((u) => [...u.emoji.replace(/\uFE0F$/u, "")].length !== 1).map((u) => `${u.name} ${u.emoji}`)).toEqual([]);
    for (const t of [1, 2, 3, 4]) expect(pool.units.some((u) => u.tier === t)).toBe(true);
  });

  it("contains chain links: units that listen to what other units emit", () => {
    const listened = new Set(pool.units.map((u) => u.forms.sleeping.when[0]!.on.on));
    for (const e of ["StatusApplied", "Heal", "StatChanged", "Summon", "Death"]) expect(listened).toContain(e);
  });
});

describe("MVP pool chain discipline", () => {
  const ONE_UNIT_EVENTS = new Set(["StatusApplied", "Heal", "StatChanged", "Summon"]);
  it("listeners to one-unit events act on one unit (no n² fan-out)", () => {
    for (const u of mvpPool().units) {
      for (const f of [u.forms.sleeping, u.forms.awoken]) {
        const on = f.when[0]!.on.on;
        if (ONE_UNIT_EVENTS.has(on)) expect([u.id, f.who[0]!.kind]).not.toEqual([u.id, expect.stringMatching(/^all/)]);
      }
    }
  });
});

describe("MVP pool targeting", () => {
  // Who a selector reaches, for a given When: friend or foe.
  const ENEMY_EVENTS = new Set(["enemyPoisoned", "enemyCursed", "enemyDies"]);
  const side = (who: string, when: string) =>
    ["front", "enemies", "random"].includes(who) || (who === "it" && ENEMY_EVENTS.has(when)) ? "foe" : "friend";
  const HOSTILE = /^(Hit|Smite|Poison|Curse|Freeze|Silence)/;

  it("helps allies and hurts enemies, in both forms", () => {
    const wrong: string[] = [];
    for (const row of ROWS) {
      const forms = [
        { who: row.who, does: [row.does] },
        { who: row.awoken.who ?? row.who, does: row.awoken.does ?? [row.does] },
      ];
      for (const f of forms) {
        for (const d of f.does) {
          const want = HOSTILE.test(d) ? "foe" : d.startsWith("Call") || d.startsWith("Revive") ? side(f.who, row.when) : "friend";
          if (side(f.who, row.when) !== want) wrong.push(`${row.name}: ${d} → ${f.who}`);
        }
      }
    }
    expect(wrong).toEqual([]);
  });
});

describe("Planter", () => {
  const content: MvpContent = { version: "test", ...mvpPool() };
  const unit = (id: string) => content.units.find((u) => u.id === id)!;
  const fight = (ids: string[]) => {
    const line = ids.map((id, i) => lineUnitOf(unit(id), `a${i}`));
    const foe = [lineUnitOf(unit("fodder"), "b0")];
    const p = { id: "p", name: "p", bot: false };
    return fightLines({ player: p, line }, { player: p, line: foe }, { battleId: "b", seed: 1, kind: "round", round: 1, runId: null, at: "2026-10-06T00:00:00.000Z", content, rules: MVP_RULES }).log;
  };
  const grew = (log: ReturnType<typeof fight>) => log.some((e) => e.type === "StatusApplied" && e.status === "Vitality" && e.unit.includes("Planter"));
  const summoned = (log: ReturnType<typeof fight>) => log.some((e) => e.type === "Summon" && e.name === "Imp");

  it("calls an Imp when the line has room, and grows", () => {
    const log = fight(["planter", "fighter"]);
    expect(summoned(log)).toBe(true);
    expect(grew(log)).toBe(true);
  });

  it("still does something in a full line: it grows", () => {
    const log = fight(["planter", "fighter", "squire", "gnat", "rose"]);
    expect(summoned(log)).toBe(false);
    expect(grew(log)).toBe(true);
  });
});
