// Numbers by simulation for one candidate unit (mission #735, M2-7).
//
//   npm run mvp:tune -- --unit <live unit id> [--scramble]   a copy of a live unit
//   npm run mvp:tune -- --unit tyrant                         a test row (TEST_ROWS)
//   npm run mvp:tune -- --row <row.json>                      a candidate Row
//   npm run mvp:tune -- --batch <rows.json>                   every candidate, one at a time
//   … [--quick] [--no-meta] [--seed N]
//
// The field is today's breaker teams, read from docs/mvp/meta-health.json
// (`npm run mvp:meta` writes it). The band comes from measuring every live
// unit the same way; it is cached in docs/mvp/tune-band.json for the content
// and settings it was measured on. The meta check with the unit added is the
// full one (~15 min); --quick runs the quick one, --no-meta skips it.
// A batch writes <rows>.tuned.json after each unit.
// The library is src/mvp/tune.ts.

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { META_FULL, META_QUICK } from "../src/mvp/meta.js";
import { TEST_ROWS, TUNE_DEFAULT, fieldOfReport, liveBand, scrambleRow, slugOf, tuneBatch, tuneUnit, type Band, type TuneResult } from "../src/mvp/tune.js";
import { ROWS, type Row } from "../src/mvp/units.js";
import { mvpContent } from "../server/src/mvp/content.js";

const argv = process.argv.slice(2);
const flag = (f: string) => argv.includes(f);
function arg(f: string): string | undefined {
  const i = argv.indexOf(f);
  return i < 0 ? undefined : argv[i + 1];
}
const log = (line: string) => console.error(line);

const seedRaw = arg("--seed");
const settings = { ...TUNE_DEFAULT, ...(seedRaw !== undefined ? { seed: Number(seedRaw) } : {}) };
const meta = flag("--no-meta") ? null : flag("--quick") ? META_QUICK : META_FULL;

const content = mvpContent();
const report = JSON.parse(readFileSync("docs/mvp/meta-health.json", "utf8"));
if (report.contentVersion !== content.version) log(`note: docs/mvp/meta-health.json is for ${report.contentVersion}, the pool is ${content.version}; rerun npm run mvp:meta for today's field`);
const field = fieldOfReport(report, content.units.map((u) => u.id));

const BAND_FILE = "docs/mvp/tune-band.json";
function band(): Band {
  const key = { contentVersion: content.version, field: report.contentVersion, settings };
  if (existsSync(BAND_FILE)) {
    const cached = JSON.parse(readFileSync(BAND_FILE, "utf8"));
    if (JSON.stringify(cached.key) === JSON.stringify(key)) return cached.band;
  }
  log(`measuring the band on ${content.units.length} live units against ${field.length} field teams…`);
  const b = liveBand(ROWS, field, settings, undefined, log);
  writeFileSync(BAND_FILE, JSON.stringify({ key, band: b }, null, 2) + "\n");
  return b;
}

function candidates(): Row[] {
  const unit = arg("--unit");
  if (unit) {
    const test = TEST_ROWS.find((r) => slugOf(r.name) === unit);
    if (test) return [test];
    const live = ROWS.find((r) => slugOf(r.name) === unit);
    if (!live) throw new Error(`no live unit or test row "${unit}"`);
    // A copy under its own id, its numbers scrambled with --scramble.
    const copy: Row = { ...live, name: `${live.name} Copy` };
    return [flag("--scramble") ? scrambleRow(copy, ROWS, settings.seed) : copy];
  }
  const file = arg("--row") ?? arg("--batch");
  if (!file) throw new Error("give --unit <id>, --row <row.json> or --batch <rows.json>");
  const data = JSON.parse(readFileSync(file, "utf8"));
  return Array.isArray(data) ? data : [data];
}

const brief = (r: Row) => `${r.pwr}/${r.hp} ${r.does} → ${JSON.stringify(r.awoken)}`;
const rows = candidates();
const b = band();
const pct = (x: number) => `${Math.round(x * 100)}%`;
log(`band ${pct(b.low)}–${pct(b.high)}, target ${pct(b.target)}; field ${field.length} teams`);
const opts = { liveRows: ROWS, field, band: b, settings, meta, log };
const batch = arg("--batch");
const results: TuneResult[] = batch
  ? tuneBatch(rows, opts, (i, r) => {
      const out = batch.replace(/\.json$/, "") + ".tuned.json";
      const prev: unknown[] = i === 0 || !existsSync(out) ? [] : JSON.parse(readFileSync(out, "utf8"));
      prev.push({ name: rows[i]!.name, pass: r.pass, reason: r.reason, row: r.row, score: r.measure.score, sleeping: r.measure.sleeping, team: r.measure.team, meta: r.meta, seconds: r.seconds });
      writeFileSync(out, JSON.stringify(prev, null, 2) + "\n");
      log(`  ${i + 1}/${rows.length} done in ${r.seconds}s → ${out}`);
    })
  : [tuneUnit(rows[0]!, opts)];

for (const [i, r] of results.entries()) {
  if (arg("--unit") && !TEST_ROWS.some((t) => slugOf(t.name) === arg("--unit"))) {
    const orig = ROWS.find((x) => slugOf(x.name) === arg("--unit"))!;
    console.log(`original  ${brief(orig)}`);
    console.log(`given     ${brief(rows[i]!)}`);
  }
  console.log(`tuned     ${brief(r.row)}  (${r.steps.length} measures, ${r.seconds}s)`);
  console.log(`${r.pass ? "PASS" : "FAIL"}: ${rows[i]!.name}: ${r.reason}`);
}
