// Builds battle-targets.html (note 11 mockup) next to a copy of the client's
// style.css, then screenshots two frames (hit, chain) at 1440x900 and 390x844.
//   node build.mjs
import { readFileSync, writeFileSync, readdirSync, copyFileSync } from "node:fs";
import { createRequire } from "node:module";
const R = "/Users/admin/Work/arena-574/scout-r3";
const OUT = new URL(".", import.meta.url).pathname;
copyFileSync(`${R}/mobile/style.css`, `${OUT}style.css`);
const icons = readdirSync(`${R}/mobile/icons`).filter((f) => f.endsWith(".svg"));
const sprite = `<svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" style="display:none">${icons
  .map((f) => {
    const svg = readFileSync(`${R}/mobile/icons/${f}`, "utf8");
    const vb = /viewBox="([^"]+)"/.exec(svg)?.[1] ?? "0 0 512 512";
    const body = svg.replace(/^[\s\S]*?<svg[^>]*>/, "").replace(/<\/svg>\s*$/, "");
    return `<symbol id="i-${f.slice(0, -4)}" viewBox="${vb}">${body}</symbol>`;
  })
  .join("")}</svg>`;

const html = `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Battle targets mockup</title>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Chakra+Petch:wght@500;700&family=Rajdhani:wght@500;600;700&family=IBM+Plex+Mono:wght@500&display=swap">
<link rel="stylesheet" href="./style.css">
<style>
/* ---- the proposal (note 11): one SVG layer over the board ---- */
.fx { position: fixed; inset: 0; width: 100vw; height: 100vh; pointer-events: none; z-index: 5; overflow: visible; }
.fx path.beam { fill: none; stroke-linecap: round; }
.fx .now path.beam { stroke-width: var(--bw, 4px); filter: drop-shadow(0 0 6px currentColor); }
.fx .was { opacity: 0.32; }
.fx .was path.beam { stroke-width: 2.5px; stroke-dasharray: 2 7; }
.fx .head { filter: drop-shadow(0 0 5px currentColor); }
.k-dmg { color: #ff6b4d; } .k-heal { color: #3fdc8f; } .k-str { color: #ffa53d; } .k-shield { color: #5aa7ff; } .k-poison { color: #b48cff; } .k-summon { color: #2ee6d4; }
/* the source glows in the beam's colour; a target gets a ring */
.bv-card.src { box-shadow: 0 0 0 2px currentColor, 0 0 16px currentColor; }
.bv-card.tgt { box-shadow: 0 0 0 2px currentColor; }
.mock-note { position: fixed; right: 316px; top: 62px; z-index: 9; font: 11px "IBM Plex Mono", monospace; color: #8a93a6; }
@media (max-width: 1023.98px) { .fx .now path.beam { --bw: 3px; } .mock-note { display: none; } }
</style></head>
<body>${sprite}
<main id="app" data-screen="battle"></main>
<svg class="fx" id="fx"></svg>
<div class="mock-note" id="note"></div>
<script>
const F = new URLSearchParams(location.search).get("f") || "hit";
const ic = (id, s = 14, cls = "") => '<svg class="ic ' + cls + '" width="' + s + '" height="' + s + '" aria-hidden="true"><use href="#i-' + id + '"/></svg>';
// [id, emoji, name, pwr, hp, maxHp, trig icon, statuses]
const YOU = [
  ["a1", "🎯", "Victim", 2, 3, 6, "broken-heart", [["shield", "tone-shield", 1]]],
  ["a2", "🌵", "Spike", 1, 5, 5, "broken-heart", []],
  ["a3", "🦇", "Bat", 2, 4, 4, "crossed-swords", [["biceps", "tone-str", 1]]],
  ["a4", "📣", "Coach", 1, 5, 5, "flying-flag", []],
];
const THEM = [
  ["b1", "🗡️", "Squire", 3, 4, 4, "flying-flag", [["shield", "tone-shield", 2]]],
  ["b2", "🏹", "Archer", 2, 5, 5, "crossed-swords", []],
  ["b3", "🐀", "Rat", 2, 4, 4, "death-skull", []],
  ["b4", "💉", "Nurse", 1, 5, 5, "broken-heart", []],
  ["b5", "🍄", "Spore", 1, 5, 5, "death-skull", [["drop", "tone-poison", 1]]],
];
// Frames: who acted, the beams (now = this wave, was = earlier waves of the beat), badges, chips, caption.
const FRAMES = {
  hit: {
    note: "Frame 1: a strike. Squire strikes Victim: one short beam, front to front.",
    hp: { a1: 1 }, src: "b1", acting: "b1",
    beams: [{ from: "b1", to: "a1", k: "dmg", icon: "crossed-swords", now: true }],
    chips: { a1: ['<span class="bv-l damage">−2</span>'] },
    badges: { b1: ["crossed-swords", "spiky-explosion", "tone-dmg"] },
    floats: { a1: "−2" },
    cap: '<span class="bv-who ghost">Them</span><span class="bv-cn tone-enemy">Squire</span> strikes <span class="bv-cn tone-ally">Victim</span> → <b class="tone-dmg">−2</b>',
  },
  chain: {
    note: "Frame 2: a chain. Archer hits Victim (dim, wave 1) → Victim is hurt → Strength to every ally (bright, wave 2).",
    hp: { a1: 1 }, src: "a1", acting: "a1",
    beams: [
      { from: "b2", to: "a1", k: "dmg", icon: "spiky-explosion", now: false },
      { from: "a1", to: "a2", k: "str", icon: "biceps", now: true },
      { from: "a1", to: "a3", k: "str", icon: "biceps", now: true },
      { from: "a1", to: "a4", k: "str", icon: "biceps", now: true },
    ],
    chips: { a1: ['<span class="bv-l damage">−2</span>'], a2: ['<span class="bv-pst tone-str">' + ic("biceps", 13) + '×1</span>'], a3: ['<span class="bv-pst tone-str">' + ic("biceps", 13) + '×2</span>'], a4: ['<span class="bv-pst tone-str">' + ic("biceps", 13) + '×1</span>'] },
    badges: { a1: ["broken-heart", "biceps", "tone-str"], b2: ["crossed-swords", "spiky-explosion", "tone-dmg"] },
    floats: { a2: "+1 PWR", a3: "+1 PWR", a4: "+1 PWR" },
    cap: '<span class="bv-who you">You</span><span class="bv-cn tone-ally">Victim</span> ' + ic("broken-heart", 14, "tone-when") + ' is hurt → <span class="bv-ct tone-str">' + ic("biceps", 14) + ' Strength</span> ×1 on <span class="bv-cn tone-ally">Spike</span>, <span class="bv-cn tone-ally">Bat</span>, <span class="bv-cn tone-ally">Coach</span>',
  },
};
const fr = FRAMES[F];
document.getElementById("note").textContent = fr.note;
function cardHtml(u, side) {
  const [id, emoji, name, pwr, hp0, max, trig, sts] = u;
  const hp = fr.hp[id] ?? hp0;
  const pct = Math.round((hp / max) * 100);
  const sts2 = sts.map(([i, t, n]) => '<span class="bv-st ' + t + '">' + ic(i, 12) + '<b>' + n + '</b></span>').join("");
  const chips = fr.chips[id] ? '<div class="bv-changes"><button class="bv-change damage"><span class="bv-pill">' + fr.chips[id].join("") + '</span></button></div>' : "";
  const b = fr.badges[id];
  const badge = b ? '<button class="bv-badge still"><span class="bv-badge-pill">' + ic(b[0], 14, "tone-when") + '<span class="bv-badge-arrow">→</span>' + ic(b[1], 14, b[2]) + '</span></button>' : "";
  const fl = fr.floats[id];
  const float = fl ? '<span class="bv-float ' + (fl.startsWith("−") ? "damage" : "buff") + '" style="animation:none;opacity:1">' + fl + '</span>' : "";
  return '<div class="bv-slot"><div class="card ' + side + ' bv-card' + (fr.acting === id ? " acting" : "") + '" data-unit="' + id + '">' +
    '<span class="trig tone-when">' + ic(trig, 12) + '</span><div class="emoji">' + emoji + '</div><div class="name">' + name + '</div>' +
    '<div class="hpbar' + (pct <= 34 ? " low" : "") + '"><i style="width:' + pct + '%"></i></div>' +
    '<div class="stats big"><span class="p">' + ic("broadsword", 11, "stat-ic") + pwr + '</span><span class="h' + (hp < max ? " hurt" : "") + '">' + ic("hearts", 11, "stat-ic") + hp + '</span></div>' +
    '<div class="foot"><div class="bv-sts">' + sts2 + '</div>' + chips + '</div></div>' + badge + float + '</div>';
}
const cols = (n) => "grid-template-columns: repeat(" + Math.max(5, n) + ", minmax(0, 1fr))";
const turns = Array.from({ length: 7 }, (_, i) => '<div class="bv-tl-turn' + (i === 4 ? " on" : i < 4 ? " past" : "") + '"><div class="bv-tl-box">' + (i === 2 ? ic("death-skull", 14, "tone-enemy") : "") + '</div><div class="bv-tl-n mono dim">' + (i ? i : "Start") + '</div></div>').join("");
const logRows = [
  ["T4", '<span class="bv-who ghost">Them</span><span class="bv-cn tone-enemy">Squire</span> strikes <span class="bv-cn tone-ally">Victim</span> → <b class="tone-dmg">−2</b>'],
  ["", '<span class="bv-who you">You</span><span class="bv-cn tone-ally">Victim</span> strikes <span class="bv-cn tone-enemy">Squire</span> → ' + ic("shield", 14, "tone-shield") + '<span class="tone-shield"> 2 blocked</span>'],
  ["T5", '<span class="bv-who ghost">Them</span><span class="bv-cn tone-enemy">Archer</span> → <span class="bv-cn tone-ally">Victim</span> <b class="tone-dmg">−2</b>'],
  ["", '<span class="bv-who you">You</span><span class="bv-cn tone-ally">Victim</span> → Strength ×1 on <span class="bv-cn tone-ally">Spike</span>, <span class="bv-cn tone-ally">Bat</span>, <span class="bv-cn tone-ally">Coach</span>'],
];
const onRow = F === "hit" ? 0 : 3;
const app = document.getElementById("app");
app.innerHTML =
  '<div class="bv-screen"><div class="hud"><span>Round 6</span><span class="dim who">vs @bot-Ives</span><span>' + (F === "hit" ? "T4" : "T5") + '</span></div>' +
  '<div class="bv-board">' +
  '<div class="label bv-lab-them"><span class="bv-dk">← front · </span>Them<span class="bv-ph"> · front first</span></div>' +
  '<div class="slots bv-line theirs" style="' + cols(THEM.length) + '">' + THEM.map((u) => cardHtml(u, "ghost")).join("") + '</div>' +
  '<button class="bv-caption tappable"><span class="bv-cap">' + fr.cap + '</span></button>' +
  '<div class="bv-clash">' + ic("crossed-swords", 28, "tone-gold") + '</div>' +
  '<div class="slots bv-line mine" style="' + cols(YOU.length) + '">' + YOU.map((u) => cardHtml(u, "you")).join("") + '</div>' +
  '<div class="label bv-lab-you">You<span class="bv-ph"> · front first</span><span class="bv-dk"> · front →</span></div>' +
  '</div>' +
  '<div class="bv-tl"><div class="bv-tl-track">' + turns + '</div></div>' +
  '<div class="bv-below"><div class="bv-stillbox" style="display:none"></div><div class="bv-recent"></div></div>' +
  '<div class="row bv-controls"><button>‹</button><button>▶</button><button>›</button><button>2×</button><button>End</button><button>↻</button>' +
  '<div class="bv-keys dim"><kbd>Space</kbd> pause · <kbd>←</kbd><kbd>→</kbd> beat · <kbd>R</kbd> replay</div></div></div>' +
  '<div class="bv-sheet panel" data-tab="log"><div class="row bv-tabs"><button class="bv-tab on">Log</button><button class="bv-tab" disabled style="opacity:.35">Why</button></div>' +
  '<div class="bv-tab-body stack bv-why-body"></div><div class="bv-tab-body bv-log">' +
  logRows.map(([t, c], i) => '<button class="bv-log-row' + (t ? " first" : "") + (i === onRow ? " on" : "") + '"' + (i > onRow ? " hidden" : "") + '><span class="bv-log-t dim mono">' + t + '</span><span class="bv-log-c">' + c + '</span></button>').join("") +
  '</div></div>';

// ---- the beams: from the source card to each target, curved away from the board's middle ----
function draw() {
  const fx = document.getElementById("fx");
  const box = (id) => document.querySelector('[data-unit="' + id + '"]').getBoundingClientRect();
  const desk = innerWidth >= 1024;
  let out = "";
  for (const b of fr.beams) {
    const s = box(b.from), t = box(b.to);
    const sideOf = (id) => id[0];
    const same = sideOf(b.from) === sideOf(b.to);
    let x1 = s.left + s.width / 2, y1 = s.top + s.height / 2, x2 = t.left + t.width / 2, y2 = t.top + t.height / 2;
    let cx, cy;
    if (same) {
      // along a line: leave and land on the card's outer edge, arc outside the line
      const up = desk ? true : sideOf(b.from) === "b";
      const edge = (r) => (up ? r.top + 6 : r.bottom - 6);
      y1 = edge(s); y2 = edge(t);
      const lift = Math.min(70, 28 + Math.abs(x2 - x1) * 0.18);
      cx = (x1 + x2) / 2; cy = (up ? Math.min(y1, y2) - lift : Math.max(y1, y2) + lift);
    } else {
      // across the clash: a gentle bow, ending short of the target's middle
      cx = (x1 + x2) / 2 + (desk ? 0 : 24); cy = (y1 + y2) / 2 - (desk ? 46 : 0);
    }
    // stop the arrow 14 px short of the target's centre so the head sits on the card
    const ex = x2, ey = y2;
    const d = "M" + x1 + "," + y1 + " Q" + cx + "," + cy + " " + ex + "," + ey;
    // arrowhead angle at the end of the quadratic
    const ang = Math.atan2(ey - cy, ex - cx) * 180 / Math.PI;
    const k = "k-" + b.k;
    out += '<g class="' + (b.now ? "now " : "was ") + k + '">' +
      '<path class="beam" d="' + d + '" stroke="currentColor"/>' +
      (b.now ? '<circle cx="' + x1 + '" cy="' + y1 + '" r="5" fill="currentColor"/>' : "") +
      '<g class="head" transform="translate(' + ex + ',' + ey + ') rotate(' + ang + ')"><path d="M0,0 L-14,-8 L-10,0 L-14,8 Z" fill="currentColor"/></g>' +
      (b.now ? '<g transform="translate(' + (ex - 11) + ',' + (ey - 11) + ')"><circle cx="11" cy="11" r="13" fill="#07080c" stroke="currentColor" stroke-width="2"/><svg class="ic" x="1" y="1" width="20" height="20"><use href="#i-' + b.icon + '"/></svg></g>' : "") +
      '</g>';
    if (b.now) {
      document.querySelector('[data-unit="' + b.from + '"]').classList.add("src", k);
      document.querySelector('[data-unit="' + b.to + '"]').classList.add("tgt", k);
    }
  }
  fx.innerHTML = out;
}
document.fonts.ready.then(() => requestAnimationFrame(draw));
addEventListener("resize", draw);
</script></body></html>`;
writeFileSync(`${OUT}battle-targets.html`, html);

const require = createRequire(`${R}/package.json`);
const { chromium } = require("playwright");
const browser = await chromium.launch();
try {
  for (const [w, hgt, tag] of [[1440, 900, "desktop"], [390, 844, "phone"]]) {
    const page = await browser.newPage({ viewport: { width: w, height: hgt }, deviceScaleFactor: tag === "phone" ? 2 : 1 });
    for (const f of ["hit", "chain"]) {
      await page.goto(`file://${OUT}battle-targets.html?f=${f}`);
      await page.waitForTimeout(900);
      await page.screenshot({ path: `${OUT}targets-${f}-${tag}.png` });
    }
    await page.close();
  }
} finally {
  await browser.close();
}
console.log("ok");
