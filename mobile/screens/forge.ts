// Codex v2 (rules v2 §6, §7; #1051). Three tabs over one ForgeView
// (src/mvp/forge-view.ts):
// - Units: pick a unit; its page lists its partners in the three states:
//   Discovered (the canonical design and who claimed it), Ready to claim
//   (three sealed designs, claimed at the forge in a run) and Not forged yet
//   (Wish puts the pair on tonight's list; a pair waiting for a word says so).
// - Today: the daily feed, what tonight's batch brought and who claimed what.
// - Dictionary: found words with credit and lineage, and "N words forged,
//   not yet found".
// claimScreen is the claim moment: three sealed cards turn over, the player
// picks one, and it shows with their name on it.
import type { Both, ClaimView, ForgeDesign, ForgePair, ForgeUnit, ForgeView, PairState } from "../../src/mvp/forge-view";
import { currentLang, t } from "../i18n";
import { button, h, screen, show } from "../ui/dom";

export type ForgeTab = "units" | "feed" | "words";

export interface ForgeArgs {
  view: ForgeView;
  tab?: ForgeTab;
  unit?: string | undefined;
  onBack: () => void;
  /** Puts a pair on tonight's list (or takes it off); the screen redraws on its own. */
  onWish?: (pair: ForgePair, on: boolean) => Promise<void> | void;
}

const tr = (s: Both): string => (currentLang() === "ru" ? s.ru : s.en);
const day = (iso: string): string => new Date(iso.length <= 10 ? iso + "T12:00:00Z" : iso).toLocaleDateString(currentLang() === "ru" ? "ru-RU" : "en-GB", { day: "numeric", month: "short" });
const unitOf = (v: { units: ForgeUnit[] }, id: string): ForgeUnit => v.units.find((u) => u.id === id)!;
const label = (u: ForgeUnit): string => `${u.emoji} ${tr(u.name)}`;
const pairLabel = (v: { units: ForgeUnit[] }, [a, b]: [string, string]): string => `${label(unitOf(v, a))} + ${label(unitOf(v, b))}`;

function head(title: string, onBack: () => void): HTMLElement {
  return h("div", { class: "row forge-head" }, button(t("codex.back"), onBack, "primary", "forge-back"), h("h1", {}, title));
}

/** A unit or a design as one wide card: emoji, name, the rule sentence. */
function wide(emoji: string, name: string, rule: string, extra: (Node | null)[] = [], cls = ""): HTMLElement {
  return h("div", { class: `forge-card ${cls}` }, h("div", { class: "forge-card-top" }, h("span", { class: "forge-emoji" }, emoji), h("b", { class: "forge-name" }, name)), h("div", { class: "forge-rule" }, rule), ...extra);
}

