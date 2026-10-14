// The unit batch report page (rules v2, #1046): every unit of the v2 pool in
// EN and RU with emoji, name, sleeping and awoken sentences, shape, tier,
// numbers, a trace line from its fixture battle, and #scene= links to the lab
// battles where it acts. Maks judges a batch from this page plus the workshop.
//   npm run mvp:batch-report -- [--out ~/reports/arena-batch1.html] [--base https://m1.twin-pogona.ts.net/arena-lab/] [--tuned]
// --tuned reads docs/mvp/tune-v2.json (npm run mvp:tune-v2) for the balance:
// the meta verdict, and each unit's field score, Awoken's margin and share.
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { contentOf } from "../server/src/mvp/content.js";
import { battle } from "../src/battle.js";
import { formText, sentenceText } from "../src/mvp/form-text.js";
import { ROWS_V2 } from "../src/mvp/units-v2.js";
import { FIXTURES } from "../src/words/fixtures.js";
import { traceLines } from "../src/words/trace.js";
import { slug } from "../src/mvp/units.js";
import { poolBattle, poolBattleId } from "../src/mvp/pool-battles.js";
import { ruleText } from "../server/src/mvp/night-batch.js";

const args = process.argv.slice(2);
const opt = (n: string) => {
  const i = args.indexOf(`--${n}`);
  return i >= 0 ? args[i + 1] : undefined;
};
const out = (opt("out") ?? "~/reports/arena-batch2.html").replace(/^~/, homedir());
const base = (opt("base") ?? "https://m1.twin-pogona.ts.net/arena-lab/").replace(/\/?$/, "/");
const tuned = args.includes("--tuned");
interface TuneUnit { name: string; fieldScore: number; awokenMargin: number; share: number; inBand: boolean; team: string[] }
interface TuneReport { median: number; band: number; healthy: boolean; checks: Record<string, { value: number; pass: boolean; rule: string }>; units: TuneUnit[]; rounds: unknown[]; seconds: number; healthChecks?: { seed: number; healthy: boolean }[] }
const TUNE_FILE = "docs/mvp/tune-v2.json";
const tune: TuneReport | null = tuned && existsSync(TUNE_FILE) ? JSON.parse(readFileSync(TUNE_FILE, "utf8")) : null;
if (tuned && !tune) throw new Error(`--tuned needs ${TUNE_FILE}: run npm run mvp:tune-v2 first`);
const pct = (x: number) => `${Math.round(x * 100)}%`;
const balanceOf = (name: string) => {
  const t = tune?.units.find((u) => u.name === name);
  if (!t || !tune) return "";
  const side = t.inBand ? "in band" : t.fieldScore > tune.median ? "strong" : "weak";
  return ` · field ${pct(t.fieldScore)} (${side}) · Awoken ${t.awokenMargin >= 0 ? "+" : ""}${pct(t.awokenMargin)} · ${pct(t.share)} of the meta`;
};
const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/"/g, "&quot;");

const content = contentOf(ROWS_V2);

// Maks's two wording rules (10-14): no coin words, one strong reading. The
// before → after list of every changed unit sentence (docs/mvp/wording-1047.json),
// and with --night <batch json> each stored night fusion re-rendered by today's words.
interface Reworded { unit: string; what: string; lang: string; before: string; after: string; why: string }
const reworded: Reworded[] = existsSync("docs/mvp/wording-1047.json") ? JSON.parse(readFileSync("docs/mvp/wording-1047.json", "utf8")) : [];
const nightFile = opt("night")?.replace(/^~/, homedir());
interface NightCandidate { name: { en: string }; text: { en: string; ru: string }; unit: unknown; abilities: unknown; statuses: unknown }
const night: { name: string; lang: string; before: string; after: string }[] = [];
if (nightFile) {
  const batch = JSON.parse(readFileSync(nightFile, "utf8")) as { results: { candidates?: NightCandidate[] }[] };
  for (const r of batch.results) for (const c of r.candidates ?? []) for (const lang of ["en", "ru"] as const) {
    if (!c.text?.[lang]) continue;
    const after = ruleText(c as never, lang);
    if (after !== c.text[lang]) night.push({ name: c.name.en, lang, before: c.text[lang], after });
  }
}
const rewordTable = (rows: { who: string; before: string; after: string; why?: string }[]) =>
  `<table class="reword"><tr><th></th><th>Before</th><th>After</th></tr>${rows.map((r) => `<tr><td>${esc(r.who)}${r.why ? `<br><small>${esc(r.why)}</small>` : ""}</td><td>${esc(r.before)}</td><td>${esc(r.after)}</td></tr>`).join("")}</table>`;
