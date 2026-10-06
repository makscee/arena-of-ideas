// Arena MVP phone client: name → home → shop → fight → battle → result.
// Plain DOM; each screen is a function that renders into #app.
// Owners: slice 8 the name, home, shop and result screens here; slice 9 the
// battle viewer (screens/battle.ts); slice 11 the stats page
// (screens/stats.ts). Shared: api.ts, content.ts, ui/.
// Home is the title menu (R2-10); it also shows DayView.lastPlayoff (a game
// opens in battleScreen) and, on dev servers only (HomeView.dev), "End day
// now" under "Dev". The shop's ☰ (Esc on desktop) is the in-run menu.
import type { BattleRecord, DayView, FightResult, HomeView, LineUnit, MvpContent, MvpRules, Offer, PlayerRef, PlayoffResult, RunView } from "../src/mvp/contract";
import { MVP_RULES, offersAt, sellValue } from "../src/mvp/contract";
import { mergeTarget } from "../src/mvp/forms";
import { buttonRefusal, plainRefusal } from "./ui/refusal";
import { ApiError, api, savedPlayer } from "./api";
import { getContent } from "./content";
import { battleScreen, type RunOutro } from "./screens/battle";
import { codexScreen, newCodexCache, type CodexState } from "./screens/codex";
import { setCodexLink } from "./ui/term";
import { statsScreen } from "./screens/stats";
import { card, roman, unitSheet, type CardUnit } from "./ui/card";
import { previewName } from "./ui/fusion";
import { icon } from "./ui/icon";
import { app, button, closable, desktopQuery, h, isDesktop, keepScreen, onKeys, overlay, screen, show, who } from "./ui/dom";
import { loadUnitRates } from "./ui/unit-stats";
import { initSound, onSoundChange, play, setSound, soundSettings } from "./ui/sound";
import { shopSound } from "./ui/sound-map";

function errorLine(): HTMLElement {
  return h("div", { class: "error", "data-testid": "error" });
}

/** True while a guarded request is out: taps are ignored until it answers,
 * so a double tap can't send the same decision twice. */
let busy = false;

async function guarded(err: HTMLElement, fn: () => Promise<void>): Promise<void> {
  if (busy) return;
  busy = true;
  app.classList.add("busy");
  try {
    err.textContent = "";
    await fn();
  } catch (e) {
    if (e instanceof ApiError && e.status === 401 && e.message.startsWith("unknown player")) {
      api.forget();
      return nameScreen();
    }
    if (e instanceof ApiError && e.status === 409) play("wrong");
    err.textContent = e instanceof ApiError && e.status === 409 ? plainRefusal(e.message) : e instanceof Error ? e.message : String(e);
  } finally {
    busy = false;
    app.classList.remove("busy");
  }
}

/** The rules as Home last saw them; the shop and result read the numbers here. */
let rules: MvpRules = MVP_RULES;
/** Today's champion for the shop's pin; Home refreshes it, the shop fetches it when missing. */
let day: DayView | null = null;

const plural = (n: number, one: string, many = `${one}s`) => `${n} ${n === 1 ? one : many}`;
const offersText = (r: MvpRules) => {
  const grow = r.offersGrowAt ?? [];
  if (grow.length === 0) return `${plural(r.offers, "offer")} a shop`;
  const last = Math.max(...grow);
  return `${plural(r.offers, "offer")} in round 1, growing to ${offersAt(r, last)} by round ${last}`;
};
// "tier II opens in round 3, III in 6, IV in 9", from the rules.
const tiersText = (r: MvpRules) =>
  r.tierOpensAt
    .map((round, i) => ({ tier: i + 1, round }))
    .filter((t) => t.round > 1)
    .map((t, i) => (i === 0 ? `tier ${roman(t.tier)} opens in round ${t.round}` : `${roman(t.tier)} in ${t.round}`))
    .join(", ") || "every tier is open from round 1";
const roundLabel = (round: number) => (round > rules.rounds ? "CROWN" : `R${round}/${rules.rounds}`);
const hearts = (n: number) => h("span", { class: "hearts", "aria-label": plural(n, "heart") }, "♥".repeat(n) + "♡".repeat(Math.max(0, rules.hearts - n)));
const openSheet = (u: Parameters<typeof unitSheet>[0], content: MvpContent) => () => closable(unitSheet(u, content));

/** A line of cards, each opening its unit sheet. */
function team(line: LineUnit[], side: "you" | "ghost", content: MvpContent, testid = ""): HTMLElement {
  return h("div", { class: "slots", ...(testid ? { "data-testid": testid } : {}) }, ...line.map((u) => card(u, { side, extra: [copiesBadge(u)], onOpen: openSheet(u, content) })));
}

/** The Sound row of the title menu and the run menu (round 3, note 16): a
 * toggle and a volume slider; M flips the same setting on desktop. Moving
 * the slider plays a click at the new level. */
function soundRow(): HTMLElement {
  const toggle = button("", () => setSound({ on: !soundSettings().on }), "sound-toggle", "sound-toggle");
  const volume = h("input", { type: "range", min: "0", max: "100", step: "5", "aria-label": "Volume", "data-testid": "sound-volume" });
  const sync = (s = soundSettings()) => {
    toggle.textContent = s.on ? "🔊 Sound on" : "🔇 Sound off";
    toggle.setAttribute("aria-pressed", String(s.on));
    volume.value = String(Math.round(s.volume * 100));
    volume.disabled = !s.on;
  };
  volume.addEventListener("input", () => setSound({ volume: Number(volume.value) / 100 }));
  volume.addEventListener("change", () => play("click"));
  const row = h("div", { class: "row sound-row", "data-testid": "sound-row" }, toggle, volume);
  sync();
  // A row set aside (the Codex over Home) still follows M; one thrown away stops listening.
  const ref = new WeakRef(row);
  const off = onSoundChange((s) => (ref.deref() ? sync(s) : off()));
  return row;
}

/** A hint's verb: "Tap" on the phone, "Click" on a desktop. */
const tapOrClick = () => (isDesktop() ? "Click" : "Tap");

/** One contextual hint: a line of text, shown where it applies. */
function hint(text: string): HTMLElement {
  return h("div", { class: "hint", "data-testid": "hint" }, text);
}

/** The rules, readable any time (Home's Rules button). */
function rulesSheet(): HTMLElement {
  const r = rules;
  const p = (t: string) => h("p", {}, t);
  return h(
    "div",
    { class: "stack rules", "data-testid": "rules" },
    h("h2", {}, "RULES"),
    h("div", { class: "label" }, "A run"),
    p(`${r.rounds} shop rounds, then the Crown: a fight against today's champion. You start with ${plural(r.hearts, "heart")}; a lost fight costs one, and at 0 the run ends before the Crown.`),
    p(`${r.goldPerRound} gold every round, no carry-over. A unit costs ${r.unitCost}, a reroll ${r.rerollCost}, selling gives back ${r.sellRefund}${r.sellRefundAwoken && r.sellRefundAwoken !== r.sellRefund ? `, ${r.sellRefundAwoken} for an Awoken or fused unit` : ""}. ${offersText(r)}; ${tiersText(r)}.`),
    h("div", { class: "label" }, "The line"),
    p(`${r.lineSize} units in a line, front first. Change the order in the shop: ${isDesktop() ? "drag a unit, or click it, then ← →" : "tap a unit, then ◀ ▶"}. Each round you fight a team another player saved at the same round.`),
    h("div", { class: "label" }, "Copies, Awoken, fusion"),
    p(`Buying a unit you own merges it in: +${r.copyGrowth.pwr} PWR / +${r.copyGrowth.hp} HP a copy. Copy ${r.copiesToAwaken} awakens it: the same When, a stronger Who or Does.`),
    p(`Two Awoken units fuse: the When of the first you ${isDesktop() ? "pick" : "tap"}, the Who of the second, the Does of both, and the stronger PWR and HP of the two, +1 PWR / +2 HP. A fused unit is final; copies of either part still merge into it. The first player to make a pair names it.`),
    h("div", { class: "label" }, "Chains"),
    p(`Units react to events. When one happens, the units it triggers fire in line order, front to back, each at most once per event. In a fight, ${isDesktop() ? "click" : "tap"} any number to see the chain that caused it.`),
    h("div", { class: "label" }, "The day"),
    p(`Beat the champion in the Crown and you are a slayer. At ${r.dayEndsAt} Moscow the slayers' best teams play a round-robin, and the winner is the next champion.`),
    p("Your rating moves once per run: every fight, the Crown too, counts against its opponent's rating (Elo), added up when the run ends. Giving up counts each heart left as a lost fight."),
    h("div", { class: "dim small", "data-testid": "icon-credits" }, "Icons: Lorc, Delapouite, Sbed, Skoll from game-icons.net, CC BY 3.0; heart-plus by Zeromancer, CC0."),
  );
}

