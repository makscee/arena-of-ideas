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
// by unit, 2–3 key moments that replay from there, and Replay / Why I lost /
// Continue (battle-done, which calls onDone).
import { boardAt, type BoardUnit } from "../../src/board";
import type { BattleRecord, BattleUnit, FightResult, MvpContent, RunView } from "../../src/mvp/contract";
import { STATUS_TERMS, termDef, termIcon, type TermId } from "../../src/glossary";
import { beatPlayOf, chainOf, damageByUnit, firingOf, keyMomentsOf, stepsOf, timingOf, traceOf, whyILost as lossChains, sidesOf, type Chain, type ChainNode, type Change, type Firing, type LossChain, type Step, type Trace, type WhenOf } from "../../src/mvp/trace";
import { displayNames } from "../../src/trace";
import type { Side } from "../../src/types";
import { card, formRich, unitSheet } from "../ui/card";
import { app, button, closable, h, onKeys, onLeave, show } from "../ui/dom";
import { icon } from "../ui/icon";

/** How long the line-up shows before the first beat, at 1×. */
const LINEUP_MS = 400;
/** How long a landed wave's motion runs, in animation time (real time × speed): the longest animation (a killing blow's shake, then its 0.5 s pop, ends at 0.92 s; a float, 0.7 s, after up to 160 ms). A beat holds its last wave at least 0.7 s, so a beat change cuts motion off at most in its fade. */
const MOTION_MS = 1000;

/** The changes that float up from a card. */
const FLOATS = new Set<Change["kind"]>(["damage", "heal", "buff", "debuff", "summon"]);

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

/** Reduced motion: nothing moves, and beats hold a little longer. */
const reduced = () => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;