const wording = reworded.length || night.length
  ? `<details class="note" open><summary><b>Wording: every changed sentence, before → after</b> (${reworded.length} on units${nightFile ? `, ${night.length} on stored night fusions` : ""})</summary>
<p>Maks's rules (10-14): a card says the chance, never a coin; every count, period, condition and target says what is counted and when it resets ("on turns 2, 4, 6…", "at a turn end with 3+ beats, it spends 3", "the first time … in a battle", "while at full HP"). Tests: no card sentence has heads/tails/flip/coin or орёл/решка/монет, and none says "every second/third turn".</p>
${rewordTable(reworded.map((r) => ({ who: `${r.unit} · ${r.what} · ${r.lang.toUpperCase()}`, before: r.before, after: r.after, why: r.why })))}
${nightFile ? `<h3>Night fusions (${esc(nightFile.split("/").pop()!)}), re-rendered</h3>${night.length ? rewordTable(night.map((r) => ({ who: `${r.name} · ${r.lang.toUpperCase()}`, before: r.before, after: r.after }))) : "<p>No sentence changed.</p>"}` : ""}
</details>`
  : "";
const scene = (id: string, lang: string) => `${base}#scene=${id}&lang=${lang}`;

/** The fixture's trace line, and the first other line that names the unit. */
function trace(name: string): string[] {
  const fx = FIXTURES[name];
  if (!fx) return [];
  const lines = traceLines(fx.input, battle(fx.input));
  return [fx.trace, ...lines.filter((l) => l !== fx.trace).slice(0, 2)];
}

const player = { id: "report", name: "report", bot: true };
/** A unit without a fixture: the lines naming it in its pool battle (the lab's `v2-unit-` scene). */
function poolTrace(name: string, awoken: boolean): string[] {
  const { own, lines } = poolBattle(content, name, awoken, player, "2026-10-13T00:00:00.000Z");
  return own.length ? own.slice(0, 3) : lines.slice(0, 2);
}

/** The newest batch first (#1047): batch 2's twenty, then batch 1's. */
const BATCH1 = 20;
const ordered = [...ROWS_V2.slice(BATCH1), ...ROWS_V2.slice(0, BATCH1)];
const cards = ordered.map((row, i) => {
  const u = content.units.find((x) => x.id === slug(row.name))!;
  const key = row.name.replace(/[^A-Za-z]/g, "");
  const fx = key.charAt(0).toLowerCase() + key.slice(1);
  const forms = (["sleeping", "awoken"] as const).map((f) => {
    const form = u.forms[f];
    const fixture = f === "sleeping" ? fx : `${fx}Awoken`;
    const hasFixture = FIXTURES[fixture] !== undefined;
    const sceneId = hasFixture ? (f === "sleeping" ? `v2-${slug(row.name)}` : `v2-${slug(row.name)}-awoken`) : poolBattleId(row.name, f === "awoken");
    const tr = (hasFixture ? trace(fixture) : poolTrace(row.name, f === "awoken")).map((l, i) => `<li class="${i === 0 ? "trace main" : "trace"}">${esc(l)}</li>`).join("");
    return `<div class="form">
  <h3>${f === "sleeping" ? "Sleeping · Спящий" : "Awoken · Пробуждённый"}</h3>
  <p class="en">${esc(sentenceText(form, content.abilities, "en", content.statuses))}</p>
  <p class="ru">${esc(sentenceText(form, content.abilities, "ru", content.statuses))}</p>
  <p class="chain"><code>${esc(formText(form, content.abilities, "en", content.statuses))}</code></p>
  ${tr ? `<ul class="traces">${tr}</ul>` : ""}
  <p class="go">Scene: <a href="${esc(scene(sceneId, "en"))}" target="_blank" rel="noopener">EN</a> <a href="${esc(scene(sceneId, "ru"))}" target="_blank" rel="noopener">RU</a> <code>${sceneId}</code></p>
</div>`;
  });
  const heading = i === 0 ? `<h2 class="batch">Batch 2 · new</h2>` : i === ROWS_V2.length - BATCH1 ? `<h2 class="batch">Batch 1 · with Maks's notes applied</h2>` : "";
  return `${heading}<section class="card" id="${u.id}">
  <header><span class="emoji">${u.emoji}</span><h2>${esc(u.name)}</h2><span class="meta">tier ${u.tier} · ${row.shape} · ${u.base.pwr}/${u.base.hp}${esc(balanceOf(row.name))}</span></header>
  <p class="arch">${esc(u.archetype)}</p>
  <div class="forms">${forms.join("\n")}</div>
</section>`;
});

