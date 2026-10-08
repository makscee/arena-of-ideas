// M4-4: Russian names and lines beside the stored units (./translate.ts).
import { describe, expect, it } from "vitest";
import { isCrudeRuName } from "./crude.js";
import { poolContent, seedUnits, servedContent } from "./pool.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore, nameKey, type MvpStore } from "./store.js";
import { cyrillic, fakeTranslator, reviewPage, ruTextProblem, translatePrompt, translateUnits, type Translator } from "./translate.js";

function seeded(store: MvpStore = new MemoryMvpStore()): MvpStore {
  seedUnits(store, new Date("2026-10-08T08:00:00.000Z"));
  return store;
}

describe("translateUnits", () => {
  it("gives every unit a Russian name and line, unique, and leaves rows and the content version alone", async () => {
    const store = seeded(new SqliteMvpStore(":memory:"));
    const rows = new Map(store.units().map((u) => [u.unitId, JSON.stringify(u.row)]));
    const before = { pool: store.currentPool()!.version, built: poolContent(store).version };
    const r = await translateUnits(store, fakeTranslator());
    expect(r.failed).toEqual([]);
    expect(r.done.length).toBe(store.units().length);
    for (const u of store.units()) {
      expect(u.texts?.ru?.name, u.unitId).toBeTruthy();
      expect(JSON.stringify(u.row)).toBe(rows.get(u.unitId));
    }
    const names = store.units().map((u) => nameKey(u.texts!.ru!.name));
    expect(new Set(names).size).toBe(names.length);
    expect(store.currentPool()!.version).toBe(before.pool);
    expect(poolContent(store).version).toBe(before.built);
  });

  it("writes nothing on a dry run, and keeps units that have a text on the next run", async () => {
    const store = seeded();
    const dry = await translateUnits(store, fakeTranslator(), { dryRun: true });
    expect(dry.done.length).toBeGreaterThan(0);
    expect(store.units().some((u) => u.texts)).toBe(false);
    await translateUnits(store, fakeTranslator());
    const again = await translateUnits(store, fakeTranslator());
    expect(again.done).toEqual([]);
    expect(again.kept.length).toBe(store.units().length);
  });

  it("refuses a taken, crude or English answer and asks again with the reason", async () => {
    const store = seeded();
    const ids = store.units().slice(0, 3).map((u) => u.unitId);
    const asked: string[][] = [];
    const tr: Translator = {
      kind: "test",
      async translate(units, _taken, notes) {
        asked.push(units.map((u) => `${u.unitId}${notes?.get(u.unitId) ? "!" : ""}`));
        if (asked.length > 1) return fakeTranslator().translate(units, _taken);
        return units.map((u, i) => ({ unitId: u.unitId, name: ["Страж", "Страж", "Knight"][i] ?? "Хуйло", line: "Стоит впереди." }));
      },
    };
    const r = await translateUnits(store, tr, { batch: 200 });
    expect(r.failed).toEqual([]);
    expect(store.unit(ids[0]!)!.texts!.ru!.name).toBe("Страж");
    // the second "Страж", the English "Knight" and every crude name are asked again, with a note
    expect(asked[1]!.filter((x) => x.endsWith("!")).length).toBe(store.units().length - 1);
  });

  it("gives up after its tries and says why", async () => {
    const store = seeded();
    const tr: Translator = { kind: "test", translate: async () => [] };
    const r = await translateUnits(store, tr, { batch: 50 });
    expect(r.done).toEqual([]);
    expect(r.failed.length).toBe(store.units().length);
    expect(r.failed[0]!.why).toBe("no answer");
    expect(reviewPage(r)).toContain("failed: no answer");
  });
});

describe("ruTextProblem", () => {
  const ok = { name: "Оруженосец", line: "Новичок, который сам себя усиливает." };
  it("passes a short Russian name and a one-sentence line", () => expect(ruTextProblem(ok, new Set())).toBeNull());
  it("refuses English, crude, taken and overlong names, and lines without a full stop", () => {
    expect(ruTextProblem({ ...ok, name: "Squire" }, new Set())).toMatch(/Russian words/);
    expect(ruTextProblem({ ...ok, name: "Ёбарь" }, new Set())).toMatch(/crude/);
    expect(ruTextProblem(ok, new Set([nameKey("оруженосец")]))).toMatch(/already/);
    expect(ruTextProblem({ ...ok, name: "Очень Длинное Имечко Здесь" }, new Set())).not.toBeNull();
    expect(ruTextProblem({ ...ok, line: "Без точки" }, new Set())).toMatch(/full stop/);
  });
  it("reads Russian mat with ё as е", () => {
    expect(isCrudeRuName("Пиздорез")).toBe(true);
    expect(isCrudeRuName("Страж Рассвета")).toBe(false);
  });
});

describe("GET /content texts", () => {
  it("carries a translated unit's Russian text and none for the rest, under the same version", async () => {
    const store = seeded();
    const content = poolContent(store);
    const [a, b] = content.units;
    store.putUnit({ ...store.unit(a!.id)!, texts: { ru: { name: "Боец", line: "Простой драчун." } } });
    const served = servedContent({ store, content });
    expect(served.version).toBe(content.version);
    expect(served.units.find((u) => u.id === a!.id)!.texts).toEqual({ ru: { name: "Боец", line: "Простой драчун." } });
    expect(served.units.find((u) => u.id === b!.id)!.texts).toBeUndefined();
    expect(content.units[0]!.texts).toBeUndefined(); // the cached pool isn't touched
  });
});

it("the fake spells English in Cyrillic and the prompt names every unit", () => {
  expect(cyrillic("Gnat")).toBe("Гнат");
  expect(translatePrompt([{ unitId: "gnat", emoji: "🦟", name: "Gnat", line: "A pest." }], ["Страж"])).toContain("gnat: 🦟 Gnat | A pest.");
});
