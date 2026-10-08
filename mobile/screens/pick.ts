// The pick screens (mission 2, M2-6, makscee/void-board#794): what the
// player's idea becomes. The reader (M2-5) offers 3 archetypes (emoji, name,
// one line); the player taps one and confirms, and the idea is read again
// for 3 readings of it: unit cards like the shop's and the Codex's, each
// with its sleeping rule as text under it (M3-2) and its sheet (keywords lit, "See Awoken"), numbers "set by simulation".
// Tap one, confirm: the idea is tested overnight. "None of these" asks for
// other options once per stage; the second time the idea comes back,
// refunded. Phone first: 360×640 shows a pick without scrolling the page
// (only the sheet's own box scrolls); the desktop puts the sheet in the side
// panel, as the Codex does.
// M3-5 (makscee/void-board#807): a proposal for a new version has no
// archetype pick; its reading pick is titled "A new version of 🦔 Quillback"
// and shows the current version's rule first (a tap opens its sheet).
import type { IdeaArchetype, IdeaReading, LibraryView, MvpContent, MyIdea, UnitContent } from "../../src/mvp/contract";
import { formSegments } from "../../src/mvp/form-text";
import { mvpPool, type Row } from "../../src/mvp/units";
import { api, ApiError } from "../api";
import { getContent } from "../content";
import { card, setCardAbilities, unitSheet } from "../ui/card";
import { button, fitText, h, isDesktop, onKeys, screen, show } from "../ui/dom";
import { richText } from "../ui/term";
import { rulesLangOpt } from "../lang";

export interface PickNav {
  /** Back to My ideas; `notice` is the line it opens with. */
  toIdeas: (notice?: string) => void;
  onUnknown: () => void;
}

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e));

/** A refusal, said on the screen shown after it. */
function sayError(text: string): void {
  const el = document.querySelector<HTMLElement>('[data-testid="error"]');
  if (el) el.textContent = text;
}

/** What My ideas says after each step. */
export const PICKED_ARCHETYPE = "Reading it as that archetype. We'll tell you when its readings are ready.";
export const PICKED_READING = "We'll test it overnight.";
export const READ_AGAIN = "Reading it once more for other options.";

/** Opens the pick its idea waits for, fetched fresh (another device may have picked). */
export async function pickScreen(ideaId: string, nav: PickNav): Promise<void> {
  let idea: MyIdea;
  try {
    idea = await api.idea(ideaId);
  } catch (e) {
    if (e instanceof ApiError && e.status === 401) return nav.onUnknown();
    return nav.toIdeas();
  }
  if (idea.state === "pick-archetype" && idea.data.archetypes?.length) return archetypeScreen(idea, nav);
  if (idea.state === "pick-reading" && idea.data.readings?.length && idea.data.archetype) {
    const content = await getContent();
    // M3-5: a proposal compares with its unit as it is now, from the Library.
    const lib = idea.data.kind === "evolve" ? await api.library().catch(() => null) : null;
    const current = lib?.units.find((l) => l.unit.id === idea.data.target)?.unit;
    return readingScreen(idea, lib ? withLib(content, lib) : content, nav, current);
  }
  nav.toIdeas();
}

/** The live content with the Library's abilities, statuses and summons, so the current version's rule reads (M3-5). */
function withLib(content: MvpContent, lib: LibraryView): MvpContent {
  const sums = new Set((content.summons ?? []).map((x) => x.id));
  return {
    ...content,
    abilities: { ...lib.abilities, ...content.abilities },
    statuses: { ...lib.statuses, ...content.statuses },
    summons: [...(content.summons ?? []), ...lib.summons.filter((x) => !sums.has(x.id))],
  };
}

/** The header both picks share: the step, and the player's own text. A
 * proposal's (M3-5) is "A new version of 🦔 Quillback", with no step. */
function head(title: string, step: string, idea: MyIdea): Node[] {
  const a = idea.data.archetype;
  const top =
    idea.data.kind === "evolve" && a
      ? h("h1", { class: "h1-long", "data-testid": "pick-title" }, `A new version of ${a.emoji} ${a.name}`)
      : h("div", { class: "row spread" }, h("h1", { "data-testid": "pick-title" }, title), h("span", { class: "dim small num" }, step));
  return [top, h("div", { class: "dim small idea-quote", "data-testid": "pick-idea" }, `“${idea.text}”`)];
}

