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
import { homeTable } from "./screens/home";
import { pickScreen } from "./screens/pick";
import { card, formRich, roman, setCardCredits, unitSheet, type CardUnit } from "./ui/card";
import { isMine, reportLink, setReporter } from "./ui/report";
import { previewName } from "./ui/fusion";
import { fuseWarning } from "./ui/fuse-warn";
import { icon } from "./ui/icon";
import { actionable, app, button, closable, desktopQuery, dismissable, h, hasKeys, isDesktop, keepFocus, keepScreen, keyboardLost, keyLabel, keyOf, onKeys, onRestore, overlay, screen, show, who, withKey } from "./ui/dom";
import { loadUnitRates } from "./ui/unit-stats";
import { startScreen, type StartControl } from "./screens/start";
import { guestName } from "./ui/guest-name";
import { coachStep, coaching, type CoachStep } from "./ui/coach";
import { coachState, coachView } from "./ui/coach-view";
import { telegramSheet } from "./screens/telegram";
import { emailAttach, emailSignIn, emailSignUp } from "./screens/email";
import { galleryScreen } from "./screens/gallery";
import { faceOfUnit, lineToPiece, piece, pieceReach } from "./ui/piece";
import { predictRun } from "./ui/predict";
import { orient, stage } from "./ui/stage";
import { fieldLayout, fit } from "./ui/stage-fit";
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

/** Where the latest tap landed, where the busy request's own tap did, and the
 * taps that came while it was out (arena#15, #16): those aren't lost but
 * replayed in order on the screen the answer drew, unless one is the same spot
 * tapped again (the double tap busy is for). A press shows at once (.pending),
 * however slow the answer, and so does a tap that waits. Shop slots are cells:
 * an empty slot tapped while a buy was out (the second tap of tap-tap) was
 * dropped, and only the last waiting tap was kept (arena#16). */
type Tap = { x: number; y: number };
let lastTap: Tap | null = null;
let busyTap: Tap | null = null;
let queuedTaps: Tap[] = [];
const TAPPABLE = "button, .card, .cell, [role=button]";
const waiting = new Set<Element>();
addEventListener("pointerdown", (e) => {
  lastTap = { x: e.clientX, y: e.clientY };
  // No drag starts while a request is out: the tap waits instead.
  if (busy && e.target instanceof Element && e.target.closest(".card, .cell")) e.stopPropagation();
}, true);
addEventListener("click", (e) => {
  if (!busy || !(e.target instanceof Element)) return;
  const el = e.target.closest(TAPPABLE);
  if (!el) return;
  e.stopImmediatePropagation();
  e.preventDefault();
  const p = { x: e.clientX, y: e.clientY };
  if (e.detail > 0 && (!busyTap || Math.hypot(p.x - busyTap.x, p.y - busyTap.y) > 24) && queuedTaps.length < 4) {
    queuedTaps.push(p);
    el.classList.add("pending");
    waiting.add(el);
  }
}, true);

async function guarded(err: HTMLElement, fn: () => Promise<void>): Promise<void> {
  if (busy) return;
  busy = true;
  busyTap = lastTap;
  app.classList.add("busy");
  const pressed = busyTap ? document.elementFromPoint(busyTap.x, busyTap.y)?.closest(TAPPABLE) : null;
  pressed?.classList.add("pending");
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
    pressed?.classList.remove("pending");
    for (const el of waiting) el.classList.remove("pending");
    waiting.clear();
    const q = queuedTaps; // set by the click listener meanwhile
    queuedTaps = [];
    lastTap = null;
    for (let i = 0; i < q.length; i++) {
      lastTap = q[i]!;
      (document.elementFromPoint(q[i]!.x, q[i]!.y)?.closest(TAPPABLE) as HTMLElement | null)?.click();
      // That tap sent a request of its own: the rest wait for its answer.
      if (busy) {
        queuedTaps = q.slice(i + 1).concat(queuedTaps);
        break;
      }
      lastTap = null;
    }
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
  // The number keys in plain words: "←/→ pick an offer, then 1–5 buys it into that slot".
  const numberKeys = [" ", kbd("1"), "–", kbd(String(rules.lineSize))];
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
  firstScreen(async (name) => void (await api.register(name)), [mail ? button(t("email.signIn"), () => emailSignIn({ back: nameScreen, onDone: signedIn }), "", "email-signin-open") : null, tg]);
}

/** The first screen (arena#31, mission #26 move 3): "One rule starts a chain."
 * over a chain that plays itself. PLAY (Enter) skips the name form: a
 * generated name, then round 1 with the coach. A name typed in the quiet field
 * under it takes the name form's way: Home. `register` makes the player. */
function firstScreen(register: (name: string) => Promise<void>, extra: (HTMLElement | null)[]): void {
  const err = errorLine();
  let going = false;
  let screen: StartControl | null = null;
  const go = (name: string | null) => {
    if (going) return;
    going = true;
    // At once, before any request: a slow server must never look like a dead button.
    err.textContent = "";
    screen?.starting();
    void (async () => {
      try {
        if (name !== null) {
          await register(name);
          return await homeScreen();
        }
        // A player made by an earlier press that failed later is reused, never made twice.
        if (!api.player) await registerGenerated(register);
        await playFirst();
      } catch (e) {
        if (e instanceof ApiError && e.status === 401 && e.message.startsWith("unknown player")) api.forget();
        err.textContent = e instanceof ApiError && e.message ? e.message : t("start.failed");
        screen?.ready();
      } finally {
        going = false;
      }
    })();
  };
  // The pool and today's champion (the demo's top row) come before the screen draws; either failing leaves a plainer demo.
  void Promise.all([getContent().then((c) => c, () => null), api.day().then((d) => d.champion?.line ?? null, () => null)]).then(([content, champion]) => (screen = startScreen({ content, champion, onPlay: () => go(null), onName: (n) => go(n), extra, err })));
}

/** A generated name the server takes (a taken or crude one is rerolled). */
async function registerGenerated(register: (name: string) => Promise<void>): Promise<void> {
  for (let tries = 0; ; tries++) {
    try {
      return await register(guestName(undefined, tries < 3 ? 3 : 5));
    } catch (e) {
      if (!(e instanceof ApiError) || ![400, 409].includes(e.status) || tries >= 5) throw e;
    }
  }
}

/** PLAY: the new player's first run, at once (Home is skipped). The run's start
 * goes out beside Home and the health check: two round trips from PLAY to the
 * table, not four (a refused start is told after Home's email check, which wins). */
async function playFirst(): Promise<void> {
  void loadUnitRates();
  const started = api.startRun().then((r) => ({ r }), (e: unknown) => ({ e }));
  const [home, content, health]: [HomeView, MvpContent, { codex?: boolean }] = await Promise.all([api.home(), getContent(), api.health().catch(() => ({}))]);
  codexOn = !!health.codex;
  // A server that wants an email first sends the player to add one (homeScreen does it).
  if (!CG && home.email?.required && !home.email.address) return homeScreen();
  rules = home.rules;
  day = home.day;
  firstRun = !home.rating || home.rating.runs === 0;
  const run = await started;
  if ("e" in run) throw run.e;
  shopScreen(run.r, content);
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
  firstRun = !home.rating || home.rating.runs === 0;
  // The run that waits, for RESUME's round and hearts (null if it ended meanwhile).
  const waiting = home.activeRunId ? await api.run(home.activeRunId).catch(() => null) : null;
  const active = waiting && waiting.phase !== "over" ? waiting : null;
  drawHome(home, content, active, ended, err, true);
  music("home");
}

/** Home's table, built from what the server said. It draws again, from the same answers, when the window changes
 * shape (`fresh` is false then: a menu the dev "End day now" opened stays shut). */
