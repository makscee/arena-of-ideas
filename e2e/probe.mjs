// Convenience runner for ONE scoped probe against an already-warm stack (see
// `npm run e2e:serve`): `npm run probe <name...>` resolves each name to a
// `probe-<name>.mjs` (generous substring, leading `probe-`/trailing `.mjs`
// optional) and runs it with the tsx loader the probes need. AOI_BASE_URL
// defaults to the warm stack's origin so an agent never retypes it; override it
// in the env to point elsewhere. This does NOT boot or tear down anything — the
// warm server outlives the probe.

import { spawnSync } from "node:child_process";
import { readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");
const BASE = process.env.AOI_BASE_URL ?? "http://localhost:5280";

const all = readdirSync(here)
  .filter((f) => f.startsWith("probe-") && f.endsWith(".mjs"))
  .sort();

const names = process.argv.slice(2);
if (names.length === 0) {
  console.error("usage: npm run probe <name...>   (e.g. beats, motion)");
  console.error("available:");
  for (const p of all) console.error(`  ${p}`);
  process.exit(2);
}

const matched = [];
for (const name of names) {
  const t = name.toLowerCase().replace(/^probe-/, "").replace(/\.mjs$/, "");
  const hits = all.filter((f) => f.toLowerCase().includes(t));
  if (hits.length === 0) {
    console.error(`no probe matches "${name}". available:`);
    for (const p of all) console.error(`  ${p}`);
    process.exit(2);
  }
  for (const h of hits) if (!matched.includes(h)) matched.push(h);
}

let failed = false;
for (const probe of matched) {
  console.log(`\n=== ${probe} (against ${BASE}) ===`);
  const r = spawnSync("node", ["--import", "tsx/esm", join(here, probe)], {
    cwd: root,
    stdio: "inherit",
    timeout: 150_000,
    env: { ...process.env, AOI_BASE_URL: BASE },
  });
  if (r.status !== 0 || r.signal) failed = true;
}
process.exit(failed ? 1 : 0);
