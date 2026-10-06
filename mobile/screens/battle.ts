// The battle viewer (mission #574, slice 9): playback with one-line captions
// (cause → effect) and the acting unit lit, tap any change to trace its chain,
// "why I lost" after a loss, speed 1×/2× and skip. Round 2 (R2-12) plays it
// beat by beat (a strike or turn end plus its cascade, in waves) with motion.
// The logic is pure and tested in src/mvp/trace.ts; this file only draws it.
//
// It opens any battle (a fight, a playoff game, the champion history); onDone
// goes on. R2-14 adds the control bar, pinned at the screen's foot: ‹ ▶/❚❚ ›,
// 1×/2×/4× (remembered per viewer, 2× from round 4), End (battle-end: the end
// card at once) and Replay from the start; on desktop Space, ←/→ and R. Played
// to the end (or ▶ on the last beat), the end card shows the outcome, damage
// by unit, 2–3 key moments that replay from there, and Replay / Why I lost
// (or won) / Continue (battle-done, which calls onDone).
//
// R2-16, at 1024px and wider (style.css, data-screen="battle"): the two lines
// face each other, fronts in the middle; a turn timeline under the board
// scrubs the fight (click or drag; boardAt is pure); a side panel holds Why
// and Log. Every piece is in the DOM at any width and CSS picks the layout,
// so crossing 1024px needs no redraw. The phone keeps its stacked rows.
import { boardAt, type BoardState, type BoardUnit } from "../../src/board";
import type { BattleRecord, BattleUnit, FightResult, MvpContent, RunView } from "../../src/mvp/contract";
import { STATUS_TERMS, termDef, termIcon, triggerLabel, type IconId, type TermId } from "../../src/glossary";
import { beatPlayOf, causeOf, chainOf, damageByUnit, keyMomentsOf, stepsOf, timelineOf, timingOf, traceOf, turnLabel, whyILost as lossChains, sidesOf, type Chain, type ChainNode, type Cause, type Change, type KeyMoment, type Step, type Trace, type WhenOf } from "../../src/mvp/trace";
import { displayNames, type NameOf } from "../../src/trace";
import type { Side, UnitDef } from "../../src/types";
import { card, formRich, unitSheet } from "../ui/card";
import { app, button, closable, fitText, h, isDesktop, onGone, onKeys, onLeave, screen, show } from "../ui/dom";
import { icon } from "../ui/icon";
import { statusesShown, STATUS_ROW_FALLBACK } from "../ui/status-row";

/** The least room the phone's end card takes under the caption (its word,
 * its line, two key moments and its buttons); with less (a phone on its side)
 * it covers the caption, then the board (placeEnd). */
const END_MIN_PX = 240;

/** The end card's Damage rows a side, until "All n" (R2-17 batch E). */
const DMG_ROWS = 3;

/** How long the line-up shows before the first beat, at 1×. */
const LINEUP_MS = 400;
/** How long a landed wave's motion runs, in animation time (real time × speed): the longest animation (a killing blow's shake, then its 0.5 s pop, ends at 0.92 s; a float, 0.7 s, after up to 160 ms). A beat holds its last wave at least 0.7 s, so a beat change cuts motion off at most in its fade. */
const MOTION_MS = 1000;
/** How long a card's When icon flashes when it fires (R3-19). */
const TRIG_FLASH_MS = 400;
/** When a hit's shake ends, in animation ms after its wave lands (80 ms in, 340 ms long). */
const SHAKE_END_MS = 420;

/** The changes that float up from a card. */
const FLOATS = new Set<Change["kind"]>(["damage", "heal", "buff", "debuff", "summon"]);

/** A unit as captions name it: its id between two private-use marks, so
 * richCaption colours each name by that unit's own side (a mirror match or a
 * summon sharing a name never takes the other side's colour). */
const TAGGED: NameOf = (id) => `\uE000${id}\uE001`;
const TAG = /\uE000([^\uE001]*)\uE001/g;

/** The playback speeds, and where the viewer's choice is kept (per device). */
const SPEEDS = [1, 2, 4] as const;
const SPEED_KEY = "arena.battleSpeed";
/** Without a remembered choice, battles from this round on play at 2×. */
const FAST_FROM_ROUND = 4;

/** The viewer's speed: the one they chose last, else 1×, or 2× from round 4.
 * localStorage may throw or be empty: then the default. */
export function speedFor(round: number, stored: string | null): number {
  const s = Number(stored);
  if ((SPEEDS as readonly number[]).includes(s)) return s;
  return round >= FAST_FROM_ROUND ? 2 : 1;
}
function storedSpeed(): string | null {
  try {
    return localStorage.getItem(SPEED_KEY);
  } catch {
    return null;
  }
}
function storeSpeed(s: number): void {
  try {
    localStorage.setItem(SPEED_KEY, String(s));
  } catch {
    // a private window: the choice lasts this battle
  }
}

/** What a run's fight adds to the end card (main.ts, R2-17 batch E): the run
 * as the fight left it (round, hearts, record, what the result costs or
 * wins), the in-run menu (☰ in the HUD, Esc) and where its last button goes
 * ("Next round", "To the Crown", "See the run"). The end card is the fight's
 * one result: no result screen follows it. */
export interface RunOutro {
  /** The end card's run line, rebuilt each time the card draws. */
  status: () => Node[];
  /** The last button's label; it calls onDone. */
  doneLabel: string;
  /** Opens the run menu; its Resume (or Esc, or a tap outside it) calls
   * resume when the battle was playing as it opened (R2-17 batch F). */
  menu: (resume?: () => void) => void;
  /** Where the menu's errors land (Codex or Title menu failing): the battle
   * shows it over everything, under the HUD (R2-17 batch F). */
  error?: HTMLElement;
}

/** A phone on its side: compact cards with one status row (style.css, R2-17 batch E). */
const shortScreen = matchMedia("(max-width: 1023.98px) and (max-height: 520px)");

/** Reduced motion: nothing moves, and beats hold a little longer. */
const reduced = () => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;