/** How to read a card (the shop's "?"): each number and mark, with a sample. */
function legendSheet(): HTMLElement {
  const r = rules;
  const row = (sample: Node, text: string) => h("div", { class: "legend-row" }, h("div", { class: "legend-sample" }, sample), h("div", {}, text));
  const span = (cls: string, t: string) => h("span", { class: cls }, t);
  const rulesBtn = button("Rules", () => (close(), closable(rulesSheet())), "grow", "legend-rules");
  const sheet = h(
    "div",
    { class: "stack legend", "data-testid": "legend" },
    h("h2", {}, "READING A CARD"),
    row(
      h("span", { class: "legend-icons" }, h("span", { class: "tone-when" }, icon("flying-flag", 16)), h("span", { class: "tone-enemy" }, icon("targeted", 16)), h("span", { class: "tone-dmg" }, icon("spiky-explosion", 16))),
      "Top: what it does in icons: when, who, what (here: at battle start, the front enemy, damage). A dot on the first says whose event: teal an ally's, pink an enemy's. Its sheet says it in words.",
    ),
    row(h("span", { class: "stats" }, span("p", "2"), "/", span("h", "6")), "PWR / HP. PWR is what its strike deals; at 0 HP it falls."),
    row(span("copies", "●●○"), `Copies toward Awoken: copy ${r.copiesToAwaken} awakens it. Each copy adds +${r.copyGrowth.pwr} PWR / +${r.copyGrowth.hp} HP.`),
    row(span("copies tag", "AWOKEN ×3"), "Awoken, its stronger form; ×3 copies merged in. Two Awoken units can fuse."),
    row(span("copies tag", "FUSED ×2"), "Two Awoken units fused into one: final, copies of either part still merge in."),
    row(span("cost", "3g ＋"), "An offer's price. ＋: you own it, so buying merges a copy in. The numeral top right (I–IV) is its tier."),
    h("div", { class: "dim small" }, "Tap any card for its sheet: what it does now, and its Awoken form one tap away."),
  );
  const close = closable(sheet, h("div", { class: "row" }, rulesBtn));
  return sheet;
}

// ---------- name ----------

function nameScreen(): void {
  // An invite-only server (slice 13) takes no new names: a player comes from
  // their invite link.
  void api.health().then((hl) => {
    if (hl.invites) show(h("h1", {}, "ARENA OF IDEAS"), h("p", { class: "dim", "data-testid": "invite-only" }, "Arena is invite-only for now. Open your invite link on this device to play."));
  }, () => {});
  const input = h("input", { placeholder: "Your name", maxlength: "24", autocomplete: "nickname", "data-testid": "name-input" });
  const err = errorLine();
  const go = () => guarded(err, async () => {
    await api.register(input.value.trim());
    await homeScreen();
  });
  input.addEventListener("keydown", (e) => e.key === "Enter" && void go());
  show(
    h("h1", {}, "ARENA OF IDEAS"),
    h("p", { class: "dim" }, "An auto-battler of chains. Pick a name; it stays on this device."),
    input,
    button("Enter", () => void go(), "primary", "name-submit"),
    err,
  );
  input.focus();
}

// ---------- home ----------

/** Home is the title menu (R2-10, docs/round2/ui.md (e)): the champion,
 * your name and rating, one big Continue run (or Play), then New run (only
 * while a run waits: it asks to abandon that run first), Codex, Stats and
 * Rules. Records live in Stats; "End day now" shows on dev servers only.
 * `ended`: the day the dev "End day now" just closed, so Home says how it
 * ended at the top and brings the playoff panel into view. */
async function homeScreen(ended: number | null = null): Promise<void> {
  const err = errorLine();
  void loadUnitRates(); // the rates hint on unit sheets (slice 11)
  const [home, content]: [HomeView, MvpContent] = await Promise.all([api.home(), getContent()]);
  rules = home.rules;
  day = home.day;
  // The run that waits, for Continue's round and hearts (null if it ended meanwhile).
  const waiting = home.activeRunId ? await api.run(home.activeRunId).catch(() => null) : null;
  const active = waiting && waiting.phase !== "over" ? waiting : null;
  const champ = home.day.champion;
  const r = home.rating;
  const play = active
    ? button("", () => void guarded(err, async () => shopScreen(await api.run(active.runId), content)), "primary", "play")
    : button("Play", () => void guarded(err, async () => shopScreen(await api.startRun(), content)), "primary", "play");
  if (active) play.replaceChildren(`Continue run · ${active.round > rules.rounds ? "Crown" : `R${active.round}`} `, hearts(active.hearts));
  const newRun = active
    ? button("New run", () => abandonSheet(active, "new", () => void guarded(err, async () => {
        // The given-up run's end and rating change first, as ☰ Abandon shows
        // them; its "New run" starts the next one.
        runOverScreen(await api.abandon(active.runId), content, "", true);
      })), "", "new-run")
    : null;
  const codex = button("Codex", () => void guarded(err, () => openCodex()), "", "codex");
  const stats = button("Stats", () => void statsScreen({ content, onBack: () => void homeScreen(), onCodex: () => void openCodex() }), "grow", "stats");
  const rulesBtn = button("Rules", () => closable(rulesSheet()), "grow", "rules-open");
  const endDay = button(
    "End day now",
    () =>
      void guarded(err, async () => {
        try {
          await api.endDay();
        } catch (e) {
          if (e instanceof ApiError && (e.status === 404 || e.status === 501)) throw new Error(e.status === 404 ? "End day is a dev tool (MVP_DEV=1)." : "The day arrives in slice 5.");
          throw e;
        }
        await homeScreen(home.day.seq);
      }),
    "small",
    "end-day",
  );
  const last = home.day.lastPlayoff ?? null;
  const justEnded = ended !== null && last?.seq === ended ? last : null;
  const playoff = playoffPanel(last, champ?.player ?? null, content, err);
  show(
    h(
      "div",
      { class: "row spread" },
      h("h1", {}, "ARENA"),
      h("div", { class: "row me" }, who(api.player?.name ?? "", "dim"), h("span", { class: "dim keep" }, "·"), h("span", { class: "num keep", "data-testid": "rating" }, `${r?.rating ?? rules.ratingStart}`)),
    ),
    // The dev "End day now" just ran: say so first, with how the day ended
    // (the playoff panel) right under it, before today's champion.
    // Desktop: the champion and the day on the left, the menu on the right
    // (.home-main, .home-side; on the phone they are one column).
    h(
      "div",
      { class: "home-main" },
      ended !== null ? h("div", { class: "notice", "data-testid": "day-ended" }, `Day ${ended} ended just now. Today is day ${home.day.seq}.`) : null,
      justEnded ? playoff : null,
      h(
        "div",
        { class: "panel stack champion", "data-testid": "champion" },
        h("div", { class: "row spread" }, h("div", { class: "label keep" }, `👑 Champion · day ${home.day.seq}`), champ ? whoMark(champ.player, "ghost-name") : null),
        champ ? team(champ.line, "ghost", content) : h("div", { class: "dim" }, "No champion yet. The day arrives soon."),
        h("div", { class: "dim small", "data-testid": "slayers" }, champ ? slayersLine(home.day.slayers) : `New champion at ${rules.dayEndsAt} Moscow.`),
      ),
      champ
        ? hint(
            champ.player.id === api.player?.id
              ? `This is your team. Others try to beat it today, and so can you, with another team. ${tapOrClick()} a card to read it.`
              : r
                ? `${tapOrClick()} a card to read it. Beat this team in the Crown to become a slayer.`
                : `This is the team to beat. ${tapOrClick()} a card to read it, then Play.`,
          )
        : null,
      justEnded ? null : playoff,
    ),
    h(
      "div",
      { class: "home-side" },
      err,
      h("div", { class: "spacer" }),
      // The menu stays on the first screen however long Home runs (a playoff's table).
      h(
        "div",
        { class: "stack footer title-menu", "data-testid": "home-actions" },
        play,
        newRun,
        codex,
        h("div", { class: "row" }, stats, rulesBtn),
        soundRow(),
        home.dev ? h("details", { class: "dev" }, h("summary", {}, "Dev"), endDay) : null,
      ),
    ),
  );
  screen("home");
  if (justEnded) playoff?.classList.add("fresh");
}

