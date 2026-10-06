// Scratch sim: bot runs in memory, fixed 5 offers vs a growing curve.
// Each curve is the rules' own shape (offers in round 1, +1 at each
// offersGrowAt round; the run engine applies it, src/mvp/contract.ts
// offersAt), so nothing here adds offers on top of MVP_RULES' growth (R2-17).
//   node --import tsx/esm docs/round2/sim/shop-curve.mts [runs=300] [seed=12345]
const R = new URL("../../..", import.meta.url).pathname;
const { mvpRuntime } = await import(`${R}/server/src/mvp/runtime.ts`);
const { mvpContent } = await import(`${R}/server/src/mvp/content.ts`);
const { MemoryMvpStore } = await import(`${R}/server/src/mvp/store.ts`);
const { MVP_RULES, offersAt } = await import(`${R}/src/mvp/contract.ts`);
const { botDecision } = await import(`${R}/server/src/mvp/bots.ts`);
const { startRun, decide } = await import(`${R}/server/src/mvp/runs.ts`);

const content = mvpContent();
const curves: Record<string, { offers: number; offersGrowAt: number[] }> = {
  fixed5: { offers: 5, offersGrowAt: [] },
  hsbg3to6: { offers: 3, offersGrowAt: [3, 6, 9] },
  grow4to7: { offers: 4, offersGrowAt: [3, 6, 9] },
  // The live rules (MVP_RULES): 3, 4 from round 2, 5 from 4, 6 from 7.
  fast3to6: { offers: MVP_RULES.offers, offersGrowAt: MVP_RULES.offersGrowAt ?? [] },
};
const RUNS = Number(process.argv[2] ?? 300);
for (const [name, curve] of Object.entries(curves)) {
  let s = Number(process.argv[3] ?? 12345);
  const seed = () => ((s = (Math.imul(s, 1664525) + 1013904223) >>> 0));
  const rules = { ...MVP_RULES, ...curve };
  console.log(`${name}: offers by round ${Array.from({ length: 12 }, (_, i) => offersAt(rules, i + 1)).join(" ")}`);
  const rt = mvpRuntime({ content, store: new MemoryMvpStore(), rules, seed, now: () => new Date("2026-10-06T10:00:00Z") });
  const awokenBy: number[] = Array(13).fill(0);
  let fused = 0, rerolls = 0, buys = 0, survived = 0, goldLeft = 0, shopRounds = 0;
  for (let k = 0; k < RUNS; k++) {
    const p = { id: `b${k}`, name: `b${k}`, bot: true };
    rt.store.addPlayer(p);
    let run = startRun(rt, p);
    let round = run.round, rerolled = 0, firstAwoken = 0, didFuse = false;
    for (let g = 0; run.phase !== "over" && g < 1000; g++) {
      if (run.round !== round) { round = run.round; rerolled = 0; }
      const d = botDecision(run, content, rules, rerolled);
      if (d.kind === "reroll") { rerolled++; rerolls++; }
      if (d.kind === "buy") buys++;
      if (d.kind === "fuse") didFuse = true;
      if (d.kind === "fight" && run.phase === "shop") {
        goldLeft += run.gold; shopRounds++;
      }
      decide(rt, run, d);
      run = rt.store.run(run.runId)!;
      if (!firstAwoken && run.line.some((u: any) => u.form === "awoken")) firstAwoken = Math.min(run.round, 12);
    }
    if (firstAwoken) awokenBy[firstAwoken]++;
    if (didFuse) fused++;
    if (run.fights.filter((f: any) => f.kind === "round").length >= 12) survived++;
  }
  let cum = 0; const cumAwoken = awokenBy.map((n) => (cum += n, Math.round((100 * cum) / RUNS)));
  console.log(name, JSON.stringify({ awokenByR3: cumAwoken[3], awokenByR6: cumAwoken[6], awokenByR9: cumAwoken[9], awokenEver: cumAwoken[12], fusedPct: Math.round(100 * fused / RUNS), reached12Pct: Math.round(100 * survived / RUNS), rerollsPerRound: +(rerolls / shopRounds).toFixed(2), buysPerRound: +(buys / shopRounds).toFixed(2), goldLeftPerRound: +(goldLeft / shopRounds).toFixed(2) }));
}