export function battleScreen(a: { battle: BattleRecord; content: MvpContent; you?: Side; fight?: FightResult; run?: RunView; outro?: RunOutro; onDone: () => void }): void {
  const { battle } = a;
  // Without a.you (a playoff game, a champion's battle) nobody here is "you":
  // side A draws in the "you" colour, but nothing reads as your win or loss.
  const you: Side = a.you ?? "A";
  const them: Side = you === "A" ? "B" : "A";
  const outcome = battle.winner === "draw" ? "draw" : battle.winner === you ? "win" : "loss";
  const owner = (s: Side) => `@${(s === "A" ? battle.player : battle.opponent).name}`;
  const log = battle.log;
  const name = displayNames(log);
  const sides = sidesOf(log);
  // Playback goes beat by beat (round 2, R2-12): a strike or a turn end plus
  // everything it sets off, its effects landing in quick waves.
  const beats = beatPlayOf(log, stepsOf(log, TAGGED, sides, a.you ? { you: a.you } : { sideName: owner }), TAGGED);
  const units = new Map<string, BattleUnit>([...battle.teamA, ...battle.teamB].map((u) => [u.id, u]));
  const emojiOf = (id: string) => units.get(id)?.emoji ?? "✨";
  const whenOf = whenLookup(units, a.content);
  // The end card's numbers (R2-14).
  const damage = damageByUnit(log, name, sides);
  // Tagged names: the card colours each unit by its side, like the captions.
  const moments = keyMomentsOf(log, beats, TAGGED, sides);

  // What richCaption highlights besides units (tagged by id): this battle's
  // statuses and the fixed words. Longest first, so "Rat King" beats "Rat".
  const logStatuses = new Set(log.flatMap((e) => (e.type === "StatusApplied" ? [e.status] : [])));
  const esc = (w: string) => w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const words = [...new Set([...Object.keys(STATUS_TERMS), ...logStatuses, "Fatigue"])].filter(Boolean).sort((p, q) => q.length - p.length);
  const captionTerms = new RegExp(`\\(\\d+ absorbed\\)|(?<![\\p{L}\\d])(?:${words.map(esc).join("|")})(?![\\p{L}\\d])|\\b(?:PWR|HP)\\b|[−+]\\d+`, "gu");

  let at = -1; // index of the beat on screen; -1 = the line-up before the first beat
  let wave = 0; // waves of that beat landed so far, minus one
  /** When each wave of the open beat landed (performance.now()), set only
   * when the playhead moved forward by itself. Every render re-applies the
   * motion of each landed wave, offset by its age, so a later wave's render
   * doesn't cut an earlier wave's lunge, shake or float short; empty after a
   * manual step, so nothing moves then. */
  let landed: number[] = [];
  // Set before setSpeedVar() runs, so motion and beat timing start in step.
  let speed = speedFor(battle.round, storedSpeed());
  let playing = true;
  let timer: ReturnType<typeof setTimeout> | null = null;
  let trace: Trace | null = null;
  /** Why's chain for the traced change (R2-15), and the step whose moment is on the board. */
  let chain: Chain | null = null;
  let chainAt: number | null = null;
  /** The changes the tapped chip held (one unit, one step), when it held more than one. */
  let traceGroup: Change[] = [];
  /** The end card is up (played out, End, or ▶ on the last beat). */
  let finished = false;
  /** The end card has been up once: from then on (a replay, a key moment)
   * the timeline shows every mark, for finding your way; before, a mark shows
   * once the playhead reaches it, so it never tells who falls when (R2-17 batch E). */
  let seenEnd = false;
  /** The desktop side panel's tab (R2-16). It opens on Log; a trace opening
   * switches to Why, and Why turns off again once nothing is traced (R3-17). */
  let tab: "why" | "log" = "log";
  /** Why's "Turn N" step shows the board as turn N starts (the end of turn
   * N−1): the HUD then reads turn N (R2-17). Any other move clears it. */
  let hudTurn: number | null = null;
  /** The end card's Damage list shows every unit (All n), not the top three a side. */
  let dmgAll = false;
  /** The board on screen and its turn, as render() last drew them: a card tap's Now sheet reads them (R3-18). */
  let shownBoard: BoardState | null = null;
  let shownTurn = 0;
  /** Each card's floating numbers, set by motion() and hung in its slot. */
  const floatsOf = new WeakMap<HTMLElement, HTMLElement[]>();

  /** Motion runs at the playback speed (CSS reads --bv-sp), so 2× never cuts it. */
  const setSpeedVar = () => { for (const row of [enemy, mine, clash]) row.style.setProperty("--bv-sp", String(speed)); };

  const hud = h("div", { class: "hud" });
  // A run's fight keeps the run's ☰ (Codex, Rules, Title menu, Abandon), as the shop has it.
  const menuBtn = a.outro ? button("☰", () => openMenu(), "menu-btn", "menu-open") : null;
  if (menuBtn) {
    menuBtn.setAttribute("aria-label", "Menu");
    menuBtn.title = "Menu (Esc)";
  }
  function openMenu(): void {
    const was = playing && !finished;
    pause();
    render();
    a.outro?.menu(was ? () => { if (!playing && !finished) play(); } : undefined);
  }
  // The menu's errors, over the board and the end card, under the HUD; ✕ clears it.
  const errBox = a.outro?.error
    ? h("div", { class: "bv-error panel row", role: "alert", "data-testid": "battle-error" }, a.outro.error, button("✕", () => { a.outro!.error!.textContent = ""; }, "bv-close", "battle-error-close"))
    : null;
  const enemy = h("div", { class: "slots bv-line theirs", "data-testid": "battle-them" });
  const mine = h("div", { class: "slots bv-line mine", "data-testid": "battle-you" });
  const caption = h("button", { class: "bv-caption", "data-testid": "caption" });
  /** Reduced motion's list of the beat's changes: below your line, its height
   * fixed, so nothing on the board moves while it fills. */
  const still = h("div", { class: "bv-stillbox" });
  const recent = h("div", { class: "bv-recent", "data-testid": "recent" });
  const sheet = h("div", { class: "bv-sheet panel", "data-testid": "trace" });
  // The desktop side panel: Why and Log tabs over the same sheet (the phone shows only Why, as a sheet).
  const whyBody = h("div", { class: "bv-tab-body stack bv-why-body" });
  const logBody = h("div", { class: "bv-tab-body bv-log", "data-testid": "battle-log" });
  const whyTab = button("Why", () => setTab("why"), "bv-tab", "tab-why");
  const logTab = button("Log", () => setTab("log"), "bv-tab", "tab-log");
  sheet.append(h("div", { class: "row bv-tabs", role: "tablist" }, whyTab, logTab), whyBody, logBody);
  const timeline = h("div", { class: "bv-tl", "data-testid": "timeline", role: "slider", "aria-label": "Turn timeline: click or drag to scrub", tabindex: "-1" });
  const clashMark = icon("crossed-swords", 28, "tone-gold");
  // The clash mark (desktop) carries fatigue's badge (R3-19).
  const clash = h("div", { class: "bv-clash" }, clashMark);
  const end = h("div", { class: "bv-end panel stack", "data-testid": "end-card" });
  const playBtn = button("❚❚", () => (playing ? pause() : play()), "", "battle-play");
  const backBtn = button("‹", () => back(), "", "battle-back");
  const fwdBtn = button("›", () => forward(), "", "battle-step");
  const speedBtn = button(`${speed}×`, () => setSpeed(SPEEDS[(SPEEDS.indexOf(speed as 1) + 1) % SPEEDS.length]!), "", "battle-speed");
  const endBtn = button("End", () => finish(), "", "battle-end");
  const replayBtn = button("↻", () => replay(), "", "battle-replay");
  replayBtn.setAttribute("aria-label", "Replay from the start");
  // Desktop's keys, as the mockup lists them (the phone hides the line).
  const kbd = (k: string) => h("kbd", {}, k);
  const keys = h("div", { class: "bv-keys dim", "data-testid": "battle-keys", "aria-hidden": "true" }, kbd("Space"), " pause · ", kbd("←"), kbd("→"), " beat · ", kbd("R"), " replay");
  const controls = h("div", { class: "row bv-controls" }, backBtn, playBtn, fwdBtn, speedBtn, endBtn, replayBtn, keys);
  // How long playback should take at 1× (the line-up plus every beat), for
  // the e2e to compare with what the screen takes.
  controls.dataset.planMs = String(LINEUP_MS + beats.reduce((t, b) => t + timingOf(b).ms, 0));

  caption.addEventListener("click", () => {
    const st = stepOn();
    const c = st?.changes[0];
    if (c) openTrace(c.eventId, st!.changes.filter((x) => x.unit === c.unit));
  });

  const lastWave = (i: number) => (beats[i]?.waves.length ?? 1) - 1;
  /** The wave on screen: the step whose caption shows. */
  const stepOn = () => beats[at]?.waves[wave];

  /** Plays the next wave of the open beat, or the next beat's first wave.
   * The wait counts from when the wave on screen landed, not from when its
   * render ended, so drawing time doesn't add up over a battle. */
  function schedule(): void {
    if (timer) clearTimeout(timer);
    const slow = reduced() ? 1.25 : 1;
    let ms: number;
    if (at < 0) ms = LINEUP_MS;
    else {
      const t = timingOf(beats[at]!);
      ms = wave < lastWave(at) ? t.at[wave + 1]! - t.at[wave]! : t.ms - t.at[wave]!;
    }
    const since = landed[wave] !== undefined ? performance.now() - landed[wave]! : 0;
    timer = setTimeout(() => {
      if (at >= 0 && wave < lastWave(at)) {
        wave++;
        landed[wave] = performance.now();
        render();
        return schedule();
      }
      if (at >= beats.length - 1) return finish();
      hudTurn = null;
      at++;
      wave = 0;
      landed = [performance.now()];
      render();
      schedule();
    }, Math.max(0, (ms * slow) / speed - since));
  }
  /** Sets the speed (and the motion's --bv-sp with it), remembered for the next battle. */
  function setSpeed(s: number): void {
    speed = s;
    speedBtn.textContent = `${speed}×`;
    setSpeedVar();
    storeSpeed(speed);
    if (playing) schedule();
  }
  const atEnd = () => at >= beats.length - 1 && wave >= lastWave(at);
  function play(): void {
    // ▶ on the last beat shows the end card; Replay starts over.
    if (finished || atEnd()) return finish();
    playing = true;
    trace = null;
    landed = [];
    playBtn.textContent = "❚❚";
    render();
    schedule();
  }
  function pause(): void {
    playing = false;
    playBtn.textContent = "▶";
    if (timer) clearTimeout(timer);
    timer = null;
  }
  /** Jumps to beat i, every wave landed. */
  function go(i: number): void {
    hudTurn = null;
    at = Math.max(-1, Math.min(beats.length - 1, i));
    wave = lastWave(at);
    landed = [];
    render();
  }
  function openTrace(eventId: number, group: Change[] = []): void {
    pause();
    trace = traceOf(log, eventId, name, sides);
    // Tagged names: Why's steps colour each unit by its own side (R2-17).
    chain = chainOf(log, eventId, { name: TAGGED, sides, whenOf });
    chainAt = null;
    traceGroup = group.length > 1 ? group : [];
    tab = "why";
    render();
  }
  /** Moves the playhead to event `id`'s moment, Why left open: the wave that
   * shows it (every wave before it landed), else, for a turn or a strike
   * that no wave shows, the start of the beat that holds it. */
  function seekEvent(id: number): void {
    pause();
    finished = false;
    let b = beats.findIndex((pb) => pb.waves.some((w) => w.eventIds.includes(id)));
    let w = b >= 0 ? beats[b]!.waves.findIndex((x) => x.eventIds.includes(id)) : 0;
    if (b < 0) {
      // A root (Turn 6, the battle's start) shows as the board before its first beat.
      const next = beats.findIndex((pb) => pb.end >= id);
      if (next < 0) b = beats.length - 1;
      else if (id >= beats[next]!.start && log[id]?.type === "Strike") b = next;
      else b = next - 1;
      w = lastWave(b);
    }
    at = b;
    wave = Math.max(0, w);
    landed = [];
    chainAt = id;
    // "Turn N" (its start, or its first pair facing) is the board as turn N
    // begins: the HUD says turn N, not the turn that just ended.
    const root = log[id];
    hudTurn = root && (root.type === "TurnStart" || root.type === "PairFaced") ? root.turn : null;
    render();
  }
  function back(): void {
    pause();
    finished = false;
    go(at - 1);
  }
  function forward(): void {
    pause();
    if (atEnd()) return finish();
    go(at + 1);
  }
  /** Jumps to the result: the last beat's board under the end card. */
  function finish(): void {
    pause();
    hudTurn = null;
    finished = true;
    seenEnd = true;
    trace = null;
    at = beats.length - 1;
    wave = lastWave(at);
    landed = [];
    render();
  }
  /** Plays from beat i (-1: the line-up), the end card put away. */
  function playFrom(i: number): void {
    pause();
    hudTurn = null;
    finished = false;
    trace = null;
    at = Math.max(-1, Math.min(beats.length - 1, i));
    wave = 0;
    landed = at >= 0 ? [performance.now()] : [];
    playing = true;
    playBtn.textContent = "❚❚";
    render();
    schedule();
  }
  const replay = () => playFrom(-1);

  // ---------- the desktop side panel and timeline (R2-16) ----------

  function setTab(t: "why" | "log"): void {
    tab = t;
    render();
    if (t === "log") logRows[curRow()]?.scrollIntoView({ block: "nearest" });
  }

  /** The Log: one row per wave, every beat of the fight, built once; render()
   * shows the rows played so far and lights the one on screen. A row's click
   * moves the board there and opens its Why. */
  const logRows: HTMLElement[] = [];
  const rowAt: { beat: number; wave: number }[] = [];
  function buildLog(): void {
    beats.forEach((pb, bi) => {
      pb.waves.forEach((st, wi) => {
        const row = h(
          "button",
          { class: `bv-log-row${wi === 0 ? " first" : ""}`, "data-testid": "log-row", "data-beat": String(bi) },
          h("span", { class: "bv-log-t dim mono" }, wi === 0 ? turnLabel(pb.turn) : ""),
          h("span", { class: "bv-log-c" }, ...(st.subjectSide ? [sideTag(st.subjectSide)] : []), ...richCaption(st.caption)),
        );
        row.addEventListener("click", () => {
          pause();
          finished = false;
          at = bi;
          wave = wi;
          landed = [];
          const c = st.changes[0];
          if (c) openTrace(c.eventId, st.changes.filter((x) => x.unit === c.unit));
          else render();
        });
        logRows.push(row);
        rowAt.push({ beat: bi, wave: wi });
      });
    });
    logBody.replaceChildren(...(logRows.length ? logRows : [h("div", { class: "dim" }, "Nothing happened.")]));
  }
  /** The log row of the wave on screen (-1 before the first beat). */
  function curRow(): number {
    if (at < 0) return -1;
    return rowAt.findIndex((r) => r.beat === at && r.wave === wave);
  }
  function drawLog(): void {
    const cur = finished ? logRows.length - 1 : curRow();
    logRows.forEach((r, i) => {
      r.hidden = i > cur;
      r.classList.toggle("on", i === cur && !finished);
    });
    if (tab === "log" && playing && cur >= 0) logRows[cur]?.scrollIntoView({ block: "nearest" });
  }

  /** The timeline: a block per turn with its marks; a playhead on the beat on
   * screen. Click or drag scrubs (pauses, the end card put away). */
  const turns = timelineOf(log, beats, sides);
  const turnEls: HTMLElement[] = [];
  const head = h("div", { class: "bv-tl-head", "aria-hidden": "true" });
  const track = h("div", { class: "bv-tl-track" });
  /** Each block's marks, laid out by layoutMarks() to what its box holds. */
  const markEls: HTMLElement[][] = [];
  const boxEls: HTMLElement[] = [];
  /** Each "+n" count and the marks it stands for. */
  const moreMarks = new WeakMap<HTMLElement, HTMLElement[]>();
  let marksObserver: ResizeObserver | null = null;
  function buildTimeline(): void {
    const every = turns.length > 30 ? 5 : turns.length > 18 ? 2 : 1;
    turns.forEach((t, i) => {
      const marks = t.marks.map((m) => {
        const tone = m.kind === "fatigue" ? "tone-gold" : m.side === you ? "tone-ally" : "tone-enemy";
        const id = m.kind === "death" ? "death-skull" : m.kind === "big" ? "spiky-explosion" : "hourglass";
        const what = m.kind === "death" ? `${name(m.unit ?? "")} falls` : m.kind === "big" ? `big hit on ${name(m.unit ?? "")}` : "fatigue sets in";
        const mk = h("span", { class: `bv-tl-mark ${m.kind}`, "data-testid": "timeline-mark", "data-kind": m.kind, "data-beat": String(m.beat), ...(m.side ? { "data-side": m.side === you ? "you" : "them" } : {}), title: what });
        mk.append(icon(id, 14, tone));
        return mk;
      });
      markEls.push(marks);
      // Battle start (turn 0) has a block of its own, "Start" (R2-17).
      const start = t.turn < 1;
      const box = h("div", { class: "bv-tl-box" }, ...(start && !marks.length ? [icon("flying-flag", 14, "dim")] : []), ...marks);
      boxEls.push(box);
      const label = start ? (turns.length <= 18 ? "Start" : "S") : i % every === 0 || i === turns.length - 1 ? String(t.turn) : "";
      const block = h("div", { class: `bv-tl-turn${start ? " start" : ""}`, "data-testid": "timeline-turn", "data-turn": String(t.turn), title: start ? "Battle start" : `Turn ${t.turn}` }, box, h("div", { class: "bv-tl-n mono dim" }, label));
      turnEls.push(block);
    });
    track.replaceChildren(...turnEls, head);
    timeline.replaceChildren(track);
    // A box shows the marks it has room for; with more, the last one it
    // shows is a count ("+2"), whose title lists the rest (R2-17).
    const layoutMarks = () => {
      boxEls.forEach((box, i) => {
        const marks = markEls[i]!;
        if (!marks.length) return;
        const room = Math.max(1, Math.floor((box.clientWidth - 4) / 15));
        const fit = marks.length <= room ? marks.length : Math.max(0, room - 1);
        const rest = marks.slice(fit);
        // drawTimeline() sets its number and title: only the marks reached so far (R2-17 batch F).
        const count = rest.length ? h("span", { class: "bv-tl-more mono", "data-testid": "timeline-more", "data-beat": rest[0]!.dataset.beat ?? "" }) : null;
        if (count) moreMarks.set(count, rest);
        box.replaceChildren(...marks.slice(0, fit), ...(count ? [count] : []));
      });
      drawTimeline();
    };
    if (typeof ResizeObserver === "function") (marksObserver = new ResizeObserver(layoutMarks)).observe(track);
    else layoutMarks();
    let down = false;
    const scrub = (x: number) => {
      const box = track.getBoundingClientRect();
      if (!box.width || !turns.length) return;
      const f = Math.max(0, Math.min(0.9999, (x - box.left) / box.width)) * turns.length;
      const t = turns[Math.floor(f)]!;
      const b = t.beats[Math.min(t.beats.length - 1, Math.floor((f % 1) * t.beats.length))]!;
      if (b === at && !finished) return;
      pause();
      finished = false;
      go(b);
    };
    track.addEventListener("pointerdown", (e) => {
      // A mark (or a count of marks) jumps to its own beat (R2-17); the
      // rest of the track scrubs by where the pointer is.
      const mark = (e.target as Element | null)?.closest?.<HTMLElement>(".bv-tl-mark, .bv-tl-more");
      if (mark?.dataset.beat) {
        pause();
        finished = false;
        go(Number(mark.dataset.beat));
        return;
      }
      down = true;
      track.setPointerCapture?.(e.pointerId);
      scrub(e.clientX);
    });
    track.addEventListener("pointermove", (e) => { if (down) scrub(e.clientX); });
    const up = () => { down = false; };
    track.addEventListener("pointerup", up);
    track.addEventListener("pointercancel", up);
  }
  function drawTimeline(): void {
    if (!turns.length) return;
    // Why's "Turn N" shows the board as turn N begins: the timeline lights
    // turn N, its playhead at the block's start, as the HUD reads it (R2-17).
    const starting = hudTurn === null ? -1 : turns.findIndex((t) => t.turn === hudTurn);
    const ti = starting >= 0 ? starting : at < 0 ? -1 : turns.findIndex((t) => t.beats.includes(at));
    turnEls.forEach((el, i) => {
      el.classList.toggle("on", i === ti);
      el.classList.toggle("past", i < ti);
    });
    // A mark ahead of the playhead waits until it is reached (or the end was seen).
    const reached = (mk: HTMLElement) => seenEnd || Number(mk.dataset.beat) <= at;
    for (const mk of track.querySelectorAll<HTMLElement>(".bv-tl-mark")) mk.classList.toggle("ahead", !reached(mk));
    // A count counts only the marks reached: a later death in its turn never shows early (R2-17 batch F).
    for (const more of track.querySelectorAll<HTMLElement>(".bv-tl-more")) {
      const rest = (moreMarks.get(more) ?? []).filter(reached);
      more.classList.toggle("ahead", !rest.length);
      more.textContent = `+${rest.length}`;
      more.title = rest.map((m) => m.title).join(", ");
      more.dataset.count = String(rest.length);
    }
    const t = turns[ti];
    const pos = !t ? 0 : starting >= 0 ? ti / turns.length : (ti + (t.beats.indexOf(at) + 1) / t.beats.length) / turns.length;
    head.style.left = `${(pos * 100).toFixed(2)}%`;
    timeline.setAttribute("aria-valuemin", "0");
    timeline.setAttribute("aria-valuemax", String(turns.at(-1)!.turn));
    timeline.setAttribute("aria-valuenow", String(t?.turn ?? 0));
    timeline.setAttribute("aria-valuetext", turnLabel(t?.turn ?? 0));
  }
  /** Frees what the battle holds outside its nodes: the playback timer, the
   * end card's resize listener and the timeline's observer. Continue runs
   * it, and so does any screen that replaces the battle (onGone). */
  const freed = new AbortController();
  function free(): void {
    if (timer) clearTimeout(timer);
    timer = null;
    freed.abort();
    marksObserver?.disconnect();
    marksObserver = null;
  }
  function leave(): void {
    free();
    a.onDone();
  }

  function unitCard(u: BoardUnit, side: Side, v: View, stsWidth: number): HTMLElement {
    const step = v.now;
    const changes = v.changes.filter((c) => c.unit === u.id);
    const el = card(units.get(u.id) ?? { emoji: emojiOf(u.id), name: u.name, stats: { pwr: u.pwr, hp: u.hp } }, {
      side: side === you ? "you" : "ghost",
      live: { stats: { pwr: u.pwr, hp: u.hp }, maxHp: u.maxHp, acting: step?.actor === u.id },
      extra: [statusChips(u, stsWidth), changes.length ? h("div", { class: "bv-changes" }, changeBadge(changes)) : null],
    });
    el.classList.add("bv-card");
    el.dataset.unit = u.id;
    if (u.silenced) el.classList.add("silenced");
    motion(el, u.id, side, v);
    triggerFlash(el, u.id, v);
    // The change's chip traces it; the rest of the card opens the unit.
    el.addEventListener("click", () => openUnit(u.id));
    return el;
  }
  /** One chip for a unit's changes this step. Two changes (a status and the
   * stat it moves: "Vitality ×2" and "+2 HP") show as two lines, each in its
   * own colour; a third and more add "+n" to the second line. The chip opens
   * the first change's trace, which lists them all, each tappable. */
  function changeBadge(changes: Change[]): HTMLElement {
    const c = changes[0]!;
    const more = changes.length - 2;
    const lines = changes.slice(0, 2).map((x, i) => h("span", { class: `bv-l ${x.kind}` }, ...changeLabel(x), i === 1 && more > 0 ? ` +${more}` : ""));
    const b = h(
      "button",
      { class: `bv-change ${c.kind}${changes.length > 1 ? " multi" : ""}`, "data-testid": "change", "data-event": String(c.eventId), "data-count": String(changes.length), "aria-label": changes.map((x) => x.label).join(", ") },
      h("span", { class: `bv-pill${changes.length > 1 ? " two" : ""}${blockedBy(c) ? " blocked" : ""}` }, ...(changes.length > 1 ? lines : changeLabel(c))),
    );
    b.addEventListener("click", (ev) => { ev.stopPropagation(); openTrace(c.eventId, changes); });
    return b;
  }
  /** How much Shield blocked of a hit that did no damage (0: not a blocked hit). */
  function blockedBy(c: Change): number {
    const e = log[c.eventId];
    return e?.type === "Hurt" && e.amount === 0 && e.absorbed ? e.absorbed : 0;
  }
  /** A hit that did nothing and that no Shield took (a 0-PWR striker). */
  function noDamage(c: Change): boolean {
    const e = log[c.eventId];
    return e?.type === "Hurt" && e.amount === 0 && !e.absorbed;
  }
  /** A change as the card shows it; a hit Shield fully blocked is the Shield
   * icon and what it blocked, never "−0". */
  function changeLabel(c: Change): (Node | string)[] {
    const blocked = blockedBy(c);
    if (blocked) return [icon("shield", 14, "tone-shield"), h("span", { class: "tone-shield" }, `${blocked}`)];
    if (noDamage(c)) return [h("span", { class: "bv-nodmg" }, "no dmg")];
    // A status landing reads as its icon and stacks ("🛡×4", not "Shield ×4"),
    // like the card's own status chips: the word was too wide for the chip
    // row and covered the trigger icon (R2-17). Its name is in the aria-label.
    const e = log[c.eventId];
    if (e?.type === "StatusApplied") {
      const def = STATUS_TERMS[e.status] ?? termDef(`status:${e.status}`);
      if (def?.icon) return [h("span", { class: `bv-pst tone-${def.tone}` }, icon(def.icon, 13), `×${e.stacks}`)];
    }
    return [c.label];
  }
  /** A card's statuses (R2-13): each one's icon and stacks, in its colour. The
   * row keeps its height when empty, so a status landing moves nothing. Two
   * rows show as many as fit the row's width (`width`, measured last render;
   * a two-digit stack is a wider chip); with more, the last chip is "+n",
   * which opens the unit's live statuses (R2-17). */
  function statusChips(u: BoardUnit, width: number): HTMLElement {
    const statuses = u.statuses;
    // The desktop's chips are drawn 1.2× larger (12 px, R2-17 batch E): the row holds fewer.
    // A short phone screen (on its side) has one status row (style.css).
    const shown = statusesShown(statuses.map((st) => st.stacks), (width > 0 ? width : STATUS_ROW_FALLBACK) / (isDesktop() ? 1.25 : 1), shortScreen.matches ? 1 : 2);
    const over = statuses.length - shown;
    let more: HTMLElement | null = null;
    if (over) {
      more = h("button", { class: "bv-st more", "data-testid": "card-status-more", "aria-label": `${over} more: all ${statuses.length} statuses` }, `+${over}`);
      more.addEventListener("click", (ev) => {
        ev.stopPropagation();
        openUnit(u.id);
      });
    }
    return h(
      "div",
      { class: "bv-sts", "data-testid": "card-statuses", "data-count": String(statuses.length) },
      ...statuses.slice(0, shown).map((st) => {
        const def = STATUS_TERMS[st.status] ?? termDef(`status:${st.status}`);
        const ic = def?.icon;
        return h(
          "span",
          { class: `bv-st tone-${def?.tone ?? "plain"}`, title: `${st.status} ${st.stacks}`, "aria-label": `${st.status} ${st.stacks}`, "data-testid": "card-status", "data-status": st.status },
          ...(ic ? [icon(ic, 12)] : [st.status.slice(0, 2)]),
          h("b", {}, `${st.stacks}`),
        );
      }),
      more,
    );
  }
  /** A unit as it is at the beat on screen (R3-18): live PWR and HP against
   * its base, every status with its icon, stacks and tip, Silenced, and its
   * ability text; a dim last line says how it entered. A fallen unit reads
   * as it fell. "Full card" opens its card sheet (both forms). */
  function nowSheet(u: BoardUnit, side: Side, fallen: boolean): HTMLElement {
    const entered = units.get(u.id);
    const summon = entered ? undefined : log.find((e) => e.type === "Summon" && e.unit === u.id);
    const base = entered ? entered.stats : summon?.type === "Summon" ? { pwr: summon.pwr, hp: summon.hp } : null;
    const body = entered ? null : summonBody(u.name);
    const ability: Node[] = entered
      ? formRich(entered.recipe, a.content)
      : body && !(body.abilities ?? []).every((x) => x === "Strike")
        ? formRich({ when: body.triggers ?? [], who: body.selectors ?? [], does: body.abilities ?? [], ...(body.condition ? { condition: body.condition } : {}) }, a.content)
        : [h("b", {}, "No ability: it fights with its PWR / HP.")];
    const pwr = h("span", { "data-testid": "now-pwr" }, `PWR ${u.pwr}`, base && base.pwr !== u.pwr ? h("span", { class: "dim" }, ` (base ${base.pwr})`) : "");
    // The max as the card's HP bar reads it: a heal past the max raises it.
    const max = Math.max(1, u.maxHp, u.hp);
    const hp = h("span", { "data-testid": "now-hp" }, fallen ? `HP 0 / ${max} · fallen` : `HP ${u.hp} / ${max}`);
    // It takes the Now sheet's place: one sheet at a time.
    const full = entered
      ? button("Full card ▸", () => {
          app.querySelector('[data-testid="now-sheet"]')?.closest(".overlay")?.remove();
          closable(unitSheet(entered, a.content));
        }, "small link", "now-full-card")
      : null;
    return h(
      "div",
      { class: "stack bv-now", "data-testid": "now-sheet", "data-unit": u.id },
      h("h2", {}, `${emojiOf(u.id)} `, unitName(u.id), " ", sideTag(side, "now-side")),
      h("div", { class: "label", "data-testid": "now-turn" }, `${turnLabel(shownTurn)} · this beat`),
      h("div", { class: "num", "data-testid": "now-stats" }, pwr, " · ", hp),
      h("div", { class: "label" }, "Statuses"),
      h(
        "div",
        { class: "stack", "data-testid": "live-statuses", "data-count": String(u.statuses.length) },
        u.silenced ? h("div", { class: "bv-live-st", "data-testid": "now-silenced" }, h("b", {}, "Silenced: "), h("span", { class: "dim" }, "its ability is off.")) : null,
        ...u.statuses.map((st) => {
          const def = STATUS_TERMS[st.status] ?? termDef(`status:${st.status}`);
          return h(
            "div",
            { class: "bv-live-st", "data-testid": "live-status", "data-status": st.status, "data-stacks": String(st.stacks) },
            h("span", { class: `bv-live-st-head tone-${def?.tone ?? "plain"}` }, ...(def?.icon ? [icon(def.icon, 18)] : []), h("b", {}, ` ${st.status} ×${st.stacks}`)),
            def?.tip ? h("span", { class: "dim" }, def.tip) : null,
          );
        }),
        !u.statuses.length && !u.silenced ? h("div", { class: "dim", "data-testid": "now-no-statuses" }, "No statuses.") : null,
      ),
      h("div", { class: "label" }, "Ability"),
      h("div", { class: "sheet-form", "data-testid": "now-ability" }, ...ability),
      base ? h("div", { class: "dim", "data-testid": "now-base" }, `Entered as ${base.pwr} PWR / ${base.hp} HP${summon ? " · summoned" : ""}`) : null,
      full,
    );
  }
  /** A summoned unit's body, by its name: the unit a summon effect in the content makes. */
  function summonBody(unitName: string): UnitDef | null {
    for (const ab of Object.values(a.content.abilities)) for (const e of ab.effects) if (e.kind === "summon" && e.unit.name === unitName) return e.unit;
    return null;
  }
  /** The cause badge over a unit that did something this beat (R3-19): [why]
   * → [what], for every wave, not only abilities: a strike shows crossed
   * swords on the striker, a Poison tick the Poison drop on its holder. It
   * pops when its wave lands and stays for the beat; the newest wave's badge
   * is bright, earlier ones dim. A tap opens that step's Why. `at` is a unit,
   * or "clash" for fatigue (the clash mark on desktop; the phone has it in
   * the caption). */
  function triggerBadge(at: string, v: View): HTMLElement | null {
    let hit: { c: Cause; step: Step; age: number | null; newest: boolean } | null = null;
    v.waves.forEach((w, i) => {
      const c = causeOf(log, w.step, whenOf);
      if (c && c.at === at) hit = { c, step: w.step, age: w.age, newest: i === v.waves.length - 1 };
    });
    if (!hit) return null;
    const { c, step, age, newest } = hit as { c: Cause; step: Step; age: number | null; newest: boolean };
    const trig = causeIcon(c);
    const eff = termIcon(c.effect as TermId, c.effectStatus);
    const effTone = (c.effectStatus ? termDef(`status:${c.effectStatus}`, a.content.statuses)?.tone : undefined) ?? termDef(c.effect as TermId, a.content.statuses)?.tone ?? "plain";
    const does = c.effectStatus ?? termDef(c.effect as TermId)?.label ?? "";
    const b = h(
      "button",
      { class: `bv-badge${newest ? "" : " past"}`, "data-testid": "trigger-badge", "data-cause": c.cause, "data-kind": c.kind, "data-trigger": c.cause.startsWith("trigger:") ? c.cause : "", "data-effect": c.effect, "aria-label": `${causeLabel(c)} → ${does}` },
      // The button is 44 px tall to tap; the pill it draws hugs the card.
      h(
        "span",
        { class: "bv-badge-pill" },
        trig ? icon(trig, 14, causeTone(c)) : h("span", { class: "tone-when" }, "⚡"),
        h("span", { class: "bv-badge-arrow" }, "→"),
        eff ? icon(eff, 14, `tone-${effTone}`) : h("span", { class: `tone-${effTone}` }, c.effect === "effect:cancel" ? "⊘" : does.slice(0, 3)),
      ),
    );
    if (age !== null && age <= MOTION_MS) b.style.setProperty("--bv-bt", `${-Math.round(age * speed)}ms`);
    else b.classList.add("still");
    b.addEventListener("click", (ev) => {
      ev.stopPropagation();
      const ch = step.changes[0];
      if (ch) openTrace(ch.eventId, step.changes.filter((x) => x.unit === ch.unit));
      else openTrace(step.eventIds[0]!, []);
    });
    return b;
  }
  /** The card's When icon (its icon line's first) flashes gold for 0.4 s
   * when that When fires, tying the card's lasting icon to the moment
   * (R3-19); after a manual step it stays lit for the wave. */
  function triggerFlash(el: HTMLElement, id: string, v: View): void {
    const w = v.waves.at(-1);
    const c = w ? causeOf(log, w.step, whenOf) : null;
    if (!w || c?.kind !== "ability" || c.at !== id) return;
    const ic = el.querySelector<HTMLElement>(".icons .ci:first-of-type");
    if (!ic) return;
    if (w.age === null || reduced()) ic.classList.add("bv-fired", "still");
    else if (w.age * speed <= TRIG_FLASH_MS) {
      ic.classList.add("bv-fired");
      ic.style.setProperty("--bv-ftt", `${-Math.round(w.age * speed)}ms`);
    }
  }
  /** A cause's icon: the When's (a status trigger shows its status), the
   * status's own, fatigue's hourglass. */
  function causeIcon(c: Cause): IconId | undefined {
    if (c.cause.startsWith("status:")) return termDef(c.cause as TermId, a.content.statuses)?.icon;
    return termIcon(c.cause as TermId, c.causeStatus);
  }
  function causeTone(c: Cause): string {
    if (c.kind === "status") return `tone-${termDef(c.cause as TermId, a.content.statuses)?.tone ?? "plain"}`;
    if (c.kind === "battle") return `tone-${termDef(c.cause as TermId)?.tone ?? "plain"}`;
    return "tone-when";
  }
  /** A cause in words, scoped like the card says it: "Ally gets Shield", "Strikes", "Poison". */
  function causeLabel(c: Cause): string {
    if (c.kind === "status") return c.cause.slice("status:".length);
    return triggerLabel(c.cause as TermId, c.causeStatus, c.causeScope);
  }
  /** A card in its slot, with the trigger badge on its outer edge when one
   * fired, and its floating numbers (outside the card: its clip-path would cut them). */
  function slot(el: HTMLElement, id: string, v: View): HTMLElement {
    return h("div", { class: "bv-slot" }, el, triggerBadge(id, v), ...(floatsOf.get(el) ?? []));
  }
  /** A card tap (or one of its status chips): the unit's Now sheet, paused on this beat (R3-18). */
  function openUnit(id: string): void {
    pause();
    render();
    if (!shownBoard) return;
    for (const side of ["A", "B"] as const) {
      const live = shownBoard.lines[side].find((x) => x.id === id);
      const fell = live ? undefined : shownBoard.graves[side].find((x) => x.id === id);
      const u = live ?? fell;
      if (u) return void closable(nowSheet(u, side, !live));
    }
  }
  function deadCard(id: string, v: View): HTMLElement | null {
    // A unit that falls this beat stays in its slot, greyed, until the beat
    // ends, so its ✝ can be tapped; so does a fallen unit that acts (a
    // death-triggered ability), lit.
    const step = v.now;
    const death = v.changes.find((c) => c.unit === id && c.kind === "death");
    if (!death && step?.actor !== id) return null;
    const el = h(
      "div",
      { class: `card bv-card dead ${sides.get(id) === you ? "you" : "ghost"}` },
      h("div", { class: "emoji" }, emojiOf(id)),
      h("div", { class: "name" }, name(id)),
      h("div", { class: "hpbar" }, h("i", { style: "width:0" })),
      h("div", { class: "stats big" }, h("span", { class: "p" }, "✝")),
      h("div", { class: "foot" }, h("div", { class: "bv-sts" })),
      death ? h("div", { class: "bv-changes" }, changeBadge(v.changes.filter((c) => c.unit === id))) : null,
    );
    el.dataset.unit = id;
    if (step?.actor === id) el.classList.add("acting");
    motion(el, id, sides.get(id) ?? you, v);
    el.addEventListener("click", () => openUnit(id));
    return el;
  }

  /** What the board shows: the open beat's changes so far (the chips), the
   * wave that just landed (the caption and the motion), and its board. */
  interface View {
    now: Step | undefined;
    changes: Change[];
    /** The beat's waves landed so far, each with its age in ms when it landed
     * by playback (null after a manual step: no motion). */
    waves: { step: Step; age: number | null }[];
  }

  /** Motion for each wave landed so far: the striker lunges, a hit target
   * flashes and shakes, a caster pulses, a dying card pops, and floating
   * numbers rise. A wave that landed `age` ms ago starts its animations
   * `age` ms in (a negative delay), so re-rendering the board for the next
   * wave keeps them going. Under prefers-reduced-motion the CSS keeps it
   * still: the target gets a ring, the numbers sit in the chips and the
   * caption lists them. */
  function motion(el: HTMLElement, id: string, side: Side, v: View): void {
    // Ages are real ms; animations run `speed` times faster, so their offset is age × speed.
    let hitAge: number | null = null;
    for (const { step, age } of v.waves) {
      if (age === null || age * speed > MOTION_MS) continue;
      const t = `${-Math.round(age * speed)}ms`;
      const mine = step.changes.filter((c) => c.unit === id);
      let move: string | null = null;
      if (step.actor === id) {
        const first = log[step.eventIds[0]!];
        const struck = first?.type === "Hurt" && first.source === "kernel" && first.causedBy !== null && log[first.causedBy]?.type === "Strike";
        move = struck ? (side === you ? "bv-lunge-up" : "bv-lunge-down") : "bv-pulse";
      }
      if (mine.some((c) => c.kind === "damage")) { move = "bv-hit"; hitAge = age * speed; }
      // A dying card (its death trigger firing) or one still shaking from a
      // hit keeps that motion: a pulse or lunge would cut the shake short.
      if (move !== "bv-hit" && (el.classList.contains("bv-dying") || (hitAge !== null && hitAge - age * speed < SHAKE_END_MS))) move = null;
      if (mine.some((c) => c.kind === "death")) {
        // The killing blow's shake plays out, then the card pops (R2-13):
        // the die starts once the shake (80 ms in, 340 ms long) is over.
        const own = age * speed;
        const start = hitAge !== null ? Math.max(-own, SHAKE_END_MS - hitAge) : -own;
        el.classList.add("bv-dying");
        el.style.setProperty("--bv-dt", `${Math.round(start)}ms`);
        if (hitAge === null) el.classList.remove("bv-lunge-up", "bv-lunge-down", "bv-pulse");
        move = null;
      }
      // The latest wave's movement wins; its delay drives the card's animation.
      if (move) {
        el.classList.remove("bv-lunge-up", "bv-lunge-down", "bv-pulse", "bv-hit");
        el.classList.add(move);
        el.style.setProperty("--bv-t", t);
      }
      const flash = mine.some((c) => c.kind === "damage" || c.kind === "death") ? "bv-flash-hit" : mine.some((c) => c.kind === "heal" || c.kind === "buff") ? "bv-glow" : null;
      if (flash) {
        el.classList.remove("bv-flash-hit", "bv-glow");
        el.classList.add(flash);
        el.style.setProperty("--bv-ft", t);
      }
    }
    // Numbers float (−n, +n, ±PWR, a summon); a status or a death already
    // sits in the card's chip, so floating it too only covers the card. One
    // float a unit at a time (R2-17 batch E: a combo's floats piled up on the
    // phone): what the beat has done to it so far, merged ("−5", "−3 PWR",
    // "+1/+3"), rising again with each wave that adds to it. The chip and Why
    // hold the details.
    let latest = -1;
    v.waves.forEach(({ step, age }, i) => {
      if (age !== null && age * speed <= MOTION_MS && step.changes.some((c) => c.unit === id && floats(c))) latest = i;
    });
    const age = latest >= 0 ? v.waves[latest]!.age : null;
    if (age === null) return;
    const f = mergedFloat(v.waves.slice(0, latest + 1).flatMap((w) => w.step.changes.filter((c) => c.unit === id && floats(c))));
    if (!f) return;
    f.style.animationDelay = `${-Math.round(age)}ms`;
    floatsOf.set(el, [f]);
  }
  /** A change that floats up from its card. */
  function floats(c: Change): boolean {
    return FLOATS.has(c.kind) && !noDamage(c);
  }
  /** One float for a unit's changes: the damage it took, else the healing,
   * else its PWR and HP changes ("+1 PWR", "+1/+3" for both, PWR / HP as a
   * card reads), else what Shield blocked, else a summon's label. */
  function mergedFloat(cs: Change[]): HTMLElement | null {
    let dmg = 0, heal = 0, pwr = 0, hp = 0, blocked = 0;
    let summon: Change | null = null;
    for (const c of cs) {
      const e = log[c.eventId];
      if (e?.type === "Hurt") { dmg += e.amount; blocked += e.amount === 0 ? (e.absorbed ?? 0) : 0; }
      else if (e?.type === "Heal") heal += e.amount;
      else if (e?.type === "StatChanged") { if (e.stat === "pwr") pwr += e.delta; else hp += e.delta; }
      else if (c.kind === "summon") summon = c;
    }
    const sign = (n: number) => (n > 0 ? `+${n}` : `−${-n}`);
    const float = (kind: string, ...kids: (Node | string)[]) => h("span", { class: `bv-float ${kind}`, "data-testid": "float", "aria-hidden": "true" }, ...kids);
    if (dmg > 0) return float("damage", `−${dmg}`);
    if (heal > 0) return float("heal", `+${heal}`);
    if (pwr || hp) {
      const kind = pwr + hp >= 0 ? "buff" : "debuff";
      if (pwr && hp) return float(kind, `${sign(pwr)}/${sign(hp)}`);
      const f = float(kind, pwr ? `${sign(pwr)} PWR` : `${sign(hp)} HP`);
      f.classList.add(pwr ? "on-pwr" : "on-hp");
      return f;
    }
    if (blocked) return float("damage", icon("shield", 14, "tone-shield"), h("span", { class: "tone-shield" }, `${blocked}`));
    return summon ? float("summon", summon.label) : null;
  }

  /** A line in the beat: the living units, and each unit that fell in this
   * beat back in the slot it held when the beat began. */
  function lineOf(side: Side, board: ReturnType<typeof boardAt>, before: ReturnType<typeof boardAt>, v: View, stsWidth: number): HTMLElement[] {
    const living: HTMLElement[] = board.lines[side].map((u) => slot(unitCard(u, side, v, stsWidth), u.id, v));
    const fallen = board.graves[side]
      .map((u) => {
        const dead = deadCard(u.id, v);
        return { card: dead && slot(dead, u.id, v), slot: before.lines[side].findIndex((b) => b.id === u.id) };
      })
      .filter((x): x is { card: HTMLElement; slot: number } => x.card !== null)
      .sort((p, q) => (p.slot < 0 ? 99 : p.slot) - (q.slot < 0 ? 99 : q.slot));
    for (const f of fallen) living.splice(f.slot < 0 ? living.length : Math.min(f.slot, living.length), 0, f.card);
    return living;
  }

  function render(): void {
    const beat = beats[at];
    const step = beat?.waves[wave];
    // The board shows every event of the waves landed so far; a later wave can
    // hold an earlier id (a status leaving folds into the hit that caused it).
    const shown = beat ? beat.waves.slice(0, wave + 1) : [];
    const upto = shown.reduce((m, w) => w.eventIds.reduce((n, id) => Math.max(n, id), m), 0);
    // Draw by the set of landed ids, not the max: a wave still to come can
    // hold a lower id than one already landed.
    const pending = new Set(beat ? beat.waves.slice(wave + 1).flatMap((w) => w.eventIds) : []);
    const now = performance.now();
    const v: View = {
      now: step,
      changes: shown.flatMap((w) => w.changes),
      waves: shown.map((w, i) => ({ step: w, age: landed[i] !== undefined ? now - landed[i]! : null })),
    };
    const board = boardAt(log, upto, pending);
    shownBoard = board;
    const before = beat ? boardAt(log, Math.max(0, beat.start - 1)) : board;
    const turn = step?.turn ?? 0;
    shownTurn = hudTurn ?? turn;
    hud.replaceChildren(
      ...(menuBtn ? [menuBtn] : []),
      h("span", {}, battle.kind === "crown" ? "Crown fight" : battle.kind === "playoff" ? "Playoff" : `Round ${battle.round}`),
      h("span", { class: "dim who", title: battle.opponent.name }, `vs @${battle.opponent.name}`),
      h("span", { "data-testid": "battle-turn" }, turnLabel(hudTurn ?? turn)),
    );
    clash.replaceChildren(clashMark, ...[triggerBadge("clash", v)].filter((x): x is HTMLElement => x !== null));
    for (const [side, row] of [[them, enemy], [you, mine]] as const) {
      // The status row's width, from the cards on screen (read before they are replaced).
      const stsWidth = row.querySelector<HTMLElement>(".bv-card:not(.dead) .bv-sts")?.getBoundingClientRect().width ?? 0;
      const cards = lineOf(side, board, before, v, stsWidth);
      row.replaceChildren(...cards);
      // A falling card beside a full line widens the row instead of wrapping it.
      row.style.gridTemplateColumns = `repeat(${Math.max(5, cards.length)}, minmax(0, 1fr))`;
      row.classList.toggle("empty", !cards.length);
      if (!cards.length) row.append(h("div", { class: "dim" }, "No one standing."));
      // Fit names and chips now, not a frame later: a chip drawn at full
      // size showed "−…" for a frame on a narrow card (R2-17 batch F).
      if (row.isConnected) fitText(row);
    }
    caption.replaceChildren(h("span", { class: "bv-cap" }, ...captionKids(step)));
    still.replaceChildren(...(reduced() ? stillList(v.changes) : []));
    still.style.display = reduced() && !finished ? "" : "none";
    fitStill();
    caption.classList.toggle("tappable", !!step?.changes.length);
    // Under the end card the phone's caption keeps only its line (style.css).
    caption.classList.toggle("ended", finished && !trace);
    recent.replaceChildren(
      ...beats.slice(Math.max(0, at - 3), Math.max(0, at)).reverse().map((pb) => {
        // A past beat reads as its first wave, the strike or tick that opened it.
        const s = pb.waves[0]!;
        const b = h("button", { class: "bv-past" }, ...(s.subjectSide ? [sideTag(s.subjectSide)] : []), ...richCaption(s.caption), pb.waves.length > 1 ? h("span", { class: "dim" }, ` +${pb.waves.length - 1}`) : null);
        b.addEventListener("click", () => {
          const c = s.changes[0];
          if (c) openTrace(c.eventId, s.changes.filter((x) => x.unit === c.unit));
        });
        return b;
      }),
    );
    recent.style.display = finished ? "none" : "";
    fitRecent();
    // The phone shows the sheet only with a trace; the desktop panel is always there (style.css).
    sheet.classList.toggle("open", !!trace);
    // Nothing traced (✕, Esc, ▶, the end card): Why is off and the panel is back on Log (R3-17).
    const backToLog = !trace && tab === "why";
    if (!trace) tab = "log";
    sheet.dataset.tab = tab;
    whyTab.disabled = !trace;
    whyTab.title = trace ? "" : "Click a number, badge or log row to see why";
    whyTab.classList.toggle("on", tab === "why");
    logTab.classList.toggle("on", tab === "log");
    whyBody.replaceChildren(...(trace ? traceView(trace) : []));
    drawLog();
    if (backToLog) logRows[curRow()]?.scrollIntoView({ block: "nearest" });
    drawTimeline();
    // A trace opened from the end card (Why I lost) sits in its place until closed.
    end.style.display = finished && !trace ? "" : "none";
    if (finished) {
      // A control in the card that redraws it (All n, Top 3) keeps the focus,
      // so the next Enter acts on it again instead of leaving (R2-17 batch F).
      const f = document.activeElement;
      const kept = f instanceof HTMLElement && end.contains(f) && f.dataset.testid ? { id: f.dataset.testid, i: [...end.querySelectorAll(`[data-testid="${f.dataset.testid}"]`)].indexOf(f) } : null;
      end.replaceChildren(...endView());
      if (kept) end.querySelectorAll<HTMLElement>(`[data-testid="${kept.id}"]`)[kept.i]?.focus();
      placeEnd();
    }
    if (errBox) errBox.style.top = `${Math.round(hud.getBoundingClientRect().bottom + 4)}px`;
    backBtn.disabled = at < 0;
    fwdBtn.disabled = finished;
    endBtn.disabled = finished;
  }

  /** Whose a unit is, as a tag: YOU / THEM, or its owner without a side. */
  function sideTag(side: Side, testid = ""): HTMLElement {
    const tag = a.you ? (side === you ? "You" : "Them") : owner(side);
    return h("span", { class: `bv-who ${side === you ? "you" : "ghost"}`, ...(testid ? { "data-testid": testid } : {}) }, tag);
  }
  /** The caption, led by whose unit it is about (the first one it names). */
  function captionKids(step: Step | undefined): Node[] {
    if (!step) return [document.createTextNode("The lines face off.")];
    const side = step.subjectSide;
    // It starts with its cause's icon, the one on the badge (R3-19).
    const c = causeOf(log, step, whenOf);
    const ic = c ? causeIcon(c) : undefined;
    const lead = c && ic ? [h("span", { class: "bv-cap-cause", "data-testid": "caption-cause", "data-cause": c.cause, title: causeLabel(c) }, icon(ic, 14, causeTone(c)))] : [];
    if (!side) return [...lead, ...richCaption(step.caption)];
    return [...lead, sideTag(side, "caption-side"), ...richCaption(step.caption)];
  }

  /** A caption with its terms highlighted (R2-13, like R2-8's unit text):
   * unit names in their side's colour, statuses with their icon and colour,
   * PWR / HP, damage and healing numbers, and a Shield block as the Shield
   * icon. Plain spans: the caption itself is the button that opens Why. */
  function richCaption(text: string): Node[] {
    const out: Node[] = [];
    let i = 0;
    for (const m of text.matchAll(TAG)) {
      if (m.index! > i) out.push(...termsIn(text.slice(i, m.index)));
      out.push(unitName(m[1]!));
      i = m.index! + m[0].length;
    }
    if (i < text.length) out.push(...termsIn(text.slice(i)));
    return out;
  }
  /** A unit's name in its own side's colour. */
  function unitName(id: string): Node {
    return h("span", { class: `bv-cn ${sides.get(id) === you ? "tone-ally" : "tone-enemy"}`, "data-unit": id }, name(id));
  }
  function termsIn(text: string): Node[] {
    const out: Node[] = [];
    let i = 0;
    for (const m of text.matchAll(captionTerms)) {
      if (m.index! > i) out.push(document.createTextNode(text.slice(i, m.index)));
      out.push(captionTerm(m[0]));
      i = m.index! + m[0].length;
    }
    if (i < text.length) out.push(document.createTextNode(text.slice(i)));
    return out;
  }
  function captionTerm(t: string): Node {
    const absorbed = /^\((\d+) absorbed\)$/.exec(t);
    if (absorbed) return h("span", { class: "bv-ct tone-shield", "data-testid": "caption-term" }, icon("shield", 14), ` ${absorbed[1]} blocked`);
    const status = STATUS_TERMS[t] ?? (logStatuses.has(t) ? termDef(`status:${t}`) : undefined);
    if (status) return h("span", { class: `bv-ct tone-${status.tone}`, "data-testid": "caption-term" }, ...(status.icon ? [icon(status.icon, 14), " "] : []), t);
    if (t === "Fatigue") return h("span", { class: "bv-ct tone-dmg", "data-testid": "caption-term" }, icon("hourglass", 14), " ", t);
    if (t === "PWR") return h("span", { class: "tone-pwr" }, t);
    if (t === "HP") return h("span", { class: "tone-hp" }, t);
    if (t.startsWith("−")) return h("b", { class: "tone-dmg" }, t);
    if (t.startsWith("+")) return h("b", { class: "tone-heal" }, t);
    return document.createTextNode(t);
  }

  /** Reduced motion: no floats, so the caption lists every change of the
   * beat so far, as the floats would have shown them. */
  function stillList(changes: Change[]): Node[] {
    if (!changes.length) return [];
    // One entry a change label, its units listed ("+1 PWR: Rose, Ace"), so a
    // buff on the whole line takes one entry, not five (R2-17).
    const groups: { c: Change; units: string[] }[] = [];
    for (const c of changes) {
      const g = groups.find((x) => x.c.kind === c.kind && x.c.label === c.label);
      if (g) { if (!g.units.includes(c.unit)) g.units.push(c.unit); }
      else groups.push({ c, units: [c.unit] });
    }
    return [
      h(
        "span",
        { class: "bv-still", "data-testid": "caption-changes" },
        ...groups.map((g) => h("span", { class: `bv-l ${g.c.kind}` }, ...g.units.flatMap((u, i) => (i ? [", ", unitName(u)] : [unitName(u)])), ` ${g.c.label}`)),
      ),
    ];
  }
  /** The reduced-motion list never runs past its box: the oldest entries
   * give way to a count ("+3 earlier") until the rest fits (R2-17). */
  function fitStill(): void {
    const list = still.querySelector(".bv-still");
    if (!list || still.style.display === "none") return;
    let dropped = 0;
    let more: HTMLElement | null = null;
    while (still.scrollHeight > still.clientHeight + 1) {
      const entries = [...list.children].filter((c) => c !== more);
      if (entries.length <= 1) break;
      entries[0]!.remove();
      dropped++;
      if (!more) list.prepend((more = h("span", { class: "bv-l dim bv-still-more" })));
      more.textContent = `+${dropped} earlier`;
    }
  }
  /** The recent beats show only rows that fit whole: when the reduced-motion
   * list above them grows a line, the oldest row goes instead of being cut
   * by the control bar (R2-17). */
  function fitRecent(): void {
    if (recent.style.display === "none") return;
    const rows = [...recent.children] as HTMLElement[];
    for (const r of rows) r.hidden = false;
    const bottom = recent.getBoundingClientRect().bottom;
    for (const r of rows) if (r.getBoundingClientRect().bottom > bottom + 0.5) r.hidden = true;
  }
  /** The end card sits under the caption on the phone (its last line, "They
   * win", stays readable) and under the HUD on desktop, as tall as what it
   * holds and clear of the timeline; it scrolls inside itself when it runs
   * long (R2-17). On a short phone screen (a phone on its side) the room
   * under the caption is too small for the card, so it covers the caption
   * (its own word says who won), else the board too. */
  function placeEnd(): void {
    if (end.style.display === "none" || !end.isConnected) return;
    const gap = 8;
    if (isDesktop()) {
      const top = hud.getBoundingClientRect().bottom + 2 * gap;
      const bottom = timeline.getBoundingClientRect().top - 2 * gap;
      end.style.top = `${Math.round(top)}px`;
      end.style.bottom = "auto";
      end.style.maxHeight = `${Math.max(0, Math.round(bottom - top))}px`;
    } else {
      // A narrow gap on the phone: every pixel goes to the three key moments.
      // Never below the screen's foot, whatever the controls do (R2-17 batch E).
      const bottom = Math.min(controls.getBoundingClientRect().top, innerHeight) - gap / 2;
      const cap = caption.getBoundingClientRect();
      // Its text, when a short screen squeezes the caption's box.
      const text = caption.firstElementChild?.getBoundingClientRect().bottom ?? cap.bottom;
      // The first that leaves the card room, each cutting nothing in half:
      // under the caption, over it, under the HUD, the screen's top. At
      // 360×640 the caption shrinks to its line once the card is up, and the
      // card's spacing is tight, so three whole moments fit under it (R2-17 batch F).
      const tops = [Math.max(cap.bottom, text) + gap / 2, cap.top, hud.getBoundingClientRect().bottom + gap / 2].filter((t) => t >= gap);
      const top = tops.find((t) => bottom - t >= END_MIN_PX) ?? gap;
      end.style.top = "";
      end.style.bottom = `${Math.round(innerHeight - bottom)}px`;
      end.style.maxHeight = `${Math.max(0, Math.round(bottom - top))}px`;
    }
  }

  function traceView(t: Trace): Node[] {
    const target = t.change ? name(t.change.unit) : "";
    return [
      h("div", { class: "row spread" }, h("div", { class: "label" }, `Why: ${t.change?.label ?? ""} ${target}`), button("✕", () => { trace = null; render(); }, "bv-close", "trace-close")),
      // Over a finished battle (a Why I lost row), the way on stays in sight:
      // back to the result, or straight on (R2-17 batch E).
      finished
        ? h(
            "div",
            { class: "row bv-trace-end", "data-testid": "trace-end" },
            button("‹ Result", () => { trace = null; render(); }, "grow", "trace-result"),
            button(a.outro?.doneLabel ?? "Continue", leave, "primary grow", "trace-done"),
          )
        : null,
      // The chip held more than one change: all of them, the traced one lit; tap another to trace it.
      traceGroup.length > 1
        ? h(
            "div",
            { class: "row bv-also", "data-testid": "trace-group" },
            h("span", { class: "dim small" }, "This step:"),
            ...traceGroup.map((x) => {
              const b = button(x.label, () => openTrace(x.eventId, traceGroup), `bv-also-btn ${x.kind}${x.eventId === t.eventId ? " on" : ""}`, "trace-group-change");
              return b;
            }),
          )
        : null,
      h("div", { class: "bv-trace-text mono", "data-testid": "trace-text" }, t.text),
      // R2-15: the whole chain, change first, back to the turn; a step's click shows its moment.
      chain ? chainView(chain, { units, content: a.content, you, name, active: chainAt, onStep: seekEvent, rich: richCaption, tag: (sd) => sideTag(sd) }) : null,
      chain ? h("div", { class: "dim bv-why-keys" }, isDesktop() ? "Click a step to see its moment; hover a highlighted word for its meaning." : "Tap a step to see its moment; hold a highlighted word for its meaning.") : null,
    ].filter((n): n is HTMLDivElement => n !== null);
  }

  /** The end card's second line (R2-17, the mockup's): how long it took, who
   * still stands and your best unit; without a side, both sides' standing. */
  function endSubtitle(): Node[] {
    const final = boardAt(log, log.length - 1);
    const team = (sd: Side) => (sd === "A" ? battle.teamA : battle.teamB);
    const standing = (sd: Side) => {
      const ids = new Set(team(sd).map((u) => u.id));
      return final.lines[sd].filter((u) => ids.has(u.id)).length;
    };
    const turnsText = `${final.turn} ${final.turn === 1 ? "turn" : "turns"}`;
    if (!a.you) return [document.createTextNode(`${turnsText} · ${owner("A")}: ${standing("A")} of ${team("A").length} standing · ${owner("B")}: ${standing("B")} of ${team("B").length}`)];
    const best = damage.find((d) => d.side === you && d.damage > 0);
    // The phone's card says it in one line: "9 turns · 1 of 3 standing · best: Bat".
    return [
      document.createTextNode(`${turnsText} · ${standing(you)} of `),
      h("span", { class: "bv-dk" }, "your "),
      document.createTextNode(`${team(you).length} `),
      h("span", { class: "bv-dk" }, "still "),
      document.createTextNode("standing"),
      ...(best ? [document.createTextNode(" · "), h("span", { class: "bv-dk" }, "your "), document.createTextNode("best: "), unitName(best.unit)] : []),
    ];
  }

  function endView(): Node[] {
    // Only a viewer with a side wins or loses; otherwise the winner is named.
    const word = !a.you ? (battle.winner === "draw" ? "DRAW" : `${owner(battle.winner)} wins`) : outcome === "win" ? "VICTORY" : outcome === "loss" ? "DEFEAT" : "DRAW";
    const cls = !a.you && battle.winner !== "draw" ? "neutral" : outcome;
    // Why I lost, or Why I won (R2-17): the chains that did the most, theirs or yours.
    const whyBtn = a.you && outcome !== "draw"
      ? button(outcome === "win" ? "Why I won" : "Why I lost", () => {
          const close = closable(whyPanel(battle, you, (id) => { close(); openTrace(id); }, outcome === "win"));
        }, "", "end-why")
      : null;
    const hide = button("✕", () => { finished = false; render(); }, "bv-close", "end-close");
    hide.setAttribute("aria-label", "See the board");
    const replayEnd = button("", replay, "bv-end-replay", "end-replay");
    replayEnd.append("↻ Replay", h("span", { class: "bv-dk" }, " from start"));
    return [
      h("div", { class: "row spread bv-end-top" }, h("div", { class: "bv-end-head" }, h("div", { class: `bv-word ${cls}`, "data-testid": "battle-word" }, word), h("div", { class: "bv-end-sub dim", "data-testid": "end-sub" }, ...endSubtitle())), hide),
      // The run line spans the card, not the head beside ✕: on a 360px phone the head is too narrow and the line wrapped (R2-17).
      ...(a.outro ? [h("div", { class: "bv-end-run", "data-testid": "end-run" }, ...a.outro.status())] : []),
      h(
        "div",
        { class: "bv-end-body" },
        h(
          "div",
          { class: "bv-end-moments stack" },
          moments.length ? h("div", { class: "label row spread" }, h("span", {}, "Key moments · ", h("span", { class: "bv-dk" }, "click"), h("span", { class: "bv-ph" }, "tap"), " to watch"), h("span", { class: "bv-end-more bv-ph" }, "Damage ↓")) : null,
          ...moments.map((m) => {
            const b = button("", () => playFrom(m.beat), `bv-moment ${m.kind}`, "key-moment");
            // Fatigue's label brings its own hourglass, and its turn is in the T column.
            b.append(h("span", { class: "bv-moment-t dim mono" }, turnLabel(beats[m.beat]?.turn ?? 0)), momentIcon(m.kind), h("span", { class: "bv-moment-l" }, ...richCaption(m.kind === "fatigue" ? m.label.replace(/\s*\(T\d+\)$/, "") : m.label)), h("span", { class: "dim bv-moment-go" }, "▶"));
            b.dataset.beat = String(m.beat);
            return b;
          }),
        ),
        damageView(),
      ),
      h("div", { class: "row bv-end-actions" }, replayEnd, whyBtn, button(a.outro?.doneLabel ?? "Continue", leave, "primary", "battle-done")),
    ];
  }
  /** A key moment's icon: a skull for a kill, a portal for a unit joining, linked rings for a combo, a burst for a big hit (fatigue's is in its label). */
  function momentIcon(kind: KeyMoment["kind"]): HTMLElement {
    if (kind === "fatigue") return h("span", { class: "bv-moment-ic" });
    const [id, tone]: [IconId, string] = kind === "kill" ? ["death-skull", "tone-enemy"] : kind === "summon" ? ["magic-portal", "tone-summon"] : kind === "combo" ? ["linked-rings", "tone-when"] : ["spiky-explosion", "tone-enemy"];
    return h("span", { class: "bv-moment-ic" }, icon(id, 16, tone));
  }

  /** Damage dealt, both sides in one list (yours first): each unit's emoji and
   * name in its side's colour, a bar scaled to the battle's biggest, the
   * number. The top three a side show; "All n" lists every unit (R2-17 batch
   * E: a long list covered the caption). A row opens the unit's sheet: the
   * lines as they entered the fight (the result screen's two lines). */
  function damageView(): HTMLElement {
    const top = Math.max(1, ...damage.map((d) => d.damage));
    const bySide = (side: Side) => damage.filter((d) => d.side === side);
    const all = bySide(you).length + bySide(them).length;
    const cap = dmgAll ? 99 : DMG_ROWS;
    const rows = (side: Side) =>
      bySide(side)
        .slice(0, cap)
        .map((d) => {
          const u = units.get(d.unit);
          const row = h(
            "div",
            { class: `bv-dmg ${side === you ? "you" : "ghost"}${u ? " open" : ""}`, "data-testid": "damage-row", title: `${d.name}: ${d.damage} damage`, ...(u ? { role: "button", tabindex: "0" } : {}) },
            // A fused unit's two emojis, smaller, in a column that holds both (R2-17 batch F: "🥁⚡" counts as two code points, so the old test missed it).
            h("span", { class: `emoji${u?.kind === "fused" ? " two" : ""}` }, emojiOf(d.unit)),
            unitName(d.unit),
            h("span", { class: "bv-dmg-bar" }, h("i", { style: `width:${Math.round((d.damage / top) * 100)}%` })),
            h("span", { class: "mono" }, String(d.damage)),
          );
          if (u) {
            row.addEventListener("click", () => closable(unitSheet(u, a.content)));
            // A button's keys: Enter or Space opens the sheet (R2-17 batch F).
            row.addEventListener("keydown", (e) => {
              if ((e.key !== "Enter" && e.key !== " ") || e.repeat) return;
              e.preventDefault();
              e.stopPropagation();
              row.click();
            });
          }
          return row;
        });
    const shown = Math.min(cap, bySide(you).length) + Math.min(cap, bySide(them).length);
    const more = all > shown || dmgAll ? button(dmgAll ? "Top 3" : `All ${all}`, () => { dmgAll = !dmgAll; render(); }, "small bv-dmg-more", "damage-more") : null;
    return h("div", { class: "bv-dmg-list stack", "data-testid": "damage-by-unit" }, h("div", { class: "label row spread" }, h("span", {}, "Damage dealt"), more), ...rows(you), ...rows(them));
  }

  // The battle fills the screen and never scrolls: the beat list below your
  // line takes what room is left, and the control bar stays pinned at the foot.
  show(
    h(
      "div",
      { class: "bv-screen" },
      hud,
      // On the phone .bv-board is display: contents (the stacked rows as before);
      // on desktop it lays your line left of the clash and theirs right, fronts in the middle.
      h(
        "div",
        { class: "bv-board" },
        // Where each front is (R2-17): on the phone both run front first from
        // the left; on desktop the fronts meet in the middle.
        h("div", { class: "label bv-lab-them" }, h("span", { class: "bv-dk" }, "← front · "), a.you ? "Them" : owner(them), h("span", { class: "bv-ph" }, " · front first")),
        enemy,
        caption,
        clash,
        mine,
        h("div", { class: "label bv-lab-you" }, a.you ? "You" : owner(you), h("span", { class: "bv-ph" }, " · front first"), h("span", { class: "bv-dk" }, " · front →")),
      ),
      timeline,
      h("div", { class: "bv-below" }, still, recent),
      controls,
    ),
    sheet,
    end,
    errBox,
  );
  screen("battle");
  addEventListener("resize", placeEnd, { signal: freed.signal });
  onGone(free);
  buildTimeline();
  buildLog();
  setSpeedVar();
  // Keys: Space plays or pauses, ←/→ step a beat, R replays. Esc (after a
  // sheet, closed in ui/dom.ts) leaves the Why tab for the Log, then opens
  // the run's menu, or goes back where a battle without a run came from.
  onKeys((e) => {
    // The end card is the result: Enter (or Space) goes on, as the result
    // screen's did, from its main button or with nothing in the card focused.
    // A focused Replay, Why, key moment or Damage row acts instead (R2-17 batch F).
    if (finished && !trace && (e.key === "Enter" || e.key === " ")) {
      const f = document.activeElement;
      if (f instanceof HTMLElement && f !== document.body && end.contains(f) && f.dataset.testid !== "battle-done") return false;
      return leave(), true;
    }
    if (e.key === " ") return playing ? pause() : play(), true;
    if (e.key === "ArrowLeft") return back(), true;
    if (e.key === "ArrowRight") return forward(), true;
    if (e.key.toLowerCase() === "r") return replay(), true;
    if (e.key === "Escape" && trace) return (trace = null), render(), true;
    if (e.key === "Escape" && a.outro) return openMenu(), true;
    if (e.key === "Escape") return leave(), true;
    return false;
  });
  // The Codex opened over the battle (a term's "Open in Codex") pauses it.
  onLeave(() => {
    pause();
    render();
  });
  render();
  schedule();
}

