import { describe, expect, it } from "vitest";
import { MVP_RULES, type MvpContent } from "./contract.js";
import { fightLines } from "./fight.js";
import { contentFormProblems, fuseUnits, lineUnitOf } from "./forms.js";
import { formText } from "./form-text.js";
import { ARCHETYPE_MAX_WORDS, EMITS, LISTENS, ROOT_WHENS, ROWS, WHEN, archetypeProblems, awokenKeeps, awokenNewPart, effectKinds, linkLoops, mvpPool, shapeKinds, sig, whenKeyOf, type WhenKey } from "./units.js";

describe("MVP pool (slice 7)", () => {
  const pool = mvpPool();
  const content = { version: "test", ...pool };

  it("has about 80 units, every one shippable in both forms", () => {
    // R4-15 cut 8 for unique archetypes: 81 → 73.
    expect(pool.units.length).toBeGreaterThanOrEqual(73);
    expect(contentFormProblems(content)).toEqual([]);
  });

  it("gives every unit its own emoji and a tier", () => {
    const emoji = pool.units.map((u) => u.emoji);
    expect(new Set(emoji).size).toBe(emoji.length);
    // One codepoint (plus an emoji presentation selector): a ZWJ sequence
    // splits into two pictures on older phones.
    expect(pool.units.filter((u) => [...u.emoji.replace(/\uFE0F$/u, "")].length !== 1).map((u) => `${u.name} ${u.emoji}`)).toEqual([]);
    for (const t of [1, 2, 3, 4]) expect(pool.units.some((u) => u.tier === t)).toBe(true);
  });

  it("gives every unit an archetype: one sentence, short, unique (round 4, note 3)", () => {
    expect(archetypeProblems(pool.units)).toEqual([]);
    for (const u of pool.units) expect(u.archetype.split(/\s+/).length).toBeLessThanOrEqual(ARCHETYPE_MAX_WORDS);
  });

  it("archetype check catches an empty, a run-on, a long and a repeated archetype", () => {
    const bad = [
      { name: "A", archetype: "" },
      { name: "B", archetype: "Hits the front. Then heals" },
      { name: "C", archetype: "One two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen." },
      { name: "D", archetype: "Shields the team every turn." },
      { name: "E", archetype: "shields the team every turn." },
    ];
    expect(archetypeProblems(bad)).toEqual([
      "A: no archetype",
      'B: archetype ends with "."',
      "B: archetype is more than one sentence",
      "C: archetype has 16 words (max 15)",
      "E: archetype starts with a capital",
      "E: same archetype as D",
    ]);
  });

  it("contains chain links: units that listen to what other units emit", () => {
    const listened = new Set(pool.units.map((u) => u.forms.sleeping.when[0]!.on.on));
    for (const e of ["StatusApplied", "Heal", "StatChanged", "Summon", "Death"]) expect(listened).toContain(e);
  });
});

describe("MVP pool chain discipline", () => {
  const ONE_UNIT_EVENTS = new Set(["StatusApplied", "Heal", "StatChanged", "Summon"]);
  it("listeners to one-unit events act on one unit (no n² fan-out)", () => {
    for (const u of mvpPool().units) {
      for (const f of [u.forms.sleeping, u.forms.awoken]) {
        const on = f.when[0]!.on.on;
        if (ONE_UNIT_EVENTS.has(on)) expect([u.id, f.who[0]!.kind]).not.toEqual([u.id, expect.stringMatching(/^all/)]);
      }
    }
  });
});

