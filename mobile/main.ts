// Arena MVP phone client: name → home → shop → fight → battle → result.
// Plain DOM; each screen is a function that renders into #app.
// Owners: slice 8 the name, home, shop and result screens here; slice 9 the
// battle viewer (screens/battle.ts); slice 11 the stats page
// (screens/stats.ts). Shared: api.ts, content.ts, ui/.
// Home also shows DayView.lastPlayoff (slice 5 fills it; a game opens in
// battleScreen) and, under "Dev", an "End day now" button (api.endDay(): 404
// without MVP_DEV=1, 501 until slice 5).
import type { BattleRecord, DayView, FightResult, HomeView, LineUnit, MvpContent, MvpRules, Offer, PlayoffResult, RunView } from "../src/mvp/contract";
import { MVP_RULES } from "../src/mvp/contract";
import { ApiError, api } from "./api";
import { getContent } from "./content";
import { battleScreen, whyILost } from "./screens/battle";
import { statsScreen } from "./screens/stats";
import { card, unitSheet, type CardUnit } from "./ui/card";
import { app, button, h, overlay, show } from "./ui/dom";

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
const roundLabel = (round: number) => (round > rules.rounds ? "CROWN" : `R${round}/${rules.rounds}`);
const hearts = (n: number) => h("span", { class: "hearts", "aria-label": plural(n, "heart") }, "♥".repeat(n) + "♡".repeat(Math.max(0, rules.hearts - n)));
const openSheet = (u: Parameters<typeof unitSheet>[0], content: MvpContent) => () => overlay(unitSheet(u, content));

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
    p(`${r.goldPerRound} gold every round, no carry-over. A unit costs ${r.unitCost}, a reroll ${r.rerollCost}, selling gives back ${r.sellRefund}. ${r.offers} offers a shop; stronger tiers open as rounds pass.`),
    h("div", { class: "label" }, "The line"),
    p(`${r.lineSize} units in a line, front first. Change the order in the shop: tap a unit, then ◀ ▶. Each round you fight a team another player saved at the same round.`),
    h("div", { class: "label" }, "Copies, Awoken, fusion"),
    p(`Buying a unit you own merges it in: +${r.copyGrowth.pwr} PWR / +${r.copyGrowth.hp} HP a copy. Copy ${r.copiesToAwaken} awakens it: the same When, a stronger Who or Does.`),
    p("Two Awoken units fuse: the When of the first you tap, the Who of the second, the Does of both, stats summed. A fused unit is final; copies of either part still merge into it. The first player to make a pair names it."),
    h("div", { class: "label" }, "Chains"),
    p("Units react to events. When one happens, the units it triggers fire in line order, front to back, each at most once per event. In a fight, tap any number to see the chain that caused it."),
    h("div", { class: "label" }, "The day"),
    p(`Beat the champion in the Crown and you are a slayer. At ${r.dayEndsAt} Moscow the slayers' best teams play a round-robin, and the winner is the next champion. Your rating moves once per run.`),
  );
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

