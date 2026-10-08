// Arena MVP phone client: name → home → shop → fight → battle → result.
// Plain DOM; each screen is a function that renders into #app.
// Owners: slice 8 the name, home, shop and result screens here; slice 9 the
// battle viewer (screens/battle.ts); slice 11 the stats page
// (screens/stats.ts). Shared: api.ts, content.ts, ui/.
// Home is the title menu (R2-10); it also shows DayView.lastPlayoff (a game
// opens in battleScreen) and, on dev servers only (HomeView.dev), "End day
// now" under "Dev". The shop's ☰ (Esc on desktop) is the in-run menu.
import type { BattleRecord, CandidateScore, DayView, FightResult, HomeView, IdeasView, LineUnit, MvpContent, MvpRules, Offer, PlayerRef, PlayoffResult, RunView } from "../src/mvp/contract";
import { benchSizeOf, lockedFull, MVP_RULES, offersAt, sellValue } from "../src/mvp/contract";
import { mergeTarget } from "../src/mvp/forms";
import { buttonRefusal, plainRefusal } from "./ui/refusal";
import { ApiError, api, savedPlayer } from "./api";
import { getContent, unitIn } from "./content";
import { battleScreen, type RunOutro } from "./screens/battle";
import { codexScreen, newCodexCache, type CodexState } from "./screens/codex";
import { setCodexLink } from "./ui/term";
import { statsScreen } from "./screens/stats";
import { ideasScreen, ideaWhy } from "./screens/ideas";
import { votePanel } from "./screens/vote";
import { card, roman, setCardCredits, unitSheet, type CardUnit } from "./ui/card";
import { previewName } from "./ui/fusion";
import { fuseWarning } from "./ui/fuse-warn";
import { icon } from "./ui/icon";
import { app, button, closable, desktopQuery, dismissable, h, isDesktop, keepScreen, onKeys, overlay, screen, show, who } from "./ui/dom";
import { loadUnitRates } from "./ui/unit-stats";
import { telegramSheet } from "./screens/telegram";
import { initSound, music, onSoundChange, play, setSound, soundSettings } from "./ui/sound";
import { shopSound } from "./ui/sound-map";
import { t, withNodes } from "./i18n";

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
  if (grow.length === 0) return t("rules.offersFixed", { n: r.offers });
  const last = Math.max(...grow);
  return t("rules.offersGrow", { n: r.offers, max: offersAt(r, last), last });
};
// "tier II opens in round 3, III in 6, IV in 9", from the rules.
const tiersText = (r: MvpRules) =>
  r.tierOpensAt
    .map((round, i) => ({ tier: i + 1, round }))
    .filter((x) => x.round > 1)
    .map((x, i) => t(i === 0 ? "rules.tierFirst" : "rules.tierNext", { tier: roman(x.tier), round: x.round }))
    .join(", ") || t("rules.tiersOpen");
const roundLabel = (round: number) => (round > rules.rounds ? t("run.crownLabel") : t("run.roundLabel", { round, rounds: rules.rounds }));
const hearts = (n: number) => h("span", { class: "hearts", "aria-label": t("run.hearts", { n }) }, "♥".repeat(n) + "♡".repeat(Math.max(0, rules.hearts - n)));
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
  const volume = h("input", { type: "range", min: "0", max: "100", step: "5", "aria-label": t("sound.volume"), "data-testid": "sound-volume" });
  // The slider is read from the event, not captured: no closure holds it.
  volume.addEventListener("input", (e) => setSound({ volume: Number((e.currentTarget as HTMLInputElement).value) / 100 }));
  volume.addEventListener("change", () => play("click"));
  const mToggle = button("", () => setSound({ music: !soundSettings().music }), "sound-toggle", "music-toggle");
  const mVolume = h("input", { type: "range", min: "0", max: "100", step: "5", "aria-label": t("sound.musicVolume"), "data-testid": "music-volume" });
  mVolume.addEventListener("input", (e) => setSound({ musicVolume: Number((e.currentTarget as HTMLInputElement).value) / 100 }));
  const row = h("div", { class: "stack sound-row", "data-testid": "sound-row" }, h("div", { class: "row" }, toggle, volume), h("div", { class: "row" }, mToggle, mVolume));
  syncSoundRow(row);
  // A row set aside (the Codex over Home) still follows M; one thrown away
  // stops listening. The listener holds the row only by its WeakRef (no
  // toggle or slider, whose parentNode would keep it alive), so a thrown-away
  // row is freed and its listener goes at the next change.
  const ref = new WeakRef(row);
  const off = onSoundChange((s) => {
    const r = ref.deref();
    if (r) syncSoundRow(r, s);
    else off();
  });
  return row;
}

/** Draws a Sound row's toggles and sliders from the settings: Sound (the
 * master, M) and Music under it (round 4, note 5). */
function syncSoundRow(row: HTMLElement, s = soundSettings()): void {
  const get = <T extends HTMLElement>(id: string) => row.querySelector<T>(`[data-testid="${id}"]`);
  const toggle = get<HTMLButtonElement>("sound-toggle");
  const volume = get<HTMLInputElement>("sound-volume");
  const mToggle = get<HTMLButtonElement>("music-toggle");
  const mVolume = get<HTMLInputElement>("music-volume");
  if (!toggle || !volume || !mToggle || !mVolume) return;
  toggle.textContent = s.on ? t("sound.on") : t("sound.off");
  toggle.setAttribute("aria-pressed", String(s.on));
  volume.value = String(Math.round(s.volume * 100));
  volume.disabled = !s.on;
  mToggle.textContent = s.music ? t("sound.musicOn") : t("sound.musicOff");
  mToggle.setAttribute("aria-pressed", String(s.music));
  mVolume.value = String(Math.round(s.musicVolume * 100));
  mVolume.disabled = !s.on || !s.music;
}

/** One contextual hint: a line of text, shown where it applies. */
function hint(text: string): HTMLElement {
  return h("div", { class: "hint", "data-testid": "hint" }, text);
}

/** The rules, readable any time (Home's Rules button). */
function rulesSheet(): HTMLElement {
  const r = rules;
  const p = (text: string) => h("p", {}, text);
  const desk = isDesktop();
  const growth = { pwr: r.copyGrowth.pwr, hp: r.copyGrowth.hp };
  return h(
    "div",
    { class: "stack rules", "data-testid": "rules" },
    h("h2", {}, t("rules.title")),
    h("div", { class: "label" }, t("rules.runLabel")),
    p(t("rules.run", { rounds: r.rounds, hearts: t("run.hearts", { n: r.hearts }) })),
    p(t("rules.gold", { gold: r.goldPerRound, unit: r.unitCost, reroll: r.rerollCost, refund: r.sellRefund, awoken: r.sellRefundAwoken && r.sellRefundAwoken !== r.sellRefund ? t("rules.goldAwoken", { n: r.sellRefundAwoken }) : "", offers: offersText(r), tiers: tiersText(r) })),
    p(t(desk ? "rules.lockDesk" : "rules.lockPhone")),
    h("div", { class: "label" }, t("rules.lineLabel")),
    p(t(desk ? "rules.lineDesk" : "rules.linePhone", { n: r.lineSize, battle: r.battleSize && r.battleSize > r.lineSize ? t("rules.lineBattle", { n: r.battleSize }) : "" })),
    ...(benchSizeOf(r) > 0 ? [p(t(desk ? "rules.benchDesk" : "rules.benchPhone", { n: r.benchSize ?? 0 }))] : []),
    h("div", { class: "label" }, t("rules.copiesLabel")),
    p(t("rules.copies", { ...growth, awaken: r.copiesToAwaken })),
    ...(r.giftChoices ? [p(t("rules.gift", { n: r.giftChoices }))] : []),
    p(t(desk ? "rules.fuseDesk" : "rules.fusePhone", growth)),
    h("div", { class: "label" }, t("rules.chainsLabel")),
    p(t(desk ? "rules.chainsDesk" : "rules.chainsPhone")),
    h("div", { class: "label" }, t("rules.dayLabel")),
    p(t("rules.day", { at: r.dayEndsAt })),
    p(t("rules.rating")),
    h("div", { class: "dim small", "data-testid": "icon-credits" }, t("rules.iconCredits")),
  );
}

