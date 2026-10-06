import { describe, expect, it } from "vitest";
import { MVP_RULES, type MvpContent } from "./contract.js";
import { fightLines } from "./fight.js";
import { contentFormProblems, lineUnitOf } from "./forms.js";
import { EMITS, LISTENS, ROOT_WHENS, ROWS, WHEN, effectKinds, linkEdges, mvpPool, sig, type WhenKey } from "./units.js";

describe("MVP pool (slice 7)", () => {
  const pool = mvpPool();
  const content = { version: "test", ...pool };

  it("has about 80 units, every one shippable in both forms", () => {
    expect(pool.units.length).toBeGreaterThanOrEqual(75);
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
      const forms = [
        { who: row.who, does: [row.does] },
        { who: row.awoken.who ?? row.who, does: row.awoken.does ?? [row.does] },
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
  const fight = (ids: string[]) => {
    const line = ids.map((id, i) => lineUnitOf(unit(id), `a${i}`));
    const foe = [lineUnitOf(unit("fodder"), "b0")];
    const p = { id: "p", name: "p", bot: false };
    return fightLines({ player: p, line }, { player: p, line: foe }, { battleId: "b", seed: 1, kind: "round", round: 1, runId: null, at: "2026-10-06T00:00:00.000Z", content, rules: MVP_RULES }).log;
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
    const log = fight(["planter", "fighter", "squire", "gnat", "rose"]);
    expect(summoned(log)).toBe(false);
    expect(grew(log)).toBe(true);
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
    expect(collisions(mvpPool(rows).units, "awoken")).toEqual(["allyDies · lastDeadAlly · Revive: Necromancer, Divinity"]);
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
  const loopsOf = (units: ReturnType<typeof mvpPool>["units"]) => {
    const edges = new Map<string, Map<string, string>>();
    for (const u of units) {
      for (const form of ["sleeping", "awoken"] as Form[]) {
        for (const [from, to] of linkEdges(u.forms[form])) {
          const out = edges.get(from) ?? new Map<string, string>();
          if (!out.has(to)) out.set(to, `${u.name} (${form})`);
          edges.set(from, out);
        }
      }
    }
    const loops = new Set<string>();
    const walk = (node: string, path: { node: string; via: string }[]) => {
      const at = path.findIndex((p) => p.node === node);
      if (at >= 0) {
        const loop = path.slice(at);
        loops.add([...loop.map((p) => `${p.node} →${p.via}→ `), node].join(""));
        return;
      }
      for (const [to, via] of edges.get(node) ?? []) walk(to, [...path, { node, via }]);
    };
    for (const n of edges.keys()) walk(n, []);
    return [...loops];
  };

  it("R4: the listen → emit graph has no loop", () => {
    expect(loopsOf(mvpPool().units)).toEqual([]);
  });

  it("R4 catches the old Equalizer ↔ Robber loop", () => {
    const rows = ROWS.map((r) => (r.name === "Robber" ? { ...r, does: "Strength 1" } : r));
    expect(loopsOf(mvpPool(rows).units)).toContain("Curse →Robber (sleeping)→ Power →Equalizer (sleeping)→ Curse");
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
