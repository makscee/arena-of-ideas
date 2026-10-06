// Scratch: today's bot runs, what a late shop round looks like (line full?).
const R = "/Users/admin/Work/arena-574/scout-r3";
const { mvpRuntime } = await import(`${R}/server/src/mvp/runtime.ts`);
const { mvpContent } = await import(`${R}/server/src/mvp/content.ts`);
const { MemoryMvpStore } = await import(`${R}/server/src/mvp/store.ts`);
const { MVP_RULES } = await import(`${R}/src/mvp/contract.ts`);
const { botDecision } = await import(`${R}/server/src/mvp/bots.ts`);
const { startRun, decide } = await import(`${R}/server/src/mvp/runs.ts`);
const { mergeTarget } = await import(`${R}/src/mvp/forms.ts`);
const content = mvpContent();
const rules = MVP_RULES;
const tiers = [1,2,3,4].map(t => content.units.filter((u: any) => u.tier === t).length);
console.log("units per tier", tiers);
let s = 777; const seed = () => ((s = (Math.imul(s, 1664525) + 1013904223) >>> 0));
const rt = mvpRuntime({ content, store: new MemoryMvpStore(), rules, seed, now: () => new Date("2026-10-06T10:00:00Z") });
const RUNS = Number(process.argv[2] ?? 200);
type Agg = { rounds: number; full: number; copyBuys: number; newBuys: number; sells: number; rerolls: number; goldLeft: number; blockedOffers: number; offersSeen: number; fused: number; awoken: number; sleeping: number };
const by: Agg[] = Array.from({ length: 13 }, () => ({ rounds: 0, full: 0, copyBuys: 0, newBuys: 0, sells: 0, rerolls: 0, goldLeft: 0, blockedOffers: 0, offersSeen: 0, fused: 0, awoken: 0, sleeping: 0 }));
const t0 = Date.now();
for (let k = 0; k < RUNS; k++) {
  const p = { id: `b${k}`, name: `b${k}`, bot: true };
  rt.store.addPlayer(p);
  let run = startRun(rt, p);
  let round = run.round, rerolled = 0, seenRound = 0;
  for (let g = 0; run.phase !== "over" && g < 1000; g++) {
    if (run.round !== round) { round = run.round; rerolled = 0; }
    if (run.phase === "shop" && seenRound !== run.round) {
      seenRound = run.round;
      const a = by[run.round]!;
      a.rounds++;
    }
    const a = by[Math.min(run.round, 12)]!;
    const d = botDecision(run, content, rules, rerolled);
    if (run.phase === "shop") {
      if (d.kind === "reroll") { rerolled++; a.rerolls++; }
      if (d.kind === "sell") a.sells++;
      if (d.kind === "buy") { const o = run.offers[d.slot]; if (mergeTarget(run.line, o.unitId) >= 0) a.copyBuys++; else a.newBuys++; }
      if (d.kind === "fight") {
        a.goldLeft += run.gold;
        const full = run.line.length >= rules.lineSize;
        if (full) a.full++;
        a.offersSeen += run.offers.length;
        if (full) a.blockedOffers += run.offers.filter((o: any) => mergeTarget(run.line, o.unitId) < 0).length;
        for (const u of run.line) { if (u.kind === "fused") a.fused++; else if (u.form === "awoken") a.awoken++; else a.sleeping++; }
      }
    }
    decide(rt, run, d);
    run = rt.store.run(run.runId)!;
  }
}
console.log(`runs ${RUNS} in ${((Date.now() - t0) / 1000).toFixed(1)}s`);
console.log("round shops full% copyBuys newBuys sells rerolls goldLeft blocked/offers line(sleep/awoken/fused)");
for (let r = 1; r <= 12; r++) {
  const a = by[r]!; const n = a.rounds || 1;
  console.log([r, a.rounds, Math.round(100 * a.full / n) + "%", (a.copyBuys / n).toFixed(2), (a.newBuys / n).toFixed(2), (a.sells / n).toFixed(2), (a.rerolls / n).toFixed(2), (a.goldLeft / n).toFixed(1), a.offersSeen ? (a.blockedOffers / a.offersSeen * 100).toFixed(0) + "%" : "-", `${(a.sleeping / n).toFixed(1)}/${(a.awoken / n).toFixed(1)}/${(a.fused / n).toFixed(1)}`].join("\t"));
}
