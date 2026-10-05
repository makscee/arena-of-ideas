// Arena MVP phone client, slice 1: name → home → shop → fight → battle →
// result. Plain DOM; each screen is a function that renders into #app.
// Owners: slice 8 the name, home, shop and result screens here; slice 9 the
// battle viewer (screens/battle.ts); slice 11 the stats page
// (screens/stats.ts). Shared: api.ts, content.ts, ui/.
// Home (slice 8) also shows DayView.lastPlayoff (slice 5 fills it; a game
// opens in battleScreen) and, on a dev server, an "End day now" button
// (api.endDay(): 404 without MVP_DEV=1, 501 until slice 5).
import type { BattleRecord, FightResult, HomeView, LineUnit, MvpContent, Offer, RunView } from "../src/mvp/contract";
import { ApiError, api } from "./api";
import { getContent } from "./content";
import { battleScreen, whyILost } from "./screens/battle";
import { statsScreen } from "./screens/stats";
import { card } from "./ui/card";
import { app, button, h, show } from "./ui/dom";

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
  const champ = home.day.champion;
  const play = home.activeRunId
    ? button("Continue run", () => void guarded(err, async () => shopScreen(await api.run(home.activeRunId!), content)), "primary grow", "play")
    : button("Play", () => void guarded(err, async () => shopScreen(await api.startRun(), content)), "primary grow", "play");
  const stats = button("Stats", () => statsScreen({ content, onBack: () => void homeScreen() }), "", "stats");
  show(
    h("div", { class: "row spread" }, h("h1", {}, "ARENA"), h("span", { class: "dim" }, `@${api.player?.name ?? ""}`)),
    h(
      "div",
      { class: "panel stack", "data-testid": "champion" },
      h("div", { class: "label" }, `Champion of ${home.day.day}`),
      champ
        ? h("div", { class: "slots" }, ...champ.line.map((u) => card(u, { side: "ghost" })))
        : h("div", { class: "dim" }, "No champion yet. The day arrives soon."),
    ),
    h(
      "div",
      { class: "panel row spread" },
      h("div", {}, h("div", { class: "label" }, "Rating"), h("div", { class: "num", "data-testid": "rating" }, `${home.rating?.rating ?? home.rules.ratingStart}`)),
      h("div", {}, h("div", { class: "label" }, "Slays"), h("div", { class: "num" }, `${home.rating?.slays ?? 0}`)),
    ),
    h(
      "div",
      { class: "panel stack dim" },
      h("div", { class: "label" }, "Rules"),
      h("div", {}, `${home.rules.rounds} shop rounds, then the Crown. ${home.rules.hearts} hearts; a lost fight costs one.`),
      h("div", {}, `${home.rules.goldPerRound} gold a round. Unit ${home.rules.unitCost}, reroll ${home.rules.rerollCost}, sell +${home.rules.sellRefund}.`),
      h("div", {}, "Your line fights front first. A copy merges in for +1 PWR / +2 HP."),
    ),
    h("div", { class: "spacer" }),
    h("div", { class: "row" }, stats, play),
    err,
  );
}

// ---------- shop ----------

