// Pacing of the battle viewer (R2-12): steps vs beats over random MVP lines.
// Run: npx tsx docs/round2/sim/beat-pacing.mts [battles]
import { battle } from "../../../src/battle.js";
import { MVP_RULES } from "../../../src/mvp/contract.js";
import { toBattleDef } from "../../../src/mvp/fight.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import { beatPlayOf, beatTiming, stepsOf } from "../../../src/mvp/trace.js";
import { mvpPool } from "../../../src/mvp/units.js";

const pool = mvpPool();
const n = Number(process.argv[2] ?? 300);
let seed = 1;
const rnd = () => ((seed = (seed * 1103515245 + 12345) % 2 ** 31) / 2 ** 31);
const line = (tag: string) =>
  Array.from({ length: 5 }, (_, i) => {
    const u = pool.units[Math.floor(rnd() * pool.units.length)]!;
    const l = lineUnitOf(u, `${tag}${i}`);
    return { ...l, stats: { pwr: l.stats.pwr + Math.floor(rnd() * 4), hp: l.stats.hp + Math.floor(rnd() * 8) } };
  });
const stepsN: number[] = [], beatsN: number[] = [], secs: number[] = [], maxMs: number[] = [];
for (let k = 0; k < n; k++) {
  const log = battle({ teamA: line("a").map(toBattleDef), teamB: line("b").map(toBattleDef), seed: k, abilities: pool.abilities, statuses: pool.statuses, chainStepCap: MVP_RULES.chainStepCap });
  const steps = stepsOf(log);
  const beats = beatPlayOf(log, stepsOf(log));
  const ms = beats.map((b) => beatTiming(b.waves.length).ms);
  stepsN.push(steps.length); beatsN.push(beats.length); secs.push(ms.reduce((a, b) => a + b, 0) / 1000); maxMs.push(Math.max(...ms));
}
const q = (xs: number[], p: number) => [...xs].sort((a, b) => a - b)[Math.floor((xs.length - 1) * p)]!;
console.log(`${n} battles`);
console.log(`steps  median ${q(stepsN, 0.5)}, p90 ${q(stepsN, 0.9)} → ${(q(stepsN, 0.5) * 0.65).toFixed(0)} s at 650 ms/step`);
console.log(`beats  median ${q(beatsN, 0.5)}, p90 ${q(beatsN, 0.9)}`);
console.log(`1× time median ${q(secs, 0.5).toFixed(1)} s, p90 ${q(secs, 0.9).toFixed(1)} s; longest beat ${Math.max(...maxMs)} ms`);
