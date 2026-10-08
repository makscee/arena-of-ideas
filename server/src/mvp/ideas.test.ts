import { describe, expect, it } from "vitest";
import { MVP_RULES, type MvpRules, type PlayerRef, type Rating } from "../../../src/mvp/contract.js";
import { mvpContent } from "./content.js";
import { grantIdea, ideasOf, spendIdea } from "./ideas.js";
import { abandon, startRun } from "./runs.js";
import { mvpRuntime } from "./runtime.js";
import { MemoryMvpStore } from "./store.js";

const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };

/** A store whose rating row says `runs` finished runs. */
function deps(rules: MvpRules = MVP_RULES) {
  const store = new MemoryMvpStore();
  const setRuns = (runs: number) => store.putRating({ player: maks, rating: 1000, runs, slays: 0, daysAsChampion: 0, playoffWins: 0 } satisfies Rating);
  return { d: { store, rules }, setRuns };
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
