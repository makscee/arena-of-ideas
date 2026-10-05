import { describe, expect, it } from "vitest";
import type { FusionDiscovery, PlayerRef, UnitContent } from "../../../src/mvp/contract.js";
import { fusionNaming, portmanteau, storedOrPortmanteau } from "./fusions.js";
import { MemoryMvpStore } from "./store.js";

const form = { when: [], who: [], does: ["a"] };
const unit = (id: string, name: string): UnitContent => ({ id, name, emoji: "x", tier: 1, base: { pwr: 1, hp: 1 }, forms: { sleeping: form, awoken: form } });
const brawler = unit("brawler", "Brawler");
const medic = unit("medic", "Medic");
const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };
const bot: PlayerRef = { id: "bot", name: "bot-Ash", bot: true };

describe("MVP fusion names seam (slice 10 owns naming)", () => {
  it("falls back to a deterministic portmanteau", () => {
    expect(portmanteau("Brawler", "Medic")).toBe("Brawdic");
    expect(portmanteau("Brawler", "Medic")).toBe(portmanteau("Brawler", "Medic"));
    expect(portmanteau("Medic", "Brawler")).not.toBe(portmanteau("Brawler", "Medic"));
  });

  it("credits a human, never a bot, and keeps a stored name and credit", () => {
    const store = new MemoryMvpStore();
    const name = storedOrPortmanteau(store);
    expect(name(brawler, medic, maks)).toEqual({ name: "Brawdic", discoveredBy: maks });
    expect(name(brawler, medic, bot)).toEqual({ name: "Brawdic", discoveredBy: null });

    const byBot: FusionDiscovery = { first: "brawler", second: "medic", name: "Ironcare", discoveredBy: null, discoveredAt: "t", nameSource: "model" };
    store.putFusion(byBot);
    expect(name(brawler, medic, maks)).toEqual({ name: "Ironcare", discoveredBy: maks });
    store.putFusion({ ...byBot, discoveredBy: maks });
    expect(name(brawler, medic, { id: "p2", name: "Eva", bot: false })).toEqual({ name: "Ironcare", discoveredBy: maks });
    expect(store.fusion("medic", "brawler")).toBeUndefined();
    expect(store.fusions()).toHaveLength(1);
  });

  it("fusionNaming: decide's namer and the preview's agree; no hooks until slice 10", () => {
    const store = new MemoryMvpStore();
    const naming = fusionNaming({ store, content: { version: "t", units: [brawler, medic], abilities: {}, statuses: {} }, now: () => new Date(0) });
    expect(naming.nameFusion(brawler, medic, maks)).toEqual(naming.peek(brawler, medic, maks));
    expect(naming.peek(brawler, medic, maks)).toEqual({ name: "Brawdic", discoveredBy: maks });
    expect(naming.hooks).toEqual({});
  });
});
