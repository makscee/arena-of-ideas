// My ideas (mission 2, M2-4, makscee/void-board#790). Home's "💡" line opens
// it: the ideas the player holds, "New idea", and the ideas they've sent with
// their stage; a `written` one can be taken back, refunding it. The write
// screen is one text box (IDEA_TEXT_MIN–MAX characters) and Send. Every text
// here is the player's own: the API never returns anyone else's. M2-9 adds
// the creator number: the days the player's units have been live, all together.
// M3-5 (makscee/void-board#807): a Library unit's sheet proposes a new version
// of it (proposeScreen: its line and current rule over the text box), and My
// ideas names a proposal "new version of 🦔 Quillback".
import { IDEA_TEXT_MAX, IDEA_TEXT_MIN, type CreditsView, type IdeaState, type MvpContent, type MyIdea, type MyIdeasView, type UnitContent } from "../../src/mvp/contract";
import { formSegments } from "../../src/mvp/form-text";
import { api, ApiError } from "../api";
import { getContent } from "../content";
import { unitSheet } from "../ui/card";
import { button, closable, h, onGone, onKeys, overlay, screen, show } from "../ui/dom";
import { richText } from "../ui/term";
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

export interface IdeasNav {
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
    await nameTargets(view.sent);
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

/** A proposal's unit, "🦔 Quillback", by its target's id (M3-5): the
 * reading stages carry it as the archetype; before that the Library has it. */
const targetNames = new Map<string, string>();
async function nameTargets(sent: MyIdea[]): Promise<void> {
  if (!sent.some((i) => i.data.kind === "evolve" && !i.data.archetype && i.data.target && !targetNames.has(i.data.target))) return;
  const lib = await api.library().catch(() => null);
  for (const l of lib?.units ?? []) targetNames.set(l.unit.id, `${l.unit.emoji} ${l.unit.name}`);
}

/** What My ideas calls an idea before its stage: "🦔 Quillback · ", or for a
 * proposal (M3-5) "new version of 🦔 Quillback · ". */
export function ideaNamed(i: MyIdea): string {
  const a = i.data.archetype;
  if (i.data.kind !== "evolve") return a ? `${a.emoji} ${a.name} · ` : "";
  const unit = a ? `${a.emoji} ${a.name}` : (i.data.target && targetNames.get(i.data.target)) || "a Library unit";
  return `new version of ${unit} · `;
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
  const named = ideaNamed(i);
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
  textScreen({
    title: "NEW IDEA",
    above: [],
    label: "Your idea for a unit, in your own words",
    placeholder: "A healer who grows stronger every time an ally falls.",
    send: (text) => api.writeIdea(text),
    sent: "Your idea is being read. We'll tell you when it's ready.",
    onBack: () => void ideasScreen(nav),
    nav,
  });
}

/** M3-5: "Propose a new version" of a Library unit: the unit's line and its
 * current rule over the text box. Back is `onBack` (the Codex); sending
 * spends one held idea and opens My ideas. `content` has the unit's abilities. */
export function proposeScreen(unit: UnitContent, content: MvpContent, nav: IdeasNav): void {
  textScreen({
    title: `A new version of ${unit.emoji} ${unit.name}`,
    long: true,
    above: [
      h("div", { class: "dim small", "data-testid": "propose-line" }, unit.archetype),
      h(
        "div",
        { class: "panel propose-now", "data-testid": "propose-rule" },
        h("span", { class: "dim small" }, "Now: "),
        ...richText(formSegments(unit.forms.sleeping, content.abilities), { size: 14 }),
      ),
    ],
    label: "What should change?",
    placeholder: "Make it hit every enemy instead of only the front one.",
    send: (text) => api.proposeVersion(unit.id, text),
    sent: `Your new version of ${unit.emoji} ${unit.name} is being read. We'll tell you when it's ready.`,
    onBack: nav.onBack,
    nav,
  });
}

interface TextScreen {
  title: string;
  /** A title too long for the big h1 (a unit's name in it). */
  long?: boolean;
  above: Node[];
  label: string;
  placeholder: string;
  send: (text: string) => Promise<unknown>;
  /** What My ideas says once it's sent. */
  sent: string;
  onBack: () => void;
  nav: IdeasNav;
}

function textScreen(o: TextScreen): void {
  const { nav } = o;
  const box = h("textarea", {
    class: "idea-box",
    "data-testid": "idea-text",
    rows: "6",
    maxlength: String(IDEA_TEXT_MAX),
    "aria-label": o.label,
    placeholder: o.placeholder,
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
      o.send(box.value)
        .then(() => ideasScreen(nav, o.sent))
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
    h("h1", { class: o.long ? "h1-long" : "", "data-testid": "write-title" }, o.title),
    ...o.above,
    h("label", { class: "label", for: "idea-box" }, o.label),
    box,
    count,
    h("div", { class: "dim small" }, "Only you see your text. Sending spends one idea."),
    err,
    h("div", { class: "spacer" }),
    h("div", { class: "row footer" }, button("Back", o.onBack, "grow", "idea-write-back"), send),
  );
  box.id = "idea-box";
  screen("ideas");
  onKeys((e) => (e.key === "Escape" ? (o.onBack(), true) : false));
  box.focus();
}
