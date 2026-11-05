import { describe, expect, it } from "vitest";
import { GLYPH_NAMES, LEGACY_GLYPH, glyphName, isGlyphName, statusGlyph } from "../../../src/mvp/glyph-names";
import { GLYPH_SPECS, glyphSpec } from "./glyphs";
import { MAX_SHAPES, shapeCount, validate } from "./spec";

describe("the glyph set (arena#41)", () => {
  it("every name has a spec, and no spec lacks a name", () => {
    expect(Object.keys(GLYPH_SPECS).sort()).toEqual([...GLYPH_NAMES].sort());
  });
  it("every spec passes the sigil validator and stays inside the shape budget", () => {
    for (const name of GLYPH_NAMES) {
      const v = validate(GLYPH_SPECS[name]);
      expect(v.problems, name).toEqual([]);
      expect(shapeCount(GLYPH_SPECS[name]), name).toBeLessThanOrEqual(MAX_SHAPES);
    }
  });
  it("a glyph is a bare silhouette: every colour is in the palette, nothing reads state", () => {
    for (const name of GLYPH_NAMES) expect(JSON.stringify(GLYPH_SPECS[name]), name).not.toMatch(/\b(hp|pwr|charge|shield|hit|fire|die|aw|beat|\bt\b)\b/);
  });
  it("every emoji of the old faces maps to a name in the set", () => {
    for (const [emoji, name] of Object.entries(LEGACY_GLYPH)) {
      expect(isGlyphName(name), emoji).toBe(true);
      expect(glyphName(emoji), emoji).toBe(name);
    }
  });
  it("an unknown name draws the spark", () => {
    expect(glyphSpec("unicorn")).toBe(GLYPH_SPECS.spark);
    expect(glyphSpec("constructor")).toBe(GLYPH_SPECS.spark);
    expect(glyphName("🦄")).toBe("spark");
  });
  it("every status the game has draws a glyph of the set, and the rest the spark", () => {
    for (const s of ["Shield", "Strength", "Vitality", "Blessing", "Charge", "Poison", "Curse", "Freeze", "Silence"]) {
      const [g] = statusGlyph(s);
      expect(isGlyphName(g), s).toBe(true);
      expect(g, s).not.toBe("spark");
    }
    expect(statusGlyph("Mystery")).toEqual(["spark", "good"]);
  });
});
