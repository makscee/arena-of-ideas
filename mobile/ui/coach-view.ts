// The coach's card (arena#31): one step of the first run's pair, in plain
// words, with the pair drawn as two small pieces before it is placed. The
// pure choice of the step is coach.ts; this only draws it.
import type { MvpContent, UnitContent } from "../../src/mvp/contract";
import { t } from "../i18n";
import { creditOf } from "./card";
import { h } from "./dom";
import { faceOfUnit, piece } from "./piece";
import type { CoachStep, CoachState, CoachUnit } from "./coach";

const byName = (content: MvpContent, name: string): UnitContent | undefined => content.units.find((u) => u.name === name);

/** What the coach reads of a pool unit: its name and its face's when word. */
export function coachUnit(u: UnitContent): CoachUnit {
  return { id: u.id, name: u.name, when: faceOfUnit(u).when.label };
}

/** The coach's view of a run: the line, the shop and the gold as they are. */
export function coachState(run: { line: { unitId: string }[]; offers: { slot: number; unitId: string; cost: number }[]; gold: number }, lineSize: number, content: MvpContent): CoachState {
  const units = new Map(content.units.map((u) => [u.id, coachUnit(u)]));
  return {
    line: run.line.map((u) => u.unitId),
    offers: run.offers.map((o) => ({ slot: o.slot, unitId: o.unitId, cost: o.cost })),
    gold: run.gold,
    room: lineSize - run.line.length,
    unit: (id) => units.get(id),
  };
}

const STEP_OF = { "buy-first": 1, "buy-second": 2, fight: 3 } as const;

/** The card for `step`. */
export function coachView(step: CoachStep, content: MvpContent): HTMLElement {
  const a = byName(content, step.anchor);
  const r = byName(content, step.answer);
  const p = (u: UnitContent | undefined, side: "you" | "shop") =>
    u ? piece({ name: u.name, emoji: u.emoji, stats: u.base, face: faceOfUnit(u), tier: u.tier, copies: 1 }, { size: "XS", side }) : null;
  const params = { anchor: step.anchor, answer: step.answer };
  const title = step.step === "buy-first" ? t("coach.buyFirst", params) : step.step === "buy-second" ? t("coach.buySecond", params) : t("coach.fight");
  const why = step.step === "buy-first" ? t("coach.buyFirstWhy", params) : step.step === "buy-second" ? t("coach.buySecondWhy", params) : t("coach.fightWhy", params);
  // Medic's author, credited once the pair is built (a seed unit has none).
  const by = step.step === "fight" && r ? creditOf(r.id)?.by : undefined;
  const credit = step.step === "fight" ? (by ? t("coach.credit", { answer: step.answer, name: by.name }) : t("coach.seed", { answer: step.answer })) : "";
  return h(
    "div",
    { class: "coach", "data-testid": "coach", "data-step": step.step },
    h("div", { class: "coach-step" }, t("coach.step", { n: STEP_OF[step.step], total: 3 })),
    h("div", { class: "coach-body" },
      h("div", { class: "coach-pair", "aria-label": t("coach.chain", params) }, p(a, "you"), h("span", { class: "coach-arrow", "aria-hidden": "true" }, "→"), p(r, "shop")),
      h("div", { class: "coach-text" }, h("span", { class: "coach-chain", "aria-hidden": "true" }, t("coach.chain", params), " · "), h("b", {}, title), " ", why, credit ? h("div", { class: "coach-by dim" }, credit) : null),
    ),
  );
}
