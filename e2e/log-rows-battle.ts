// A long late-game battle (R4-3): the most Log waves among random lines of the
// shipped units with awoken units and fusions. Writes the BattleRecord to
// argv[2] and prints its waves, Log rows and folded turns. Used by
// e2e/probe-log-rows.mjs. With argv[4] "loop", the fight with the most
// folded turns (R4-13) instead: e2e/probe-log-fold.mjs.
import { writeFileSync } from "node:fs";
import { MVP_RULES } from "../src/mvp/contract.js";
import { fightLines } from "../src/mvp/fight.js";
import { fuseUnits, lineUnitOf } from "../src/mvp/forms.js";
import { beatPlayOf, foldTurnsOf, isLogFold, logRowsOf, sidesOf, stepsOf } from "../src/mvp/trace.js";
import { mvpContent } from "../server/src/mvp/content.js";
const content = mvpContent();
const P = { id: "p", name: "me", bot: false }, Q = { id: "q", name: "bot", bot: true };
let r = 12345;
const rnd = (n: number) => { r = (r * 1103515245 + 12345) % 2147483648; return r % n; };
const unit = () => content.units[rnd(content.units.length)]!;
const pick = (p: string) =>
  Array.from({ length: 5 }, (_, k) => {
    if (rnd(5) < 2) {
      const a = lineUnitOf(unit(), `${p}${k}`, 3), b = lineUnitOf(unit(), `${p}${k}b`, 3);
      try { return fuseUnits(a, b, { name: `${a.name}${b.name}`, discoveredBy: null }, content); } catch { return a; }
    }
    return lineUnitOf(unit(), `${p}${k}`, 1 + rnd(3));
  });
const loop = process.argv[4] === "loop";
let best: { rec: ReturnType<typeof fightLines>; waves: number; rows: number; items: number; folds: number } | null = null;
for (let i = 0; i < Number(process.argv[3] ?? 200); i++) {
  const rec = fightLines({ player: P, line: pick("a") }, { player: Q, line: pick("b") }, { battleId: "long", seed: i, kind: "playoff", round: 0, runId: null, at: "2026-10-07T19:00:00.000Z", content, rules: MVP_RULES });
  // Count as the viewer does: it names units by id (two Treants are two names),
  // from your side.
  const byId = (id: string) => id;
  const beats = beatPlayOf(rec.log, stepsOf(rec.log, byId, sidesOf(rec.log), { you: "A" }), byId);
  const waves = beats.reduce((s, b) => s + b.waves.length, 0);
  const rows = logRowsOf(rec.log, beats, byId);
  const items = foldTurnsOf(rec.log, rows, byId);
  const folds = items.filter(isLogFold).length;
  if (!best || (loop ? folds > best.folds : waves > best.waves)) best = { rec, waves, rows: rows.length, items: items.length, folds };
}
writeFileSync(process.argv[2]!, JSON.stringify(best!.rec));
console.log(JSON.stringify({ events: best!.rec.log.length, waves: best!.waves, rows: best!.rows, items: best!.items, folds: best!.folds }));
