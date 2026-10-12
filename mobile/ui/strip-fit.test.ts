import { describe, expect, test } from "vitest";
import { mvpPool } from "../../src/mvp/units";
import { RU_STATUSES, ruStatusName } from "../../src/describe-ru";
import { changeOf, setTraceLang } from "../../src/mvp/trace";
import type { BattleEvent } from "../../src/types";
import { numFormOf, planStrip, type StripMeasure } from "./strip-fit";

// Pixel widths in the 16px client font, a little generous: names ~10 px a character, the bold mono number ~12.
const NAME_PX = 10;
const NUM_PX = 12;
const EM = 16;
const ACTOR_MIN = (6 * 0.62 + 0.8) * EM;
const TARGET_MIN = 2.4 * EM;
const GAP = 4;
const LINE = 287; // a 360 px phone's strip line
// Arrow, arrow, the effect icon, a "+n" more targets, the dots at their least (28), and the gaps between those and the two names.
const OTHERS = 13 + 13 + 18 + 24 + 28 + GAP * 6;

const names = (): string[] => {
  const pool = mvpPool();
  const own = pool.units.flatMap((u) => [u.name, u.texts?.ru?.name ?? u.name]);
  // A fused unit's generated name, and a name no content has yet.
  return [...own, "Заглушительница", "Stormfang-the-Unbroken-of-Many-Words", "Ы"];
};

/** Every number the strip can show, in both languages: the real labels from changeOf, with the content's longest status names and big counts. */
function labels(): { label: string; kind: string }[] {
  const out: { label: string; kind: string }[] = [];
  const statuses = [...Object.keys(mvpPool().statuses), ...Object.keys(RU_STATUSES), ...Object.keys(RU_STATUSES).map(ruStatusName)];
  for (const lang of ["en", "ru"] as const) {
    setTraceLang(lang);
    const evs: unknown[] = [
      { id: 1, type: "Hurt", unit: "u", amount: 0, absorbed: 0 },
      { id: 1, type: "Hurt", unit: "u", amount: 0, absorbed: 12 },
      { id: 1, type: "Hurt", unit: "u", amount: 128 },
      { id: 1, type: "Heal", unit: "u", amount: 128 },
      { id: 1, type: "StatChanged", unit: "u", stat: "pwr", delta: -12 },
      { id: 1, type: "StatChanged", unit: "u", stat: "hp", delta: 12 },
      { id: 1, type: "Silenced", unit: "u" },
      ...statuses.map((status) => ({ id: 1, type: "StatusApplied", unit: "u", status, stacks: 99 })),
    ];
    for (const e of evs) {
      const c = changeOf(e as BattleEvent);
      if (c) out.push({ label: c.label, kind: c.kind });
    }
  }
  setTraceLang(undefined);
  return out;
}

const widthOf = (text: string, px: number) => [...text].length * px;

function measureOf(actor: string, target: string | null, label: string, kind: string): StripMeasure {
  const f = numFormOf(label, kind);
  const short = f.short === null ? label : f.short;
  return {
    avail: LINE,
    others: OTHERS,
    actor: { nat: widthOf(actor, NAME_PX), min: Math.min(widthOf(actor, NAME_PX), ACTOR_MIN) },
    target: target === null ? null : { nat: widthOf(target, NAME_PX), min: Math.min(widthOf(target, NAME_PX), TARGET_MIN) },
    num: { full: widthOf(label, NUM_PX) + GAP, short: short ? widthOf(short, NUM_PX) + GAP : 0 },
  };
}

describe("action strip fit", () => {
  test("the longest actor, target and effect words in the content (EN and RU) always fit a 287 px line", () => {
    const all = names();
    const longest = [...all].sort((a, b) => [...b].length - [...a].length).slice(0, 5);
    const shortest = [...all].sort((a, b) => [...a].length - [...b].length).slice(0, 3);
    let n = 0;
    for (const { label, kind } of labels()) {
      for (const actor of [...longest, ...shortest]) {
        for (const target of [...longest, ...shortest, null]) {
          const m = measureOf(actor, target, label, kind);
          const p = planStrip(m);
          const total = m.others + p.actor + p.target + (p.short ? m.num.short : m.num.full);
          expect(total, `${actor} → ${target} ${label}`).toBeLessThanOrEqual(LINE);
          expect(p.actor).toBeGreaterThanOrEqual(m.actor.min);
          expect(p.target).toBeGreaterThanOrEqual(m.target?.min ?? 0);
          n++;
        }
      }
    }
    expect(n).toBeGreaterThan(500);
  });

  test("the failing line from the field (Silencer → Imp, «заглушён») drops the word to its icon, names stay", () => {
    const p = planStrip(measureOf("Silencer", "Imp", "заглушён", "silence"));
    expect(p.short).toBe(true);
    expect(p.actor).toBe(80);
    expect(p.target).toBe(30);
  });

  test("the order is fixed: target shrinks, then actor, then the word; a roomy line changes nothing", () => {
    const base = measureOf("Silencer", "Distractor", "Curse ×1", "status");
    expect(planStrip({ ...base, avail: 2000 })).toEqual({ actor: 80, target: 100, short: false });
    // 30 px short: only the target gives.
    const need = base.others + 80 + 100 + base.num.full;
    expect(planStrip({ ...base, avail: need - 30 })).toEqual({ actor: 80, target: 70, short: false });
    // Past the target's floor the actor gives.
    const p2 = planStrip({ ...base, avail: base.others + 80 + TARGET_MIN + base.num.full - 5 });
    expect(p2.target).toBe(Math.ceil(TARGET_MIN));
    expect(p2.actor).toBeLessThan(80);
    expect(p2.short).toBe(false);
    // Past both floors the word goes, and the names take the room back.
    const p3 = planStrip({ ...base, avail: base.others + ACTOR_MIN + TARGET_MIN + base.num.full - 1 });
    expect(p3.short).toBe(true);
    expect(p3.actor + p3.target).toBeGreaterThan(Math.ceil(ACTOR_MIN) + Math.ceil(TARGET_MIN));
  });

  test("a line that holds everything leaves the names at their exact natural width (no rounding that cuts a letter)", () => {
    const m = { ...measureOf("Gnat", "Prepper", "1", "damage"), avail: 400 };
    const nat = { actor: 36.4, target: 61.2 };
    const p = planStrip({ ...m, actor: { nat: nat.actor, min: 36.4 }, target: { nat: nat.target, min: 38.4 } });
    expect(p).toEqual({ actor: nat.actor, target: nat.target, short: false });
  });

  test("numFormOf keeps the count, or nothing when the number is only a word", () => {
    expect(numFormOf("Curse ×1", "status")).toMatchObject({ word: "Curse", count: "×1", short: "×1" });
    expect(numFormOf("12 blocked", "damage")).toMatchObject({ word: "blocked", count: "12", short: "12" });
    expect(numFormOf("no damage", "damage")).toMatchObject({ short: "0", wordOnly: true });
    expect(numFormOf("заглушён", "silence")).toMatchObject({ short: "", wordOnly: true });
    expect(numFormOf("−4", "damage")).toMatchObject({ short: null });
    expect(numFormOf("+2 ATK", "buff")).toMatchObject({ word: "ATK", count: "+2", short: "+2" });
    expect(numFormOf("+3", "heal")).toMatchObject({ short: null });
  });
});