describe("MVP pool targeting", () => {
  // Who a selector reaches, for a given When: friend or foe.
  const ENEMY_EVENTS = new Set(["enemyPoisoned", "enemyCursed", "enemyDies"]);
  const side = (who: string, when: string) =>
    ["front", "enemies", "random"].includes(who) || (who === "it" && ENEMY_EVENTS.has(when)) ? "foe" : "friend";
  const HOSTILE = /^(Hit|Smite|Poison|Curse|Freeze|Silence)/;

  it("helps allies and hurts enemies, in both forms", () => {
    const wrong: string[] = [];
    for (const row of ROWS) {
      const a = row.awoken;
      const forms = [
        { who: row.who, does: [row.does] },
        { who: a.who ?? row.who, does: [...(a.before ?? []), a.more ?? row.does, ...(a.add ?? [])] },
        ...(a.also ? [{ who: a.also.who, does: [a.also.does] }] : []),
      ];
      for (const f of forms) {
        for (const d of f.does) {
          const want = HOSTILE.test(d) ? "foe" : d.startsWith("Call") || d.startsWith("Revive") ? side(f.who, row.when) : "friend";
          if (side(f.who, row.when) !== want) wrong.push(`${row.name}: ${d} → ${f.who}`);
        }
      }
    }
    expect(wrong).toEqual([]);
  });
});

describe("Planter", () => {
  const content: MvpContent = { version: "test", ...mvpPool() };
  const unit = (id: string) => content.units.find((u) => u.id === id)!;
  const fight = (ids: string[], rules = MVP_RULES) => {
    const line = ids.map((id, i) => lineUnitOf(unit(id), `a${i}`));
    const foe = [lineUnitOf(unit("fodder"), "b0")];
    const p = { id: "p", name: "p", bot: false };
    return fightLines({ player: p, line }, { player: p, line: foe }, { battleId: "b", seed: 1, kind: "round", round: 1, runId: null, at: "2026-10-06T00:00:00.000Z", content, rules }).log;
  };
  const grew = (log: ReturnType<typeof fight>) => log.some((e) => e.type === "StatusApplied" && e.status === "Vitality" && e.unit.includes("Planter"));
  const summoned = (log: ReturnType<typeof fight>) => log.some((e) => e.type === "Summon" && e.name === "Imp");

  it("calls an Imp when the line has room, and grows", () => {
    const log = fight(["planter", "fighter"]);
    expect(summoned(log)).toBe(true);
    expect(grew(log)).toBe(true);
  });

  it("an awoken Planter's Treant enters at the front and strikes in the first clash (R3)", () => {
    const line = [lineUnitOf(unit("planter"), "a0", 3), lineUnitOf(unit("fighter"), "a1")];
    const foe = [lineUnitOf(unit("fodder"), "b0")];
    const p = { id: "p", name: "p", bot: false };
    const log = fightLines({ player: p, line }, { player: p, line: foe }, { battleId: "b", seed: 1, kind: "round", round: 1, runId: null, at: "2026-10-06T00:00:00.000Z", content, rules: MVP_RULES }).log;
    const s = log.find((e) => e.type === "Summon" && e.name === "Treant");
    if (s?.type !== "Summon") throw new Error("no Treant summoned");
    expect(s.front).toBe(true);
    const pair = log.find((e) => e.type === "PairFaced");
    expect(pair?.type === "PairFaced" && pair.a).toBe(s.unit);
    expect(log.some((e) => e.type === "Strike" && e.striker === s.unit && e.turn === pair!.turn)).toBe(true);
  });

  it("still does something in a full line: it grows", () => {
    // A team of 5 fills the line only under a stored run's line of 5 (R4-10:
    // new runs fight on a line of 8, so there it calls the Imp too).
    const { battleSize: _bs, ...lineOf5 } = MVP_RULES;
    const log = fight(["planter", "fighter", "squire", "gnat", "rose"], lineOf5);
    expect(summoned(log)).toBe(false);
    expect(grew(log)).toBe(true);
    expect(summoned(fight(["planter", "fighter", "squire", "gnat", "rose"]))).toBe(true);
  });
});

