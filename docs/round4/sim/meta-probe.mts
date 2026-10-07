// R4-18 probe: score teams against a field of teams (a meta-health report's
// breaker teams) on the current content, in seconds, to try number changes
// before a full `npm run mvp:meta`. Same fight setup as scripts/meta-health.ts.
// Run: node --import tsx/esm docs/round4/sim/meta-probe.mts <report.json> [team ...]
//   team = comma-separated unit ids, front first; none = the report's top 12 breakers.
import { readFileSync } from "node:fs";
import { MVP_RULES, type LineUnit, type PlayerRef, type UnitContent } from "../../../src/mvp/contract.ts";
import { fightLines } from "../../../src/mvp/fight.ts";
import { lineUnitOf } from "../../../src/mvp/forms.ts";
import { mvpContent } from "../../../server/src/mvp/content.ts";

const content = mvpContent();
const byId = new Map(content.units.map((u) => [u.id, u]));
const report = JSON.parse(readFileSync(process.argv[2]!, "utf8")) as { breakers: { unit: string; team: string[]; fieldScore: number }[] };
const field = report.breakers.map((b) => b.team).filter((t) => t.every((id) => byId.has(id)));
const SIM: PlayerRef = { id: "probe", name: "probe", bot: true };
const line = (t: string[]): LineUnit[] => t.map((id, i) => lineUnitOf(byId.get(id) as UnitContent, `u${i}`, 3));
let battles = 0, capped = 0;
function score(a: string[], b: string[], seeds = 2): number {
  let s = 0;
  for (let i = 0; i < seeds; i++) for (const side of ["A", "B"] as const) {
    const [la, lb] = side === "A" ? [line(a), line(b)] : [line(b), line(a)];
    const rec = fightLines({ player: SIM, line: la }, { player: SIM, line: lb }, { battleId: "probe", seed: 1 + i, kind: "round", round: MVP_RULES.rounds, runId: null, at: new Date(0).toISOString(), content, rules: MVP_RULES });
    battles++;
    if (rec.log.some((e) => e.type === "ChainCapped")) capped++;
    s += rec.winner === side ? 1 : rec.winner === "draw" ? 0.5 : 0;
  }
  return s / (2 * seeds);
}
const teams = process.argv.length > 3
  ? process.argv.slice(3).map((t) => t.split(","))
  : [...report.breakers].sort((a, b) => b.fieldScore - a.fieldScore).slice(0, 12).map((b) => b.team);
for (const t of teams) {
  const f = field.reduce((s, o) => s + score(t, o), 0) / field.length;
  console.log(`${(f * 100).toFixed(0).padStart(3)}%  ${t.join(",")}`);
}
console.log(`cap ${(100 * capped / battles).toFixed(1)}% of ${battles}`);