async function homeScreen(): Promise<void> {
  const err = errorLine();
  const [home, content]: [HomeView, MvpContent] = await Promise.all([api.home(), getContent()]);
  rules = home.rules;
  day = home.day;
  const champ = home.day.champion;
  const r = home.rating;
  const play = home.activeRunId
    ? button("Continue", () => void guarded(err, async () => shopScreen(await api.run(home.activeRunId!), content)), "primary grow", "play")
    : button("Play", () => void guarded(err, async () => shopScreen(await api.startRun(), content)), "primary grow", "play");
  const stats = button("Stats", () => statsScreen({ content, onBack: () => void homeScreen() }), "", "stats");
  const rulesBtn = button("Rules", () => overlay(rulesSheet()), "", "rules-open");
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
        await homeScreen();
      }),
    "small",
    "end-day",
  );
  show(
    h("div", { class: "row spread" }, h("h1", {}, "ARENA"), h("span", { class: "dim" }, `@${api.player?.name ?? ""}`)),
    h(
      "div",
      { class: "panel stack champion", "data-testid": "champion" },
      h("div", { class: "row spread" }, h("div", { class: "label" }, `👑 Champion · day ${home.day.seq}`), champ ? h("span", { class: "ghost-name" }, `@${champ.player.name}`) : null),
      champ ? team(champ.line, "ghost", content) : h("div", { class: "dim" }, "No champion yet. The day arrives soon."),
      h("div", { class: "dim small", "data-testid": "slayers" }, `${plural(home.day.slayers, "slayer")} today · new champion at ${rules.dayEndsAt} Moscow`),
    ),
    champ ? hint(r ? "Tap a card to read it. Beat this team in the Crown to become a slayer." : "This is the team to beat. Tap a card to read it, then Play.") : null,
    playoffPanel(home.day.lastPlayoff ?? null, content, err),
    h(
      "div",
      { class: "panel records", "data-testid": "records" },
      record("Rating", r?.rating ?? rules.ratingStart, "rating"),
      record("Runs", r?.runs ?? 0),
      record("Slays", r?.slays ?? 0),
      record("Days 👑", r?.daysAsChampion ?? 0),
      record("Playoff W", r?.playoffWins ?? 0),
    ),
    h("div", { class: "spacer" }),
    h("div", { class: "row" }, rulesBtn, stats, play),
    h("details", { class: "dev" }, h("summary", {}, "Dev"), endDay),
    err,
  );
}

/** Yesterday's playoff: the winner, the table and each game (opens in the viewer). */
function playoffPanel(p: PlayoffResult | null, content: MvpContent, err: HTMLElement): HTMLElement | null {
  if (!p) return null;
  const watch = (battleId: string, label: string) =>
    button(label, () => void guarded(err, async () => {
      const battle = await api.battle(battleId);
      battleScreen({ battle, content, onDone: () => void homeScreen() });
    }), "small game");
  return h(
    "div",
    { class: "panel stack", "data-testid": "playoff" },
    h("div", { class: "label" }, `Playoff · day ${p.seq}`),
    h("div", {}, p.winner ? `👑 @${p.winner.name} took the throne.` : p.entrants.length ? "No winner." : "No slayers; the champion stays."),
    ...p.standings.map((s) => h("div", { class: "num small" }, `@${s.player.name} · ${s.wins}W ${s.draws}D ${s.losses}L`)),
    p.games.length ? h("div", { class: "games" }, ...p.games.map((g) => watch(g.battleId, `@${g.a.name} v @${g.b.name}`))) : null,
  );
}

// ---------- shop ----------

/** What the shop is in the middle of: nothing, a unit picked, or a fuse waiting for its second unit. */
type Pick = { mode: "none" } | { mode: "picked"; index: number } | { mode: "fuse"; first: number };

