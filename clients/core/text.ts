// The game as plain text (#957), for clients without a screen of cards: an
// agent over MCP, a Claude Code pane, a terminal. Built on the same shared
// helpers the phone uses (src/mvp/form-text.ts for card text, src/mvp/trace.ts
// for battle rows and the why-chain, src/glossary.ts), so the words match the
// app's. English by default; `lang: "ru"` reads Russian where the app does.
import type { BattleRecord, FightResult, HomeView, LineUnit, MvpContent, MvpRules, RunView, UnitContent } from "../../src/mvp/contract.js";
import type { Lang } from "../../src/describe.js";
import { GLOSSARY, termDef, type FixedTermId, type TermId } from "../../src/glossary.js";
import { formText } from "../../src/mvp/form-text.js";
import { beatPlayOf, endCaption, foldTurnsOf, isLogFold, logRowsOf, setTraceLang, sidesOf, stepsOf, traceOf } from "../../src/mvp/trace.js";
import { displayNames } from "../../src/trace.js";
import type { Side } from "../../src/types.js";

export interface TextOpts {
  lang?: Lang;
}

const ruLang = (o: TextOpts): Lang | undefined => (o.lang === "ru" ? "ru" : undefined);

/** A pool unit's name in `lang` (its Russian name once translated). */
export function unitName(unitId: string, fallback: string, content: MvpContent, o: TextOpts = {}): string {
  if (o.lang !== "ru") return fallback;
  const u = content.units.find((x) => x.id === unitId) ?? content.left?.find((x) => x.id === unitId);
  return u?.texts?.ru?.name ?? fallback;
}

const formWord = (u: LineUnit, o: TextOpts): string =>
  u.kind === "fused" ? (o.lang === "ru" ? "слияние" : "fused") : u.form === "awoken" ? (o.lang === "ru" ? "пробуждён" : "Awoken") : `${u.copies}/3`;

/** One unit on the board: "🛡️ Keeper 2/5 Awoken: <what it does>". */
export function lineUnitText(u: LineUnit, content: MvpContent, o: TextOpts = {}): string {
  const name = u.kind === "fused" ? u.name : unitName(u.unitId, u.name, content, o);
  return `${u.emoji} ${name} ${u.stats.pwr}/${u.stats.hp} ${formWord(u, o)}: ${formText(u.recipe, content.abilities, ruLang(o))}`;
}

/** A unit's card: name, tier, stats, what it's about, both forms. */
export function cardText(u: UnitContent, content: MvpContent, o: TextOpts = {}): string {
  const ru = o.lang === "ru";
  const line = (ru && u.texts?.ru?.line) || u.archetype;
  return [
    `${u.emoji} ${unitName(u.id, u.name, content, o)} · T${u.tier} · ${u.base.pwr}/${u.base.hp}`,
    line,
    `${ru ? "Спит" : "Sleeping"}: ${formText(u.forms.sleeping, content.abilities, ruLang(o))}`,
    `${ru ? "Пробуждён" : "Awoken"}: ${formText(u.forms.awoken, content.abilities, ruLang(o))}`,
  ].join("\n");
}

/** A fight's result in one line: "Round 3 vs @Mira: win (♥3)". */
export function fightText(f: FightResult): string {
  const what = f.kind === "crown" ? "Crown" : f.kind === "playoff" ? "Playoff" : `Round ${f.round}`;
  return `${what} vs @${f.opponent.player.name}: ${f.outcome}${f.heartsLost ? ` (−${f.heartsLost}♥, ♥${f.heartsAfter} left)` : ""} · battle ${f.battleId}`;
}

/** A run as a client acts on it: the header, then the numbered shop, line
 * and bench. The numbers are what a Decision takes: `buy {slot}` the shop's,
 * `sell {index}`, `reorder {from, to}` and `fuse {first, second}` the board
 * slots (the line from 0, then the bench). */
export function runText(run: RunView, content: MvpContent, rules: MvpRules, o: TextOpts = {}): string {
  const out: string[] = [];
  const round = run.phase === "crown" ? "Crown" : run.round > rules.rounds ? "Crown" : `Round ${run.round}/${rules.rounds}`;
  const vs = run.nextOpponent ? ` · vs @${run.nextOpponent.player.name}` : "";
  out.push(`Arena · ${round} · ♥${run.hearts} · ${run.gold}g · ${run.wins}W ${run.losses}L${vs} · ${run.phase}${run.endedBy ? ` (${run.endedBy})` : ""}`);
  if (run.phase === "over") {
    if (run.rating) out.push(`Rating ${run.rating.before} → ${run.rating.after}`);
  }
  if (run.gift?.length) out.push(`GIFT   ${run.gift.map((id, i) => `${i}: ${unitName(id, content.units.find((u) => u.id === id)?.name ?? id, content, o)}`).join(" · ")}  (gift {pick} or null to skip)`);
  if (run.offers.length) {
    out.push("SHOP");
    for (const of of run.offers) {
      const u = content.units.find((x) => x.id === of.unitId);
      const name = u ? `${u.emoji} ${unitName(u.id, u.name, content, o)} ${u.base.pwr}/${u.base.hp}` : of.unitId;
      out.push(`  [${of.slot}] ${name} T${of.tier} ${of.cost}g${of.locked ? " 🔒" : ""}${u ? `: ${formText(u.forms.sleeping, content.abilities, ruLang(o))}` : ""}`);
    }
  }
  out.push(`LINE (${run.line.length}/${rules.lineSize}, front first)`);
  run.line.forEach((u, i) => out.push(`  [${i}] ${lineUnitText(u, content, o)}`));
  if (run.bench.length) {
    out.push("BENCH");
    run.bench.forEach((u, i) => out.push(`  [${rules.lineSize + i}] ${lineUnitText(u, content, o)}`));
  }
  const last = run.fights.at(-1);
  if (last) out.push(`LAST   ${fightText(last)}`);
  return out.join("\n");
}