describe("one hero per shape (round 3, docs/round3/units.md 1b)", () => {
  type Form = "sleeping" | "awoken";
  const collisions = (units: ReturnType<typeof mvpPool>["units"], form: Form) => {
    const by = new Map<string, string[]>();
    for (const u of units) by.set(sig(u.forms[form]), [...(by.get(sig(u.forms[form])) ?? []), u.name]);
    return [...by].filter(([, names]) => names.length > 1).map(([s, names]) => `${s}: ${names.join(", ")}`);
  };

  it("sig is When · Who kind · the set of effect kinds, numbers ignored", () => {
    const u = (id: string) => mvpPool().units.find((x) => x.id === id)!;
    expect(sig(u("necromancer").forms.sleeping)).toBe("allyDies · lastDeadAlly · Revive");
    expect(sig(u("divinity").forms.sleeping)).toBe("allyDies · allAllies · Bless");
    expect(sig(u("divinity").forms.awoken)).toBe("allyDies · allAllies · Bless+Shield");
    expect(effectKinds(["Freeze 2 + Curse 1", "Call Imp", "Smite"])).toEqual(["Call", "Curse", "Freeze", "Smite"]);
  });

  it("sig ignores Strength and Vitality riders and counts each family (Heal, Mend; Hit, Smite) as one kind", () => {
    expect(shapeKinds(["Call Golem + Strength 1"])).toEqual(["Call"]);
    expect(shapeKinds(["Shield 1", "Vitality 1", "Strength 2"])).toEqual(["Shield"]);
    expect(shapeKinds(["Mend"])).toEqual(shapeKinds(["Heal 1"]));
    expect(shapeKinds(["Mend", "Heal 2"])).toEqual(["Heal"]);
    expect(shapeKinds(["Smite"])).toEqual(shapeKinds(["Hit 2"]));
    // A form that only grows stats keeps them: that is its job.
    expect(shapeKinds(["Strength 2", "Vitality 1"])).toEqual(["Strength", "Vitality"]);
  });

  it("R1: no two units share a sleeping signature", () => {
    expect(collisions(mvpPool().units, "sleeping")).toEqual([]);
  });

  it("R2: no two units share an Awoken signature", () => {
    expect(collisions(mvpPool().units, "awoken")).toEqual([]);
  });

  it("catches a copied hero: Divinity given Necromancer's ability is a collision", () => {
    const necro = ROWS.find((r) => r.name === "Necromancer")!;
    const rows = ROWS.map((r) => (r.name === "Divinity" ? { ...r, who: necro.who, does: necro.does, awoken: necro.awoken } : r));
    expect(collisions(mvpPool(rows).units, "sleeping")).toEqual(["allyDies · lastDeadAlly · Revive: Necromancer, Divinity"]);
    expect(collisions(mvpPool(rows).units, "awoken")).toEqual(["allyDies · lastDeadAlly · Call+Revive: Necromancer, Divinity"]);
  });

  it("catches a hero that differs only by a rider or a heal word: Sexton + Strength, Medic with Mend", () => {
    const sexton = ROWS.find((r) => r.name === "Sexton")!;
    const medic = ROWS.find((r) => r.name === "Medic")!;
    const rows = ROWS.map((r) =>
      r.name === "Morbid" ? { ...r, when: sexton.when, who: sexton.who, does: "Call Golem + Strength 1" }
      : r.name === "Priest" ? { ...r, when: medic.when, who: medic.who, does: "Mend" }
      : r);
    expect(collisions(mvpPool(rows).units, "sleeping")).toEqual([
      "turnEnd · allAllies · Heal: Medic, Priest",
      "allyDies · holder · Call: Sexton, Morbid",
    ]);
  });

  it("warns (only) when one unit's Awoken shape is another's sleeping shape", () => {
    const sleeping = new Map(mvpPool().units.map((u) => [sig(u.forms.sleeping), u.name]));
    const cross = mvpPool()
      .units.filter((u) => sleeping.has(sig(u.forms.awoken)) && sleeping.get(sig(u.forms.awoken)) !== u.name)
      .map((u) => `${u.name} Awoken = ${sleeping.get(sig(u.forms.awoken))} sleeping (${sig(u.forms.awoken)})`);
    if (cross.length) console.warn(`cross-form shapes:\n  ${cross.join("\n  ")}`);
  });

  // The listen → emit graph over link events; returns each loop found, as
  // "Power →Equalizer (sleeping)→ Curse →Robber (sleeping)→ Power".
  const loopsOf = linkLoops;

  // R3 is relaxed in round 4 (note 8): a numbers-only Awoken is allowed, so
  // awokenNewPart only describes what an Awoken adds; awokenKeeps is the rule.
  it("R3 relaxed: a numbers-only Awoken form ships", () => {
    const u = { ...mvpPool().units.find((x) => x.id === "fighter")! };
    u.forms = { ...u.forms, awoken: { ...u.forms.sleeping, does: ["Hit 2"] } };
    expect(awokenNewPart(u.forms.sleeping, u.forms.awoken)).toBeNull();
    expect(awokenKeeps(u.forms.sleeping, u.forms.awoken)).toBeNull();
    expect(contentFormProblems({ version: "t", ...mvpPool(), units: [u] })).toEqual([]);
  });

  it("R3 counts a new Who or a new effect kind, never a number, a family word or a stat rider", () => {
    const form = (who: string, does: string[]) => ({ when: WHEN.start, who: [{ kind: who }], does }) as Parameters<typeof awokenNewPart>[0];
    expect(awokenNewPart(form("frontEnemy", ["Hit 1"]), form("frontEnemy", ["Hit 3"]))).toBeNull();
    expect(awokenNewPart(form("frontEnemy", ["Hit 1"]), form("frontEnemy", ["Smite"]))).toBeNull();
    expect(awokenNewPart(form("holder", ["Shield 2"]), form("holder", ["Shield 2", "Strength 1"]))).toBeNull();
    expect(awokenNewPart(form("allAllies", ["Strength 1"]), form("allAllies", ["Strength 2", "Vitality 1"]))).toBeNull();
    expect(awokenNewPart(form("holder", ["Call Imp"]), form("holder", ["Call Wolf"]))).toBeNull();
    expect(awokenNewPart(form("frontEnemy", ["Hit 1"]), form("allEnemies", ["Hit 1"]))).toBe("Who frontEnemy → allEnemies");
    expect(awokenNewPart(form("holder", ["Strength 2"]), form("holder", ["Strength 2", "Shield 2"]))).toBe("adds Shield");
  });

  it("R4: the listen → emit graph has no loop", () => {
    expect(loopsOf(mvpPool().units)).toEqual([]);
  });

  it("R4 catches the old Equalizer ↔ Robber loop", () => {
    const rows = ROWS.map((r) => (r.name === "Robber" ? { ...r, does: "Strength 1" } : r));
    // Lightning's Awoken form also curses on Power, and is found first.
    expect(loopsOf(mvpPool(rows).units)).toContain("Curse →Robber (sleeping)→ Power →Lightning (awoken)→ Curse");
    const noLightning = rows.map((r) => (r.name === "Lightning" ? { ...r, awoken: { more: "Hit 3" } } : r));
    expect(loopsOf(mvpPool(noLightning).units)).toContain("Curse →Robber (sleeping)→ Power →Equalizer (sleeping)→ Curse");
  });

  it("R4 catches the old Priest loop: a Blessing's save is a Heal, and Priest blessed healed allies", () => {
    const rows = ROWS.map((r) => (r.name === "Priest" ? { ...r, when: "allyHealed" as WhenKey, who: "it" as const, does: "Bless 1" } : r));
    expect(loopsOf(mvpPool(rows).units)).toContain("Heal →Priest (sleeping)→ Heal");
  });

  it("R4's map covers every When and effect kind, so a new one can't hide a loop", () => {
    for (const k of Object.keys(WHEN) as WhenKey[]) expect([k, k in LISTENS || ROOT_WHENS.includes(k)]).toEqual([k, true]);
    const kinds = new Set(mvpPool().units.flatMap((u) => [...effectKinds(u.forms.sleeping.does), ...effectKinds(u.forms.awoken.does)]));
    for (const k of kinds) expect([k, k in EMITS]).toEqual([k, true]);
  });

  it("\"Ally\" in a trigger always means another ally (otherAlly, never the self-counting ally)", () => {
    const selfCounting: string[] = [];
    for (const u of mvpPool().units) {
      for (const form of ["sleeping", "awoken"] as Form[]) {
        for (const w of u.forms[form].when) {
          if (Object.values(w.on).includes("ally")) selfCounting.push(`${u.name} (${form}): ${JSON.stringify(w.on)}`);
        }
      }
    }
    expect(selfCounting).toEqual([]);
  });
});

