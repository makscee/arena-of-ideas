// The test scenes' index page (#1026): one row per scene in
// server/src/mvp/scenes.ts, grouped by feature, with its EN and RU links on
// the lab server. Visited rows grey out (this browser only).
//   npm run mvp:scenes-index -- [--out ~/reports/arena-scenes.html] [--base https://m1.twin-pogona.ts.net/arena-lab/]
import { writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { sceneList } from "../server/src/mvp/scenes.js";

const args = process.argv.slice(2);
const opt = (n: string) => {
  const i = args.indexOf(`--${n}`);
  return i >= 0 ? args[i + 1] : undefined;
};
const out = (opt("out") ?? "~/reports/arena-scenes.html").replace(/^~/, homedir());
const base = (opt("base") ?? "https://m1.twin-pogona.ts.net/arena-lab/").replace(/\/?$/, "/");
const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/"/g, "&quot;");

const scenes = sceneList();
/** Report pages that sit next to this index in ~/reports (#1065): name, file, what it shows. */
const REPORTS: [string, string, string][] = [
  ["20 designed fusions", "arena-fusions-20.html", "each pair, its 3 candidates and the pick (#1065)"],
  ["Discover a fusion", "arena-discover.html", "click-and-play checkpoints, one Fuse from each undiscovered pair (#1065)"],
];
const groups = [...new Set(scenes.map((s) => s.group))];
const link = (id: string, lang: string) => `${base}#scene=${id}&lang=${lang}`;
const rows = (g: string) =>
  scenes
    .filter((s) => s.group === g)
    .map(
      (s) => `<li class="row" data-id="${s.id}">
  <div class="what"><b>${esc(s.title)}</b><span>${esc(s.look)}</span><code>${s.id}${s.desktop ? " · desktop too" : ""}</code></div>
  <div class="go"><a href="${esc(link(s.id, "en"))}" target="_blank" rel="noopener">EN</a><a href="${esc(link(s.id, "ru"))}" target="_blank" rel="noopener">RU</a></div>
</li>`,
    )
    .join("\n");

const html = `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Arena test scenes</title>
<style>
:root{--bg:#f6f3ee;--fg:#1d1a22;--dim:#6b6474;--card:#fff;--line:#e2dcd2;--acc:#6b3fa0;--done:#b9b3bf}
@media (prefers-color-scheme: dark){:root:not([data-theme="light"]){--bg:#17141c;--fg:#eee9f3;--dim:#9a92a6;--card:#221e29;--line:#353041;--acc:#b48ef0;--done:#5a5463}}
:root[data-theme="dark"]{--bg:#17141c;--fg:#eee9f3;--dim:#9a92a6;--card:#221e29;--line:#353041;--acc:#b48ef0;--done:#5a5463}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--fg);font:16px/1.4 system-ui,sans-serif}
main{max-width:760px;margin:0 auto;padding:16px}
h1{font-size:22px;margin:4px 0}p.lead{color:var(--dim);margin:0 0 12px}
h2{font-size:15px;text-transform:uppercase;letter-spacing:.06em;color:var(--dim);margin:22px 0 8px}
ul{list-style:none;margin:0;padding:0;display:grid;gap:8px}
.row{display:flex;gap:10px;align-items:center;background:var(--card);border:1px solid var(--line);border-radius:12px;padding:10px 10px 10px 14px}
.what{flex:1;min-width:0;display:flex;flex-direction:column}.what span{color:var(--dim);font-size:14px}.what code{color:var(--dim);font-size:12px}
.go{display:flex;gap:6px}.go a{display:inline-flex;align-items:center;justify-content:center;min-width:52px;min-height:48px;border-radius:10px;background:var(--acc);color:#fff;font-weight:600;text-decoration:none}
.row.seen{opacity:.55}.row.seen .go a{background:var(--done)}
button{font:inherit;border:1px solid var(--line);background:var(--card);color:var(--fg);border-radius:8px;padding:8px 12px;min-height:40px}
</style></head><body><main>
<h1>Arena test scenes</h1>
<p class="lead">${scenes.length} scenes, each opens its screen from one link on the lab server (tailnet). On the phone you see the phone layout; open the same link on a computer for desktop. Add <code>&amp;theme=brass|felt</code> or <code>&amp;speed=2</code> to a link. Visited rows grey out. <button id="reset">Clear visited</button></p>
<h2>Reports</h2>
<ul>
${REPORTS.map(([t, f, look]) => `<li class="row" data-id="report-${f}">\n  <div class="what"><b>${esc(t)}</b><span>${esc(look)}</span><code>${f}</code></div>\n  <div class="go"><a href="${f}" target="_blank" rel="noopener">Open</a></div>\n</li>`).join("\n")}
</ul>
${groups.map((g) => `<h2>${esc(g)}</h2>\n<ul>\n${rows(g)}\n</ul>`).join("\n")}
</main>
<script>
const KEY="arena-scenes.seen";
const load=()=>{try{return JSON.parse(localStorage.getItem(KEY)||"{}")}catch{return {}}};
const save=(s)=>{try{localStorage.setItem(KEY,JSON.stringify(s))}catch{}};
const paint=()=>{const s=load();document.querySelectorAll(".row").forEach(r=>r.classList.toggle("seen",!!s[r.dataset.id]))};
document.querySelectorAll(".go a").forEach(a=>a.addEventListener("click",()=>{const s=load();s[a.closest(".row").dataset.id]=1;save(s);paint()}));
document.getElementById("reset").addEventListener("click",()=>{save({});paint()});
paint();
</script>
</body></html>
`;
writeFileSync(out, html);
console.log(`wrote ${out}: ${scenes.length} scenes in ${groups.length} groups, links on ${base}`);
