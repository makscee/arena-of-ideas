// #918: a real battle where a revive lands mid-line. The front Squire falls
// first and Necromancer raises it, so it returns to the front, not behind
// Necromancer. Writes the BattleRecord to argv[2], with `expect`: the unit
// ids of side A's line, front first, right after the first revive, for the
// probe to compare with the board. Used by e2e/probe-revive-place.mjs.
import { writeFileSync } from "node:fs";
import { boardAt } from "../src/board.js";
import { MVP_RULES } from "../src/mvp/contract.js";
import { fightLines } from "../src/mvp/fight.js";
import { lineUnitOf } from "../src/mvp/forms.js";
import { mvpContent } from "../server/src/mvp/content.js";
const content = mvpContent();
const unit = (id: string) => content.units.find((u) => u.id === id)!;
const P = { id: "p", name: "me", bot: false }, Q = { id: "q", name: "bot", bot: true };
const a = [lineUnitOf(unit("squire"), "a1", 1), lineUnitOf(unit("squire"), "a2", 2), lineUnitOf(unit("necromancer"), "a3", 1)];
const b = [{ ...lineUnitOf(unit("squire"), "b0", 3), stats: { pwr: 6, hp: 80 } }];
const rec = fightLines({ player: P, line: a }, { player: Q, line: b }, { battleId: "revive-place", seed: 1, kind: "round", round: 3, runId: null, at: "2026-10-11T00:00:00.000Z", content, rules: MVP_RULES });
const i = rec.log.findIndex((e) => e.type === "Summon" && e.resurrected && e.side === "A");
const rev = rec.log[i];
if (!rev || rev.type !== "Summon" || rev.slot !== 0) { console.log(`no revive to the front: ${JSON.stringify(rev)}`); process.exit(1); }
const expect = boardAt(rec.log, i).lines.A.map((u) => u.id);
writeFileSync(process.argv[2]!, JSON.stringify({ ...rec, expect }));
console.log(`revive-place battle: ${rev.unit} back at slot ${rev.slot}, line ${expect.join(", ")}`);
