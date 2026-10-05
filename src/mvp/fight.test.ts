import { describe, expect, it } from "vitest";
import { DEFAULT_RUN_POOL, stressAbilities, stressRegistry } from "../index.js";
import { MVP_RULES, type MvpContent, type PlayerRef, type UnitContent } from "./contract.js";
import { fightLines } from "./fight.js";
import { lineUnitOf } from "./forms.js";
import { applyMvpDecision, initMvpRun, synthGhost } from "./run.js";

const units: UnitContent[] = DEFAULT_RUN_POOL.map((d, i) => {
  const form = { when: d.triggers ?? [], who: d.selectors ?? [], does: d.abilities ?? [] };
  return { id: `u${i}`, name: d.name, emoji: "x", tier: 1, base: d.base, forms: { sleeping: form, awoken: form } };
});
const content: MvpContent = { version: "t", units, abilities: stressAbilities, statuses: stressRegistry };
const me: PlayerRef = { id: "p", name: "me", bot: false };
const them: PlayerRef = { id: "q", name: "them", bot: false };
const at = "2026-10-05T00:01:00.000Z";

describe("fightLines: one line against another", () => {
  it("is the run's fight: same record for the same lines and seed", () => {
    let s = initMvpRun({ runId: "r", player: me, seed: 9, content, day: 1, startedAt: at });
    s = applyMvpDecision(s, { kind: "buy", slot: 0 }, content).state;
    const ghost = synthGhost({ content, round: 1, seed: 5, ghostId: "g", createdAt: at });
    const step = applyMvpDecision(s, { kind: "fight" }, content, { fight: { ghost, battleId: "b", battleSeed: 11, at } });
    const direct = fightLines(
      { player: me, line: s.line },
      { player: ghost.player, line: ghost.line },
      { battleId: "b", seed: 11, kind: "round", round: 1, runId: "r", at, content, rules: MVP_RULES },
    );
    expect(direct).toEqual(step.battle);
  });

  it("plays a playoff game outside any run, with the rules' chain cap", () => {
    const a = [lineUnitOf(units[0]!, "a1"), lineUnitOf(units[1]!, "a2")];
    const b = [lineUnitOf(units[2]!, "b1"), lineUnitOf(units[3]!, "b2")];
    const o = { battleId: "p1", seed: 3, kind: "playoff" as const, round: 0, runId: null, at, content, rules: MVP_RULES };
    const rec = fightLines({ player: me, line: a }, { player: them, line: b }, o);
    expect(rec).toMatchObject({ runId: null, kind: "playoff", round: 0, player: me, opponent: them, contentVersion: "t", at });
    expect(rec.teamA.map((u) => u.uid)).toEqual(["a1", "a2"]);
    expect(rec.log.at(-1)?.type).toBe("BattleEnd");
    expect(fightLines({ player: me, line: a }, { player: them, line: b }, o)).toEqual(rec);
    // The rules' cap reaches the kernel (which refuses a cap below 1).
    expect(() => fightLines({ player: me, line: a }, { player: them, line: b }, { ...o, rules: { ...MVP_RULES, chainStepCap: 0 } })).toThrow(/chainStepCap/);
  });
});
