import { describe, expect, it } from "vitest";
import { DEFAULT_CHAIN_STEP_CAP, TURN_CAP } from "../battle.js";
import { DEFAULT_RUN_POOL, stressAbilities, stressRegistry } from "../index.js";
import { MVP_RULES, ROUND3_TURN_CAP, type MvpContent, type PlayerRef, type UnitContent } from "./contract.js";
import { fightLines } from "./fight.js";
import { lineUnitOf } from "./forms.js";
import { applyMvpDecision, initMvpRun, synthGhost } from "./run.js";
import { TIME_UP_CAPTION, captionOf } from "./trace.js";

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

  it("caps a cascade at 32 steps; a run started under 64 keeps 64 (note 19)", () => {
    expect(MVP_RULES.chainStepCap).toBe(32);
    expect(DEFAULT_CHAIN_STEP_CAP).toBe(32);
    // Every Echo pings all enemies after any unit is hit: each hit fans out ×5.
    const form = { when: [{ kind: "trigger" as const, on: { on: "Hurt" as const } }], who: [{ kind: "allEnemies" as const }], does: ["Ping"] };
    const echo: UnitContent = { id: "echo", name: "Echo", emoji: "x", tier: 1, base: { pwr: 1, hp: 30 }, forms: { sleeping: form, awoken: form } };
    const c: MvpContent = { ...content, units: [echo], abilities: { ...stressAbilities, Ping: { name: "Ping", family: "Strike", effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }] } } };
    const line = (p: string) => Array.from({ length: 5 }, (_, i) => lineUnitOf(echo, `${p}${i}`));
    const caps = (rules: typeof MVP_RULES) => {
      const s = { ...initMvpRun({ runId: "r", player: me, seed: 9, content: c, day: 1, startedAt: at, rules }), line: line("a") };
      const ghost = { ...synthGhost({ content: c, round: 1, seed: 5, ghostId: "g", createdAt: at }), line: line("b") };
      const log = applyMvpDecision(s, { kind: "fight" }, c, { fight: { ghost, battleId: "b", battleSeed: 11, at } }).battle!.log;
      return [...new Set(log.flatMap((e) => (e.type === "ChainCapped" ? [captionOf(log, e.id)] : [])))];
    };
    expect(caps(MVP_RULES)).toEqual(["Chain stopped after 32 steps"]);
    expect(caps({ ...MVP_RULES, chainStepCap: 64 })).toEqual(["Chain stopped after 64 steps"]);
  });

  it("a round-3 run ends a fight still going after turn 30 as a draw, Time's up; a run started before keeps 200 (R3-26)", () => {
    // Round 4 dropped the cap from new runs (sudden death ends their fights); round-3 runs keep theirs.
    expect(MVP_RULES.turnCap).toBeUndefined();
    const { suddenDeathAt: _sd, ...noSudden } = MVP_RULES;
    const round3 = { ...noSudden, turnCap: ROUND3_TURN_CAP };
    // Walls: 1 PWR and more HP than 200 turns of Fatigue take, so only the clock ends it.
    const form = { when: [{ kind: "trigger" as const, on: { on: "BattleStart" as const } }], who: [{ kind: "holder" as const }], does: ["Strike"] };
    const wall: UnitContent = { id: "wall", name: "Wall", emoji: "x", tier: 1, base: { pwr: 1, hp: 100_000 }, forms: { sleeping: form, awoken: form } };
    const c: MvpContent = { ...content, units: [wall] };
    const fight = (rules: typeof MVP_RULES, crown = false) => {
      const s0 = initMvpRun({ runId: "r", player: me, seed: 9, content: c, day: 1, startedAt: at, rules });
      const s = { ...s0, line: [lineUnitOf(wall, "a")], ...(crown ? { phase: "crown" as const, round: rules.rounds + 1 } : {}) };
      const ghost = { ...synthGhost({ content: c, round: 1, seed: 5, ghostId: "g", createdAt: at }), line: [lineUnitOf(wall, "b")] };
      return applyMvpDecision(s, { kind: "fight" }, c, { fight: { ghost, battleId: "b", battleSeed: 11, at } });
    };
    const step = fight(round3);
    const end = step.battle!.log.at(-1)!;
    expect(end).toMatchObject({ type: "BattleEnd", winner: "draw", turns: 30, timeUp: true });
    expect(captionOf(step.battle!.log, end.id)).toBe(TIME_UP_CAPTION);
    // It counts as any draw: no heart lost, the run goes on; at the Crown, no slay.
    expect(step.fight).toMatchObject({ outcome: "draw", heartsLost: 0, heartsAfter: MVP_RULES.hearts });
    expect(step.state).toMatchObject({ round: 2, hearts: MVP_RULES.hearts, phase: "shop" });
    expect(fight(round3, true).state).toMatchObject({ phase: "over", endedBy: "crown-lost" });
    // A run stored before the cap has no turnCap: the kernel's 200.
    expect(fight(noSudden).battle!.log.at(-1)).toMatchObject({ winner: "draw", turns: TURN_CAP, timeUp: true });
    // A new run: sudden death from turn 20 wears through even 100,000 HP, with no clock.
    const now = fight(MVP_RULES).battle!.log;
    expect(MVP_RULES.suddenDeathAt).toBe(20);
    expect(now.at(-1)).toMatchObject({ type: "BattleEnd", winner: "draw" });
    expect(now.at(-1)).not.toHaveProperty("timeUp");
    expect(now.at(-1)!.turn).toBeLessThan(40);
    expect(now.filter((e) => e.type === "Fatigue" && e.turn >= 20).every((e) => e.type === "Fatigue" && e.suddenDeath)).toBe(true);
  });
});
