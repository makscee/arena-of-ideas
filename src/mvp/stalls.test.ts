// Every battle ends by itself (R3-26, R4-1). Normal fights end by turn 20
// (p99 17), but some lines stall: a Blessing re-armed every turn beats Fatigue
// (Divinity's Awoken with a death each turn, or an "Ally gains PWR" fusion with
// a blessing second part fed by War Drummer), and two Necromancers revive each
// other. Round 3 cut them off at turnCap 30 ("Time's up: draw"). Round 4's
// sudden death ends them instead: from turn 20 Fatigue doubles, pierces
// Shield and Blessing, and Summon and Revive do nothing, so every fight here
// ends by turn 21 (a fed fan-out fusion by turn 22), and none ever reaches the kernel's 200-turn net.

import { describe, expect, it } from "vitest";
import type { BattleEvent } from "../types.js";
import { MVP_RULES, type LineUnit, type MvpContent, type PlayerRef, type UnitContent } from "./contract.js";
import { fightLines } from "./fight.js";
import { fuseUnits, lineUnitOf } from "./forms.js";
import { mvpPool, whenKeyOf } from "./units.js";

const pool = mvpPool();
const content: MvpContent = { version: "stalls", ...pool };
const p: PlayerRef = { id: "p", name: "p", bot: false };
const unit = (id: string): UnitContent => {
  const u = pool.units.find((x) => x.id === id);
  if (!u) throw new Error(`no unit ${id} in the pool`);
  return u;
};

/** The sweep's limits: the last turn a fight may reach (sudden death's
 * second turn), and no fight comes near 5,000 events. */
const LAST_TURN = MVP_RULES.suddenDeathAt! + 1;
const MAX_EVENTS = 5_000;

interface Fought {
  label: string;
  events: number;
  turns: number;
  timeUp: boolean;
}

function fight(a: LineUnit[], b: LineUnit[], seed: number, label: string): Fought {
  const { log } = fightLines({ player: p, line: a }, { player: p, line: b }, { battleId: "x", seed, kind: "round", round: 9, runId: null, at: "2026-10-07T00:00:00.000Z", content, rules: MVP_RULES });
  const end = log.at(-1) as Extract<BattleEvent, { type: "BattleEnd" }>;
  return { label, events: log.length, turns: end.turns, timeUp: end.timeUp === true };
}

/** Every battle over by sudden death's second turn, by itself (never time's
 * up), and well under 5,000 events. */
function expectBounded(all: Fought[], lastTurn = LAST_TURN): void {
  expect(MVP_RULES.turnCap, "new runs have no turn cap").toBeUndefined();
  expect(all.filter((f) => f.turns > lastTurn).map((f) => `${f.label}: turn ${f.turns}`)).toEqual([]);
  expect(all.filter((f) => f.timeUp).map((f) => f.label)).toEqual([]);
  expect(all.filter((f) => f.events > MAX_EVENTS).map((f) => `${f.label}: ${f.events} events`)).toEqual([]);
}

