// Trace helper tests — the shared cause-chain narration the text replay and
// the web viewer both consume. The contract: ancestry walks causedBy to the
// kernel beat, names resolve like the replay's, and a death's lineage reads
// "Poison tick ← applied turn N" straight off the log.

import { describe, expect, test } from "vitest";
import { battle } from "./battle.js";
import { renderReplay } from "./replay.js";
import { ancestry, deathCauseChain, displayNames, shortDesc } from "./trace.js";
import type { BattleInput, UnitDef } from "./types.js";
import { stressRegistry, Venomancer } from "./content/stress.js";
import { stressAbilities } from "./content/stress.js";

const dummy = (name: string, hp = 10, pwr = 2): UnitDef => ({ name, base: { hp, pwr }, ability: "Strike" });

const poisonBattle: BattleInput = {
  teamA: [Venomancer, dummy("Backup", 20, 1)],
  teamB: [dummy("Victim", 16, 1)],
  seed: 3,
  statuses: stressRegistry,
  abilities: stressAbilities,
};

describe("displayNames", () => {
  test("resolves every unit id to its bare name — the Xn: instance prefix never shows (#065 item 3)", () => {
    const log = battle({ teamA: [dummy("Twin"), dummy("Twin")], teamB: [dummy("Solo")], seed: 1, abilities: stressAbilities });
    const name = displayNames(log);
    // Duplicates show the same bare name — the team tint, not an id prefix, tells
    // the two apart in the UI; the full id stays on data-unit / event ids.
    expect(name("A1:Twin")).toBe("Twin");
    expect(name("A2:Twin")).toBe("Twin");
    expect(name("B1:Solo")).toBe("Solo");
    expect(name("nobody")).toBe("nobody"); // unknown ids pass through
  });
});

describe("ancestry", () => {
  test("walks causedBy from direct parent to a kernel beat", () => {
    const log = battle(poisonBattle);
    const hurt = log.find((e) => e.type === "Hurt" && e.source === "kernel" && e.causedBy !== null)!;
    const chain = ancestry(log, hurt.id);
    expect(chain.length).toBeGreaterThan(0);
    expect(chain[0]!.id).toBe(hurt.causedBy); // direct parent first
    expect(chain[chain.length - 1]!.causedBy).toBeNull(); // root is a kernel beat
    for (let i = 1; i < chain.length; i++) expect(chain[i]!.id).toBe(chain[i - 1]!.causedBy);
  });

  test("a kernel beat has empty ancestry", () => {
    const log = battle(poisonBattle);
    expect(log[0]!.causedBy).toBeNull();
    expect(ancestry(log, 0)).toEqual([]);
  });
});

describe("deathCauseChain", () => {
  test("a poison death traces tick and application turn", () => {
    const log = battle(poisonBattle);
    const death = log.find((e) => e.type === "Death")!;
    const chain = deathCauseChain(log, death.causedBy!, displayNames(log));
    expect(chain.some((p) => p.includes("Poison tick"))).toBe(true);
    expect(chain.some((p) => p.match(/Poison applied turn \d+/))).toBe(true);
  });

  test("matches what the text replay prints for the same death", () => {
    const log = battle(poisonBattle);
    const death = log.find((e) => e.type === "Death")!;
    const chain = deathCauseChain(log, death.causedBy!, displayNames(log));
    expect(renderReplay(log)).toContain(`dies ← ${chain.join(" ← ")}`);
  });

  test("a plain strike death names the striker", () => {
    const log = battle({ teamA: [dummy("Bruiser", 10, 9)], teamB: [dummy("Frail", 8, 1)], seed: 0, abilities: stressAbilities });
    const death = log.find((e) => e.type === "Death" && e.unit === "B1:Frail")!;
    const chain = deathCauseChain(log, death.causedBy!, displayNames(log));
    expect(chain.join(" ← ")).toMatch(/struck by Bruiser for \d+/);
  });
});

describe("shortDesc", () => {
  test("covers the named event shapes", () => {
    const log = battle(poisonBattle);
    const name = displayNames(log);
    const byType = (t: string) => log.find((e) => e.type === t);
    expect(shortDesc(byType("Strike"), name)).toMatch(/'s strike$/);
    expect(shortDesc(byType("Hurt"), name)).toMatch(/was hurt$/);
    expect(shortDesc(byType("Death"), name)).toMatch(/died$/);
    expect(shortDesc(byType("TurnStart"), name)).toBe("the turn began");
    expect(shortDesc(undefined, name)).toBe("an earlier event");
  });
});