const html = `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Arena unit batches</title>
<style>
:root{--bg:#f6f1e7;--ink:#1d1a16;--dim:#6b6257;--card:#fffaf0;--line:#1d1a16;--acc:#b5471b}
@media(prefers-color-scheme:dark){:root{--bg:#1b1916;--ink:#f1eadb;--dim:#a39a8c;--card:#262220;--line:#f1eadb;--acc:#f0925f}}
body{margin:0;padding:24px 16px 64px;background:var(--bg);color:var(--ink);font:16px/1.45 Georgia,serif;max-width:900px;margin-inline:auto}
h1{font-size:28px;margin:0 0 4px}.sub{color:var(--dim);margin:0 0 24px}
.card{background:var(--card);border:2px solid var(--line);border-radius:14px;padding:16px;margin:0 0 20px}
header{display:flex;align-items:baseline;gap:10px;flex-wrap:wrap}.emoji{font-size:30px}h2{margin:0;font-size:22px}.meta{color:var(--dim);font-size:14px}
.arch{margin:6px 0 10px;font-style:italic}
.forms{display:grid;grid-template-columns:1fr;gap:12px}@media(min-width:720px){.forms{grid-template-columns:1fr 1fr}}
.form{border-top:1px dashed var(--dim);padding-top:8px}.form h3{margin:0 0 6px;font-size:14px;color:var(--dim);font-weight:normal}
.form p{margin:4px 0}.ru{color:var(--dim)}ul{margin:6px 0;padding-left:18px;font-size:14px}.status{color:var(--dim)}.trace{font-family:ui-monospace,Menlo,monospace;font-size:13px}.trace.main{color:var(--acc)}
.go a{margin-right:8px}code{font-size:12px;color:var(--dim)}
.batch{margin:32px 0 12px;font-size:20px}.chain code{font-size:12px}
.note{border-left:4px solid var(--acc);padding:8px 12px;margin:0 0 24px;background:var(--card)}
.reword{border-collapse:collapse;font-size:13px;width:100%;margin:8px 0}.reword td,.reword th{border-top:1px solid var(--dim);padding:4px 6px;vertical-align:top;text-align:left}.reword small{color:var(--dim)}
</style></head><body>
<h1>Arena · rules v2 · the 40 units (rethink, #1063)</h1>
<p class="sub">Unit batch 2 of 3 (twenty new units, every shape class and tier) first, then batch 1 with Maks's notes applied: card sentences a person would say (the word chain in grey below each), lower HP. Mission branch <code>mission-1043-rules-v2</code>. Design: <a href="arena-rules-v2.html">arena-rules-v2.html</a>. Scenes open on the lab.</p>
${tune ? `<div class="note"><b>Tuned.</b> PWR and HP come from <code>npm run mvp:tune-v2</code>${tune.rounds.length ? `: ${tune.rounds.length} rounds of the quick meta check move each unit toward the pool's median field score` : `, read on the numbers as they are: each unit's field score against the pool's median`} (${pct(tune.median)} ± ${pct(tune.band)}), averaged over several quick meta checks; then the full meta check: <b>${tune.healthy ? "healthy" : "not healthy"}</b>${tune.healthChecks ? ` (${tune.healthChecks.filter((h) => h.healthy).length} of ${tune.healthChecks.length} full checks healthy, one per seed)` : ` (${Object.values(tune.checks).map((c) => `${esc(c.rule)}: ${c.value < 1 && c.value > 0 ? pct(c.value) : c.value} ${c.pass ? "✓" : "✗"}`).join("; ")})`}. ${tune.units.filter((u) => u.inBand).length} of ${tune.units.length} units in band${tune.units.some((u) => !u.inBand) ? `; outside it: ${tune.units.filter((u) => !u.inBand).map((u) => `${esc(u.name)} ${pct(u.fieldScore)}`).join(", ")}` : ""}. Field: the unit's best team's mean score against the meta's teams. Awoken: what three copies add over sleeping, same team.</div>` : ""}
${tuned ? "" : `<div class="note"><b>Untuned draft.</b> The numbers are placeholders; the tune and the meta run come next. Units with a fixture battle link it; the others link their pool battle (the lab's <code>v2-unit-</code> scenes).</div>`}
${wording}
${cards.join("\n")}
</body></html>
`;
writeFileSync(out, html);
console.log(`wrote ${out}: ${ordered.length} units`);