/** "The game has no words yet for …": a part of the text the reader couldn't make (M2-5). */
function cantLine(idea: MyIdea): HTMLElement | null {
  const parts = idea.data.cantExpress ?? [];
  if (!parts.length) return null;
  return h("div", { class: "dim small", "data-testid": "pick-cant" }, `The game has no words yet for ${parts.map((p) => `“${p}”`).join(", ")}, so the options leave it out.`);
}

/** "None of these": once per stage it reads again; the second time it gives the idea back. */
function noneButton(idea: MyIdea, stage: "archetypes" | "readings", nav: PickNav, err: HTMLElement): HTMLButtonElement {
  const last = !!idea.data.declined?.[stage];
  const label = last ? "None of these: take my idea back" : "None of these: read it again";
  const b = button(
    label,
    () => {
      b.disabled = true;
      api
        .declineOptions(idea.ideaId)
        .then(() => nav.toIdeas(last ? "Your idea is back: you can write another." : READ_AGAIN))
        .catch((e: unknown) => {
          if (e instanceof ApiError && e.status === 401) return nav.onUnknown();
          err.textContent = errorText(e);
          b.disabled = false;
        });
    },
    "small link pick-none",
    "pick-none",
  );
  return b;
}

/** Confirm, after a tap; a refusal (the name or shape was just taken) reopens the pick as it now is. */
function confirmButton(send: () => Promise<unknown>, done: string, idea: MyIdea, nav: PickNav, err: HTMLElement): HTMLButtonElement {
  const b = button(
    "Confirm",
    () => {
      b.disabled = true;
      err.textContent = "";
      send()
        .then(() => nav.toIdeas(done))
        .catch((e: unknown) => {
          if (e instanceof ApiError && e.status === 401) return nav.onUnknown();
          if (e instanceof ApiError && e.status === 409) return void pickScreen(idea.ideaId, nav).then(() => sayError(errorText(e)));
          err.textContent = errorText(e);
          b.disabled = false;
        });
    },
    "primary grow",
    "pick-confirm",
  );
  b.disabled = true;
  return b;
}

// ---------- the archetype ----------

function archetypeRow(a: IdeaArchetype, i: number, onTap: () => void): HTMLButtonElement {
  const row = h(
    "button",
    { class: "pick-arch", "data-testid": "pick-archetype", "data-index": String(i), "aria-pressed": "false" },
    h("span", { class: "pick-emoji" }, a.emoji),
    h("span", { class: "pick-words" }, h("span", { class: "pick-name" }, a.name), h("span", { class: "dim small" }, a.line)),
  );
  row.addEventListener("click", onTap);
  return row;
}

function archetypeScreen(idea: MyIdea, nav: PickNav): void {
  const options = idea.data.archetypes!;
  const err = h("div", { class: "error", "data-testid": "error" });
  let picked = -1;
  const confirm = confirmButton(() => api.pickArchetype(idea.ideaId, picked), PICKED_ARCHETYPE, idea, nav, err);
  const rows = options.map((a, i) =>
    archetypeRow(a, i, () => {
      picked = i;
      rows.forEach((r, k) => r.setAttribute("aria-pressed", String(k === i)));
      confirm.disabled = false;
    }),
  );
  show(
    ...head("ARCHETYPE", "1 / 2", idea),
    h("div", { class: "dim small" }, "What is your unit? Pick one."),
    h("div", { class: "stack pick-list", "data-testid": "pick-archetypes" }, ...rows),
    cantLine(idea),
    noneButton(idea, "archetypes", nav, err),
    err,
    h("div", { class: "spacer" }),
    h("div", { class: "row footer" }, button("Back", () => nav.toIdeas(), "grow", "pick-back"), confirm),
  );
  screen("ideas");
  onKeys((e) => (e.key === "Escape" ? (nav.toIdeas(), true) : false));
}

// ---------- the reading ----------

/** A reading as a unit of the content: its Row with no numbers yet, and the
 * abilities and summons it needs (a reading may use a Does the live pool doesn't). */
function readingPool(a: IdeaArchetype, r: IdeaReading, i: number) {
  const row: Row = { name: a.name, emoji: a.emoji, archetype: a.line, tier: 1, pwr: 0, hp: 0, when: r.when, who: r.who, does: r.does, awoken: r.awoken };
  const pool = mvpPool([row]);
  return { ...pool.units[0]!, id: `reading-${i}`, pool };
}

