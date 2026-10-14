// The two pages of makscee/void-board#1065 from the batch's JSON and the picks:
//   arena-fusions-20.html  the 20 designed fusions: the pair and why, both
//                          Awoken parents, the 3 candidates, the pick and why;
//                          the 10 undiscovered pairs by name only.
//   arena-discover.html    one checkpoint link per undiscovered pair (the
//                          workshop, one Fuse from discovering it), a fresh run.
//
//   node --import tsx/esm scripts/fusions-1065-report.ts --dir docs/mission1043/fusions-1065 --invite '<admin invite link>' --out ~/reports
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import type { Candidate, PairResult } from "../server/src/mvp/night-batch.js";
import { ruleText } from "../server/src/mvp/night-batch.js";
import { contentOf } from "../server/src/mvp/content.js";
import { lineUnitOf } from "../src/mvp/forms.js";
import { toBattleDef } from "../src/mvp/fight.js";
import { ROWS_V2 } from "../src/mvp/units-v2.js";

const arg = (name: string, dflt?: string): string | undefined => {
  const i = process.argv.indexOf(`--${name}`);
  return i > 0 ? process.argv[i + 1] : dflt;
};
const dir = arg("dir", "docs/mission1043/fusions-1065")!;
const out = arg("out", dir)!;
const invite = arg("invite", "")!;

interface Picks {
  /** pair key (sorted ids) → why it was chosen. */
  reasons: Record<string, string>;
  /** pair key → the final candidate's EN name and why. */
  picks: Record<string, { name: string; why: string }>;
  /** pair key → what to look for at its checkpoint (no spoilers). */
  discover: Record<string, string>;
}
const batch = JSON.parse(readFileSync(join(dir, "night-1065.json"), "utf8")) as { results: PairResult[]; ranAt: string };
const picks = JSON.parse(readFileSync(join(dir, "picks.json"), "utf8")) as Picks;
const content = contentOf(ROWS_V2);
const unit = (id: string) => content.units.find((u) => u.id === id)!;
const esc = (s: string): string => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
const result = (key: string) => batch.results.find((r) => r.pair.key === key)!;
const label = (id: string) => `${unit(id).emoji} ${unit(id).name}`;

function parentCard(id: string): string {
  const u = unit(id);
  const def = toBattleDef(lineUnitOf(u, id, 3));
  const text = ruleText({ unit: def, abilities: content.abilities, statuses: content.statuses }, "en");
  return `<div class="parent"><div class="pn">${esc(label(id))} <small>Awoken · ${def.base.pwr}/${def.base.hp}</small></div><p>${esc(text)}</p></div>`;
}

function candCard(c: Candidate, picked: boolean): string {
  const k = c.check;
  const lean = c.lean === "twist" ? "twist" : `lean ${c.lean}`;
  const score = k ? `wins ${Math.round(k.score * 100)}% vs the field · scale ×${k.scale}` : "not measured";
  return `<article class="cand${picked ? " pick" : ""}${c.state !== "ready" ? " off" : ""}">
<div class="ch"><b>${esc(c.name.en)}</b> <span class="ru">${esc(c.name.ru)}</span> <span class="tag">${lean}</span>${picked ? ' <span class="tag pk">picked</span>' : ""}</div>
<div class="st">${c.unit.base.pwr} PWR · ${c.unit.base.hp} HP · ${score}${c.state !== "ready" ? ` · <span class="bad">${esc(c.why ?? c.state)}</span>` : ""}</div>
<p class="rule">${esc(c.text.en || "(no sentence)")}</p>
<p class="intent"><i>Intent:</i> ${esc(c.intent)}</p>
${c.flavour.en ? `<p class="fl">${esc(c.flavour.en)}</p>` : ""}
</article>`;
}

const STYLE = `<style>
:root{--bg:#f6f4ef;--panel:#fff;--text:#1d1b18;--dim:#6b665e;--line:#ddd7cc;--acc:#2f6f4f;--bad:#a33}
@media (prefers-color-scheme:dark){:root{--bg:#16150f;--panel:#211f19;--text:#ece8df;--dim:#a29c90;--line:#3a362d;--acc:#7cc79f;--bad:#e88}}
body{background:var(--bg);color:var(--text);font:15px/1.45 system-ui,sans-serif;margin:0 auto;max-width:1100px;padding:16px}
h1{font-size:22px;margin:8px 0}h2{font-size:18px;margin:28px 0 6px}small,.dim{color:var(--dim)}
section{background:var(--panel);border:1px solid var(--line);border-radius:10px;padding:12px 14px;margin:14px 0}
.why{margin:2px 0 10px;color:var(--dim)}.parents{display:grid;grid-template-columns:1fr 1fr;gap:10px}
.parent{border:1px dashed var(--line);border-radius:8px;padding:8px}.parent p{margin:4px 0 0}
.cands{display:grid;grid-template-columns:repeat(3,1fr);gap:10px;margin-top:10px}
.cand{border:2px solid var(--line);border-radius:8px;padding:8px}.cand.pick{border-color:var(--acc)}.cand.off{opacity:.65}
.cand p{margin:4px 0}.rule{font-weight:500}.intent,.fl,.st{font-size:13px;color:var(--dim)}.ru{color:var(--dim)}
.tag{font-size:12px;border:1px solid var(--line);border-radius:6px;padding:0 5px}.tag.pk{border-color:var(--acc);color:var(--acc)}
.bad{color:var(--bad)}.final{margin-top:10px;border-left:3px solid var(--acc);padding-left:8px}
a{color:var(--acc)}ol li{margin:10px 0}code{font-size:13px}
@media (max-width:720px){.parents,.cands{grid-template-columns:1fr}}
</style>`;