/** How to read a card (the shop's "?"): each number and mark, with a sample. */
function legendSheet(): HTMLElement {
  const r = rules;
  const row = (sample: Node, text: string) => h("div", { class: "legend-row" }, h("div", { class: "legend-sample" }, sample), h("div", {}, text));
  const span = (cls: string, text: string) => h("span", { class: cls }, text);
  const rulesBtn = button(t("menu.rules"), () => (close(), closable(rulesSheet())), "grow", "legend-rules");
  const sheet = h(
    "div",
    { class: "stack legend", "data-testid": "legend" },
    h("h2", {}, t("legend.title")),
    row(
      h("span", { class: "legend-icons" }, h("span", { class: "tone-when" }, icon("flying-flag", 16)), h("span", { class: "tone-enemy" }, icon("targeted", 16)), h("span", { class: "tone-dmg" }, icon("spiky-explosion", 16))),
      t("legend.icons"),
    ),
    row(h("span", { class: "stats" }, span("p", "2"), "/", span("h", "6")), t("legend.stats")),
    row(span("copies", "●●○"), t("legend.copies", { awaken: r.copiesToAwaken, pwr: r.copyGrowth.pwr, hp: r.copyGrowth.hp })),
    row(span("copies tag", t("legend.awokenTag")), t("legend.awoken", { n: r.copiesToAwaken })),
    ...(r.giftChoices ? [row(span("copies tag", t("legend.giftTag")), t("legend.gift", { n: r.giftChoices }))] : []),
    row(span("copies tag", t("legend.fusedTag")), t("legend.fused")),
    row(span("cost", t("legend.costTag")), t("legend.cost")),
    row(span("cost", t("legend.lockedTag")), t("legend.locked")),
    h("div", { class: "dim small" }, t(isDesktop() ? "legend.sheetDesk" : "legend.sheetPhone")),
  );
  const close = closable(sheet, h("div", { class: "row" }, rulesBtn));
  return sheet;
}

// ---------- name ----------

function nameScreen(): void {
  music("home");
  // An invite-only server (slice 13) takes no new names: a player comes from
  // their invite link. One open to all joins them as the open link does. Nothing but the title shows until /health says which.
  show(h("h1", {}, t("name.title")));
  void api.health().then(
    (hl) => {
      const tg = telegramLogin(hl.telegram);
      if (hl.open) joinForm("", tg);
      else if (hl.invites) show(h("h1", {}, t("name.title")), h("p", { class: "dim", "data-testid": "invite-only" }, t("name.inviteOnly")), tg);
      else nameForm(tg);
    },
    () => nameForm(),
  );
}

/** M4-6: "Log in with Telegram" on the welcome screens, when the server has it. */
function telegramLogin(mode: false | "bot" | "fake" | undefined): HTMLElement | null {
  if (!mode) return null;
  return button(t("tg.login"), () => telegramSheet({ link: false, fake: mode === "fake", onDone: () => void homeScreen() }), "", "tg-login");
}

/** M4-6: the title menu's Telegram row: Link Telegram, or linked with Unlink. */
function telegramRow(linked: boolean, err: HTMLElement): HTMLElement {
  const link = () => void guarded(err, async () => {
    const hl = await api.health();
    telegramSheet({ link: true, fake: hl.telegram === "fake", onDone: () => void homeScreen() });
  });
  return linked
    ? h(
        "div",
        { class: "row", "data-testid": "tg-row" },
        h("span", { class: "dim small grow", "data-testid": "tg-linked" }, t("tg.linked")),
        button(t("tg.unlink"), () => void guarded(err, async () => { await api.telegramUnlink(); await homeScreen(); }), "small", "tg-unlink"),
      )
    : button(t("tg.link"), link, "", "tg-link");
}

