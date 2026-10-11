import { describe, expect, it } from "vitest";
import { IDEA_TEXT_MAX, MVP_RULES, type DayState, type MvpRules, type PlayerRef, type Rating } from "../../../src/mvp/contract.js";
import { mvpContent } from "./content.js";
import { cancelIdea, grantIdea, IdeaRefused, ideasOf, myIdeas, spendIdea, writeIdea } from "./ideas.js";
import { abandon, startRun } from "./runs.js";
import { mvpRuntime } from "./runtime.js";
import { MemoryMvpStore } from "./store.js";

const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };
const NOW = "2026-10-08T08:00:00.000Z";

/** A store on day `seq` (the test moves it), and a player's ideas as Home
 * sees them: held, refilled up to 3 at the day's end. */
function deps(rules: MvpRules = MVP_RULES) {
  const store = new MemoryMvpStore();
  let seq = 1;
  const endsAt = () => new Date(Date.parse("2026-10-09T01:00:00.000Z") + (seq - 1) * 86_400_000).toISOString();
  const today = (): DayState => ({ seq, day: `day ${seq}`, startedAt: NOW, endsAt: endsAt() });
  const d = { store, rules, now: () => new Date(NOW), today };
  const seen = (held: number) => ({ held, perDay: rules.ideaHold ?? 3, refillsAt: endsAt(), ready: 0 });
  return { d, nextDay: () => seq++, seen };
}

describe("ideas a day (#915)", () => {
  it("a fresh player has 3; writing 1 leaves 2; the next day brings 3 back", () => {
    const { d, nextDay, seen } = deps();
    expect(ideasOf(d, maks.id)).toEqual(seen(3));
    writeIdea(d, maks.id, "A knight who guards the weakest ally.");
    expect(ideasOf(d, maks.id)).toEqual(seen(2));
    nextDay();
    expect(ideasOf(d, maks.id)).toEqual(seen(3));
  });

  it("holding 3 at the day change stays 3: unspent ones don't stack", () => {
    const { d, nextDay, seen } = deps();
    expect(ideasOf(d, maks.id).held).toBe(3);
    nextDay();
    nextDay();
    expect(ideasOf(d, maks.id)).toEqual(seen(3));
  });

  it("writing 3 in a day leaves 0, and a 4th write is refused until the next day", () => {
    const { d, nextDay, seen } = deps();
    for (let i = 1; i <= 3; i++) writeIdea(d, maks.id, `Idea number ${i} of the day.`);
    expect(ideasOf(d, maks.id)).toEqual(seen(0));
    let refusal: unknown;
    try {
      writeIdea(d, maks.id, "A fourth idea on the same day.");
    } catch (err) {
      refusal = err;
    }
    expect(refusal).toBeInstanceOf(IdeaRefused);
    expect((refusal as IdeaRefused).status).toBe(409);
    expect((refusal as IdeaRefused).message).toBe("You have no ideas left today. They come back at the day's end.");
    expect(myIdeas(d, maks.id).sent).toHaveLength(3);
    nextDay();
    expect(ideasOf(d, maks.id)).toEqual(seen(3));
  });

  it("runs earn no ideas", () => {
    let n = 7;
    const rt = mvpRuntime({ content: mvpContent(), seed: () => (n = (n * 1103515245 + 12345) >>> 0) });
    spendIdea(rt, maks.id);
    expect(ideasOf(rt, maks.id).held).toBe(2);
    for (let i = 0; i < 4; i++) abandon(rt, startRun(rt, maks));
    expect(ideasOf(rt, maks.id).held).toBe(2);
  });

  it("refills on the arena day's boundary: the champion's day end", () => {
    let n = 7;
    let now = new Date("2026-10-08T08:00:00.000Z");
    const rt = mvpRuntime({ content: mvpContent(), seed: () => (n = (n * 1103515245 + 12345) >>> 0), now: () => now });
    spendIdea(rt, maks.id);
    const { refillsAt } = ideasOf(rt, maks.id);
    expect(refillsAt).toBe(rt.today().endsAt);
    now = new Date(Date.parse(refillsAt) - 1000);
    expect(ideasOf(rt, maks.id).held).toBe(2);
    now = new Date(Date.parse(refillsAt) + 1000);
    expect(ideasOf(rt, maks.id).held).toBe(3);
    expect(rt.today().seq).toBe(2);
  });

  it("a player from before this rule keeps what they held, and is topped up to 3", () => {
    const { d, seen } = deps();
    d.store.putRating({ player: maks, rating: 1000, runs: 7, slays: 0, daysAsChampion: 0, playoffWins: 0 } satisfies Rating);
    d.store.putIdeaCounts(maks.id, { spent: 1, granted: 1, forfeited: 0 }); // held 2 under 1-per-3-runs
    expect(ideasOf(d, maks.id)).toEqual(seen(3));
    expect(d.store.ideaCounts(maks.id)).toMatchObject({ spent: 1, granted: 1, held: 3, day: 1 });
  });

  it("dev +1 idea adds one up to 3", () => {
    const { d, seen } = deps();
    spendIdea(d, maks.id);
    spendIdea(d, maks.id);
    expect(grantIdea(d, maks.id)).toEqual(seen(2));
    expect(grantIdea(d, maks.id)).toEqual(seen(3));
    expect(grantIdea(d, maks.id)).toEqual(seen(3));
    expect(d.store.ideaCounts(maks.id).granted).toBe(2);
  });

  it("takes the ideas a day from the rules", () => {
    const { d, nextDay, seen } = deps({ ...MVP_RULES, ideaHold: 5 });
    expect(ideasOf(d, maks.id)).toEqual(seen(5));
    spendIdea(d, maks.id);
    nextDay();
    expect(ideasOf(d, maks.id)).toEqual(seen(5));
  });

  it("spends only an idea held", () => {
    const { d } = deps();
    for (let i = 0; i < 3; i++) expect(spendIdea(d, maks.id)).toBe(true);
    expect(spendIdea(d, maks.id)).toBe(false);
    expect(ideasOf(d, maks.id).held).toBe(0);
  });
});