/** One confirm before a run is given up (R2-2's abandon): it says what the
 * rating takes. `why` "menu" is ☰ Abandon run; "new" is New run on the title
 * menu, which ends the waiting run first. */
function abandonSheet(run: RunView, why: "menu" | "new", onConfirm: () => void): void {
  const crown = run.round > rules.rounds;
  const cost = crown
    ? "It counts as a lost Crown, and your rating moves for it."
    : run.hearts === 1
      ? "The 1 heart left counts as a lost fight, and your rating moves for it."
      : `Every heart left counts as a lost fight: ${run.hearts} losses, and your rating moves for them.`;
  const close = overlay(
    h("div", { class: "label" }, why === "new" ? `A run waits · ${crown ? "the Crown" : `round ${run.round} of ${rules.rounds}`}` : "Abandon run"),
    h("p", { "data-testid": "abandon-text" }, `${why === "new" ? "Abandon this run? You'll see how it went, then start a new one. " : "End this run now? "}${cost}`),
    h(
      "div",
      { class: "row sheet-actions" },
      button("Cancel", () => close(), "grow", "abandon-cancel"),
      button("Abandon", () => (close(), onConfirm()), "danger grow", "abandon-confirm"),
    ),
  );
}

/** Where the Codex's Back goes: the screen it opened over, kept as it was. */
let codexBack: (() => void) | null = null;

/** Opens the Codex over the current screen (the title menu, a run's shop, a
 * battle); from inside the Codex (a term link in a unit sheet) it changes tab
 * and keeps the same Back. */
let codexCache = newCodexCache();
/** Redraws the open Codex (its inspector comes and goes at 1024px). */
let codexRedraw: (() => void) | null = null;
async function openCodex(state?: Partial<CodexState>): Promise<void> {
  if (app.dataset.screen !== "codex" || !codexBack) {
    // A battle under it pauses (keepScreen), and its fetches start fresh.
    const keep = keepScreen();
    const desk = isDesktop();
    codexCache = newCodexCache();
    // The shop's desktop layout is more than CSS: crossed 1024px meanwhile, Back redraws it.
    codexBack = () => {
      keep();
      if (isDesktop() !== desk && app.dataset.screen === "shop") rerender?.();
    };
  }
  // A deep link (a term's "Open in Codex") is a fresh Codex: no tab keeps the
  // scroll it had, whatever was open before (R2-17); fetched data stays.
  if (state) codexCache.scroll = {};
  const back = codexBack;
  const content = await getContent();
  const onBack = () => ((codexBack = null), (codexRedraw = null), back());
  codexRedraw = () => void codexScreen({ content, state: { ...codexCache.state, term: undefined, scope: undefined }, onBack, cache: codexCache });
  await codexScreen({ content, state, onBack, cache: codexCache });
}
// Every highlighted term's "Open in Codex" lands on its Keywords row (a scoped trigger's line in it).
setCodexLink((term, scope) => void openCodex({ tab: "keywords", term, scope }));

/** The in-run menu (☰ in the HUD, Esc on desktop): Resume, Codex, Rules,
 * Title menu (the run waits on the server; Continue brings it back) and
 * Abandon run with one confirm, which ends on the run-over screen. `fought`:
 * on the result screen, the fight just shown, whose round the label names
 * (the run itself has moved on to the next one). */
function runMenu(run: RunView, content: MvpContent, err: HTMLElement, fought?: FightResult, resume?: () => void): void {
  if (app.querySelector('[data-testid="run-menu"]')) return;
  const round = fought ? fought.round : run.round;
  const crown = fought ? fought.kind === "crown" : round > rules.rounds;
  const codex = button("Codex", () => (close(), void guarded(err, () => openCodex())), "", "menu-codex");
  const menu = h(
    "div",
    { class: "stack run-menu", "data-testid": "run-menu" },
    h("div", { class: "label" }, run.phase === "over" ? "Run · over" : crown ? "Run · the Crown" : `Run · round ${round} of ${rules.rounds}`),
    // Over a battle that was playing, Resume plays on (R2-17 batch F).
    button("Resume", () => (close(), resume?.()), "primary", "menu-resume"),
    codex,
    button("Rules", () => (close(), closable(rulesSheet())), "", "menu-rules"),
    soundRow(),
    button("Title menu", () => (close(), void guarded(err, () => homeScreen())), "", "menu-title"),
    run.phase === "over" ? null : h("div", { class: "dim small" }, "The run waits; Continue brings you back."),
    run.phase === "over" ? null : button("Abandon run…", () => (close(), abandonSheet(run, "menu", () => void guarded(err, async () => runOverScreen(await api.abandon(run.runId), content)))), "danger", "menu-abandon"),
  );
  const close = overlay(menu);
  // A tap outside the menu (or the battle's Esc) is a Resume too.
  const back = menu.closest(".overlay");
  if (resume && back) back.addEventListener("click", (e) => e.target === back && resume());
}

/** The champion card's last line: what today's slayers mean at the day's end. */
function slayersLine(n: number): string {
  const at = `${rules.dayEndsAt} Moscow`;
  if (n === 0) return `No slayers yet. If nobody slays it by ${at}, this team stays champion.`;
  if (n === 1) return `1 slayer today. At ${at} the slayer's team takes the crown, unless others slay it too.`;
  return `${n} slayers today. At ${at} their teams play a round-robin for the crown.`;
}

/** "@name", with 🤖 after a bot's. */
function whoMark(p: PlayerRef, cls = ""): HTMLElement {
  const el = who(p.name, cls);
  return p.bot ? h("span", { class: "who-mark" }, el, h("span", { class: "bot", "aria-label": "bot" }, "🤖")) : el;
}

/** How a day ended, in one sentence. With no playoff to show (no slayers,
 * or one who won without a game) the sentence is all there is; `champion` is
 * today's, the one who stayed or was crowned. Slayers may be bots (🤖). */
function playoffSummary(p: PlayoffResult, champion: PlayerRef | null): (Node | string)[] {
  if (p.entrants.length === 0) return champion ? ["No slayers, so ", whoMark(champion), " stays champion."] : ["No slayers, and no champion yet."];
  if (p.entrants.length === 1) {
    const only = p.winner ?? p.entrants[0]!;
    return [whoMark(only), " was the only slayer, so ", only.bot ? "its" : "their", " team is the new champion."];
  }
  const bots = p.entrants.filter((x) => x.bot).length;
  const field = `${p.entrants.length} slayers${bots === p.entrants.length ? ", all bots" : bots ? `, ${bots} of them ${bots === 1 ? "a bot" : "bots"}` : ""}`;
  return p.winner ? ["👑 ", whoMark(p.winner), ` won the playoff (${field}) and is the new champion.`] : [`The playoff (${field}) had no winner.`];
}

/** A day's end: a sentence, and with a real playoff (two or more slayers)
 * its table, compact, and its games behind a tap (each opens in the viewer). */
