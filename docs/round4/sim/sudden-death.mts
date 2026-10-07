// R4-1 check: many late-game fights under the new rules (sudden death from
// MVP_RULES.suddenDeathAt) against the same fights with no sudden death and the
// kernel's 200-turn net (engine.md (1)'s baseline). Lines, at 3–5 copies: half
// the meta-health breaker and equilibrium teams (docs/mvp/meta-health.json),
// half random units, of those half led by a fusion of two random awoken units.
// Run: node --import tsx/esm docs/round4/sim/sudden-death.mts [fights=6000]
import { MVP_RULES, type LineUnit, type MvpContent, type PlayerRef } from "../../../src/mvp/contract.js";
import { fightLines } from "../../../src/mvp/fight.js";
import { fuseUnits, lineUnitOf } from "../../../src/mvp/forms.js";
import { mvpContent } from "../../../server/src/mvp/content.js";
import type { BattleEvent } from "../../../src/types.js";
import { readFileSync } from "node:fs";

const N = Number(process.argv[2] ?? 6000);
const content: MvpContent = mvpContent();
const p: PlayerRef = { id: "p", name: "p", bot: false };
const { suddenDeathAt: _sd, turnCap: _tc, ...baseline } = MVP_RULES;
let s = 684;
const rnd = () => (s = (Math.imul(s, 1103515245) + 12345) >>> 0) / 2 ** 32;
const pick = <T,>(xs: T[]) => xs[Math.floor(rnd() * xs.length)]!;
const copies = () => 3 + Math.floor(rnd() * 3);
const meta = JSON.parse(readFileSync(new URL("../../mvp/meta-health.json", import.meta.url), "utf8"));
const known = new Set(content.units.map((u) => u.id));
const metaTeams: string[][] = [...meta.breakers, ...meta.equilibrium].map((t: { team: string[] }) => t.team).filter((t) => t.every((id) => known.has(id)));
const line = (side: string): LineUnit[] => {
  if (metaTeams.length && rnd() < 0.5) return pick(metaTeams).map((id, i) => lineUnitOf(content.units.find((u) => u.id === id)!, `${side}${i}`, copies()));
  const units = Array.from({ length: MVP_RULES.lineSize }, () => pick(content.units));
  const out = units.map((u, i) => lineUnitOf(u, `${side}${i}`, copies()));
  if (rnd() < 0.5) {
    const [a, b] = [pick(content.units), pick(content.units)];
    out[0] = fuseUnits(lineUnitOf(a, `${side}f1`, 3), lineUnitOf(b, `${side}f2`, 3), { name: `${a.name}+${b.name}`, discoveredBy: null }, content, MVP_RULES);
  }
  return out;
};
const end = (log: BattleEvent[]) => log.at(-1) as Extract<BattleEvent, { type: "BattleEnd" }>;
const stat = { neverEnd: [0, 0], maxTurn: [0, 0], draws: [0, 0], changed: 0, sdFights: 0, failed: 0, pierced: 0 };
const turns: number[] = [];
for (let k = 0; k < N; k++) {
  const [a, b] = [line("a"), line("b")];
  const o = (rules: typeof MVP_RULES) => ({ battleId: "sim", seed: k, kind: "round" as const, round: 9, runId: null, at: "2026-10-07T00:00:00.000Z", content, rules });
  const logs = [fightLines({ player: p, line: a }, { player: p, line: b }, o(baseline)).log, fightLines({ player: p, line: a }, { player: p, line: b }, o(MVP_RULES)).log];
  logs.forEach((log, i) => {
    const e = end(log);
    if (e.timeUp) stat.neverEnd[i]!++;
    stat.maxTurn[i] = Math.max(stat.maxTurn[i]!, e.turns);
    if (e.winner === "draw") stat.draws[i]!++;
  });
  const now = logs[1]!;
  turns.push(end(now).turns);
  if (end(logs[0]!).winner !== end(now).winner) stat.changed++;
  if (end(now).turns >= MVP_RULES.suddenDeathAt!) stat.sdFights++;
  stat.failed += now.filter((e) => e.type === "SummonFailed").length;
  stat.pierced += now.filter((e) => e.type === "Hurt" && e.pierced).length;
}
turns.sort((x, y) => x - y);
const q = (f: number) => turns[Math.min(turns.length - 1, Math.floor(turns.length * f))];
console.log(`${N} late-game fights            no sudden death (net 200) | sudden death from ${MVP_RULES.suddenDeathAt}`);
console.log(`never end (time's up)          ${stat.neverEnd[0]} | ${stat.neverEnd[1]}`);
console.log(`max turn                       ${stat.maxTurn[0]} | ${stat.maxTurn[1]}`);
console.log(`draws                          ${stat.draws[0]} | ${stat.draws[1]}`);
console.log(`winner changed                 ${stat.changed} (${((100 * stat.changed) / N).toFixed(2)}%)`);
console.log(`new rules: median turn ${q(0.5)}, p90 ${q(0.9)}, p99 ${q(0.99)}; ${stat.sdFights} fights reached sudden death; ${stat.pierced} pierced hits, ${stat.failed} summons/revives stopped`);
