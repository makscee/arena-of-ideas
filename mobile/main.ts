// Arena MVP phone client: name → home → shop → fight → battle → result.
// Plain DOM; each screen is a function that renders into #app.
// Owners: slice 8 the name, home, shop and result screens here; slice 9 the
// battle viewer (screens/battle.ts); slice 11 the stats page
// (screens/stats.ts). Shared: api.ts, content.ts, ui/.
// Home also shows DayView.lastPlayoff (slice 5 fills it; a game opens in
// battleScreen) and, under "Dev", an "End day now" button (api.endDay(): 404
// without MVP_DEV=1, 501 until slice 5).
import type { BattleRecord, DayView, FightResult, HomeView, LineUnit, MvpContent, MvpRules, Offer, PlayerRef, PlayoffResult, RunView } from "../src/mvp/contract";
import { MVP_RULES, offersAt } from "../src/mvp/contract";
import { ApiError, api } from "./api";
import { getContent } from "./content";
import { battleScreen, whyILost } from "./screens/battle";
import { statsScreen } from "./screens/stats";
import { card, unitSheet, type CardUnit } from "./ui/card";
import { app, button, closable, h, overlay, show, who } from "./ui/dom";
import { loadUnitRates } from "./ui/unit-stats";

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
    err.textContent = e instanceof Error ? e.message : String(e);
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
const roundLabel = (round: number) => (round > rules.rounds ? "CROWN" : `R${round}/${rules.rounds}`);
const hearts = (n: number) => h("span", { class: "hearts", "aria-label": plural(n, "heart") }, "♥".repeat(n) + "♡".repeat(Math.max(0, rules.hearts - n)));
const openSheet = (u: Parameters<typeof unitSheet>[0], content: MvpContent) => () => closable(unitSheet(u, content));

/** A line of cards, each opening its unit sheet. */
function team(line: LineUnit[], side: "you" | "ghost", content: MvpContent, testid = ""): HTMLElement {
  return h("div", { class: "slots", ...(testid ? { "data-testid": testid } : {}) }, ...line.map((u) => card(u, { side, onOpen: openSheet(u, content) })));
}

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
    p(`${r.goldPerRound} gold every round, no carry-over. A unit costs ${r.unitCost}, a reroll ${r.rerollCost}, selling gives back ${r.sellRefund}. ${offersText(r)}; stronger tiers open as rounds pass.`),
    h("div", { class: "label" }, "The line"),
    p(`${r.lineSize} units in a line, front first. Change the order in the shop: tap a unit, then ◀ ▶. Each round you fight a team another player saved at the same round.`),
    h("div", { class: "label" }, "Copies, Awoken, fusion"),
    p(`Buying a unit you own merges it in: +${r.copyGrowth.pwr} PWR / +${r.copyGrowth.hp} HP a copy. Copy ${r.copiesToAwaken} awakens it: the same When, a stronger Who or Does.`),
    p("Two Awoken units fuse: the When of the first you tap, the Who of the second, the Does of both, stats summed. A fused unit is final; copies of either part still merge into it. The first player to make a pair names it."),
    h("div", { class: "label" }, "Chains"),
    p("Units react to events. When one happens, the units it triggers fire in line order, front to back, each at most once per event. In a fight, tap any number to see the chain that caused it."),
    h("div", { class: "label" }, "The day"),
    p(`Beat the champion in the Crown and you are a slayer. At ${r.dayEndsAt} Moscow the slayers' best teams play a round-robin, and the winner is the next champion.`),
    p("Your rating moves once per run: every fight, the Crown too, counts against its opponent's rating (Elo), added up when the run ends. Giving up counts each heart left as a lost fight."),
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
    row(h("span", { class: "stats" }, span("p", "2"), " / ", span("h", "6")), "PWR / HP. PWR is what its strike deals; at 0 HP it falls."),
    row(h("span", { class: "rates" }, span("w", "W59%"), " P9%"), "Rates from every run since the units last changed. W: how often a team with it won its fight. P: how often it was on a finished run's line. — means no runs yet."),
    row(span("copies", "●●○"), `Copies toward Awoken: copy ${r.copiesToAwaken} awakens it. Each copy adds +${r.copyGrowth.pwr} PWR / +${r.copyGrowth.hp} HP.`),
    row(span("copies tag", "AWOKEN ×3"), "Awoken, its stronger form; ×3 copies merged in. Two Awoken units can fuse."),
    row(span("copies tag", "FUSED ×2"), "Two Awoken units fused into one: final, copies of either part still merge in. \"by @name\" is who discovered it."),
    row(span("cost", "3g ＋"), "An offer's price. ＋: you own it, so buying merges a copy in."),
    h("div", { class: "dim small" }, "Tap any card for its full sheet: what it does, sleeping and Awoken."),
  );
  const close = closable(sheet, h("div", { class: "row" }, rulesBtn));
  return sheet;
}

