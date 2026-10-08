// A real battle the turn cap stops (R3-26): mirror lines of a fused
// Equalizer + Prepper fed by War Drummer re-bless the whole line every turn,
// so Fatigue never kills and the battle reaches round 3's turn cap. New runs
// have sudden death instead (R4-1), so this plays under a round-3 run's rules:
// the viewer must still show such stored battles. Writes the BattleRecord to
// argv[2]. Used by e2e/probe-time-up.mjs.
import { writeFileSync } from "node:fs";
import { MVP_RULES, ROUND3_TURN_CAP } from "../src/mvp/contract.js";
import { fightLines } from "../src/mvp/fight.js";
import { fuseUnits, lineUnitOf } from "../src/mvp/forms.js";
import { mvpContent } from "../server/src/mvp/content.js";
const content = mvpContent();
const { suddenDeathAt: _sd, ...noSudden } = MVP_RULES;
const rules = { ...noSudden, turnCap: ROUND3_TURN_CAP };
const unit = (id: string) => content.units.find((u) => u.id === id)!;
const P = { id: "p", name: "me", bot: false }, Q = { id: "q", name: "bot", bot: true };
const line = (side: string) => [
  fuseUnits(lineUnitOf(unit("equalizer"), `${side}0`, 3), lineUnitOf(unit("prepper"), `${side}9`, 3), { name: "Equapper", discoveredBy: null }, content, MVP_RULES),
  ...["war-drummer", "fighter", "bulwark"].map((id, i) => lineUnitOf(unit(id), `${side}${i + 1}`)),
];
const rec = fightLines({ player: P, line: line("a") }, { player: Q, line: line("b") }, { battleId: "time-up", seed: 1, kind: "round", round: 9, runId: null, at: "2026-10-07T06:00:00.000Z", content, rules });
const end = rec.log.at(-1);
if (end?.type !== "BattleEnd" || !end.timeUp || end.turns !== ROUND3_TURN_CAP) throw new Error(`no time-up: ${JSON.stringify(end)}`);
writeFileSync(process.argv[2]!, JSON.stringify(rec));
console.log("time-up battle", rec.log.length, "events", JSON.stringify(end));
