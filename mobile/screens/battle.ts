// The battle viewer (mission #574, slice 9): playback with one-line captions
// (cause → effect) and the acting unit lit, tap any change to trace its chain,
// "why I lost" after a loss, speed 1×/2× and skip. Round 2 (R2-12) plays it
// beat by beat (a strike or turn end plus its cascade, in waves) with motion.
// The logic is pure and tested in src/mvp/trace.ts; this file only draws it.
//
// It opens any battle (a fight, a playoff game, the champion history); onDone
// goes on. Skip (data-testid="battle-skip", tapped by the phone e2e) calls
// onDone at once; the result screen shows whyILost() after a loss. Played to
// the end, the viewer shows the outcome, "why I lost", and battle-done.
import { boardAt, type BoardUnit } from "../../src/board";
import type { BattleRecord, BattleUnit, FightResult, MvpContent, RunView } from "../../src/mvp/contract";
import { beatPlayOf, stepsOf, timingOf, traceOf, whyILost as lossChains, sidesOf, type Change, type LossChain, type Step, type Trace } from "../../src/mvp/trace";
import { displayNames } from "../../src/trace";
import type { Side } from "../../src/types";
import { card, unitSheet } from "../ui/card";
import { button, closable, h, show } from "../ui/dom";

/** How long the line-up shows before the first beat, at 1×. */
const LINEUP_MS = 400;
/** How long a landed wave's motion runs: the longest animation (a float, 0.7 s, after up to 160 ms). A beat holds its last wave at least 0.7 s, so a beat change cuts a float off at most in its fade. */
const MOTION_MS = 900;

