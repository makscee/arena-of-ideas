// My ideas (mission 2, M2-4, makscee/void-board#790). Home's "💡" line opens
// it: the ideas the player holds, "New idea", and the ideas they've sent with
// their stage; a `written` one can be taken back, refunding it. The write
// screen is one text box (IDEA_TEXT_MIN–MAX characters) and Send. Every text
// here is the player's own: the API never returns anyone else's. M2-9 adds
// the creator number: the days the player's units have been live, all together.
import { IDEA_TEXT_MAX, IDEA_TEXT_MIN, type CreditsView, type IdeaState, type MyIdea, type MyIdeasView } from "../../src/mvp/contract";
import { api, ApiError } from "../api";
import { getContent } from "../content";
import { unitSheet } from "../ui/card";
import { button, closable, h, onGone, onKeys, overlay, screen, show } from "../ui/dom";
import { pickScreen } from "./pick";

/** How My ideas names each stage. */
export const IDEA_STAGE: Record<IdeaState, string> = {
  written: "being read",
  reading: "being read",
  "pick-archetype": "Ready: pick its archetype",
  "pick-reading": "Ready: pick its reading",
  simulating: "being tested: we'll test it overnight",
  voting: "in the vote",
  live: "live",
  failed: "didn't pass",
  library: "in the library",
};

