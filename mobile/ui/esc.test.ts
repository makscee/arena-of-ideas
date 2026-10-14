import { describe, expect, it } from "vitest";
import { backStays, escStep } from "./esc";

describe("escStep", () => {
  const none = { popover: false, overlays: 0, field: "none" } as const;
  it("closes the popover before the sheet under it", () => {
    expect(escStep({ popover: true, overlays: 2, field: "plain" })).toBe("popover");
  });
  it("closes the top overlay before a field or the screen", () => {
    expect(escStep({ ...none, overlays: 1, field: "clearable" })).toBe("overlay");
  });
  it("lets the browser clear a search, then blurs it", () => {
    expect(escStep({ ...none, field: "clearable" })).toBe("native");
    expect(escStep({ ...none, field: "plain" })).toBe("blur");
  });
  it("hands the key to the screen when nothing is open", () => {
    expect(escStep(none)).toBe("screen");
  });
});

describe("backStays", () => {
  it("keeps Back in the game while a sheet is open, on any screen", () => {
    expect(backStays({ overlays: 1, screen: "home" })).toBe(true);
    expect(backStays({ overlays: 1, screen: "" })).toBe(true);
  });
  it("keeps Back in the game in a battle: Back is its Esc", () => {
    expect(backStays({ overlays: 0, screen: "battle" })).toBe(true);
  });
  it("leaves Back to the history on every other screen: it steps to the screen before", () => {
    for (const screen of ["home", "shop", "codex", "stats", "over", "ideas", "forge", ""]) expect(backStays({ overlays: 0, screen })).toBe(false);
  });
});
