// Fusion stats: measure today's fused bodies, then compare formulas in fights.
//   node --import tsx/esm fusion.mts [runs=120]
const R = "/Users/admin/Work/arena-574/scout-r3";
const { mvpRuntime } = await import(`${R}/server/src/mvp/runtime.ts`);
const { mvpContent } = await import(`${R}/server/src/mvp/content.ts`);
const { MemoryMvpStore } = await import(`${R}/server/src/mvp/store.ts`);
const { MVP_RULES } = await import(`${R}/src/mvp/contract.ts`);
const { botDecision } = await import(`${R}/server/src/mvp/bots.ts`);
const { startRun, decide } = await import(`${R}/server/src/mvp/runs.ts`);
const { fightLines } = await import(`${R}/src/mvp/fight.ts`);
const { lineUnitOf } = await import(`${R}/src/mvp/forms.ts`);

const t0 = Date.now();
const content = mvpContent();
const byId = new Map(content.units.map((u: any) => [u.id, u]));
const rules = MVP_RULES;
const aw = (id: string) => { const u: any = byId.get(id); return { pwr: u.base.pwr + 2, hp: u.base.hp + 4 }; };

// 1. static: awoken by tier, fused (sum) over all ordered pairs
const tiers: Record<number, { p: number; h: number; n: number }> = {};
for (const u of content.units as any[]) { const a = aw(u.id); const t = (tiers[u.tier] ??= { p: 0, h: 0, n: 0 }); t.p += a.pwr; t.h += a.hp; t.n++; }
console.log("awoken@3 avg by tier:", Object.entries(tiers).map(([t, v]) => `T${t} ${(v.p / v.n).toFixed(1)}/${(v.h / v.n).toFixed(1)}`).join("  "));

// 2. bot runs: lines from round 9 on
const RUNS = Number(process.argv[2] ?? 120);
let s = 12345; const seed = () => ((s = (Math.imul(s, 1664525) + 1013904223) >>> 0));
const rt = mvpRuntime({ content, store: new MemoryMvpStore(), rules, seed, now: () => new Date("2026-10-06T10:00:00Z") });
const lines: any[][] = []; const noFuse: any[][] = [];
const fusedAt12: any[] = []; const awokenAt12: any[] = [];
let fusedRuns = 0;
for (let k = 0; k < 2 * RUNS; k++) {
  const fuses = k < RUNS;
  const p = { id: `b${k}`, name: `b${k}`, bot: true }; rt.store.addPlayer(p);
  let run = startRun(rt, p); let round = run.round, rr = 0, did = false;
  for (let g = 0; run.phase !== "over" && g < 1000; g++) {
    if (run.round !== round) { round = run.round; rr = 0; }
    const d = botDecision(run, content, rules, rr, () => fuses);
    if (d.kind === "reroll") rr++;
    if (d.kind === "fuse") did = true;
    if (d.kind === "fight" && run.phase === "shop" && run.round >= 9) (fuses ? lines : noFuse).push(Object.assign(structuredClone(run.line), { round: run.round }));
    if (fuses && d.kind === "fight" && run.phase === "shop" && run.round === 12) for (const u of run.line) (u.kind === "fused" ? fusedAt12 : u.form === "awoken" ? awokenAt12 : []).push(u);
    decide(rt, run, d); run = rt.store.run(run.runId)!;
  }
  if (fuses && did) fusedRuns++;
}
const avg = (xs: any[], f: (u: any) => number) => xs.length ? (xs.reduce((a, u) => a + f(u), 0) / xs.length).toFixed(1) : "-";
const pct = (xs: number[], q: number) => { const a = [...xs].sort((x, y) => x - y); return a[Math.floor(q * (a.length - 1))]; };
console.log(`bot runs ${RUNS}: fused in ${Math.round(100 * fusedRuns / RUNS)}%; ${((Date.now() - t0) / 1000).toFixed(1)}s`);
console.log(`round-12 fused units n=${fusedAt12.length}: avg ${avg(fusedAt12, (u) => u.stats.pwr)}/${avg(fusedAt12, (u) => u.stats.hp)} copies ${avg(fusedAt12, (u) => u.copies)}  hp p50 ${pct(fusedAt12.map((u) => u.stats.hp), 0.5)} p90 ${pct(fusedAt12.map((u) => u.stats.hp), 0.9)} max ${pct(fusedAt12.map((u) => u.stats.hp), 1)}`);
console.log(`round-12 awoken units n=${awokenAt12.length}: avg ${avg(awokenAt12, (u) => u.stats.pwr)}/${avg(awokenAt12, (u) => u.stats.hp)} copies ${avg(awokenAt12, (u) => u.copies)}`);

