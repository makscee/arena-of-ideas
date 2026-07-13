import { expect, test } from "vitest";
import { awakenFusion, buy, fuse, initRun, type RunState } from "../../src/run.js";
import { Necromancer, Silencer, stressAbilities, stressRegistry } from "../../src/content/stress.js";
import { extractReplaySteps } from "./runs.js";

function purchase(s: RunState, def: typeof Necromancer): RunState {
  return buy({ ...s, gold: 999, offers: [def] }, 0);
}

test("server replay extraction preserves ordered fusion and permanent Awakening choice", () => {
  let s = initRun({ seed: 8, pool: [Necromancer, Silencer], statuses: stressRegistry, abilities: stressAbilities });
  for (let i = 0; i < 3; i++) s = purchase(s, Necromancer);
  for (let i = 0; i < 3; i++) s = purchase(s, Silencer);
  s = fuse(s, 1, 0); // explicit Silencer + Necromancer, not line order
  s = purchase(s, Necromancer);
  s = purchase(s, Silencer);
  s = purchase(s, Necromancer);
  s = awakenFusion(s, "selector");

  const steps = extractReplaySteps(s.log);
  expect(steps.filter((step) => step.kind === "fuse")).toEqual([{ kind: "fuse", primary: 1, secondary: 0 }]);
  expect(steps.at(-1)).toEqual({ kind: "awakenFusion", path: "selector" });
  expect(steps.filter((step) => step.kind === "buy")).toHaveLength(9);
  expect(extractReplaySteps(s.log)).toEqual(steps);
});
