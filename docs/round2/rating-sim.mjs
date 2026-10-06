// Arena MVP rating: current formula vs candidates. One file, runs in seconds.
// Model: each player has a fixed true skill t (Elo points). A run's team is
// t + luck (luck ~ N(0, LUCK) per run, so a run's team can be good or bad).
// 12 rounds vs ghosts picked at random from the round's newest 200 saved teams
// (any rating, like pickGhost), 5 hearts, draws cost no heart, a Crown fight vs
// the champion after round 12. P(win) = logistic on the 400 scale; DRAW of
// fights are draws. Bots: fixed rating, never move, their ghosts sit in the pool.
const LUCK = 80, DRAW = 0.05, ROUNDS = 12, HEARTS = 5, POOL = 200;
let seed = 12345;
const rnd = () => { seed = (seed * 1664525 + 1013904223) >>> 0; return seed / 4294967296; };
const gauss = () => Math.sqrt(-2 * Math.log(rnd() + 1e-12)) * Math.cos(2 * Math.PI * rnd());
const pWin = (a, b) => 1 / (1 + 10 ** ((b - a) / 400));
const play = (a, b) => { if (rnd() < DRAW) return 0.5; return rnd() < pWin(a, b) ? 1 : 0; };

// ---------- rating methods ----------
// Each: start() -> state; delta(state, fights, ctx) -> new state; show(state) -> number
// fights: [{s: 0|0.5|1, opp: {r, rd}, crown?}], ctx: {slay, abandoned, forfeits:[opp...]}
const elo = (r, o) => 1 / (1 + 10 ** ((o - r) / 400));
const M = {};
M.current = { // live code: share of round fights vs field 1000, K=32, +0.25 slay
  start: () => ({ r: 1000, n: 0 }),
  update(st, f, ctx) {
    const rounds = f.filter((x) => !x.crown);
    const share = rounds.length ? rounds.reduce((a, x) => a + x.s, 0) / rounds.length : 0;
    const actual = Math.min(1, Math.max(0, share + (ctx.slay ? 0.25 : 0)));
    return { r: Math.round(st.r + 32 * (actual - elo(st.r, 1000))), n: st.n + 1 };
  },
};
M.haldane = { // current, with the stop-rule bias fixed: losses-1 / fights-1 when hearts ran out
  start: () => ({ r: 1000, n: 0 }),
  update(st, f, ctx) {
    const rounds = f.filter((x) => !x.crown);
    const pts = rounds.reduce((a, x) => a + x.s, 0);
    let share = rounds.length ? pts / rounds.length : 0;
    if (ctx.outOfHearts && rounds.length > 1) share = (rounds.length - HEARTS - 0 >= 0 ? (pts) / (rounds.length - 1) : share);
    const actual = Math.min(1, Math.max(0, share + (ctx.slay ? 0.25 : 0)));
    return { r: Math.round(st.r + 32 * (actual - elo(st.r, 1000))), n: st.n + 1 };
  },
};
const perFight = (kOf) => ({ // Elo per fight vs the ghost's rating, summed once per run
  start: () => ({ r: 1000, n: 0 }),
  update(st, f, ctx) {
    const K = kOf(st.n);
    let d = 0;
    for (const x of f) d += x.s - elo(st.r, x.opp.r);
    for (const o of ctx.forfeits) d += 0 - elo(st.r, o.r);
    return { r: Math.round(st.r + K * d), n: st.n + 1 };
  },
});
M.fight16 = perFight(() => 16);
M.fight10 = perFight(() => 10);
M.fightSteps = perFight((n) => (n < 5 ? 32 : n < 15 ? 16 : 10));
// Glicko-2, one run = one rating period (Glickman, glicko2.pdf), scale centred on 1000.
const Q = 173.7178, TAU = 0.5;
M.glicko2 = {
  start: () => ({ r: 1000, rd: 350, vol: 0.06, n: 0 }),
  update(st, f, ctx) {
    const mu = (st.r - 1000) / Q, phi = st.rd / Q, sig = st.vol;
    const games = [...f.map((x) => [x.s, x.opp]), ...ctx.forfeits.map((o) => [0, o])];
    if (!games.length) return st;
    let vinv = 0, sum = 0;
    for (const [s, o] of games) {
      const mj = (o.r - 1000) / Q, pj = (o.rd ?? 60) / Q;
      const g = 1 / Math.sqrt(1 + 3 * pj * pj / Math.PI ** 2);
      const E = 1 / (1 + Math.exp(-g * (mu - mj)));
      vinv += g * g * E * (1 - E); sum += g * (s - E);
    }
    const v = 1 / vinv, delta = v * sum, a = Math.log(sig * sig);
    const fx = (x) => { const ex = Math.exp(x); return ex * (delta * delta - phi * phi - v - ex) / (2 * (phi * phi + v + ex) ** 2) - (x - a) / (TAU * TAU); };
    let A = a, B; if (delta * delta > phi * phi + v) B = Math.log(delta * delta - phi * phi - v); else { let k = 1; while (fx(a - k * TAU) < 0) k++; B = a - k * TAU; }
    let fA = fx(A), fB = fx(B);
    for (let i = 0; i < 60 && Math.abs(B - A) > 1e-6; i++) { const C = A + (A - B) * fA / (fB - fA), fC = fx(C); if (fC * fB <= 0) { A = B; fA = fB; } else fA /= 2; B = C; fB = fC; }
    const vol = Math.exp(A / 2), phiStar = Math.sqrt(phi * phi + vol * vol);
    const phiNew = 1 / Math.sqrt(1 / (phiStar * phiStar) + 1 / v);
    const muNew = mu + phiNew * phiNew * sum;
    return { r: Math.round(1000 + Q * muNew), rd: Math.max(45, Q * phiNew), vol, n: st.n + 1 };
  },
};