function shopScreen(run: RunView, content: MvpContent): void {
  if (run.phase === "over") return runOverScreen(run, content);
  const crown = run.phase === "crown";
  const err = errorLine();
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
    if (crown) return hint("The Crown: your line against today's champion. Win it to become a slayer.");
    const almost = run.line.find((u) => u.kind === "unit" && u.form === "sleeping" && u.copies === rules.copiesToAwaken - 1);
    if (awoken >= 2) return hint("Two Awoken units can fuse: tap one, then Fuse.");
    if (almost && run.offers.some((o) => o.unitId === almost.unitId)) return hint(`One more ${almost.name} awakens it. It's in the shop.`);
    if (run.line.length === 0) return hint("Tap an offer to read it and buy it. Your line fights front first.");
    if (run.round === 1 && run.line.length > 0 && run.gold < rules.unitCost) return hint("Out of gold for units. Fight when ready.");
    if (run.line.length > 1 && run.round <= 2) return hint("Tap a unit in your line to move, sell or read it.");
    if (almost) return hint(`A 3rd ${almost.name} awakens it.`);
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
      let blocked = affordable ? "" : `Needs ${o.cost}g`;
      if (res instanceof ApiError) blocked = res.message;
      else if (res) {
        const before = new Map(run.line.map((x) => [x.uid, x]));
        const changed = res.run.line.find((x) => !before.has(x.uid) || before.get(x.uid)!.copies !== x.copies);
        if (changed) {
          const was = before.get(changed.uid);
          const label = !was ? "Joins your line" : was.form !== changed.form ? "Awakens!" : `Merges in: ×${changed.copies}`;
          after = h("div", { class: `stack after${was && was.form !== changed.form ? " awakens" : ""}`, "data-testid": "buy-preview" }, h("div", { class: "label" }, label), h("div", { class: "preview-card" }, card(changed, { side: "you", extra: [copiesBadge(changed)] })));
        }
      }
      const buy = button(blocked || `Buy ${o.cost}g`, () => (close(), void decide({ kind: "buy", slot: o.slot })), "primary grow", "buy");
      buy.disabled = blocked !== "";
      const close = overlay(
        ...(u ? [unitSheet(u, content)] : [h("h2", {}, o.unitId)]),
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
  fight.disabled = run.line.length === 0;

  const opp = run.nextOpponent;
  const pin = h("div", { class: "pin", "data-testid": "champion-pin" });
  const fillPin = (d: DayView | null) => {
    const ch = d?.champion;
    if (!ch) return pin.replaceChildren(h("span", { class: "dim" }, "👑 No champion yet"));
    const b = h("button", { class: "pin-btn", "data-testid": "champion-pin-open" }, h("span", {}, "👑"), h("span", { class: "ghost-name" }, `@${ch.player.name}`), h("span", { class: "pin-emoji" }, ch.line.map((u) => u.emoji).join("")));
    b.addEventListener("click", () => overlay(h("div", { class: "label" }, `Champion of day ${d!.seq} · @${ch.player.name}`), team(ch.line, "ghost", content), hint("Tap a card to read it.")));
    pin.replaceChildren(b);
  };
  fillPin(day);
  if (!day) void api.day().then((d) => ((day = d), fillPin(d))).catch(() => {});

  renderLine();
  show(
    h(
      "div",
      { class: "hud", "data-testid": "hud" },
      h("span", { "data-testid": "round" }, roundLabel(run.round)),
      hearts(run.hearts),
      h("span", { class: "gold", "data-testid": "gold" }, `${run.gold}g`),
    ),
    h(
      "div",
      { class: "row spread opp" },
      h("span", { class: "dim", "data-testid": "next-opponent" }, opp ? `${crown ? "Crown vs" : "Next:"} @${opp.player.name}${opp.player.bot ? " 🤖" : ""}` : crown ? "Crown vs today's champion" : "Next: a team saved at this round"),
      pin,
    ),
    h("div", { class: "label" }, "Your line · front first"),
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

/** ●●○ toward awakening for a sleeping unit; AWOKEN or FUSED otherwise. */
function copiesBadge(u: LineUnit): HTMLElement {
  if (u.kind === "fused") return h("div", { class: "copies tag" }, `FUSED ×${u.copies}`);
  if (u.form === "awoken") return h("div", { class: "copies tag" }, `AWOKEN ×${u.copies}`);
  const n = rules.copiesToAwaken;
  return h("div", { class: "copies pips", "aria-label": `${u.copies} of ${n} copies` }, "●".repeat(Math.min(u.copies, n)) + "○".repeat(Math.max(0, n - u.copies)));
}

// ---------- battle, then result ----------

async function fightScreens(run: RunView, fight: FightResult, content: MvpContent): Promise<void> {
  const battle = await api.battle(fight.battleId);
  battleScreen({ battle, content, you: "A", fight, run, onDone: () => resultScreen(run, fight, battle, content) });
}

function resultScreen(run: RunView, fight: FightResult, battle: BattleRecord, content: MvpContent): void {
  const end = battle.log.at(-1);
  const turns = end && end.type === "BattleEnd" ? end.turns : 0;
  const word = fight.outcome === "win" ? "VICTORY" : fight.outcome === "loss" ? "DEFEAT" : "DRAW";
  const label = fight.kind === "crown" ? "CROWN" : roundLabel(fight.round);
  const sub =
    fight.kind === "crown"
      ? fight.outcome === "win"
        ? "You beat the champion. You are a slayer today."
        : "The champion holds."
      : fight.heartsLost > 0
        ? `−${plural(fight.heartsLost, "heart")}`
        : fight.outcome === "draw"
          ? "A draw costs no heart."
          : "";
  show(
    h("div", { class: "hud" }, h("span", { "data-testid": "result-round" }, label), hearts(run.hearts), h("span", { class: "dim" }, `${run.wins}W ${run.losses}L`)),
    h("div", { class: `outcome ${fight.outcome}`, "data-testid": "outcome" }, word),
    h("div", { class: "dim", style: "text-align:center" }, `vs @${fight.opponent.player.name} · ${plural(turns, "turn")}`),
    sub ? h("div", { class: fight.heartsLost > 0 ? "error center" : "center" }, sub) : null,
    h("div", { class: "label" }, "You"),
    team(battle.teamA, "you", content),
    h("div", { class: "label" }, `@${battle.opponent.name}`),
    team(battle.teamB, "ghost", content),
    fight.outcome === "loss" ? whyILost(battle, content, "A") : null,
    h("div", { class: "spacer" }),
    h(
      "div",
      { class: "row" },
      button("Replay", () => battleScreen({ battle, content, you: "A", fight, run, onDone: () => resultScreen(run, fight, battle, content) }), "", "replay"),
      button(run.phase === "over" ? "See the run" : run.phase === "crown" ? "To the Crown" : "Next round", () => shopScreen(run, content), "primary grow", "continue"),
    ),
  );
}

function runOverScreen(run: RunView, content: MvpContent): void {
  const why =
    run.endedBy === "out-of-hearts"
      ? "Out of hearts."
      : run.endedBy === "no-champion"
        ? `All ${rules.rounds} rounds survived. No champion to face yet.`
        : run.endedBy === "crown-won"
          ? "👑 You beat the champion: you are a slayer today."
          : run.endedBy === "crown-lost"
            ? "The champion held the Crown."
            : run.endedBy ?? "";
  const draws = run.fights.filter((f) => f.outcome === "draw").length;
  const rc = run.rating;
  const delta = rc ? rc.after - rc.before : 0;
  show(
    h("h1", {}, "RUN OVER"),
    h(
      "div",
      { class: "panel stack", "data-testid": "run-over" },
      h("div", {}, why),
      h("div", { class: "num" }, `${plural(run.wins, "win")} · ${plural(run.losses, "loss", "losses")}${draws ? ` · ${plural(draws, "draw")}` : ""}`),
      h("div", { class: "dim" }, run.round > rules.rounds ? "Reached the Crown" : `Ended in round ${Math.min(run.round, rules.rounds)} of ${rules.rounds}`),
      rc ? h("div", { class: "num", "data-testid": "rating-change" }, `Rating ${rc.before} → ${rc.after} (${delta >= 0 ? "+" : ""}${delta})`) : null,
    ),
    run.line.length ? h("div", { class: "label" }, "Your last line") : null,
    run.line.length ? team(run.line, "you", content) : null,
    h("div", { class: "spacer" }),
    button("Home", () => void homeScreen(), "primary", "home"),
  );
}

// ---------- boot ----------

if (api.player) {
  const err = errorLine();
  void guarded(err, homeScreen).then(() => {
    if (err.textContent) show(h("h1", {}, "ARENA"), err, button("Retry", () => location.reload(), "primary"));
  });
} else nameScreen();