function nameForm(tg: HTMLElement | null = null): void {
  const input = h("input", { placeholder: t("name.placeholder"), maxlength: "24", autocomplete: "nickname", "data-testid": "name-input" });
  const err = errorLine();
  const go = () => guarded(err, async () => {
    await api.register(input.value.trim());
    await homeScreen();
  });
  input.addEventListener("keydown", (e) => e.key === "Enter" && void go());
  show(
    h("h1", {}, t("name.title")),
    h("p", { class: "dim" }, t("name.intro")),
    input,
    button(t("name.enter"), () => void go(), "primary", "name-submit"),
    tg,
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
    : button(t("home.play"), () => void guarded(err, async () => shopScreen(await api.startRun(), content)), "primary", "play");
  if (active) play.replaceChildren(t("home.continue", { where: active.round > rules.rounds ? t("home.continueCrown") : t("home.continueRound", { round: active.round }) }), hearts(active.hearts));
  const newRun = active
    ? button(t("home.newRun"), () => abandonSheet(active, "new", () => void guarded(err, async () => {
        // The given-up run's end and rating change first, as ☰ Abandon shows
        // them; its "New run" starts the next one.
        runOverScreen(await api.abandon(active.runId), content, "", true);
      })), "", "new-run")
    : null;
  const codex = button(t("menu.codex"), () => void guarded(err, () => openCodex()), "", "codex");
  const stats = button(t("home.stats"), () => void statsScreen({ content, onBack: () => void homeScreen(), onCodex: () => void openCodex() }), "grow", "stats");
  const rulesBtn = button(t("menu.rules"), () => closable(rulesSheet()), "grow", "rules-open");
  const endDay = button(
    t("dev.endDay"),
    () =>
      void guarded(err, async () => {
        try {
          await api.endDay();
        } catch (e) {
          if (e instanceof ApiError && (e.status === 404 || e.status === 501)) throw new Error(e.status === 404 ? t("dev.endDayDev") : t("dev.endDaySlice"));
          throw e;
        }
        await homeScreen(home.day.seq);
      }),
    "small",
    "end-day",
  );
  const grantIdea = button(t("dev.grantIdea"), () => void guarded(err, async () => { await api.grantIdea(); await homeScreen(); }), "small", "grant-idea");
  // M2-9: a tier I unit becomes your idea, entered today: its cards show 💡 and NEW, its sheet "idea by @you".
  const creditUnit = button(t("dev.creditUnit"), () => void guarded(err, async () => { setCardCredits((await api.creditUnit()).units); await homeScreen(); }), "small", "credit-unit");
  // M2-8: a candidate without the model (M2-5, M2-6), the overnight check now,
  // fake votes, and who qualifies.
  const devNote = (text: string) => ((err.textContent = text), err.classList.add("dev-note"));
  const seedCandidate = button(t("dev.seedCandidate"), () => void guarded(err, async () => { await api.seedCandidate(); devNote(t("dev.seeded")); }), "small", "seed-candidate");
  const overnight = button(t("dev.overnight"), () => void guarded(err, async () => {
    const { started } = await api.overnightCheck();
    devNote(started ? t("dev.checking", { n: started }) : t("dev.noCheck"));
  }), "small", "overnight-check");
  const fakeVotes = button(t("dev.fakeVotes"), () => void guarded(err, async () => { await api.fakeVotes(); closable(candidatesSheet(await api.candidates())); }), "small", "fake-votes");
  const candidates = button(t("dev.candidates"), () => void guarded(err, async () => void closable(candidatesSheet(await api.candidates()))), "small", "candidates");
  const last = home.day.lastPlayoff ?? null;
  const justEnded = ended !== null && last?.seq === ended ? last : null;
  const playoff = playoffPanel(last, champ?.player ?? null, content, err);
  show(
    h(
      "div",
      { class: "row spread" },
      h("h1", {}, t("home.title")),
      h("div", { class: "row me" }, who(api.player?.name ?? "", "dim"), h("span", { class: "dim keep" }, "·"), h("span", { class: "num keep", "data-testid": "rating" }, `${r?.rating ?? rules.ratingStart}`)),
    ),
    // The dev "End day now" just ran: say so first, with how the day ended
    // (the playoff panel) right under it, before today's champion.
    // Desktop: the champion and the day on the left, the menu on the right
    // (.home-main, .home-side; on the phone they are one column).
    h(
      "div",
      { class: "home-main" },
      ended !== null ? h("div", { class: "notice", "data-testid": "day-ended" }, t("home.dayEnded", { ended, today: home.day.seq })) : null,
      justEnded ? playoff : null,
      h(
        "div",
        { class: "panel stack champion", "data-testid": "champion" },
        h("div", { class: "row spread" }, h("div", { class: "label keep" }, t("home.champion", { day: home.day.seq })), champ ? whoMark(champ.player, "ghost-name") : null),
        champ ? team(champ.line, "ghost", content) : h("div", { class: "dim" }, t("home.noChampion")),
        h("div", { class: "dim small", "data-testid": "slayers" }, champ ? slayersLine(home.day.slayers) : t("home.newChampionAt", { at: rules.dayEndsAt })),
      ),
      champ
        ? hint(
            champ.player.id === api.player?.id
              ? t(isDesktop() ? "home.hintOwnDesk" : "home.hintOwnPhone")
              : r
                ? t(isDesktop() ? "home.hintRatedDesk" : "home.hintRatedPhone")
                : t(isDesktop() ? "home.hintNewDesk" : "home.hintNewPhone"),
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
        home.ideas ? ideasLine(home.ideas) : null,
        play,
        newRun,
        codex,
        h("div", { class: "row" }, stats, rulesBtn),
        soundRow(),
        api.ownInvite ? ownLinkRow(api.ownInvite) : null,
        home.telegram?.enabled ? telegramRow(home.telegram.linked, err) : null,
        home.dev
          ? h("details", { class: "dev" }, h("summary", {}, t("dev.title")), h("div", { class: "row wrap" }, endDay, grantIdea, creditUnit, seedCandidate, overnight, fakeVotes, candidates))
          : null,
      ),
      // M2-8's vote card, under the menu: quiet, and Play stays on the first screen.
      votePanel(content),
    ),
  );
  screen("home");
  music("home");
  if (justEnded) playoff?.classList.add("fresh");
}

/** Dev (M2-8): every candidate's votes and score, qualified ones first. */
function candidatesSheet(list: CandidateScore[]): HTMLElement {
  const pct = (x: number) => `${Math.round(x * 100)}%`;
  return h(
    "div",
    { class: "stack", "data-testid": "candidates" },
    h("div", { class: "label" }, t("dev.candidates")),
    ...(list.length
      ? list.map((c) =>
          h(
            "div",
            { class: "idea-row", "data-testid": "candidate-row", "data-qualified": String(c.qualified) },
            h("div", { class: "grow" }, `${c.emoji} ${c.name}`, h("div", { class: "dim small num" }, t("dev.candidateScore", { won: c.won, votes: c.votes, share: pct(c.share), novelty: pct(c.novelty), score: pct(c.score) }))),
            h("div", { class: c.qualified ? "keep" : "dim keep" }, c.qualified ? t("dev.qualified") : t("dev.notYet")),
          ),
        )
      : [h("div", { class: "dim" }, t("dev.noCandidates"))]),
  );
}

/** Home's quiet ideas line (M2-3): the ideas held, or the runs until the
 * next, in My ideas' words ("💡 1 more run for an idea", M3-2). Tapping it
 * opens My ideas (M2-4, screens/ideas.ts). */
function ideasLine(ideas: IdeasView): HTMLElement {
  // M2-6: an idea waiting for its pick comes first: "💡 Your idea is ready".
  const text = ideas.ready
    ? t("home.ideasReady", { n: ideas.ready })
    : ideas.held > 0 || ideas.nextIn === null
      ? t("home.ideasHeld", { n: ideas.held })
      : t("home.ideasNext", { why: ideaWhy(ideas.nextIn) });
  const b = button(text, () => void ideasScreen({ onBack: () => void homeScreen(), onUnknown: () => (api.forget(), nameScreen()) }), "small link ideas", "ideas");
  if (ideas.ready) b.dataset.ready = String(ideas.ready);
  return b;
}

/** One confirm before a run is given up (R2-2's abandon): it says what the
 * rating takes. `why` "menu" is ☰ Abandon run; "new" is New run on the title
 * menu, which ends the waiting run first. */
function abandonSheet(run: RunView, why: "menu" | "new", onConfirm: () => void): void {
  const crown = run.round > rules.rounds;
  const cost = crown ? t("abandon.costCrown") : t("abandon.costHearts", { n: run.hearts });
  const close = overlay(
    h("div", { class: "label" }, why === "new" ? (crown ? t("abandon.waitsCrown") : t("abandon.waitsRound", { round: run.round, rounds: rules.rounds })) : t("abandon.title")),
    h("p", { "data-testid": "abandon-text" }, t(why === "new" ? "abandon.askNew" : "abandon.askMenu", { cost })),
    h(
      "div",
      { class: "row sheet-actions" },
      button(t("abandon.cancel"), () => close(), "grow", "abandon-cancel"),
      button(t("abandon.confirm"), () => (close(), onConfirm()), "danger grow", "abandon-confirm"),
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
  const onUnknown = () => ((codexBack = null), (codexRedraw = null), api.forget(), nameScreen());
  codexRedraw = () => void codexScreen({ content, state: { ...codexCache.state, term: undefined, scope: undefined }, onBack, onUnknown, cache: codexCache });
  await codexScreen({ content, state, onBack, onUnknown, cache: codexCache });
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
  const codex = button(t("menu.codex"), () => (close(), void guarded(err, () => openCodex())), "", "menu-codex");
  const menu = h(
    "div",
    { class: "stack run-menu", "data-testid": "run-menu" },
    h("div", { class: "label" }, run.phase === "over" ? t("menu.runOver") : crown ? t("menu.runCrown") : t("menu.runRound", { round, rounds: rules.rounds })),
    // Over a battle that was playing, Resume plays on (R2-17 batch F).
    button(t("menu.resume"), () => (close(), resume?.()), "primary", "menu-resume"),
    codex,
    button(t("menu.rules"), () => (close(), closable(rulesSheet())), "", "menu-rules"),
    soundRow(),
    button(t("menu.titleMenu"), () => (close(), void guarded(err, () => homeScreen())), "", "menu-title"),
    run.phase === "over" ? null : h("div", { class: "dim small" }, t("menu.waits")),
    run.phase === "over" ? null : button(t("menu.abandon"), () => (close(), abandonSheet(run, "menu", () => void guarded(err, async () => runOverScreen(await api.abandon(run.runId), content)))), "danger", "menu-abandon"),
  );
  // A tap outside the menu, or Esc, is a Resume too.
  const close = dismissable((close) => (close(), resume?.()), menu);
}

/** The champion card's last line: what today's slayers mean at the day's end. */
function slayersLine(n: number): string {
  const at = rules.dayEndsAt;
  return n === 0 ? t("home.slayersNone", { at }) : t("home.slayers", { n, at });
}

/** "@name", with 🤖 after a bot's. */
function whoMark(p: PlayerRef, cls = ""): HTMLElement {
  const el = who(p.name, cls);
  return p.bot ? h("span", { class: "who-mark" }, el, h("span", { class: "bot", "aria-label": t("who.bot") }, "🤖")) : el;
}

/** How a day ended, in one sentence. With no playoff to show (no slayers,
 * or one who won without a game) the sentence is all there is; `champion` is
 * today's, the one who stayed or was crowned. Slayers may be bots (🤖). */
function playoffSummary(p: PlayoffResult, champion: PlayerRef | null): (Node | string)[] {
  if (p.entrants.length === 0) return champion ? withNodes(t("playoff.noSlayersStays"), { who: whoMark(champion) }) : [t("playoff.noSlayers")];
  if (p.entrants.length === 1) {
    const only = p.winner ?? p.entrants[0]!;
    return withNodes(t(only.bot ? "playoff.onlySlayerBot" : "playoff.onlySlayer"), { who: whoMark(only) });
  }
  const bots = p.entrants.filter((x) => x.bot).length;
  const field = t("playoff.field", { n: p.entrants.length, bots: bots === p.entrants.length ? t("playoff.allBots") : bots ? t("playoff.someBots", { n: bots }) : "" });
  return p.winner ? withNodes(t("playoff.won", { field }), { who: whoMark(p.winner) }) : [t("playoff.noWinner", { field })];
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
    btn.replaceChildren(...withNodes(t("playoff.versus"), { a: whoMark(a), b: whoMark(b) }));
    return btn;
  };
  const played = p.entrants.length >= 2;
  const games = h("div", { class: "games", "data-testid": "playoff-games" }, ...p.games.map((g) => watch(g.battleId, g.a, g.b)));
  games.hidden = true;
  const toggle = button(t("playoff.watch", { n: p.games.length }), () => {
    games.hidden = !games.hidden;
    toggle.textContent = games.hidden ? t("playoff.watch", { n: p.games.length }) : t("playoff.hide");
  }, "small", "playoff-games-open");
  return h(
    "div",
    { class: "panel stack playoff", "data-testid": "playoff" },
    h("div", { class: "label" }, played ? t("playoff.label", { day: p.seq }) : t("playoff.dayEnded", { day: p.seq })),
    h("div", { "data-testid": "playoff-summary" }, ...playoffSummary(p, champion)),
    played
      ? h(
          "div",
          { class: "standings" },
          ...p.standings.map((s, i) => h("div", { class: "standing", "data-testid": "playoff-standing" }, h("span", { class: "dim" }, `${i + 1}`), whoMark(s.player), h("span", { class: "num" }, t("playoff.record", { w: s.wins, d: s.draws, l: s.losses })))),
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
function shopScreen(run: RunView, content: MvpContent, notice = "", selected = -1, offer = -1): void {
  if (run.phase === "over") return runOverScreen(run, content, notice);
  const desk = isDesktop();
  const crown = run.phase === "crown";
  // The reigning champion's Crown is their own team; a win is still a slay.
  const ownCrown = crown && run.nextOpponent?.player.id === run.player.id;
  const err = errorLine();
  err.textContent = notice;
  // Board slots (R3-13): 0..L−1 the line, L..L+B−1 the bench; pick and
  // every decision use them.
  const L = rules.lineSize;
  const B = benchSizeOf(rules);
  const unitAt = (slot: number): LineUnit | undefined => (slot < L ? run.line[slot] : run.bench[slot - L]);
  const zoneStart = (slot: number) => (slot < L ? 0 : L);
  const zoneLen = (slot: number) => (slot < L ? run.line.length : run.bench.length);
  const board = [...run.line, ...run.bench];
  const owns = (unitId: string) => mergeTarget(board, unitId) >= 0;
  let pick: Pick = desk && unitAt(selected) ? { mode: "picked", index: selected } : { mode: "none" };
  const unitOf = (id: string) => unitIn(content, id); // a run on an older pool may hold a unit that left
  /** Buy previews still out: a decision waits for them, or the server may
   * apply a buy first and refuse the preview of its slot (a 409, R4-19). */
  const previewsOut = new Set<Promise<unknown>>();
  const decide = (d: Parameters<typeof api.decide>[1], select = -1, offer = -1) =>
    guarded(err, async () => {
      await Promise.allSettled([...previewsOut]);
      const res = await api.decide(run.runId, d);
      if (res.fight) return fightScreens(res.run, res.fight, content);
      play(shopSound(d, run, res.run));
      shopScreen(res.run, content, "", select, offer);
    });
  /** Lock or unlock an offer (free); on desktop it stays chosen in the inspector. */
  const lock = (o: Offer) => void decide({ kind: "lock", slot: o.slot }, -1, desk ? o.slot : -1);
  const lockLabel = (o: Offer) => (o.locked ? t("board.unlock") : t("board.lock"));
  // The last shop round has no Lock: the Crown clears the offers. Unlock stays.
  const lastShop = run.round >= rules.rounds;
  const canLock = (o: Offer) => !run.gift && (o.locked === true || !lastShop);

  const line = h("div", { class: "slots", "data-testid": "line" });
  const bench = h("div", { class: "slots bench-row", "data-testid": "bench" });
  const actions = h("div", { class: "row actions", "data-testid": "actions" });
  const hintSlot = h("div", {});
  const awoken = board.filter((u) => u.kind === "unit" && u.form === "awoken").length;
  // Desktop: the inspector, the card under the mouse, an offer clicked, a fusion waiting for its Fuse.
  const inspector = h("aside", { class: "inspector stack", "data-testid": "inspector" });
  type At = { kind: "line"; index: number } | { kind: "offer"; slot: number };
  let hover: At | null = null;
  let chosen: number | null = desk && run.offers[offer] ? offer : null;
  let fuseView: HTMLElement | null = null;

  /** Move the unit at board slot `from` to `to` (a reorder), keeping it selected where it lands. */
  const moveTo = (from: number, to: number) => {
    if (from === to) return;
    const across = (from < L) !== (to < L);
    // Within a zone, a drop past its last unit moves to the end; across
    // zones, an empty slot takes the unit at that zone's end (packed).
    const dest = across ? (unitAt(to) ? to : zoneStart(to) + zoneLen(to)) : Math.min(to, zoneStart(to) + zoneLen(to) - 1);
    if (dest === from) return;
    void decide({ kind: "reorder", from, to: dest }, dest);
  };
  /** B, "To bench", "To line": into the other zone's first empty slot, else a
   * swap with its last unit. */
  const otherZone = (slot: number): number | null => {
    if (B === 0) return null;
    if (slot < L) return run.bench.length < B ? L + run.bench.length : L + B - 1;
    return run.line.length < L ? run.line.length : L - 1;
  };
  const slotCard = (slot: number): HTMLElement => {
    const u = unitAt(slot);
    const onBench = slot >= L;
    const id = onBench ? `bench-${slot - L}` : `line-${slot}`;
    if (!u) {
      const e = h("div", { class: "card empty", "data-testid": `${id}-empty` }, h("div", { class: "dim" }, onBench ? "" : `${slot + 1}`));
      if (desk && !crown) dropOn(e, slot);
      return e;
    }
    const c = card(u, { side: "you", extra: [copiesBadge(u)], testid: id });
    if (pick.mode === "picked" && pick.index === slot) c.classList.add("selected");
    if (pick.mode === "fuse") {
      if (pick.first === slot) c.classList.add("selected");
      else if (u.kind === "unit" && u.form === "awoken") c.classList.add("fusable");
      else c.classList.add("muted");
    }
    c.addEventListener("click", () => {
      play("click");
      if (pick.mode === "fuse") {
        if (pick.first === slot) pick = { mode: "none" };
        else if (u.kind === "unit" && u.form === "awoken") return void fusePreview(pick.first, slot);
        else return;
      } else if (!desk && !crown && pick.mode === "picked" && pick.index >= L && !onBench && run.line.length >= L) {
        // Phone: a bench unit selected and the line full, a tap on a line unit swaps the two.
        return moveTo(pick.index, slot);
      } else
        // A second tap on the phone puts it down (its buttons go); on desktop a
        // click keeps it in hand, so the next B, S or F still has a unit (Esc lets go).
        pick = !desk && pick.mode === "picked" && pick.index === slot ? { mode: "none" } : { mode: "picked", index: slot };
      chosen = null;
      renderLine();
    });
    if (desk) desktopCard(c, { kind: "line", index: slot });
    return c;
  };

  const renderLine = () => {
    line.replaceChildren(...Array.from({ length: L }, (_, i) => slotCard(i)));
    bench.replaceChildren(
      h("div", { class: "bench-label label" }, h("span", {}, t("board.bench")), h("span", {}, `${run.bench.length}/${B}`)),
      ...Array.from({ length: B }, (_, i) => slotCard(L + i)),
    );
    for (const o of offers.children) o.classList.toggle("selected", chosen !== null && (o as HTMLElement).dataset.testid === `offer-${chosen}`);
    if (desk) renderInspector(true);
    else actions.replaceChildren(...actionButtons());
    hintSlot.replaceChildren(...[giftBanner() ?? shopHint()].filter((x): x is HTMLElement => x !== null));
  };

  /** Desktop: hovering a card reads it in the inspector; a line card drags onto another to reorder. */
  const desktopCard = (c: HTMLElement, at: At) => {
    c.addEventListener("mouseenter", () => ((hover = at), renderInspector()));
    c.addEventListener("mouseleave", () => ((hover = null), renderInspector()));
    if (at.kind !== "line" || crown) return;
    const i = at.index;
    c.draggable = true;
    c.addEventListener("dragstart", (e) => {
      e.dataTransfer?.setData("text/plain", `board:${i}`);
      c.classList.add("dragging");
    });
    c.addEventListener("dragend", () => c.classList.remove("dragging"));
    dropOn(c, i);
  };
  /** Desktop: a card or an empty slot of the line or the bench takes a dragged unit. */
  const dropOn = (c: HTMLElement, slot: number) => {
    c.addEventListener("dragover", (e) => (e.preventDefault(), c.classList.add("drop")));
    c.addEventListener("dragleave", () => c.classList.remove("drop"));
    c.addEventListener("drop", (e) => {
      e.preventDefault();
      c.classList.remove("drop");
      const from = Number(/^board:(\d+)$/.exec(e.dataTransfer?.getData("text/plain") ?? "")?.[1] ?? NaN);
      if (Number.isInteger(from) && from !== slot) moveTo(from, slot);
    });
  };

  const actionButtons = (): HTMLElement[] => {
    if (pick.mode === "fuse") return [h("span", { class: "dim grow" }, desk ? t("board.fuseSecondDesk") : t("board.fuseSecondPhone")), button(desk ? t("board.cancelDesk") : t("board.cancel"), () => ((pick = { mode: "none" }), renderLine()), "", "fuse-cancel")];
    if (pick.mode !== "picked") return [];
    const i = pick.index;
    const u = unitAt(i)!;
    const info = button(t("board.info"), openSheet(u, content), "", "info");
    if (crown) return desk ? [] : [info];
    const left = button(desk ? "◀ ←" : "◀", () => void decide({ kind: "reorder", from: i, to: i - 1 }, i - 1), "", "move-left");
    const right = button(desk ? "→ ▶" : "▶", () => void decide({ kind: "reorder", from: i, to: i + 1 }, i + 1), "", "move-right");
    left.disabled = i === zoneStart(i);
    right.disabled = i >= zoneStart(i) + zoneLen(i) - 1;
    // On desktop the inspector already is the sheet: no Info.
    const out: HTMLElement[] = desk ? [left, right] : [left, right, info];
    const other = otherZone(i);
    if (other !== null) {
      // Into the other zone's first empty slot, else a swap with its back unit: the button says which.
      const swaps = unitAt(other) !== undefined;
      // The phone's row of buttons stays one row: "Swap" there, the hint says with whom.
      const word = i < L ? (swaps ? (desk ? t("board.swapBench", { n: B }) : t("board.swap")) : t("board.toBench")) : swaps ? (desk ? t("board.swapBack") : t("board.swap")) : t("board.toLine");
      out.push(button(desk ? `${word} · B` : word, () => moveTo(i, other), "", i < L ? "to-bench" : "to-line"));
    }
    if (u.kind === "unit" && u.form === "awoken" && awoken >= 2 && !run.gift) out.push(button(desk ? t("board.fuseDesk") : t("board.fuse"), () => ((pick = { mode: "fuse", first: i }), renderLine()), "", "fuse"));
    const value = sellValue(rules, u);
    out.push(button(desk ? t("board.sellDesk", { n: value }) : t("board.sell", { n: value }), () => void decide({ kind: "sell", index: i }), "danger", "sell"));
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
    if (at?.kind === "line" && unitAt(at.index)) {
      const mine = at.index === sel;
      inspector.replaceChildren(
        h("div", { class: "label" }, (mine ? t("board.selected") : "") + (at.index < L ? t("board.inLine", { n: at.index + 1 }) : t("board.onBench", { n: at.index - L + 1 }))),
        unitSheet(unitAt(at.index)!, content),
        mine || pick.mode === "fuse" ? h("div", { class: "row actions", "data-testid": "actions" }, ...actionButtons()) : h("div", { class: "dim small" }, crown ? t("board.crownFinal") : t("board.clickSelect")),
      );
      return ratesToFoot();
    }
    if (at?.kind === "offer") {
      const n = run.offers.findIndex((o) => o.slot === at.slot);
      const o = run.offers[n];
      if (!o) return;
      const head = h("div", { class: "label" }, t("board.offerHead", { n: n + 1 }));
      inspector.replaceChildren(head, h("div", { class: "dim small" }, "…"));
      void offerBody(o).then(
        ({ sheet, blocked }) => {
          if (inspected !== key) return;
          const buy = button(buttonRefusal(blocked) || t("board.buyDesk", { cost: o.cost, n: n + 1 }), () => void decide({ kind: "buy", slot: o.slot }), "primary grow", "buy");
          buy.disabled = blocked !== "";
          const lockBtn = canLock(o) ? [button(`${lockLabel(o)} · L`, () => lock(o), "", "lock")] : [];
          inspector.replaceChildren(head, sheet, h("div", { class: "row" }, ...lockBtn, buy));
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
      h("div", { class: "dim" }, crown ? t("inspector.idleCrown") : t("inspector.idle")),
    );
  };

  /** The hint that matters most right now, or none. */
  const shopHint = (): HTMLElement | null => {
    if (!desk && !crown && pick.mode === "picked" && pick.index >= L && run.line.length >= L) return hint(t("hint.lineFullSwap"));
    if (pick.mode !== "none") return null;
    // run.ts refuses every decision but the fight in the crown phase: the line is final.
    if (crown) return hint(ownCrown ? t("hint.ownCrown") : t("hint.crown"));
    // Only a unit whose next copy is on offer right now.
    const almost = board.find((u) => u.kind === "unit" && u.form === "sleeping" && u.copies === rules.copiesToAwaken - 1 && run.offers.some((o) => o.unitId === u.unitId));
    const canBuy = run.offers.some((o) => o.cost <= run.gold);
    if (awoken >= 2 && !run.gift) return hint(desk ? t("hint.fuseDesk") : t("hint.fusePhone"));
    if (almost && run.offers.some((o) => o.unitId === almost.unitId && o.cost <= run.gold)) return hint(t("hint.oneMore", { name: almost.name }));
    if (run.line.length === 0 && run.bench.length > 0) return hint(t("hint.moveToLine"));
    if (run.line.length === 0 && !canBuy) return hint(t("hint.noGold"));
    if (run.line.length === 0) return hint(desk ? t("hint.buyDesk") : t("hint.buyPhone"));
    if (run.round === 1 && run.line.length > 0 && run.gold < rules.unitCost) return hint(t("hint.outOfGold"));
    if (run.line.length > 1 && run.round <= 2) return hint(desk ? t("hint.moveDesk") : t("hint.movePhone"));
    if (almost) {
      const o = run.offers.find((o) => o.unitId === almost.unitId)!;
      const keep = !o.locked && !lastShop && o.cost > run.gold ? t("hint.lockToKeep") : "";
      return hint(t("hint.oneMoreCost", { name: almost.name, cost: o.cost, keep }));
    }
    const keepIt = lastShop ? undefined : run.offers.find((o) => !o.locked && o.cost > run.gold && owns(o.unitId));
    if (keepIt) return hint(t("hint.ownedCosts", { name: unitOf(keepIt.unitId)?.name ?? t("hint.yourUnit"), cost: keepIt.cost, gold: run.gold }));
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
      const fusedOf = (res: { run: RunView }, o: { first: number }) => [...res.run.line, ...res.run.bench].find((u) => u.uid === unitAt(o.first)!.uid)!;
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
          h("span", { class: "k" }, t("fuse.when")), ` · ${unitAt(o.first)!.name} → `,
          h("span", { class: "k" }, t("fuse.who")), ` · ${unitAt(o.second)!.name} → `,
          h("span", { class: "k" }, t("fuse.does")), t("fuse.both"),
        );
        // Fusions that hurt their own team stay (round 2); the preview says so plainly.
        const warn = fuseWarning(views[at]!.recipe, [unitAt(o.first)!.recipe, unitAt(o.second)!.recipe], content);
        body.replaceChildren(
          h("div", { class: "preview-card" }, card(shown, { side: "you", extra: [copiesBadge(shown)] })),
          recipe,
          ...(warn ? [h("div", { class: "fuse-warn", role: "note", "data-testid": "fuse-warn" }, warn)] : []),
          h(
            "div",
            { class: "row sheet-actions" },
            button(t("fuse.cancel"), () => close(), "grow", "preview-cancel"),
            button(t("fuse.swap"), () => ((at = 1 - at), render()), "", "preview-swap"),
            button(t("fuse.confirm"), () => (close(), void fuse(o, views[at]!)), "primary grow", "preview-confirm"),
          ),
          unitSheet(fused, content, { preview: true }),
        );
      };
      render();
      const head = h("div", { class: "label" }, t("fuse.preview"));
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
      const fused = [...res.run.line, ...res.run.bench].find((u) => u.uid === unitAt(o.first)!.uid);
      const me = savedPlayer()?.id;
      const discovered = previewed.fusion?.name === "" || previewed.fusion?.discoveredBy === null;
      if (!fused || !discovered || fused.fusion?.discoveredBy?.id !== me) return;
      play("discover");
      closable(
        h("div", { class: "label" }, t("fuse.new")),
        h("h2", { class: "reveal", "data-testid": "fusion-reveal" }, t("fuse.discovered", { name: fused.name })),
        h("div", { class: "preview-card" }, card(fused, { side: "you", extra: [copiesBadge(fused)] })),
        unitSheet(fused, content),
      );
    });

  /** Why an offer can't be bought now, decided here so no request goes out
   * (a full line used to cost a 409 on every preview); "" when it can. */
  const buyBlock = (o: Offer): string => {
    if (run.gift) return t("buy.giftFirst");
    if (run.gold < o.cost) return t("buy.needs", { cost: o.cost });
    if (run.line.length >= L && run.bench.length >= B && !owns(o.unitId)) return B > 0 ? t("refusal.bothFull") : t("refusal.lineFull");
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
  const buyPreview = (o: Offer) => {
    const ask = api.preview(run.runId, { kind: "buy", slot: o.slot }).catch((e: unknown) => (e instanceof ApiError && e.status === 409 ? e : Promise.reject(e)));
    previewsOut.add(ask);
    void ask.finally(() => previewsOut.delete(ask)).catch(() => {});
    return ask;
  };
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
      const before = new Map(board.map((x) => [x.uid, x]));
      const changed = [...res.run.line, ...res.run.bench].find((x) => !before.has(x.uid) || before.get(x.uid)!.copies !== x.copies);
      if (changed) {
        const was = before.get(changed.uid);
        const label = !was ? (res.run.bench.some((x) => x.uid === changed.uid) ? t("buy.toBench") : t("buy.toLine")) : was.form !== changed.form ? t("buy.awakens") : t("buy.mergesIn", { copies: changed.copies });
        if (was) mine = { now: was, next: changed };
        after = h("div", { class: `stack after${was && was.form !== changed.form ? " awakens" : ""}`, "data-testid": "buy-preview" }, h("div", { class: "label" }, label), h("div", { class: "preview-card" }, card(changed, { side: "you", extra: [copiesBadge(changed)] })));
      }
    }
    // Without a preview (no gold, a full line), an owned unit still shows your copy as it is.
    const owned = mine ? null : board.find((x) => x.kind === "unit" && x.unitId === o.unitId) ?? null;
    const sheet = mine ? unitSheet(mine.next, content, { from: mine.now.stats }) : owned ? unitSheet(owned, content) : u ? unitSheet(u, content) : h("h2", {}, o.unitId);
    // What buying does goes inside the sheet, above its last line (the rates hint).
    if (after) sheet.insertBefore(after, sheet.querySelector('[data-testid="unit-rates"]'));
    return { sheet, blocked };
  };
  /** The phone's offer sheet: the body in an overlay, with Close and Buy. */
  const offerSheet = (o: Offer) =>
    guarded(err, async () => {
      const { sheet, blocked } = await offerBody(o);
      const buy = button(buttonRefusal(blocked) || t("buy.button", { cost: o.cost }), () => (close(), void decide({ kind: "buy", slot: o.slot })), "primary grow", "buy");
      buy.disabled = blocked !== "";
      const lockBtn = canLock(o) ? [button(lockLabel(o), () => (close(), lock(o)), "", "lock")] : [];
      const close = overlay(sheet, h("div", { class: "row sheet-actions" }, button(t("buy.close"), () => close(), "", "offer-close"), ...lockBtn, buy));
    });

  const offers = h(
    "div",
    { class: "slots", "data-testid": "offers" },
    ...run.offers.map((o: Offer) => {
      const u = unitOf(o.unitId);
      const cu: CardUnit = { unitId: o.unitId, emoji: u?.emoji ?? "?", name: u?.name ?? o.unitId, stats: u?.base ?? { pwr: 0, hp: 0 }, ...(u ? { recipe: u.forms.sleeping } : {}) };
      const owned = owns(o.unitId);
      // The lock is a corner mark, so "3g ＋" keeps one line at 360 px.
      const price = `${t("shop.gold", { n: o.cost })}${owned ? " ＋" : ""}`;
      const c = card(cu, { side: "you", tier: o.tier, extra: [h("div", { class: "cost" }, price)], testid: `offer-${o.slot}` });
      if (o.locked) c.append(h("span", { class: "lock-mark", "aria-label": t("offer.locked") }, "🔒"));
      if (run.gold < o.cost) c.classList.add("poor");
      if (o.locked) c.classList.add("locked");
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
        c.addEventListener("contextmenu", (e) => (e.preventDefault(), canLock(o) && lock(o)));
        desktopCard(c, { kind: "offer", slot: o.slot });
      }
      return c;
    }),
  );

  const reroll = button(t("shop.reroll", { cost: rules.rerollCost }), () => void decide({ kind: "reroll" }), "", "reroll");
  // A reroll with locked offers filling the whole shop would redraw nothing
  // (run.ts refuses it); empty slots still refill.
  const allLocked = lockedFull({ offers: run.offers, rules, round: run.round });
  reroll.disabled = run.gold < rules.rerollCost || allLocked || !!run.gift;
  if (allLocked) reroll.title = t("shop.allLocked");
  const fight = button(crown ? t("shop.fightChampion") : t("shop.fight"), () => void decide({ kind: "fight" }), "primary grow", "fight");
  // An empty line can fight (and lose a heart) once nothing is affordable, so a broke run moves on.
  fight.disabled = (run.line.length === 0 && run.offers.some((o) => o.cost <= run.gold)) || !!run.gift;
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
      h("div", { class: "label" }, ownCrown ? t("foe.ownCrown") : t("foe.crown"), who(opp.player.name, "ghost-name")),
      line ? team(line, "ghost", content, "crown-foe-line") : h("div", { class: "dim small" }, settled ? t("foe.lineInFight") : "…"),
    );
  };
  const fillPin = (d: DayView | null) => {
    const ch = d?.champion;
    if (!ch) return pin.replaceChildren(h("span", { class: "dim" }, t("pin.noChampion")));
    const b = h("button", { class: "pin-btn", "data-testid": "champion-pin-open" }, h("span", {}, "👑"), who(ch.player.name, "ghost-name"), h("span", { class: "pin-emoji" }, ch.line.map((u) => u.emoji).join("")));
    b.addEventListener("click", () => closable(h("div", { class: "label" }, t("pin.championOfDay", { seq: d!.seq }), who(ch.player.name)), team(ch.line, "ghost", content), hint(isDesktop() ? t("pin.readCardClick") : t("pin.readCardTap"))));
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


  // ---------- the awakening gift (R3-16) ----------
  // The copy that awakens a unit brings a free pick of 1 of 3 units (R3-15,
  // run.gift). The chooser opens over the shop; Esc, a tap outside and "Make
  // room" set it aside behind a banner (Open gift), and while it waits only
  // selling and reordering go through. A gift set aside because line and
  // bench were full opens again once a sale makes room.
  const full = run.line.length >= L && run.bench.length >= B;
  const giftKey = run.gift ? `${run.runId}:${run.round}:${run.gift.join(",")}` : "";
  const giftPickable = (id: string) => !full || owns(id);
  // Line and bench full and some pick can't join (one you own still merges in):
  // the chooser says so and offers Make room, and a sale reopens it.
  const cramped = (gift: string[]) => gift.some((id) => !giftPickable(id));
  function giftBanner(): HTMLElement | null {
    if (!run.gift || crown) return null;
    const stuck = cramped(run.gift);
    return h(
      "div",
      { class: "gift-banner row", "data-testid": "gift-banner" },
      h("span", { class: "grow" }, stuck ? t("gift.waitingFull") : t("gift.waiting")),
      button(t("gift.open"), () => openGift(), "primary small", "gift-open"),
    );
  }
  function openGift(): void {
    const gift = run.gift;
    if (!gift) return;
    giftAside = null;
    const stuck = cramped(gift);
    const none = !gift.some(giftPickable);
    const tier = unitOf(gift[0]!)?.tier;
    // Desktop: the read pane, the hovered card's full sheet (the first card's to start).
    const readers: (() => HTMLElement)[] = [];
    const read = h("div", { class: "gift-read", "data-testid": "gift-read" });
    const readGift = (i: number) => {
      if (read.dataset.card === String(i)) return;
      read.dataset.card = String(i);
      read.replaceChildren(h("div", { class: "label" }, t("gift.of", { i: i + 1, n: gift.length })), readers[i]!());
      choices.forEach((ch, j) => ch.classList.toggle("inspected", j === i));
    };
    const choices = gift.map((id, i) => {
      const u = unitOf(id);
      // Yours: a copy merges in, a fused unit's part included (owns()).
      const mine = board[mergeTarget(board, id)] ?? null;
      const cu: CardUnit = { unitId: id, emoji: u?.emoji ?? "?", name: u?.name ?? id, stats: u?.base ?? { pwr: 0, hp: 0 }, ...(u ? { recipe: u.forms.sleeping } : {}) };
      const c = card(cu, { side: "you", ...(u ? { tier: u.tier } : {}), extra: mine ? [h("div", { class: "cost" }, "＋")] : [], testid: `gift-card-${i}` });
      if (mine) c.classList.add("owned");
      const sheetOf = () => (mine ? unitSheet(mine, content) : u ? unitSheet(u, content) : h("h2", {}, id));
      readers.push(() => sheetOf());
      // Desktop (R4-5): hovering a card reads it in the pane beside the cards
      // (a click too); a phone taps it open as a sheet over the chooser.
      if (desk) {
        c.addEventListener("mouseenter", () => readGift(i));
        c.addEventListener("click", () => readGift(i));
      } else c.addEventListener("click", () => (play("click"), void closable(sheetOf())));
      const ok = giftPickable(id);
      const pickBtn = button(ok ? (mine ? t("gift.pickOwned") : t("gift.pick")) : t("gift.full"), () => (close(), void decide({ kind: "gift", pick: i })), "primary", `gift-pick-${i}`);
      pickBtn.disabled = !ok;
      return h("div", { class: "gift-choice" }, c, pickBtn);
    });
    const aside = () => {
      giftAside = { key: giftKey, full: stuck };
      close();
      renderLine();
    };
    const kids = [
      h("div", { class: "label" }, t("gift.label")),
      h("h2", { class: "reveal", "data-testid": "gift-title" }, t("gift.title")),
      h("div", { class: "dim small" }, tier ? t("gift.freeTier", { tier: roman(tier) }) : t("gift.free")),
      ...(stuck ? [h("div", { class: "hint", "data-testid": "gift-full" }, none ? t("gift.noneFits") : t("gift.onlyOwned"))] : []),
      h("div", { class: "gift-choices", "data-testid": "gift-choices" }, ...choices),
      h("div", { class: "dim small" }, desk ? t("gift.readDesk") : t("gift.readPhone")),
      h(
        "div",
        { class: "row sheet-actions" },
        stuck ? button(t("gift.makeRoom"), aside, "grow", "gift-make-room") : null,
        button(t("gift.skip"), () => (close(), void decide({ kind: "gift", pick: null })), stuck ? "" : "grow", "gift-skip"),
      ),
    ];
    const close = desk ? dismissable(aside, h("div", { class: "gift-split" }, h("div", { class: "stack gift-main" }, ...kids), read)) : dismissable(aside, ...kids);
    if (desk) {
      read.closest(".sheet")?.classList.add("gift-sheet");
      readGift(0);
    }
  }

  // "?" explains a card's numbers; until a player has opened it once, it says so.
  const legendBtn = button(seen("legend") ? "?" : t("shop.legendNew"), () => (markSeen("legend"), (legendBtn.textContent = "?"), legendBtn.classList.remove("new"), legendSheet()), seen("legend") ? "small" : "small new", "legend-open");
  renderLine();

  const menuBtn = button("☰", () => runMenu(run, content, err), "menu-btn", "menu-open");
  menuBtn.setAttribute("aria-label", t("shop.menu"));
  if (desk) menuBtn.title = t("shop.menuEsc");

  // The number keys in plain words: "1–6 buy the offer with that number".
  const n = Math.min(7, run.offers.length);
  // One line at 1440 px (R3-26): short words, the keys say the rest.
  const numberKeys = n === 0 ? [] : n === 1 ? [t("keys.or"), kbd("1")] : [t("keys.or"), kbd("1"), "–", kbd(String(n))];
  const keysLine =
    desk && !crown
      ? h("div", { class: "dim small keys", "data-testid": "keys" }, t("keys.lead"), ...numberKeys, t("keys.buys"), kbd("R"), t("keys.reroll"), kbd("L"), t("keys.lock"), kbd("Space"), t("keys.fight"), kbd("←"), kbd("→"), t("keys.move"), kbd("F"), t("keys.fuse"), kbd("S"), t("keys.sell"), ...(B > 0 ? [kbd("B"), t("keys.bench")] : []), kbd("M"), t("keys.sound"), kbd("Esc"), t("keys.menu"))
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
        crown ? h("span", {}) : h("span", { class: "gold", "data-testid": "gold" }, t("shop.gold", { n: run.gold })),
      ),
      h(
        "div",
        { class: "row spread opp" },
        h("span", { class: "dim", "data-testid": "next-opponent" }, ...(opp ? [`${crown ? t("opp.crownVs") : t("opp.next")} `, who(opp.player.name), `${opp.player.bot ? " 🤖" : ""}${ownCrown ? t("opp.yourChampionTeam") : ""}`] : [crown ? t("opp.crownVsChampion") : t("opp.nextSaved")])),
        crown ? null : pin,
      ),
    ),
    h(
      "div",
      { class: "board" },
      h(
        "div",
        { class: "row spread line-head" },
        h("div", { class: "label" }, crown ? t("line.crown") : desk ? t("line.desk") : t("line.phone")),
        h("div", { class: "row" }, legendBtn, button(t("shop.rules"), () => closable(rulesSheet()), "small", "shop-rules")),
      ),
      line,
      B > 0 ? bench : null,
      desk ? null : actions,
      hintSlot,
      crown ? null : h("div", { class: "label" }, desk ? t("shop.offersDesk", { n: run.offers.length }) : t("shop.offersPhone")),
      crown ? foe : offers,
      keysLine,
    ),
    desk ? inspector : null,
    h("div", { class: "spacer" }),
    h("div", { class: "row shop-foot" }, crown ? null : reroll, fight),
    err,
  );
  screen("shop");
  music("shop", run.runId);
  // The gift opens by itself, unless it was set aside (and, if set aside for
  // a full board, there is still no room).
  if (run.gift && !crown && (giftAside?.key !== giftKey || (giftAside.full && !cramped(run.gift)))) openGift();
  rerender = () => shopScreen(run, content, err.textContent ?? "", pick.mode === "picked" ? pick.index : -1);
  // Keys: Esc steps back (the fusion, the selection), then opens the ☰ run
  // menu, at every width (a sheet over the shop closes first: ui/dom.ts).
  // Desktop: 1–7 buy, R reroll, L lock the chosen or hovered offer, Space
  // fight, ← → move the selected unit, F fuse, S sell.
  onKeys((e) => {
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
    if (!desk) return false;
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
    if (k === "l") {
      // The chosen offer, else the one under the mouse.
      const slot = chosen ?? (hover?.kind === "offer" ? hover.slot : null);
      const o = slot === null ? undefined : run.offers[slot];
      if (!o || !canLock(o)) return false;
      lock(o);
      return true;
    }
    if ((k === "ArrowLeft" || k === "ArrowRight") && sel >= 0) {
      const step = k === "ArrowLeft" ? -1 : 1;
      // A press while the last move is still out counts too (R2-17 batch E):
      // the shop that answers applies it.
      if (busy) return (moveQueue += step), true;
      const to = sel + step;
      if (to >= zoneStart(sel) && to < zoneStart(sel) + zoneLen(sel)) void decide({ kind: "reorder", from: sel, to }, to);
      return true;
    }
    // B, S or F with nothing in hand says so instead of doing nothing.
    if ((k === "b" || k === "s" || k === "f") && sel < 0 && pick.mode !== "fuse") {
      err.textContent = t("shop.selectFirst");
      play("wrong");
      return true;
    }
    if (k === "b" && sel >= 0) {
      const other = otherZone(sel);
      if (other === null) return false;
      moveTo(sel, other);
      return true;
    }
    if (k === "s" && sel >= 0) {
      void decide({ kind: "sell", index: sel });
      return true;
    }
    if (k === "f") {
      const u = unitAt(sel);
      if (pick.mode === "fuse") pick = { mode: "none" };
      else if (u && u.kind === "unit" && u.form === "awoken" && awoken >= 2 && !run.gift) pick = { mode: "fuse", first: sel };
      else return false;
      renderLine();
      return true;
    }
    return false;
  });
  if (!desk) return;
  // ← / → pressed while the last move was out: the unit goes on by as many slots.
  const queued = moveQueue;
  moveQueue = 0;
  if (queued && pick.mode === "picked" && !crown) {
    const from = pick.index;
    const to = Math.max(zoneStart(from), Math.min(zoneStart(from) + zoneLen(from) - 1, from + queued));
    // After the answering request lets go (guarded ignores a decision while one is out).
    if (to !== from) setTimeout(() => void decide({ kind: "reorder", from, to }, to), 0);
  }

}

