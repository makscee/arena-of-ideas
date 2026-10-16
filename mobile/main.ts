// Arena MVP phone client: name → home → shop → fight → battle → result.
// Plain DOM; each screen is a function that renders into #app.
// Owners: slice 8 the name, home, shop and result screens here; slice 9 the
// battle viewer (screens/battle.ts); slice 11 the stats page
// (screens/stats.ts). Shared: api.ts, content.ts, ui/.
// Home is the title menu (R2-10); it also shows DayView.lastPlayoff (a game
// opens in battleScreen) and, on dev servers only (HomeView.dev), "End day
// now" under "Dev". The shop's ☰ (Esc on desktop) is the in-run menu.
import { sceneId } from "./scene";
import { CG } from "./cg";
import { cgApplySettings, cgInit, cgLoading, cgPlaytestHooks, cgUserToken, cgWatchGameplay, onCgLogin } from "./crazygames";
import type { BattleRecord, CandidateScore, DayView, FightResult, HomeView, IdeasView, LineUnit, MvpContent, MvpRules, Offer, PlayerRef, PlayoffResult, RunView, UnitContent } from "../src/mvp/contract";
import { benchSizeOf, lockedFull, MVP_RULES, offersAt, sellValue } from "../src/mvp/contract";
import { mergeTarget } from "../src/mvp/forms";
import type { ForgeOffer, ForgeSlot, ForgeView, RunForge } from "../src/mvp/codex";
import { buttonRefusal, plainRefusal } from "./ui/refusal";
import { ApiError, api, savedPlayer } from "./api";
import { getContent, unitIn } from "./content";
import { battleScreen, type RunOutro } from "./screens/battle";
import { codexScreen, newCodexCache, type CodexState } from "./screens/codex";
import { claimScreen, forgeScreen, type ForgeTab } from "./screens/forge";
import { forgeDesignOf, forgeUnitOf } from "../src/mvp/forge-view";
import { setCodexLink } from "./ui/term";
import { ideasScreen, refillTime } from "./screens/ideas";
import { votePanel } from "./screens/vote";
import { pickScreen } from "./screens/pick";
import { card, formRich, roman, setCardCredits, unitSheet, type CardUnit } from "./ui/card";
import { isMine, reportLink, setReporter } from "./ui/report";
import { previewName } from "./ui/fusion";
import { fuseWarning } from "./ui/fuse-warn";
import { icon } from "./ui/icon";
import { actionable, app, button, closable, desktopQuery, dismissable, h, hasKeys, isDesktop, keepFocus, keepScreen, keyboardLost, keyLabel, keyOf, onKeys, onRestore, overlay, screen, show, who, withKey } from "./ui/dom";
import { loadUnitRates } from "./ui/unit-stats";
import { telegramSheet } from "./screens/telegram";
import { emailAttach, emailSignIn, emailSignUp } from "./screens/email";
import { initSound, music, onSoundChange, play, setSound, soundSettings } from "./ui/sound";
import { shopSound } from "./ui/sound-map";
import { t, withNodes } from "./i18n";
import { chooseLang, LANGS, uiLang } from "./lang";
import { applyTheme, chooseTheme, THEMES, uiTheme } from "./theme";

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
      // The cg build has no name screen: a new guest (#1218), once this call settles.
      if (CG) return void setTimeout(() => void cgBoot());
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
const HEART = "M12 20.5 4.2 13C2.6 11.4 2 10 2 8.5A5.5 5.5 0 0 1 12 5.2 5.5 5.5 0 0 1 22 8.5c0 1.5-.6 2.9-2.2 4.5z";
/** The run's hearts as drawn hearts (C-Shop): full ones red, lost ones dim. */
function heartIcons(n: number): HTMLElement {
  const box = h("span", { class: "hearts heart-icons", role: "img", "aria-label": t("run.hearts", { n }) });
  for (let i = 0; i < rules.hearts; i++) {
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("viewBox", "0 0 24 24");
    svg.setAttribute("class", i < n ? "heart full" : "heart lost");
    svg.setAttribute("aria-hidden", "true");
    const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
    path.setAttribute("d", HEART);
    svg.append(path);
    box.append(svg);
  }
  return box;
}
/** Desktop's run track (C-Shop): a dot per round, won green, lost red, this one big; the Crown at the end. */
function runTrack(run: RunView): HTMLElement {
  const last = new Map<number, string>();
  for (const f of run.fights) if (f.round <= rules.rounds) last.set(f.round, f.outcome);
  const dots = Array.from({ length: rules.rounds }, (_, i) => {
    const r = i + 1;
    // A round fought shows its result, even as the run ends on it (the fight's end card, M5-4).
    const cls = last.get(r) ?? (r === run.round ? "now" : r < run.round ? "draw" : "");
    return h("span", { class: `track-dot ${cls}` }, String(r));
  });
  const crown = h("span", { class: `track-dot crown${run.round > rules.rounds ? " now" : ""}`, "aria-hidden": "true" }, "👑");
  return h("span", { class: "track", "data-testid": "round", role: "img", "aria-label": roundLabel(run.round) }, ...dots, crown);
}
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

/** The language switch, beside Sound (M4-2): Русский / English, each in its
 * own words. Picking the other one keeps it on this device and redraws the
 * page in it. */
function langRow(): HTMLElement {
  const now = uiLang();
  const pick = (code: (typeof LANGS)[number]) => {
    const b = button(t(`lang.${code}`), () => code !== now && chooseLang(code), code === now ? "lang-on" : "", `lang-${code}`);
    b.setAttribute("aria-pressed", String(code === now));
    b.lang = code;
    return b;
  };
  return h("div", { class: "row lang-row", role: "group", "aria-label": t("lang.label"), "data-testid": "lang-row" }, ...LANGS.map(pick));
}

/** The palette switch, under the language (M5-1): Night Plum, Ink & Brass,
 * Felt Table, each with its swatch. Picking one keeps it on this device and
 * recolours the page at once. */
function themeRow(): HTMLElement {
  const pick = (code: (typeof THEMES)[number]) => {
    const swatch = h("span", { class: "swatch", "data-theme": code, "aria-hidden": "true" }, h("i", {}), h("i", {}), h("i", {}));
    const b = button("", () => {
      chooseTheme(code);
      for (const o of row.querySelectorAll("button")) {
        const on = o === b;
        o.classList.toggle("theme-on", on);
        o.setAttribute("aria-pressed", String(on));
      }
    }, code === uiTheme() ? "theme-on" : "", `theme-${code}`);
    b.append(swatch, h("span", {}, t(`theme.${code}`)));
    b.setAttribute("aria-pressed", String(code === uiTheme()));
    return b;
  };
  const row: HTMLElement = h("div", { class: "row theme-row", role: "group", "aria-label": t("theme.label"), "data-testid": "theme-row" }, ...THEMES.map(pick));
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

/** The Codex's How to play (M5-6): the rules, then how to read a card. It
 * replaces the Rules sheet and the shop's "Reading a card" sheet. */
const howTo = (): Node[] => [rulesPage(), legendPage(), ...(isDesktop() ? [keysPage()] : [])];

/** Desktop's keys, all in one place (Simplify: the keys sit on their buttons, How to play lists them). */
function keysPage(): HTMLElement {
  // The number keys in plain words: "1–6 buy the offer with that number".
  const grow = rules.offersGrowAt ?? [];
  const n = Math.min(7, grow.length ? offersAt(rules, Math.max(...grow)) : rules.offers);
  const numberKeys = n === 0 ? [] : n === 1 ? [t("keys.or"), kbd("1")] : [t("keys.or"), kbd("1"), "–", kbd(String(n))];
  const B = benchSizeOf(rules);
  return h(
    "div",
    { class: "stack" },
    h("div", { class: "label" }, t("keys.title")),
    h("div", { class: "keys", "data-testid": "keys" }, t("keys.lead"), ...numberKeys, t("keys.buys"), kbd("R"), t("keys.reroll"), kbd("L"), t("keys.lock"), kbd("Space"), t("keys.fight"), kbd("←"), kbd("→"), t("keys.move"), kbd("F"), t("keys.fuse"), kbd("S"), t("keys.sell"), ...(B > 0 ? [kbd("B"), t("keys.bench")] : []), kbd("M"), t("keys.sound"), kbd("Esc"), t("keys.menu")),
  );
}

/** The rules, from the numbers Home last saw. */
function rulesPage(): HTMLElement {
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
    ...(r.giftChoices ? [p(t(r.giftTierAhead ? "rules.giftAhead" : "rules.gift", { n: r.giftChoices }))] : []),
    p(t(desk ? "rules.fuseDesk" : "rules.fusePhone", growth)),
    h("div", { class: "label" }, t("rules.chainsLabel")),
    p(t(desk ? "rules.chainsDesk" : "rules.chainsPhone")),
    h("div", { class: "label" }, t("rules.dayLabel")),
    p(t("rules.day", { at: r.dayEndsAt })),
    p(t("rules.rating")),
  );
}