// ---------- name ----------

function nameScreen(): void {
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

/** Home. `ended`: the day the dev "End day now" just closed, so Home says
 * how it ended at the top and brings the playoff panel into view. */
async function homeScreen(ended: number | null = null): Promise<void> {
  const err = errorLine();
  void loadUnitRates(); // the rates on unit cards (slice 11)
  const [home, content]: [HomeView, MvpContent] = await Promise.all([api.home(), getContent()]);
  rules = home.rules;
  day = home.day;
  const champ = home.day.champion;
  const r = home.rating;
  const play = home.activeRunId
    ? button("Continue", () => void guarded(err, async () => shopScreen(await api.run(home.activeRunId!), content)), "primary grow", "play")
    : button("Play", () => void guarded(err, async () => shopScreen(await api.startRun(), content)), "primary grow", "play");
  const stats = button("Stats", () => void statsScreen({ content, onBack: () => void homeScreen() }), "", "stats");
  const rulesBtn = button("Rules", () => closable(rulesSheet()), "", "rules-open");
  const record = (label: string, value: string | number, testid = "") =>
    h("div", { class: "record" }, h("div", { class: "num", ...(testid ? { "data-testid": testid } : {}) }, `${value}`), h("div", { class: "label" }, label));
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
    h("div", { class: "row spread" }, h("h1", {}, "ARENA"), who(api.player?.name ?? "", "dim")),
    // The dev "End day now" just ran: say so first, with how the day ended
    // (the playoff panel) right under it, before today's champion.
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
            ? "This is your team: today the others try to beat it in the Crown. Tap a card to read it."
            : r
              ? "Tap a card to read it. Beat this team in the Crown to become a slayer."
              : "This is the team to beat. Tap a card to read it, then Play.",
        )
      : null,
    justEnded ? null : playoff,
    h(
      "div",
      { class: "panel records", "data-testid": "records" },
      record("Rating", r?.rating ?? rules.ratingStart, "rating"),
      record("Runs", r?.runs ?? 0),
      record("Slays", r?.slays ?? 0),
      record("Days 👑", r?.daysAsChampion ?? 0),
      record("Playoff W", r?.playoffWins ?? 0),
    ),
    h("details", { class: "dev" }, h("summary", {}, "Dev"), endDay),
    err,
    h("div", { class: "spacer" }),
    // Play stays on the first screen however long Home runs (a playoff's table).
    h("div", { class: "row footer", "data-testid": "home-actions" }, rulesBtn, stats, play),
  );
  if (justEnded) playoff?.classList.add("fresh");
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