function playoffPanel(p: PlayoffResult | null, champion: PlayerRef | null, content: MvpContent, err: HTMLElement): HTMLElement | null {
  if (!p) return null;
  const watch = (battleId: string, a: PlayerRef, b: PlayerRef) => {
    const btn = button("", () => void guarded(err, async () => {
      const battle = await api.battle(battleId);
      battleScreen({ battle, content, onDone: () => void homeScreen() });
    }), "small game", "playoff-game");
    btn.replaceChildren(whoMark(a), " v ", whoMark(b));
    return btn;
  };
  const played = p.entrants.length >= 2;
  const games = h("div", { class: "games", "data-testid": "playoff-games" }, ...p.games.map((g) => watch(g.battleId, g.a, g.b)));
  games.hidden = true;
  const toggle = button(`Watch the games (${p.games.length})`, () => {
    games.hidden = !games.hidden;
    toggle.textContent = games.hidden ? `Watch the games (${p.games.length})` : "Hide the games";
  }, "small", "playoff-games-open");
  return h(
    "div",
    { class: "panel stack playoff", "data-testid": "playoff" },
    h("div", { class: "label" }, played ? `Playoff · day ${p.seq}` : `Day ${p.seq} ended`),
    h("div", { "data-testid": "playoff-summary" }, ...playoffSummary(p, champion)),
    played
      ? h(
          "div",
          { class: "standings" },
          ...p.standings.map((s, i) => h("div", { class: "standing", "data-testid": "playoff-standing" }, h("span", { class: "dim" }, `${i + 1}`), whoMark(s.player), h("span", { class: "num" }, `${s.wins}W ${s.draws}D ${s.losses}L`))),
        )
      : null,
    played && p.games.length ? toggle : null,
    played && p.games.length ? games : null,
  );
}

// ---------- shop ----------

/** What the shop is in the middle of: nothing, a unit picked, or a fuse waiting for its second unit. */
type Pick = { mode: "none" } | { mode: "picked"; index: number } | { mode: "fuse"; first: number };

/** The shop. `selected`: the line slot to keep selected on desktop (a unit
 * just moved by key or drag stays in hand). At 1024px and wider (ui/dom.ts
 * isDesktop) the same shop is a top bar, a wide board and a right inspector
 * that reads the hovered or selected card instead of pop-up sheets, with
 * mouse and keys; below it the phone layout is as it was. */
