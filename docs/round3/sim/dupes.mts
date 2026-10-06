// Near-duplicate sweep + awoken classification over the MVP roster.
const R = "/Users/admin/Work/arena-574/scout-r3";
const { ROWS } = await import(`${R}/src/mvp/units.ts`);
const kind = (d: string) => d.split(" + ").map((x) => x.replace(/ \d+$/, "")).join("+");
const num = (d: string) => (d.match(/\d+/g) ?? []).map(Number).reduce((a, b) => a + b, 0);
// signature: when / who / does-kind (sleeping)
const sig = new Map<string, string[]>();
for (const r of ROWS) {
  const k = `${r.when} · ${r.who} · ${kind(r.does)}`;
  sig.set(k, [...(sig.get(k) ?? []), `${r.name}(T${r.tier})`]);
}
console.log("== exact sleeping signature collisions (when/who/does-kind)");
for (const [k, v] of sig) if (v.length > 1) console.log(`${k}: ${v.join(", ")}`);
// looser: who+does-kind (same job, different trigger)
const job = new Map<string, string[]>();
for (const r of ROWS) {
  const k = `${r.who} · ${kind(r.does)}`;
  job.set(k, [...(job.get(k) ?? []), `${r.name}(${r.when},T${r.tier})`]);
}
console.log("\n== same Who+Does kind (different When)");
for (const [k, v] of job) if (v.length > 1) console.log(`${k}: ${v.join(", ")}`);
console.log("\n== awoken classification");
const cls: Record<string, string[]> = {};
for (const r of ROWS) {
  const aw = r.awoken.does ?? [r.does];
  const who = r.awoken.who ?? r.who;
  let c: string;
  if (who !== r.who) c = "new Who";
  else if (aw.length === 1 && kind(aw[0]) === kind(r.does)) c = "numbers only";
  else if (aw.length === 2 && kind(aw[0]) === kind(r.does) && aw[0] === r.does) c = "same + small rider";
  else if (aw.length === 2 && kind(aw[0]) === kind(r.does)) c = "bigger + rider";
  else c = "new Does";
  (cls[c] ??= []).push(`${r.name} [${r.when}/${r.who}/${r.does} → ${who}/${aw.join(", ")}]`);
}
for (const [c, v] of Object.entries(cls)) { console.log(`\n-- ${c} (${v.length})`); for (const x of v) console.log("  " + x); }