/** The title screen as text: who you are, the day and its champion, the runs
 * going (ranked and practice), the ideas held. A home with no rating is a
 * caller with no login. */
export function homeText(h: HomeView, content: MvpContent, o: TextOpts = {}): string {
  const out: string[] = [];
  const r = h.rating;
  if (r) out.push(`@${r.player.name} · rating ${r.rating} · ${r.runs} runs · ${r.slays} slays · ${r.daysAsChampion} days as champion`);
  else out.push("Not logged in: use login (email code) or set ARENA_TOKEN.");
  out.push(`Day ${h.day.seq} (${h.day.day}), ends ${h.day.endsAt} · ${h.day.slayers} slayer${h.day.slayers === 1 ? "" : "s"} so far`);
  const c = h.day.champion;
  if (c) out.push(`Champion: @${c.player.name} (since ${c.since})`, ...c.line.map((u) => `  ${lineUnitText(u, content, o)}`));
  else out.push("Champion: none yet");
  if (r) {
    out.push(`Ranked run: ${h.activeRunId ?? "none"} (needs your tap on the phone; agents can read it) · Practice run: ${h.practiceRunId ?? "none (start_run begins one)"}`);
    if (h.ideas) out.push(`Ideas: ${h.ideas.held}/${h.ideas.perDay} held${h.ideas.ready ? `, ${h.ideas.ready} ready for your pick` : ""}`);
  }
  return out.join("\n");
}

/** Runs `fn` with the trace captions in `lang`, then puts English back
 * (src/mvp/trace.ts keeps one language at a time). */
function inLang<T>(o: TextOpts, fn: () => T): T {
  setTraceLang(ruLang(o));
  try {
    return fn();
  } finally {
    setTraceLang(undefined);
  }
}

export interface BattleTextOpts extends TextOpts {
  /** The reader's side: the end reads "You win" / "They win". */
  you?: Side;
  /** "summary": the teams, the end and the key rows; "steps": every row, by turn. */
  view?: "summary" | "steps";
}

/** A battle replayed as text: the teams, then its rows turn by turn, as the
 * phone's Log tab reads them (repeated turns folded), then the end. Each
 * row ends with `#<id>`: the event `traceText` explains. */
export function battleText(b: BattleRecord, o: BattleTextOpts = {}): string {
  return inLang(o, () => {
    const log = b.log;
    const name = displayNames(log);
    const sides = sidesOf(log);
    const p = o.you ? { you: o.you } : { sideName: (s: Side) => `@${(s === "A" ? b.player : b.opponent).name}` };
    const beats = beatPlayOf(log, stepsOf(log, name, sides, p), name);
    const rows = logRowsOf(log, beats, name);
    const team = (s: Side) => (s === "A" ? b.teamA : b.teamB).map((u) => `${u.emoji} ${u.name} ${u.stats.pwr}/${u.stats.hp}`).join(", ");
    const out = [`Battle ${b.battleId} · ${b.kind}${b.kind === "round" ? ` ${b.round}` : ""} · @${b.player.name} (A) vs @${b.opponent.name} (B)`, `A: ${team("A")}`, `B: ${team("B")}`];
    const end = log.at(-1);
    const endLine = end?.type === "BattleEnd" ? endCaption(end.winner, p) : "";
    if (o.view === "summary") {
      out.push(`${rows.length} rows over ${end?.type === "BattleEnd" ? end.turns : "?"} turns`, endLine);
      return out.join("\n");
    }
    let turn = -1;
    for (const it of foldTurnsOf(log, rows, name)) {
      if (isLogFold(it)) {
        turn = it.turnTo;
        out.push(`${it.turn === it.turnTo ? `Turn ${it.turn}` : `Turns ${it.turn}–${it.turnTo}`}`, `  … ${it.caption}`);
        continue;
      }
      if (it.turn !== turn) {
        turn = it.turn;
        out.push(turn === 0 ? "Start" : `Turn ${turn}`);
      }
      const side = it.subjectSide ? `${it.subjectSide} ` : "";
      out.push(`  ${side}${it.caption}  #${it.eventIds.at(-1)}`);
    }
    // The last row is usually the end itself ("You win").
    if (!out.at(-1)?.includes(endLine)) out.push(endLine);
    return out.join("\n");
  });
}

/** Why an event happened: its cause chain, nearest first ("−5 ← Archer ← Smith"). */
export function traceText(b: BattleRecord, eventId: number, o: TextOpts = {}): string {
  return inLang(o, () => {
    const t = traceOf(b.log, eventId);
    const lines = [t.text, ...t.links.map((l) => `  ← ${l.name}${l.side ? ` (${l.side})` : ""} via ${l.via} #${l.eventId}`)];
    return lines.join("\n");
  });
}

/** The glossary: every term the cards use, "Label: rule". Statuses come from
 * `content` when given (player-made ones too). */
export function glossaryText(content?: MvpContent, o: TextOpts = {}): string {
  const ids: TermId[] = [...(Object.keys(GLOSSARY) as FixedTermId[]), ...Object.keys(content?.statuses ?? {}).map((s) => `status:${s}` as TermId)];
  const seen = new Set<string>();
  const out: string[] = [];
  for (const id of ids) {
    const d = termDef(id, content?.statuses ?? {}, ruLang(o));
    if (!d || seen.has(d.label)) continue;
    seen.add(d.label);
    out.push(`${d.label}: ${d.tip}`);
  }
  return out.join("\n");
}