function shopScreen(run: RunView, content: MvpContent): void {
  if (run.phase === "over") return runOverScreen(run);
  const err = errorLine();
  let selected: number | null = null;
  const decide = (d: Parameters<typeof api.decide>[1]) =>
    guarded(err, async () => {
      const res = await api.decide(run.runId, d);
      if (res.fight) return fightScreens(res.run, res.fight, content);
      shopScreen(res.run, content);
    });

  const line = h("div", { class: "slots", "data-testid": "line" });
  const actions = h("div", { class: "row" });
  const renderLine = () => {
    line.replaceChildren(
      ...Array.from({ length: 5 }, (_, i) => {
        const u: LineUnit | undefined = run.line[i];
        if (!u) return h("div", { class: "card empty" }, h("div", { class: "dim" }, `${i + 1}`));
        const c = card(u, { side: "you", extra: [u.copies > 1 ? h("div", { class: "copies" }, `×${u.copies}`) : null], testid: `line-${i}` });
        if (selected === i) c.classList.add("selected");
        c.addEventListener("click", () => {
          selected = selected === i ? null : i;
          renderLine();
        });
        return c;
      }),
    );
    actions.replaceChildren(
      ...(selected === null
        ? [h("span", { class: "dim" }, "Tap a unit to move or sell it.")]
        : [
            button("◀", () => void decide({ kind: "reorder", from: selected!, to: selected! - 1 }), "", "move-left"),
            button("▶", () => void decide({ kind: "reorder", from: selected!, to: selected! + 1 }), "", "move-right"),
            button(`Sell +${1}`, () => void decide({ kind: "sell", index: selected! }), "danger", "sell"),
          ]),
    );
    const [l, r] = actions.querySelectorAll("button");
    if (selected !== null && l && r) {
      (l as HTMLButtonElement).disabled = selected === 0;
      (r as HTMLButtonElement).disabled = selected >= run.line.length - 1;
    }
  };
  renderLine();

  const offers = h(
    "div",
    { class: "slots", "data-testid": "offers" },
    ...run.offers.map((o: Offer) => {
      const u = content.units.find((x) => x.id === o.unitId);
      const c = card(
        { unitId: o.unitId, emoji: u?.emoji ?? "?", name: u?.name ?? o.unitId, stats: u?.base ?? { pwr: 0, hp: 0 } },
        { side: "you", extra: [h("div", { class: "cost" }, `${o.cost}g`)], testid: `offer-${o.slot}` },
      );
      if (run.gold < o.cost) c.style.opacity = "0.45";
      c.addEventListener("click", () => void decide({ kind: "buy", slot: o.slot }));
      return c;
    }),
  );

  const reroll = button("Reroll 1g", () => void decide({ kind: "reroll" }), "", "reroll");
  reroll.disabled = run.gold < 1;
  const fight = button("Fight", () => void decide({ kind: "fight" }), "primary grow", "fight");
  fight.disabled = run.line.length === 0;

  show(
    h(
      "div",
      { class: "hud", "data-testid": "hud" },
      h("span", {}, `R${run.round}/12`),
      h("span", { class: "hearts" }, "♥".repeat(run.hearts)),
      h("span", { class: "gold", "data-testid": "gold" }, `${run.gold}g`),
    ),
    h("div", { class: "label" }, "Your line · front first"),
    line,
    actions,
    h("div", { class: "label" }, "Shop · tap to buy"),
    offers,
    h("div", { class: "spacer" }),
    h("div", { class: "row" }, reroll, fight),
    err,
  );
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
  show(
    h("div", { class: "hud" }, h("span", {}, `Round ${fight.round}`), h("span", { class: "hearts" }, "♥".repeat(run.hearts))),
    h("div", { class: `outcome ${fight.outcome}`, "data-testid": "outcome" }, word),
    h("div", { class: "dim", style: "text-align:center" }, `vs @${fight.opponent.player.name} · ${turns} turns · ${battle.log.length} events`),
    h("div", { class: "label" }, "You"),
    h("div", { class: "slots" }, ...battle.teamA.map((u) => card(u, { side: "you" }))),
    h("div", { class: "label" }, "Them"),
    h("div", { class: "slots" }, ...battle.teamB.map((u) => card(u, { side: "ghost" }))),
    fight.heartsLost > 0 ? h("div", { class: "error" }, `−${fight.heartsLost} heart`) : h("div", {}),
    fight.outcome === "loss" ? whyILost(battle, content, "A") : null,
    h("div", { class: "spacer" }),
    button(run.phase === "over" ? "See the run" : "Next round", () => shopScreen(run, content), "primary", "continue"),
  );
}

function runOverScreen(run: RunView): void {
  const why = run.endedBy === "out-of-hearts" ? "Out of hearts." : run.endedBy === "no-champion" ? "All 12 rounds survived. No champion to face yet." : run.endedBy ?? "";
  show(
    h("h1", {}, "RUN OVER"),
    h("div", { class: "panel stack", "data-testid": "run-over" }, h("div", {}, why), h("div", { class: "num" }, `${run.wins} wins · ${run.losses} losses · round ${run.round}`)),
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
