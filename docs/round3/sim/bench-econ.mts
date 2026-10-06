// Scratch econ sim (no battles, all 12 rounds): what a bench of N and freeze
// change in the shop. Greedy human-ish policy: fuse 2 Awoken, buy copies,
// fill with the best tier, else (bench) stash a higher tier, else swap the
// weakest single; reroll leftover gold hunting copies. Freeze (opt): a copy
// you can't afford stays for next round.
const R = "/Users/admin/Work/arena-574/scout-r3";
const { mvpContent } = await import(`${R}/server/src/mvp/content.ts`);
const { MVP_RULES, offersAt } = await import(`${R}/src/mvp/contract.ts`);
const content = mvpContent();
const rules = MVP_RULES;
type U = { id: string; tier: number; copies: number; fused: boolean; pwr: number; hp: number; parts: string[] };
function rng(seed: number) { let s = seed >>> 0; return () => ((s = (Math.imul(s, 1664525) + 1013904223) >>> 0) / 2 ** 32); }
const score = (u: U) => u.pwr * 2 + u.hp + (u.copies >= 3 ? 6 : 0) + (u.fused ? 10 : 0);
const awoken = (u: U) => !u.fused && u.copies >= 3;
function sim(bench: number, freeze: boolean, seed: number) {
  const r = rng(seed);
  let line: U[] = [], store: U[] = [];
  let frozen: string[] = [];
  const st = { awakenings: 0, fusions: 0, lateActionable: 0, lateOffers: 0, lateCopyBuys: 0, lateNewBuys: 0, rerolls: 0, wasted: 0, freezes: 0 };
  for (let round = 1; round <= 12; round++) {
    let gold = rules.goldPerRound;
    const open = content.units.filter((u: any) => rules.tierOpensAt[u.tier - 1] <= round);
    const roll = (keep: string[]) => { const n = offersAt(rules, round) - keep.length; return [...keep, ...Array.from({ length: Math.max(0, n) }, () => open[Math.floor(r() * open.length)].id)]; };
    let offers = roll(frozen); frozen = [];
    const all = () => [...line, ...store];
    const owner = (id: string) => all().find((u) => u.parts.includes(id));
    const tierOf = (id: string) => content.units.find((u: any) => u.id === id).tier;
    const base = (id: string) => content.units.find((u: any) => u.id === id).base;
    let firstLook = true;
    for (let guard = 0; guard < 60; guard++) {
      // fuse
      const aw = all().filter(awoken).sort((a, b) => score(b) - score(a));
      if (aw.length >= 2) {
        const [a, b] = aw; const f: U = { id: a.id, tier: Math.max(a.tier, b.tier), copies: a.copies + b.copies, fused: true, pwr: a.pwr + b.pwr, hp: a.hp + b.hp, parts: [...a.parts, ...b.parts] };
        line = line.filter((x) => x !== a && x !== b); store = store.filter((x) => x !== a && x !== b);
        (line.length < rules.lineSize ? line : store).push(f); st.fusions++; continue;
      }
      // fill the line from the bench
      while (line.length < rules.lineSize && store.length) { store.sort((a, b) => score(b) - score(a)); line.push(store.shift()!); }
      if (firstLook && round >= 7) {
        firstLook = false;
        st.lateOffers += offers.length;
        st.lateActionable += offers.filter((id) => owner(id) || line.length < rules.lineSize || store.length < bench || line.some((u) => u.copies === 1 && !u.fused && u.tier < tierOf(id))).length;
      }
      const copyAt = offers.findIndex((id) => owner(id));
      if (copyAt >= 0 && gold >= 3) {
        const u = owner(offers[copyAt])!; u.copies++; u.pwr += 1; u.hp += 2; if (u.copies === 3 && !u.fused) st.awakenings++;
        offers.splice(copyAt, 1); gold -= 3; if (round >= 7) st.lateCopyBuys++; continue;
      }
      if (copyAt >= 0 && freeze && gold < 3) { frozen.push(offers[copyAt]); offers.splice(copyAt, 1); st.freezes++; continue; }
      const best = offers.map((id, i) => ({ id, i, t: tierOf(id) })).sort((a, b) => b.t - a.t)[0];
      const add = () => { const b = base(best.id); const u: U = { id: best.id, tier: best.t, copies: 1, fused: false, pwr: b.pwr, hp: b.hp, parts: [best.id] }; offers.splice(best.i, 1); gold -= 3; if (round >= 7) st.lateNewBuys++; return u; };
      if (best && gold >= 3 && line.length < rules.lineSize) { line.push(add()); continue; }
      // bench: stash a higher tier than the weakest single (or any, while it has room and we're early)
      const weak = [...line, ...store].filter((u) => u.copies === 1 && !u.fused).sort((a, b) => a.tier - b.tier || score(a) - score(b))[0];
      if (best && gold >= 3 && store.length < bench && (!weak || best.t >= weak.tier)) { store.push(add()); continue; }
      if (best && gold >= 3 && weak && weak.tier < best.t) { line = line.filter((x) => x !== weak); store = store.filter((x) => x !== weak); gold += 1; continue; }
      if (gold >= 1 + (gold >= 4 ? 0 : 0)) { gold -= 1; st.rerolls++; offers = roll(frozen.splice(0)); continue; }
      break;
    }
    st.wasted += gold;
    // field the best 5
    const pool = [...line, ...store].sort((a, b) => score(b) - score(a)); line = pool.slice(0, rules.lineSize); store = pool.slice(rules.lineSize);
  }
  return { ...st, power: line.reduce((s, u) => s + u.pwr * 2 + u.hp, 0) };
}
const N = 2000;
for (const [bench, freeze] of [[0, false], [0, true], [2, false], [3, false], [3, true]] as const) {
  const tot: any = {};
  for (let k = 0; k < N; k++) for (const [key, v] of Object.entries(sim(bench, freeze, 1000 + k))) tot[key] = (tot[key] ?? 0) + v;
  const out: any = {}; for (const [key, v] of Object.entries(tot)) out[key] = +((v as number) / N).toFixed(2);
  out.lateActionablePct = Math.round(100 * tot.lateActionable / tot.lateOffers);
  delete out.lateActionable; delete out.lateOffers;
  console.log(`bench ${bench}${freeze ? " +freeze" : ""}`, JSON.stringify(out));
}