/** The awakening gift the player set aside (Esc, a tap outside, Make room),
 * and whether the board was full then: the shop doesn't reopen it by itself
 * until a sale makes room (or a new gift comes). */
let giftAside: { key: string; full: boolean } | null = null;

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
  // The word is its own span: the bench's short cards show only "×n" (R3-14).
  if (u.kind === "fused") return h("div", { class: "copies tag" }, h("span", { class: "tag-word" }, t("copies.fused")), `×${u.copies}`);
  if (u.form === "awoken") return h("div", { class: "copies tag" }, h("span", { class: "tag-word" }, t("copies.awoken")), `×${u.copies}`);
  const n = rules.copiesToAwaken;
  return h("div", { class: "copies pips", "aria-label": t("copies.aria", { copies: u.copies, n }) }, "●".repeat(Math.min(u.copies, n)) + "○".repeat(Math.max(0, n - u.copies)));
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
    const word = fight.outcome === "win" ? t("fight.won") : fight.outcome === "loss" ? t("fight.lost") : t("fight.drew");
    const lost = fight.heartsLost > 0 ? t("fight.heartsLost", { hearts: t("fight.hearts", { n: fight.heartsLost }) }) : "";
    const why = e instanceof Error ? e.message : String(e);
    return shopScreen(now, content, t("fight.replayFailed", { word, name: fight.opponent.player.name, lost, why }));
  }
  music("battle", run.runId);
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
        ? t("fight.slayer")
        : own
          ? t("fight.ownHolds")
          : t("fight.championHolds")
      : fight.heartsLost > 0
        ? t("fight.heartsLostShort", { hearts: t("fight.hearts", { n: fight.heartsLost }) })
        : fight.outcome === "draw"
          ? t("fight.noHeartLost")
          : "";
  const err = errorLine();
  return {
    status: () => [
      h("span", { class: "mono", "data-testid": "result-round" }, label),
      hearts(run.hearts),
      h("span", { class: "mono dim" }, record(run)),
      sub ? h("span", { class: fight.heartsLost > 0 ? "error" : "", "data-testid": "result-sub" }, sub) : null,
    ].filter((x): x is HTMLElement => x !== null),
    doneLabel: run.phase === "over" ? t("fight.seeRun") : run.phase === "crown" ? t("fight.toCrown") : t("fight.nextRound"),
    menu: (resume) => runMenu(run, content, err, fight, resume),
    // The menu's errors show over the battle, not only inside the end card (R2-17 batch F).
    error: err,
  };
}