// 3. formulas: rebuild fused stats from parts (assume 3+3 copies at the fuse, the rest merged after)
type F = (a: { pwr: number; hp: number }, b: { pwr: number; hp: number }) => { pwr: number; hp: number };
const FORMULAS: Record<string, { f: F; grow: { pwr: number; hp: number } }> = {
  "sum (today)": { f: (a, b) => ({ pwr: a.pwr + b.pwr, hp: a.hp + b.hp }), grow: { pwr: 1, hp: 2 } },
  "max+1/+2": { f: (a, b) => ({ pwr: Math.max(a.pwr, b.pwr) + 1, hp: Math.max(a.hp, b.hp) + 2 }), grow: { pwr: 1, hp: 2 } },
  "max + half min": { f: (a, b) => ({ pwr: Math.max(a.pwr, b.pwr) + Math.floor(Math.min(a.pwr, b.pwr) / 2), hp: Math.max(a.hp, b.hp) + Math.floor(Math.min(a.hp, b.hp) / 2) }), grow: { pwr: 1, hp: 2 } },
  "sum pwr, max+2 hp": { f: (a, b) => ({ pwr: a.pwr + b.pwr, hp: Math.max(a.hp, b.hp) + 2 }), grow: { pwr: 1, hp: 2 } },
};
function restat(line: any[], name: string): any[] {
  const { f, grow } = FORMULAS[name]!;
  return line.map((u) => {
    if (u.kind !== "fused") return u;
    const extra = Math.max(0, u.copies - 6);
    const st = f(aw(u.fusion.first), aw(u.fusion.second));
    return { ...u, stats: { pwr: st.pwr + extra * grow.pwr, hp: st.hp + extra * grow.hp } };
  });
}
const withF = lines.filter((l) => l.some((u) => u.kind === "fused"));
const without = noFuse;
console.log(`lines r9+: ${lines.length}, with a fused unit ${withF.length}, full lines without ${without.length}`);
const P = { id: "sim", name: "sim", bot: true }; const AT = new Date(0).toISOString();
const N = Math.min(500, withF.length * 3);
for (const name of Object.keys(FORMULAS)) {
  let wins = 0, games = 0, turns = 0, frontFused = 0;
  const hp: number[] = [];
  let r = 777; const rnd = () => ((r = (Math.imul(r, 1664525) + 1013904223) >>> 0) / 2 ** 32);
  for (let i = 0; i < N; i++) {
    const a = restat(withF[Math.floor(rnd() * withF.length)]!, name);
    const same = without.filter((l: any) => l.round === (a as any).round || true);
    const b = same[Math.floor(rnd() * same.length)]!;
    for (const u of a) if (u.kind === "fused") hp.push(u.stats.hp);
    if (a[0].kind === "fused") frontFused++;
    const sideA = rnd() < 0.5;
    const rec = fightLines({ player: P, line: sideA ? a : b }, { player: P, line: sideA ? b : a }, { battleId: "x", seed: i + 1, kind: "round", round: 12, runId: null, at: AT, content, rules });
    const w = rec.winner; const mine = sideA ? "A" : "B";
    wins += w === mine ? 1 : w === "draw" ? 0.5 : 0; games++;
    turns += (rec.log.at(-1) as any).turns;
  }
  console.log(`${name.padEnd(18)} fused-line win ${(100 * wins / games).toFixed(0)}%  fused hp p50 ${pct(hp, 0.5)} p90 ${pct(hp, 0.9)}  avg turns ${(turns / games).toFixed(1)}  (fused in front ${Math.round(100 * frontFused / N)}%)`);
}
console.log(`total ${((Date.now() - t0) / 1000).toFixed(1)}s`);
