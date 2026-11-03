// The first run's coach (arena#31, mission #26 move 3): in round 1 of a
// player's first run it builds a useful pair out of the shop that is really
// there. A pair is a unit that sets a chain off (it strikes, or is hit) and a
// unit that answers it from behind (the face's "Ally hit" / "Ally strike").
// Units join the line in the order they are bought, so "Medic behind Gambler"
// is: buy Gambler, then buy Medic. Pure: no DOM, no network.

/** The when words of a unit that answers an ally. */
const ANSWERS = new Set(["Ally hit", "Ally strike"]);

/** The anchors the coach likes, best first: strikers whose strike the answerer follows. */
const ANCHORS = ["Gambler", "Hornet", "Frostling", "Leech", "Daredevil", "Duelist", "Ox", "Sniper"];
/** The answerers it likes, best first. */
const ANSWERERS = ["Medic", "Drummer", "Bodyguard", "Battery"];

export interface CoachUnit {
  id: string;
  name: string;
  /** The unit's when word ("Strikes", "Ally hit"), from its face. */
  when: string;
}

export interface CoachOffer {
  slot: number;
  unitId: string;
  cost: number;
}

export interface CoachState {
  /** Unit ids on the line, front first. */
  line: string[];
  offers: CoachOffer[];
  gold: number;
  /** Free line slots. */
  room: number;
  unit: (id: string) => CoachUnit | undefined;
}

/** What the coach says to do next. `anchor`/`answer` name the pair. */
export type CoachStep =
  | { step: "buy-first"; offer: number; anchor: string; answer: string }
  | { step: "buy-second"; offer: number; anchor: string; answer: string }
  | { step: "fight"; anchor: string; answer: string };

const answers = (u: CoachUnit | undefined): boolean => !!u && ANSWERS.has(u.when);
const rank = (list: string[], name: string): number => {
  const i = list.indexOf(name);
  return i < 0 ? list.length : i;
};

/** The pair the coach would build from this shop and line, or null when the
 * shop holds none (then the player just plays). */
export function coachStep(s: CoachState): CoachStep | null {
  // A pair already on the line: anchor then answerer right behind it.
  for (let i = 0; i + 1 < s.line.length; i++) {
    const a = s.unit(s.line[i]!);
    const r = s.unit(s.line[i + 1]!);
    if (a && r && !answers(a) && answers(r)) return { step: "fight", anchor: a.name, answer: r.name };
  }
  const afford = s.offers.filter((o) => o.cost <= s.gold);
  const pairCost = (a: CoachOffer, r: CoachOffer) => a.cost + r.cost;
  const offerUnit = (o: CoachOffer) => s.unit(o.unitId);

  // The line's last unit is the anchor when it is not an answerer: buy its answerer.
  const last = s.line.length ? s.unit(s.line[s.line.length - 1]!) : undefined;
  if (last && !answers(last) && s.room > 0) {
    const pick = afford
      .filter((o) => answers(offerUnit(o)))
      .sort((x, y) => rank(ANSWERERS, offerUnit(x)!.name) - rank(ANSWERERS, offerUnit(y)!.name))[0];
    if (pick) return { step: "buy-second", offer: pick.slot, anchor: last.name, answer: offerUnit(pick)!.name };
  }

  // An empty line (or a last unit that cannot anchor): buy an anchor that has an answerer in the shop.
  if (s.line.length === 0 && s.room >= 2) {
    const anchors = afford.filter((o) => {
      const u = offerUnit(o);
      return u && !answers(u) && u.when !== "";
    });
    const answerers = afford.filter((o) => answers(offerUnit(o)));
    let best: { a: CoachOffer; r: CoachOffer } | null = null;
    for (const a of anchors)
      for (const r of answerers) {
        if (a.slot === r.slot || pairCost(a, r) > s.gold) continue;
        const score = rank(ANCHORS, offerUnit(a)!.name) * 10 + rank(ANSWERERS, offerUnit(r)!.name);
        const cur = best && rank(ANCHORS, offerUnit(best.a)!.name) * 10 + rank(ANSWERERS, offerUnit(best.r)!.name);
        if (!best || score < cur!) best = { a, r };
      }
    if (best) return { step: "buy-first", offer: best.a.slot, anchor: offerUnit(best.a)!.name, answer: offerUnit(best.r)!.name };
  }
  return null;
}

/** Should the coach show: round 1 of a player's first run, nothing fought yet. */
export function coaching(o: { round: number; fights: number; firstRun: boolean }): boolean {
  return o.firstRun && o.round === 1 && o.fights === 0;
}
