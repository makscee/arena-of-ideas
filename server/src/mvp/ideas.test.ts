import { describe, expect, it } from "vitest";
import { IDEA_TEXT_MAX, MVP_RULES, type MvpRules, type PlayerRef, type Rating } from "../../../src/mvp/contract.js";
import { mvpContent } from "./content.js";
import { cancelIdea, grantIdea, IdeaRefused, ideasOf, myIdeas, spendIdea, writeIdea } from "./ideas.js";
import { abandon, startRun } from "./runs.js";
import { mvpRuntime } from "./runtime.js";
import { MemoryMvpStore } from "./store.js";

const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };

/** A store whose rating row says `runs` finished runs. */
function deps(rules: MvpRules = MVP_RULES) {
  const store = new MemoryMvpStore();
  const setRuns = (runs: number) => store.putRating({ player: maks, rating: 1000, runs, slays: 0, daysAsChampion: 0, playoffWins: 0 } satisfies Rating);
  return { d: { store, rules, now: () => new Date("2026-10-08T08:00:00.000Z") }, setRuns };
}

describe("earning ideas (M2-3)", () => {
  it("earns 1 idea per 3 finished runs, given-up runs included", () => {
    let n = 7;
    const rt = mvpRuntime({ content: mvpContent(), seed: () => (n = (n * 1103515245 + 12345) >>> 0) });
    expect(ideasOf(rt, maks.id)).toEqual({ held: 0, nextIn: 3 });
    const finishOne = () => abandon(rt, startRun(rt, maks));
    finishOne();
    finishOne();
    expect(ideasOf(rt, maks.id)).toEqual({ held: 0, nextIn: 1 });
    finishOne();
    expect(ideasOf(rt, maks.id)).toEqual({ held: 1, nextIn: 3 });
  });

  it("holds at most 3: runs finished at the cap earn nothing, even once one is spent", () => {
    const { d, setRuns } = deps();
    setRuns(9);
    expect(ideasOf(d, maks.id)).toEqual({ held: 3, nextIn: null });
    setRuns(12 + 1); // 3 more runs at the cap, then 1
    expect(ideasOf(d, maks.id)).toEqual({ held: 3, nextIn: null });
    expect(spendIdea(d, maks.id)).toBe(true);
    expect(ideasOf(d, maks.id)).toEqual({ held: 2, nextIn: 2 });
    setRuns(15);
    expect(ideasOf(d, maks.id)).toEqual({ held: 3, nextIn: null });
  });

  it("forfeits the same whether Home was read at every run or only at the end", () => {
    const often = deps();
    for (let runs = 1; runs <= 20; runs++) {
      often.setRuns(runs);
      ideasOf(often.d, maks.id);
      if (runs === 11) spendIdea(often.d, maks.id);
    }
    const once = deps();
    once.setRuns(11);
    spendIdea(once.d, maks.id);
    once.setRuns(20);
    expect(ideasOf(once.d, maks.id)).toEqual(ideasOf(often.d, maks.id));
    expect(ideasOf(once.d, maks.id)).toEqual({ held: 3, nextIn: null });
  });

  it("spends only an idea held", () => {
    const { d, setRuns } = deps();
    expect(spendIdea(d, maks.id)).toBe(false);
    setRuns(3);
    expect(spendIdea(d, maks.id)).toBe(true);
    expect(spendIdea(d, maks.id)).toBe(false);
    expect(ideasOf(d, maks.id)).toEqual({ held: 0, nextIn: 3 });
  });

  it("dev +1 idea adds one up to the cap", () => {
    const { d, setRuns } = deps();
    setRuns(2);
    expect(grantIdea(d, maks.id)).toEqual({ held: 1, nextIn: 1 });
    expect(grantIdea(d, maks.id)).toEqual({ held: 2, nextIn: 1 });
    expect(grantIdea(d, maks.id)).toEqual({ held: 3, nextIn: null });
    expect(grantIdea(d, maks.id)).toEqual({ held: 3, nextIn: null });
    expect(d.store.ideaCounts(maks.id).granted).toBe(3);
  });

  it("takes the rate and cap from the rules", () => {
    const { d, setRuns } = deps({ ...MVP_RULES, ideaEveryRuns: 2, ideaHold: 5 });
    setRuns(9);
    expect(ideasOf(d, maks.id)).toEqual({ held: 4, nextIn: 1 });
    setRuns(10);
    expect(ideasOf(d, maks.id)).toEqual({ held: 5, nextIn: null });
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
    const { d } = deps();
    grantIdea(d, maks.id);
    grantIdea(d, maks.id);
    const idea = writeIdea(d, maks.id, "  A healer who grows stronger every time an ally falls.  ");
    expect(idea).toMatchObject({ playerId: maks.id, text: "A healer who grows stronger every time an ally falls.", state: "written", createdAt: "2026-10-08T08:00:00.000Z", data: {} });
    expect(myIdeas(d, maks.id)).toEqual({ ideas: { held: 1, nextIn: 3 }, sent: [{ ideaId: idea.ideaId, text: idea.text, state: "written", createdAt: idea.createdAt, data: {} }] });
  });

  it("takes 10–400 characters after trimming, counted as characters", () => {
    const { d } = deps();
    grantIdea(d, maks.id);
    refused(() => writeIdea(d, maks.id, "   too short  "), 400);
    refused(() => writeIdea(d, maks.id, "x".repeat(IDEA_TEXT_MAX + 1)), 400);
    expect(myIdeas(d, maks.id).ideas.held).toBe(1); // a refused text spends nothing
    expect(writeIdea(d, maks.id, "🔥".repeat(IDEA_TEXT_MAX)).text).toHaveLength(IDEA_TEXT_MAX * 2);
  });

  it("refuses with no idea held", () => {
    const { d } = deps();
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
    const { d } = deps();
    grantIdea(d, maks.id);
    grantIdea(d, "p2");
    const mine = writeIdea(d, maks.id, "A knight who guards the weakest ally.");
    const theirs = writeIdea(d, "p2", "A rat that spreads plague on death.");
    refused(() => cancelIdea(d, maks.id, theirs.ideaId), 404);
    refused(() => cancelIdea(d, maks.id, "no-such-id"), 404);
    expect(myIdeas(d, maks.id).ideas.held).toBe(0);
    cancelIdea(d, maks.id, mine.ideaId);
    expect(myIdeas(d, maks.id)).toEqual({ ideas: { held: 1, nextIn: 3 }, sent: [] });
    refused(() => cancelIdea(d, maks.id, mine.ideaId), 404);
    d.store.putIdea({ ...theirs, state: "reading" });
    refused(() => cancelIdea(d, "p2", theirs.ideaId), 409);
    expect(d.store.idea(theirs.ideaId)).toBeDefined();
  });

  it("won't cancel while holding the cap: the refund would be forfeited", () => {
    const { d, setRuns } = deps();
    setRuns(9);
    const idea = writeIdea(d, maks.id, "A knight who guards the weakest ally.");
    setRuns(12);
    expect(myIdeas(d, maks.id).ideas).toEqual({ held: 3, nextIn: null });
    refused(() => cancelIdea(d, maks.id, idea.ideaId), 409);
    writeIdea(d, maks.id, "Another idea to make room.");
    cancelIdea(d, maks.id, idea.ideaId);
    expect(myIdeas(d, maks.id).ideas).toEqual({ held: 3, nextIn: null });
  });
});
