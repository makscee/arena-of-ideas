// A real fight that opens on a long Start chain (#913), like a strong
// champion's: random lines of the shipped units, the longest first beat of
// the first fights tried. Writes the BattleRecord to argv[2].
// Used by e2e/probe-chain-read.mjs.
import { writeFileSync } from "node:fs";
import { MVP_RULES } from "../src/mvp/contract.js";
import { fightLines } from "../src/mvp/fight.js";
import { lineUnitOf } from "../src/mvp/forms.js";
import { beatPlayOf, stepsOf } from "../src/mvp/trace.js";
import { mvpContent } from "../server/src/mvp/content.js";
const content = mvpContent();
const P = { id: "p", name: "me", bot: false }, Q = { id: "q", name: "champ", bot: true };
let r = 913;
const rnd = (n: number) => { r = (r * 1103515245 + 12345) % 2147483648; return r % n; };
let best: { n: number; rec: unknown } | null = null;
for (let i = 0; i < 1500; i++) {
  const pick = (p: string) => Array.from({ length: 5 }, (_, k) => lineUnitOf(content.units[rnd(content.units.length)]!, `${p}${k}`, 1 + rnd(4)));
  const rec = fightLines({ player: P, line: pick("a") }, { player: Q, line: pick("b") }, { battleId: "chain", seed: i, kind: "crown", round: 5, runId: null, at: "2026-10-11T00:00:00.000Z", content, rules: MVP_RULES });
  if (rec.log.some((e) => e.type === "ChainCapped")) continue;
  const beats = beatPlayOf(rec.log, stepsOf(rec.log));
  const n = beats[0]?.waves.length ?? 0;
  if (beats.length > 6 && n <= 24 && (!best || n > best.n)) best = { n, rec };
}
if (!best) { console.log("none"); process.exit(1); }
writeFileSync(process.argv[2]!, JSON.stringify(best.rec));
console.log("start chain", best.n, "steps");
