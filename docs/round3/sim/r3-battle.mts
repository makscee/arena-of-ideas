// Round-3 scout sim (notes 2, 10, 14, 19). Run with the scout checkout's tsx:
//   /Users/admin/Work/arena-574/scout-r3/node_modules/.bin/tsx r3-battle.mts [battles]
const R = "/Users/admin/Work/arena-574/scout-r3";
const { battle } = await import(`${R}/src/battle.ts`);
const { battle: battleFront } = await import("./battle-front.ts"); // summons + revives at the front
const { battle: battleFrontS } = await import("./battle-front-summon.ts"); // summons only at the front
const { MVP_RULES } = await import(`${R}/src/mvp/contract.ts`);
const { toBattleDef } = await import(`${R}/src/mvp/fight.ts`);
const { lineUnitOf } = await import(`${R}/src/mvp/forms.ts`);
const { beatPlayOf, stepsOf, timingOf, firingOf } = await import(`${R}/src/mvp/trace.ts`);
const { mvpPool } = await import(`${R}/src/mvp/units.ts`);

const pool = mvpPool();
const n = Number(process.argv[2] ?? 2000);
let seed = 7;
const rnd = () => ((seed = (seed * 1103515245 + 12345) % 2 ** 31) / 2 ** 31);
const line = (tag: string) =>
  Array.from({ length: 5 }, (_, i) => {
    const u = pool.units[Math.floor(rnd() * pool.units.length)]!;
    const copies = 1 + Math.floor(rnd() * 3); // some awoken
    return { name: u.name, l: lineUnitOf(u, `${tag}${i}`, copies) };
  });