/** The "why I lost" card for side `you`: the 2–3 enemy chains that did the
 * most; with `won`, "why I won": your chains that did the most to them
 * (R2-17). Each row opens that chain's trace in the battle (onTrace). */
function whyPanel(battle: BattleRecord, you: Side, onTrace: (eventId: number) => void, won = false): HTMLElement {
  const chains = lossChains(battle.log, won ? (you === "A" ? "B" : "A") : you);
  return h(
    "div",
    { class: `panel stack${won ? " why-won" : ""}`, "data-testid": won ? "why-won" : "why-lost" },
    h("div", { class: "row spread" }, h("div", { class: "label" }, won ? "Why I won" : "Why I lost"), h("div", { class: "dim small" }, isDesktop() ? "click a row for its chain" : "tap a row for its chain")),
    ...(chains.length
      ? chains.map((c) => {
          const row = h(
            "button",
            { class: `bv-why${won ? " won" : ""}` },
            h("span", { class: "bv-why-chain" }, c.text),
            h("span", { class: "mono dim bv-why-num" }, [c.damage ? `${c.damage} dmg` : "", c.heal ? `+${c.heal} heal` : "", c.kills ? `${c.kills} ${c.kills === 1 ? "kill" : "kills"}` : ""].filter(Boolean).join(" · ")),
          );
          row.addEventListener("click", () => onTrace(c.sampleEventId));
          return row;
        })
      : [h("div", { class: "dim" }, won ? "None of your chains hurt them: fatigue ended it." : "No enemy chain hurt you: fatigue ended it.")]),
  );
}

