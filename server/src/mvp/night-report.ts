// The night batch's report page (makscee/void-board#1050): each pair with its
// three candidates in EN and RU, the rule as the card says it, the check and
// tune numbers and a trace line. Self-contained HTML for m1:~/reports.

import type { PairResult } from "./night-batch.js";

export interface BatchFile {
  day: string;
  designer: string;
  ranAt: string;
  seconds: number;
  parents: Record<string, { emoji?: string }>;
  results: PairResult[];
}

const esc = (s: string): string => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

export function nightReport(b: BatchFile): string {
  const emoji = (id: string) => b.parents[id]?.emoji ?? "";
  const all = b.results.flatMap((r) => r.candidates);
  const pairs = b.results
    .map((r) => {
      const head = `<h2>${emoji(r.pair.a)}${emoji(r.pair.b)} ${esc(r.pair.a)} + ${esc(r.pair.b)} <small>${r.pair.why}</small></h2>`;
      if (r.error) return `<section>${head}<p class="bad">design failed: ${esc(r.error)}</p></section>`;
      const cards = r.candidates
        .map((c) => {
          const chk = c.check;
          const nums = chk ? `score ${chk.score.toFixed(2)} · scale ×${chk.scale} · ${chk.games} games${chk.capped ? ` · ${chk.capped} capped` : ""}${chk.timeUp ? ` · ${chk.timeUp} time-up` : ""} · ${chk.inBand ? "in band" : "out of band"}` : "";
          return `<article class="card ${c.state}">
<div class="lean">${c.lean === "twist" ? "twist" : `lean ${c.lean === "A" ? esc(r.pair.a) : esc(r.pair.b)}`} · <b>${c.state === "ready" ? "ready to claim" : c.state === "word" ? "waiting for a word" : "rejected"}</b></div>
<div class="names"><span lang="en">${esc(c.name.en)}</span><span lang="ru">${esc(c.name.ru)}</span></div>
<div class="stats">${c.unit.base.hp} HP · ${c.unit.base.pwr} PWR</div>
<p lang="en">${esc(c.text.en || c.intent)}</p><p lang="ru">${esc(c.text.ru)}</p>
<p class="fl" lang="en">${esc(c.flavour.en)}</p><p class="fl" lang="ru">${esc(c.flavour.ru)}</p>
${c.why ? `<p class="why">${esc(c.why)}</p>` : ""}${nums ? `<p class="num">${nums}</p>` : ""}${chk?.trace ? `<p class="trace">${esc(chk.trace)}</p>` : ""}
<details><summary>designer's intent and rule JSON</summary><p>${esc(c.intent)}</p><pre>${esc(JSON.stringify({ unit: c.unit, abilities: c.abilities, statuses: c.statuses }, null, 1))}</pre></details>
</article>`;
        })
        .join("\n");
      return `<section>${head}<div class="cards">${cards}</div></section>`;
    })
    .join("\n");
  return `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Arena night batch ${esc(b.day)}</title>
<style>
:root{--bg:#f6f3ec;--panel:#fff;--text:#1d1b16;--dim:#6b665c;--ok:#2f7d4a;--warn:#a86b00;--bad:#b3322b;--line:#d9d3c5}
@media (prefers-color-scheme:dark){:root{--bg:#16150f;--panel:#211f18;--text:#ece7da;--dim:#9b9586;--ok:#6cc58c;--warn:#e0a640;--bad:#ec7068;--line:#3a372d}}
body{margin:0;background:var(--bg);color:var(--text);font:15px/1.45 system-ui,sans-serif}
main{max-width:1100px;margin:0 auto;padding:16px}
h1{margin:.2em 0}h2{font-size:1.1em;margin:1.4em 0 .5em}h2 small{color:var(--dim);font-weight:400}
.sum{color:var(--dim)}
.cards{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:12px}
.card{background:var(--panel);border:2px solid var(--line);border-radius:10px;padding:10px 12px}
.card.ready{border-color:var(--ok)}.card.word{border-color:var(--warn)}.card.rejected{border-color:var(--bad);opacity:.8}
.lean{font-size:.85em;color:var(--dim)}.names{font-size:1.15em;font-weight:600;display:flex;gap:.6em;flex-wrap:wrap}
.names [lang=ru]{color:var(--dim)}.stats{font-size:.85em;color:var(--dim)}
p{margin:.35em 0}p[lang=ru]{color:var(--dim)}.fl{font-style:italic}
.num,.why{font-size:.85em;color:var(--dim)}.trace{font-family:ui-monospace,monospace;font-size:.8em}.bad{color:var(--bad)}
pre{white-space:pre-wrap;word-break:break-word;font-size:.75em;max-height:300px;overflow:auto}
</style></head><body><main>
<h1>Night batch · ${esc(b.day)}</h1>
<p class="sum">${b.results.length} pairs · ${all.filter((c) => c.state === "ready").length} ready to claim · ${all.filter((c) => c.state === "word").length} waiting for a word · ${all.filter((c) => c.state === "rejected").length} rejected · designer ${esc(b.designer)} · ${b.seconds}s · ${esc(b.ranAt)}</p>
${pairs}
</main></body></html>
`;
}
