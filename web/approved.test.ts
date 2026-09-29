// Logged-out play resolves approved Units' Abilities (#466): the shipped set plus
// every Ability the committed registry carries.

import { describe, expect, test } from "vitest";
import { stressAbilities } from "../src/index.js";
import approvedJson from "../registry/approved-units.json";
import { approvedAbilities, approvedUnits } from "./approved.js";

describe("approvedAbilities", () => {
  test("keeps the shipped Abilities and adds every committed registry Ability", () => {
    const abilities = approvedAbilities();
    for (const id of Object.keys(stressAbilities)) expect(abilities[id]).toEqual(stressAbilities[id]);
    const committed = (approvedJson as { abilities?: Record<string, unknown> }).abilities ?? {};
    for (const [id, ability] of Object.entries(committed)) expect(abilities[id]).toEqual(ability);
  });

  test("every approved Unit's Ability resolves", () => {
    const abilities = approvedAbilities();
    for (const unit of approvedUnits()) for (const id of unit.abilities ?? []) expect(abilities[id], `${unit.name} → ${id}`).toBeDefined();
  });
});
