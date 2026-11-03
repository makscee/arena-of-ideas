import { describe, expect, it } from "vitest";
import { coachStep, coaching, type CoachState, type CoachUnit } from "./coach";

const UNITS: Record<string, CoachUnit> = {
  gambler: { id: "gambler", name: "Gambler", when: "Strikes" },
  hornet: { id: "hornet", name: "Hornet", when: "Strikes" },
  medic: { id: "medic", name: "Medic", when: "Ally hit" },
  drummer: { id: "drummer", name: "Drummer", when: "Ally strike" },
  lantern: { id: "lantern", name: "Lantern", when: "Turn start" },
};
const shop = (ids: string[], line: string[] = [], gold = 10): CoachState => ({
  line,
  offers: ids.map((unitId, slot) => ({ slot, unitId, cost: 3 })),
  gold,
  room: 5 - line.length,
  unit: (id) => UNITS[id],
});

describe("the first run's coach", () => {
  it("starts with the striker when the shop holds a pair", () => {
    expect(coachStep(shop(["lantern", "medic", "gambler"]))).toEqual({ step: "buy-first", offer: 2, anchor: "Gambler", answer: "Medic" });
  });
  it("then buys the answerer behind it", () => {
    expect(coachStep(shop(["lantern", "medic", "gambler"], ["gambler"], 7))).toEqual({ step: "buy-second", offer: 1, anchor: "Gambler", answer: "Medic" });
  });
  it("says Fight once the pair stands in order", () => {
    expect(coachStep(shop(["lantern"], ["gambler", "medic"], 4))).toEqual({ step: "fight", anchor: "Gambler", answer: "Medic" });
  });
  it("does not call an answerer in front of its anchor a pair", () => {
    expect(coachStep(shop(["lantern"], ["medic", "gambler"], 4))).toBeNull();
  });
  it("prefers the best pair the shop offers", () => {
    expect(coachStep(shop(["drummer", "hornet", "medic"]))).toEqual({ step: "buy-first", offer: 1, anchor: "Hornet", answer: "Medic" });
  });
  it("stays silent when the shop has no answerer, or the pair costs too much", () => {
    expect(coachStep(shop(["lantern", "hornet", "gambler"]))).toBeNull();
    expect(coachStep(shop(["gambler", "medic"], [], 5))).toBeNull();
  });
  it("coaches round 1 of a first run before its first fight only", () => {
    expect(coaching({ round: 1, fights: 0, firstRun: true })).toBe(true);
    expect(coaching({ round: 1, fights: 0, firstRun: false })).toBe(false);
    expect(coaching({ round: 2, fights: 1, firstRun: true })).toBe(false);
  });
});
