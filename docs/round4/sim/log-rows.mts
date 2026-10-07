// R4-3 (log grouping): Log rows per battle before and after grouping, on the
// longest of many random late-game fights (awoken units and fusions).
//   npx tsx docs/round4/sim/log-rows.mts [battles] [dump.txt]
import { writeFileSync } from "node:fs";
import { battle } from "../../../src/battle.ts";
import { MVP_RULES } from "../../../src/mvp/contract.ts";
import { toBattleDef } from "../../../src/mvp/fight.ts";
import { fuseUnits, lineUnitOf } from "../../../src/mvp/forms.ts";
import { mvpPool } from "../../../src/mvp/units.ts";
import * as trace from "../../../src/mvp/trace.ts";

const pool = mvpPool();
const content = pool as never;
const n = Number(process.argv[2] ?? 400);
let seed = 11;
const rnd = () => (seed = (seed * 1103515245 + 12345) % 2 ** 31) / 2 ** 31;
const pick = () => pool.units[Math.floor(rnd() * pool.units.length)]!;
const line = (tag: string) =>
  Array.from({ length: 5 }, (_, i) => {
    if (rnd() < 0.4) {
      const a = lineUnitOf(pick(), `${tag}${i}`, 3), b = lineUnitOf(pick(), `${tag}${i}b`, 3);
      try { return fuseUnits(a, b, { name: `${a.name}${b.name}`, discoveredBy: null }, content); } catch { return a; }
    }
    return lineUnitOf(pick(), `${tag}${i}`, 1 + Math.floor(rnd() * 3));
  });
const groupLog = (trace as Record<string, unknown>).logRowsOf as undefined | ((log: never, beats: never) => unknown[]);
const runs = (lines: string[]) => lines.filter((c, i) => c === lines[i - 1]).length;
let runsBefore = 0, runsAfter = 0;
const out: { k: number; events: number; before: number; after: number; log: unknown[]; beats: trace.PlayBeat[] }[] = [];
for (let k = 0; k < n; k++) {
  const log = battle({ teamA: line("a").map(toBattleDef), teamB: line("b").map(toBattleDef), seed: k, abilities: pool.abilities, statuses: pool.statuses, chainStepCap: MVP_RULES.chainStepCap, ...(MVP_RULES.turnCap !== undefined ? { turnCap: MVP_RULES.turnCap } : {}) });
  const beats = trace.beatPlayOf(log, trace.stepsOf(log));
  const before = beats.reduce((s, b) => s + b.waves.length, 0);
  const rows = groupLog ? (groupLog(log as never, beats as never) as { caption: string }[]) : [];
  const after = groupLog ? rows.length : NaN;
  runsBefore += runs(beats.flatMap((b) => b.waves.map((w) => w.caption)));
  runsAfter += runs(rows.map((r) => r.caption));
  out.push({ k, events: log.length, before, after, log, beats });
}
out.sort((x, y) => y.before - x.before);
console.log("battle  events  rows before  rows after");
for (const o of out.slice(0, 8)) console.log(`${String(o.k).padStart(6)}  ${String(o.events).padStart(6)}  ${String(o.before).padStart(11)}  ${String(o.after).padStart(10)}`);
const tot = (f: (o: (typeof out)[0]) => number) => out.reduce((s, o) => s + f(o), 0);
console.log(`all ${n}: ${tot((o) => o.before)} → ${tot((o) => o.after)} rows; a line repeating the one above: ${runsBefore} → ${runsAfter}`);
const dump = process.argv[3];
if (dump) {
  const o = out[0]!;
  const lines = groupLog
    ? (groupLog(o.log as never, o.beats as never) as { turn: number; caption: string }[]).map((r) => `T${r.turn} ${r.caption}`)
    : o.beats.flatMap((b) => b.waves.map((w) => `T${b.turn} ${w.caption}`));
  writeFileSync(dump, lines.join("\n") + "\n");
}
