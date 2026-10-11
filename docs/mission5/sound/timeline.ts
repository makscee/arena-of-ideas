// The cues of one session, home → shop → a battle with a chain, as the game
// asks for them (void-board#922): render.py turns it into audio.
//   node --import tsx/esm docs/mission5/sound/timeline.ts <sound-map module> <out.json> [speed]
import { writeFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { resolve } from "node:path";
import { MVP_RULES, type MvpContent, type PlayerRef } from "../../../src/mvp/contract";
import { fightLines } from "../../../src/mvp/fight";
import { lineUnitOf } from "../../../src/mvp/forms";
import { LINEUP_MS, beatPlayOf, stepsOf, timingOf, weightsOf } from "../../../src/mvp/trace";
import { mvpPool } from "../../../src/mvp/units";

const map = await import(pathToFileURL(resolve(process.argv[2]!)).href);
const out = process.argv[3]!;
const speed = Number(process.argv[4] ?? 1);
const events: { t: number; cue: unknown }[] = [];
const at = (t: number, c: unknown) => c && events.push({ t, cue: c });

// Home 6 s, then the shop: taps, buys, a merge, a third copy that awakens,
// a reroll, a lock, a reorder, a sell.
const HOME = 6000;
let t = HOME;
const shop: [number, string][] = [[0, "click"], [600, "coin"], [1500, "click"], [1900, "coin"], [2800, "merge"], [3800, "click"], [4100, "level-up"],
  [5200, "reroll"], [6000, "freeze"], [6700, "click"], [7000, "move-left"], [7900, "sell"], [8600, "click"]];
for (const [dt, k] of shop) at(t + dt, map.cue(k));
// A fuse that discovers a new combination: fuse and discover fire together.
at(t + 9400, map.cue("fuse"));
at(t + 9420, map.cue("discover"));
t += 12500;
const SHOP = t;

// The battle: the first seeded fight with a beat of 4+ waves and a death.
const pool = mvpPool();
const content: MvpContent = { version: "t", units: pool.units, abilities: pool.abilities, statuses: pool.statuses };
const p = (id: string): PlayerRef => ({ id, name: id, bot: false });
let r = 4242;
const rand = (n: number) => ((r = (r * 1103515245 + 12345) % 2147483648), r % n);
for (let seed = 0; ; seed++) {
  const line = (s: string) => Array.from({ length: 5 }, (_, k) => lineUnitOf(pool.units[rand(pool.units.length)]!, `${s}${k}`, 1 + rand(4)));
  const { log } = fightLines({ player: p("a"), line: line("a") }, { player: p("b"), line: line("b") }, { battleId: "x", seed, kind: "round", round: 5, runId: null, at: "2026-10-05T00:00:00Z", content, rules: MVP_RULES });
  const beats = beatPlayOf(log, stepsOf(log));
  if (!beats.some((b) => b.waves.length >= 5) || !log.some((e) => e.type === "Death")) continue;
  const weights = weightsOf(log, beats);
  at(t, map.cue("start"));
  t += LINEUP_MS / speed;
  beats.forEach((b, i) => {
    const cues = map.beatCues(b, log);
    const tm = timingOf(b, weights[i]);
    b.waves.forEach((_, w) => at(t + tm.at[w]! / speed, cues[w]));
    t += tm.ms / speed;
  });
  const won = log.findLast((e) => e.type === "BattleEnd") as { winner?: string } | undefined;
  for (const c of map.endSound(won?.winner === "A" ? "win" : won?.winner === "B" ? "loss" : "draw", "round")) at(t + (c.delay ?? 0), c);
  writeFileSync(out, JSON.stringify({ speed, seed, home: [0, HOME], shop: [HOME, SHOP], battle: [SHOP, t + 3500], waves: Math.max(...beats.map((b) => b.waves.length)), events }));
  console.log(`seed ${seed}: ${beats.length} beats, ${events.length} cues, ${((t + 3500) / 1000).toFixed(1)} s`);
  break;
}