const SUMMONERS = ["Planter", "Summoner", "Sexton", "Fungoid", "Morbid", "Necromancer", "Divinity"];
const stat: Record<string, Record<string, { w: number; n: number }>> = { base: {}, front: {}, frontS: {} };
const add = (k: string, name: string, s: number) => { const t = (stat[k]![name] ??= { w: 0, n: 0 }); t.w += s; t.n++; };
let cap = { 64: 0, 32: 0, 16: 0 }, flip32 = 0, flipFront = 0, flipFrontS = 0, summonBattles = 0;
const waveKinds: Record<string, number> = {};
let waves = 0;
const ms1x: number[] = [], beatsN: number[] = [], beatKinds = { quiet: 0, full: 0, kill: 0, big: 0, summon: 0 };
const proposed: number[] = [];
for (let k = 0; k < n; k++) {
  const a = line("a"), b = line("b");
  const input = (capN: number) => ({ teamA: a.map((x) => toBattleDef(x.l)), teamB: b.map((x) => toBattleDef(x.l)), seed: k, abilities: pool.abilities, statuses: pool.statuses, chainStepCap: capN });
  const log = battle(input(64));
  const w = log.at(-1).winner;
  const log32 = battle(input(32));
  const log16 = battle(input(16));
  const lf = battleFront(input(64));
  const lfs = battleFrontS(input(64));
  for (const [c, l] of [[64, log], [32, log32], [16, log16]] as const) if (l.some((e: any) => e.type === "ChainCapped")) (cap as any)[c]++;
  if (log32.at(-1).winner !== w) flip32++;
  if (log.some((e: any) => e.type === "Summon")) summonBattles++;
  if (lf.at(-1).winner !== w) flipFront++;
  if (lfs.at(-1).winner !== w) flipFrontS++;
  for (const [key, l] of [["base", log], ["front", lf], ["frontS", lfs]] as const) {
    const win = l.at(-1).winner;
    const sA = win === "A" ? 1 : win === "draw" ? 0.5 : 0;
    for (const nm of new Set(a.map((x) => x.name))) if (SUMMONERS.includes(nm)) add(key, nm, sA);
    for (const nm of new Set(b.map((x) => x.name))) if (SUMMONERS.includes(nm)) add(key, nm, 1 - sA);
  }
  // waves: which ones carry a unit firing (a trigger badge today)
  const steps = stepsOf(log);
  const beats = beatPlayOf(log, steps);
  beatsN.push(beats.length);
  ms1x.push(beats.reduce((t: number, pb: any) => t + timingOf(pb).ms, 0) / 1000);
  let p = 0;
  for (const pb of beats) {
    const kinds = new Set(pb.waves.flatMap((s: any) => s.changes.map((c: any) => c.kind)));
    const dmg = pb.waves.flatMap((s: any) => s.changes).filter((c: any) => c.kind === "damage").map((c: any) => log[c.eventId].amount ?? 0);
    const quiet = pb.waves.length === 1 && [...kinds].every((x) => x === "damage" || x === "status");
    const kill = kinds.has("death");
    const big = Math.max(0, ...dmg) >= 4;
    const summon = kinds.has("summon");
    if (quiet) beatKinds.quiet++; else beatKinds.full++;
    if (kill) beatKinds.kill++;
    if (big) beatKinds.big++;
    if (summon) beatKinds.summon++;
    // proposed timing: waves 260 ms apart (cap 1.6 s span), hold 900; quiet 1000; full ≥1500, ≤2600; +400 kill, +250 big, +300 summon
    const nW = pb.waves.length;
    const gap = nW > 1 ? Math.min(220, 1400 / (nW - 1)) : 0;
    let t = quiet ? 900 : Math.min(2200, Math.max(1300, (nW - 1) * gap + 800));
    if (kill) t += 350; else if (big) t += 200;
    if (summon) t += 250;
    p += t;
    for (const s of pb.waves) {
      waves++;
      const f = firingOf(log, s);
      const first = log[s.eventIds[0]];
      let kind: string;
      if (f) kind = "unit ability (badge today)";
      else if (first.source === "kernel") {
        const parent = first.causedBy !== null ? log[first.causedBy] : null;
        kind = first.type === "Hurt" && parent?.type === "Strike" ? "strike hit" : first.type === "Hurt" && parent?.type === "Fatigue" ? "fatigue" : first.type === "Death" ? "death (kernel)" : `kernel ${first.type}`;
      } else if (first.source.status) kind = `status ability (${first.source.status})`;
      else kind = `other ${first.type}`;
      waveKinds[kind] = (waveKinds[kind] ?? 0) + 1;
    }
  }
  proposed.push((p + 900) / 1000);
}
const q = (xs: number[], pp: number) => [...xs].sort((x, y) => x - y)[Math.floor((xs.length - 1) * pp)]!;
console.log(`${n} random battles (5 random pool units a side, 1-3 copies)`);
console.log(`chain capped: cap64 ${(cap[64] / n * 100).toFixed(2)}%, cap32 ${(cap[32] / n * 100).toFixed(2)}%, cap16 ${(cap[16] / n * 100).toFixed(2)}%; winner changes 64->32: ${flip32} (${(flip32 / n * 100).toFixed(2)}%)`);
console.log(`battles with a summon: ${(summonBattles / n * 100).toFixed(1)}%; winner changes with summons+revives at front: ${flipFront} (${(flipFront / n * 100).toFixed(1)}%), summons only: ${flipFrontS} (${(flipFrontS / n * 100).toFixed(1)}%)`);
console.log("unit score (base -> summons+revives front / summons-only front):");
for (const nm of SUMMONERS) {
  const s = (k: string) => { const t = stat[k]![nm]; return t ? `${(t.w / t.n * 100).toFixed(0)}%` : "-"; };
  console.log(`  ${nm.padEnd(12)} ${s("base")} -> ${s("front")} / ${s("frontS")}  (n=${stat.base![nm]?.n ?? 0})`);
}
console.log(`waves: ${waves}`);
for (const [k, v] of Object.entries(waveKinds).sort((x, y) => y[1] - x[1])) console.log(`  ${k.padEnd(34)} ${(v / waves * 100).toFixed(1)}%`);
const tb = beatKinds.quiet + beatKinds.full;
console.log(`beats: median ${q(beatsN, 0.5)}, p90 ${q(beatsN, 0.9)}; quiet ${(beatKinds.quiet / tb * 100).toFixed(0)}%, kill ${(beatKinds.kill / tb * 100).toFixed(0)}%, big hit ${(beatKinds.big / tb * 100).toFixed(0)}%, summon ${(beatKinds.summon / tb * 100).toFixed(0)}%`);
console.log(`1x time today: median ${q(ms1x, 0.5).toFixed(1)} s, p90 ${q(ms1x, 0.9).toFixed(1)} s (2x: ${(q(ms1x, 0.5) / 2).toFixed(1)} s)`);
console.log(`1x time proposed: median ${q(proposed, 0.5).toFixed(1)} s, p90 ${q(proposed, 0.9).toFixed(1)} s (2x: ${(q(proposed, 0.5) / 2).toFixed(1)} s)`);
