// M4-4: a Russian page shows a unit's Russian name and line; a unit without one keeps English.
import { describe, expect, it } from "vitest";
import { mvpPool } from "../src/mvp/units";
import type { MvpContent } from "../src/mvp/contract";
import { localContent, unitName } from "./unit-names";

const pool = mvpPool();
const [a, b] = pool.units;
const content: MvpContent = { version: "v", ...pool, units: [{ ...a!, texts: { ru: { name: "Боец", line: "Простой драчун." } } }, b!], left: [] };

describe("localContent", () => {
  it("reads the Russian name and line on a Russian page, English for a unit without them", () => {
    const ru = localContent(content, "ru");
    expect(ru.version).toBe("v");
    expect(ru.units[0]).toMatchObject({ id: a!.id, name: "Боец", archetype: "Простой драчун." });
    expect(ru.units[1]).toMatchObject({ id: b!.id, name: b!.name, archetype: b!.archetype });
  });
  it("leaves an English page as it came", () => {
    expect(localContent(content, "en")).toBe(content);
  });
});

describe("unitName", () => {
  it("names a server-sent unit in Russian, but never a fused unit's own name", () => {
    localContent(content, "ru");
    expect(unitName(a!.id, a!.name, "ru")).toBe("Боец");
    expect(unitName(a!.id, "Stormfang", "ru")).toBe("Stormfang");
    expect(unitName(b!.id, b!.name, "ru")).toBe(b!.name);
    expect(unitName(a!.id, a!.name, "en")).toBe(a!.name);
  });
});