export function battleScreen(a: { battle: BattleRecord; content: MvpContent; you?: Side; fight?: FightResult; run?: RunView; onDone: () => void }): void {
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
  const beats = beatPlayOf(log, stepsOf(log, name, sides, a.you ? { you: a.you } : { sideName: owner }), name);
  const lost = a.you !== undefined && outcome === "loss";
  const units = new Map<string, BattleUnit>([...battle.teamA, ...battle.teamB].map((u) => [u.id, u]));
  const emojiOf = (id: string) => units.get(id)?.emoji ?? "✨";
  const whenOf = whenLookup(units, a.content);
  // The end card's numbers (R2-14).
  const damage = damageByUnit(log, name, sides);
  const moments = keyMomentsOf(log, beats, name, sides);

  // What richCaption highlights: this battle's unit names (by side), its
  // statuses, and the fixed words. Longest first, so "Rat King" beats "Rat".
  const unitSide = new Map<string, Side>();
  for (const [id, s] of sides) unitSide.set(name(id), s);
  const logStatuses = new Set(log.flatMap((e) => (e.type === "StatusApplied" ? [e.status] : [])));
  const esc = (w: string) => w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const words = [...new Set([...unitSide.keys(), ...Object.keys(STATUS_TERMS), ...logStatuses, "Fatigue"])].filter(Boolean).sort((p, q) => q.length - p.length);
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

  /** Motion runs at the playback speed (CSS reads --bv-sp), so 2× never cuts it. */
  const setSpeedVar = () => { for (const row of [enemy, mine]) row.style.setProperty("--bv-sp", String(speed)); };

  const hud = h("div", { class: "hud" });
  const enemy = h("div", { class: "slots bv-line theirs", "data-testid": "battle-them" });
  const mine = h("div", { class: "slots bv-line mine", "data-testid": "battle-you" });
  const caption = h("button", { class: "bv-caption", "data-testid": "caption" });
  /** Reduced motion's list of the beat's changes: below your line, its height
   * fixed, so nothing on the board moves while it fills. */
  const still = h("div", { class: "bv-stillbox" });
  const recent = h("div", { class: "bv-recent", "data-testid": "recent" });
  const sheet = h("div", { class: "bv-sheet panel", "data-testid": "trace" });
  const end = h("div", { class: "bv-end panel stack", "data-testid": "end-card" });
  const playBtn = button("❚❚", () => (playing ? pause() : play()), "", "battle-play");
  const backBtn = button("‹", () => back(), "", "battle-back");
  const fwdBtn = button("›", () => forward(), "", "battle-step");
  const speedBtn = button(`${speed}×`, () => setSpeed(SPEEDS[(SPEEDS.indexOf(speed as 1) + 1) % SPEEDS.length]!), "", "battle-speed");
  const endBtn = button("End", () => finish(), "", "battle-end");
  const replayBtn = button("↻", () => replay(), "", "battle-replay");
  replayBtn.setAttribute("aria-label", "Replay from the start");
  const controls = h("div", { class: "row bv-controls" }, backBtn, playBtn, fwdBtn, speedBtn, endBtn, replayBtn);
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
    at = Math.max(-1, Math.min(beats.length - 1, i));
    wave = lastWave(at);
    landed = [];
    render();
  }
  function openTrace(eventId: number, group: Change[] = []): void {
    pause();
    trace = traceOf(log, eventId, name, sides);
    chain = chainOf(log, eventId, { name, sides, whenOf });
    chainAt = null;
    traceGroup = group.length > 1 ? group : [];
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
    finished = true;
    trace = null;
    at = beats.length - 1;
    wave = lastWave(at);
    landed = [];
    render();
  }
  /** Plays from beat i (-1: the line-up), the end card put away. */
  function playFrom(i: number): void {
    pause();
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
  function leave(): void {
    if (timer) clearTimeout(timer);
    a.onDone();
  }

  function unitCard(u: BoardUnit, side: Side, v: View): HTMLElement {
    const step = v.now;
    const changes = v.changes.filter((c) => c.unit === u.id);
    const el = card(units.get(u.id) ?? { emoji: emojiOf(u.id), name: u.name, stats: { pwr: u.pwr, hp: u.hp } }, {
      side: side === you ? "you" : "ghost",
      live: { stats: { pwr: u.pwr, hp: u.hp }, maxHp: u.maxHp, acting: step?.actor === u.id },
      extra: [statusChips(u.statuses), changes.length ? h("div", { class: "bv-changes" }, changeBadge(changes)) : null],
    });
    el.classList.add("bv-card");
    el.dataset.unit = u.id;
    if (u.silenced) el.classList.add("silenced");
    motion(el, u.id, side, v);
    // The change's chip traces it; the rest of the card opens the unit.
    el.addEventListener("click", () => openUnit(u.id, changes[0]));
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
  /** A change as the card shows it; a hit Shield fully blocked is the Shield
   * icon and what it blocked, never "−0". */
  function changeLabel(c: Change): (Node | string)[] {
    const blocked = blockedBy(c);
    if (blocked) return [icon("shield", 14, "tone-shield"), h("span", { class: "tone-shield" }, `${blocked}`)];
    return [c.label];
  }
  /** A card's statuses (R2-13): each one's icon and stacks, in its colour. The
   * row keeps its height when empty, so a status landing moves nothing. */
  function statusChips(statuses: { status: string; stacks: number }[]): HTMLElement {
    return h(
      "div",
      { class: "bv-sts", "data-testid": "card-statuses" },
      ...statuses.map((st) => {
        const def = STATUS_TERMS[st.status] ?? termDef(`status:${st.status}`);
        const ic = def?.icon;
        return h(
          "span",
          { class: `bv-st tone-${def?.tone ?? "plain"}`, title: `${st.status} ${st.stacks}`, "aria-label": `${st.status} ${st.stacks}`, "data-testid": "card-status", "data-status": st.status },
          ...(ic ? [icon(ic, 12)] : [st.status.slice(0, 2)]),
          h("b", {}, `${st.stacks}`),
        );
      }),
    );
  }
  /** The trigger badge over a unit whose ability fired this beat: [When icon]
   * → [Does icon]. It pops when its wave lands, stays for the beat, and opens
   * that step's Why. */
  function triggerBadge(id: string, v: View): HTMLElement | null {
    let hit: { f: Firing; step: Step; age: number | null } | null = null;
    for (const w of v.waves) {
      const f = firingOf(log, w.step, whenOf);
      if (f && f.unit === id) hit = { f, step: w.step, age: w.age };
    }
    if (!hit) return null;
    const { f, step, age } = hit;
    const trig = f.trigger ? termIcon(f.trigger as TermId, f.triggerStatus) : undefined;
    const eff = termIcon(f.effect as TermId);
    const effTone = termDef(f.effect as TermId)?.tone ?? "plain";
    const when = f.trigger ? (termDef(f.trigger as TermId)?.label ?? "") : "";
    const does = f.effectStatus ?? termDef(f.effect as TermId)?.label ?? "";
    const b = h(
      "button",
      { class: "bv-badge", "data-testid": "trigger-badge", "data-trigger": f.trigger ?? "", "data-effect": f.effect, "aria-label": `${when}${f.triggerStatus ? ` (${f.triggerStatus})` : ""} → ${does}` },
      trig ? icon(trig, 14, "tone-when") : h("span", { class: "tone-when" }, "⚡"),
      h("span", { class: "bv-badge-arrow" }, "→"),
      eff ? icon(eff, 14, `tone-${effTone}`) : h("span", { class: `tone-${effTone}` }, does.slice(0, 3)),
    );
    if (age !== null && age <= MOTION_MS) b.style.setProperty("--bv-bt", `${-Math.round(age * speed)}ms`);
    else b.classList.add("still");
    b.addEventListener("click", (ev) => {
      ev.stopPropagation();
      const c = step.changes[0];
      if (c) openTrace(c.eventId, step.changes.filter((x) => x.unit === c.unit));
    });
    return b;
  }
  /** A card in its slot, with the trigger badge above it when one fired. */
  function slot(el: HTMLElement, id: string, v: View): HTMLElement {
    return h("div", { class: "bv-slot" }, el, triggerBadge(id, v));
  }
  /** A card tap: the unit's sheet when it entered the battle; a summon has none, so its change's trace. */
  function openUnit(id: string, change: Change | undefined): void {
    const u = units.get(id);
    if (!u) return change ? openTrace(change.eventId) : undefined;
    pause();
    render();
    closable(unitSheet(u, a.content));
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
    el.addEventListener("click", () => openUnit(id, death));
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
    let k = 0;
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
      if (mine.some((c) => c.kind === "death")) {
        // The killing blow's shake plays out, then the card pops (R2-13):
        // the die starts once the shake (80 ms in, 340 ms long) is over.
        const own = age * speed;
        const start = hitAge !== null ? Math.max(-own, 420 - hitAge) : -own;
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
      // Numbers float (−n, +n, ±PWR, a summon); a status or a death already
      // sits in the card's chip, so floating it too only covers the card.
      mine.filter((c) => FLOATS.has(c.kind)).slice(0, 3).forEach((c, i) => {
        const f = h("span", { class: `bv-float ${c.kind}`, "aria-hidden": "true" }, c.label);
        f.style.animationDelay = `${Math.round((i * 80) / speed - age)}ms`;
        const e = log[c.eventId];
        // ±PWR sits over PWR (left), ±HP over HP (right); a blocked hit shows the Shield.
        if (e?.type === "StatChanged") f.classList.add(e.stat === "pwr" ? "on-pwr" : "on-hp");
        if (blockedBy(c)) f.replaceChildren(...changeLabel(c));
        f.style.setProperty("--k", String(k++ % 3));
        el.append(f);
      });
    }
  }

  /** A line in the beat: the living units, and each unit that fell in this
   * beat back in the slot it held when the beat began. */
  function lineOf(side: Side, board: ReturnType<typeof boardAt>, before: ReturnType<typeof boardAt>, v: View): HTMLElement[] {
    const living: HTMLElement[] = board.lines[side].map((u) => slot(unitCard(u, side, v), u.id, v));
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
    const before = beat ? boardAt(log, Math.max(0, beat.start - 1)) : board;
    const turn = step?.turn ?? 0;
    hud.replaceChildren(
      h("span", {}, battle.kind === "crown" ? "Crown fight" : battle.kind === "playoff" ? "Playoff" : `Round ${battle.round}`),
      h("span", { class: "dim who", title: battle.opponent.name }, `vs @${battle.opponent.name}`),
      h("span", {}, turn ? `T${turn}` : "—"),
    );
    for (const [side, row] of [[them, enemy], [you, mine]] as const) {
      const cards = lineOf(side, board, before, v);
      row.replaceChildren(...cards);
      // A falling card beside a full line widens the row instead of wrapping it.
      row.style.gridTemplateColumns = `repeat(${Math.max(5, cards.length)}, minmax(0, 1fr))`;
      row.classList.toggle("empty", !cards.length);
      if (!cards.length) row.append(h("div", { class: "dim" }, "No one standing."));
    }
    caption.replaceChildren(h("span", { class: "bv-cap" }, ...captionKids(step)));
    still.replaceChildren(...(reduced() ? stillList(v.changes) : []));
    still.style.display = reduced() && !finished ? "" : "none";
    caption.classList.toggle("tappable", !!step?.changes.length);
    recent.replaceChildren(
      ...beats.slice(Math.max(0, at - 3), Math.max(0, at)).reverse().map((pb) => {
        // A past beat reads as its first wave, the strike or tick that opened it.
        const s = pb.waves[0]!;
        const b = h("button", { class: "bv-past" }, ...(s.subjectSide ? [sideTag(s.subjectSide)] : []), s.caption, pb.waves.length > 1 ? h("span", { class: "dim" }, ` +${pb.waves.length - 1}`) : null);
        b.addEventListener("click", () => {
          const c = s.changes[0];
          if (c) openTrace(c.eventId, s.changes.filter((x) => x.unit === c.unit));
        });
        return b;
      }),
    );
    recent.style.display = finished ? "none" : "";
    sheet.style.display = trace ? "" : "none";
    if (trace) sheet.replaceChildren(...traceView(trace));
    // A trace opened from the end card (Why I lost) sits in its place until closed.
    end.style.display = finished && !trace ? "" : "none";
    if (finished) end.replaceChildren(...endView());
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
    if (!side) return richCaption(step.caption);
    return [sideTag(side, "caption-side"), ...richCaption(step.caption)];
  }

  /** A caption with its terms highlighted (R2-13, like R2-8's unit text):
   * unit names in their side's colour, statuses with their icon and colour,
   * PWR / HP, damage and healing numbers, and a Shield block as the Shield
   * icon. Plain spans: the caption itself is the button that opens Why. */
  function richCaption(text: string): Node[] {
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
    const side = unitSide.get(t);
    if (side) return h("span", { class: `bv-cn ${side === you ? "tone-ally" : "tone-enemy"}` }, t);
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
    return [h("span", { class: "bv-still", "data-testid": "caption-changes" }, ...changes.map((c) => h("span", { class: `bv-l ${c.kind}` }, `${name(c.unit)} ${c.label}`)))];
  }

  function traceView(t: Trace): Node[] {
    const target = t.change ? name(t.change.unit) : "";
    return [
      h("div", { class: "row spread" }, h("div", { class: "label" }, `Why: ${t.change?.label ?? ""} ${target}`), button("✕", () => { trace = null; render(); }, "bv-close", "trace-close")),
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
      chain ? chainView(chain, { units, content: a.content, you, name, active: chainAt, onStep: seekEvent }) : null,
    ].filter((n): n is HTMLDivElement => n !== null);
  }

  function endView(): Node[] {
    // Only a viewer with a side wins or loses; otherwise the winner is named.
    const word = !a.you ? (battle.winner === "draw" ? "DRAW" : `${owner(battle.winner)} wins`) : outcome === "win" ? "VICTORY" : outcome === "loss" ? "DEFEAT" : "DRAW";
    const cls = !a.you && battle.winner !== "draw" ? "neutral" : outcome;
    const whyBtn = lost
      ? button("Why I lost", () => {
          const close = closable(whyPanel(battle, a.content, you, (id) => { close(); openTrace(id); }));
        }, "", "end-why")
      : null;
    const hide = button("✕", () => { finished = false; render(); }, "bv-close", "end-close");
    hide.setAttribute("aria-label", "See the board");
    return [
      h("div", { class: "row spread bv-end-top" }, h("div", { class: `bv-word ${cls}`, "data-testid": "battle-word" }, word), hide),
      damageView(),
      moments.length ? h("div", { class: "label" }, "Key moments") : null,
      ...moments.map((m) => {
        const b = button("", () => playFrom(m.beat), `bv-moment ${m.kind}`, "key-moment");
        b.append(h("span", { class: "bv-moment-t dim" }, `T${beats[m.beat]?.turn ?? ""}`), h("span", { class: "bv-moment-l" }, m.label), h("span", { class: "dim" }, "▶"));
        b.dataset.beat = String(m.beat);
        return b;
      }),
      h("div", { class: "row bv-end-actions" }, button("Replay", replay, "", "end-replay"), whyBtn, button("Continue", leave, "primary grow", "battle-done")),
    ].filter((n): n is HTMLElement => n !== null);
  }

  /** Damage by unit, both sides: a bar per unit, scaled to the battle's biggest. */
  function damageView(): HTMLElement {
    const top = Math.max(1, ...damage.map((d) => d.damage));
    const col = (side: Side) =>
      h(
        "div",
        { class: `bv-dmg-col ${side === you ? "you" : "ghost"}` },
        h("div", { class: "label" }, a.you ? (side === you ? "You" : "Them") : owner(side)),
        ...damage
          .filter((d) => d.side === side)
          .slice(0, 6)
          .map((d) =>
            h(
              "div",
              { class: "bv-dmg", "data-testid": "damage-row", title: `${d.name}: ${d.damage} damage` },
              h("span", { class: "emoji" }, emojiOf(d.unit)),
              h("span", { class: "bv-dmg-bar" }, h("i", { style: `width:${Math.round((d.damage / top) * 100)}%` })),
              h("span", { class: "mono" }, String(d.damage)),
            ),
          ),
      );
    return h("div", { class: "bv-dmg-grid", "data-testid": "damage-by-unit" }, col(you), col(them));
  }

  // The battle fills the screen and never scrolls: the beat list below your
  // line takes what room is left, and the control bar stays pinned at the foot.
  show(
    h(
      "div",
      { class: "bv-screen" },
      hud,
      h("div", { class: "label" }, a.you ? "Them" : owner(them)),
      enemy,
      caption,
      mine,
      h("div", { class: "label" }, a.you ? "You · front first" : `${owner(you)} · front first`),
      h("div", { class: "bv-below" }, still, recent),
      controls,
    ),
    sheet,
    end,
  );
  setSpeedVar();
  // Desktop keys: Space plays or pauses, ←/→ step a beat, R replays.
  onKeys((e) => {
    const over = app.querySelector(".overlay");
    if (over) return e.key === "Escape" ? (over.remove(), true) : false;
    if (e.key === " ") return playing ? pause() : play(), true;
    if (e.key === "ArrowLeft") return back(), true;
    if (e.key === "ArrowRight") return forward(), true;
    if (e.key.toLowerCase() === "r") return replay(), true;
    if (e.key === "Escape" && trace) return (trace = null), render(), true;
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
 * most. Each row opens that chain's trace (onTrace, or a sheet of its own). */
function whyPanel(battle: BattleRecord, content: MvpContent, you: Side, onTrace?: (eventId: number) => void): HTMLElement {
  const chains = lossChains(battle.log, you);
  return h(
    "div",
    { class: "panel stack", "data-testid": "why-lost" },
    h("div", { class: "row spread" }, h("div", { class: "label" }, "Why I lost"), h("div", { class: "dim small" }, "tap a row for its chain")),
    ...(chains.length
      ? chains.map((c) => {
          const row = h(
            "button",
            { class: "bv-why" },
            h("span", { class: "bv-why-chain" }, c.text),
            h("span", { class: "mono dim bv-why-num" }, [c.damage ? `${c.damage} dmg` : "", c.heal ? `+${c.heal} heal` : "", c.kills ? `${c.kills} ${c.kills === 1 ? "kill" : "kills"}` : ""].filter(Boolean).join(" · ")),
          );
          row.addEventListener("click", () => {
            if (onTrace) return onTrace(c.sampleEventId);
            const t = traceOf(battle.log, c.sampleEventId);
            const units = new Map<string, BattleUnit>([...battle.teamA, ...battle.teamB].map((u) => [u.id, u]));
            const ch = chainOf(battle.log, c.sampleEventId, { whenOf: whenLookup(units, content) });
            closable(
              h("div", { class: "stack why-sheet", "data-testid": "why-sheet" },
                h("h2", { class: "ghost-name" }, c.text),
                h("div", {}, chainSummary(c)),
                h("div", { class: "label" }, SAMPLE_LABEL[c.sampleKind]),
                h("div", { class: "bv-trace-text mono", "data-testid": "trace-text" }, t.text),
                chainView(ch, { units, content, you, name: displayNames(battle.log) }),
              ),
            );
          });
          return row;
        })
      : [h("div", { class: "dim" }, "No enemy chain hurt you: fatigue ended it.")]),
  );
}

/** The "why I lost" card for the result screen, after a loss (a skipped
 * battle included). Null when `you` didn't lose. */
export function whyILost(battle: BattleRecord, content: MvpContent, you: Side): HTMLElement | null {
  if (battle.winner === "draw" || battle.winner === you) return null;
  return whyPanel(battle, content, you);
}

/** The why sheet's label over the traced change (LossChain.sampleKind). */
const SAMPLE_LABEL: Record<LossChain["sampleKind"], string> = {
  kill: "Its biggest killing blow, traced",
  hit: "Its biggest hit, traced",
  heal: "Its biggest heal, traced",
  none: "Its first change, traced",
};

/** One sentence for a why-I-lost row's numbers, so the totals and the traced
 * hit can't read as a contradiction: "11 damage to your units over 4 hits,
 * 2 kills. +3 healing to theirs over 2 heals." */
function chainSummary(c: LossChain): string {
  const parts: string[] = [];
  if (c.hits) parts.push(`${c.damage} damage to your units over ${c.hits} ${c.hits === 1 ? "hit" : "hits"}${c.kills ? `, ${c.kills} ${c.kills === 1 ? "kill" : "kills"}` : ""}.`);
  if (c.heals) parts.push(`${c.heal} healing to theirs over ${c.heals} ${c.heals === 1 ? "heal" : "heals"}.`);
  return parts.join(" ") || `${c.times} changes this fight.`;
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
 * its trigger's icon. With onStep each step is a button that shows its
 * moment on the board (`active` is the one on screen). */
function chainView(c: Chain, o: { units: Map<string, BattleUnit>; content: MvpContent; you: Side; name: (id: string) => string; active?: number | null; onStep?: (eventId: number) => void }): HTMLElement {
  const stepIcon = (n: ChainNode) => {
    const id = n.trigger ? termIcon(n.trigger as TermId, n.triggerStatus) : undefined;
    const tone = n.kind === "firing" ? "tone-when" : n.kind === "root" ? "tone-gold" : "dim";
    return id ? icon(id, 18, tone) : h("span", { class: tone }, n.kind === "firing" ? "⚡" : "•");
  };
  const body = (n: ChainNode): Node[] => {
    const who = n.side ? (n.side === o.you ? "tone-ally" : "tone-enemy") : "";
    if (n.kind !== "firing") return [h("span", { class: "bv-step-text" }, n.text)];
    const u = o.units.get(n.unit ?? "");
    const label = n.trigger ? termDef(n.trigger as TermId)?.label : undefined;
    if (n.status) {
      const tip = termDef(`status:${n.status}` as TermId, o.content.statuses)?.tip;
      return [h("span", { class: "bv-step-head" }, h("b", { class: who }, n.text), label ? h("span", { class: "dim" }, ` · ${label}`) : null), tip ? h("span", { class: "bv-step-text dim" }, tip) : null].filter((x): x is HTMLElement => x !== null);
    }
    const w = u && n.ref?.when !== undefined ? u.recipe.when[n.ref.when] : undefined;
    const fired = u && w ? formRich({ ...u.recipe, when: [w] }, o.content) : [];
    return [
      h("span", { class: "bv-step-head" }, h("span", { class: "emoji" }, u?.emoji ?? "✨"), " ", h("b", { class: who }, o.name(n.unit ?? "")), h("span", { class: "dim" }, `'s ability${label ? ` · ${label}` : ""}`)),
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
      b.addEventListener("click", () => o.onStep!(n.eventId));
      return b;
    }),
  );
}