const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? "" : "s"}`;
/** Why New idea is off: "1 more run for an idea" (Home's ideas line says the same). */
export const ideaWhy = (runs: number) => `${plural(runs, "more run")} for an idea`;
const chars = (s: string) => [...s.trim()].length;
const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e));

interface IdeasNav {
  onBack: () => void;
  /** The device's player is gone (a 401): back to the name screen. */
  onUnknown: () => void;
}

/** How often My ideas looks again while an idea is with the reader. */
const READ_POLL_MS = 10_000;

/** My ideas; `notice` is what just happened (an idea sent, a pick made), said on top. */
export async function ideasScreen(nav: IdeasNav, notice: string | null = null): Promise<void> {
  const back = button("Back", nav.onBack, "primary grow", "ideas-back");
  const escBack = (e: KeyboardEvent) => (e.key === "Escape" ? (nav.onBack(), true) : false);
  let view: MyIdeasView;
  let you: CreditsView["you"] = null;
  try {
    // The creator number (M2-9) is extra: the page shows without it.
    [view, you] = await Promise.all([api.myIdeas(), api.credits().then((c) => c.you, () => null)]);
  } catch (e) {
    if (e instanceof ApiError && e.status === 401) return nav.onUnknown();
    show(h("h1", {}, "MY IDEAS"), h("div", { class: "error", "data-testid": "error" }, errorText(e)), h("div", { class: "spacer" }), h("div", { class: "row footer" }, back));
    onKeys(escBack);
    return;
  }
  const err = h("div", { class: "error", "data-testid": "error" });
  const { held, nextIn } = view.ideas;
  const holdLine =
    held > 0
      ? `💡 You hold ${plural(held, "idea")}.${nextIn === null ? " That's the most you can: send one to keep earning." : ""}`
      : "💡 You hold no ideas.";
  // M3-2: with none held, New idea is off (muted, nothing to press) and says why beside it.
  const write = button("New idea", () => writeScreen(nav), held > 0 ? "primary" : "primary off", "idea-new");
  if (held === 0) write.disabled = true;
  const newIdea = held > 0 ? write : h("div", { class: "off-row" }, write, h("span", { class: "dim small", "data-testid": "idea-new-why" }, ideaWhy(nextIn ?? 0)));
  show(
    h("h1", {}, "MY IDEAS"),
    notice ? h("div", { class: "notice", "data-testid": "idea-sent" }, notice) : null,
    h("div", { "data-testid": "ideas-held" }, holdLine),
    newIdea,
    err,
    h("div", { class: "label" }, "Sent"),
    h(
      "div",
      { class: "panel stack", "data-testid": "ideas-sent" },
      ...(view.sent.length ? view.sent.map((i) => sentRow(i, nav, err)) : [h("div", { class: "dim" }, "Nothing sent yet. Your ideas show here with how far they've got.")]),
    ),
    you ? creatorLine(you) : null,
    h("div", { class: "dim small" }, "Only you see what you write."),
    h("div", { class: "spacer" }),
    h("div", { class: "row footer" }, back),
  );
  screen("ideas");
  onKeys(escBack);
  // An idea with the reader is ready within minutes: look again while this screen is up.
  if (view.sent.some((i) => i.state === "written" || i.state === "reading")) {
    const shown = document.querySelector('[data-testid="ideas-sent"]');
    const timer = setTimeout(() => {
      if (shown?.isConnected && !document.querySelector(".overlay")) void ideasScreen(nav, notice);
    }, READ_POLL_MS);
    onGone(() => clearTimeout(timer));
  }
}

/** Your creator number: the days your units have been live, summed over every stay. */
function creatorLine(you: NonNullable<CreditsView["you"]>): HTMLElement {
  return h(
    "div",
    { class: "dim small", "data-testid": "creator-number" },
    `💡 Creator number: ${you.days}. `,
    you.units ? `The days your ${you.units === 1 ? "unit has" : `${plural(you.units, "unit")} have`} been live, all together.` : "Once an idea of yours is a unit, each day it's live adds one.",
  );
}

function sentRow(i: MyIdea, nav: IdeasNav, err: HTMLElement): HTMLElement {
  const ready = i.state === "pick-archetype" || i.state === "pick-reading";
  const action =
    i.state === "written"
      ? button("Cancel", () => cancelSheet(i, nav, err), "small", "idea-cancel")
      : ready
        ? button("Pick", () => void pickScreen(i.ideaId, { toIdeas: (n) => void ideasScreen(nav, n ?? null), onUnknown: nav.onUnknown }), "small primary", "idea-pick")
        : i.state === "live" && i.data.unitId
          ? button("Its card", () => void openUnit(i.data.unitId!, err), "small", "idea-card")
          : null;
  const named = i.data.archetype ? `${i.data.archetype.emoji} ${i.data.archetype.name} · ` : "";
  return h(
    "div",
    { class: `idea-row${ready ? " ready" : ""}`, "data-testid": "idea-sent-row", "data-state": i.state },
    h(
      "div",
      { class: "grow" },
      h("div", { class: "idea-text" }, i.text),
      h("div", { class: ready ? "small idea-ready" : "dim small", "data-testid": "idea-stage" }, `${named}${IDEA_STAGE[i.state]}`),
      i.state === "failed" && i.data.failure ? h("div", { class: "dim small", "data-testid": "idea-failure" }, `${i.data.failure} Your idea was refunded.`) : null,
    ),
    action,
  );
}

/** A live idea's unit: its sheet, from the live content. */
async function openUnit(unitId: string, err: HTMLElement): Promise<void> {
  const content = await getContent().catch(() => null);
  const unit = content?.units.find((u) => u.id === unitId);
  if (!content || !unit) return void (err.textContent = "Its card shows once the pool has it.");
  closable(unitSheet(unit, content));
}

function cancelSheet(i: MyIdea, nav: IdeasNav, err: HTMLElement): void {
  const close = overlay(
    h("div", { class: "label" }, "Cancel idea"),
    h("p", {}, "Take this idea back? Its text is deleted, and you get the idea back to write another."),
    h(
      "div",
      { class: "row sheet-actions" },
      button("Keep it", () => close(), "grow", "idea-keep"),
      button(
        "Take back",
        () => {
          close();
          void api
            .cancelIdea(i.ideaId)
            .then(() => ideasScreen(nav))
            .catch((e: unknown) => {
              if (e instanceof ApiError && e.status === 401) return nav.onUnknown();
              err.textContent = errorText(e);
            });
        },
        "danger grow",
        "idea-cancel-confirm",
      ),
    ),
  );
}

/** One text box and Send; sending spends one held idea. */
function writeScreen(nav: IdeasNav): void {
  const toList = () => void ideasScreen(nav);
  const box = h("textarea", {
    class: "idea-box",
    "data-testid": "idea-text",
    rows: "6",
    maxlength: String(IDEA_TEXT_MAX),
    "aria-label": "Your idea for a unit, in your own words",
    placeholder: "A healer who grows stronger every time an ally falls.",
  });
  const count = h("div", { class: "dim small num", "data-testid": "idea-count" });
  const err = h("div", { class: "error", "data-testid": "error" });
  let sending = false;
  const send = button(
    "Send",
    () => {
      if (sending) return;
      sending = true;
      send.disabled = true;
      err.textContent = "";
      api
        .writeIdea(box.value)
        .then(() => ideasScreen(nav, "Your idea is being read. We'll tell you when it's ready."))
        .catch((e: unknown) => {
          if (e instanceof ApiError && e.status === 401) return nav.onUnknown();
          err.textContent = errorText(e);
          sending = false;
          update();
        });
    },
    "primary grow",
    "idea-send",
  );
  const update = () => {
    const n = chars(box.value);
    count.textContent = n < IDEA_TEXT_MIN ? `${n} / ${IDEA_TEXT_MAX} · at least ${IDEA_TEXT_MIN}` : `${n} / ${IDEA_TEXT_MAX}`;
    send.disabled = sending || n < IDEA_TEXT_MIN || n > IDEA_TEXT_MAX;
  };
  box.addEventListener("input", update);
  update();
  show(
    h("h1", {}, "NEW IDEA"),
    h("label", { class: "label", for: "idea-box" }, "Your idea for a unit, in your own words"),
    box,
    count,
    h("div", { class: "dim small" }, "Only you see your text. Sending spends one idea."),
    err,
    h("div", { class: "spacer" }),
    h("div", { class: "row footer" }, button("Back", toList, "grow", "idea-write-back"), send),
  );
  box.id = "idea-box";
  screen("ideas");
  onKeys((e) => (e.key === "Escape" ? (toList(), true) : false));
  box.focus();
}