export function forgeScreen(a: ForgeArgs): void {
  const v = a.view;
  let tab: ForgeTab = a.tab ?? "units";
  let unit = a.unit ?? v.units[0]!.id;
  const tabs = h("div", { class: "tabs", role: "tablist" });
  const body = h("div", { class: "stack forge-body" });
  const tabBtn = (id: ForgeTab, text: string) => {
    const b = button(text, () => ((tab = id), draw()), tab === id ? "on" : "", `forge-tab-${id}`);
    b.setAttribute("role", "tab");
    return b;
  };
  const draw = (): void => {
    tabs.replaceChildren(tabBtn("units", t("forge.tab.units")), tabBtn("feed", t("forge.tab.feed")), tabBtn("words", t("forge.tab.words")));
    body.replaceChildren(...(tab === "units" ? unitPage() : tab === "feed" ? feed() : dictionary()));
  };

  const unitPage = (): Node[] => {
    const u = unitOf(v, unit);
    const picker = h(
      "div",
      { class: "forge-picker", "data-testid": "forge-picker" },
      ...v.units.map((x) => {
        const b = button(x.emoji, () => ((unit = x.id), draw()), x.id === unit ? "on" : "", "forge-pick");
        b.title = tr(x.name);
        return b;
      }),
    );
    const mine = v.pairs.filter((p) => p.a === unit || p.b === unit);
    const partner = (p: ForgePair) => unitOf(v, p.a === unit ? p.b : p.a);
    const group = (state: PairState, key: "forge.discovered" | "forge.ready" | "forge.unforged", row: (p: ForgePair) => HTMLElement): Node[] => {
      const ps = mine.filter((p) => p.state === state);
      if (!ps.length) return [];
      return [h("div", { class: "label", "data-testid": `forge-${state}` }, t(key, { n: ps.length })), ...ps.map(row)];
    };
    const discovered = (p: ForgePair): HTMLElement => {
      const d = p.design!;
      return wide(d.emoji, tr(d.name), tr(d.rule), [h("div", { class: "dim small forge-with" }, "+ " + label(partner(p))), h("div", { class: "forge-credit small" }, t("forge.discoveredBy", { name: p.claimedBy!, date: day(p.claimedAt!) }))], "found");
    };
    const ready = (p: ForgePair): HTMLElement =>
      h("div", { class: "forge-pair ready", "data-testid": "forge-ready-row" }, h("span", { class: "forge-emoji" }, partner(p).emoji), h("div", { class: "grow" }, h("b", {}, tr(partner(p).name)), h("div", { class: "small" }, "🔒 " + t("forge.sealed"))));
    const unforged = (p: ForgePair): HTMLElement => {
      const n = p.wishes ?? 0;
      const side = p.waitingWord
        ? h("span", { class: "forge-waiting small" }, t("forge.waitingWord", { word: tr(p.waitingWord) }))
        : button(p.wished ? "★ " + t("forge.wished") : "☆ " + t("forge.wish"), () => {
            p.wished = !p.wished;
            p.wishes = n + (p.wished ? 1 : -1);
            draw();
            void a.onWish?.(p, !!p.wished);
          }, p.wished ? "forge-wish on" : "forge-wish", "forge-wish");
      return h(
        "div",
        { class: "forge-pair unforged", "data-testid": "forge-unforged-row" },
        h("span", { class: "forge-emoji" }, partner(p).emoji),
        h("div", { class: "grow" }, h("b", {}, tr(partner(p).name)), h("div", { class: "dim small" }, n ? t("forge.wishes", { n }) : t("forge.notForged"))),
        side,
      );
    };
    return [
      picker,
      wide(u.emoji, tr(u.name), tr(u.rule), [h("div", { class: "dim small" }, `${u.hp} ${currentLang() === "ru" ? "ОЗ" : "HP"} · ${u.pwr} ${currentLang() === "ru" ? "АТК" : "PWR"}`)], "unit"),
      h("h2", { class: "forge-sub" }, t("forge.partners", { name: tr(u.name) })),
      ...group("discovered", "forge.discovered", discovered),
      ...group("ready", "forge.ready", ready),
      ...(mine.some((p) => p.state === "ready") ? [h("div", { class: "dim small" }, t("forge.readyHint"))] : []),
      ...group("unforged", "forge.unforged", unforged),
      ...(mine.some((p) => p.state === "unforged") ? [h("div", { class: "dim small" }, t("forge.wishHint"))] : []),
    ];
  };

  const feed = (): Node[] => {
    const days = [...new Set(v.feed.map((f) => f.at.slice(0, 10)))];
    if (!days.length) return [h("div", { class: "dim" }, t("forge.feedEmpty"))];
    return days.flatMap((d) => [
      h("div", { class: "label" }, day(d)),
      ...v.feed
        .filter((f) => f.at.startsWith(d))
        .map((f) => {
          const line =
            f.kind === "arrived"
              ? ["🌙", t("forge.feedArrived", { n: f.n ?? 0 })]
              : f.kind === "claimed"
                ? ["🔨", t("forge.feedClaimed", { name: f.by!, fusion: tr(f.name!) })]
                : ["📖", t("forge.feedWord", { name: f.by!, word: tr(f.word!) })];
          return h("div", { class: `forge-feed ${f.kind}`, "data-testid": "forge-feed-row" }, h("span", { class: "forge-emoji" }, line[0]!), h("div", { class: "grow" }, h("div", {}, line[1]!), f.pair ? h("div", { class: "dim small" }, pairLabel(v, f.pair)) : null));
        }),
    ]);
  };

  const dictionary = (): Node[] => [
    h("div", { class: "forge-notfound", "data-testid": "forge-notfound" }, "✦ " + t("forge.notFound", { n: v.forgedNotFound })),
    h("div", { class: "dim small" }, t("forge.wordsHint")),
    ...v.words.map((w) =>
      h(
        "div",
        { class: "forge-card word", "data-testid": "forge-word" },
        h("div", { class: "forge-card-top" }, h("b", { class: "forge-word-name" }, tr(w.word)), h("span", { class: `forge-chip ${w.kind}` }, t(w.kind === "composed" ? "forge.composed" : "forge.forged"))),
        h("div", { class: "forge-rule" }, tr(w.means)),
        h("div", { class: "forge-credit small" }, t("forge.foundIn", { fusion: `${tr(w.foundIn.name)} (${pairLabel(v, w.foundIn.pair)})`, name: w.foundBy, date: day(w.foundAt) })),
        w.of ? h("div", { class: "dim small" }, t("forge.madeOf", { words: w.of.join(" · ") })) : null,
        h("div", { class: "dim small" }, t("forge.usedIn", { names: w.usedIn.map(tr).join(", ") })),
      ),
    ),
  ];

  draw();
  show(h("div", { class: "forge stack", "data-testid": "forge" }, head(t("forge.title"), a.onBack), tabs, body));
  screen("forge");
}

