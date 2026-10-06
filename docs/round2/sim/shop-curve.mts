// Scratch sim: bot runs in memory, fixed 5 offers vs a growing curve.
const R = new URL("../../..", import.meta.url).pathname;
const { mvpRuntime } = await import(`${R}/server/src/mvp/runtime.ts`);
const { mvpContent } = await import(`${R}/server/src/mvp/content.ts`);
const { MemoryMvpStore } = await import(`${R}/server/src/mvp/store.ts`);
const { MVP_RULES } = await import(`${R}/src/mvp/contract.ts`);
const { botDecision } = await import(`${R}/server/src/mvp/bots.ts`);
const { startRun, decide } = await import(`${R}/server/src/mvp/runs.ts`);

const content = mvpContent();
const curves: Record<string, (round: number) => number> = {
  fixed5: () => 5,
  hsbg3to6: (r) => (r >= 9 ? 6 : r >= 6 ? 5 : r >= 3 ? 4 : 3),
  grow4to7: (r) => (r >= 9 ? 7 : r >= 6 ? 6 : r >= 3 ? 5 : 4),
  fast3to6: (r) => (r >= 7 ? 6 : r >= 4 ? 5 : r >= 2 ? 4 : 3),
};
const RUNS = Number(process.argv[2] ?? 300);
for (const [name, curve] of Object.entries(curves)) {
  let s = Number(process.argv[3] ?? 12345);
  const seed = () => ((s = (Math.imul(s, 1664525) + 1013904223) >>> 0));
  const rules = { ...MVP_RULES, offers: curve(1) };
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
        run = { ...run, rules: { ...run.rules, offers: curve(run.round + 1) } };
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