describe("Priest's Blessing can't re-arm itself (R3-7 check of 7ee5eec1)", () => {
  const content: MvpContent = { version: "test", ...mvpPool() };
  const contentWith = (rows: typeof ROWS): MvpContent => ({ version: "test", ...mvpPool(rows) });
  // The probe line from the check: Bulwark + Medic + Priest, against hitters.
  const saves = (c: MvpContent, seed: number) => {
    const unit = (id: string) => c.units.find((u) => u.id === id)!;
    const line = ["bulwark", "medic", "priest"].map((id, i) => lineUnitOf(unit(id), `a${i}`));
    const foe = ["ruin", "crusader", "duelist", "sniper", "emberling"].map((id, i) => lineUnitOf(unit(id), `b${i}`));
    const p = { id: "p", name: "p", bot: false };
    const log = fightLines({ player: p, line }, { player: p, line: foe }, { battleId: "b", seed, kind: "round", round: 1, runId: null, at: "2026-10-06T00:00:00.000Z", content: c, rules: MVP_RULES }).log;
    const by = new Map<string, number>();
    for (const e of log) if (e.type === "Intercepted" && e.original === "Death" && e.unit) by.set(e.unit, (by.get(e.unit) ?? 0) + 1);
    return Math.max(0, ...by.values());
  };
  const seeds = Array.from({ length: 20 }, (_, i) => i + 1);

  it("each ally is saved at most once: Priest blesses only at battle start", () => {
    expect(Math.max(...seeds.map((s) => saves(content, s)))).toBeLessThanOrEqual(1);
  });

  it("the old Priest (ally healed: bless it) saved one ally again and again", () => {
    const old = contentWith(ROWS.map((r) => (r.name === "Priest" ? { ...r, when: "allyHealed" as WhenKey, who: "it" as const, does: "Bless 1" } : r)));
    expect(Math.max(...seeds.map((s) => saves(old, s)))).toBeGreaterThan(1);
  });
});