/** The changes that float up from a card. */
const FLOATS = new Set<Change["kind"]>(["damage", "heal", "buff", "debuff", "summon"]);

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

  let at = -1; // index of the beat on screen; -1 = the line-up before the first beat
  let wave = 0; // waves of that beat landed so far, minus one
  /** When each wave of the open beat landed (performance.now()), set only
   * when the playhead moved forward by itself. Every render re-applies the
   * motion of each landed wave, offset by its age, so a later wave's render
   * doesn't cut an earlier wave's lunge, shake or float short; empty after a
   * manual step, so nothing moves then. */
  let landed: number[] = [];
  let speed = 1;
  let playing = true;
  let timer: ReturnType<typeof setTimeout> | null = null;
  let trace: Trace | null = null;
  /** The changes the tapped chip held (one unit, one step), when it held more than one. */
  let traceGroup: Change[] = [];
  let finished = false;

  const hud = h("div", { class: "hud" });
  const enemy = h("div", { class: "slots bv-line", "data-testid": "battle-them" });
  const mine = h("div", { class: "slots bv-line", "data-testid": "battle-you" });
  const caption = h("button", { class: "bv-caption", "data-testid": "caption" });
  const recent = h("div", { class: "bv-recent", "data-testid": "recent" });
  const sheet = h("div", { class: "bv-sheet panel", "data-testid": "trace" });
  const end = h("div", { class: "bv-end stack" });
  const playBtn = button("❚❚", () => (playing ? pause() : play()), "", "battle-play");
  const backBtn = button("‹", () => { pause(); go(at - 1); }, "", "battle-back");
  const fwdBtn = button("›", () => { pause(); go(at + 1); }, "", "battle-step");
  const speedBtn = button("1×", () => { speed = speed === 1 ? 2 : 1; speedBtn.textContent = `${speed}×`; if (playing) schedule(); }, "", "battle-speed");
  const skipBtn = button("Skip", () => skip(), "", "battle-skip");
  const controls = h("div", { class: "row bv-controls" }, backBtn, playBtn, fwdBtn, speedBtn, skipBtn);
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
  function play(): void {
    if (finished) return;
    if (at >= beats.length - 1 && wave >= lastWave(at)) at = -1;
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
    traceGroup = group.length > 1 ? group : [];
    render();
  }
  function skip(): void {
    pause();
    leave();
  }
  function finish(): void {
    pause();
    finished = true;
    render();
  }
  function leave(): void {
    if (timer) clearTimeout(timer);
    a.onDone();
  }

  function unitCard(u: BoardUnit, side: Side, v: View): HTMLElement {
    const step = v.now;
    const changes = v.changes.filter((c) => c.unit === u.id);
    const statuses = u.statuses.map((s) => `${s.status} ${s.stacks}`).join(" · ");
    const el = card(units.get(u.id) ?? { emoji: emojiOf(u.id), name: u.name, stats: { pwr: u.pwr, hp: u.hp } }, {
      side: side === you ? "you" : "ghost",
      live: { stats: { pwr: u.pwr, hp: u.hp }, acting: step?.actor === u.id },
      extra: [
        statuses ? h("div", { class: "bv-status" }, statuses) : null,
        changes.length ? h("div", { class: "bv-changes" }, changeBadge(changes)) : null,
      ],
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
    const lines = changes.slice(0, 2).map((x, i) => h("span", { class: `bv-l ${x.kind}` }, i === 1 && more > 0 ? `${x.label} +${more}` : x.label));
    const b = h(
      "button",
      { class: `bv-change ${c.kind}${changes.length > 1 ? " multi" : ""}`, "data-testid": "change", "data-event": String(c.eventId), "data-count": String(changes.length), "aria-label": changes.map((x) => x.label).join(", ") },
      h("span", { class: `bv-pill${changes.length > 1 ? " two" : ""}` }, ...(changes.length > 1 ? lines : [c.label])),
    );
    b.addEventListener("click", (ev) => { ev.stopPropagation(); openTrace(c.eventId, changes); });
    return b;
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
      death ? h("div", { class: "bv-changes" }, changeBadge(v.changes.filter((c) => c.unit === id))) : null,
    );
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
    for (const { step, age } of v.waves) {
      if (age === null || age > MOTION_MS) continue;
      const t = `${-Math.round(age)}ms`;
      const mine = step.changes.filter((c) => c.unit === id);
      let move: string | null = null;
      if (step.actor === id) {
        const first = log[step.eventIds[0]!];
        const struck = first?.type === "Hurt" && first.source === "kernel" && first.causedBy !== null && log[first.causedBy]?.type === "Strike";
        move = struck ? (side === you ? "bv-lunge-up" : "bv-lunge-down") : "bv-pulse";
      }
      if (mine.some((c) => c.kind === "damage")) move = "bv-hit";
      if (mine.some((c) => c.kind === "death")) move = "bv-dying";
      // The latest wave's movement wins; its delay drives the card's animation.
      if (move) {
        el.classList.remove("bv-lunge-up", "bv-lunge-down", "bv-pulse", "bv-hit", "bv-dying");
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
        f.style.animationDelay = `${i * 80 - Math.round(age)}ms`;
        f.style.setProperty("--k", String(k++ % 3));
        el.append(f);
      });
    }
  }

  /** A line in the beat: the living units, and each unit that fell in this
   * beat back in the slot it held when the beat began. */
  function lineOf(side: Side, board: ReturnType<typeof boardAt>, before: ReturnType<typeof boardAt>, v: View): HTMLElement[] {
    const living: HTMLElement[] = board.lines[side].map((u) => unitCard(u, side, v));
    const fallen = board.graves[side]
      .map((u) => ({ card: deadCard(u.id, v), slot: before.lines[side].findIndex((b) => b.id === u.id) }))
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
    const now = performance.now();
    const v: View = {
      now: step,
      changes: shown.flatMap((w) => w.changes),
      waves: shown.map((w, i) => ({ step: w, age: landed[i] !== undefined ? now - landed[i]! : null })),
    };
    const board = boardAt(log, upto);
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
    caption.replaceChildren(...captionKids(step), ...(reduced() ? stillList(v.changes) : []));
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
    end.style.display = finished ? "" : "none";
    if (finished) end.replaceChildren(...endView());
    backBtn.disabled = at < 0;
    fwdBtn.disabled = at >= beats.length - 1 && wave >= lastWave(at);
    controls.style.display = finished ? "none" : "";
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
    if (!side) return [document.createTextNode(step.caption)];
    return [sideTag(side, "caption-side"), document.createTextNode(step.caption)];
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
      ...t.links.map((l, i) =>
        h(
          "div",
          { class: `bv-link ${l.side === you ? "you" : "ghost"}`, "data-testid": "trace-link" },
          h("span", { class: "emoji" }, emojiOf(l.unit)),
          // Whose unit, then its name: "THEM Taser".
          h("span", { class: "bv-link-name" }, ...(l.side ? [sideTag(l.side)] : []), l.name),
          h("span", { class: "dim" }, i === 0 ? viaText(l.via) : `${viaText(l.via)}, set off the one above`),
        ),
      ),
      t.links.length ? null : h("div", { class: "dim" }, "No unit acted: the rules did this."),
    ].filter((n): n is HTMLDivElement => n !== null);
  }

  function endView(): Node[] {
    // Only a viewer with a side wins or loses; otherwise the winner is named.
    const word = !a.you ? (battle.winner === "draw" ? "DRAW" : `${owner(battle.winner)} wins`) : outcome === "win" ? "VICTORY" : outcome === "loss" ? "DEFEAT" : "DRAW";
    const cls = !a.you && battle.winner !== "draw" ? "neutral" : outcome;
    const why = lost ? whyPanel(battle, you, (id) => openTrace(id)) : null;
    return [h("div", { class: `bv-word ${cls}`, "data-testid": "battle-word" }, word), ...(why ? [why] : []), button("Continue", leave, "primary", "battle-done")];
  }

  show(hud, h("div", { class: "label" }, a.you ? "Them" : owner(them)), enemy, caption, mine, h("div", { class: "label" }, a.you ? "You · front first" : `${owner(you)} · front first`), recent, sheet, end, h("div", { class: "spacer" }), controls);
  render();
  schedule();
}

/** The "why I lost" card for side `you`: the 2–3 enemy chains that did the
 * most. Each row opens that chain's trace (onTrace, or a sheet of its own). */
function whyPanel(battle: BattleRecord, you: Side, onTrace?: (eventId: number) => void): HTMLElement {
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
            closable(
              h("div", { class: "stack why-sheet", "data-testid": "why-sheet" },
                h("h2", { class: "ghost-name" }, c.text),
                h("div", {}, chainSummary(c)),
                h("div", { class: "label" }, SAMPLE_LABEL[c.sampleKind]),
                h("div", { class: "bv-trace-text mono", "data-testid": "trace-text" }, t.text),
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
export function whyILost(battle: BattleRecord, _content: MvpContent, you: Side): HTMLElement | null {
  if (battle.winner === "draw" || battle.winner === you) return null;
  return whyPanel(battle, you);
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
