// The battle viewer (mission #574, slice 9): playback with one-line captions
// (cause → effect) and the acting unit lit, tap any change to trace its chain,
// "why I lost" after a loss, speed 1×/2× and skip. The logic is pure and
// tested in src/mvp/trace.ts; this file only draws it.
//
// It opens any battle (a fight, a playoff game, the champion history); onDone
// goes on. Skip (data-testid="battle-skip", tapped by the phone e2e) calls
// onDone at once; the result screen shows whyILost() after a loss. Played to
// the end, the viewer shows the outcome, "why I lost", and battle-done.
import { boardAt, type BoardUnit } from "../../src/board";
import type { BattleRecord, BattleUnit, FightResult, MvpContent, RunView } from "../../src/mvp/contract";
import { stepsOf, traceOf, whyILost as lossChains, sidesOf, type Change, type Step, type Trace } from "../../src/mvp/trace";
import { displayNames } from "../../src/trace";
import type { Side } from "../../src/types";
import { card } from "../ui/card";
import { button, h, overlay, show } from "../ui/dom";

/** Milliseconds a step stays on screen at 1×. */
const STEP_MS = 650;

export function battleScreen(a: { battle: BattleRecord; content: MvpContent; you?: Side; fight?: FightResult; run?: RunView; onDone: () => void }): void {
  const { battle } = a;
  const you: Side = a.you ?? "A";
  const them: Side = you === "A" ? "B" : "A";
  const outcome = battle.winner === "draw" ? "draw" : battle.winner === you ? "win" : "loss";
  const log = battle.log;
  const name = displayNames(log);
  const sides = sidesOf(log);
  const steps = stepsOf(log, name, sides);
  const lost = a.you !== undefined && outcome === "loss";
  const units = new Map<string, BattleUnit>([...battle.teamA, ...battle.teamB].map((u) => [u.id, u]));
  const emojiOf = (id: string) => units.get(id)?.emoji ?? "✨";

  let at = -1; // index of the step on screen; -1 = the line-up before the first step
  let speed = 1;
  let playing = true;
  let timer: ReturnType<typeof setTimeout> | null = null;
  let trace: Trace | null = null;
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

  caption.addEventListener("click", () => {
    const c = steps[at]?.changes[0];
    if (c) openTrace(c.eventId);
  });

  function schedule(): void {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      if (at >= steps.length - 1) return finish();
      go(at + 1);
      schedule();
    }, STEP_MS / speed);
  }
  function play(): void {
    if (finished) return;
    if (at >= steps.length - 1) at = -1;
    playing = true;
    trace = null;
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
  function go(i: number): void {
    at = Math.max(-1, Math.min(steps.length - 1, i));
    render();
  }
  function openTrace(eventId: number): void {
    pause();
    trace = traceOf(log, eventId, name, sides);
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

  function unitCard(u: BoardUnit, side: Side, step: Step | undefined): HTMLElement {
    const changes = step?.changes.filter((c) => c.unit === u.id) ?? [];
    const statuses = u.statuses.map((s) => `${s.status} ${s.stacks}`).join(" · ");
    const el = card(units.get(u.id) ?? { emoji: emojiOf(u.id), name: u.name, stats: { pwr: u.pwr, hp: u.hp } }, {
      side: side === you ? "you" : "ghost",
      live: { stats: { pwr: u.pwr, hp: u.hp }, acting: step?.actor === u.id },
      extra: [
        statuses ? h("div", { class: "bv-status" }, statuses) : null,
        changes.length ? h("div", { class: "bv-changes" }, ...changes.map(changeBadge)) : null,
      ],
    });
    el.classList.add("bv-card");
    el.dataset.unit = u.id;
    if (u.silenced) el.classList.add("silenced");
    // The whole card is the tap target for its change (44 px and more).
    if (changes[0]) el.addEventListener("click", () => openTrace(changes[0]!.eventId));
    return el;
  }
  function changeBadge(c: Change): HTMLElement {
    const b = h("button", { class: `bv-change ${c.kind}`, "data-testid": "change", "data-event": String(c.eventId) }, c.label);
    b.addEventListener("click", (ev) => { ev.stopPropagation(); openTrace(c.eventId); });
    return b;
  }
  function deadCard(id: string, step: Step): HTMLElement | null {
    // A unit that falls this step still shows, crossed out, so its ✝ can be tapped.
    const death = step.changes.find((c) => c.unit === id && c.kind === "death");
    if (!death) return null;
    const el = h(
      "div",
      { class: `card bv-card dead ${sides.get(id) === you ? "you" : "ghost"}` },
      h("div", { class: "emoji" }, emojiOf(id)),
      h("div", { class: "name" }, name(id)),
      h("div", { class: "bv-changes" }, ...step.changes.filter((c) => c.unit === id).map(changeBadge)),
    );
    return el;
  }

  function render(): void {
    const step = steps[at];
    const upto = step ? step.eventIds.at(-1)! : 0;
    const board = boardAt(log, upto);
    const turn = step?.turn ?? 0;
    hud.replaceChildren(
      h("span", {}, battle.kind === "crown" ? "Crown fight" : battle.kind === "playoff" ? "Playoff" : `Round ${battle.round}`),
      h("span", { class: "dim" }, `vs @${battle.opponent.name}`),
      h("span", {}, turn ? `T${turn}` : "—"),
    );
    for (const [side, row] of [[them, enemy], [you, mine]] as const) {
      const living = board.lines[side].map((u) => unitCard(u, side, step));
      const falling = step ? board.graves[side].flatMap((u) => deadCard(u.id, step) ?? []) : [];
      row.replaceChildren(...living, ...falling);
      row.classList.toggle("empty", !living.length && !falling.length);
      if (!living.length && !falling.length) row.append(h("div", { class: "dim" }, "No one standing."));
    }
    caption.textContent = step ? step.caption : "The lines face off.";
    caption.classList.toggle("tappable", !!step?.changes.length);
    recent.replaceChildren(
      ...steps.slice(Math.max(0, at - 3), Math.max(0, at)).reverse().map((s) => {
        const b = h("button", { class: "bv-past" }, s.caption);
        b.addEventListener("click", () => {
          const c = s.changes[0];
          if (c) openTrace(c.eventId);
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
    fwdBtn.disabled = at >= steps.length - 1;
    controls.style.display = finished ? "none" : "";
  }

  function traceView(t: Trace): Node[] {
    const target = t.change ? name(t.change.unit) : "";
    return [
      h("div", { class: "row spread" }, h("div", { class: "label" }, `Why: ${t.change?.label ?? ""} ${target}`), button("✕", () => { trace = null; render(); }, "bv-close", "trace-close")),
      h("div", { class: "bv-trace-text mono", "data-testid": "trace-text" }, t.text),
      ...t.links.map((l, i) =>
        h(
          "div",
          { class: `bv-link ${l.side === you ? "you" : "ghost"}` },
          h("span", { class: "emoji" }, emojiOf(l.unit)),
          h("span", {}, l.name),
          h("span", { class: "dim" }, i === 0 ? viaText(l.via) : `${viaText(l.via)}, set off the one above`),
        ),
      ),
      t.links.length ? null : h("div", { class: "dim" }, "No unit acted: the rules did this."),
    ].filter((n): n is HTMLDivElement => n !== null);
  }

  function endView(): Node[] {
    const word = outcome === "win" ? "VICTORY" : outcome === "loss" ? "DEFEAT" : "DRAW";
    const why = lost ? whyPanel(battle, you, (id) => openTrace(id)) : null;
    return [h("div", { class: `bv-word ${outcome}` }, word), ...(why ? [why] : []), button("Continue", leave, "primary", "battle-done")];
  }

  show(hud, h("div", { class: "label" }, a.you ? "Them" : `@${(you === "A" ? battle.opponent : battle.player).name}`), enemy, caption, mine, h("div", { class: "label" }, a.you ? "You · front first" : `@${(you === "A" ? battle.player : battle.opponent).name} · front first`), recent, sheet, end, h("div", { class: "spacer" }), controls);
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
    h("div", { class: "label" }, "Why I lost"),
    ...(chains.length
      ? chains.map((c) => {
          const row = h(
            "button",
            { class: "bv-why" },
            h("span", { class: "bv-why-chain" }, c.text),
            h("span", { class: "mono dim" }, [c.damage ? `${c.damage} dmg` : "", c.heal ? `+${c.heal} heal` : "", c.kills ? `${c.kills} ✝` : ""].filter(Boolean).join(" · ")),
          );
          row.addEventListener("click", () => {
            if (onTrace) return onTrace(c.sampleEventId);
            const t = traceOf(battle.log, c.sampleEventId);
            overlay(h("div", { class: "bv-trace-text mono", "data-testid": "trace-text" }, t.text), h("div", { class: "dim" }, `${c.times} times this fight.`));
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

function viaText(via: string): string {
  return via === "strike" ? "strike" : via === "ability" ? "ability" : via;
}