describe("Summoner's Awoken form (R3-8)", () => {
  const content: MvpContent = { version: "test", ...mvpPool() };
  const unit = (id: string) => content.units.find((u) => u.id === id)!;

  it("calls a Warg, a body with a job of its own: its strikes poison the front enemy", () => {
    const line = [lineUnitOf(unit("summoner"), "a0", 3), lineUnitOf(unit("bulwark"), "a1")];
    const foe = ["duelist", "fighter"].map((id, i) => lineUnitOf(unit(id), `b${i}`, 3));
    const p = { id: "p", name: "p", bot: false };
    const log = fightLines({ player: p, line }, { player: p, line: foe }, { battleId: "b", seed: 1, kind: "round", round: 1, runId: null, at: "2026-10-06T00:00:00.000Z", content, rules: MVP_RULES }).log;
    const warg = log.find((e) => e.type === "Summon" && e.name === "Warg");
    if (warg?.type !== "Summon") throw new Error("no Warg summoned");
    expect(log.some((e) => e.type === "StatusApplied" && e.status === "Poison" && typeof e.source === "object" && e.source.unit === warg.unit)).toBe(true);
    expect(content.summons?.find((s) => s.name === "Warg")?.form?.does).toEqual(["Poison 1"]);
  });

  it("R3 counts the summoned body's job: Call Imp → Call Warg adds Poison", () => {
    const f = (does: string[]) => ({ ...unit("summoner").forms.sleeping, does });
    expect(awokenNewPart(f(["Call Imp"]), f(["Call Warg"]))).toBe("adds Poison");
  });
});