describe("writing ideas (M2-4)", () => {
  const refused = (fn: () => unknown, status: number) => {
    try {
      fn();
    } catch (err) {
      expect(err).toBeInstanceOf(IdeaRefused);
      expect((err as IdeaRefused).status).toBe(status);
      return;
    }
    throw new Error("not refused");
  };

  it("writes one, spending a held idea; it shows under Sent as written", () => {
    const { d, seen } = deps();
    const idea = writeIdea(d, maks.id, "  A healer who grows stronger every time an ally falls.  ");
    expect(idea).toMatchObject({ playerId: maks.id, text: "A healer who grows stronger every time an ally falls.", state: "written", createdAt: "2026-10-08T08:00:00.000Z", data: {} });
    expect(myIdeas(d, maks.id)).toEqual({ ideas: seen(2), sent: [{ ideaId: idea.ideaId, text: idea.text, state: "written", createdAt: idea.createdAt, data: {} }] });
  });

  it("takes 10–400 characters after trimming, counted as characters", () => {
    const { d } = deps();
    refused(() => writeIdea(d, maks.id, "   too short  "), 400);
    refused(() => writeIdea(d, maks.id, "x".repeat(IDEA_TEXT_MAX + 1)), 400);
    expect(myIdeas(d, maks.id).ideas.held).toBe(3); // a refused text spends nothing
    expect(writeIdea(d, maks.id, "🔥".repeat(IDEA_TEXT_MAX)).text).toHaveLength(IDEA_TEXT_MAX * 2);
  });

  it("refuses with no idea held", () => {
    const { d } = deps();
    for (let i = 0; i < 3; i++) spendIdea(d, maks.id);
    refused(() => writeIdea(d, maks.id, "A knight who guards the weakest ally."), 409);
    expect(myIdeas(d, maks.id).sent).toEqual([]);
  });

  it("lists only the player's own ideas, newest first", () => {
    const { d } = deps();
    grantIdea(d, maks.id);
    grantIdea(d, maks.id);
    grantIdea(d, "p2");
    const a = writeIdea(d, maks.id, "First idea of the day.");
    const b = writeIdea(d, maks.id, "Second idea of the day.");
    writeIdea(d, "p2", "Someone else's secret idea.");
    expect(myIdeas(d, maks.id).sent.map((i) => i.ideaId)).toEqual([b.ideaId, a.ideaId]);
    expect(JSON.stringify(myIdeas(d, maks.id))).not.toContain("secret");
  });

  it("cancels a written idea and refunds it; never someone else's or one being read", () => {
    const { d, seen } = deps();
    const mine = writeIdea(d, maks.id, "A knight who guards the weakest ally.");
    const theirs = writeIdea(d, "p2", "A rat that spreads plague on death.");
    refused(() => cancelIdea(d, maks.id, theirs.ideaId), 404);
    refused(() => cancelIdea(d, maks.id, "no-such-id"), 404);
    expect(myIdeas(d, maks.id).ideas.held).toBe(2);
    cancelIdea(d, maks.id, mine.ideaId);
    expect(myIdeas(d, maks.id)).toEqual({ ideas: seen(3), sent: [] });
    refused(() => cancelIdea(d, maks.id, mine.ideaId), 404);
    d.store.putIdea({ ...theirs, state: "reading" });
    refused(() => cancelIdea(d, "p2", theirs.ideaId), 409);
    expect(d.store.idea(theirs.ideaId)).toBeDefined();
  });

  it("won't cancel while holding the cap: the refund would be forfeited", () => {
    const { d, nextDay, seen } = deps();
    const idea = writeIdea(d, maks.id, "A knight who guards the weakest ally.");
    nextDay();
    expect(myIdeas(d, maks.id).ideas).toEqual(seen(3));
    refused(() => cancelIdea(d, maks.id, idea.ideaId), 409);
    writeIdea(d, maks.id, "Another idea to make room.");
    cancelIdea(d, maks.id, idea.ideaId);
    expect(myIdeas(d, maks.id).ideas).toEqual(seen(3));
  });
});