// ---------- world ----------
function world(method, opts = {}) {
  const botTrue = opts.botTrue ?? 1000, botRating = opts.botRating ?? 1000, botShare = opts.botShare ?? 0.3;
  const champ = { t: 1350, r: opts.champR ?? 1250, rd: 60 };
  const pool = Array.from({ length: ROUNDS + 1 }, () => []);
  const humans = Array.from({ length: 400 }, () => ({ t: 1000 + 150 * gauss(), st: method.start(), strat: "honest" }));
  const addGhost = (r, g) => { const p = pool[r]; p.push(g); if (p.length > POOL) p.shift(); };
  const pick = (r) => {
    if (!pool[r].length || rnd() < botShare) return { team: botTrue + LUCK * gauss(), r: botRating, rd: 60, bot: true };
    return pool[r][Math.floor(rnd() * pool[r].length)];
  };
  // one run; returns {delta, wins, losses}
  function run(p, record = true) {
    const luck = LUCK * gauss(), team = p.t + luck;
    const fights = []; let hearts = HEARTS, slay = false, outOfHearts = false, abandoned = false; const forfeits = [];
    const rating = p.st.r, rd = p.st.rd ?? 60;
    let r = 1;
    for (; r <= ROUNDS; r++) {
      const opp = pick(r);
      // dodgers
      if (p.strat === "quitBadTeam" && r === 4 && luck < -40) { abandoned = true; if (opts.penalty) for (let h = 0; h < hearts; h++) forfeits.push(opp); break; }
      if (p.strat === "quitAhead" && fights.length >= 2 && fights.reduce((a, x) => a + x.s, 0) / fights.length >= 0.7) { abandoned = true; if (opts.penalty) for (let h = 0; h < hearts; h++) forfeits.push(opp); break; }
      const s = play(team, opp.team);
      fights.push({ s, opp });
      const live = opts.liveGhost ? rating + 10 * fights.reduce((a, x) => a + x.s - elo(rating, x.opp.r), 0) : rating;
      if (record) addGhost(r, { team, r: live, rd });
      if (s === 0 && --hearts === 0) { outOfHearts = true; break; }
    }
    if (!outOfHearts && !abandoned) { const s = rnd() < pWin(team, champ.t) ? 1 : 0; fights.push({ s, opp: champ, crown: true }); slay = s === 1; }
    // current formula with penalty: forfeited hearts count as lost round fights
    if (opts.penalty && forfeits.length) for (const o of forfeits) if (method === M.current || method === M.haldane) fights.push({ s: 0, opp: o });
    const ctx = { slay, outOfHearts, abandoned, forfeits: method === M.current || method === M.haldane ? [] : forfeits };
    const before = p.st.r; p.st = method.update(p.st, fights, ctx);
    return p.st.r - before;
  }
  // warm up: everyone plays 40 runs, interleaved
  for (let i = 0; i < 400 * 40; i++) run(humans[Math.floor(rnd() * humans.length)]);
  return { humans, run, champ };
}

// ---------- measurements ----------
const mean = (a) => a.reduce((x, y) => x + y, 0) / a.length;
const sd = (a) => { const m = mean(a); return Math.sqrt(mean(a.map((x) => (x - m) ** 2))); };
function cohort(w, method, t, n, runs, strat = "honest", startR) {
  const ps = Array.from({ length: n }, () => ({ t, st: method.start(), strat }));
  if (startR !== undefined) for (const p of ps) p.st.r = startR;
  const hist = ps.map(() => []);
  for (let k = 0; k < runs; k++) {
    ps.forEach((p, i) => { w.run(p, strat === "honest"); hist[i].push(p.st.r); });
    for (let j = 0; j < 8; j++) w.run(w.humans[Math.floor(rnd() * w.humans.length)]); // keep the pool live
  }
  return hist;
}
// Drift: a veteran whose rating equals their true skill (t=1000): mean change per run, rating reset each run.
function drift(w, method, t) {
  const ds = [];
  const vet = method.start(); vet.n = 200; vet.r = t; if (vet.rd) vet.rd = 60;
  for (let k = 0; k < 6000; k++) { const p = { t, st: { ...vet }, strat: "honest" }; ds.push(w.run(p, false)); }
  return { mean: mean(ds), sd: sd(ds) };
}