describe("Necromancer's Awoken form (R3-26)", () => {
  const content: MvpContent = { version: "test", ...mvpPool() };
  const unit = (id: string) => content.units.find((u) => u.id === id)!;
  const p = { id: "p", name: "p", bot: false };
  const fight = (copies: number) => {
    const line = [lineUnitOf(unit("fodder"), "a0"), lineUnitOf(unit("necromancer"), "a1", copies)];
    const foe = ["duelist", "fighter"].map((id, i) => lineUnitOf(unit(id), `b${i}`, 3));
    return fightLines({ player: p, line }, { player: p, line: foe }, { battleId: "b", seed: 1, kind: "round", round: 1, runId: null, at: "2026-10-06T00:00:00.000Z", content, rules: MVP_RULES }).log;
  };

  it("revives the fallen ally at 1 HP (R4-18) and raises an Imp at the front: something new, not a stat rider", () => {
    const log = fight(3);
    const back = log.find((e) => e.type === "Summon" && e.resurrected);
    if (back?.type !== "Summon") throw new Error("nobody revived");
    expect(back.atHp).toBe(1);
    const imp = log.slice(log.indexOf(back)).find((e) => e.type === "Summon" && e.name === "Imp");
    if (imp?.type !== "Summon") throw new Error("no Imp raised");
    expect(imp.front).toBe(true);
    expect(awokenNewPart(unit("necromancer").forms.sleeping, unit("necromancer").forms.awoken)).toBe("adds Call");
  });

  it("sleeping, it only revives at 1 HP", () => {
    const log = fight(1);
    const back = log.find((e) => e.type === "Summon" && e.resurrected);
    if (back?.type !== "Summon") throw new Error("nobody revived");
    expect(back.atHp).toBe(1);
    expect(log.some((e) => e.type === "Summon" && e.name === "Imp")).toBe(false);
  });
});

describe("no copied heroes (R3-26, Maks's note 1)", () => {
  const pool = mvpPool();
  const ENEMY_EVENTS = new Set(["enemyPoisoned", "enemyCursed", "enemyDies"]);
  const FAMILY: Record<string, string> = { Smite: "Hit", Mend: "Heal" };
  // Who it reaches (an event's unit is a friend or a foe by the When) and
  // every effect kind it has, riders included; the When and numbers ignored.
  // An "and" clause is part of the job, with its own Who.
  const job = (form: (typeof pool.units)[number]["forms"]["awoken"]) => {
    const clause = (c: { who: typeof form.who; does: string[] }) => {
      const who = c.who.map((w) => (w.kind === "eventUnit" ? (ENEMY_EVENTS.has(whenKeyOf(form)) ? "thatEnemy" : "thatAlly") : w.kind)).join("+");
      return `${who} ← ${[...new Set(effectKinds(c.does).map((k) => FAMILY[k] ?? k))].sort().join("+")}`;
    };
    return [form, ...(form.also ?? [])].map(clause).join(", and ");
  };
  const groups = (key: (u: (typeof pool.units)[number]) => string) => {
    const by = new Map<string, string[]>();
    for (const u of pool.units) by.set(key(u), [...(by.get(key(u)) ?? []), u.name]);
    return [...by].filter(([, names]) => names.length > 1);
  };
  /** A form's text without its When ("Hit: 2 damage to front enemy." → "2 damage to front enemy."). */
  const doesText = (form: (typeof pool.units)[number]["forms"]["awoken"]) => formText(form, pool.abilities).replace(/^[^:]*: /, "");

  // The vocabulary has ~40 one-target jobs for 81 units, so some Awoken jobs
  // repeat (docs: r326 content sweep). This count may only go down.
  const SHARED_AWOKEN_JOBS = 42; // R3-26: 52 → 45; R4-14: 42
  it("Awoken forms that share Who + effect kinds with another unit's Awoken (When ignored) don't grow", () => {
    const shared = groups((u) => job(u.forms.awoken));
    const count = shared.reduce((n, [, names]) => n + names.length, 0);
    expect(count, shared.map(([j, names]) => `${j}: ${names.join(", ")}`).join("\n")).toBeLessThanOrEqual(SHARED_AWOKEN_JOBS);
  });

  // Units whose text copies another's in both forms, When aside. Left by
  // R3-26 (the quick meta or the vocabulary blocked a fix); the list may only shrink.
  const BOTH_FORMS_TWINS = [
    "1 Strength to all allies. / 1 Strength to all allies, then heal all allies for 1.: Victim, War Drummer",
    "2 Poison to all enemies. / 2 Poison and 1 Curse to all enemies.: Plague Doctor, Virus",
    "1 Curse to front enemy. / 1 Curse and 1 damage to front enemy.: Wane, Equalizer",
  ];
  it("no unit copies another's text in both forms, When aside (except the listed twins)", () => {
    expect(groups((u) => `${doesText(u.forms.sleeping)} / ${doesText(u.forms.awoken)}`).map(([t, names]) => `${t}: ${names.join(", ")}`)).toEqual(BOTH_FORMS_TWINS);
  });

  // Sleeping forms that read the same once the When is dropped. One effect on
  // one target leaves too few texts for 81 units; these are the ones left, and
  // the list may only shrink (a fixed group must leave it).
  const SLEEPING_TWINS = [
    "1 Shield to all allies. Fodder, Prepper, Commander, Keeper",
    "2 Strength to self. Squire, Henchman, Lilith",
    "1 damage to random enemy. Gnat, Bat",
    "1 Curse to all enemies. Spore, Morbid",
    "heal it for 1. Nurse, Almsgiver",
    "1 Strength to all allies. Coach, Victim, War Drummer",
    // R4-15's approved changes (docs/round4/units.md): Wire numbs like Taser
    // opens, and Pathologist and Icebinder freeze what Poison or Curse marks.
    "1 Freeze to front enemy. Taser, Wire",
    "2 damage to front enemy. Rose, Crusader",
    "1 Curse to front enemy. Saboteur, Physician, Wane, Equalizer",
    "1 Vitality to self. Distractor, Robber",
    "1 Strength to it. Sanctifier, Enhancer",
    "1 Strength to self. Battery, Berserker",
    "2 Poison to all enemies. Plague Rat, Plague Doctor, Virus",
    "2 damage to all enemies. Emberling, Ritualist, Ruin",
    "1 Freeze to it. Icebinder, Pathologist",
    "2 damage to random enemy. Trickster, Lightning, Battle Mage",
    // R4-18: Priest silences the front enemy (was random) to break the
    // ally-dies engine; Silencer does it once at battle start.
    "silence front enemy. Silencer, Priest",
  ];
  it("every unit's sleeping text differs from every other's by more than the When (except the listed twins)", () => {
    expect(groups((u) => doesText(u.forms.sleeping)).map(([t, names]) => `${t} ${names.join(", ")}`)).toEqual(SLEEPING_TWINS);
  });
});