function viaText(via: string): string {
  return via === "strike" ? "strike" : via === "ability" ? "ability" : via;
}

/** The When a stamped firing answered (AbilityRef.when): a unit's from the
 * recipe it fought with, a status's from its def. */
function whenLookup(units: Map<string, BattleUnit>, content: MvpContent): WhenOf {
  return (ref) => {
    if (ref.when === undefined) return undefined;
    if (ref.status === undefined) return units.get(ref.unit)?.recipe.when[ref.when];
    const def = content.statuses[ref.status];
    return def?.triggers?.[ref.when] ?? def?.abilities[ref.ability]?.whens?.[ref.when];
  };
}

/** Why's chain (R2-15): the change, then each cause, back to the turn. A
 * firing reads as its unit and the When → Does that fired; every step shows
 * its trigger's icon (fatigue its hourglass, an interception its status's).
 * With onStep each step is a button that shows its moment on the board
 * (`active` is the one on screen): a tap anywhere on it seeks, its words
 * included; a word's meaning is a long press (or a hover) away (R2-17).
 * With `rich` (names tagged by id) and `tag`, change and event steps read
 * like the captions: side tags, side-coloured names, highlighted terms. */
function chainView(c: Chain, o: { units: Map<string, BattleUnit>; content: MvpContent; you: Side; name: (id: string) => string; active?: number | null; onStep?: (eventId: number) => void; rich?: (text: string) => Node[]; tag?: (side: Side) => HTMLElement }): HTMLElement {
  const stepIcon = (n: ChainNode) => {
    const tone = n.kind === "firing" ? "tone-when" : n.kind === "root" ? "tone-gold" : "dim";
    let id = n.trigger ? termIcon(n.trigger as TermId, n.triggerStatus) : undefined;
    if (!id && n.event === "Fatigue") id = "hourglass";
    if (!id && n.event === "Intercepted") id = (n.triggerStatus ? (STATUS_TERMS[n.triggerStatus] ?? termDef(`status:${n.triggerStatus}` as TermId, o.content.statuses))?.icon : undefined) ?? "breaking-chain";
    return id ? icon(id, 18, tone) : h("span", { class: tone }, n.kind === "firing" ? "⚡" : "•");
  };
  const text = (t: string): Node[] => (o.rich ? o.rich(t) : [document.createTextNode(t)]);
  const tagOf = (n: ChainNode): HTMLElement[] => (o.tag && n.side ? [o.tag(n.side)] : []);
  const body = (n: ChainNode): Node[] => {
    const who = n.side ? (n.side === o.you ? "tone-ally" : "tone-enemy") : "";
    if (n.kind === "root") return [h("span", { class: "bv-step-text" }, n.text)];
    if (n.kind !== "firing") return [h("span", { class: "bv-step-text" }, ...tagOf(n), ...text(n.text))];
    const u = o.units.get(n.unit ?? "");
    // Scoped like the card says it: "Ally gets Shield", "Ally dies" (R3-19).
    const label = n.trigger ? triggerLabel(n.trigger as TermId, n.triggerStatus, n.triggerScope) : undefined;
    if (n.status) {
      const tip = termDef(`status:${n.status}` as TermId, o.content.statuses)?.tip;
      return [h("span", { class: "bv-step-head" }, ...tagOf(n), h("b", { class: who }, ...text(n.text)), label ? h("span", { class: "dim" }, ` · ${label}`) : null), tip ? h("span", { class: "bv-step-text dim" }, tip) : null].filter((x): x is HTMLElement => x !== null);
    }
    const w = u && n.ref?.when !== undefined ? u.recipe.when[n.ref.when] : undefined;
    const fired = u && w ? formRich({ ...u.recipe, when: [w] }, o.content) : [];
    return [
      h("span", { class: "bv-step-head" }, ...tagOf(n), h("span", { class: "emoji" }, u?.emoji ?? "✨"), " ", h("b", { class: who }, o.name(n.unit ?? "")), h("span", { class: "dim" }, `'s ability${label ? ` · ${label}` : ""}`)),
      fired.length ? h("span", { class: "bv-step-text" }, ...fired) : null,
    ].filter((x): x is HTMLElement => x !== null);
  };
  return h(
    "div",
    { class: "bv-chain", "data-testid": "why-chain" },
    ...c.nodes.map((n, i) => {
      const kids = [h("span", { class: "bv-step-ic" }, stepIcon(n)), h("span", { class: "bv-step-body" }, ...(i ? [h("span", { class: "bv-step-by dim" }, "caused by")] : []), ...body(n))];
      const attrs = { class: `bv-step ${n.kind}${o.active === n.eventId ? " on" : ""}`, "data-testid": "why-step", "data-kind": n.kind, "data-event": String(n.eventId) };
      if (!o.onStep) return h("div", attrs, ...kids);
      const b = h("button", attrs, ...kids);
      seekOnTap(b, () => o.onStep!(n.eventId));
      return b;
    }),
  );
}

