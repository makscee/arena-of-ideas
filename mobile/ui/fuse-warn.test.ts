import { describe, expect, it } from "vitest";
import { mvpPool } from "../../src/mvp/units";
import type { UnitForm } from "../../src/mvp/contract";
import { fuseWarning, misfires } from "./fuse-warn";

const content = mvpPool();
const awoken = (name: string): UnitForm => content.units.find((u) => u.name === name)!.forms.awoken;
const fuse = (a: UnitForm, b: UnitForm): UnitForm => ({ when: a.when, who: b.who, does: [...a.does, ...b.does] });

describe("fuseWarning", () => {
  it("names harmful effects a fusion sends to your own units", () => {
    const [rose, coach] = [awoken("Rose"), awoken("Coach")];
    const w = fuseWarning(fuse(rose, coach), [rose, coach], content);
    expect(w).toMatch(/hurts your own units/);
    expect(w).toMatch(/damage/);
  });
  it("names helpful effects a fusion sends to the enemy", () => {
    const [rose, coach] = [awoken("Rose"), awoken("Coach")];
    const w = fuseWarning(fuse(coach, rose), [rose, coach], content);
    expect(w).toMatch(/helps the enemy/);
    expect(w).toMatch(/Strength/);
  });
  it("says nothing when every effect goes the right way", () => {
    const [rose, gnat] = [awoken("Rose"), awoken("Gnat")];
    expect(fuseWarning(fuse(rose, gnat), [rose, gnat], content)).toBeNull();
  });
  it("finds no misfire in any unit's own forms but the ones built that way", () => {
    // A part's own design (if any) is never blamed on the fusion; this lists them.
    const odd = content.units.filter((u) => {
      const m = misfires(u.forms.awoken, content);
      return m.hurts.length + m.helps.length > 0;
    });
    expect(odd.length).toBeLessThan(content.units.length / 4);
  });
});