function shopScreen(run: RunView, content: MvpContent, notice = "", selected = -1): void {
  if (run.phase === "over") return runOverScreen(run, content, notice);
  const desk = isDesktop();
  const crown = run.phase === "crown";
  // The reigning champion's Crown is their own team; a win is still a slay.
  const ownCrown = crown && run.nextOpponent?.player.id === run.player.id;
  const err = errorLine();
  err.textContent = notice;
  let pick: Pick = desk && run.line[selected] ? { mode: "picked", index: selected } : { mode: "none" };
  const unitOf = (id: string) => content.units.find((x) => x.id === id);
  const decide = (d: Parameters<typeof api.decide>[1], select = -1) =>
    guarded(err, async () => {
      const res = await api.decide(run.runId, d);
      if (res.fight) return fightScreens(res.run, res.fight, content);
      play(shopSound(d, run, res.run));
      shopScreen(res.run, content, "", select);
    });

  const line = h("div", { class: "slots", "data-testid": "line" });
  const actions = h("div", { class: "row actions", "data-testid": "actions" });
  const hintSlot = h("div", {});
  const awoken = run.line.filter((u) => u.kind === "unit" && u.form === "awoken").length;
  // Desktop: the inspector, the card under the mouse, an offer clicked, a fusion waiting for its Fuse.
  const inspector = h("aside", { class: "inspector stack", "data-testid": "inspector" });
  type At = { kind: "line"; index: number } | { kind: "offer"; slot: number };
  let hover: At | null = null;
  let chosen: number | null = null;
  let fuseView: HTMLElement | null = null;

  const renderLine = () => {
    line.replaceChildren(
      ...Array.from({ length: rules.lineSize }, (_, i) => {
        const u: LineUnit | undefined = run.line[i];
        if (!u) return h("div", { class: "card empty" }, h("div", { class: "dim" }, `${i + 1}`));
        const c = card(u, { side: "you", extra: [copiesBadge(u)], testid: `line-${i}` });
        if (pick.mode === "picked" && pick.index === i) c.classList.add("selected");
        if (pick.mode === "fuse") {
          if (pick.first === i) c.classList.add("selected");
          else if (u.kind === "unit" && u.form === "awoken") c.classList.add("fusable");
          else c.classList.add("muted");
        }
        c.addEventListener("click", () => {
          play("click");
          if (pick.mode === "fuse") {
            if (pick.first === i) pick = { mode: "none" };
            else if (u.kind === "unit" && u.form === "awoken") return void fusePreview(pick.first, i);
            else return;
          } else pick = pick.mode === "picked" && pick.index === i ? { mode: "none" } : { mode: "picked", index: i };
          chosen = null;
          renderLine();
        });
        if (desk) desktopCard(c, { kind: "line", index: i });
        return c;
      }),
    );
    for (const o of offers.children) o.classList.toggle("selected", chosen !== null && (o as HTMLElement).dataset.testid === `offer-${chosen}`);
    if (desk) renderInspector(true);
    else actions.replaceChildren(...actionButtons());
    hintSlot.replaceChildren(...[shopHint()].filter((x): x is HTMLElement => x !== null));
  };

  /** Desktop: hovering a card reads it in the inspector; a line card drags onto another to reorder. */
  const desktopCard = (c: HTMLElement, at: At) => {
    c.addEventListener("mouseenter", () => ((hover = at), renderInspector()));
    c.addEventListener("mouseleave", () => ((hover = null), renderInspector()));
    if (at.kind !== "line" || crown) return;
    const i = at.index;
    c.draggable = true;
    c.addEventListener("dragstart", (e) => {
      e.dataTransfer?.setData("text/plain", `line:${i}`);
      c.classList.add("dragging");
    });
    c.addEventListener("dragend", () => c.classList.remove("dragging"));
    c.addEventListener("dragover", (e) => (e.preventDefault(), c.classList.add("drop")));
    c.addEventListener("dragleave", () => c.classList.remove("drop"));
    c.addEventListener("drop", (e) => {
      e.preventDefault();
      c.classList.remove("drop");
      const from = Number(/^line:(\d+)$/.exec(e.dataTransfer?.getData("text/plain") ?? "")?.[1] ?? NaN);
      if (Number.isInteger(from) && from !== i) void decide({ kind: "reorder", from, to: i }, i);
    });
  };

  const actionButtons = (): HTMLElement[] => {
    if (pick.mode === "fuse") return [h("span", { class: "dim grow" }, desk ? "Click the second Awoken unit." : "Tap the second Awoken unit."), button(desk ? "Cancel · Esc" : "Cancel", () => ((pick = { mode: "none" }), renderLine()), "", "fuse-cancel")];
    if (pick.mode !== "picked") return [];
    const i = pick.index;
    const u = run.line[i]!;
    const info = button("Info", openSheet(u, content), "", "info");
    if (crown) return desk ? [] : [info];
    const left = button(desk ? "◀ ←" : "◀", () => void decide({ kind: "reorder", from: i, to: i - 1 }, i - 1), "", "move-left");
    const right = button(desk ? "→ ▶" : "▶", () => void decide({ kind: "reorder", from: i, to: i + 1 }, i + 1), "", "move-right");
    left.disabled = i === 0;
    right.disabled = i >= run.line.length - 1;
    // On desktop the inspector already is the sheet: no Info.
    const out: HTMLElement[] = desk ? [left, right] : [left, right, info];
    if (u.kind === "unit" && u.form === "awoken" && awoken >= 2) out.push(button(desk ? "Fuse · F" : "Fuse", () => ((pick = { mode: "fuse", first: i }), renderLine()), "", "fuse"));
    const value = sellValue(rules, u);
    out.push(button(desk ? `Sell +${value}g · S` : `Sell +${value}`, () => void decide({ kind: "sell", index: i }), "danger", "sell"));
    return out;
  };

  /** Desktop: a unit's win and pick rates are a subtle hint (docs/round2/
   * README.md): the inspector's last line, at its foot, not mid-panel. */
  const ratesToFoot = () => {
    const r = inspector.querySelector('[data-testid="unit-rates"]');
    if (r) inspector.append(r);
  };
  /** Desktop: the hovered card, else the selected one (or a fusion waiting
   * for its Fuse), else how to use the board. `force` redraws even when the
   * same card is shown (the selection changed). */
  let inspected = "";
  const renderInspector = (force = false) => {
    if (fuseView) return inspector.replaceChildren(fuseView);
    const at: At | null = hover ?? (pick.mode === "picked" ? { kind: "line", index: pick.index } : chosen !== null ? { kind: "offer", slot: chosen } : null);
    const key = at ? `${at.kind}:${at.kind === "line" ? at.index : at.slot}` : "none";
    if (key === inspected && !force) return;
    inspected = key;
    const sel = pick.mode === "picked" ? pick.index : -1;
    if (at?.kind === "line" && run.line[at.index]) {
      const mine = at.index === sel;
      inspector.replaceChildren(
        h("div", { class: "label" }, `${mine ? "Selected · " : ""}In your line, slot ${at.index + 1}`),
        unitSheet(run.line[at.index]!, content),
        mine || pick.mode === "fuse" ? h("div", { class: "row actions", "data-testid": "actions" }, ...actionButtons()) : h("div", { class: "dim small" }, crown ? "Your line is final for the Crown." : "Click to select it: move, fuse or sell."),
      );
      return ratesToFoot();
    }
    if (at?.kind === "offer") {
      const n = run.offers.findIndex((o) => o.slot === at.slot);
      const o = run.offers[n];
      if (!o) return;
      const head = h("div", { class: "label" }, `Shop · offer ${n + 1} · key ${n + 1}`);
      inspector.replaceChildren(head, h("div", { class: "dim small" }, "…"));
      void offerBody(o).then(
        ({ sheet, blocked }) => {
          if (inspected !== key) return;
          const buy = button(buttonRefusal(blocked) || `Buy ${o.cost}g · ${n + 1}`, () => void decide({ kind: "buy", slot: o.slot }), "primary grow", "buy");
          buy.disabled = blocked !== "";
          inspector.replaceChildren(head, sheet, h("div", { class: "row" }, buy));
          ratesToFoot();
        },
        (e: unknown) => {
          if (inspected === key) inspector.replaceChildren(head, h("div", { class: "error" }, e instanceof Error ? e.message : String(e)));
        },
      );
      return;
    }
    inspector.replaceChildren(
      ...(pick.mode === "fuse" ? [h("div", { class: "row actions", "data-testid": "actions" }, ...actionButtons())] : []),
      h("div", { class: "dim" }, crown ? "Hover a card to read it here." : "Hover a card to read it here. Click a unit in your line to select it; double-click an offer to buy it."),
    );
  };

  /** The hint that matters most right now, or none. */
  const shopHint = (): HTMLElement | null => {
    if (pick.mode !== "none") return null;
    // run.ts refuses every decision but the fight in the crown phase: the line is final.
    if (crown) return hint(ownCrown ? "The Crown: today's champion is your own team. Beat it to be a slayer again; a loss costs a heart. Your line is final." : "The Crown: your line, as it is, against today's champion. Win it to become a slayer; a loss costs a heart.");
    // Only a unit whose next copy is on offer right now.
    const almost = run.line.find((u) => u.kind === "unit" && u.form === "sleeping" && u.copies === rules.copiesToAwaken - 1 && run.offers.some((o) => o.unitId === u.unitId));
    const canBuy = run.offers.some((o) => o.cost <= run.gold);
    if (awoken >= 2) return hint(desk ? "Two Awoken units can fuse: select one, then F." : "Two Awoken units can fuse: tap one, then Fuse.");
    if (almost && run.offers.some((o) => o.unitId === almost.unitId && o.cost <= run.gold)) return hint(`One more ${almost.name} awakens it. It's in the shop.`);
    if (run.line.length === 0 && !canBuy) return hint("No gold for a unit. Fight to move on: an empty line loses, and costs a heart.");
    if (run.line.length === 0) return hint(desk ? "Double-click an offer, or press its number, to buy it. Your line fights front first." : "Tap an offer to read it and buy it. Your line fights front first.");
    if (run.round === 1 && run.line.length > 0 && run.gold < rules.unitCost) return hint("Out of gold for units. Fight when ready.");
    if (run.line.length > 1 && run.round <= 2) return hint(desk ? "Drag a unit to move it, or click it: ← → move it, S sells it." : "Tap a unit in your line to move, sell or read it.");
    if (almost) return hint(`One more ${almost.name} awakens it. It's in the shop for ${run.offers.find((o) => o.unitId === almost.unitId)!.cost}g.`);
    return null;
  };

  /** Fuse preview: the result card from a dry run, Swap to try the other
   * order, then confirm. Both orders are fetched up front (a preview writes
   * nothing), so Swap is instant. A pair nobody has fused comes without a
   * name; the fuse reveals it. On desktop it fills the inspector (Esc
   * cancels) instead of a pop-up sheet. */
  const fusePreview = (first: number, second: number) =>
    guarded(err, async () => {
      const orders = [
        { first, second },
        { first: second, second: first },
      ] as const;
      const [ab, ba] = await Promise.all(orders.map((o) => api.preview(run.runId, { kind: "fuse", ...o })));
      // The fused unit keeps the first part's uid, in the front-most slot.
      const fusedOf = (res: { run: RunView }, o: { first: number }) => res.run.line.find((u) => u.uid === run.line[o.first]!.uid) ?? res.run.line[Math.min(first, second)]!;
      const views = [fusedOf(ab!, orders[0]), fusedOf(ba!, orders[1])];
      let at = 0;
      const body = h("div", { class: "stack" });
      const render = () => {
        const o = orders[at]!;
        const fused = previewName(views[at]!);
        const shown = previewName(views[at]!, "???");
        const recipe = h(
          "div",
          { class: "recipe-line", "data-testid": "fusion-recipe" },
          h("span", { class: "k" }, "When"), ` · ${run.line[o.first]!.name} → `,
          h("span", { class: "k" }, "Who"), ` · ${run.line[o.second]!.name} → `,
          h("span", { class: "k" }, "Does"), " · both",
        );
        body.replaceChildren(
          h("div", { class: "preview-card" }, card(shown, { side: "you", extra: [copiesBadge(shown)] })),
          recipe,
          h(
            "div",
            { class: "row sheet-actions" },
            button("Cancel", () => close(), "grow", "preview-cancel"),
            button("⇄ Swap", () => ((at = 1 - at), render()), "", "preview-swap"),
            button("Fuse", () => (close(), void fuse(o, views[at]!)), "primary grow", "preview-confirm"),
          ),
          unitSheet(fused, content, { preview: true }),
        );
      };
      render();
      const head = h("div", { class: "label" }, "Fusion preview");
      pick = { mode: "none" };
      let close: () => void;
      if (desk) {
        fuseView = h("div", { class: "stack", "data-testid": "fuse-preview" }, head, body);
        close = () => ((fuseView = null), renderLine());
      } else close = overlay(head, body);
      renderLine();
    });

  /** Confirm a fuse; a pair nobody had fused (or only bots had) then shows
   * its name with "You discovered". */
  const fuse = (o: { first: number; second: number }, previewed: LineUnit) =>
    guarded(err, async () => {
      const res = await api.decide(run.runId, { kind: "fuse", ...o });
      play(shopSound({ kind: "fuse", ...o }, run, res.run));
      shopScreen(res.run, content);
      const fused = res.run.line.find((u) => u.uid === run.line[o.first]!.uid);
      const me = savedPlayer()?.id;
      const discovered = previewed.fusion?.name === "" || previewed.fusion?.discoveredBy === null;
      if (!fused || !discovered || fused.fusion?.discoveredBy?.id !== me) return;
      play("discover");
      closable(
        h("div", { class: "label" }, "New fusion"),
        h("h2", { class: "reveal", "data-testid": "fusion-reveal" }, `✨ You discovered ${fused.name}`),
        h("div", { class: "preview-card" }, card(fused, { side: "you", extra: [copiesBadge(fused)] })),
        unitSheet(fused, content),
      );
    });

  /** Why an offer can't be bought now, decided here so no request goes out
   * (a full line used to cost a 409 on every preview); "" when it can. */
  const buyBlock = (o: Offer): string => {
    if (run.gold < o.cost) return `Needs ${o.cost}g`;
    if (run.line.length >= rules.lineSize && mergeTarget(run.line, o.unitId) < 0) return "Line full: sell or fuse first";
    return "";
  };
  /** Desktop's double-click and number keys: a blocked offer says why instead of asking the server. */
  const buy = (o: Offer) => {
    const why = buyBlock(o);
    if (why) return void ((err.textContent = why), play("wrong"));
    void decide({ kind: "buy", slot: o.slot });
  };

  /** An offer's sheet: its form, and what buying it does to your line;
   * `blocked` says why it can't be bought now ("" when it can). The buy's dry
   * run is asked once per offer a shop (hovering on desktop asks again and
   * again); the sheet is built fresh each time. */
  type OfferBody = { sheet: HTMLElement; blocked: string };
  const previews = new Map<number, ReturnType<typeof buyPreview>>();
  const buyPreview = (o: Offer) => api.preview(run.runId, { kind: "buy", slot: o.slot }).catch((e: unknown) => (e instanceof ApiError && e.status === 409 ? e : Promise.reject(e)));
  const offerBody = async (o: Offer): Promise<OfferBody> => {
    const u = unitOf(o.unitId);
    let blocked = buyBlock(o);
    const affordable = blocked === "";
    let ask = previews.get(o.slot);
    if (affordable && !ask) {
      ask = buyPreview(o);
      previews.set(o.slot, ask);
      ask.catch(() => previews.delete(o.slot)); // a failed dry run is asked again next time
    }
    const res = affordable ? await ask! : null;
    let after: HTMLElement | null = null;
    // A unit you own: the sheet shows your copy, from now to after buying.
    let mine: { now: LineUnit; next: LineUnit } | null = null;
    if (res instanceof ApiError) blocked = plainRefusal(res.message);
    else if (res) {
      const before = new Map(run.line.map((x) => [x.uid, x]));
      const changed = res.run.line.find((x) => !before.has(x.uid) || before.get(x.uid)!.copies !== x.copies);
      if (changed) {
        const was = before.get(changed.uid);
        const label = !was ? "Joins your line" : was.form !== changed.form ? "Awakens!" : `Merges in: ×${changed.copies}`;
        if (was) mine = { now: was, next: changed };
        after = h("div", { class: `stack after${was && was.form !== changed.form ? " awakens" : ""}`, "data-testid": "buy-preview" }, h("div", { class: "label" }, label), h("div", { class: "preview-card" }, card(changed, { side: "you", extra: [copiesBadge(changed)] })));
      }
    }
    // Without a preview (no gold, a full line), an owned unit still shows your copy as it is.
    const owned = mine ? null : run.line.find((x) => x.kind === "unit" && x.unitId === o.unitId) ?? null;
    const sheet = mine ? unitSheet(mine.next, content, { from: mine.now.stats }) : owned ? unitSheet(owned, content) : u ? unitSheet(u, content) : h("h2", {}, o.unitId);
    // What buying does goes inside the sheet, above its last line (the rates hint).
    if (after) sheet.insertBefore(after, sheet.querySelector('[data-testid="unit-rates"]'));
    return { sheet, blocked };
  };
  /** The phone's offer sheet: the body in an overlay, with Close and Buy. */
  const offerSheet = (o: Offer) =>
    guarded(err, async () => {
      const { sheet, blocked } = await offerBody(o);
      const buy = button(buttonRefusal(blocked) || `Buy ${o.cost}g`, () => (close(), void decide({ kind: "buy", slot: o.slot })), "primary grow", "buy");
      buy.disabled = blocked !== "";
      const close = overlay(sheet, h("div", { class: "row sheet-actions" }, button("Close", () => close(), "", "offer-close"), buy));
    });

  const offers = h(
    "div",
    { class: "slots", "data-testid": "offers" },
    ...run.offers.map((o: Offer) => {
      const u = unitOf(o.unitId);
      const cu: CardUnit = { unitId: o.unitId, emoji: u?.emoji ?? "?", name: u?.name ?? o.unitId, stats: u?.base ?? { pwr: 0, hp: 0 }, ...(u ? { recipe: u.forms.sleeping } : {}) };
      const owned = mergeTarget(run.line, o.unitId) >= 0;
      const c = card(cu, { side: "you", tier: o.tier, extra: [h("div", { class: "cost" }, owned ? `${o.cost}g ＋` : `${o.cost}g`)], testid: `offer-${o.slot}` });
      if (run.gold < o.cost) c.classList.add("poor");
      if (owned) c.classList.add("owned");
      if (!desk) c.addEventListener("click", () => (play("click"), void offerSheet(o)));
      else {
        // Click selects it (the inspector holds it), double-click buys it.
        c.addEventListener("click", () => {
          play("click");
          chosen = chosen === o.slot ? null : o.slot;
          if (pick.mode === "picked") pick = { mode: "none" };
          renderLine();
        });
        c.addEventListener("dblclick", () => buy(o));
        desktopCard(c, { kind: "offer", slot: o.slot });
      }
      return c;
    }),
  );

  const reroll = button(`Reroll ${rules.rerollCost}g`, () => void decide({ kind: "reroll" }), "", "reroll");
  reroll.disabled = run.gold < rules.rerollCost;
  const fight = button(crown ? "Fight the champion" : "Fight", () => void decide({ kind: "fight" }), "primary grow", "fight");
  // An empty line can fight (and lose a heart) once nothing is affordable, so a broke run moves on.
  fight.disabled = run.line.length === 0 && run.offers.some((o) => o.cost <= run.gold);
  if (desk) {
    reroll.append(" ", kbd("R"));
    fight.append(" ", kbd("Space"));
  }

  const opp = run.nextOpponent;
  const pin = h("div", { class: "pin", "data-testid": "champion-pin" });
  // The Crown has no shop: where the offers were, the team you are about to
  // face, as plainly as your own line (R2-17 batch E: it was only the pin).
  const foe = crown ? h("div", { class: "stack crown-foe", "data-testid": "crown-foe" }) : null;
  /** The Crown's foe, named by the run's own opponent (what "Crown vs" says
   * and the fight uses); the day gives its line only when its champion is
   * that player, so a day that turned over since never shows another team
   * (R2-17 batch F). */
  const fillFoe = (d: DayView | null, settled = d !== null) => {
    if (!foe || !opp) return;
    const ch = d?.champion;
    const line = ch && ch.player.id === opp.player.id ? ch.line : null;
    foe.replaceChildren(
      h("div", { class: "label" }, ownCrown ? "You face · your champion team · " : "You face · today's champion · ", who(opp.player.name, "ghost-name")),
      line ? team(line, "ghost", content, "crown-foe-line") : h("div", { class: "dim small" }, settled ? "Their line shows in the fight." : "…"),
    );
  };
  const fillPin = (d: DayView | null) => {
    const ch = d?.champion;
    if (!ch) return pin.replaceChildren(h("span", { class: "dim" }, "👑 No champion yet"));
    const b = h("button", { class: "pin-btn", "data-testid": "champion-pin-open" }, h("span", {}, "👑"), who(ch.player.name, "ghost-name"), h("span", { class: "pin-emoji" }, ch.line.map((u) => u.emoji).join("")));
    b.addEventListener("click", () => closable(h("div", { class: "label" }, `Champion of day ${d!.seq} · `, who(ch.player.name)), team(ch.line, "ghost", content), hint(`${tapOrClick()} a card to read it.`)));
    pin.replaceChildren(b);
  };
  if (crown) {
    // "Crown vs @X" and the foe's line already name the champion: no pin.
    fillFoe(day);
    void api.day().then((d) => ((day = d), fillFoe(d))).catch(() => fillFoe(day, true));
  } else {
    fillPin(day);
    if (!day) void api.day().then((d) => ((day = d), fillPin(d))).catch(() => {});
  }

  // "?" explains a card's numbers; until a player has opened it once, it says so.
  const legendBtn = button(seen("legend") ? "?" : "? Cards", () => (markSeen("legend"), (legendBtn.textContent = "?"), legendBtn.classList.remove("new"), legendSheet()), seen("legend") ? "small" : "small new", "legend-open");
  renderLine();

  const menuBtn = button("☰", () => runMenu(run, content, err), "menu-btn", "menu-open");
  menuBtn.setAttribute("aria-label", "Menu");
  if (desk) menuBtn.title = "Menu (Esc)";

  // The number keys in plain words: "1–6 buy the offer with that number".
  const n = Math.min(7, run.offers.length);
  const numberKeys = n === 0 ? [] : n === 1 ? [kbd("1"), " buys the offer · "] : [kbd("1"), "–", kbd(String(n)), " buy the offer with that number · "];
  const keysLine =
    desk && !crown
      ? h("div", { class: "dim small keys", "data-testid": "keys" }, "Hover a card to read it → · click selects · drag reorders · double-click buys · ", ...numberKeys, kbd("R"), " reroll · ", kbd("Space"), " fight · ", kbd("←"), kbd("→"), " move · ", kbd("F"), " fuse · ", kbd("S"), " sell · ", kbd("M"), " sound · ", kbd("Esc"), " menu")
      : null;
  show(
    h(
      "div",
      { class: "topbar", "data-testid": "topbar" },
      h(
        "div",
        { class: "hud", "data-testid": "hud" },
        menuBtn,
        h("span", { "data-testid": "round" }, roundLabel(run.round)),
        hearts(run.hearts),
        // The Crown has no shop: no gold to show.
        crown ? h("span", {}) : h("span", { class: "gold", "data-testid": "gold" }, `${run.gold}g`),
      ),
      h(
        "div",
        { class: "row spread opp" },
        h("span", { class: "dim", "data-testid": "next-opponent" }, ...(opp ? [`${crown ? "Crown vs" : "Next:"} `, who(opp.player.name), `${opp.player.bot ? " 🤖" : ""}${ownCrown ? " (your champion team)" : ""}`] : [crown ? "Crown vs today's champion" : "Next: a team saved at this round"])),
        crown ? null : pin,
      ),
    ),
    h(
      "div",
      { class: "board" },
      h(
        "div",
        { class: "row spread line-head" },
        h("div", { class: "label" }, crown ? "Your line · front first · final" : desk ? "Your line · front first · drag to reorder" : "Your line · front first"),
        h("div", { class: "row" }, legendBtn, button("Rules", () => closable(rulesSheet()), "small", "shop-rules")),
      ),
      line,
      desk ? null : actions,
      hintSlot,
      crown ? null : h("div", { class: "label" }, desk ? `Shop · ${plural(run.offers.length, "offer")}` : "Shop · tap to read and buy"),
      crown ? foe : offers,
      keysLine,
    ),
    desk ? inspector : null,
    h("div", { class: "spacer" }),
    h("div", { class: "row shop-foot" }, crown ? null : reroll, fight),
    err,
  );
  screen("shop");
  rerender = () => shopScreen(run, content, err.textContent ?? "", pick.mode === "picked" ? pick.index : -1);
  if (!desk) return;
  // ← / → pressed while the last move was out: the unit goes on by as many slots.
  const queued = moveQueue;
  moveQueue = 0;
  if (queued && pick.mode === "picked" && !crown) {
    const from = pick.index;
    const to = Math.max(0, Math.min(run.line.length - 1, from + queued));
    // After the answering request lets go (guarded ignores a decision while one is out).
    if (to !== from) setTimeout(() => void decide({ kind: "reorder", from, to }, to), 0);
  }

  // Keys: 1–7 buy, R reroll, Space fight, ← → move the selected unit, F fuse,
  // S sell; Esc steps back (a sheet, the fusion, the selection), then opens
  // the ☰ run menu.
  onKeys((e) => {
    const open = app.querySelector(".overlay");
    if (open) return e.key === "Escape" ? (open.remove(), true) : false;
    const k = e.key.length === 1 ? e.key.toLowerCase() : e.key;
    const sel = pick.mode === "picked" ? pick.index : -1;
    if (k === "Escape") {
      if (fuseView) fuseView = null;
      else if (pick.mode !== "none") pick = { mode: "none" };
      else if (chosen !== null) chosen = null;
      else return runMenu(run, content, err), true;
      renderLine();
      return true;
    }
    if (k === " ") {
      if (!fight.disabled) void decide({ kind: "fight" });
      return true;
    }
    if (crown) return false;
    if (/^[1-7]$/.test(k)) {
      const o = run.offers[Number(k) - 1];
      if (o) buy(o);
      return true;
    }
    if (k === "r") {
      if (!reroll.disabled) void decide({ kind: "reroll" });
      else play("wrong");
      return true;
    }
    if ((k === "ArrowLeft" || k === "ArrowRight") && sel >= 0) {
      const step = k === "ArrowLeft" ? -1 : 1;
      // A press while the last move is still out counts too (R2-17 batch E):
      // the shop that answers applies it.
      if (busy) return (moveQueue += step), true;
      const to = sel + step;
      if (to >= 0 && to < run.line.length) void decide({ kind: "reorder", from: sel, to }, to);
      return true;
    }
    if (k === "s" && sel >= 0) {
      void decide({ kind: "sell", index: sel });
      return true;
    }
    if (k === "f") {
      const u = run.line[sel];
      if (pick.mode === "fuse") pick = { mode: "none" };
      else if (u && u.kind === "unit" && u.form === "awoken" && awoken >= 2) pick = { mode: "fuse", first: sel };
      else return false;
      renderLine();
      return true;
    }
    return false;
  });
}

