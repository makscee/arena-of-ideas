// Slice B of mission arena#34: every live unit has a sigil spec that passes the
// engine's validate. The pixel checks (inside its circle, fills 44 px, alive, no
// two share a silhouette at 44 px) need a browser: `npm run mvp:portraits`.
import { describe, expect, test } from "vitest";
import { MAX_COPIES, MAX_SHAPES, shapeCount, validate, type Layer, type Spec } from "../../mobile/ui/sigil/spec.js";
import duel from "../../mobile/ui/sigil/duel-specs.json";
import { PORTRAITS, portraitOf } from "./portraits.js";
import { mvpPool } from "./units.js";
import { ROWS_V2 } from "./units-v2.js";

const pool = mvpPool(ROWS_V2);
/** The bodies a Chrysalis becomes (units-v2.ts `butterfly`): on the field, never in the shop. */
const TRANSFORMS = ["Butterfly", "Monarch"];
/** Every summoned body the name grammar can call (units.ts SUMMON_EMOJI). */
const SUMMONED = ["Imp", "Wolf", "Golem", "Wraith", "Treant", "Warg", "Ghoul", "Puffball", "Chick", "Head", "Spiderling"];
const live = [...pool.units.map((u) => u.name), ...SUMMONED, ...TRANSFORMS];

const layers = (list: Layer[] | undefined): Layer[] => (list ?? []).flatMap((l) => [l, ...layers(l.layers)]);

describe("every live unit has a sigil spec", () => {
  test("the pool has 60 units and every one is in the file", () => {
    expect(pool.units).toHaveLength(60);
    expect(live.filter((n) => !PORTRAITS[n])).toEqual([]);
  });
  test("the file holds no spec for a unit that is not live", () => {
    expect(Object.keys(PORTRAITS).filter((n) => !live.includes(n))).toEqual([]);
  });
  test.each(live)("%s passes validate within the budget", (name) => {
    const spec = PORTRAITS[name] as Spec;
    expect(validate(spec)).toEqual({ ok: true, problems: [] });
    expect(shapeCount(spec)).toBeLessThanOrEqual(MAX_SHAPES);
    expect(Object.keys(spec).every((k) => k === "body" || k === "mark")).toBe(true);
  });
  test.each(live)("%s is short: about 12 layers, copies capped", (name) => {
    const spec = PORTRAITS[name] as Spec;
    const all = [...layers(spec.body), ...layers(spec.mark)];
    expect(all.length).toBeLessThanOrEqual(14);
    for (const l of all) if (typeof l.count === "number") expect(l.count).toBeLessThanOrEqual(MAX_COPIES);
  });
  test.each(live)("%s has a mark and grows something awoken", (name) => {
    const spec = PORTRAITS[name] as Spec;
    expect(spec.mark?.length, "a mark: the sign of its rule").toBeGreaterThan(0);
    expect(JSON.stringify(spec), "aw grows something on the awoken form").toMatch(/\baw\b/);
  });
  test("no two units share a spec", () => {
    const seen = new Map<string, string>();
    for (const n of live) {
      const key = JSON.stringify(PORTRAITS[n]!.body);
      expect(seen.get(key), `${n} has the body of ${seen.get(key)}`).toBeUndefined();
      seen.set(key, n);
    }
  });
  test("the duel's 16 hand-written specs are the start: same body, its mark kept first", () => {
    for (const [name, spec] of Object.entries(duel) as [string, Spec][]) {
      const mine = PORTRAITS[name] as Spec;
      expect(mine.body, name).toEqual(spec.body);
      expect(mine.mark?.slice(0, spec.mark?.length ?? 0), name).toEqual(spec.mark ?? []);
    }
  });
});

describe("a unit exposes its portrait", () => {
  test("every pool unit and summoned body in the content carries its spec", () => {
    for (const u of pool.units) expect(u.portrait, u.name).toEqual(PORTRAITS[u.name]);
    expect(pool.summons.length).toBeGreaterThan(0);
    for (const s of pool.summons) expect(s.portrait, s.name).toEqual(PORTRAITS[s.name]);
  });
  test("the content's copy is its own: editing it leaves the file alone", () => {
    const u = mvpPool(ROWS_V2).units[0]!;
    const before = JSON.stringify(PORTRAITS[u.name]);
    (u.portrait!.body[0] as Layer).fill = "ink";
    expect(JSON.stringify(PORTRAITS[u.name])).toBe(before);
  });
  test("portraitOf knows a name or says nothing (the sigil then draws the generic one)", () => {
    expect(portraitOf("Hourglass")).toBe(PORTRAITS.Hourglass);
    for (const name of ["Nobody", "constructor", "__proto__", "toString"]) expect(portraitOf(name)).toBeUndefined();
  });
});