/** "3W 1D 2L": the run's record, draws only when there are any. */
function record(run: RunView): string {
  const draws = run.fights.filter((f) => f.outcome === "draw").length;
  return draws ? t("run.recordDraws", { wins: run.wins, draws, losses: run.losses }) : t("run.record", { wins: run.wins, losses: run.losses });
}

/** Why a run ended, as a sentence, and how far it got; never "Reached the
 * Crown" for a run that had no champion to fight. */
function runEnd(run: RunView): { why: string; reach: string } {
  const round = t("end.round", { round: Math.min(run.round, rules.rounds), rounds: rules.rounds });
  const own = run.fights.some((f) => f.kind === "crown" && f.opponent.player.id === run.player.id);
  switch (run.endedBy) {
    case "out-of-hearts":
      return { why: t("end.outOfHearts"), reach: round };
    case "no-champion":
      return { why: t("end.noChampion"), reach: t("end.survived", { rounds: rules.rounds }) };
    case "crown-won":
      return own
        ? { why: t("end.beatOwn"), reach: t("end.wonCrown") }
        : { why: t("end.beatChampion"), reach: t("end.wonCrown") };
    case "crown-lost":
      return { why: own ? t("end.ownHeld") : t("end.championHeld"), reach: t("end.reachedCrown") };
    case "abandoned": {
      const n = run.forfeit?.fights ?? 0;
      return run.round > rules.rounds
        ? { why: t("end.gaveUpCrown"), reach: t("end.reachedCrown") }
        : { why: t("end.gaveUp", { n }), reach: round };
    }
    case "content-changed":
      return { why: t("end.contentChanged"), reach: round };
    default:
      return { why: t("end.ended"), reach: round };
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
    h("h1", {}, t("over.title")),
    err,
    h(
      "div",
      { class: "panel stack", "data-testid": "run-over" },
      h("div", { "data-testid": "run-why" }, why),
      h("div", { class: "num", "data-testid": "run-record" }, t("over.record", { wins: t("over.wins", { n: run.wins }), draws: t("over.draws", { n: draws }), losses: t("over.losses", { n: run.losses }) })),
      h("div", { class: "dim" }, reach),
      rc ? h("div", { class: "num", "data-testid": "rating-change" }, t("over.rating", { before: rc.before, after: rc.after, delta: `${delta >= 0 ? "+" : ""}${delta}` })) : null,
      // A subtle why: each fight is rated against its opponent (Elo), so the
      // change is K × (wins got − wins expected at your rating).
      rc ? h("div", { class: "dim small num", "data-testid": "rating-why" }, t("over.ratingWhy", { expected: rc.expected.toFixed(1), actual: +rc.actual.toFixed(1) })) : null,
    ),
    run.line.length ? h("div", { class: "over-line" }, h("div", { class: "label" }, t("over.lastLine")), team(run.line, "you", content)) : null,
    votePanel(content),
    h("div", { class: "spacer" }),
    h("div", { class: "row footer" }, ...(newRun ? [button(t("over.home"), home, "grow", "home"), button(t("over.newRun"), next, "primary grow", "new-run-start")] : [button(t("over.home"), home, "primary grow", "home")])),
  );
  screen("over");
  music("shop", run.runId);
  onKeys((e) => (e.key === "Enter" ? ((newRun ? next : home)(), true) : e.key === "Escape" ? (home(), true) : false));
}

