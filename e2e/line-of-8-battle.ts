// R4-10: a real battle on a line of 8 (Sexton + Necromancer fused,
// "Mortwrought", at the back of a full team of 5 on both sides: Ghouls, Imps
// and revives fill each line past 5, up to 8). Writes the BattleRecord to
// argv[2]. Used by e2e/probe-line-of-8.mjs.
import { writeFileSync } from "node:fs";
import { boardAt } from "../src/board.js";
import { MVP_RULES } from "../src/mvp/contract.js";
import { fightLines } from "../src/mvp/fight.js";
import { fuseUnits, lineUnitOf } from "../src/mvp/forms.js";
import { mvpContent } from "../server/src/mvp/content.js";
const content = mvpContent();
const unit = (id: string) => content.units.find((u) => u.id === id)!;
const P = { id: "p", name: "me", bot: false }, Q = { id: "q", name: "bot", bot: true };
const team = (s: string) => [
  ...[1, 2, 3, 4].map((k) => lineUnitOf(unit("squire"), `${s}${k}`, 1)),
  fuseUnits(lineUnitOf(unit("sexton"), `${s}0`, 3), lineUnitOf(unit("necromancer"), `${s}9`, 3), { name: "Mortwrought", discoveredBy: null }, content, MVP_RULES),
];
const rec = fightLines({ player: P, line: team("a") }, { player: Q, line: team("b") }, { battleId: "line-of-8", seed: Number(process.argv[3] ?? 1), kind: "round", round: 9, runId: null, at: "2026-10-07T00:00:00.000Z", content, rules: MVP_RULES });
const peak = { A: 0, B: 0 };
/** The first turn whose end finds both lines full (8 each), for the probe to seek to. */
let fullTurn = -1;
rec.log.forEach((e, i) => {
  if (e.type === "TurnEnd" && fullTurn < 0) {
    const b = boardAt(rec.log, i);
    if (b.lines.A.length === 8 && b.lines.B.length === 8) fullTurn = b.turn;
  }
  if (e.type !== "Summon") return;
  const b = boardAt(rec.log, i);
  peak.A = Math.max(peak.A, b.lines.A.length);
  peak.B = Math.max(peak.B, b.lines.B.length);
});
if (peak.A < 8 && peak.B < 8) { console.log(`no line reached 8 (A ${peak.A}, B ${peak.B})`); process.exit(1); }
if (fullTurn < 0) { console.log("no turn ends with both lines at 8"); process.exit(1); }
writeFileSync(process.argv[2]!, JSON.stringify(rec));
writeFileSync(`${process.argv[2]!}.turn`, String(fullTurn));
console.log(`line-of-8 battle: peak A ${peak.A}, B ${peak.B}, both full at the end of turn ${fullTurn}, ${rec.log.length} events, winner ${rec.winner}`);
