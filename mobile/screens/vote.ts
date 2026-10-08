// The either/or vote card (mission 2, M2-8, makscee/void-board#793): "Which
// would you rather see in the game?", a candidate unit and a typical live one
// in the server's random order. One tap picks, Skip skips, and the next card
// takes its place. Quiet: nothing shows when there's no card. Home and the
// run-over screen each show one.
import type { MvpContent, UnitContent, VoteCard } from "../../src/mvp/contract";
import { api } from "../api";
import { addCardAbilities, card, formText } from "../ui/card";
import { button, h } from "../ui/dom";

/** A box that fills itself with the player's next vote card, if any. */
export function votePanel(content: MvpContent): HTMLElement {
  const box = h("div", { class: "vote-box", "data-testid": "vote-box" });
  let sending = false;
  const render = (c: VoteCard | null, voted: boolean) => {
    if (!c) return box.replaceChildren(...(voted ? [h("div", { class: "dim small", "data-testid": "vote-thanks" }, "Thanks for voting.")] : []));
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
    const choice = (u: UnitContent) => {
      const el = h(
        "button",
        { class: "vote-pick", "data-testid": "vote-pick", "data-unit": u.id, "aria-label": `${u.name}` },
        card({ emoji: u.emoji, name: u.name, stats: u.base, recipe: u.forms.sleeping, unitId: u.id }, { side: "you", tier: u.tier }),
        h("div", { class: "small vote-text" }, formText(u.forms.sleeping, pool)),
        h("div", { class: "dim small vote-text" }, `Awoken: ${formText(u.forms.awoken, pool)}`),
      );
      el.addEventListener("click", () => void send(u.id));
      return el;
    };
    box.replaceChildren(
      h(
        "div",
        { class: "panel stack vote", "data-testid": "vote-card" },
        h("div", { class: "row spread" }, h("div", { class: "label keep" }, "Which would you rather see in the game?"), button("Skip", () => void send(null), "small link", "vote-skip")),
        h("div", { class: "vote-pair" }, choice(c.units[0]), choice(c.units[1])),
      ),
    );
  };
  api.nextVote().then((r) => render(r.card, false), () => box.replaceChildren());
  return box;
}