/** This device's own invite link (R4-20), after a join through the open link. */
const ownLink = (code: string) => `${location.origin}${location.pathname}#invite=${code}`;

/** Home's "Your link (for another device)": folded, so the key isn't on screen
 * until asked for. */
function ownLinkRow(code: string): HTMLElement {
  const link = ownLink(code);
  const field = h("input", { readonly: "", value: link, "data-testid": "own-link" });
  field.addEventListener("focus", () => field.select());
  const copy = button(t("link.copy"), () => {
    void navigator.clipboard?.writeText(link).then(() => (copy.textContent = t("link.copied")), () => field.select());
  }, "small", "own-link-copy");
  return h(
    "details",
    { class: "dev", "data-testid": "own-link-row" },
    h("summary", {}, t("link.summary")),
    h("div", { class: "dim small" }, t("link.about")),
    h("div", { class: "row" }, field, copy),
  );
}

// ---------- boot ----------

/** `#invite=<code>` in the address (slice 13): open the link, then drop the
 * code from the address bar and go Home as its player. The code rides in the
 * fragment, which the browser never sends, so no access log holds it. A
 * device that is already another player asks first. A dead link offers Home
 * on a device with its own session, and otherwise asks for a new link. */
const inviteCode = new URLSearchParams(location.hash.slice(1)).get("invite");
/** `#join=<code>` (R4-20): the open join link. Anyone with it picks a name
 * and plays as a new player; the fragment then becomes their own #invite=
 * link, so a bookmark keeps working. A bad or rotated code shows the invite
 * screen. */