/** How to read a card: each number and mark, with a sample. */
function legendPage(): HTMLElement {
  const r = rules;
  const row = (sample: Node, text: string) => h("div", { class: "legend-row" }, h("div", { class: "legend-sample" }, sample), h("div", {}, text));
  const span = (cls: string, text: string) => h("span", { class: cls }, text);
  return h(
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
}

// ---------- name ----------

/** Arena is dark while rules v2 are built (#1044): a server that answers /health with `dark` (holding/server.mjs) has no game behind it, so this replaces every screen. */
function holdingScreen(): void {
  music("home");
  show(
    h("h1", {}, t("name.title")),
    h("h2", { "data-testid": "holding-title" }, t("holding.title")),
    h("p", { class: "dim", "data-testid": "holding-lead" }, t("holding.lead")),
    h("ul", { class: "holding-list", "data-testid": "holding-list" }, h("li", {}, t("holding.units")), h("li", {}, t("holding.fusions")), h("li", {}, t("holding.codex"))),
    langRow(),
  );
}

function nameScreen(): void {
  music("home");
  // An invite-only server (slice 13) takes no new names: a player comes from
  // their invite link. One open to all joins them as the open link does. Nothing but the title shows until /health says which.
  show(h("h1", {}, t("name.title")));
  void api.health().then(
    (hl) => {
      const tg = telegramLogin(hl.telegram);
      // #929: with email login on, an invite-only server signs up and in by email.
      if (hl.invites && hl.mail) {
        if (hl.open) emailSignUp({ join: "", extra: tg, onDone: signedIn });
        else
          show(
            h("h1", {}, t("name.title")),
            h("p", { class: "dim", "data-testid": "invite-only" }, t("name.inviteOnly")),
            button(t("email.signIn"), () => emailSignIn({ back: nameScreen, onDone: signedIn }), "primary", "email-signin-open"),
            tg,
          );
      } else if (hl.open) joinForm("", tg);
      else if (hl.invites) show(h("h1", {}, t("name.title")), h("p", { class: "dim", "data-testid": "invite-only" }, t("name.inviteOnly")), tg);
      else nameForm(tg, !!hl.mail);
    },
    () => nameForm(),
  );
}

/** #929: this device just became a player by email: Home, with no join code left in the address. */
const signedIn = () => (dropInvite(), void homeScreen());

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

function nameForm(tg: HTMLElement | null = null, mail = false): void {
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
    mail ? button(t("email.signIn"), () => emailSignIn({ back: nameScreen, onDone: signedIn }), "", "email-signin-open") : null,
    tg,
    err,
  );
  input.focus();
}

// ---------- home ----------

/** Home is the Throne (mission 5, M5-3, canvas C-Throne and the Simplify
 * map): the champion of the day as the hero, one big Play (or Continue, with
 * New run under it while a run waits), the idea meter (it opens My ideas),
 * your last run, then two quiet links, Codex and Champions (Stats). Sound,
 * language, palette, Rules, your link and Telegram sit behind the gear;
 * "End day now" shows on dev servers only.
 * `ended`: the day the dev "End day now" just closed, so Home says how it
 * ended at the top and brings the playoff panel into view. */
async function homeScreen(ended: number | null = null): Promise<void> {
  const err = errorLine();
  void loadUnitRates(); // the rates hint on unit sheets (slice 11)
  const [home, content, health]: [HomeView, MvpContent, { codex?: boolean }] = await Promise.all([api.home(), getContent(), api.health().catch(() => ({}))]);
  codexOn = !!health.codex;
  // #929: a player from before email login adds an email before playing.
  // Never in the CrazyGames build (#1218): its players have no email.
  if (!CG && home.email?.required && !home.email.address) return emailAttach({ playerName: api.player?.name ?? "", onDone: () => void homeScreen() });
  rules = home.rules;
  day = home.day;
  // The run that waits, for Continue's round and hearts (null if it ended meanwhile).
  const waiting = home.activeRunId ? await api.run(home.activeRunId).catch(() => null) : null;
  const active = waiting && waiting.phase !== "over" ? waiting : null;
  const champ = home.day.champion;
  const r = home.rating;
  const own = champ?.player.id === api.player?.id;
  const play = active
    ? button("", () => void guarded(err, async () => shopScreen(await api.run(active.runId), content)), "primary", "play")
    : button("", () => void guarded(err, async () => shopScreen(await api.startRun(), content)), "primary", "play");
  play.classList.add("play-big");
  play.append(
    active
      ? h("span", { class: "play-main" }, t("home.continue", { where: active.round > rules.rounds ? t("home.continueCrown") : t("home.continueRound", { round: active.round }) }), hearts(active.hearts))
      : h("span", { class: "play-main" }, t("home.play")),
  );
  if (isDesktop()) play.append(h("kbd", { class: "play-key" }, "Enter"));
  if (!active) play.append(h("span", { class: "play-sub" }, t(!champ ? "home.playSubNone" : own ? "home.playSubOwn" : "home.playSub", { n: rules.rounds })));
  const newRun = active
    ? button(t("home.newRun"), () => abandonSheet(active, "new", () => void guarded(err, async () => {
        // The given-up run's end and rating change first, as ☰ Abandon shows
        // them; its "New run" starts the next one.
        runOverScreen(await api.abandon(active.runId), content, "", true);
      })), "small link", "new-run")
    : null;
  const codex = withKey(button(t("menu.codex"), () => void guarded(err, () => openCodex()), "throne-link", "codex"), "C");
  const stats = withKey(button(t("home.stats"), () => void guarded(err, () => openCodex({ tab: "champions" })), "throne-link", "stats"), "S");
  const forge = codexOn ? withKey(button(t("menu.forge"), () => void guarded(err, () => openForge()), "throne-link", "forge-open"), "F") : null;
  const words = codexOn ? withKey(button(t("menu.dictionary"), () => void guarded(err, () => openForge("words")), "throne-link", "dictionary-open"), "D") : null;
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
  const rating = h("span", { class: "num", "data-testid": "rating" }, `${r?.rating ?? rules.ratingStart}`);
  const pill = h("div", { class: "rating-pill keep", title: t("home.ratingAria") }, rating);
  const gear = button("", () => settingsSheet(home, err), "gear keep", "settings");
  gear.setAttribute("aria-label", t("home.settings"));
  if (isDesktop()) gear.title = `${t("home.settings")} (G)`;
  gear.append(GEAR_SVG());
  show(
    h("header", { class: "row throne-top" }, h("h1", {}, t("home.title")), h("div", { class: "grow" }), pill, gear),
    // The dev "End day now" just ran: say so first, with how the day ended
    // (the playoff panel) right under it, before today's champion.
    // Desktop: the champion and the day on the left, Play and the rest on the
    // right (.home-main, .home-side; on the phone they are one column).
    h(
      "div",
      { class: "home-main" },
      ended !== null ? h("div", { class: "notice", "data-testid": "day-ended" }, t("home.dayEnded", { ended, today: home.day.seq })) : null,
      justEnded ? playoff : null,
      h(
        "section",
        { class: "stack champion throne-hero", "data-testid": "champion" },
        h("div", { class: "crown-badge", "aria-hidden": "true" }, CROWN_SVG()),
        h("div", { class: "hero-label" }, t("home.champion")),
        champ ? h("h2", { class: "hero-name" }, whoMark(champ.player)) : h("div", {}, t("home.noChampion")),
        champ && !champ.player.bot && !isMine(champ.player.id) ? reportLink("name", champ.player.id, `@${champ.player.name}`) : null,
        own ? h("div", { class: "hero-day", "data-testid": "own-team" }, t("home.ownTeam")) : null,
        h("div", { class: "hero-day", "data-testid": "slayers" }, champ ? slayersLine(home.day.slayers) : t("home.newChampionAt", { at: rules.dayEndsAt })),
        champ ? h("div", { class: "hero-team" }, team(champ.line, "ghost", content)) : null,
      ),
      justEnded ? null : playoff,
    ),
    h(
      "div",
      { class: "home-side" },
      h(
        "div",
        { class: "stack title-menu", "data-testid": "home-actions" },
        play,
        newRun,
        err,
        home.ideas ? ideasMeter(home.ideas) : null,
        lastRunPanel(r?.rating ?? null, pill),
      ),
      h("div", { class: "spacer" }),
      h("nav", { class: "throne-nav" }, codex, forge, words, stats),
      home.dev
        ? h("details", { class: "dev" }, h("summary", {}, t("dev.title")), h("div", { class: "row wrap" }, endDay, grantIdea, creditUnit, seedCandidate, overnight, fakeVotes, candidates))
        : null,
      // M2-8's vote card, under the menu: quiet, and Play stays on the first screen.
      votePanel(content),
    ),
  );
  screen("home");
  music("home");
  if (justEnded) playoff?.classList.add("fresh");
  // Keys (#1064): Enter plays, and a letter on each link (it is on the button).
  const press = (id: string) => document.querySelector<HTMLButtonElement>(`[data-testid="${id}"]`);
  const letters: Record<string, string> = { c: "codex", f: "forge-open", d: "dictionary-open", s: "stats", i: "ideas", g: "settings" };
  onKeys((e) => {
    const k = keyOf(e);
    const b = k === "Enter" ? press("play") : press(letters[k] ?? "");
    if (!b || b.disabled) return false;
    b.click();
    return true;
  });
  if (isDesktop() && (document.activeElement === document.body || !app.contains(document.activeElement))) play.focus({ preventScroll: true });
}