const names = ["current", "haldane", "fight16", "fight10", "fightSteps", "glicko2"];
const out = {};
for (const name of names) {
  seed = 777;
  const m = M[name], w = world(m);
  const pop = w.humans; // population bias: rating - truth after warm-up, by skill band
  const bands = [[-Infinity, 850], [850, 1150], [1150, Infinity]].map(([lo, hi]) => {
    const g = pop.filter((p) => p.t >= lo && p.t < hi); return Math.round(mean(g.map((p) => p.st.r - p.t)));
  });
  const res = { popBias: bands };
  for (const t of [1000]) res.drift = drift(w, m, t);
  res.driftStrong = drift(w, m, 1300);
  const conv = {};
  for (const t of [700, 1300]) {
    const h = cohort(w, m, t, 40, 120);
    const avgAt = (k) => Math.round(mean(h.map((x) => x[k - 1])));
    const tail = h.map((x) => mean(x.slice(80)));
    const fin = mean(tail);
    const within = () => { for (let i = 0; i < 120; i++) if (Math.abs(mean(h.map((x) => x[i])) - fin) <= 50) return i + 1; return ">120"; };
    conv[t] = { at1: avgAt(1), at5: avgAt(5), at10: avgAt(10), at30: avgAt(30), final: Math.round(fin), runsToWithin50ofFinal: within(), noise: Math.round(mean(h.map((x) => sd(x.slice(80))))) };
  }
  res.conv = conv;
  // dodgers (t=1000), with and without the abandon penalty, vs honest
  const dodge = {};
  for (const pen of [false, true]) {
    seed = 99;
    const w2 = world(m, { penalty: pen });
    const hon = cohort(w2, m, 1000, 30, 100, "honest", 1000).map((x) => mean(x.slice(50)));
    const bad = cohort(w2, m, 1000, 30, 100, "quitBadTeam", 1000).map((x) => mean(x.slice(50)));
    const ahead = cohort(w2, m, 1000, 30, 100, "quitAhead", 1000).map((x) => mean(x.slice(50)));
    dodge[pen ? "penalty" : "noPenalty"] = { honest: Math.round(mean(hon)), quitBadTeam: Math.round(mean(bad)), quitAhead: Math.round(mean(ahead)) };
  }
  res.dodge = dodge;
  // bots weaker than their fixed rating (true 850, rated 1000, half the pool)
  seed = 5;
  const w3 = world(m, { botTrue: 850, botRating: 1000, botShare: 0.5 });
  res.weakBots = Math.round(mean(w3.humans.map((p) => p.st.r - p.t)));
  out[name] = res;
  console.log(name, JSON.stringify(res));
}

// Outcome table for a 1000-rated veteran vs 1000-rated ghosts, champion rated 1250
const tbl = [["0-5", 0, 5, null], ["3-5", 3, 5, null], ["6-5", 6, 5, null], ["7-5 +Crown lost", 7, 5, null], ["12-0 +Crown lost", 12, 0, 0], ["12-0 +Crown won", 12, 0, 1], ["9-3 +Crown won", 9, 3, 1]];
console.log("\noutcomes (veteran 1000 vs 1000s, champion 1250):");
for (const [lbl, wn, ls, crown] of tbl) {
  const f = [...Array(wn).fill(1), ...Array(ls).fill(0)].map((s) => ({ s, opp: { r: 1000, rd: 60 } }));
  if (crown !== null) f.push({ s: crown, opp: { r: 1250, rd: 60 }, crown: true });
  const ctx = { slay: crown === 1, outOfHearts: ls === 5, forfeits: [] };
  const row = {};
  for (const name of names) {
    const st = M[name].start(); st.r = 1000; st.n = 200; if (st.rd) st.rd = 60;
    const st0 = M[name].start(); st0.r = 1000; // a brand-new player
    row[name] = M[name].update(st, f, ctx).r - 1000;
    row[name + "_new"] = M[name].update(st0, f, ctx).r - 1000;
  }
  console.log(lbl.padEnd(18), JSON.stringify(row));
}

// Why strong players sit a little low under per-fight Elo: the champion's team
// is stronger than its owner's rating, and late-round ghosts are lucky survivors.
console.log("\nvariants (fight10): drift per run at rating = skill 1000 / 1300");
for (const [lbl, o] of [["as above (champion rated 1250, true 1350)", {}], ["champion rated at its true 1350", { champR: 1350 }], ["+ ghost carries owner's live in-run rating", { champR: 1350, liveGhost: true }]]) {
  seed = 4242; const w = world(M.fight10, o);
  const a = drift(w, M.fight10, 1000), b = drift(w, M.fight10, 1300);
  console.log(lbl.padEnd(44), a.mean.toFixed(1), b.mean.toFixed(1));
}
