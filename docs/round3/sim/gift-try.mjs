// R3-15 how to try, against a LOCAL server (BASE, default 127.0.0.1:8795; never the live one):
// play until a copy awakens a unit, show the 3 gift choices, try a buy (refused), pick one.
const B = (process.env.BASE ?? "http://127.0.0.1:8795") + "/api/v1";
let me = "";
const call = async (m, p, b, ok = true) => {
  const r = await fetch(B + p, { method: m, headers: { "content-type": "application/json", ...(me ? { "X-Arena-Player": me } : {}) }, ...(b ? { body: JSON.stringify(b) } : {}) });
  const j = await r.json();
  if (ok && !r.ok) throw new Error(`${p} ${JSON.stringify(b)} → ${r.status} ${JSON.stringify(j)}`);
  return { status: r.status, ...j };
};
me = (await call("POST", "/players", { name: "gift-try" })).id;
const show = (u) => `${u.name}×${u.copies} ${u.form}`;
for (let runs = 1; runs <= 20; runs++) {
  let run = await call("POST", "/runs");
  const d = async (x) => (run = (await call("POST", `/runs/${run.runId}/decisions`, x)).run);
  while (run.phase === "shop" && !run.gift) {
    const owned = (id) => [...run.line, ...run.bench].some((u) => u.unitId === id && u.kind === "unit");
    const room = run.line.length + run.bench.length < 8;
    const pick = run.offers.find((o) => owned(o.unitId)) ?? (room ? run.offers[0] : undefined);
    if (pick && run.gold >= pick.cost) await d({ kind: "buy", slot: pick.slot });
    else if (run.gold >= 1) await d({ kind: "reroll" });
    else await d({ kind: "fight" });
  }
  if (!run.gift) { if (run.phase === "crown") await call("POST", `/runs/${run.runId}/abandon`); continue; }
  const awake = [...run.line, ...run.bench].filter((u) => u.form === "awoken").map(show);
  console.log(`round ${run.round}: awoken ${awake.join(", ")}`);
  console.log("gift choices:", run.gift.join(", "));
  const refused = await call("POST", `/runs/${run.runId}/decisions`, { kind: "buy", slot: 0 }, false);
  console.log(`buy before picking → ${refused.status} ${refused.error}`);
  const id = run.gift[0];
  if (run.line.length + run.bench.length >= 8 && ![...run.line, ...run.bench].some((u) => u.unitId === id)) {
    await d({ kind: "sell", index: 7 });
    console.log("line and bench full: sold the last bench unit to make room");
  }
  await d({ kind: "gift", pick: 0 });
  const where = run.line.some((u) => u.unitId === id) ? "the line" : "the bench";
  console.log(`picked ${id}: it is on ${where}`);
  console.log(`line ${run.line.map(show).join(", ")}`);
  console.log(`bench ${run.bench.map(show).join(", ") || "(empty)"}; gift now ${run.gift ? run.gift.join(", ") + " (the pick awakened another unit)" : "gone"}`);
  console.log("OK");
  process.exit(0);
}
console.log("no awakening in 20 runs");
process.exit(1);