function drawHome(home: HomeView, content: MvpContent, active: RunView | null, ended: number | null, err: HTMLElement, fresh: boolean): void {
  const champ = home.day.champion;
  const r = home.rating;
  const own = champ?.player.id === api.player?.id;
  const draw = () => drawHome(home, content, active, ended, err, false);
  // PLAY, or RESUME while a run waits: over the middle of your five slots.
  const play = active
    ? button("", () => void guarded(err, async () => shopScreen(await api.run(active.runId), content)), "primary", "play")
    : button("", () => void guarded(err, async () => shopScreen(await api.startRun(), content)), "primary", "play");
  play.classList.add("play-big");
  play.append(
    active
      ? h("span", { class: "play-main" }, t("home.resume"), h("span", { class: "play-where" }, active.round > rules.rounds ? t("home.continueCrown") : t("home.resumeRound", { round: active.round }), " ", hearts(active.hearts)))
      : h("span", { class: "play-main" }, t("start.play")),
  );
  if (isDesktop()) play.append(h("kbd", { class: "play-key" }, "Enter"));
  const note = active ? null : h("div", { class: "hm-note", "data-testid": "home-note" }, t(!champ ? "home.playSubNone" : own ? "home.playSubOwn" : "home.note", { n: rules.rounds }));
  const newRun = active
    ? button(t("home.newRun"), () => abandonSheet(active, "new", () => void guarded(err, async () => {
        // The given-up run's end and rating change first, as ☰ Abandon shows
        // them; its "New run" starts the next one.
        runOverScreen(await api.abandon(active.runId), content, "", true);
      })), "hm-newrun link", "new-run")
    : null;

  // The banner: the champion's name, how many slayers, and how long till the new day.
  const hours = Math.max(1, Math.ceil((new Date(home.day.endsAt).getTime() - Date.now()) / 3_600_000));
  const banner = h(
    "div",
    { class: "hm-banner", "data-testid": "home-banner" },
    h("span", { class: "crown-badge", "aria-hidden": "true" }, CROWN_SVG()),
    h("span", { class: "hm-banner-main" }, ...(champ ? withNodes(t("home.banner"), { who: whoMark(champ.player) }) : [t("home.bannerNone")])),
    h(
      "small",
      { class: "hm-banner-sub", "data-testid": "slayers" },
      champ ? (own ? t("home.ownTeam") : t("home.bannerSlayers", { n: home.day.slayers, h: hours })) : t("home.newChampionAt", { at: rules.dayEndsAt }),
    ),
  );
  const last = home.day.lastPlayoff ?? null;
  const justEnded = ended !== null && last?.seq === ended ? last : null;
  const notice = ended !== null ? h("div", { class: "hm-notice", "data-testid": "day-ended" }, t("home.dayEnded", { ended, today: home.day.seq })) : null;

  // The HUD plates: RATING n (the last run's change joins it), and the ≡ Esc menu.
  const rating = h("span", { class: "num", "data-testid": "rating" }, `${r?.rating ?? rules.ratingStart}`);
  const pill = h("div", { class: "plate hm-rate keep", "data-testid": "home-rating", title: t("home.ratingAria") }, h("span", { class: "hm-rate-l" }, t("home.rating")), " ", rating);
  const lastRun = lastRunPanel(r?.rating ?? null, pill);
  const playoff = playoffPanel(last, champ?.player ?? null, content, err);
  const menu = button("", () => homeMenu(home, err, lastRun, playoff, justEnded !== null), "plate hm-menu", "settings");
  menu.setAttribute("aria-label", t("home.menu"));
  menu.append(h("span", { class: "hm-burger", "aria-hidden": "true" }, "≡"));
  if (hasKeys()) (menu.title = `${t("home.menu")} (Esc)`, menu.append(h("kbd", {}, "Esc")));

  // The tray: the Workshop card (the ideas meter lives in it), the vote, and the rows.
  const row = (label: string, key: string, sub: string, open: () => void, testid: string) => {
    const b = button("", open, "hm-rowbtn", testid);
    b.append(h("b", { class: "hm-rowname" }, label), " ", ...(hasKeys() ? [h("kbd", {}, key.toUpperCase()), " "] : []), h("span", { class: "hm-rowsub" }, sub));
    return b;
  };
  const forgeRow = codexOn ? row(t("menu.forge"), "F", "", () => void guarded(err, () => openForge()), "forge-open") : null;
  if (forgeRow) void api.fusions().then((f) => (forgeRow.querySelector(".hm-rowsub")!.textContent = t("home.rowFusions", { n: f.length })), () => undefined);
  const rows = [
    row(t("menu.codex"), "C", t("home.rowUnits", { n: content.units.length }), () => void guarded(err, () => openCodex()), "codex"),
    forgeRow,
    row(t("home.stats"), "S", t("home.rowEvery"), () => void guarded(err, () => openCodex({ tab: "champions" })), "stats"),
  ];
  const vote = votePanel(content, true);

  show(
    homeTable({
      content,
      champion: champ?.line ?? null,
      title: t("name.title"),
      rating: pill,
      menu,
      banner,
      notice,
      toBeat: t("home.toBeat"),
      yourLine: t("home.yourLine"),
      play,
      note,
      newRun,
      workshop: workshopCard(home.ideas),
      vote,
      rows,
      err,
      redraw: draw,
    }),
  );
  screen("home");
  // Keys (#1064): Enter plays, Esc opens the ≡ menu, and a letter on each row (it is on the button).
  const press = (id: string) => document.querySelector<HTMLButtonElement>(`[data-testid="${id}"]`);
  const letters: Record<string, string> = { c: "codex", f: "forge-open", s: "stats", i: "ideas", g: "settings", Escape: "settings" };
  onKeys((e) => {
    const k = keyOf(e);
    // The Dictionary sits in the ≡ menu; D still opens it from Home.
    if (k === "d" && codexOn) return void guarded(err, () => openForge("words")), true;
    const b = k === "Enter" ? press("play") : press(letters[k] ?? "");
    if (!b || b.disabled) return false;
    b.click();
    return true;
  });
  // The dev "End day now" just ran: the ≡ menu opens at once, with how the day ended in it.
  if (justEnded && fresh) homeMenu(home, err, lastRun, playoff, true);
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
const CROWN_SVG = svg("M3 18h18L19 7l-5 4-2-6-2 6-5-4z", { width: "30", height: "30", fill: "var(--paper)", stroke: "var(--outline)", "stroke-width": "2.2", "stroke-linejoin": "round" });

/** The ≡ menu (Esc on Home): a window over the table with everything Home used to carry on its page: who you are,
 * the last run and the last playoff, Sound and Music, the language, the palette, the Dictionary, Rules, this
 * device's link, Telegram, and on a dev server "End day now" with the other dev tools. `lastRun`/`playoff` are
 * made with Home (the last run's change joins the RATING plate), so they are only put in the window here. */
function homeMenu(home: HomeView, err: HTMLElement, lastRun: HTMLElement | null, playoff: HTMLElement | null, ended: boolean): void {
  if (app.querySelector('[data-testid="settings-sheet"]')) return;
  const champ = home.day.champion;
  const close = closable(
    h(
      "div",
      { class: "stack settings", "data-testid": "settings-sheet", "data-scroll": "" },
      h("h2", {}, t("home.settings")),
      h("div", { class: "row me" }, h("span", { class: "dim keep" }, t("home.playingAs")), who(api.player?.name ?? "")),
      home.email?.address ? h("div", { class: "row me", "data-testid": "settings-email" }, h("span", { class: "dim keep" }, t("email.label")), h("span", { class: "grow" }, home.email.address)) : null,
      ended ? playoff : null,
      lastRun,
      ended ? null : playoff,
      champ && !champ.player.bot && !isMine(champ.player.id) ? reportLink("name", champ.player.id, `@${champ.player.name}`) : null,
      soundRow(),
      langRow(),
      themeRow(),
      codexOn ? withKey(button(t("menu.dictionary"), () => (close(), void guarded(err, () => openForge("words"))), "", "dictionary-open"), "D") : null,
      button(t("codex.tab.howto"), () => (close(), void guarded(err, () => openCodex({ tab: "howto" }))), "", "rules-open"),
      api.ownInvite && !CG ? ownLinkRow(api.ownInvite) : null,
      home.telegram?.enabled ? telegramRow(home.telegram.linked, err) : null,
      home.dev ? devTools(home, err) : null,
    ),
  );
}

/** Dev servers only (HomeView.dev): "End day now" and the idea tools, folded in the ≡ menu. */
function devTools(home: HomeView, err: HTMLElement): HTMLElement {
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
  return h("details", { class: "dev" }, h("summary", {}, t("dev.title")), h("div", { class: "row wrap" }, endDay, grantIdea, creditUnit, seedCandidate, overnight, fakeVotes, candidates));
}

/** The Workshop card in Home's tray (it replaces the ideas meter): how the player's ideas stand in one line,
 * the meter (ideas left today, when they come back) under it, and a tap opens the Workshop (My ideas). */
function workshopCard(ideas: IdeasView | null): HTMLElement {
  const b = button("", () => void openWorkshop(), "hm-workshop", "ideas");
  const status = h("span", { class: "hm-ws-status", "data-testid": "workshop-status" }, ideas?.ready ? t("home.workshopReady", { n: ideas.ready }) : t("home.workshopNone"));
  b.append(h("span", { class: "hm-ws-head" }, h("b", { class: "hm-ws-title" }, t("home.workshop")), ideas?.ready ? h("i", { class: "hm-dot", "aria-hidden": "true" }) : null), status);
  if (ideas) {
    // The ideas meter's words: the ideas left today, and when the rest come back.
    const label = ideas.ready ? t("home.ideasReady", { n: ideas.ready }) : ideas.held > 0 ? t("home.ideasHeld", { n: ideas.held }) : t("home.ideasNone");
    const meter = h("span", { class: "hm-ws-meter" }, h("span", { "data-testid": "ideas-label" }, label));
    if (!ideas.ready && ideas.held < ideas.perDay) meter.append(" ", h("span", { class: "idea-runs", "data-testid": "ideas-refill" }, t("home.ideaRefill", { n: ideas.perDay - ideas.held, time: refillTime(ideas.refillsAt) })));
    b.append(meter);
    if (ideas.ready) b.dataset.ready = String(ideas.ready);
  }
  // Ideas on their way (read, simulated, voted on) are the line when none waits for a pick.
  if (!ideas?.ready) void api.myIdeas().then((v) => {
    const moving = v.sent.filter((i) => ["written", "reading", "simulating", "voting"].includes(i.state)).length;
    if (moving > 0) status.textContent = t("home.workshopMoving", { n: moving });
  }, () => undefined);
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
    codexCache = newCodexCache();
    codexBack = keep;
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
/** The signed-in player has no finished run yet: round 1 of their run has the coach (arena#31). */
let firstRun = false;
void api.health().then((h) => (codexOn = !!h.codex), () => undefined);
setReporter((r) => api.report(r), () => api.player?.id ?? savedPlayer()?.id ?? "");
/** The Workshop (My ideas, its write and pick screens) opens over the current screen as the Codex does: from a table
 * (the shop, a battle, Home) the table stays dimmed behind it, and Back or Esc returns to it untouched (arena#32). */
async function openWorkshop(notice: string | null = null): Promise<void> {
  const back = keepScreen();
  await ideasScreen({ onBack: back, onUnknown: () => (api.forget(), nameScreen()) }, notice);
}
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
  const workshop = button(t("menu.workshop"), () => (close(), void guarded(err, () => openWorkshop())), "", "menu-ideas");
  const menu = h(
    "div",
    { class: "stack run-menu", "data-testid": "run-menu" },
    h("div", { class: "label" }, run.phase === "over" ? t("menu.runOver") : crown ? t("menu.runCrown") : t("menu.runRound", { round, rounds: rules.rounds })),
    // Over a battle that was playing, Resume plays on (R2-17 batch F).
    withKey(button(t("menu.resume"), () => (close(), resume?.()), "primary", "menu-resume"), "Esc"),
    codex,
    forge,
    workshop,
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

/** The shop, as the table (arena#28): one stage at every size (ui/stage.ts, table.css), tap-tap
 * and keys, the inspector floating over it instead of pop-up sheets. `selected`: the line slot to
 * keep selected (a unit just moved by key or drag stays in hand). */
/** Where each offer stands in the shop row (arena#14): the server renumbers the
 * offers after a buy, so the row remembers positions: a bought offer leaves a
 * gap and the others stay where they were. `at[position]` is an index in run.offers, or -1.
 * Fewer offers than before: each remaining one is matched, in order, to the
 * unit it was at its old position (so a buy by any path, or one whose
 * redraw was missed, still leaves its gap); anything else lays the row out afresh. */
let offerRow: { runId: string; round: number; at: number[]; ids: string[] } | null = null;
function offerPositions(run: RunView): number[] {
  const live = run.offers.length;
  const prev = offerRow && offerRow.runId === run.runId && offerRow.round === run.round ? offerRow : null;
  const prevLive = prev ? prev.at.filter((i) => i >= 0).length : 0;
  let at: number[] = run.offers.map((_, i) => i);
  if (prev && prevLive === live) at = prev.at;
  else if (prev && live < prevLive) {
    const matched: number[] = prev.at.map(() => -1);
    let next = 0;
    prev.ids.forEach((id, pos) => {
      if (id && next < live && run.offers[next]!.unitId === id) matched[pos] = next++;
    });
    if (next === live) at = matched;
  }
  const n = Math.max(offersAt(rules, run.round), at.length);
  at = Array.from({ length: n }, (_, i) => at[i] ?? -1);
  offerRow = { runId: run.runId, round: run.round, at, ids: at.map((i) => (i >= 0 ? run.offers[i]!.unitId : "")) };
  return at;
}
/** The offer or the Fight button the coach points at, lit (its card is the tray's coach card). */
function mountCoach(step: CoachStep): void {
  const target = step.step === "fight" ? document.querySelector<HTMLElement>('[data-testid="fight"]') : document.querySelector<HTMLElement>(`[data-testid="offer-${step.offer}"]`);
  target?.classList.add("coach-target");
}
/** Fight was pressed: the button is down and the table starts its transition, before the answer (arena#28). */
function pressFight(): void {
  const st = document.querySelector<HTMLElement>(".stage");
  if (!st || st.classList.contains("fighting")) return;
  st.classList.add("fighting");
  document.querySelector('[data-testid="fight"]')?.classList.add("pressed");
  st.append(h("div", { class: "tb-fight-banner", "data-testid": "fight-banner", role: "status" }, t("fight.starting")));
}
/** Reroll was pressed: the gold is spent and the offers roll, before the new ones arrive. */
function pressReroll(run: RunView): void {
  document.querySelector('[data-testid="reroll"]')?.classList.add("pressed");
  document.querySelector('[data-testid="offers"]')?.classList.add("rolling");
  const gold = document.querySelector('[data-testid="gold"]');
  if (gold) gold.textContent = t("shop.gold", { n: Math.max(0, run.gold - rules.rerollCost) });
}
function shopScreen(run: RunView, content: MvpContent, notice = "", selected = -1, offer = -1): void {
  if (run.phase === "over") return runOverScreen(run, content, notice);
  // The table (arena#28): one stage and one set of controls at every size. `kb`
  // is only a keyboard being there: it adds the key caps and the key hints.
  const kb = hasKeys();
  const tall = orient() === "P";
  /** The landscape stage's rows (their wells, your line, the bench, the tray) for the height this window gives it. */
  const lay = tall ? null : fieldLayout(fit("L", window.innerWidth, window.innerHeight, true).h);
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
  // The first run's coach (arena#31): round 1, before the first fight, builds a pair from the shop that is there.
  const coachNow: CoachStep | null = !crown && !run.gift && !notice && coaching({ round: run.round, fights: run.fights.length, firstRun }) ? coachStep(coachState(run, L, content)) : null;
  const owns = (unitId: string) => mergeTarget(board, unitId) >= 0;
  let pick: Pick = unitAt(selected) ? { mode: "picked", index: selected } : { mode: "none" };
  const unitOf = (id: string) => unitIn(content, id); // a run on an older pool may hold a unit that left
  /** Buy previews still out: a decision waits for them, or the server may
   * apply a buy first and refuse the preview of its slot (a 409, R4-19). */
  const previewsOut = new Set<Promise<unknown>>();
  /** The answer to a decision: the run as the server has it, or the table put back as it was with why. */
  const answered = async (d: Parameters<typeof api.decide>[1], select: number, offer: number, shown: boolean) => {
    try {
      await Promise.allSettled([...previewsOut]);
      const res = await api.decide(run.runId, d);
      if (res.fight) return fightScreens(res.run, res.fight, content, run);
      play(shopSound(d, run, res.run));
      shopScreen(res.run, content, "", select, offer);
    } catch (e) {
      // What was drawn before the answer is taken back, with the reason (a refused buy puts the offer back).
      if (shown && !(e instanceof ApiError && e.status === 401)) {
        play("wrong");
        shopScreen(run, content, e instanceof ApiError && e.status === 409 ? plainRefusal(e.message) : e instanceof Error ? e.message : String(e));
      }
      throw e;
    }
  };
  /** Presses show at once (arena#28): the result predicted by the shop's own rules is drawn before the
   * request goes out; a reroll or a fight shows its pressed state and starts its motion. */
  const decide = (d: Parameters<typeof api.decide>[1], select = -1, offer = -1) =>
    guarded(err, async () => {
      if (d.kind === "fight") pressFight();
      else if (d.kind === "reroll") pressReroll(run);
      const next = predictRun(run, d, content, rules);
      if (next) shopScreen(next, content, "", select, offer);
      await answered(d, select, offer, next !== null || d.kind === "fight" || d.kind === "reroll");
    });
  /** Lock or unlock an offer (free); the offer is put down, so the inspector never covers the shop. */
  const lock = (o: Offer) => void decide({ kind: "lock", slot: o.slot });
  const lockLabel = (o: Offer) => (o.locked ? t("board.unlock") : t("board.lock"));
  // The last shop round has no Lock: the Crown clears the offers. Unlock stays.
  const lastShop = run.round >= rules.rounds;
  const canLock = (o: Offer) => !run.gift && (o.locked === true || !lastShop);

  const line = h("div", { class: "tb-row tb-line", "data-testid": "line" });
  const bench = h("div", { class: "tb-row tb-bench", "data-testid": "bench" });
  const hintSlot = h("div", { class: "hint-slot tb-coach" });
  const awoken = board.filter((u) => u.kind === "unit" && u.form === "awoken").length;
  // Desktop: the inspector, the card under the mouse, an offer clicked, a fusion waiting for its Fuse.
  const inspector = h("aside", { class: "inspector tb-insp stack", "data-testid": "inspector", hidden: "" });
  type At = { kind: "line"; index: number } | { kind: "offer"; slot: number };
  let chosen: number | null = run.offers[offer] ? offer : null;
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
  /** The piece's side per layout (mission #1286 slice 2): the phone's line of
   * five keeps the when tab's icon only, the bench is a size down. */
  /** A panel is open: it docks on the right and the field narrows, so the line's pieces are drawn smaller. */
  const docked = () => chosen !== null || pick.mode !== "none" || fuseView !== null;
  /** With a panel open the columns are narrow: a When word stays wherever it fits its column (10 characters at the floor size). */
  const whenFits = (u: LineUnit) => lineToPiece(u, content).face.when.label.length <= 10;
  const pieceSizes = () =>
    tall || !lay ? { line: 56, bench: 44, offer: 64, their: 48, lineWord: false, benchWord: false, benchBare: true } : { line: docked() ? lay.sDock : lay.s, bench: lay.sb, offer: 72, their: lay.sb, lineWord: !docked(), benchWord: false, benchBare: true };
  /** Tap-tap: the offer in hand (chosen) or the unit in hand (picked) lights
   * every slot it can go to; the twin it merges into is gold. */
  /** The board slot an offer's copy merges into, or -1. */
  const twinSlot = (unitId: string) => {
    const i = mergeTarget(board, unitId);
    return i < 0 ? -1 : i < run.line.length ? i : L + i - run.line.length;
  };
  /** A cell's height for a piece of side `s`: the tab above, the rim numbers, the name and the foot below. */
  const floor = tall ? 0 : 18;
  /** The inspector's piece is drawn at 0.72 on the portrait stage: its text floor grows to read 11 px there. */
  const sheetFloor = tall ? 16 : 18;
  /** The panel's sheet: compact (the piece beside the head), its hero a piece of 96 stage px (a phone's 56). */
  const sheetOpts = (where?: string) => ({ floor: sheetFloor, compact: true, heroPx: tall ? 60 : 88, heroWord: !tall, ...(where ? { where } : {}) });
  const cellStyle = (s: number, bare = false) => {
    const r = pieceReach(s, floor);
    const top = Math.max(15, r.above + 2);
    return `--s:${s}px;padding-top:${top}px;height:${Math.round(top + r.below + 7 + (bare ? 0 : floor ? 30 : 20))}px`;
  };
  /** A buy goes to the line while it has room (the audit's "slot 5 went to the bench" was a tap on the lit bench beside it): the bench takes one only once the line is full, or to merge into a twin there. */
  const benchWhileLineHasRoom = (o: Offer, slot: number) => slot >= L && run.line.length < L && twinSlot(o.unitId) !== slot;
  /** The coach's one slot for the offer it points at: the first free line slot, which is right behind the anchor (arena#31: players put Medic in front of Gambler). Null: no limit. */
  const coachSlotFor = (o: Offer): number | null => (coachNow && coachNow.step !== "fight" && coachNow.offer === o.slot ? run.line.length : null);
  const lightTarget = (el: HTMLElement, slot: number) => {
    const o = chosen !== null ? run.offers[chosen] : undefined;
    const u = unitAt(slot);
    if (o) {
      const only = coachSlotFor(o);
      if (only !== null && slot !== only) return;
      if (benchWhileLineHasRoom(o, slot)) return;
      if (buyBlock(o) && !(u && twinSlot(o.unitId) === slot && run.gold >= o.cost && !run.gift)) return;
      if (u && twinSlot(o.unitId) === slot) {
        el.classList.add("lit", "lit-twin");
        el.append(h("span", { class: "lit-tag" }, u.kind === "unit" && u.form === "sleeping" && u.copies + 1 >= rules.copiesToAwaken ? t("slot.awakens") : t("slot.merges")));
      } else if (!u || !owns(o.unitId)) el.classList.add("lit");
    } else if (pick.mode === "picked" && pick.index !== slot && destFor(run, pick.index, slot) !== null && !u) el.classList.add("lit");
  };
  /** The second tap of tap-tap, on a slot: buys the offer in hand there, or
   * moves the unit in hand there. False: not a second tap (the slot's own click goes on). */
  const tapSlot = (slot: number): boolean => {
    if (crown) return false;
    const o = chosen !== null ? run.offers[chosen] : undefined;
    if (o) {
      const u = unitAt(slot);
      if (benchWhileLineHasRoom(o, slot)) return false;
      // An offer you own goes to its twin; a tap on another unit of yours picks that one instead.
      if (u && owns(o.unitId) && twinSlot(o.unitId) !== slot) return false;
      if (u && !owns(o.unitId) && run.line.length >= L && run.bench.length >= B) return false;
      play("click");
      buyAt(o, slot);
      return true;
    }
    if (pick.mode === "picked" && pick.index !== slot && !unitAt(slot) && destFor(run, pick.index, slot) !== null) {
      play("click");
      moveTo(pick.index, slot);
      return true;
    }
    return false;
  };
  /** While the coach speaks (it names the one slot to use), no other slot glows and Fight does not warn. Fight with empty line slots and gold left warns (arena#15, #16): a first
   * round lost with one or two units was the commonest unexplained loss. */
  const wantsFill = () => !coachNow && !crown && !run.gift && run.line.length > 0 && run.line.length < L && run.offers.some((o) => o.cost <= run.gold);
  const slotCard = (slot: number): HTMLElement => {
    const u = unitAt(slot);
    const onBench = slot >= L;
    const id = onBench ? `bench-${slot - L}` : `line-${slot}`;
    if (!u) {
      // A line slot left empty while gold can still fill it glows (arena#16), with Fight's warning.
      const want = !onBench && wantsFill();
      const e = h("div", { class: want ? "cell empty want" : "cell empty", "data-testid": `${id}-empty`, style: onBench ? cellStyle(pieceSizes().bench, pieceSizes().benchBare) : cellStyle(pieceSizes().line) }, h("div", { class: "cell-well" }, onBench ? "" : `${slot + 1}`));
      if (!crown) dropTarget(e, slot);
      if (!crown) lightTarget(e, slot);
      e.addEventListener("click", () => tapSlot(slot));
      return e;
    }
    const sz = pieceSizes();
    const c = actionable(
      h("div", { class: ["cell", "you", onBench && sz.benchBare ? "bare" : "", u.kind === "fused" ? "fused" : u.form === "awoken" ? "awoken" : ""].filter(Boolean).join(" "), "data-copies": String(u.copies), "data-testid": id, style: onBench ? cellStyle(sz.bench, sz.benchBare) : cellStyle(sz.line) }, piece(lineToPiece(u, content), { size: "L", side: "you", px: onBench ? sz.bench : sz.line, word: onBench ? sz.benchWord : sz.lineWord || whenFits(u), floor })),
      `${u.name}, ${onBench ? t("board.onBench", { n: slot - L + 1 }) : t("board.inLine", { n: slot + 1 })}`,
    );
    if (pick.mode === "picked" && pick.index === slot) c.classList.add("selected");
    if (!crown) lightTarget(c, slot);
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
      if (tapSlot(slot)) return;
      play("click");
      if (pick.mode === "fuse") {
        if (pick.first === slot) pick = { mode: "none" };
        else if (fusesWith(pick.first, slot)) return void fusePreview(pick.first, slot);
        else return;
      } else if (!crown && pick.mode === "picked" && pick.index >= L && !onBench && run.line.length >= L) {
        // A bench unit in hand and the line full: a tap on a line unit swaps the two.
        return moveTo(pick.index, slot);
      } else
        // A tap keeps the unit in hand (the inspector reads it), so the next B, S or F still has one (Esc lets go).
        pick = { mode: "picked", index: slot };
      chosen = null;
      renderLine();
    });
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
    renderInspector(true);
    // The coach card in the tray: a gift waiting, else the hint of the moment, else how a buy works. With
    // something in hand and a slot lit it says what a tap on that slot does.
    const banner = giftBanner();
    const lit = line.querySelector(".lit") || bench.querySelector(".lit");
    const step = !lit ? null : chosen !== null ? t("hint.tapSlotBuy") : pick.mode === "picked" ? t("hint.tapSlotMove") : null;
    const said = banner ? null : (pick.mode === "picked" ? shopHint() : null) ?? step ?? shopHint();
    // The first run's coach (arena#31) is the tray's card: its step, the pair, why.
    hintSlot.replaceChildren(banner ?? (coachNow ? coachView(coachNow, content, { slot: run.line.length + 1, keys: kb }) : hint(said ?? (kb ? t("hint.buyDesk") : t("shop.howPhone")))));
    hintSlot.classList.toggle("quiet", !banner && !coachNow && said === null);
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
    const only = coachSlotFor(o);
    if (only !== null && to !== only && coachNow && coachNow.step !== "fight")
      return void ((err.textContent = t(coachNow.step === "buy-first" ? "coach.slotFirst" : "coach.slotBehind", { anchor: coachNow.anchor, answer: coachNow.answer, n: only + 1 })), play("wrong"));
    const d = { kind: "buy", slot: o.slot } as const;
    /** The server's side of it: the buy, then the move to where it was dropped. */
    const buyThenMove = async () => {
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
          // The buy stands even if the move is refused: draw the shop as bought, never the one before it.
          res = await api.decide(run.runId, { kind: "reorder", from, to: dest }).then((r) => ((sel = dest), r), () => res);
        }
      }
      play(shopSound(d, run, res.run));
      shopScreen(res.run, content, "", sel, -1);
    };
    void guarded(err, async () => {
      // The unit is drawn where it goes as bought, before the server answers (the move is predicted too).
      const bought = predictRun(run, d, content, rules);
      if (bought) {
        const had = new Set(board.map((u) => u.uid));
        const now = [...bought.line, ...bought.bench];
        const j = now.findIndex((u) => !had.has(u.uid));
        const from = j < 0 ? -1 : j < bought.line.length ? j : L + j - bought.line.length;
        const dest = from < 0 ? null : destFor(bought, from, to);
        const moved = dest === null ? bought : predictRun(bought, { kind: "reorder", from, to: dest }, content, rules) ?? bought;
        shopScreen(moved, content, "", dest ?? -1, -1);
      }
      try {
        await Promise.allSettled([...previewsOut]);
        await buyThenMove();
      } catch (e) {
        if (bought && !(e instanceof ApiError && e.status === 401)) {
          play("wrong");
          shopScreen(run, content, e instanceof ApiError && e.status === 409 ? plainRefusal(e.message) : e instanceof Error ? e.message : String(e));
        }
        throw e;
      }
    });
  };

  const actionButtons = (): HTMLElement[] => {
    if (pick.mode === "fuse") return [h("span", { class: "dim grow" }, kb ? t("board.fuseSecondDesk") : t("board.fuseSecondPhone")), button(kb ? t("board.cancelDesk") : t("board.cancel"), () => ((pick = { mode: "none" }), renderLine()), "", "fuse-cancel")];
    if (pick.mode !== "picked") return [];
    const i = pick.index;
    const u = unitAt(i)!;
    if (crown) return [];
    const left = button(kb ? "◀ ←" : "◀", () => void decide({ kind: "reorder", from: i, to: i - 1 }, i - 1), "", "move-left");
    const right = button(kb ? "→ ▶" : "▶", () => void decide({ kind: "reorder", from: i, to: i + 1 }, i + 1), "", "move-right");
    left.disabled = i === zoneStart(i);
    right.disabled = i >= zoneStart(i) + zoneLen(i) - 1;
    // The inspector already is the sheet: no Info.
    const out: HTMLElement[] = [left, right];
    const other = otherZone(i);
    if (other !== null) {
      // Into the other zone's first empty slot, else a swap with its back unit: the button says which.
      const swaps = unitAt(other) !== undefined;
      // The phone's row of buttons stays one row: "Swap" there, the hint says with whom.
      const word = i < L ? (swaps ? (kb ? t("board.swapBench", { n: B }) : t("board.swap")) : t("board.toBench")) : swaps ? (kb ? t("board.swapBack") : t("board.swap")) : t("board.toLine");
      out.push(button(kb ? `${word} · B` : word, () => moveTo(i, other), "", i < L ? "to-bench" : "to-line"));
    }
    if (canFuse(i)) out.push(button(kb ? t("board.fuseDesk") : t("board.fuse"), () => ((pick = { mode: "fuse", first: i }), renderLine()), "", "fuse"));
    const value = sellValue(rules, u);
    out.push(button(kb ? t("board.sellDesk", { n: value }) : t("board.sell", { n: value }), () => void decide({ kind: "sell", index: i }), "danger", "sell"));
    return out;
  };

  /** Desktop: a unit's win and pick rates are a subtle hint (docs/round2/
   * README.md): the inspector's last line, at its foot, not mid-panel. */
  const ratesToFoot = () => {
    const r = inspector.querySelector('[data-testid="unit-rates"]');
    // The sheet's last line, in its body (the compact sheet lets it share a row with the small facts).
    if (r) (inspector.querySelector(".sheet-compact > .sheet-body") ?? inspector).append(r);
  };
  /** Desktop: the hovered card, else the selected one (or a fusion waiting
   * for its Fuse), else how to use the board. `force` redraws even when the
   * same card is shown (the selection changed). */
  let inspected = "";
  /** A cue that the text goes on (a phone's sheet scrolls only when the rule is taller than it): a fade and a chevron at the foot of the body. */
  const cueScroll = () => {
    const body = inspector.querySelector<HTMLElement>(".insp-body");
    inspector.querySelector('[data-testid="insp-more"]')?.remove();
    if (!body || body.scrollHeight <= body.clientHeight + 1) return;
    const cue = h("div", { class: "insp-more", "data-testid": "insp-more", "aria-hidden": "true" }, "▾");
    inspector.append(cue);
    const sync = () => cue.classList.toggle("done", body.scrollTop + body.clientHeight >= body.scrollHeight - 2);
    body.addEventListener("scroll", sync);
    sync();
  };
  /** The body's text changes (See Awoken, the preview arriving): the cue is worked out again. */
  const cueWatch = new MutationObserver(() => requestAnimationFrame(cueScroll));
  const layoutInspector = () => {
    inspector.querySelector(":scope > button.insp-x")?.remove();
    const rows = [...inspector.children].filter((c) => c.classList.contains("row"));
    const body = h("div", { class: "insp-body", "data-scroll": "", style: "--scroll-cue:1" }, ...[...inspector.children].filter((c) => !rows.includes(c)).map((c) => c as HTMLElement));
    inspector.replaceChildren(...(body.childElementCount ? [body] : []), ...rows);
    cueWatch.disconnect();
    if (body.childElementCount) cueWatch.observe(body, { childList: true, subtree: true, characterData: true });
    if (inspector.isConnected) cueScroll();
  };
  const renderInspector = (force = false) => {
    drawInspector(force);
    inspector.hidden = inspector.childElementCount === 0;
    // The field narrows to make room for a docked panel (landscape).
    inspector.closest(".stage")?.classList.toggle("with-panel", !inspector.hidden);
    // What reads scrolls; the buttons (Buy, Lock, move, sell) stay at the card's foot.
    layoutInspector();
    // A close on the card (the mockup's "Esc ✕"): the offers under it on a phone are out of reach otherwise.
    if (!inspector.hidden) inspector.prepend(closeBtn());
  };
  const closeBtn = () =>
    button(
      "✕",
      () => {
        fuseView = null;
        pick = { mode: "none" };
        chosen = null;
        renderLine();
      },
      "insp-x",
      "inspector-close",
    );
  const drawInspector = (force: boolean) => {
    if (fuseView) return inspector.replaceChildren(fuseView);
    const at: At | null = (pick.mode === "picked" ? { kind: "line", index: pick.index } : chosen !== null ? { kind: "offer", slot: chosen } : null);
    const key = at ? `${at.kind}:${at.kind === "line" ? at.index : at.slot}` : "none";
    if (key === inspected && !force) return;
    inspected = key;
    const sel = pick.mode === "picked" ? pick.index : -1;
    if (at?.kind === "line" && unitAt(at.index)) {
      const mine = at.index === sel;
      inspector.replaceChildren(
        h("div", { class: "label" }, (mine ? t("board.selected") : "") + (at.index < L ? t("board.inLine", { n: at.index + 1 }) : t("board.onBench", { n: at.index - L + 1 }))),
        unitSheet(unitAt(at.index)!, content, sheetOpts(at.index < L ? t("panel.slot", { n: at.index + 1 }) : t("panel.bench", { n: at.index - L + 1 }))),
        mine || pick.mode === "fuse" ? h("div", { class: "row actions", "data-testid": "actions" }, ...actionButtons()) : h("div", { class: "dim small" }, crown ? t("board.crownFinal") : t("board.clickSelect")),
      );
      return ratesToFoot();
    }
    if (at?.kind === "offer") {
      const n = run.offers.findIndex((o) => o.slot === at.slot);
      const o = run.offers[n];
      if (!o) return;
      const head = h("div", { class: "label" }, t("board.offerHead", { n: n + 1 }));
      /** The card: its sheet, then Lock and Buy at the foot. */
      const card_ = (sheet: HTMLElement, blocked: string) => {
        const buy = button(buttonRefusal(blocked) || t("board.buyDesk", { cost: o.cost, n: n + 1 }), () => void decide({ kind: "buy", slot: o.slot }), "primary grow", "buy");
        buy.disabled = blocked !== "";
        const lockBtn = canLock(o) ? [button(`${lockLabel(o)} · L`, () => lock(o), "", "lock")] : [];
        inspector.replaceChildren(head, sheet, h("div", { class: "row" }, ...lockBtn, buy));
        ratesToFoot();
        layoutInspector();
        inspector.prepend(closeBtn());
      };
      // At once: the unit's own rule and numbers (your copy, if you own one); what buying does follows with the preview.
      const own = board.find((x) => x.kind === "unit" && x.unitId === o.unitId);
      const pool = unitOf(o.unitId);
      card_(own ? unitSheet(own, content, sheetOpts(t("panel.offer", { n: n + 1 }))) : pool ? unitSheet(pool, content, sheetOpts(t("panel.offer", { n: n + 1 }))) : h("h2", {}, o.unitId), buyBlock(o));
      void offerBody(o).then(
        ({ sheet, blocked }) => {
          // What buying does arrives after the rule: swapped in once no term's tip is open on it (the swap would close the tip under the reader's finger).
          const swap = () => {
            if (inspected !== key) return;
            if (document.querySelector('[data-testid="term-popover"]')) return void setTimeout(swap, 300);
            card_(sheet, blocked);
          };
          swap();
        },
        (e: unknown) => {
          if (inspected === key) inspector.append(h("div", { class: "error" }, e instanceof Error ? e.message : String(e)));
        },
      );
      return;
    }
    inspector.replaceChildren(...(pick.mode === "fuse" ? [h("div", { class: "row actions", "data-testid": "actions" }, ...actionButtons())] : []));
  };

  /** The hint that matters most right now, or none. */
  const shopHint = (): string | null => {
    if (!crown && pick.mode === "picked" && pick.index >= L && run.line.length >= L) return t("hint.lineFullSwap");
    if (pick.mode !== "none") return null;
    // run.ts refuses every decision but the fight in the crown phase: the line is final.
    if (crown) return ownCrown ? t("hint.ownCrown") : t("hint.crown");
    // Only a unit whose next copy is on offer right now.
    const almost = board.find((u) => u.kind === "unit" && u.form === "sleeping" && u.copies === rules.copiesToAwaken - 1 && run.offers.some((o) => o.unitId === u.unitId));
    const canBuy = run.offers.some((o) => o.cost <= run.gold);
    if (anyFuse()) return kb ? t("hint.fuseDesk") : t("hint.fusePhone");
    if (almost && run.offers.some((o) => o.unitId === almost.unitId && o.cost <= run.gold)) return t("hint.oneMore", { name: almost.name });
    if (run.line.length === 0 && run.bench.length > 0) return t("hint.moveToLine");
    if (run.line.length === 0 && !canBuy) return t("hint.noGold");
    if (run.line.length === 0) return kb ? t("hint.buyDesk") : t("hint.buyPhone");
    if (run.round === 1 && run.line.length > 0 && run.gold < rules.unitCost) return t("hint.outOfGold");
    if (run.line.length > 1 && run.round <= 2) return kb ? t("hint.moveDesk") : t("hint.movePhone");
    if (almost) {
      const o = run.offers.find((o) => o.unitId === almost.unitId)!;
      const keep = !o.locked && !lastShop && o.cost > run.gold ? t("hint.lockToKeep") : "";
      return t("hint.oneMoreCost", { name: almost.name, cost: o.cost, keep });
    }
    const keepIt = lastShop ? undefined : run.offers.find((o) => !o.locked && o.cost > run.gold && owns(o.unitId));
    if (keepIt) return t("hint.ownedCosts", { name: unitOf(keepIt.unitId)?.name ?? t("hint.yourUnit"), cost: keepIt.cost, gold: run.gold });
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
          unitSheet(fused, content, { preview: true, floor: sheetFloor }),
        );
      };
      render();
      const head = h("div", { class: "label" }, t("fuse.preview"));
      pick = { mode: "none" };
      fuseView = h("div", { class: "stack", "data-testid": "fuse-preview" }, head, body);
      const close = () => ((fuseView = null), renderLine());
      renderLine();
      focusPreview();
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
        unitSheet(fused, content, { preview: true, floor: sheetFloor }),
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
          unitSheet(chosen, content, { preview: true, floor: sheetFloor }),
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
    fuseView = h("div", { class: "stack", "data-testid": "fuse-preview" }, head, body);
    close = () => ((fuseView = null), renderLine());
    renderLine();
    focusPreview();
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
        unitSheet(fused, content, { floor: sheetFloor }),
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
    const where = t("panel.offer", { n: run.offers.findIndex((x) => x.slot === o.slot) + 1 });
    const sheet = mine ? unitSheet(mine.next, content, { from: mine.now.stats, ...sheetOpts(where) }) : owned ? unitSheet(owned, content, sheetOpts(where)) : u ? unitSheet(u, content, sheetOpts(where)) : h("h2", {}, o.unitId);
    // What buying does goes inside the sheet, above its last line (the rates hint).
    // The XL sheet's text column holds the rates line (mission #1286).
    const body = sheet.querySelector(".sheet-body") ?? sheet;
    const rates = body.querySelector(':scope > [data-testid="unit-rates"]');
    if (after) body.insertBefore(after, rates);
    const fusions = offerFusions(o.slot);
    if (fusions.length) body.insertBefore(h("div", { class: "stack offer-fusions", "data-testid": "offer-fusions" }, ...fusions.map((f) => h("div", { class: `small${f.awakens ? "" : " dim"}` }, `⚗ ${offerLine(f)}`))), rates);
    return { sheet, blocked };
  };
  const offerSlots = offerPositions(run);
  // The shop row has the tray's width: more offers than three shrink their pieces to fit.
  const manyOffers = offerSlots.length > 3;
  const offerPx = Math.min(pieceSizes().offer, Math.floor((tall ? 366 : manyOffers ? 626 : 462) / offerSlots.length) - 14);
  const offers = h(
    "div",
    { class: "tb-row tb-offers", style: `--n:${offerSlots.length}`, "data-testid": "offers" },
    // Every slot keeps its place: a bought offer leaves an empty slot, the rest never shift (CrazyGames audit, arena#14).
    ...offerSlots.map((at, pos) => (at >= 0 ? run.offers[at]! : pos)).map((o: Offer | number) => {
      if (typeof o === "number") return h("div", { class: "cell offer-gap", "data-testid": `offer-gap-${o}`, "aria-hidden": "true", style: cellStyle(offerPx) });
      const u = unitOf(o.unitId);
      const owned = owns(o.unitId);
      const price = `${t("shop.gold", { n: o.cost })}${owned ? " ＋" : ""}`;
      // The piece (mission #1286): price and lock on its foot, the tier beside them.
      const pu = { name: u?.name ?? o.unitId, emoji: u?.emoji ?? "?", stats: u?.base ?? { pwr: 0, hp: 0 }, face: u ? faceOfUnit(u) : { when: { icon: "✨", label: "" }, headline: { icon: "✨" } }, tier: o.tier, copies: 1 };
      const c = actionable(
        h("div", { class: "cell offer", "data-testid": `offer-${o.slot}`, "data-name": pu.name, style: cellStyle(offerPx) }, piece(pu, { size: "L", side: "shop", px: offerPx, floor, word: !tall && offerPx >= 68, price: o.cost, lock: o.locked === true })),
        `${pu.name}, ${price}`,
      );
      if (run.gold < o.cost) c.classList.add("poor");
      if (o.locked) c.classList.add("locked");
      if (owned) c.classList.add("owned");
      // Feedback on every tap (CrazyGames audit, arena#13): the card pulses once.
      c.addEventListener("click", () => (c.classList.remove("tapped"), void c.offsetWidth, c.classList.add("tapped")));
      c.addEventListener("animationend", () => c.classList.remove("tapped"));
      if (!crown) dragCard(c, { kind: "offer", offer: o });
      // Tap-tap: the first tap takes it in hand (the inspector reads it) and lights the slots it can
      // go to; a tap on a lit slot buys it there; a second tap on it puts it down.
      c.addEventListener("click", () => {
        play("click");
        if (chosen !== o.slot) {
          // Feedback on every tap: a card you can't buy says why, right away.
          const why = buyBlock(o);
          if (why) (err.textContent = why), play("wrong"), c.classList.add("nudge"), setTimeout(() => c.classList.remove("nudge"), 400);
        }
        chosen = chosen === o.slot ? null : o.slot;
        if (pick.mode === "picked") pick = { mode: "none" };
        renderLine();
      });
      c.addEventListener("dblclick", () => buy(o));
      c.addEventListener("contextmenu", (e) => (e.preventDefault(), canLock(o) && lock(o)));
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
  // A fight with gold to spend and empty line slots says so on the button
  // (arena#15): a first round lost with one unit was the commonest unexplained loss.
  const emptySlots = L - run.line.length;
  const fight = button(crown ? t("shop.fightChampion") : t("shop.fight"), () => void decide({ kind: "fight" }), "primary grow", "fight");
  if (wantsFill()) {
    fight.classList.add("half-empty");
    fight.append(h("span", { class: "fight-warn", "data-testid": "fight-warn" }, t("shop.fillFirst", { n: emptySlots })));
  }
  // An empty line can fight (and lose a heart) once nothing is affordable, so a broke run moves on.
  fight.disabled = (run.line.length === 0 && run.offers.some((o) => o.cost <= run.gold)) || !!run.gift;
  if (kb) {
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
  /** One unit of the Crown's team, as a piece on their rim; a tap reads it in a sheet. */
  const theirCell = (u: LineUnit) =>
    actionable(
      h("div", { class: "cell them bare", style: cellStyle(pieceSizes().their, true) }, piece(lineToPiece(u, content), { size: "L", side: "them", px: pieceSizes().their, word: false, floor })),
      u.name,
    );
  const fillFoe = (d: DayView | null, settled = d !== null) => {
    if (!foe || !opp) return;
    const ch = d?.champion;
    const line = ch && ch.player.id === opp.player.id ? ch.line : null;
    foe.replaceChildren(
      line
        ? h("div", { class: "crown-foe-line tb-their-cells", "data-testid": "crown-foe-line", "aria-label": `${ownCrown ? t("foe.ownCrown") : t("foe.crown")} ${opp.player.name}` }, ...line.map((u) => theirCell(u)))
        : h("div", { class: "dim small foe-wait" }, settled ? t("foe.lineInFight") : "…"),
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
      if (!tall) {
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
      h("div", { class: "dim small" }, tall ? t("gift.readPhone") : t("gift.readDesk")),
      h(
        "div",
        { class: "row sheet-actions" },
        stuck ? button(t("gift.makeRoom"), aside, "grow", "gift-make-room") : null,
        button(t("gift.skip"), () => (close(), void decide({ kind: "gift", pick: null })), stuck ? "" : "grow", "gift-skip"),
      ),
    ];
    const close = !tall ? dismissable(aside, h("div", { class: "gift-split" }, h("div", { class: "stack gift-main" }, ...kids), read)) : dismissable(aside, ...kids);
    if (!tall) {
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
  if (kb) (menuBtn.title = t("shop.menuEsc"), withKey(menuBtn, "Esc"));

  const nextOpp = h("span", { class: "dim next-opp", "data-testid": "next-opponent" }, ...(opp ? [`${crown ? t("opp.crownVs") : t("opp.next")} `, who(opp.player.name), `${opp.player.bot ? " 🤖" : ""}${ownCrown ? t("opp.yourChampionTeam") : ""}`] : [crown ? t("opp.crownVsChampion") : t("opp.nextSaved")]));
  // The table (arena#28): one stage. The HUD's plates on top (menu, the round track, hearts at
  // every size, gold), their line and your line (the bench beside it) in the middle, and the
  // tray at the foot: the coach's hint, the shop, Reroll and Fight. The inspector floats over it.
  const plate = (cls: string, ...kids: (Node | null)[]) => h("div", { class: `plate ${cls}` }, ...kids);
  const hud = h(
    "div",
    { class: "tb-hud", "data-testid": "hud" },
    plate("menu", menuBtn),
    tall ? null : plate("help", legendBtn),
    plate("round", tall ? h("span", { class: "round-pill", "data-testid": "round" }, roundLabel(run.round)) : runTrack(run)),
    h("span", { class: "hud-gap" }),
    plate("hearts", heartIcons(run.hearts)),
    // The Crown has no shop: no gold to show.
    crown ? null : plate("coins", h("span", { class: "coin", "aria-hidden": "true" }), h("span", { class: "gold", "data-testid": "gold" }, t("shop.gold", { n: run.gold }))),
  );
  const theirs = h(
    "div",
    { class: "tb-their", "data-testid": "their-line" },
    // Their line stays hidden until the fight (the Crown shows the champion's team); wells keep its place.
    ...(crown ? [foe!] : Array.from({ length: L }, (_, i) => h("div", { class: "cell empty ghost", "data-testid": `their-${i}`, "aria-hidden": "true", style: lay ? `--s:${Math.round(lay.wellsH * 0.8)}px;padding-top:4px;height:${lay.wellsH}px` : cellStyle(pieceSizes().their) }, h("div", { class: "cell-well" }, "?")))),
  );
  const table = stage(
    [
      hud,
      h("div", { class: "tb-side them" }, nextOpp, crown ? null : pin),
      theirs,
      h("div", { class: "tb-side you" }, h("div", { class: "label" }, crown ? t("line.crown") : t("line.phone"))),
      tall ? plate("help tb-help", legendBtn) : null,
      line,
      B > 0 ? bench : null,
      h(
        "div",
        { class: manyOffers ? "tb-tray many" : "tb-tray", "data-testid": "tray" },
        hintSlot,
        crown ? null : h("div", { class: "tb-shop" }, h("div", { class: "label" }, t("shop.offersDesk", { n: run.offers.length })), offers),
        h("div", { class: "tb-go" }, crown ? null : reroll, fight),
      ),
      inspector,
      err,
    ],
    () => shopScreen(run, content, err.textContent ?? "", pick.mode === "picked" ? pick.index : -1, chosen ?? -1),
    !tall,
  );
  table.classList.add("t-shop");
  table.classList.toggle("with-panel", !inspector.hidden);
  cueScroll();
  if (lay) for (const [k, v] of Object.entries({ "wells-top": lay.wellsTop, "wells-h": lay.wellsH, "line-top": lay.lineTop, "bench-top": lay.benchTop, "tray-top": lay.trayTop })) table.style.setProperty(`--${k}`, `${v}px`);
  show(table);
  screen("shop");
  if (coachNow) mountCoach(coachNow);
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
  // With a keyboard: ← → pick an offer (or move the unit in hand), 1–5 put the offer in hand
  // into that slot (or pick the unit there), R reroll, L lock the chosen offer, Space fight, F fuse, S sell.
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
    // 1–5 are the line, 6–8 the bench slots after it (the board's own numbering).
    if (/^[1-9]$/.test(k) && Number(k) <= L + B) {
      const slot = Number(k) - 1;
      if (tapSlot(slot)) return true;
      if (pick.mode === "none" && chosen === null && unitAt(slot)) pick = { mode: "picked", index: slot };
      else {
        err.textContent = chosen !== null ? t("hint.tapSlotBuy") : t("shop.selectFirst");
        play("wrong");
        return true;
      }
      renderLine();
      return true;
    }
    if (k === "r") {
      if (!reroll.disabled) void decide({ kind: "reroll" });
      else play("wrong");
      return true;
    }
    if (k === "l") {
      const o = chosen === null ? undefined : run.offers[chosen];
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
    // No unit in hand: ← → walk the shop's offers, taking each in hand.
    if ((k === "ArrowLeft" || k === "ArrowRight") && run.offers.length > 0 && pick.mode === "none") {
      const n = run.offers.length;
      const at = chosen === null ? -1 : run.offers.findIndex((o) => o.slot === chosen);
      const next = at < 0 ? (k === "ArrowRight" ? 0 : n - 1) : (at + (k === "ArrowRight" ? 1 : n - 1)) % n;
      chosen = run.offers[next]!.slot;
      play("click");
      renderLine();
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
desktopQuery.addEventListener("change", () => (app.dataset.screen === "codex" ? codexRedraw?.() : undefined));

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
  const plate = (cls: string, ...kids: HTMLElement[]) => h("div", { class: `plate ${cls}` }, ...kids);
  return [
    plate("round", orient() === "P" ? h("span", { class: "round-pill", "data-testid": "round" }, roundLabel(fought.round)) : runTrack(run)),
    plate("hearts", heartIcons(run.hearts)),
    ...(run.phase === "crown" || run.round > rules.rounds ? [] : [plate("coins", h("span", { class: "coin", "aria-hidden": "true" }), h("span", { class: "gold", "data-testid": "gold" }, t("shop.gold", { n: run.gold })))]),
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
/** `#gallery[=L|M|S|XS]` (#1291): every unit as a piece at every size, no
 * sign-in, for judging the piece and screenshot tests (./screens/gallery.ts). */
const galleryHash = new URLSearchParams(location.hash.slice(1));
const galleryLink = galleryHash.has("gallery");
function openGallery(): void {
  const size = galleryHash.get("gallery");
  const err = errorLine();
  void guarded(err, async () => galleryScreen(await getContent(), size === "L" || size === "M" || size === "S" || size === "XS" ? size : null, galleryHash.get("detail"), galleryHash.has("on"))).then(() => {
    if (err.textContent) show(h("h1", {}, "Gallery"), err);
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
        else if (entry.click) settle(() => document.querySelector<HTMLElement>(`[data-testid="${entry.click}"]`)?.click());
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
  // Open to all, no join link: the first screen (arena#31).
  if (code === "")
    return firstScreen(async (name) => {
      const s = await api.join(code, name);
      history.replaceState(null, "", ownLink(s.invite));
    }, [tg]);
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
  if (hash.has("invite") || hash.has("join") || hash.has("battle") || hash.has("scene") || hash.has("checkpoint") || hash.has("gallery")) location.reload();
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
  // A slow server (a guest is made, Home is fetched) leaves the title alone on a dark screen: say it is loading.
  show(h("h1", {}, t("name.title")), h("p", { class: "hint", "data-testid": "loading" }, t("name.loading")));
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
  if (galleryLink) {
    openGallery();
    return;
  }
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