export interface ClaimArgs {
  claim: ClaimView;
  units: ForgeUnit[];
  viewer: string;
  /** Opens with this candidate already claimed (a scene of the moment after). */
  picked?: number | undefined;
  onClaim?: (i: number) => Promise<void> | void;
  onDone: () => void;
}

/** The claim moment: three sealed cards turn over, one is picked, it carries your name. */
export function claimScreen(a: ClaimArgs): void {
  const [ua, ub] = a.claim.pair.map((id) => a.units.find((u) => u.id === id)!) as [ForgeUnit, ForgeUnit];
  const pair = `${ua.emoji} ${tr(ua.name)} + ${ub.emoji} ${tr(ub.name)}`;
  let chosen: number | null = null;
  const lean = (d: ForgeDesign): string => (d.lean === "twist" ? t("forge.leanTwist") : t("forge.leanA", { name: tr((d.lean === "a" ? ua : ub).name) }));
  const cardOf = (d: ForgeDesign, i: number, extra: (Node | null)[] = []): HTMLElement =>
    wide(d.emoji, tr(d.name), tr(d.rule), [h("div", { class: "forge-flavour small" }, tr(d.flavour)), h("div", { class: "forge-lean small" }, lean(d)), ...(d.newWords ?? []).map((w) => h("div", { class: "forge-chip forged small", "data-testid": "forge-new-word" }, "✦ " + t("forge.newWord", { word: tr(w) }))), ...extra], `claim lean-${d.lean}`);

  const choosing = (): void => {
    const confirm = button(t("forge.pickOne"), () => {
      if (chosen === null) return;
      void a.onClaim?.(chosen);
      done(chosen);
    }, "primary", "forge-claim");
    confirm.disabled = true;
    const cards = a.claim.candidates.map((d, i) => {
      const el = cardOf(d, i);
      el.dataset.testid = "forge-candidate";
      el.style.setProperty("--i", String(i));
      el.classList.add("sealed");
      el.addEventListener("click", () => {
        chosen = i;
        for (const c of cards) c.classList.toggle("picked", c === el);
        confirm.disabled = false;
        confirm.textContent = t("forge.claimIt", { name: tr(d.name) });
      });
      return el;
    });
    show(h("div", { class: "forge stack claim-moment", "data-testid": "forge-claim-moment" }, head(t("forge.claimTitle"), a.onDone), h("div", { class: "forge-sub" }, pair), h("div", { class: "dim small" }, t("forge.claimLead", { pair })), h("div", { class: "stack forge-candidates" }, ...cards), confirm));
    screen("forge");
  };

  const done = (i: number): void => {
    const d = a.claim.candidates[i]!;
    const today = new Date().toISOString();
    const el = cardOf(d, i, [h("div", { class: "forge-signed", "data-testid": "forge-signed" }, t("forge.claimedBy", { name: a.viewer, date: day(today) }))]);
    el.classList.add("claimed");
    show(
      h(
        "div",
        { class: "forge stack claim-moment", "data-testid": "forge-claimed" },
        head(t("forge.claimTitle"), a.onDone),
        h("div", { class: "forge-sub" }, pair),
        el,
        h("div", { class: "small" }, t("forge.claimedLead", { name: tr(d.name), pair })),
        h("div", { class: "forge-sealed-away" }, ...a.claim.candidates.filter((_, j) => j !== i).map(() => h("div", { class: "forge-card sealed-back" }, "🔒")), h("div", { class: "dim small" }, t("forge.sealedAway"))),
        button(t("codex.back"), a.onDone, "primary", "forge-done"),
      ),
    );
    screen("forge");
  };

  if (a.picked !== undefined) done(a.picked);
  else choosing();
}
