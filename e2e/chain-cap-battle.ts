// A real capped battle (R3-23): random lines of the shipped units until a
// cascade hits MVP_RULES.chainStepCap. Writes the BattleRecord to argv[2].
// Used by e2e/probe-chain-cap.mjs.
import { writeFileSync } from "node:fs";
import { MVP_RULES } from "../src/mvp/contract.js";
import { fightLines } from "../src/mvp/fight.js";
import { lineUnitOf } from "../src/mvp/forms.js";
import { mvpContent } from "../server/src/mvp/content.js";
const content = mvpContent();
const P = { id: "p", name: "me", bot: false }, Q = { id: "q", name: "bot", bot: true };
let r = 12345;
const rnd = (n: number) => { r = (r * 1103515245 + 12345) % 2147483648; return r % n; };
for (let i = 0; i < 200000; i++) {
  const pick = (p: string) => Array.from({ length: 5 }, (_, k) => lineUnitOf(content.units[rnd(content.units.length)]!, `${p}${k}`, 1 + rnd(4)));
  const rec = fightLines({ player: P, line: pick("a") }, { player: Q, line: pick("b") }, { battleId: "capped", seed: i, kind: "playoff", round: 0, runId: null, at: "2026-10-06T19:00:00.000Z", content, rules: MVP_RULES });
  const c = rec.log.find((e) => e.type === "ChainCapped");
  if (c?.type === "ChainCapped" && c.steps !== MVP_RULES.chainStepCap) throw new Error(`capped at ${c.steps}, rules say ${MVP_RULES.chainStepCap}`);
  if (c) { writeFileSync(process.argv[2]!, JSON.stringify(rec)); console.log("found", i, JSON.stringify(c)); process.exit(0); }
}
console.log("none");
process.exit(1);
