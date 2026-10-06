// R3-13 how to try, against a LOCAL server (BASE, default 127.0.0.1:8795; never the live one):
// fill the line, buy new units (they land in slots 5-7), then buy copies of a bench unit until it awakens there.
const B = (process.env.BASE ?? "http://127.0.0.1:8795") + "/api/v1";
let me = "";
const call = async (m, p, b) => {
  const r = await fetch(B + p, { method: m, headers: { "content-type": "application/json", ...(me ? { "X-Arena-Player": me } : {}) }, ...(b ? { body: JSON.stringify(b) } : {}) });
  const j = await r.json();
  if (!r.ok) throw new Error(`${p} ${JSON.stringify(b)} → ${r.status} ${JSON.stringify(j)}`);
  return j;
};
me = (await call("POST", "/players", { name: "bench-try" })).id;
const show = (u) => `${u.name}×${u.copies} ${u.form}`;
for (let runs = 1; runs <= 20; runs++) {
  let run = await call("POST", "/runs");
  const d = async (x) => (run = (await call("POST", `/runs/${run.runId}/decisions`, x)).run);
  while (run.phase === "shop") {
    const onBench = (id) => run.bench.some((u) => u.unitId === id);
    const owned = (id) => onBench(id) || run.line.some((u) => u.unitId === id);
    const pick = run.offers.find((o) => onBench(o.unitId)) ?? (run.bench.length < 3 ? run.offers.find((o) => !owned(o.unitId)) : undefined);
    if (pick && run.gold >= 3) {
      const before = run.bench.length;
      await d({ kind: "buy", slot: pick.slot });
      if (run.bench.length > before) console.log(`line ${run.line.length}/5 full, new buy landed in slot ${4 + run.bench.length}:`, show(run.bench.at(-1)));
      else if (onBench(pick.unitId)) console.log("copy merged on the bench:", show(run.bench.find((u) => u.unitId === pick.unitId)));
      const awake = run.bench.find((u) => u.form === "awoken");
      if (awake) {
        console.log("line:", run.line.map(show).join(", "));
        console.log("bench:", run.bench.map(show).join(", "));
        console.log(`OK: ${awake.name} awoke on the bench (run ${runs}, round ${run.round})`);
        process.exit(0);
      }
    } else if (run.gold >= 1) await d({ kind: "reroll" });
    else await d({ kind: "fight" });
  }
  if (run.phase === "crown") await call("POST", `/runs/${run.runId}/abandon`);
}
console.log("no awakening in 20 runs");
process.exit(1);
