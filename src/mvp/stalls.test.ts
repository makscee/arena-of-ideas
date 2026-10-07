// Every battle ends sanely (R3-26). Normal fights end by turn 20 (p99 17), but
// some lines stall: a Blessing re-armed every turn beats Fatigue (Divinity's
// Awoken with a death each turn, or an "Ally gains PWR" fusion with a
// blessing second part fed by War Drummer), so nobody dies and the battle ran
// to the kernel's 200 turns (17k–58k events, up to 10 MB of log). The rules'
// turnCap (30) now ends those as "Time's up: draw". This sweep keeps the cap a
// backstop, not a way battles usually end: a new unit or fusion that stalls
// shows up here by name.

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

/** The sweep's limits. A stalled battle at the cap logs ~5–6k events (about
 * 200 a turn); no battle that ends by itself comes near 5,000. */
const MAX_TIME_UP_SHARE = 0.005;
const MAX_EVENTS = 10_000;
const MAX_EVENTS_DECIDED = 5_000;

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

/** Every battle under the cap, and the decided ones well under 5,000 events. */
function expectBounded(all: Fought[]): void {
  expect(all.filter((f) => f.turns > MVP_RULES.turnCap!).map((f) => f.label)).toEqual([]);
  expect(all.filter((f) => f.events > MAX_EVENTS).map((f) => `${f.label}: ${f.events} events`)).toEqual([]);
  expect(all.filter((f) => !f.timeUp && f.events > MAX_EVENTS_DECIDED).map((f) => `${f.label}: ${f.events} events`)).toEqual([]);
}

describe("every battle ends sanely (R3-26)", () => {
  it("random mirror lines at 3 copies: at most 0.5% run out of time", { timeout: 60_000 }, () => {
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
    const timeUp = all.filter((f) => f.timeUp).map((f) => f.label);
    expect(timeUp.length, `time's up in: ${timeUp.join(", ")}`).toBeLessThanOrEqual(Math.floor(all.length * MAX_TIME_UP_SHARE));
  });

  it("every fan-out fusion (a one-unit When + a group Who): only the known pairs run out of time", { timeout: 60_000 }, () => {
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
    // The open question for Maks (Fatigue vs Blessing, or a fusion rule): these
    // stall with War Drummer feeding them. The cap makes them a draw at turn 30.
    const KNOWN = new Set(["lightning", "equalizer"].flatMap((a) => ["prepper", "divinity", "king", "fruiter"].map((b) => `${a}+${b}`)));
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
    expectBounded(all);
    const fresh = all.filter((f) => f.timeUp && !KNOWN.has(f.label.split(" ")[0]!)).map((f) => f.label);
    expect(fresh, "a new fusion stalls to the turn cap").toEqual([]);
    expect(all.filter((f) => f.timeUp).length).toBeLessThanOrEqual(Math.max(KNOWN.size, Math.floor(all.length * MAX_TIME_UP_SHARE)));
  });
});