const joinCode = new URLSearchParams(location.hash.slice(1)).get("join");
const dropInvite = () => {
  const url = new URL(location.href);
  url.hash = "";
  history.replaceState(null, "", url.href.replace(/#$/, ""));
};
function openInvite(code: string): void {
  const err = errorLine();
  void guarded(err, async () => {
    const mine = api.player;
    if (mine) {
      const theirs = await api.invitePlayer(code);
      if (theirs.id !== mine.id) return switchScreen(code, mine, theirs);
    }
    await api.redeem(code);
    dropInvite();
    await homeScreen();
  }).then(() => {
    if (!err.textContent) return;
    const own = api.player && api.hasToken;
    const dead = err.textContent.startsWith("no such invite");
    show(
      h("h1", {}, t("invite.title")),
      h("p", { class: "dim", "data-testid": "invite-bad" }, dead ? (own ? t("invite.deadOwn") : t("invite.dead")) : err.textContent),
      own
        ? button(t("invite.home"), () => (dropInvite(), void guarded(errorLine(), () => homeScreen())), "primary", "invite-home")
        : dead
          ? h("span", {})
          : button(t("invite.retry"), () => location.reload(), "primary"),
    );
  });
}
function openJoinLink(code: string): void {
  const err = errorLine();
  void guarded(err, async () => {
    await api.joinCheck(code);
    const mine = api.player;
    if (mine && api.hasToken) return joinSwitchScreen(code, mine);
    joinForm(code);
  }).then(() => {
    if (!err.textContent) return;
    const own = api.player && api.hasToken;
    const dead = err.textContent.startsWith("no such join link");
    show(
      h("h1", {}, t("invite.title")),
      h("p", { class: "dim", "data-testid": "invite-bad" }, dead ? (own ? t("invite.deadOwn") : t("invite.dead")) : err.textContent),
      own
        ? button(t("invite.home"), () => (dropInvite(), void guarded(errorLine(), () => homeScreen())), "primary", "invite-home")
        : dead
          ? h("span", {})
          : button(t("invite.retry"), () => location.reload(), "primary"),
    );
  });
}
/** A device that already plays as someone opens the join link: stay, or start a new player here. */
function joinSwitchScreen(code: string, mine: PlayerRef): void {
  const stay = () => (dropInvite(), void guarded(errorLine(), () => homeScreen()));
  show(
    h("h1", {}, t("invite.title")),
    h("p", { "data-testid": "join-switch" }, t("join.playingAs", { name: mine.name })),
    h("p", { class: "dim" }, t("join.needsLink", { name: mine.name })),
    h("div", { class: "row footer" }, button(t("join.stay"), stay, "grow", "join-stay"), button(t("join.newPlayer"), () => joinForm(code), "primary grow", "join-new")),
  );
  onKeys((e) => (e.key === "Escape" ? (stay(), true) : false));
}
function joinForm(code: string, tg: HTMLElement | null = null): void {
  music("home");
  const input = h("input", { placeholder: t("join.yourName"), maxlength: "24", autocomplete: "nickname", "data-testid": "join-name" });
  const err = errorLine();
  const go = () => guarded(err, async () => {
    const s = await api.join(code, input.value.trim());
    history.replaceState(null, "", ownLink(s.invite));
    await homeScreen();
  });
  input.addEventListener("keydown", (e) => e.key === "Enter" && void go());
  show(
    h("h1", {}, t("join.title")),
    h("p", { class: "dim" }, t("join.about")),
    input,
    button(t("join.play"), () => void go(), "primary", "join-submit"),
    tg,
    err,
  );
  input.focus();
}
function switchScreen(code: string, mine: PlayerRef, theirs: PlayerRef): void {
  const stay = () => (dropInvite(), void guarded(errorLine(), () => homeScreen()));
  const go = () => void guarded(errorLine(), async () => {
    await api.redeem(code);
    dropInvite();
    await homeScreen();
  });
  show(
    h("h1", {}, t("invite.title")),
    h("p", { "data-testid": "invite-switch" }, t("switch.for", { theirs: theirs.name, mine: mine.name })),
    h("p", { class: "dim" }, t("switch.then", { theirs: theirs.name, mine: mine.name })),
    h("div", { class: "row footer" }, button(t("join.stay"), stay, "grow", "invite-stay"), button(t("switch.go"), go, "primary grow", "invite-switch-go")),
  );
  onKeys((e) => (e.key === "Escape" ? (stay(), true) : false));
}

initSound();
// A link pasted into a tab that already shows the game only changes the
// fragment, which reloads nothing: start over so the link opens.
addEventListener("hashchange", () => {
  const hash = new URLSearchParams(location.hash.slice(1));
  if (hash.has("invite") || hash.has("join")) location.reload();
});

if (inviteCode) {
  openInvite(inviteCode);
} else if (joinCode) {
  openJoinLink(joinCode);
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
    if (err.textContent) show(h("h1", {}, t("invite.title")), err, button(t("invite.retry"), () => location.reload(), "primary"));
  });
} else nameScreen();