const svg = (paths: string, attrs: Record<string, string>) => () => {
  const el = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  for (const [k, v] of Object.entries({ viewBox: "0 0 24 24", "aria-hidden": "true", ...attrs })) el.setAttribute(k, v);
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  path.setAttribute("d", paths);
  el.append(path);
  return el;
};
const GEAR_SVG = svg("M9 12a3 3 0 1 0 6 0a3 3 0 1 0-6 0M12 2v3M12 19v3M4.2 4.2l2.1 2.1M17.7 17.7l2.1 2.1M2 12h3M19 12h3M4.2 19.8l2.1-2.1M17.7 6.3l2.1-2.1", { width: "20", height: "20", fill: "none", stroke: "currentColor", "stroke-width": "2.4", "stroke-linecap": "round" });
const CROWN_SVG = svg("M3 18h18L19 7l-5 4-2-6-2 6-5-4z", { width: "30", height: "30", fill: "var(--paper)", stroke: "var(--outline)", "stroke-width": "2.2", "stroke-linejoin": "round" });

/** The gear's sheet (M5-3): everything that was the title menu's second rank.
 * Sound and Music, the language, the palette, Rules (until the Codex's How to
 * play), this device's link and Telegram. */
function settingsSheet(home: HomeView, err: HTMLElement): void {
  const close = closable(
    h(
      "div",
      { class: "stack settings", "data-testid": "settings-sheet" },
      h("h2", {}, t("home.settings")),
      h("div", { class: "row me" }, h("span", { class: "dim keep" }, t("home.playingAs")), who(api.player?.name ?? "")),
      home.email?.address ? h("div", { class: "row me", "data-testid": "settings-email" }, h("span", { class: "dim keep" }, t("email.label")), h("span", { class: "grow" }, home.email.address)) : null,
      soundRow(),
      langRow(),
      themeRow(),
      button(t("codex.tab.howto"), () => (close(), void guarded(err, () => openCodex({ tab: "howto" }))), "", "rules-open"),
      api.ownInvite && !CG ? ownLinkRow(api.ownInvite) : null,
      home.telegram?.enabled ? telegramRow(home.telegram.linked, err) : null,
    ),
  );
}

/** The Throne's idea meter (M2-3, M5-3; #915 ideas a day): the ideas left
 * today and how many come back at the day's end, or one waiting for its pick
 * first. Tapping it opens My ideas (M2-4, screens/ideas.ts). */
function ideasMeter(ideas: IdeasView): HTMLElement {
  // M2-6: an idea waiting for its pick comes first: "💡 Your idea is ready".
  const label = ideas.ready ? t("home.ideasReady", { n: ideas.ready }) : ideas.held > 0 ? t("home.ideasHeld", { n: ideas.held }) : t("home.ideasNone");
  const b = button("", () => void ideasScreen({ onBack: () => void homeScreen(), onUnknown: () => (api.forget(), nameScreen()) }), "idea-meter", "ideas");
  b.append(h("span", { class: "grow idea-label", "data-testid": "ideas-label" }, label));
  if (!ideas.ready && ideas.held < ideas.perDay)
    b.append(h("span", { class: "idea-runs", "data-testid": "ideas-refill" }, t("home.ideaRefill", { n: ideas.perDay - ideas.held, time: refillTime(ideas.refillsAt) })));
  if (ideas.ready) b.dataset.ready = String(ideas.ready);
  return withKey(b, "I");
}

/** This device's last finished run (M5-3): the run-over screen keeps its id,
 * the Throne reads it back. Per device, like the palette; null if none. */
const LAST_RUN_KEY = "arena.lastRun";
function rememberLastRun(run: RunView): void {
  try {
    localStorage.setItem(LAST_RUN_KEY, JSON.stringify({ player: run.player.id, runId: run.runId }));
  } catch {
    // Private mode: the Throne just shows no last run.
  }
}
function lastRunId(): string | null {
  try {
    const saved = JSON.parse(localStorage.getItem(LAST_RUN_KEY) ?? "null") as { player?: string; runId?: string } | null;
    return saved && saved.player === api.player?.id && saved.runId ? saved.runId : null;
  } catch {
    return null;
  }
}

/** The Throne's "Last run": a pip per fight (won, drawn, lost; the Crown a
 * crown), how it ended and the record. It fills in when the run arrives; the
 * rating pill then shows the run's change when it is the latest one. */
function lastRunPanel(rating: number | null, pill: HTMLElement): HTMLElement | null {
  const id = lastRunId();
  if (!id) return null;
  const panel = h("section", { class: "stack last-run", "data-testid": "last-run", hidden: "" });
  void api.run(id).then(
    (run) => {
      if (run.phase !== "over") return panel.remove();
      const draws = run.fights.filter((f) => f.outcome === "draw").length;
      panel.append(
        h("div", { class: "display" }, t("home.lastRun")),
        h("div", { class: "fight-pips" }, ...run.fights.map((f) => h("span", { class: `fpip ${f.outcome}`, title: f.outcome }, f.kind === "crown" ? "👑" : String(f.round)))),
        h("div", { class: "small" }, runEnd(run).why),
        h("div", { class: "small dim num" }, t("over.record", { wins: t("over.wins", { n: run.wins }), draws: t("over.draws", { n: draws }), losses: t("over.losses", { n: run.losses }) })),
      );
      panel.hidden = false;
      const rc = run.rating;
      if (rc && rc.after === rating && rc.after !== rc.before) {
        const delta = rc.after - rc.before;
        pill.append(h("span", { class: `delta ${delta > 0 ? "up" : "down"}`, "data-testid": "rating-delta" }, `${delta > 0 ? "+" : "−"}${Math.abs(delta)}`));
      }
    },
    () => panel.remove(),
  );
  return panel;
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
      withKey(button(t("abandon.cancel"), () => close(), "grow", "abandon-cancel"), "Esc"),
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
  codexRedraw = () => void codexScreen({ content, state: { ...codexCache.state, term: undefined, scope: undefined }, onBack, onUnknown, cache: codexCache, howTo });
  await codexScreen({ content, state, onBack, onUnknown, cache: codexCache, howTo });
}
/** Rules v2's Codex (#1051): unit pages with their partners, Today, the
 * Dictionary, read from GET /codex. It opens over the current screen, as
 * the Codex does; a wish redraws it from the server. `codexOn`: whether the
 * server runs the Codex (/health), so its links show only then. */
let codexOn = false;
void api.health().then((h) => (codexOn = !!h.codex), () => undefined);
setReporter((r) => api.report(r), () => api.player?.id ?? savedPlayer()?.id ?? "");
async function openForge(tab: ForgeTab = "units"): Promise<void> {
  const back = keepScreen();
  let view = await api.codex();
  let unit: string | undefined;
  const draw = (at: ForgeTab) =>
    forgeScreen({
      view,
      tab: at,
      unit,
      onBack: back,
      onWish: async (p, _on, state) => {
        unit = state.unit;
        await api.wish(p.a, p.b);
        view = await api.codex();
        draw(state.tab);
      },
    });
  draw(tab);
}
// Every highlighted term's "Open in Codex" lands on its Keywords row (a scoped trigger's line in it).
setCodexLink((term, scope) => void openCodex({ tab: "keywords", term, scope }));

/** The browser's Back or Forward onto a screen that is gone (#1146): it is
 * built again from the server, as its own button would. The run's group is
 * the run that waits (the shop), else the last one's end. */