/** ← / → presses the desktop shop took while a move was still out. */
let moveQueue = 0;

/** A key, drawn as a keycap (the desktop shop). */
const kbd = (k: string) => h("kbd", {}, k);

/** Redraws the shop when the width crosses 1024px: its desktop layout is
 * more than CSS (the inspector, the keys). Other screens only restyle. */
let rerender: (() => void) | null = null;
desktopQuery.addEventListener("change", () => (app.dataset.screen === "shop" ? rerender?.() : app.dataset.screen === "codex" ? codexRedraw?.() : undefined));

/** Per-device "seen it once" flags (localStorage may throw or be empty: then everything is new). */
function seen(key: string): boolean {
  try {
    return localStorage.getItem(`arena.seen.${key}`) === "1";
  } catch {
    return false;
  }
}
function markSeen(key: string): void {
  try {
    localStorage.setItem(`arena.seen.${key}`, "1");
  } catch {
    /* private mode: it just stays new */
  }
}

/** ●●○ toward awakening for a sleeping unit; AWOKEN or FUSED otherwise. */
function copiesBadge(u: LineUnit): HTMLElement {
  if (u.kind === "fused") return h("div", { class: "copies tag" }, `FUSED ×${u.copies}`);
  if (u.form === "awoken") return h("div", { class: "copies tag" }, `AWOKEN ×${u.copies}`);
  const n = rules.copiesToAwaken;
  return h("div", { class: "copies pips", "aria-label": `${u.copies} of ${n} copies` }, "●".repeat(Math.min(u.copies, n)) + "○".repeat(Math.max(0, n - u.copies)));
}