describe("awokenKeeps: Awoken builds on the sleeping form (round 4, note 8)", () => {
  const content: MvpContent = { version: "test", ...mvpPool() };
  const unit = (id: string) => content.units.find((u) => u.id === id)!;
  const form = (who: string, does: string[], also?: { who: string; does: string[] }[]) =>
    ({ when: WHEN.start, who: [{ kind: who }], does, ...(also ? { also: also.map((c) => ({ who: [{ kind: c.who }], does: c.does })) } : {}) }) as Parameters<typeof awokenKeeps>[0];

  // R4-14 fixed the rest, and R4-15 redesigned Commander. None may join this list.
  const KNOWN_FAILURES: string[] = [];

  it("every Awoken form keeps every sleeping part, the same or wider Who, numbers no lower", () => {
    const fails = content.units.filter((u) => awokenKeeps(u.forms.sleeping, u.forms.awoken) !== null).map((u) => u.name);
    expect(fails).toEqual(KNOWN_FAILURES);
  });

  it("keeps: bigger numbers, added Does, a wider Who, a bigger body, an \"and\" clause", () => {
    expect(awokenKeeps(form("frontEnemy", ["Hit 1"]), form("frontEnemy", ["Hit 2"]))).toBeNull();
    expect(awokenKeeps(form("frontEnemy", ["Hit 1"]), form("frontEnemy", ["Hit 1", "Curse 1"]))).toBeNull();
    expect(awokenKeeps(form("frontEnemy", ["Hit 1"]), form("allEnemies", ["Hit 1"]))).toBeNull();
    expect(awokenKeeps(form("holder", ["Strength 1"]), form("allAllies", ["Strength 1"]))).toBeNull();
    expect(awokenKeeps(form("holder", ["Call Imp"]), form("holder", ["Call Treant"]))).toBeNull();
    expect(awokenKeeps(form("lastDeadAlly", ["Revive 2"]), form("lastDeadAlly", ["Revive 2 + Call Imp"]))).toBeNull();
    expect(awokenKeeps(form("holder", ["Strength 2"]), form("holder", ["Strength 2"], [{ who: "frontEnemy", does: ["Silence"] }]))).toBeNull();
  });

  it("catches: a lower number, a dropped Does, a narrower or different Who, a smaller body", () => {
    expect(awokenKeeps(form("allAllies", ["Heal 3"]), form("allAllies", ["Heal 2", "Bless 1"]))).toMatch(/Heal 3/);
    expect(awokenKeeps(form("holder", ["Strength 2"]), form("frontEnemy", ["Silence"]))).toMatch(/Strength 2/);
    expect(awokenKeeps(form("allEnemies", ["Curse 1"]), form("frontEnemy", ["Curse 1"]))).toMatch(/Curse 1/);
    expect(awokenKeeps(form("randomEnemy", ["Hit 2"]), form("eventUnit", ["Hit 2"]))).toMatch(/Hit 2/);
    expect(awokenKeeps(form("holder", ["Call Wolf"]), form("holder", ["Call Imp"]))).toMatch(/Call Wolf/);
    // An "and" clause aimed elsewhere doesn't keep the sleeping part.
    expect(awokenKeeps(form("holder", ["Strength 2"]), form("holder", ["Shield 1"], [{ who: "frontEnemy", does: ["Strength 2"] }]))).toMatch(/Strength 2/);
  });

  it("Henchman's Awoken: ally dies → 2 Strength to me, and Silence the front enemy", () => {
    const h = unit("henchman");
    expect(h.forms.awoken.who).toEqual(h.forms.sleeping.who);
    expect(h.forms.awoken.does).toEqual(["Strength 2"]);
    expect(h.forms.awoken.also).toEqual([{ who: [{ kind: "frontEnemy" }], does: ["Silence"] }]);
    expect(formText(h.forms.awoken, content.abilities)).toBe("Ally dies: 2 Strength to self, and silence front enemy.");
  });

  it("an Awoken Henchman does both in battle: it grows, and the front enemy is silenced", () => {
    const p = { id: "p", name: "p", bot: false };
    const line = [lineUnitOf(unit("fodder"), "a0"), lineUnitOf(unit("henchman"), "a1", 3)];
    const foe = ["duelist", "bulwark"].map((id, i) => lineUnitOf(unit(id), `b${i}`, 3));
    const log = fightLines({ player: p, line }, { player: p, line: foe }, { battleId: "b", seed: 1, kind: "round", round: 1, runId: null, at: "2026-10-07T00:00:00.000Z", content, rules: MVP_RULES }).log;
    const death = log.find((e) => e.type === "Death" && e.unit.includes("Fodder"));
    if (!death) throw new Error("Fodder never died");
    const after = log.filter((e) => e.causedBy === death.id || (e.id > death.id && typeof e.source !== "string" && e.source.unit.includes("Henchman")));
    const grew = after.find((e) => e.type === "StatusApplied" && e.status === "Strength" && e.unit.includes("Henchman"));
    const silenced = after.find((e) => e.type === "Silenced" && e.unit.startsWith("B"));
    expect(grew, "Henchman gains Strength").toBeDefined();
    expect(silenced, "the front enemy is silenced").toBeDefined();
    // Both come from the one reaction: Strength first, then the "and" clause.
    expect(grew!.id).toBeLessThan(silenced!.id);
  });

  it("a fusion keeps each part's \"and\" clause with its own Who", () => {
    const h = lineUnitOf(unit("henchman"), "a0", 3);
    const f = lineUnitOf(unit("fighter"), "a1", 3);
    const fused = fuseUnits(f, h, { name: "X", discoveredBy: null }, content);
    expect(fused.recipe.who).toEqual(h.recipe.who);
    expect(fused.recipe.does).toEqual(["Hit 1", "Strength 2"]);
    expect(fused.recipe.also).toEqual([{ who: [{ kind: "frontEnemy" }], does: ["Silence"] }]);
    const back = fuseUnits(h, f, { name: "Y", discoveredBy: null }, content);
    expect(back.recipe.who).toEqual(f.recipe.who);
    expect(back.recipe.also).toEqual(h.recipe.also);
  });
});
