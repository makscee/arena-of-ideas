// The first screen (arena#31, mission #26 move 3): "One rule starts a chain."
// over a three-step chain that plays itself (Gambler strikes → Drummer counts
// it → the line behind powers up), and PLAY, which skips the name form: the
// caller makes a generated name and opens round 1. A quiet name field stays
// under PLAY for a player who wants their own.
import type { MvpContent, UnitContent } from "../../src/mvp/contract";
import { t } from "../i18n";
import { faceOfUnit, piece } from "../ui/piece";
import { button, h, onGone, show, withKey } from "../ui/dom";

/** The three pieces of the chain, by name; any units of the pool stand in when one has left it. */
const CHAIN = ["Gambler", "Drummer", "Hornet"] as const;
const STEP_MS = 1500;

export interface StartOptions {
  content: MvpContent | null;
  /** PLAY: make a generated name, open round 1. */
  onPlay: () => void;
  /** A typed name: the name form's path. */
  onName: (name: string) => void;
  /** Shown under the buttons. */
  extra?: (HTMLElement | null)[];
  err: HTMLElement;
}

const reduced = (): boolean => matchMedia("(prefers-reduced-motion: reduce)").matches;

function chainUnits(content: MvpContent | null): UnitContent[] {
  const pool = content?.units ?? [];
  const named = CHAIN.map((n) => pool.find((u) => u.name === n));
  return named.every(Boolean) ? (named as UnitContent[]) : pool.slice(0, 3);
}

/** The chain that plays itself: step 1 the first piece strikes, 2 the second counts it, 3 the line behind powers up. */
function chain(units: UnitContent[]): { root: HTMLElement; stop: () => void } {
  const steps = [t("start.step1", { name: units[0]?.name ?? "" }), t("start.step2", { name: units[1]?.name ?? "" }), t("start.step3")];
  const pieces = units.slice(0, 3).map((u, i) =>
    h("div", { class: "start-piece", "data-k": `piece-${i}` }, piece({ name: u.name, emoji: u.emoji, stats: u.base, face: faceOfUnit(u), tier: u.tier, copies: 1 }, { size: "L", side: i === 0 ? "you" : "shop", testid: `start-piece-${i}` })),
  );
  const wires = [1, 2].map((n) => h("div", { class: "start-wire", "data-k": `wire-${n}` }, h("span", { class: "start-wire-n" }, String(n))));
  const caption = h("div", { class: "start-caption", "data-testid": "start-caption", "aria-live": "polite" });
  const row = h("div", { class: "start-chain", "data-testid": "start-chain" }, pieces[0] ?? "", wires[0]!, pieces[1] ?? "", wires[1]!, pieces[2] ?? "");
  const root = h("div", { class: "start-demo" }, row, caption);
  const light = (step: number) => {
    root.dataset.step = String(step);
    pieces.forEach((p, i) => p.classList.toggle("lit", i + 1 <= step));
    wires.forEach((w, i) => w.classList.toggle("lit", i + 2 <= step));
    caption.textContent = steps.slice(0, step).join(" → ");
  };
  // Reduced motion: the whole chain, still.
  if (reduced() || units.length < 3) {
    light(3);
    return { root, stop: () => {} };
  }
  let step = 0;
  light(1);
  const timer = setInterval(() => light((step = (step + 1) % 4 || 1)), STEP_MS);
  return { root, stop: () => clearInterval(timer) };
}

export function startScreen(o: StartOptions): void {
  const demo = chain(chainUnits(o.content));
  const input = h("input", { placeholder: t("start.ownName"), maxlength: "24", autocomplete: "nickname", "aria-label": t("start.ownName"), "data-testid": "name-input" });
  const play = withKey(button(t("start.play"), () => o.onPlay(), "primary play-start", "start-play"), "Enter");
  const named = button(t("name.enter"), () => o.onName(input.value.trim()), "small", "name-submit");
  // Enter in the name field plays under that name; anywhere else it is PLAY.
  input.addEventListener("keydown", (e) => {
    if (e.key !== "Enter") return;
    e.stopPropagation();
    e.preventDefault();
    if (input.value.trim()) o.onName(input.value.trim());
    else o.onPlay();
  });
  const onKey = (e: KeyboardEvent) => {
    if (e.key === "Enter" && !(e.target instanceof HTMLInputElement) && !(e.target instanceof HTMLButtonElement)) o.onPlay();
  };
  addEventListener("keydown", onKey);
  show(
    h(
      "div",
      { class: "start", "data-testid": "start" },
      h("h1", { class: "start-brand" }, t("name.title")),
      h("h2", { class: "start-title", "data-testid": "start-title" }, t("start.title")),
      h("p", { class: "start-lead dim" }, t("start.lead")),
      demo.root,
      play,
      h("div", { class: "row start-name" }, input, named),
      ...(o.extra ?? []),
      o.err,
    ),
  );
  onGone(() => (demo.stop(), removeEventListener("keydown", onKey)));
  play.focus();
}