// ---------- battle, then result ----------

async function fightScreens(run: RunView, fight: FightResult, content: MvpContent): Promise<void> {
  let battle: BattleRecord;
  try {
    battle = await api.battle(fight.battleId);
  } catch (e) {
    // The fight is already decided: never leave the player on the pre-fight
    // shop. Show the run as it is now, with the outcome and what failed.
    const now = await api.run(run.runId).catch(() => run);
    const word = fight.outcome === "win" ? "Won" : fight.outcome === "loss" ? "Lost" : "Drew";
    const lost = fight.heartsLost > 0 ? ` (−${plural(fight.heartsLost, "heart")})` : "";
    const why = e instanceof Error ? e.message : String(e);
    return shopScreen(now, content, `${word} vs @${fight.opponent.player.name}${lost}. The replay didn't load: ${why}`);
  }
  battleScreen({ battle, content, you: "A", fight, run, outro: outroOf(run, fight, content), onDone: () => shopScreen(run, content) });
}

/** What the battle's end card adds for a run's fight (R2-17 batch E): it is
 * the fight's one result, so it carries what the result screen showed. The
 * round fought (R3/12, or CROWN), the hearts and record after it, and what
 * the result costs or wins; the run's ☰; its last button goes straight on: to
 * the next round's shop, the Crown, or the run's end (the screens that add
 * something). Both lines are on the board and in the card's Damage list,
 * each unit's sheet a tap away; "vs @name" is in the battle's HUD. */
