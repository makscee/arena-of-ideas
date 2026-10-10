import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { pickTheme, THEME_SWATCH, THEMES } from "./theme";

const css = readFileSync(new URL("./style.css", import.meta.url), "utf8");

describe("palette switch (M5-1)", () => {
  it("defaults to Night Plum, takes a saved choice, and ?theme= wins", () => {
    expect(pickTheme(null, null)).toBe("plum");
    expect(pickTheme(null, "nonsense")).toBe("plum");
    expect(pickTheme(null, "felt")).toBe("felt");
    expect(pickTheme("brass", "felt")).toBe("brass");
    expect(pickTheme("nope", "felt")).toBe("felt");
  });

  it("every palette sets every colour role in style.css", () => {
    const block = (sel: string) => {
      const at = css.indexOf(sel);
      expect(at, sel).toBeGreaterThanOrEqual(0);
      return css.slice(at, css.indexOf("}", at));
    };
    const roles = (s: string) => [...s.matchAll(/(--[a-z0-9-]+):\s*#/g)].map((m) => m[1]).sort();
    const plum = roles(block(":root, .swatch[data-theme=\"plum\"] {"));
    expect(plum.length).toBeGreaterThanOrEqual(8);
    for (const t of THEMES.filter((x) => x !== "plum")) expect(roles(block(`:root[data-theme="${t}"]`)), t).toEqual(plum);
    // The swatches in the switch are the token values.
    for (const t of THEMES) {
      const b = t === "plum" ? block(":root, .swatch[data-theme=\"plum\"] {") : block(`:root[data-theme="${t}"]`);
      const [ground, you, them, action] = THEME_SWATCH[t];
      expect(b).toContain(`--ground: ${ground};`);
      expect(b).toContain(`--you: ${you};`);
      expect(b).toContain(`--them: ${them};`);
      expect(b).toContain(`--action: ${action};`);
    }
  });

  it("has no stray hex colour outside the token block", () => {
    const end = css.indexOf("/* end of tokens */");
    expect(end).toBeGreaterThan(0);
    const rest = css.slice(end).replace(/\/\*[\s\S]*?\*\//g, "");
    expect(rest.match(/#[0-9a-fA-F]{3,8}\b/g) ?? []).toEqual([]);
    expect(rest.match(/rgba?\(/g) ?? []).toEqual([]);
  });
});