function shopScreen(run: RunView, content: MvpContent, notice = ""): void {
  if (run.phase === "over") return runOverScreen(run, content, notice);
  const crown = run.phase === "crown";
  // The reigning champion's Crown is their own team: no slay to win.
  const ownCrown = crown && run.nextOpponent?.player.id === run.player.id;
  const err = errorLine();
  err.textContent = notice;
  let pick: Pick = { mode: "none" };
  const unitOf = (id: string) => content.units.find((x) => x.id === id);
  const decide = (d: Parameters<typeof api.decide>[1]) =>
    guarded(err, async () => {
      const res = await api.decide(run.runId, d);
      if (res.fight) return fightScreens(res.run, res.fight, content);
      shopScreen(res.run, content);
    });

  const line = h("div", { class: "slots", "data-testid": "line" });
  const actions = h("div", { class: "row actions", "data-testid": "actions" });
  const hintSlot = h("div", {});
  const awoken = run.line.filter((u) => u.kind === "unit" && u.form === "awoken").length;

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
          if (pick.mode === "fuse") {
            if (pick.first === i) pick = { mode: "none" };
            else if (u.kind === "unit" && u.form === "awoken") return void fusePreview(pick.first, i);
            else return;
          } else pick = pick.mode === "picked" && pick.index === i ? { mode: "none" } : { mode: "picked", index: i };
          renderLine();
        });
        return c;
      }),
    );
    actions.replaceChildren(...actionButtons());
    hintSlot.replaceChildren(...[shopHint()].filter((x): x is HTMLElement => x !== null));
  };

  const actionButtons = (): HTMLElement[] => {
    if (pick.mode === "fuse") return [h("span", { class: "dim grow" }, "Tap the second Awoken unit."), button("Cancel", () => ((pick = { mode: "none" }), renderLine()), "", "fuse-cancel")];
    if (pick.mode !== "picked") return [];
    const i = pick.index;
    const u = run.line[i]!;
    const info = button("Info", openSheet(u, content), "", "info");
    if (crown) return [info];
    const left = button("◀", () => void decide({ kind: "reorder", from: i, to: i - 1 }), "", "move-left");
    const right = button("▶", () => void decide({ kind: "reorder", from: i, to: i + 1 }), "", "move-right");
    left.disabled = i === 0;
    right.disabled = i >= run.line.length - 1;
    const out: HTMLElement[] = [left, right, info];
    if (u.kind === "unit" && u.form === "awoken" && awoken >= 2) out.push(button("Fuse", () => ((pick = { mode: "fuse", first: i }), renderLine()), "", "fuse"));
    out.push(button(`Sell +${rules.sellRefund}`, () => void decide({ kind: "sell", index: i }), "danger", "sell"));
    return out;
  };

  /** The hint that matters most right now, or none. */
  const shopHint = (): HTMLElement | null => {
    if (pick.mode !== "none") return null;
    // run.ts refuses every decision but the fight in the crown phase: the line is final.
    if (crown) return hint(ownCrown ? "The Crown: today's champion is your own team. Beating it doesn't count as a slay. Your line is final." : "The Crown: your line, as it is, against today's champion. Win it to become a slayer; a loss costs a heart.");
    // Only a unit whose next copy is on offer right now.
    const almost = run.line.find((u) => u.kind === "unit" && u.form === "sleeping" && u.copies === rules.copiesToAwaken - 1 && run.offers.some((o) => o.unitId === u.unitId));
    const canBuy = run.offers.some((o) => o.cost <= run.gold);
    if (awoken >= 2) return hint("Two Awoken units can fuse: tap one, then Fuse.");
    if (almost && run.offers.some((o) => o.unitId === almost.unitId && o.cost <= run.gold)) return hint(`One more ${almost.name} awakens it. It's in the shop.`);
    if (run.line.length === 0 && !canBuy) return hint("No gold for a unit. Fight to move on: an empty line loses, and costs a heart.");
    if (run.line.length === 0) return hint("Tap an offer to read it and buy it. Your line fights front first.");
    if (run.round === 1 && run.line.length > 0 && run.gold < rules.unitCost) return hint("Out of gold for units. Fight when ready.");
    if (run.line.length > 1 && run.round <= 2) return hint("Tap a unit in your line to move, sell or read it.");
    if (almost) return hint(`One more ${almost.name} awakens it. It's in the shop for ${run.offers.find((o) => o.unitId === almost.unitId)!.cost}g.`);
    return null;
  };

  /** Fuse preview: the result card from a dry run, then confirm. */
  const fusePreview = (first: number, second: number) =>
    guarded(err, async () => {
      const res = await api.preview(run.runId, { kind: "fuse", first, second });
      const fused = res.run.line.find((u) => u.uid === run.line[first]!.uid) ?? res.run.line[first]!;
      const close = overlay(
        h("div", { class: "label" }, "Fusion preview"),
        h("div", { class: "preview-card" }, card(fused, { side: "you", extra: [copiesBadge(fused)] })),
        h("div", { class: "row sheet-actions" }, button("Cancel", () => close(), "grow", "preview-cancel"), button("Fuse", () => (close(), void decide({ kind: "fuse", first, second })), "primary grow", "preview-confirm")),
        unitSheet(fused, content),
      );
      pick = { mode: "none" };
      renderLine();
    });

  /** An offer's sheet: both forms, and what buying it does to your line. */
  const offerSheet = (o: Offer) =>
    guarded(err, async () => {
      const u = unitOf(o.unitId);
      const affordable = run.gold >= o.cost;
      const res = affordable ? await api.preview(run.runId, { kind: "buy", slot: o.slot }).catch((e: unknown) => (e instanceof ApiError && e.status === 409 ? e : Promise.reject(e))) : null;
      let after: HTMLElement | null = null;
      // A unit you own: the sheet shows your copy, from now to after buying.
      let mine: { now: LineUnit; next: LineUnit } | null = null;
      let blocked = affordable ? "" : `Needs ${o.cost}g`;
      if (res instanceof ApiError) blocked = res.message;
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
      const buy = button(blocked || `Buy ${o.cost}g`, () => (close(), void decide({ kind: "buy", slot: o.slot })), "primary grow", "buy");
      buy.disabled = blocked !== "";
      // Without a preview (no gold), an owned unit still shows your copy as it is.
      const owned = mine ? null : run.line.find((x) => x.kind === "unit" && x.unitId === o.unitId) ?? null;
      const close = overlay(
        ...(mine ? [unitSheet(mine.next, content, { from: mine.now.stats })] : owned ? [unitSheet(owned, content)] : u ? [unitSheet(u, content)] : [h("h2", {}, o.unitId)]),
        ...(after ? [after] : []),
        h("div", { class: "row sheet-actions" }, button("Close", () => close(), "", "offer-close"), buy),
      );
    });

  const offers = h(
    "div",
    { class: "slots", "data-testid": "offers" },
    ...run.offers.map((o: Offer) => {
      const u = unitOf(o.unitId);
      const cu: CardUnit = { unitId: o.unitId, emoji: u?.emoji ?? "?", name: u?.name ?? o.unitId, stats: u?.base ?? { pwr: 0, hp: 0 } };
      const owned = run.line.find((x) => x.unitId === o.unitId || x.fusion?.second === o.unitId);
      const c = card(cu, { side: "you", extra: [h("div", { class: "cost" }, owned ? `${o.cost}g ＋` : `${o.cost}g`)], testid: `offer-${o.slot}` });
      if (run.gold < o.cost) c.classList.add("poor");
      if (owned) c.classList.add("owned");
      c.addEventListener("click", () => void offerSheet(o));
      return c;
    }),
  );

  const reroll = button(`Reroll ${rules.rerollCost}g`, () => void decide({ kind: "reroll" }), "", "reroll");
  reroll.disabled = run.gold < rules.rerollCost;
  const fight = button(crown ? "Fight the champion" : "Fight", () => void decide({ kind: "fight" }), "primary grow", "fight");
  // An empty line can fight (and lose a heart) once nothing is affordable, so a broke run moves on.
  fight.disabled = run.line.length === 0 && run.offers.some((o) => o.cost <= run.gold);

  const opp = run.nextOpponent;
  const pin = h("div", { class: "pin", "data-testid": "champion-pin" });
  const fillPin = (d: DayView | null) => {
    const ch = d?.champion;
    if (!ch) return pin.replaceChildren(h("span", { class: "dim" }, "👑 No champion yet"));
    const b = h("button", { class: "pin-btn", "data-testid": "champion-pin-open" }, h("span", {}, "👑"), who(ch.player.name, "ghost-name"), h("span", { class: "pin-emoji" }, ch.line.map((u) => u.emoji).join("")));
    b.addEventListener("click", () => closable(h("div", { class: "label" }, `Champion of day ${d!.seq} · `, who(ch.player.name)), team(ch.line, "ghost", content), hint("Tap a card to read it.")));
    pin.replaceChildren(b);
  };
  fillPin(day);
  if (!day) void api.day().then((d) => ((day = d), fillPin(d))).catch(() => {});

  // "?" explains a card's numbers; until a player has opened it once, it says so.
  const legendBtn = button(seen("legend") ? "?" : "? Cards", () => (markSeen("legend"), (legendBtn.textContent = "?"), legendBtn.classList.remove("new"), legendSheet()), seen("legend") ? "small" : "small new", "legend-open");
  renderLine();
  show(
    h(
      "div",
      { class: "hud", "data-testid": "hud" },
      h("span", { "data-testid": "round" }, roundLabel(run.round)),
      hearts(run.hearts),
      // The Crown has no shop: no gold to show.
      crown ? h("span", {}) : h("span", { class: "gold", "data-testid": "gold" }, `${run.gold}g`),
    ),
    h(
      "div",
      { class: "row spread opp" },
      h("span", { class: "dim", "data-testid": "next-opponent" }, ...(opp ? [`${crown ? "Crown vs" : "Next:"} `, who(opp.player.name), `${opp.player.bot ? " 🤖" : ""}${ownCrown ? " (your own team)" : ""}`] : [crown ? "Crown vs today's champion" : "Next: a team saved at this round"])),
      pin,
    ),
    h(
      "div",
      { class: "row spread line-head" },
      h("div", { class: "label" }, crown ? "Your line · front first · final" : "Your line · front first"),
      h("div", { class: "row" }, legendBtn, button("Rules", () => closable(rulesSheet()), "small", "shop-rules")),
    ),
    line,
    actions,
    hintSlot,
    crown ? null : h("div", { class: "label" }, "Shop · tap to read and buy"),
    crown ? null : offers,
    h("div", { class: "spacer" }),
    h("div", { class: "row" }, crown ? null : reroll, fight),
    err,
  );
}

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
  battleScreen({ battle, content, you: "A", fight, run, onDone: () => resultScreen(run, fight, battle, content) });
}

