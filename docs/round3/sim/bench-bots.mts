// R3-13 sim check: bot runs with the bench (rules.benchSize 3) against the same
// bots without it (benchSize 0), same seeds. Fusions and awakenings per run
// must not fall; also how far runs get, and how full the bench is at the end.
// Run: node --import tsx/esm docs/round3/sim/bench-bots.mts [runs]
import { MVP_RULES, type MvpRules } from "../../../src/mvp/contract.ts";
import { playBotRun } from "../../../server/src/mvp/bots.ts";
import { mvpContent } from "../../../server/src/mvp/content.ts";
import { mvpRuntime } from "../../../server/src/mvp/runtime.ts";

const RUNS = Number(process.argv[2] ?? 200);
const content = mvpContent();

function sim(rules: MvpRules) {
  let n = 7;
  let fusions = 0;
  const rt = mvpRuntime({ content, rules, seed: () => (n = (n * 1103515245 + 12345) >>> 0), hooks: [{ onFuse: () => void fusions++ }] });
  let awoken = 0, rounds = 0, crowns = 0, benchEnd = 0, linePower = 0;
  for (let i = 0; i < RUNS; i++) {
    const run = playBotRun(rt);
    awoken += [...run.line, ...run.bench].filter((u) => u.form === "awoken").length;
    rounds += run.fights.length;
    if (run.endedBy === "crown-won" || run.endedBy === "crown-lost" || run.endedBy === "no-champion") crowns++;
    benchEnd += run.bench.length;
    linePower += run.line.reduce((s, u) => s + u.stats.pwr * 2 + u.stats.hp, 0);
  }
  const per = (x: number) => (x / RUNS).toFixed(2);
  return { fusionsPerRun: per(fusions), awokenAtEnd: per(awoken), fightsPerRun: per(rounds), reachedCrown: `${((100 * crowns) / RUNS).toFixed(0)}%`, benchAtEnd: per(benchEnd), linePowerAtEnd: per(linePower) };
}

console.log(`${RUNS} bot runs each, same seeds, no champion (runs end no-champion after round 12)`);
console.table({ "no bench": sim({ ...MVP_RULES, benchSize: 0 }), "bench 3": sim(MVP_RULES) });
