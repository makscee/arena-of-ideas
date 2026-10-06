// R3-15 sim check: bot runs with the awakening gift (rules.giftChoices 3)
// against the same bots without it (giftChoices 0), same seeds, on the
// server's own runtime (the in-process world `npm run mvp:server` serves).
// Gifts per run, picks and skips, and fight win rates: of the two worlds, and
// within the gift world of runs that took a gift against runs that didn't.
// Run: node --import tsx/esm docs/round3/sim/gift-bots.mts [runs]
import { MVP_RULES, type MvpRules } from "../../../src/mvp/contract.ts";
import { playBotRun, seedChampion } from "../../../server/src/mvp/bots.ts";
import { mvpContent } from "../../../server/src/mvp/content.ts";
import { mvpRuntime } from "../../../server/src/mvp/runtime.ts";

const RUNS = Number(process.argv[2] ?? 300);
const content = mvpContent();

type Tally = { runs: number; fights: number; wins: number; crowns: number; slays: number };
const tally = (): Tally => ({ runs: 0, fights: 0, wins: 0, crowns: 0, slays: 0 });

async function sim(rules: MvpRules) {
  let n = 7;
  const picks = new Map<string, number>();
  let gifts = 0, picked = 0, skipped = 0;
  const rt = mvpRuntime({
    content,
    rules,
    seed: () => (n = (n * 1103515245 + 12345) >>> 0),
    hooks: [{
      onDecision: (before, d) => {
        if (d.kind !== "gift") return;
        gifts++;
        if (d.pick === null) skipped++;
        else (picked++, picks.set(before.runId, (picks.get(before.runId) ?? 0) + 1));
      },
    }],
  });
  await seedChampion(rt);
  const all = tally(), withGift = tally(), without = tally();
  for (let i = 0; i < RUNS; i++) {
    const run = playBotRun(rt);
    for (const t of [all, picks.has(run.runId) ? withGift : without]) {
      t.runs++;
      const rounds = run.fights.filter((f) => f.kind === "round");
      t.fights += rounds.length;
      t.wins += rounds.filter((f) => f.outcome === "win").length;
      if (run.endedBy === "crown-won" || run.endedBy === "crown-lost") t.crowns++;
      if (run.endedBy === "crown-won") t.slays++;
    }
  }
  const pct = (a: number, b: number) => (b ? `${((100 * a) / b).toFixed(1)}%` : "–");
  const row = (t: Tally) => ({ runs: t.runs, roundWinRate: pct(t.wins, t.fights), fightsPerRun: (t.fights / Math.max(1, t.runs)).toFixed(2), reachedCrown: pct(t.crowns, t.runs), slew: pct(t.slays, t.runs) });
  return { gifts: { giftsPerRun: (gifts / RUNS).toFixed(2), pickedPerRun: (picked / RUNS).toFixed(2), skippedPerRun: (skipped / RUNS).toFixed(2), runsWithAPick: pct(picks.size, RUNS) }, all: row(all), withGift: row(withGift), without: row(without) };
}

// Each world plays its own bots' ghosts, so the two worlds' win rates sit near
// 50% by construction; the split inside the gift world shows the gift's edge.
const off = await sim({ ...MVP_RULES, giftChoices: 0 });
const on = await sim(MVP_RULES);
console.log(`${RUNS} bot runs per world, same seeds; each world seeds its own day-1 champion`);
console.table({ "gift world": on.gifts });
console.table({ "no gift (whole world)": off.all, "gift (whole world)": on.all, "gift world: runs that took a gift": on.withGift, "gift world: runs that took none": on.without });
