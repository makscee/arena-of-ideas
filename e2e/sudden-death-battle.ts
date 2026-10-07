// A real battle sudden death ends (R4-1): mirror lines of two Necromancers
// (3 copies each) reviving the line ran to the kernel's 200 turns before; now
// Fatigue doubles from turn 20, goes through everything, and the revives fail
// with "sudden death". Writes the BattleRecord to argv[2]. Used by
// e2e/probe-sudden-death.mjs.
import { writeFileSync } from "node:fs";
import { MVP_RULES } from "../src/mvp/contract.js";
import { fightLines } from "../src/mvp/fight.js";
import { lineUnitOf } from "../src/mvp/forms.js";
import { mvpContent } from "../server/src/mvp/content.js";
const content = mvpContent();
const unit = (id: string) => content.units.find((u) => u.id === id)!;
const P = { id: "p", name: "me", bot: false }, Q = { id: "q", name: "bot", bot: true };
const line = (side: string) => ["necromancer", "necromancer", "guardian", "robber", "fodder"].map((id, i) => lineUnitOf(unit(id), `${side}${i}`, 3));
const rec = fightLines({ player: P, line: line("a") }, { player: Q, line: line("b") }, { battleId: "sudden-death", seed: 64, kind: "round", round: 9, runId: null, at: "2026-10-07T06:00:00.000Z", content, rules: MVP_RULES });
const end = rec.log.at(-1);
if (end?.type !== "BattleEnd" || end.timeUp || end.turns < MVP_RULES.suddenDeathAt!) throw new Error(`no sudden death: ${JSON.stringify(end)}`);
if (!rec.log.some((e) => e.type === "SummonFailed")) throw new Error("no revive stopped");
writeFileSync(process.argv[2]!, JSON.stringify(rec));
console.log("sudden-death battle", rec.log.length, "events", JSON.stringify(end));
