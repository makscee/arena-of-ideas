// M4-4: Russian fusion names beside the English ones (./fusion-names-ru.ts).
import { describe, expect, it } from "vitest";
import type { FusionDiscovery } from "../../../src/mvp/contract.js";
import { cleanRuName, nameFusionsRu, namerMessagesRu, type RuNamer } from "./fusion-names-ru.js";
import { seedUnits } from "./pool.js";
import { MemoryMvpStore } from "./store.js";

function world() {
  const store = new MemoryMvpStore();
  seedUnits(store, new Date("2026-10-08T08:00:00.000Z"));
  const [a, b, c] = store.units().map((u) => u.unitId);
  const f = (first: string, second: string, name: string, at: string): FusionDiscovery => ({ first, second, name, discoveredBy: null, discoveredAt: at, nameSource: "model" });
  store.putFusion(f(a!, b!, "Stormfang", "2026-10-08T09:00:00.000Z"));
  store.putFusion(f(b!, c!, "Moonhide", "2026-10-08T10:00:00.000Z"));
  store.putUnit({ ...store.unit(a!)!, texts: { ru: { name: "Боец", line: "Простой драчун." } } });
  return { store, a: a!, b: b!, c: c! };
}

describe("nameFusionsRu", () => {
  it("names each stored discovery in Russian beside the English, by the parts' Russian names", async () => {
    const { store, a, b, c } = world();
    const asked: string[] = [];
    const names = ["Грозоклык", "Лунокож"];
    const namer: RuNamer = async (x, y) => (asked.push(`${x}+${y}`), names[asked.length - 1] ?? null);
    const r = await nameFusionsRu(store, namer);
    expect(r.named.map((n) => n.name)).toEqual(["Грозоклык", "Лунокож"]);
    expect(asked[0]).toMatch(/^Боец\+/);
    expect(store.fusion(a, b)).toMatchObject({ name: "Stormfang", texts: { ru: { name: "Грозоклык" } } });
    expect(store.fusion(b, c)!.texts!.ru!.name).toBe("Лунокож");
    // the second pass finds nothing to name
    expect((await nameFusionsRu(store, namer)).named).toEqual([]);
  });

  it("asks again for an unusable or taken answer, leaves English after three, and writes nothing on a dry run", async () => {
    const { store, a, b, c } = world();
    const answers = ["Stormfang", "Боец", "🔥", "Грозоклык", "Грозоклык", "Грозоклык", "Грозоклык"];
    const r = await nameFusionsRu(store, async () => answers.shift() ?? null, { dryRun: true });
    // Stormfang: English, a unit's Russian name, an emoji; Moonhide: the next answer
    expect(r.failed.map((f) => f.name)).toEqual(["Stormfang"]);
    expect(r.named.map((n) => [n.fusion.name, n.name])).toEqual([["Moonhide", "Грозоклык"]]);
    expect(store.fusion(a, b)!.texts).toBeUndefined();
    expect(store.fusion(b, c)!.texts).toBeUndefined();
  });

  it("stops the pass when the namer is down", async () => {
    const { store } = world();
    const r = await nameFusionsRu(store, async () => {
      throw new Error("namer answered 503");
    });
    expect(r).toMatchObject({ named: [], down: true });
  });
});

describe("cleanRuName", () => {
  it("keeps a short Cyrillic name and refuses the rest", () => {
    expect(cleanRuName("«грозоклык».", "Боец", "Мошка")).toBe("Грозоклык");
    expect(cleanRuName("Имя: Лунный страж\nпотому что", "Боец", "Мошка")).toBe("Лунный Страж");
    expect(cleanRuName("Stormfang", "Боец", "Мошка")).toBeNull();
    expect(cleanRuName("БоецМошка", "Боец", "Мошка")).toBeNull();
    expect(cleanRuName("Пиздорез", "Боец", "Мошка")).toBeNull();
  });
  it("asks the model in Russian, the pair last", () => {
    const m = namerMessagesRu("Боец", "Мошка");
    expect(m[0]!.role).toBe("system");
    expect(m.at(-1)!.content).toBe("Сливаются: Боец и Мошка. Имя:");
  });
});