onRestore((group) =>
  (async () => {
    try {
      if (group === "codex") return await openCodex();
      if (group === "forge") return await openForge();
      if (group === "ideas") return await ideasScreen({ onBack: () => void homeScreen(), onUnknown: () => (api.forget(), nameScreen()) });
      if (group === "run") {
        const id = (await api.home()).activeRunId ?? lastRunId();
        if (id) {
          const [run, content] = [await api.run(id), await getContent()];
          return run.phase === "over" ? runOverScreen(run, content) : shopScreen(run, content);
        }
      }
      return await homeScreen();
    } catch (e) {
      // A screen that can't be rebuilt (offline, a gone run): Home is always there.
      console.warn("restore", group, e);
      await homeScreen().catch(() => undefined);
    }
  })(),
);

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
  const forge = codexOn ? button(t("menu.forge"), () => (close(), void guarded(err, () => openForge())), "", "menu-forge") : null;
  const menu = h(
    "div",
    { class: "stack run-menu", "data-testid": "run-menu" },
    h("div", { class: "label" }, run.phase === "over" ? t("menu.runOver") : crown ? t("menu.runCrown") : t("menu.runRound", { round, rounds: rules.rounds })),
    // Over a battle that was playing, Resume plays on (R2-17 batch F).
    withKey(button(t("menu.resume"), () => (close(), resume?.()), "primary", "menu-resume"), "Esc"),
    codex,
    forge,
    soundRow(),
    langRow(),
    themeRow(),
    button(t("menu.titleMenu"), () => (close(), void guarded(err, () => homeScreen())), "", "menu-title"),
    run.phase === "over" ? null : h("div", { class: "dim small" }, t("menu.waits")),
    run.phase === "over" ? null : button(t("menu.abandon"), () => (close(), abandonSheet(run, "menu", () => void guarded(err, async () => runOverScreen(await api.abandon(run.runId), content)))), "danger", "menu-abandon"),
  );
  // A tap outside the menu, or Esc, is a Resume too.
  const close = dismissable((close) => (close(), resume?.()), menu);
}

/** The champion card's last line: today's slayers, and when the day ends. */
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

