// The either/or vote card (mission 2, M2-8, makscee/void-board#793): "Which
// would you rather see in the game?", a candidate unit and a typical live one
// in the server's random order. One tap picks, Skip skips, and the next card
// takes its place. Quiet: nothing shows when there's no card. Home and the
// run-over screen each show one. M3-6: a new version of a Library unit is
// tagged "new version of Quillback", its Library version "Quillback, unchanged".
import type { MvpContent, UnitContent, VoteCard } from "../../src/mvp/contract";
import { api } from "../api";
import { addCardAbilities, card, formText } from "../ui/card";
import { sigilIcon } from "../ui/glyph";
import { faceOfUnit, piece } from "../ui/piece";
import { t } from "../i18n";
import { actionable, button, h } from "../ui/dom";
import { orient } from "../ui/stage";
import { localUnit } from "../unit-names";

/** A box that fills itself with the player's next vote card, if any. `compact`: Home's tray (arena#31): the
 * question, Skip and the two candidates as pieces with their names, no rule text (a pick's title has it). */
export function votePanel(content: MvpContent, compact = false): HTMLElement {
  const box = h("div", { class: "vote-box", "data-testid": "vote-box" });
  let sending = false;
  const render = (c: VoteCard | null, voted: boolean) => {
    if (!c) return box.replaceChildren(...(voted ? [h("div", { class: compact ? "hm-vote hm-thanks" : "dim small", "data-testid": "vote-thanks" }, t("vote.thanks"))] : []));
    addCardAbilities(c.pool.abilities);
    const known = new Set((content.summons ?? []).map((s) => s.id));
    const pool: MvpContent = {
      ...content,
      abilities: { ...content.abilities, ...c.pool.abilities },
      statuses: { ...content.statuses, ...c.pool.statuses },
      summons: [...(content.summons ?? []), ...c.pool.summons.filter((s) => !known.has(s.id))],
    };
    const send = async (pick: string | null) => {
      if (sending) return;
      sending = true;
      try {
        render((await api.vote({ candidateId: c.candidateId, otherId: c.otherId, pick })).card, true);
      } catch {
        box.replaceChildren();
      } finally {
        sending = false;
      }
    };
    // Both sides get the line, so the two cards stay level.
    const tag = (u: UnitContent) =>
      c.candidateKind === "idea"
        ? []
        : u.id !== c.candidateId
          ? [h("div", { class: "small vote-tag" }, "\u00a0")]
          : [h("div", { class: "small vote-tag", "data-testid": "vote-tag" }, c.candidateKind === "version" ? t("vote.newVersionOf", { name: u.name }) : t("vote.unchanged", { name: u.name }), " ", sigilIcon(u.portrait, 20))];
    const choice = (u: UnitContent) => {
      const el = h(
        "button",
        { class: "vote-pick", "data-testid": "vote-pick", "data-unit": u.id, "aria-label": `${u.name}` },
        ...tag(u),
        card({ emoji: u.emoji, name: u.name, stats: u.base, recipe: u.forms.sleeping, unitId: u.id, ...(u.portrait ? { portrait: u.portrait } : {}) }, { side: "you", tier: u.tier }),
        h("div", { class: "small vote-text" }, formText(u.forms.sleeping, pool)),
        h("div", { class: "dim small vote-text" }, t("vote.awoken", { text: formText(u.forms.awoken, pool) })),
      );
      el.addEventListener("click", () => void send(u.id));
      return actionable(el, u.name);
    };
    if (compact) {
      // The landscape stage is drawn at 0.625 on 800×450: its text starts at 18 (11 there), the phone's is plain.
      const wide = orient() === "L";
      const pick = (u: UnitContent) => {
        const el = h(
          "button",
          { class: "hm-pick", "data-testid": "vote-pick", "data-unit": u.id, "aria-label": u.name, title: `${u.name}: ${formText(u.forms.sleeping, pool)}` },
          piece({ name: u.name, emoji: u.emoji, stats: u.base, face: faceOfUnit(u), tier: u.tier, copies: 1, ...(u.portrait ? { portrait: u.portrait } : {}) }, { size: "M", side: "shop", px: wide ? 60 : 44, floor: wide ? 18 : 0 }),
        );
        el.addEventListener("click", () => void send(u.id));
        return el;
      };
      return box.replaceChildren(
        h(
          "div",
          { class: "hm-vote", "data-testid": "vote-card" },
          h("div", { class: "hm-vote-head" }, h("div", { class: "hm-vote-q" }, t("vote.questionShort")), button(t("vote.skip"), () => void send(null), "hm-skip keep", "vote-skip")),
          h("div", { class: "hm-vote-pair" }, pick(localUnit(c.units[0])), pick(localUnit(c.units[1]))),
        ),
      );
    }
    box.replaceChildren(
      h(
        "div",
        { class: "panel stack vote", "data-testid": "vote-card" },
        h("div", { class: "row spread" }, h("div", { class: "label" }, t("vote.question")), button(t("vote.skip"), () => void send(null), "small link keep", "vote-skip")),
        h("div", { class: "vote-pair" }, choice(localUnit(c.units[0])), choice(localUnit(c.units[1]))),
      ),
    );
  };
  api.nextVote().then((r) => render(r.card, false), () => box.replaceChildren());
  return box;
}