/** How long a press on a word in a Why step must last to open its meaning. */
const LONG_PRESS_MS = 450;

/** A Why step's tap seeks, wherever it lands, its highlighted words
 * included: they are most of the step (R2-17). A long press on a word (or a
 * right-click) opens that word's meaning instead; a mouse hover still shows
 * its tip (ui/term.ts). */
function seekOnTap(step: HTMLElement, seek: () => void): void {
  let timer: ReturnType<typeof setTimeout> | null = null;
  let opening = false;
  let pressed = false;
  const cancel = () => { if (timer) clearTimeout(timer); timer = null; };
  const openTerm = (t: HTMLElement) => {
    opening = true;
    t.click();
    opening = false;
  };
  step.addEventListener(
    "click",
    (e) => {
      if (opening) return; // the word's own handler, called by a long press
      e.stopPropagation();
      e.preventDefault();
      if (pressed) { pressed = false; return; } // the click a long press ends with
      seek();
    },
    { capture: true },
  );
  step.addEventListener("pointerdown", (e) => {
    pressed = false;
    const t = (e.target as Element | null)?.closest?.<HTMLElement>(".t");
    if (!t || !step.contains(t)) return;
    cancel();
    timer = setTimeout(() => {
      timer = null;
      pressed = true;
      openTerm(t);
    }, LONG_PRESS_MS);
  });
  for (const ev of ["pointerup", "pointercancel", "pointerleave"]) step.addEventListener(ev, cancel);
  step.addEventListener("contextmenu", (e) => {
    const t = (e.target as Element | null)?.closest?.<HTMLElement>(".t");
    e.preventDefault();
    cancel();
    if (t && step.contains(t) && !pressed) openTerm(t);
    pressed = true;
  });
}