function outroOf(run: RunView, fight: FightResult, content: MvpContent): RunOutro {
  const label = fight.kind === "crown" ? "CROWN" : roundLabel(fight.round);
  const own = fight.kind === "crown" && fight.opponent.player.id === run.player.id;
  // Short notes: they share the run line with the round, hearts and record,
  // which is 310px wide on a 360px phone (R2-17). The hearts show a Crown
  // loss's cost; the run-over screen says the rest in full.
  const sub =
    fight.kind === "crown"
      ? fight.outcome === "win"
        ? "Slayer today!"
        : own
          ? "Your team holds."
          : "The champion holds."
      : fight.heartsLost > 0
        ? `−${plural(fight.heartsLost, "heart")}`
        : fight.outcome === "draw"
          ? "No heart lost."
          : "";
  const err = errorLine();
  return {
    status: () => [
      h("span", { class: "mono", "data-testid": "result-round" }, label),
      hearts(run.hearts),
      h("span", { class: "mono dim" }, record(run)),
      sub ? h("span", { class: fight.heartsLost > 0 ? "error" : "", "data-testid": "result-sub" }, sub) : null,
    ].filter((x): x is HTMLElement => x !== null),
    doneLabel: run.phase === "over" ? "See the run" : run.phase === "crown" ? "To the Crown" : "Next round",
    menu: (resume) => runMenu(run, content, err, fight, resume),
    // The menu's errors show over the battle, not only inside the end card (R2-17 batch F).
    error: err,
  };
}

/** "3W 1D 2L": the run's record, draws only when there are any. */
function record(run: RunView): string {
  const draws = run.fights.filter((f) => f.outcome === "draw").length;
  return `${run.wins}W ${draws ? `${draws}D ` : ""}${run.losses}L`;
}

/** Why a run ended, as a sentence, and how far it got; never "Reached the
 * Crown" for a run that had no champion to fight. */
function runEnd(run: RunView): { why: string; reach: string } {
  const round = `Ended in round ${Math.min(run.round, rules.rounds)} of ${rules.rounds}.`;
  const own = run.fights.some((f) => f.kind === "crown" && f.opponent.player.id === run.player.id);
  switch (run.endedBy) {
    case "out-of-hearts":
      return { why: "Out of hearts.", reach: round };
    case "no-champion":
      return { why: "No champion to face yet, so the run ends here.", reach: `Survived all ${rules.rounds} rounds.` };
    case "crown-won":
      return own
        ? { why: "👑 You beat your own champion team: you are a slayer today.", reach: "Won the Crown." }
        : { why: "👑 You beat the champion: you are a slayer today.", reach: "Won the Crown." };
    case "crown-lost":
      return { why: own ? "Your champion team held the Crown." : "The champion held the Crown.", reach: "Reached the Crown." };
    case "abandoned": {
      const n = run.forfeit?.fights ?? 0;
      return run.round > rules.rounds
        ? { why: "You gave up at the Crown: it counts as a lost Crown.", reach: "Reached the Crown." }
        : { why: n === 1 ? "You gave up: the 1 heart left counts as a lost fight." : `You gave up: the ${n} hearts left count as lost fights.`, reach: round };
    }
    case "content-changed":
      return { why: "The game's units changed since this run began, so it ended here. Your rating stays as it was.", reach: round };
    default:
      return { why: "The run ended.", reach: round };
  }
}

/** `newRun`: the run was given up through Home's New run, so the footer
 * starts the next run (Home stays a tap away). */
function runOverScreen(run: RunView, content: MvpContent, notice = "", newRun = false): void {
  const { why, reach } = runEnd(run);
  const err = errorLine();
  err.textContent = notice;
  const home = () => void homeScreen();
  const next = () => void guarded(err, async () => shopScreen(await api.startRun(), content));
  const draws = run.fights.filter((f) => f.outcome === "draw").length;
  const rc = run.rating;
  const delta = rc ? rc.after - rc.before : 0;
  show(
    h("h1", {}, "RUN OVER"),
    err,
    h(
      "div",
      { class: "panel stack", "data-testid": "run-over" },
      h("div", { "data-testid": "run-why" }, why),
      h("div", { class: "num", "data-testid": "run-record" }, `${plural(run.wins, "win")} · ${plural(draws, "draw")} · ${plural(run.losses, "loss", "losses")}`),
      h("div", { class: "dim" }, reach),
      rc ? h("div", { class: "num", "data-testid": "rating-change" }, `Rating ${rc.before} → ${rc.after} (${delta >= 0 ? "+" : ""}${delta})`) : null,
      // A subtle why: each fight is rated against its opponent (Elo), so the
      // change is K × (wins got − wins expected at your rating).
      rc ? h("div", { class: "dim small num", "data-testid": "rating-why" }, `expected ${rc.expected.toFixed(1)} wins, got ${+rc.actual.toFixed(1)}`) : null,
    ),
    run.line.length ? h("div", { class: "over-line" }, h("div", { class: "label" }, "Your last line"), team(run.line, "you", content)) : null,
    h("div", { class: "spacer" }),
    h("div", { class: "row footer" }, ...(newRun ? [button("Home", home, "grow", "home"), button("New run", next, "primary grow", "new-run-start")] : [button("Home", home, "primary grow", "home")])),
  );
  screen("over");
  onKeys((e) => (e.key === "Enter" ? ((newRun ? next : home)(), true) : false));
}

// ---------- boot ----------

initSound();

/** `?invite=<code>` in the address (slice 13): open the link, then drop the
 * code from the address bar and go Home as its player. A link that fails
 * keeps its code, so Retry tries it again. */
const inviteCode = new URLSearchParams(location.search).get("invite");
if (inviteCode) {
  const err = errorLine();
  void guarded(err, async () => {
    await api.redeem(inviteCode);
    const url = new URL(location.href);
    url.searchParams.delete("invite");
    history.replaceState(null, "", url);
    await homeScreen();
  }).then(() => {
    if (err.textContent)
      show(
        h("h1", {}, "ARENA"),
        h("p", { class: "dim", "data-testid": "invite-bad" }, err.textContent.startsWith("no such invite") ? "This invite link doesn't work. Ask Maks for a new one." : err.textContent),
        button("Retry", () => location.reload(), "primary"),
      );
  });
} else if (api.player) {
  const err = errorLine();
  void guarded(err, async () => {
    // A name from before invites (no token) is no login on an invite-only
    // server: forget it and ask for the link.
    if (!api.hasToken && (await api.health()).invites) {
      api.forget();
      return nameScreen();
    }
    await homeScreen();
  }).then(() => {
    if (err.textContent) show(h("h1", {}, "ARENA"), err, button("Retry", () => location.reload(), "primary"));
  });
} else nameScreen();