const designed = Object.keys(picks.picks);
const hidden = Object.keys(picks.discover);
const sections = designed
  .map((key, i) => {
    const r = result(key);
    const p = picks.picks[key]!;
    return `<section id="${key}"><h2>${i + 1}. ${esc(label(r.pair.a))} + ${esc(label(r.pair.b))} → ${esc(p.name)}</h2>
<p class="why">Why this pair: ${esc(picks.reasons[key] ?? "")}</p>
<div class="parents">${parentCard(r.pair.a)}${parentCard(r.pair.b)}</div>
<div class="cands">${r.candidates.map((c) => candCard(c, c.name.en === p.name)).join("")}</div>
<p class="final"><b>Pick: ${esc(p.name)}.</b> ${esc(p.why)}</p></section>`;
  })
  .join("\n");
const page1 = `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Arena fusions 20</title>${STYLE}</head><body>
<h1>Arena v2 · 20 designed fusions</h1>
<p class="dim">Card <a href="https://github.com/makscee/void-board/issues/1065">#1065</a>. Each pair: two Awoken units of the 40, three candidates from the night batch's designer (Opus) and quick tune (vs 16 seeded Awoken lines and 2 breakers; in band = wins 35-75%), and the one picked. The picks are discovered in the workshop Codex, credited to “the forge”. A fusion's PWR and HP are the stronger Awoken parent's plus half of the other's, rounded down (Maks, 10-14; it was their sum). Every candidate was re-tuned under those stats (the tune scales only the rule's numbers, ×0.25–×3). Magnet+Prism, Hydra+Lone Wolf and Alchemist+Hornet are back in place of Candle+Medic, Frostling+Sniper and Coin Thief+Hexer: under the sum they won 75-94%, now all nine of their candidates are in band. A pick that stays out of band after the retune is kept, its numbers shown in red: ${esc(designed.filter((k) => result(k).candidates.find((c) => c.name.en === picks.picks[k]!.name)?.state !== "ready").map((k) => picks.picks[k]!.name).join(", ") || "none")}. Retuned ${esc(batch.ranAt.slice(0, 16).replace("T", " "))} UTC.</p>
${sections}
<h2>10 more, waiting to be discovered</h2>
<p class="dim">Three sealed candidates each, unclaimed. Play them from <a href="arena-discover.html">arena-discover.html</a>.</p>
<ul>${hidden.map((k) => `<li>${esc(label(result(k).pair.a))} + ${esc(label(result(k).pair.b))}</li>`).join("")}</ul>
</body></html>`;

const base = "https://m1.twin-pogona.ts.net/arena-workshop/";
const code = /#invite=([^&]+)/.exec(invite)?.[1] ?? "";
const link = (hash: string) => `${base}#${[code ? `invite=${code}` : "", hash].filter(Boolean).join("&")}`;
const page2 = `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Arena discover</title>${STYLE}</head><body>
<h1>Arena v2 · discover a fusion</h1>
<p class="dim">Card <a href="https://github.com/makscee/void-board/issues/1065">#1065</a>. Each link signs you in to the workshop (your admin invite) and puts your run in the shop with two Awoken units of one pair on the line: one Fuse left. Keys only: <b>Tab</b> to the first unit, <b>Enter</b>, <b>F</b>, <b>Tab</b> to the second, <b>Enter</b>: the forge opens with three sealed cards; <b>1</b>–<b>3</b> picks one, <b>Enter</b> claims it. A checkpoint works again after you discovered other pairs; on a pair you already discovered it fuses straight into your pick.</p>
<p><a href="${link("")}">▶ A full fresh run</a> <span class="dim">(Home → Play)</span></p>
<ol>${hidden
  .map((k) => {
    const r = result(k);
    return `<li><a href="${link(`checkpoint=${r.pair.a}+${r.pair.b}`)}">${esc(label(r.pair.a))} + ${esc(label(r.pair.b))}</a><br><span class="dim">Look for: ${esc(picks.discover[k]!)}</span></li>`;
  })
  .join("")}</ol>
<p class="dim">The 20 already discovered: <a href="arena-fusions-20.html">arena-fusions-20.html</a>.</p>
</body></html>`;
writeFileSync(join(out, "arena-fusions-20.html"), page1);
writeFileSync(join(out, "arena-discover.html"), page2);
console.log(`wrote ${join(out, "arena-fusions-20.html")} and ${join(out, "arena-discover.html")}`);