/** A forge scene's pair to open in the next shop, and a button to click once it shows (#1049). */
let forgeScene: [number, number, string | undefined] | null = null;

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
      if (res.fight) return fightScreens(res.run, res.fight, content, run);
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
  const hintSlot = h("div", { class: "hint-slot" });
  const awoken = board.filter((u) => u.kind === "unit" && u.form === "awoken").length;
  // Desktop: the inspector, the card under the mouse, an offer clicked, a fusion waiting for its Fuse.
  const inspector = h("aside", { class: "inspector stack", "data-testid": "inspector" });
  type At = { kind: "line"; index: number } | { kind: "offer"; slot: number };
  let hover: At | null = null;
  let chosen: number | null = desk && run.offers[offer] ? offer : null;
  let fuseView: HTMLElement | null = null;
  // Rules v2 (#1049 reopened): with the Codex on, two Awoken units fuse only
  // when their pair has a design (discovered or ready to claim). The run's
  // forge says which board pairs have one and which offers bring one within
  // reach; until it answers, nothing hints at a fuse. Codex off: v1's rule.
  let forge: RunForge | null = null;
  const awokenAt = (slot: number) => {
    const u = unitAt(slot);
    return !!u && u.kind === "unit" && u.form === "awoken";
  };
  const designedPairs = (slot: number) => (forge?.pairs ?? []).filter((p) => p.state !== "unforged" && (p.first === slot || p.second === slot));
  const canFuse = (slot: number) => awokenAt(slot) && !run.gift && (codexOn ? designedPairs(slot).length > 0 : awoken >= 2);
  const fusesWith = (first: number, slot: number) => awokenAt(slot) && (!codexOn || designedPairs(first).some((p) => p.first === slot || p.second === slot));
  const anyFuse = () => !run.gift && (codexOn ? (forge?.pairs ?? []).some((p) => p.state !== "unforged") : awoken >= 2);
  /** A designed pair's fusion as a mark says it: its name once discovered, else "ready to claim". */
  const fusionName = (p: { state: string; design?: { name: string; emoji?: string; texts?: { ru?: { name?: string } } } }) =>
    p.design ? `${p.design.emoji ?? ""} ${(uiLang() === "ru" ? p.design.texts?.ru?.name : undefined) ?? p.design.name}`.trim() : t("fuse.toClaim");
  const nameOf = (id: string) => unitOf(id)?.name ?? id;
  /** The mark on a board unit of a fusion you can make now: ⚗, the fusions in its label. */
  const fuseMark = (pairs: ForgeSlot[], slot: number) => {
    const list = pairs.map((p) => `${nameOf(unitAt(p.first === slot ? p.second : p.first)!.unitId)} → ${fusionName(p)}`).join("; ");
    return h("span", { class: "fuse-mark", "data-testid": `fuse-mark-${slot}`, title: t("fuse.mark", { list }), "aria-label": t("fuse.mark", { list }) }, "⚗");
  };
  /** What an offer brings within reach (the shop card's mark and its sheet's line), best first: one buy away before close. */
  const offerFusions = (slot: number): ForgeOffer[] => (forge?.offers ?? []).filter((f) => f.slot === slot).sort((x, y) => Number(y.awakens) - Number(x.awakens));
  const offerLine = (f: ForgeOffer) => {
    const partner = unitAt(f.partner);
    const pair = `${partner ? `${partner.emoji} ${partner.name}` : "?"} + ${nameOf(f.a === partner?.unitId ? f.b : f.a)}`;
    return t(f.awakens ? "fuse.offerNow" : "fuse.offerNear", { pair, fusion: fusionName(f) });
  };

  /** Desktop: the keyboard goes to the fusion in the inspector (its primary button), as it does into a sheet. */
  const focusPreview = () => queueMicrotask(() => (fuseView?.querySelector<HTMLElement>("button.primary:not(:disabled)") ?? fuseView?.querySelector<HTMLElement>("button:not(:disabled)"))?.focus({ preventScroll: true }));

  /** Where a unit at board slot `from` of run `r` lands when dropped on `to`, or null for no move. */
  const destFor = (r: RunView, from: number, to: number): number | null => {
    if (from === to) return null;
    const lineLen = r.line.length;
    const benchLen = r.bench.length;
    const occupied = to < L ? to < lineLen : to - L < benchLen;
    const across = (from < L) !== (to < L);
    const start = to < L ? 0 : L;
    const len = to < L ? lineLen : benchLen;
    // Within a zone, a drop past its last unit moves to the end; across
    // zones, an empty slot takes the unit at that zone's end (packed).
    const dest = across ? (occupied ? to : start + len) : Math.min(to, start + len - 1);
    return dest === from ? null : dest;
  };
  /** Move the unit at board slot `from` to `to` (a reorder), keeping it selected where it lands. */
  const moveTo = (from: number, to: number) => {
    const dest = destFor(run, from, to);
    if (dest !== null) void decide({ kind: "reorder", from, to: dest }, dest);
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
      if (!crown) dropTarget(e, slot);
      return e;
    }
    const c = actionable(card(u, { side: "you", extra: [copiesBadge(u)], testid: id }), `${u.name}, ${onBench ? t("board.onBench", { n: slot - L + 1 }) : t("board.inLine", { n: slot + 1 })}`);
    if (pick.mode === "picked" && pick.index === slot) c.classList.add("selected");
    if (pick.mode === "fuse") {
      if (pick.first === slot) c.classList.add("selected");
      else if (fusesWith(pick.first, slot)) c.classList.add("fusable");
      else c.classList.add("muted");
    } else if (codexOn && !crown && canFuse(slot)) {
      // A fusion you can make now: both units of a designed pair wear the mark.
      c.classList.add("fuse-ready");
      c.append(fuseMark(designedPairs(slot), slot));
    }
    c.addEventListener("click", () => {
      play("click");
      if (pick.mode === "fuse") {
        if (pick.first === slot) pick = { mode: "none" };
        else if (fusesWith(pick.first, slot)) return void fusePreview(pick.first, slot);
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
    if (!crown) {
      dropTarget(c, slot);
      dragCard(c, { kind: "board", slot });
    }
    return c;
  };

  // A redraw replaces the cards: one that lands under a press or a drag (the
  // forge answering late) waits until it ends, or the pointer loses its card.
  let pressing = false;
  let redrawLater = false;
  const renderLine = () => (pressing ? void (redrawLater = true) : keepFocus(drawLine));
  const drawLine = () => {
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

  /** Desktop: hovering a card reads it in the inspector; the keyboard reads a card too. */
  const desktopCard = (c: HTMLElement, at: At) => {
    c.addEventListener("mouseenter", () => ((hover = at), renderInspector()));
    c.addEventListener("mouseleave", () => ((hover = null), renderInspector()));
    // The keyboard reads a card as the mouse does (#1064): Tab to it and the inspector shows it.
    c.addEventListener("focus", () => ((hover = at), renderInspector()));
    c.addEventListener("blur", () => ((hover = null), renderInspector()));
  };

  /** What a drag carries: a board unit (by slot) or a shop offer (by its slot). */
  type Drag = { kind: "board"; slot: number } | { kind: "offer"; offer: Offer };
  /** Drag with pointer events, at every size (mouse and touch alike): a move
   * past ~6px starts it, a shorter one stays a tap. The card follows the
   * pointer; the slots it can take light up; a sell zone appears under it. */
  const dragCard = (c: HTMLElement, src: Drag) => {
    c.classList.add("draggable");
    c.addEventListener("pointerdown", (down) => {
      if (!down.isPrimary || (down.pointerType === "mouse" && down.button !== 0)) return;
      const x0 = down.clientX;
      const y0 = down.clientY;
      let ghost: HTMLElement | null = null;
      let sellZone: HTMLElement | null = null;
      let over: HTMLElement | null = null;
      const targets = () => [...line.querySelectorAll<HTMLElement>("[data-drop-slot]"), ...bench.querySelectorAll<HTMLElement>("[data-drop-slot]")];
      const at = (e: PointerEvent) => document.elementsFromPoint(e.clientX, e.clientY).find((el): el is HTMLElement => el instanceof HTMLElement && (el.dataset.dropSlot !== undefined || el.dataset.dropSell !== undefined));
      const place = (e: PointerEvent) => {
        const r = c.getBoundingClientRect();
        ghost!.style.transform = `translate(${e.clientX - r.width / 2}px, ${e.clientY - r.height / 2}px)`;
      };
      const finish = () => {
        pressing = false;
        c.removeEventListener("pointermove", move);
        c.removeEventListener("pointerup", up);
        c.removeEventListener("pointercancel", cancel);
        window.removeEventListener("keydown", esc, true);
        try { c.releasePointerCapture(down.pointerId); } catch {}
        ghost?.remove();
        sellZone?.remove();
        c.classList.remove("dragging");
        app.classList.remove("drag-on");
        for (const el of targets()) el.classList.remove("drop-ok", "drop");
        if (ghost) c.addEventListener("click", (e) => (e.stopImmediatePropagation(), e.preventDefault()), { capture: true, once: true });
        if (redrawLater) ((redrawLater = false), renderLine());
      };
      const move = (e: PointerEvent) => {
        if (!ghost) {
          if (Math.hypot(e.clientX - x0, e.clientY - y0) < 6) return;
          const r = c.getBoundingClientRect();
          ghost = c.cloneNode(true) as HTMLElement;
          ghost.removeAttribute("data-testid");
          ghost.className = `${c.className} drag-ghost`;
          ghost.style.width = `${r.width}px`;
          ghost.style.height = `${r.height}px`;
          document.body.append(ghost);
          c.classList.add("dragging");
          app.classList.add("drag-on");
          const value = src.kind === "board" ? sellValue(rules, unitAt(src.slot)!) : 0;
          if (src.kind === "board") {
            sellZone = h("div", { class: "sell-zone", "data-drop-sell": "1", "data-testid": "sell-zone" }, t("board.sell", { n: value }));
            document.body.append(sellZone);
          }
          for (const el of targets()) if (src.kind !== "board" || Number(el.dataset.dropSlot) !== src.slot) el.classList.add("drop-ok");
          sellZone?.classList.add("drop-ok");
        }
        place(e);
        const now = at(e);
        if (now !== over) {
          over?.classList.remove("drop");
          over = now ?? null;
          over?.classList.add("drop");
        }
      };
      const up = (e: PointerEvent) => {
        const hit = ghost ? at(e) : undefined;
        finish();
        if (!hit) return;
        if (hit.dataset.dropSell !== undefined) {
          if (src.kind === "board") void decide({ kind: "sell", index: src.slot });
          return;
        }
        const to = Number(hit.dataset.dropSlot);
        if (src.kind === "board") moveTo(src.slot, to);
        else buyAt(src.offer, to);
      };
      const cancel = () => finish();
      const esc = (e: KeyboardEvent) => e.key === "Escape" && (e.stopPropagation(), finish());
      pressing = true;
      c.setPointerCapture(down.pointerId);
      c.addEventListener("pointermove", move);
      c.addEventListener("pointerup", up);
      c.addEventListener("pointercancel", cancel);
      window.addEventListener("keydown", esc, true);
    });
  };
  /** A card or an empty slot of the line or the bench takes a dragged unit or offer. */
  const dropTarget = (c: HTMLElement, slot: number) => {
    c.dataset.dropSlot = String(slot);
  };
  /** An offer dropped on a slot: buy it, then move the new unit there (a twin merges where it stands). */
  const buyAt = (o: Offer, to: number) => {
    const why = buyBlock(o);
    if (why) return void ((err.textContent = why), play("wrong"));
    const d = { kind: "buy", slot: o.slot } as const;
    void guarded(err, async () => {
      await Promise.allSettled([...previewsOut]);
      let res = await api.decide(run.runId, d);
      if (res.fight) return fightScreens(res.run, res.fight, content, run);
      const had = new Set(board.map((u) => u.uid));
      const now = [...res.run.line, ...res.run.bench];
      const i = now.findIndex((u) => !had.has(u.uid));
      let sel = -1;
      if (i >= 0) {
        const from = i < res.run.line.length ? i : L + i - res.run.line.length;
        const dest = destFor(res.run, from, to);
        if (dest !== null) {
          res = await api.decide(run.runId, { kind: "reorder", from, to: dest });
          sel = dest;
        }
      }
      play(shopSound(d, run, res.run));
      shopScreen(res.run, content, "", sel, -1);
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
    if (canFuse(i)) out.push(button(desk ? t("board.fuseDesk") : t("board.fuse"), () => ((pick = { mode: "fuse", first: i }), renderLine()), "", "fuse"));
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
    if (anyFuse()) return hint(desk ? t("hint.fuseDesk") : t("hint.fusePhone"));
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
      // Rules v2: with the Codex on, the pair's forge decides (a 404: v1's splice preview).
      // Rules v2 never previews a splice: only a pair the forge lists fuses.
      const now = await api.forge(run.runId).catch((e: unknown) => {
        if (e instanceof ApiError && e.status === 404) return null;
        throw e;
      });
      if (now) {
        const slot = now.pairs.find((p) => (p.first === first && p.second === second) || (p.first === second && p.second === first));
        return slot ? forgeSheet(slot) : undefined;
      }
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
            withKey(button(t("fuse.cancel"), () => close(), "grow", "preview-cancel"), "Esc"),
            button(t("fuse.swap"), () => keepFocus(() => ((at = 1 - at), render())), "", "preview-swap"),
            withKey(button(t("fuse.confirm"), () => (close(), void fuse(o, views[at]!)), "primary grow", "preview-confirm"), "Enter"),
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
      if (desk) focusPreview();
    });

  /** Rules v2's forge (#1049, server/src/mvp/codex.ts): the pair's Codex
   * state decides the sheet. Discovered: its canonical design and credit,
   * Fuse. Ready to claim: the three sealed designs as cards; Claim one and it
   * becomes the pair's fusion for everyone, with your name. Unforged: Wish,
   * which puts the pair on tonight's list. */
  const forgeSheet = async (slot: ForgeSlot) => {
    const o = { first: slot.first, second: slot.second };
    const names = { a: unitAt(o.first)!.name, b: unitAt(o.second)!.name };
    const fusedOf = (res: { run: RunView }) => [...res.run.line, ...res.run.bench].find((u) => u.uid === unitAt(o.first)!.uid)!;
    const head = h("div", { class: "label" }, t(`forge.${slot.state}`));
    const body = h("div", { class: "stack", "data-testid": `forge-${slot.state}` });
    let close: () => void = () => {};
    const cancel = () => withKey(button(t("fuse.cancel"), () => close(), "grow", "preview-cancel"), "Esc");
    if (slot.state === "discovered") {
      const fused = fusedOf(await api.preview(run.runId, { kind: "fuse", ...o }));
      const date = slot.claimedAt ? new Date(slot.claimedAt).toLocaleDateString(uiLang()) : "";
      body.replaceChildren(
        h("div", { class: "preview-card" }, card(fused, { side: "you", extra: [copiesBadge(fused)] })),
        h("div", { class: "dim small", "data-testid": "forge-credit" }, t("forge.by", { name: slot.claimedBy?.name ?? "?", date })),
        h("div", { class: "row sheet-actions" }, cancel(), withKey(button(t("fuse.confirm"), () => (close(), void fuse(o, fused)), "primary grow", "preview-confirm"), "Enter")),
        unitSheet(fused, content, { preview: true }),
      );
    } else if (slot.state === "ready") {
      const designs = await Promise.all((slot.candidates ?? []).map((_, pick) => api.preview(run.runId, { kind: "fuse", ...o, pick }).then(fusedOf)));
      let at = 0;
      const render = () => {
        const chosen = designs[at]!;
        const flavour = (i: number) => {
          const d = slot.candidates![i]!;
          return (uiLang() === "ru" ? d.texts?.ru?.flavour : undefined) ?? d.flavour ?? "";
        };
        body.replaceChildren(
          h("div", { class: "dim small" }, t("forge.sealed", { n: designs.length })),
          h(
            "div",
            { class: "seal-list", "data-testid": "forge-cards" },
            // #1120: one row per design, the card beside its rule (the sentence
            // the Codex says), the flavour small under it: pick by the mechanic
            ...designs.map((u, i) => {
              const words = h("span", { class: "seal-words" }, h("b", { class: "seal-name" }, u.name), h("span", { class: "seal-rule", "data-testid": `forge-rule-${i}` }, ...(u.recipe ? formRich(u.recipe, content) : [])), h("span", { class: "seal-flavour" }, flavour(i)));
              const b = h("button", { class: `seal${i === at ? " on" : ""}`, "data-testid": `forge-card-${i}`, "aria-pressed": String(i === at) }, card(u, { side: "you" }), words);
              b.addEventListener("click", () => keepFocus(() => ((at = i), render())));
              return withKey(b, String(i + 1));
            }),
          ),
          h("div", { class: "row sheet-actions" }, cancel(), withKey(button(t("forge.claim", { name: chosen.name }), () => (close(), void fuse({ ...o, pick: at }, chosen, slot)), "primary grow", "forge-claim"), "Enter")),
          unitSheet(chosen, content, { preview: true }),
        );
      };
      render();
    } else {
      const render = (view: ForgeView) =>
        body.replaceChildren(
          h("div", { class: "dim small" }, t("forge.notYet", names)),
          h("div", { class: "small", "data-testid": "forge-wishes" }, t("forge.wishes", { n: view.wishes })),
          h(
            "div",
            { class: "row sheet-actions" },
            cancel(),
            view.wished
              ? h("button", { class: "grow", disabled: "", "data-testid": "forge-wished" }, t("forge.wished"))
              : button(t("forge.wish"), () => void guarded(err, async () => render(await api.wish(slot.a, slot.b))), "primary grow", "forge-wish"),
          ),
        );
      render(slot);
    }
    // Keys (#1064): 1–3 look at the sealed design with that number.
    body.addEventListener("keydown", (e) => {
      if (slot.state !== "ready" || e.ctrlKey || e.metaKey || e.altKey || !/^[1-3]$/.test(keyOf(e))) return;
      const b = body.querySelector<HTMLButtonElement>(`[data-testid="forge-card-${Number(keyOf(e)) - 1}"]`);
      if (b) (e.preventDefault(), e.stopPropagation(), b.click());
    });
    pick = { mode: "none" };
    if (desk) {
      fuseView = h("div", { class: "stack", "data-testid": "fuse-preview" }, head, body);
      close = () => ((fuseView = null), renderLine());
    } else close = overlay(head, body);
    renderLine();
    if (desk) focusPreview();
  };

  /** Confirm a fuse; a pair nobody had fused (or only bots had) then shows
   * its name with "You discovered". A claim (rules v2, `claim`: the ready
   * pair's forge) shows the claim moment: the picked design with your name
   * on it, the other two sealed away (#1051); Back returns to the shop. */
  const fuse = (o: { first: number; second: number; pick?: number }, previewed: LineUnit, claim?: ForgeSlot) =>
    guarded(err, async () => {
      const res = await api.decide(run.runId, { kind: "fuse", ...o });
      play(shopSound({ kind: "fuse", ...o }, run, res.run));
      const claimed = !!claim;
      if (claim?.candidates && o.pick !== undefined) {
        const parts: [UnitContent | undefined, UnitContent | undefined] = [unitIn(content, claim.a), unitIn(content, claim.b)];
        play("discover");
        return claimScreen({
          claim: { pair: [claim.a, claim.b], candidates: claim.candidates.map((d) => forgeDesignOf(d, parts, content)) },
          units: parts.filter((u): u is UnitContent => !!u).map((u) => forgeUnitOf(u, content)),
          viewer: api.player?.name ?? savedPlayer()?.name ?? "",
          picked: o.pick,
          onDone: () => shopScreen(res.run, content),
        });
      }
      shopScreen(res.run, content);
      const fused = [...res.run.line, ...res.run.bench].find((u) => u.uid === unitAt(o.first)!.uid);
      // A scene's player lives only in api.player, never saved.
      const me = api.player?.id ?? savedPlayer()?.id;
      const discovered = claimed || previewed.fusion?.name === "" || previewed.fusion?.discoveredBy === null;
      if (!fused || !discovered || fused.fusion?.discoveredBy?.id !== me) return;
      play("discover");
      closable(
        h("div", { class: "label" }, t("fuse.new")),
        h("h2", { class: "reveal", "data-testid": "fusion-reveal" }, t(claimed ? "forge.claimed" : "fuse.discovered", { name: fused.name })),
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
    const fusions = offerFusions(o.slot);
    if (fusions.length) sheet.insertBefore(h("div", { class: "stack offer-fusions", "data-testid": "offer-fusions" }, ...fusions.map((f) => h("div", { class: `small${f.awakens ? "" : " dim"}` }, `⚗ ${offerLine(f)}`))), sheet.querySelector('[data-testid="unit-rates"]'));
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
      const c = actionable(card(cu, { side: "you", tier: o.tier, extra: [h("div", { class: "cost" }, price)], testid: `offer-${o.slot}` }), `${cu.name}, ${price}`);
      if (o.locked) c.append(h("span", { class: "lock-mark", "aria-label": t("offer.locked") }, "🔒"));
      if (run.gold < o.cost) c.classList.add("poor");
      if (o.locked) c.classList.add("locked");
      if (owned) c.classList.add("owned");
      if (!crown) dragCard(c, { kind: "offer", offer: o });
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
  /** Rules v2: an offer that brings a designed fusion within reach wears a
   * mark naming it: bright when buying it awakens the unit (one buy away),
   * quiet when it would still need awakening. */
  const markOffers = () => {
    for (const o of run.offers) {
      const f = offerFusions(o.slot)[0];
      const c = offers.querySelector<HTMLElement>(`[data-testid="offer-${o.slot}"]`);
      if (!f || !c) continue;
      c.classList.add(f.awakens ? "fuse-offer" : "fuse-near");
      c.append(h("div", { class: "fuse-tag", "data-testid": `fuse-offer-${o.slot}`, title: offerLine(f) }, f.awakens ? (f.design ? `⚗ ${fusionName(f)}` : t("fuse.offerClaim")) : "⚗"));
    }
  };
  if (codexOn && !crown)
    void api.forge(run.runId).then(
      (f) => {
        forge = f;
        markOffers();
        renderLine();
      },
      () => undefined,
    );

  // C-Shop: "Reroll" with its price in a gold chip.
  const reroll = button(t("shop.reroll"), () => void decide({ kind: "reroll" }), "reroll-btn", "reroll");
  reroll.append(h("span", { class: "cost-chip" }, t("shop.gold", { n: rules.rerollCost })));
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
    b.addEventListener("click", () => closable(h("div", { class: "label" }, t("pin.championOfDay", { seq: d!.seq }), who(ch.player.name)), team(ch.line, "ghost", content)));
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
    // A tier ahead of the shop (#916), unless the shop already sells the top tier.
    const shopTier = rules.tierOpensAt.reduce((top, round, i) => (round <= run.round ? i + 1 : top), 0);
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
      const c = actionable(card(cu, { side: "you", ...(u ? { tier: u.tier } : {}), extra: mine ? [h("div", { class: "cost" }, "＋")] : [], testid: `gift-card-${i}` }), cu.name);
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
      const pickBtn = withKey(button(ok ? (mine ? t("gift.pickOwned") : t("gift.pick")) : t("gift.full"), () => (close(), void decide({ kind: "gift", pick: i })), "primary", `gift-pick-${i}`), String(i + 1));
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
      h("div", { class: "dim small", "data-testid": "gift-free" }, tier ? t(tier > shopTier ? "gift.freeTierAhead" : "gift.freeTier", { tier: roman(tier) }) : t("gift.free")),
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
    // Keys (#1064): 1–3 pick the card with that number (it is on its button).
    document.querySelector<HTMLElement>('[data-testid="gift-title"]')?.closest(".sheet")?.addEventListener("keydown", (e) => {
      const ev = e as KeyboardEvent;
      if (ev.ctrlKey || ev.metaKey || ev.altKey || !/^[1-9]$/.test(keyOf(ev))) return;
      const b = document.querySelector<HTMLButtonElement>(`[data-testid="gift-pick-${Number(keyOf(ev)) - 1}"]`);
      if (b && !b.disabled) (ev.preventDefault(), b.click());
    });
  }

  // "?" opens the Codex's How to play (M5-6: the rules and how to read a card); until a player has opened it once, it says so.
  const legendBtn = button(seen("legend") ? "?" : t("shop.legendNew"), () => (markSeen("legend"), void guarded(err, () => openCodex({ tab: "howto" }))), seen("legend") ? "small" : "small new", "legend-open");
  renderLine();

  const menuBtn = button("☰", () => runMenu(run, content, err), "menu-btn", "menu-open");
  menuBtn.setAttribute("aria-label", t("shop.menu"));
  if (desk) (menuBtn.title = t("shop.menuEsc"), withKey(menuBtn, "Esc"));

  const nextOpp = h("span", { class: "dim next-opp", "data-testid": "next-opponent" }, ...(opp ? [`${crown ? t("opp.crownVs") : t("opp.next")} `, who(opp.player.name), `${opp.player.bot ? " 🤖" : ""}${ownCrown ? t("opp.yourChampionTeam") : ""}`] : [crown ? t("opp.crownVsChampion") : t("opp.nextSaved")]));
  show(
    h(
      "div",
      { class: "topbar", "data-testid": "topbar" },
      h(
        "div",
        { class: "hud", "data-testid": "hud" },
        menuBtn,
        desk ? runTrack(run) : h("span", { class: "round-pill", "data-testid": "round" }, roundLabel(run.round)),
        heartIcons(run.hearts),
        // The Crown has no shop: no gold to show.
        crown ? h("span", {}) : h("span", { class: "gold", "data-testid": "gold" }, t("shop.gold", { n: run.gold })),
      ),
      h(
        "div",
        { class: "row spread opp" },
        desk ? null : nextOpp,
        crown ? null : pin,
      ),
    ),
    h(
      "div",
      { class: "board" },
      // C-Shop: the line, the bench and the shop are three outlined panels on one board.
      h(
        "section",
        { class: "run-panel line-panel" },
        h(
          "div",
          { class: "row spread line-head" },
          h("div", { class: "label" }, crown ? t("line.crown") : t("line.phone")),
          // Desktop: who's next sits on the line's panel (C-Shop), the top bar keeps the track.
          h("div", { class: "row" }, desk ? nextOpp : null, legendBtn),
        ),
        line,
        // The phone keeps the bench in the line's panel: one frame less, so round 8 fits 640 px.
        desk || B === 0 ? null : bench,
      ),
      // Desktop: the bench and the hint share a row (C-Shop's bench beside the fuse note).
      desk ? h("div", { class: "bench-hint" }, B > 0 ? h("section", { class: "run-panel bench-panel" }, bench) : null, hintSlot) : null,
      desk ? null : actions,
      desk ? null : hintSlot,
      h("section", { class: "run-panel shop-panel" }, crown ? null : h("div", { class: "label" }, desk ? t("shop.offersDesk", { n: run.offers.length }) : t("shop.offersPhone")), crown ? foe : offers),
    ),
    desk ? inspector : null,
    h("div", { class: "spacer" }),
    h("div", { class: "row shop-foot" }, crown ? null : reroll, fight),
    err,
  );
  screen("shop");
  // The keyboard stays with the unit in hand (#1064): a move or a merge redraws the
  // shop, and the focus that was on a board card follows the selection.
  if (hasKeys() && pick.mode === "none" && keyboardLost()) document.querySelector<HTMLElement>('[data-testid="line-0"], [data-testid="bench-0"], [data-testid="offer-0"]')?.focus({ preventScroll: true });
  if (hasKeys() && pick.mode === "picked") {
    const on = document.activeElement;
    if (on === document.body || (on instanceof HTMLElement && /^(line|bench)-\d+$/.test(on.dataset.testid ?? ""))) {
      const at = pick.index;
      document.querySelector<HTMLElement>(`[data-testid="${at < L ? `line-${at}` : `bench-${at - L}`}"]`)?.focus({ preventScroll: true });
    }
  }
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
    const k = keyOf(e);
    const sel = pick.mode === "picked" ? pick.index : -1;
    if (k === "Escape") {
      if (fuseView) fuseView = null;
      else if (pick.mode !== "none") pick = { mode: "none" };
      else if (chosen !== null) chosen = null;
      else return runMenu(run, content, err), true;
      renderLine();
      return true;
    }
    if (!hasKeys()) return false;
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
      else if (canFuse(sel)) pick = { mode: "fuse", first: sel };
      else return false;
      renderLine();
      return true;
    }
    return false;
  });
  // A forge scene (#1049) opens its pair's forge at once.
  if (forgeScene) {
    const f = forgeScene;
    forgeScene = null;
    // After the scene's own guarded call lets go (guarded ignores a call while one is out).
    setTimeout(() =>
      void fusePreview(f[0], f[1]).then(() => {
        if (f[2]) document.querySelector<HTMLElement>(`[data-testid="${f[2]}"]`)?.click();
      }),
    );
  }
  if (!hasKeys()) return;
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
const kbd = (k: string) => h("kbd", {}, keyLabel(k));

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

async function fightScreens(run: RunView, fight: FightResult, content: MvpContent, before: RunView = run): Promise<void> {
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
  battleScreen({ battle, content, you: "A", fight, run, outro: { ...outroOf(run, fight, content), bar: (over) => runBar(over ? run : before, before) }, onDone: () => shopScreen(run, content) });
}

/** The shop's top bar, on the fight (M5-4: one run board): the round (the
 * track on desktop), hearts and gold; the Crown has no gold. The phone's pill
 * names the round fought (`fought`), as the end card does. */
function runBar(run: RunView, fought: RunView = run): HTMLElement[] {
  return [
    isDesktop() ? runTrack(run) : h("span", { class: "round-pill", "data-testid": "round" }, roundLabel(fought.round)),
    heartIcons(run.hearts),
    ...(run.phase === "crown" || run.round > rules.rounds ? [] : [h("span", { class: "gold", "data-testid": "gold" }, t("shop.gold", { n: run.gold }))]),
  ];
}

/** What the battle's end card adds for a run's fight (R2-17 batch E): it is
 * the fight's one result, so it carries what the result screen showed. The
 * round fought (R3/12, or CROWN), the hearts and record after it, and what
 * the result costs or wins; the run's ☰; its last button goes straight on: to
 * the next round's shop, the Crown, or the run's end (the screens that add
 * something). Both lines are on the board and in the card's Damage list,
 * each unit's sheet a tap away; "vs @name" is in the battle's HUD. */
function outroOf(run: RunView, fight: FightResult, content: MvpContent): RunOutro {
  const label = fight.kind === "crown" ? t("run.crownLabel") : roundLabel(fight.round);
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
  rememberLastRun(run);
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
    h("div", { class: "row footer" }, ...(newRun ? [withKey(button(t("over.home"), home, "grow", "home"), "Esc"), withKey(button(t("over.newRun"), next, "primary grow", "new-run-start"), "Enter")] : [withKey(button(t("over.home"), home, "primary grow", "home"), "Enter")])),
  );
  screen("over");
  music("shop", run.runId);
  onKeys((e) => (keyOf(e) === "Enter" ? ((newRun ? next : home)(), true) : keyOf(e) === "Escape" ? (home(), true) : false));
  (document.querySelector<HTMLElement>('[data-testid="new-run-start"], [data-testid="home"]'))?.focus({ preventScroll: true });
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
const inviteCode = CG ? null : new URLSearchParams(location.hash.slice(1)).get("invite");
/** `#join=<code>` (R4-20): the open join link. Anyone with it picks a name
 * and plays as a new player; the fragment then becomes their own #invite=
 * link, so a bookmark keeps working. A bad or rotated code shows the invite
 * screen. */
const joinCode = CG ? null : new URLSearchParams(location.hash.slice(1)).get("join");
/** `#battle=<id>` (mission 1007 follow-up): open one recorded battle in the
 * viewer straight away, no sign-in, so a change to the battle view can be
 * judged from a list of links. Battles are public by id (GET /battles/:id);
 * a slay still hidden today answers 404, as in the playoff panel. */
const battleLink = new URLSearchParams(location.hash.slice(1)).get("battle");
function openBattleLink(id: string): void {
  const err = errorLine();
  void guarded(err, async () => {
    const [battle, content] = await Promise.all([api.battle(id), getContent()]);
    battleScreen({ battle, content, onDone: () => location.reload() });
  }).then(() => {
    if (err.textContent) show(h("h1", {}, "Battle " + id), err);
  });
}
/** `#checkpoint=a+b` (#1065): a dev world (the workshop, an admin) puts your
 * run in the shop with Awoken a and b on the line, one Fuse from discovering
 * them. With `invite=` in the same link it opens once the invite is in. */
const checkpointPair = CG ? null : new URLSearchParams(location.hash.slice(1)).get("checkpoint");
function openCheckpoint(pair: string): void {
  const err = errorLine();
  void guarded(err, async () => {
    // URLSearchParams reads the link's "+" as a space
    const [a = "", b = ""] = pair.split(/[+ ]/);
    const { runId } = await api.checkpoint(a, b);
    const url = new URL(location.href);
    url.hash = "";
    history.replaceState(null, "", url.href.replace(/#$/, ""));
    shopScreen(await api.run(runId), await getContent());
  }).then(() => {
    if (err.textContent) show(h("h1", {}, "Checkpoint " + pair), err, button(t("invite.retry"), () => location.reload(), "primary"));
  });
}
/** `#scene=<id>` (#1026, ./scene.ts): a scenes server builds the fixture
 * (server/src/mvp/scenes.ts) and this opens its screen as its player, who
 * lives in memory only. */
function openScene(id: string): void {
  const err = errorLine();
  const settle = (fn: () => void) => setTimeout(fn, 300);
  void guarded(err, async () => {
    const { entry } = await api.scene(id);
    const content = await getContent();
    switch (entry.screen) {
      case "name":
        return nameScreen();
      case "holding":
        return holdingScreen();
      case "home":
        await homeScreen(entry.ended ?? null);
        if (entry.click) settle(() => document.querySelector<HTMLElement>(`[data-testid="${entry.click}"]`)?.click());
        if (entry.scroll) settle(() => document.querySelector(`[data-testid="${entry.scroll}"]`)?.scrollIntoView({ block: "center" }));
        return;
      case "shop":
        if (entry.forge) forgeScene = [entry.forge[0], entry.forge[1], entry.click];
        return shopScreen(await api.run(entry.runId), content);
      case "run-over":
        return runOverScreen(await api.run(entry.runId), content);
      case "battle": {
        const battle = await api.battle(entry.battleId);
        battleScreen({ battle, content, onDone: () => location.reload() });
        const keys = entry.keys ?? [];
        keys.forEach((key, i) => setTimeout(() => document.dispatchEvent(new KeyboardEvent("keydown", { key: key === "Space" ? " " : key, code: key, bubbles: true })), 600 + i * 120));
        return;
      }
      case "codex":
        return openCodex({ tab: entry.tab });
      case "ideas":
        await ideasScreen({ onBack: () => void homeScreen(), onUnknown: () => location.reload() });
        if (entry.click) settle(() => document.querySelector<HTMLElement>(`[data-testid="${entry.click}"]`)?.click());
        return;
      case "forge":
        return forgeScreen({ view: entry.view, tab: entry.tab, unit: entry.unit, onBack: () => location.reload() });
      case "claim":
        return claimScreen({ claim: entry.claim, units: entry.view.units, viewer: entry.view.viewer, picked: entry.picked, onDone: () => forgeScreen({ view: entry.view, tab: "feed", onBack: () => location.reload() }) });
      case "pick":
        return pickScreen(entry.ideaId, { toIdeas: (notice) => void ideasScreen({ onBack: () => void homeScreen(), onUnknown: () => location.reload() }, notice ?? null), onUnknown: () => location.reload() });
    }
  }).then(() => {
    if (err.textContent) show(h("h1", {}, "Scene " + id), err);
  });
}
const dropInvite = () => {
  const url = new URL(location.href);
  url.hash = "";
  history.replaceState(null, "", url.href.replace(/#$/, ""));
};
function openInvite(code: string): void {
  const err = errorLine();
  let redeemed = false;
  void guarded(err, async () => {
    const mine = api.player;
    if (mine) {
      const theirs = await api.invitePlayer(code);
      if (theirs.id !== mine.id) return switchScreen(code, mine, theirs);
    }
    await api.redeem(code);
    redeemed = true;
    dropInvite();
    if (checkpointPair) return; // opened once this guarded call settles (guarded ignores a nested one)
    await homeScreen();
  }).then(() => {
    if (!err.textContent && checkpointPair && redeemed) openCheckpoint(checkpointPair);
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
  onKeys((e) => (keyOf(e) === "Escape" ? (stay(), true) : false));
}
/** The join link's sign-up: by email (#929) when the server has it, else a name. */
function joinForm(code: string, tg: HTMLElement | null = null): void {
  music("home");
  void api.health().then(
    (hl) => (hl.invites && hl.mail ? emailSignUp({ join: code, extra: tg, onDone: signedIn }) : nameJoinForm(code, tg)),
    () => nameJoinForm(code, tg),
  );
}
function nameJoinForm(code: string, tg: HTMLElement | null): void {
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
  const go = () => {
    const err = errorLine();
    void guarded(err, async () => {
      await api.redeem(code);
      dropInvite();
      if (!checkpointPair) await homeScreen();
    }).then(() => {
      // the checkpoint once this guarded call settles (guarded ignores a nested one)
      if (!err.textContent && checkpointPair) openCheckpoint(checkpointPair);
    });
  };
  show(
    h("h1", {}, t("invite.title")),
    h("p", { "data-testid": "invite-switch" }, t("switch.for", { theirs: theirs.name, mine: mine.name })),
    h("p", { class: "dim" }, t("switch.then", { theirs: theirs.name, mine: mine.name })),
    h("div", { class: "row footer" }, button(t("join.stay"), stay, "grow", "invite-stay"), button(t("switch.go"), go, "primary grow", "invite-switch-go")),
  );
  onKeys((e) => (keyOf(e) === "Escape" ? (stay(), true) : false));
}

document.documentElement.lang = uiLang();
applyTheme();
initSound();
// A link pasted into a tab that already shows the game only changes the
// fragment, which reloads nothing: start over so the link opens.
addEventListener("hashchange", () => {
  const hash = new URLSearchParams(location.hash.slice(1));
  if (hash.has("invite") || hash.has("join") || hash.has("battle") || hash.has("scene") || hash.has("checkpoint")) location.reload();
});

if (sceneId) {
  openScene(sceneId);
} else {
  // A dark server (holding/server.mjs) holds the page while the game is rebuilt: ask before anything else.
  void api.health().then(
    (hl) => (hl.dark ? holdingScreen() : boot()),
    () => boot(),
  );
}
/** #1218, the cg build: the SDK inits on the loading screen; a device with
 * no player becomes a guest at once, and a logged-in CrazyGames user (now or
 * later) becomes their account's player, a guest's progress kept. */
async function cgBoot(): Promise<void> {
  show(h("h1", {}, t("name.title")));
  await cgInit();
  // #1219: loading runs from here until Home (or the error) shows; gameplay
  // follows the screen from then on.
  cgLoading(true);
  cgApplySettings();
  cgPlaytestHooks();
  cgWatchGameplay(app);
  const login = async () => {
    const token = await cgUserToken();
    if (!token) return false;
    try {
      await api.crazyGames(token);
      return true;
    } catch (err) {
      console.warn("CrazyGames login:", err);
      return false;
    }
  };
  onCgLogin(() => void login().then((ok) => { if (ok) void homeScreen(); }));
  const err = errorLine();
  await guarded(err, async () => {
    if (!(await login()) && !api.hasToken) await api.guest();
    await homeScreen();
  });
  cgLoading(false);
  if (err.textContent) show(h("h1", {}, t("name.title")), err, button(t("invite.retry"), () => location.reload(), "primary"));
}
function boot(): void {
  if (CG && !battleLink) {
    void cgBoot();
    return;
  }
  if (battleLink) {
    openBattleLink(battleLink);
  } else if (inviteCode) {
    openInvite(inviteCode);
  } else if (joinCode) {
    openJoinLink(joinCode);
  } else if (checkpointPair && api.player) {
    openCheckpoint(checkpointPair);
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
}
