// R4-2: a real battle with "No room" (Sexton + Necromancer fused at the back
// of a full line: one slot frees, the Ghoul fills it, the revive and the Imp
// find no room). Writes the BattleRecord to argv[2]. Used by e2e/probe-no-room.mjs.
import { writeFileSync } from "node:fs";
import { MVP_RULES } from "../src/mvp/contract.js";
import { fightLines } from "../src/mvp/fight.js";
import { fuseUnits, lineUnitOf } from "../src/mvp/forms.js";
import { mvpContent } from "../server/src/mvp/content.js";
const content = mvpContent();
const unit = (id: string) => content.units.find((u) => u.id === id)!;
const P = { id: "p", name: "me", bot: false }, Q = { id: "q", name: "bot", bot: true };
const fused = fuseUnits(lineUnitOf(unit("sexton"), "a0", 3), lineUnitOf(unit("necromancer"), "a9", 3), { name: "Mortwrought", discoveredBy: null }, content, MVP_RULES);
const a = [...[1, 2, 3, 4].map((k) => lineUnitOf(unit("squire"), `a${k}`, 1)), fused];
const b = [{ ...lineUnitOf(unit("squire"), "b0", 3), stats: { pwr: 6, hp: 80 } }];
const rec = fightLines({ player: P, line: a }, { player: Q, line: b }, { battleId: "no-room", seed: 1, kind: "round", round: 9, runId: null, at: "2026-10-07T00:00:00.000Z", content, rules: MVP_RULES });
const n = rec.log.filter((e) => e.type === "NoRoom").length;
if (!n) { console.log("no NoRoom event"); process.exit(1); }
writeFileSync(process.argv[2]!, JSON.stringify(rec));
console.log(`no-room battle: ${n} No room events, ${rec.log.length} events`);