function resultScreen(run: RunView, fight: FightResult, battle: BattleRecord, content: MvpContent): void {
  const end = battle.log.at(-1);
  const turns = end && end.type === "BattleEnd" ? end.turns : 0;
  const word = fight.outcome === "win" ? "VICTORY" : fight.outcome === "loss" ? "DEFEAT" : "DRAW";
  const label = fight.kind === "crown" ? "CROWN" : roundLabel(fight.round);
  const own = fight.kind === "crown" && fight.opponent.player.id === run.player.id;
  const lostHearts = fight.heartsLost > 0 ? ` −${plural(fight.heartsLost, "heart")}.` : "";
  const sub =
    fight.kind === "crown"
      ? own
        ? fight.outcome === "win"
          ? "That was your own champion team: beating it doesn't count as a slay."
          : `Your own champion team holds.${lostHearts}`
        : fight.outcome === "win"
          ? "You beat the champion. You are a slayer today."
          : `The champion holds.${lostHearts}`
      : fight.heartsLost > 0
        ? `−${plural(fight.heartsLost, "heart")}`
        : fight.outcome === "draw"
          ? "A draw costs no heart."
          : "";
  // After a loss, "why I lost" comes first, under a compact header, so its
  // rows show above the sticky buttons; the two lines follow.
  const why = fight.outcome === "loss" ? whyILost(battle, content, "A") : null;
  show(
    h("div", { class: "hud" }, h("span", { "data-testid": "result-round" }, label), hearts(run.hearts), h("span", { class: "dim" }, record(run))),
    h("div", { class: `outcome ${fight.outcome}${why ? " compact" : ""}`, "data-testid": "outcome" }, word),
    h("div", { class: "dim", style: "text-align:center" }, "vs ", who(fight.opponent.player.name), ` · ${plural(turns, "turn")}`),
    sub ? h("div", { class: fight.heartsLost > 0 ? "error center" : "center", "data-testid": "result-sub" }, sub) : null,
    why,
    h("div", { class: "label" }, "You"),
    team(battle.teamA, "you", content),
    h("div", { class: "label" }, who(battle.opponent.name)),
    team(battle.teamB, "ghost", content),
    h("div", { class: "spacer" }),
    h(
      "div",
      { class: "row footer", "data-testid": "result-actions" },
      button("Replay", () => battleScreen({ battle, content, you: "A", fight, run, onDone: () => resultScreen(run, fight, battle, content) }), "", "replay"),
      button(run.phase === "over" ? "See the run" : run.phase === "crown" ? "To the Crown" : "Next round", () => shopScreen(run, content), "primary grow", "continue"),
    ),
  );
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
        ? { why: "You beat your own champion team. It doesn't count as a slay: the others try to beat it today.", reach: "Won the Crown." }
        : { why: "👑 You beat the champion: you are a slayer today.", reach: "Won the Crown." };
    case "crown-lost":
      return { why: own ? "Your own champion team held the Crown." : "The champion held the Crown.", reach: "Reached the Crown." };
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

function runOverScreen(run: RunView, content: MvpContent, notice = ""): void {
  const { why, reach } = runEnd(run);
  const draws = run.fights.filter((f) => f.outcome === "draw").length;
  const rc = run.rating;
  const delta = rc ? rc.after - rc.before : 0;
  show(
    h("h1", {}, "RUN OVER"),
    notice ? h("div", { class: "error", "data-testid": "error" }, notice) : null,
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
    run.line.length ? h("div", { class: "label" }, "Your last line") : null,
    run.line.length ? team(run.line, "you", content) : null,
    h("div", { class: "spacer" }),
    h("div", { class: "row footer" }, button("Home", () => void homeScreen(), "primary grow", "home")),
  );
}

// ---------- boot ----------

if (api.player) {
  const err = errorLine();
  void guarded(err, homeScreen).then(() => {
    if (err.textContent) show(h("h1", {}, "ARENA"), err, button("Retry", () => location.reload(), "primary"));
  });
} else nameScreen();