function readingScreen(idea: MyIdea, content0: MvpContent, nav: PickNav, current?: UnitContent): void {
  const a = idea.data.archetype!;
  const readings = idea.data.readings!;
  const units = readings.map((r, i) => readingPool(a, r, i));
  const content: MvpContent = {
    ...content0,
    units: [...content0.units, ...units.map(({ pool: _, ...u }) => u)],
    abilities: Object.assign({}, content0.abilities, ...units.map((u) => u.pool.abilities)),
    summons: [...(content0.summons ?? []), ...units.flatMap((u) => u.pool.summons).filter((s) => !content0.summons?.some((x) => x.id === s.id))],
  };
  // The cards draw their icon line from the card abilities: add the readings' own.
  setCardAbilities(content.abilities);
  const err = h("div", { class: "error", "data-testid": "error" });
  let picked = -1;
  const confirm = confirmButton(() => api.pickReading(idea.ideaId, picked), PICKED_READING, idea, nav, err);
  const desk = isDesktop();
  const detail = h(desk ? "aside" : "div", { class: desk ? "codex-insp stack" : "panel pick-detail", "data-testid": "pick-detail" }, h("div", { class: "dim" }, current ? "Tap a reading, or Now, to read it." : "Tap a reading to read it."));
  const cards = units.map((u, i) => {
    const c = card({ ...u, stats: { pwr: 0, hp: 0 }, unitId: u.id, form: "sleeping", recipe: u.forms.sleeping }, { side: "you", testid: "pick-reading", unset: true, onOpen: () => tap(i) });
    c.dataset.index = String(i);
    return c;
  });
  // M3-2: each card's sleeping rule as text under it (the sheet's words), so
  // the three compare at a glance; a tap off its keywords picks it too.
  const cols = cards.map((c, i) => {
    const rule = h("div", { class: "pick-rule", "data-testid": "pick-rule" }, ...richText(formSegments(units[i]!.forms.sleeping, content.abilities, rulesLangOpt()), { size: 13 }));
    rule.addEventListener("click", (e) => {
      if (!(e.target as HTMLElement).closest("button")) tap(i);
    });
    return h("div", { class: "pick-col" }, c, rule);
  });
  // M3-5: the version it would replace, first, for comparison: its rule; a tap reads its sheet (it isn't an option).
  const now = current
    ? h("button", { class: "propose-now", "data-testid": "pick-current" }, h("span", { class: "dim small" }, "Now: "), ...richText(formSegments(current.forms.sleeping, content.abilities, rulesLangOpt()), { size: 13 }))
    : null;
  now?.addEventListener("click", (e) => {
    if ((e.target as HTMLElement).closest("button") !== now) return; // a keyword's own tip
    cards.forEach((c) => c.classList.remove("inspected"));
    now.classList.add("inspected");
    detail.replaceChildren(unitSheet(current!, { ...content, units: [...content.units, current!] }));
  });
  const tap = (i: number) => {
    picked = i;
    now?.classList.remove("inspected");
    cards.forEach((c, k) => c.classList.toggle("inspected", k === i));
    cols.forEach((c, k) => c.classList.toggle("picked", k === i));
    const { pool: _, ...u } = units[i]!;
    detail.replaceChildren(unitSheet(u, content, { candidate: true }));
    confirm.disabled = false;
  };
  const main = h(
    "div",
    { class: desk ? "codex-main stack" : "pick-main" },
    ...head("READING", "2 / 2", idea),
    idea.data.kind === "evolve"
      ? h("div", { class: "dim small", "data-testid": "pick-archline" }, a.line)
      : h("div", { class: "pick-archline", "data-testid": "pick-archline" }, `${a.emoji} ${a.name}`, h("span", { class: "dim small" }, ` · ${a.line}`)),
    now,
    h("div", { class: "pick-cards", "data-testid": "pick-readings" }, ...cols),
    cantLine(idea),
    desk ? null : detail,
    noneButton(idea, "readings", nav, err),
    err,
    // On a phone the sheet takes the room left (and scrolls in itself); on desktop the spacer does.
    desk ? h("div", { class: "spacer" }) : null,
    h("div", { class: "row footer pick-foot" }, button("Back", () => nav.toIdeas(), "grow", "pick-back"), confirm),
  );
  show(main, desk ? detail : null);
  screen(desk ? "codex" : "ideas");
  fitText();
  onKeys((e) => (e.key === "Escape" ? (nav.toIdeas(), true) : false));
}