describe("every battle ends by itself (R3-26, R4-1)", () => {
  it("the known stall teams, which ran to the kernel's 200 turns, now end by turn 21", () => {
    const fused = (side: string, a: string, b: string) =>
      fuseUnits(lineUnitOf(unit(a), `${side}0`, 3), lineUnitOf(unit(b), `${side}9`, 3), { name: `${a}+${b}`, discoveredBy: null }, content, MVP_RULES);
    const plain = (side: string, ids: string[], from = 1, copies = 1) => ids.map((id, i) => lineUnitOf(unit(id), `${side}${i + from}`, copies));
    const teams: Array<[string, number, (side: string) => LineUnit[]]> = [
      // A Blessing re-armed every turn: War Drummer feeds an "ally gains PWR" fusion that blesses the line.
      ["re-blessed (Equalizer+Prepper)", 1, (s) => [fused(s, "equalizer", "prepper"), ...plain(s, ["war-drummer", "fighter", "bulwark"])]],
      ["re-blessed (Lightning+Divinity)", 1, (s) => [fused(s, "lightning", "divinity"), ...plain(s, ["war-drummer", "fighter", "bulwark"])]],
      // Two revivers bringing each other back. (R4-14: Guardian and Robber's
      // stronger Awoken forms broke the stall, so Priest and Prepper stand in.)
      ["two Necromancers", 64, (s) => plain(s, ["necromancer", "necromancer", "priest", "prepper", "fodder"], 0, 3)],
    ];
    // They stalled on round 3's line of 5 (two Necromancers have room to
    // finish each other on R4-10's line of 8), so both runs keep that line.
    const { battleSize: _bs, ...lineOf5 } = MVP_RULES;
    const { suddenDeathAt: _sd, ...noSudden } = lineOf5;
    const run = (team: (side: string) => LineUnit[], seed: number, rules: typeof MVP_RULES) =>
      fightLines({ player: p, line: team("a") }, { player: p, line: team("b") }, { battleId: "x", seed, kind: "round", round: 9, runId: null, at: "2026-10-07T00:00:00.000Z", content, rules }).log;
    const all: Fought[] = [];
    for (const [label, seed, team] of teams) {
      // Without sudden death (and no cap) each runs to the kernel's 200: a real stall.
      expect(run(team, seed, noSudden).at(-1), label).toMatchObject({ type: "BattleEnd", timeUp: true, turns: 200 });
      const log = run(team, seed, lineOf5);
      const end = log.at(-1) as Extract<BattleEvent, { type: "BattleEnd" }>;
      all.push({ label, events: log.length, turns: end.turns, timeUp: end.timeUp === true });
      expect(end.turns, label).toBeGreaterThanOrEqual(MVP_RULES.suddenDeathAt!);
      expect(log.some((e) => e.type === "Fatigue" && e.suddenDeath), label).toBe(true);
    }
    expectBounded(all);
  });

  it("random mirror lines at 3 copies all end by turn 21", { timeout: 60_000 }, () => {
    // Mirrors are where stalls live (non-mirror lines: 0 of 6,000 sampled).
    let s = 574;
    const rnd = () => (s = (Math.imul(s, 1103515245) + 12345) >>> 0) / 2 ** 32;
    const ids = pool.units.map((u) => u.id);
    const all: Fought[] = [];
    for (let k = 0; k < 500; k++) {
      const pick = Array.from({ length: MVP_RULES.lineSize }, () => ids[Math.floor(rnd() * ids.length)]!);
      const line = (side: string) => pick.map((id, i) => lineUnitOf(unit(id), `${side}${i}`, 3));
      all.push(fight(line("a"), line("b"), k, pick.join("·")));
    }
    expectBounded(all);
  });

  it("every fan-out fusion (a one-unit When + a group Who) ends by turn 22", { timeout: 60_000 }, () => {
    // What the bots' fansOut refuses but a human can fuse (content.md, R3-26).
    // A feeder makes the fused unit's When fire every turn.
    const FEEDERS: Record<string, string[]> = {
      allyPower: ["war-drummer", "commander", "coach"],
      allyShield: ["keeper", "fodder", "prepper"],
      allyHealed: ["medic", "doctor", "harvest"],
      allySummoned: ["fungoid", "sexton", "summoner"],
      enemyPoisoned: ["rot", "rat", "injector"],
      enemyCursed: ["physician", "spore", "saboteur"],
    };
    const has = (id: string) => pool.units.some((u) => u.id === id);
    const feeder = (key: string) => {
      const id = FEEDERS[key]?.find(has);
      if (!id) throw new Error(`no feeder for a fused unit's When "${key}"`);
      return id;
    };
    const fillers = ["fighter", "bulwark", "rose"].map(unit);
    const ONE_UNIT = new Set(["StatusApplied", "Heal", "StatChanged", "Summon"]);
    const all: Fought[] = [];
    let pairs = 0;
    for (const a of pool.units)
      for (const b of pool.units) {
        if (a.id === b.id) continue;
        const [x, y] = [lineUnitOf(a, "x", 3), lineUnitOf(b, "y", 3)];
        const fansOut = x.recipe.when.some((w) => w.kind === "trigger" && ONE_UNIT.has(w.on.on)) && y.recipe.who.some((w) => w.kind.startsWith("all"));
        if (!fansOut) continue;
        pairs++;
        const fed = [unit(feeder(whenKeyOf(a.forms.awoken))), fillers[0]!, fillers[1]!];
        for (const [fill, rest] of [["fed", fed], ["plain", fillers]] as const) {
          const line = (side: string) => [
            fuseUnits(lineUnitOf(a, `${side}0`, 3), lineUnitOf(b, `${side}9`, 3), { name: `${a.name}+${b.name}`, discoveredBy: null }, content, MVP_RULES),
            ...rest.map((u, i) => lineUnitOf(u, `${side}${i + 1}`)),
          ];
          all.push(fight(line("a"), line("b"), 1, `${a.id}+${b.id} (${fill})`));
        }
      }
    // About 630 ordered pairs today; the sweep must not quietly empty.
    expect(pairs).toBeGreaterThan(300);
    // One turn more than the known stalls: since R4-14, Almsgiver+King fed by
    // Keeper heals and grows its whole line on every ally Shield (its Awoken
    // keeps the Heal), so its line outlives 40 and falls to 80 on turn 22.
    expectBounded(all, LAST_TURN + 1);
  });
});
