import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { ALL_WORDS } from "../../../src/words/index.js";
import { MemoryMvpStore } from "./store.js";
import { wordCard } from "./words-cli.js";
import { dictionary, forgedNotFound, forgedWords, forgedWordsIn, foundWordsIn, pairKey, requestForPair, requestForUnit, requestWord, sealedLine, waitingOn, wordKey } from "./words.js";

const at = new Date("2026-10-14T01:00:00.000Z");
const steal = { word: "steal", kind: "steal", card: 2001 };

describe("the word queue (#1052)", () => {
  it("takes a request from each of the three sources into one queue", () => {
    const s = new MemoryMvpStore();
    expect(requestWord(s, "idea", "i1", { word: "steal", part: "steals the enemy's gold" }, at)).toBe(true);
    expect(requestForPair(s, "Shadow", "Magpie", [{ word: "Steal", part: "takes a power from the front enemy" }], at)).toBe(1);
    expect(requestForUnit(s, "u7", [{ word: "steal", part: "an Awoken that steals" }, { word: "hush", part: "silences for a turn" }], at)).toBe(2);
    expect(s.wordRequests().map((r) => [r.source, r.ref, r.word])).toEqual([
      ["idea", "i1", "steal"],
      ["night", "Magpie+Shadow", "Steal"],
      ["batch", "u7", "steal"],
      ["batch", "u7", "hush"],
    ]);
  });

  it("counts a source asking twice for the same thing once, and ignores blanks", () => {
    const s = new MemoryMvpStore();
    expect(requestWord(s, "night", "a+b", { word: "steal", part: "x" }, at)).toBe(true);
    expect(requestWord(s, "night", "a+b", { word: "  STEAL ", part: "y" }, at)).toBe(false);
    expect(requestWord(s, "night", "a+b", { word: "  ", part: "y" }, at)).toBe(false);
    expect(requestWord(s, "batch", "a+b", { word: "steal", part: "y" }, at)).toBe(true);
    expect(s.wordRequests()).toHaveLength(2);
  });

  it("order is dropped: A+B and B+A are one pair", () => {
    expect(pairKey("Shadow", "Magpie")).toBe(pairKey("Magpie", "Shadow"));
  });

  it("seals a pair while its word is not forged, and un-seals once it is listed", () => {
    const s = new MemoryMvpStore();
    requestForPair(s, "Shadow", "Magpie", [{ word: "steal", part: "x" }], at);
    expect(waitingOn(s, pairKey("Magpie", "Shadow"), dictionary(s, []))).toEqual(["steal"]);
    expect(sealedLine(["steal"])).toBe("waiting for a new word: steal");
    expect(sealedLine(["steal"], "ru")).toMatch(/нового слова: steal/);
    expect(waitingOn(s, pairKey("Magpie", "Shadow"), dictionary(s, [steal]))).toEqual([]);
    expect(waitingOn(s, "Cat+Dog", dictionary(s, []))).toEqual([]);
  });
});

describe("the Dictionary (#1052)", () => {
  it("runs a word requested → forged → found, credited to the first claim only", () => {
    const s = new MemoryMvpStore();
    requestForPair(s, "Shadow", "Magpie", [{ word: "steal", part: "x" }], at);
    expect(dictionary(s, []).map((e) => [e.word, e.state])).toEqual([["steal", "requested"]]);
    expect(dictionary(s, [steal])[0]).toMatchObject({ state: "forged", kind: "steal", card: 2001 });
    expect(forgedNotFound(dictionary(s, [steal]))).toBe(1);

    const design = { name: "Pickpocket", abilities: { Take: { effects: [{ kind: "steal", stat: "pwr" }] } } };
    expect(forgedWordsIn({ name: "Plain", effects: [{ kind: "damage" }] }, [steal])).toEqual([]);
    expect(foundWordsIn(s, design, pairKey("Shadow", "Magpie"), "kim", at, [steal])).toEqual([steal]);
    expect(foundWordsIn(s, design, "Cat+Dog", "lee", at, [steal])).toEqual([]);
    const [e] = dictionary(s, [steal]);
    expect(e.state).toBe("found");
    expect(e.found).toMatchObject({ pair: "Magpie+Shadow", playerId: "kim", kind: "steal" });
    expect(forgedNotFound(dictionary(s, [steal]))).toBe(0);
  });

  it("reads the forged list from the repo, and every entry is a registered word with a card", () => {
    const list = forgedWords();
    expect(list.length).toBeGreaterThan(0);
    for (const f of list) {
      expect(ALL_WORDS.some((w) => w.kind === f.kind), `${f.word}: no word "${f.kind}" in the registry`).toBe(true);
      expect(Number.isInteger(f.card) && f.card > 0, `${f.word}: card`).toBe(true);
      expect(wordKey(f.word)).toBe(f.word);
    }
    expect(forgedWords(join(tmpdir(), "no-such-forged-words.json"))).toEqual([]);
    const dir = mkdtempSync(join(tmpdir(), "forged-"));
    writeFileSync(join(dir, "f.json"), JSON.stringify([steal]));
    expect(forgedWords(join(dir, "f.json"))).toEqual([steal]);
  });
});

describe("the word card (#1052)", () => {
  it("names the word, the asks of every source, and the steps", () => {
    const s = new MemoryMvpStore();
    requestWord(s, "idea", "i1", { word: "steal", part: "steals the enemy's gold" }, at);
    requestForPair(s, "Shadow", "Magpie", [{ word: "steal", part: "takes a power" }], at);
    const { title, body } = wordCard(dictionary(s, [])[0]);
    expect(title).toBe("Arena v2 · word forge: steal");
    expect(body).toContain("- idea `i1`: steals the enemy's gold (2026-10-14)");
    expect(body).toContain("- night `Magpie+Shadow`: takes a power");
    expect(body).toMatch(/golden/);
    expect(body).toMatch(/mvp:meta/);
    expect(body).not.toContain("{{");
  });
});
