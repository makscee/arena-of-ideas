// The rules v2 fixtures (#1045, src/words/fixtures.ts) as test scenes: each
// fixture battle (the same battle its golden pins) is stored as a battle
// record under a stable id, `fixture-<name>`, so `#scene=v2-<name>` opens it on
// the lab server. The fixtures are not in the pool: the scenes server merges
// their abilities, statuses and unit sheets into the content it serves
// (withSceneFixtures), on a scenes server only, so the live game is untouched.

import { battle, winnerOf } from "../../../src/battle.js";
import type { BattleRecord, BattleUnit, MvpContent, PlayerRef, UnitContent, UnitForm } from "../../../src/mvp/contract.js";
import type { BattleEvent, Side, UnitDef } from "../../../src/types.js";
import { FIXTURE_ABILITIES, FIXTURE_STATUSES, FIXTURES } from "../../../src/words/fixtures.js";

const EMOJI: Record<string, string> = {
  Hourglass: "⏳", Mirror: "🪞", Candle: "🕯️", Gambler: "🎲", Nautilus: "🐚", Magnet: "🧲", Raven: "🐦‍⬛", Seed: "🌱",
  "Déjà Vu": "🔁", "Grave Tree": "🪦", Archer: "🏹", Hedgehog: "🦔", Brute: "🪨",
};

/** `v2-deja-vu`: the fixture unit's id in the served content. */
export const fixtureUnitId = (name: string): string =>
  `v2-${name.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase().replace(/[^a-z0-9]+/g, "-")}`;

const formOf = (d: UnitDef): UnitForm => ({
  when: d.triggers ?? [],
  who: d.selectors ?? [],
  does: d.abilities ?? [],
  ...(d.condition ? { condition: d.condition } : {}),
  ...(d.also?.length ? { also: d.also.map((c) => ({ who: c.selectors, does: c.abilities })) } : {}),
});

const battleUnit = (d: UnitDef, id: string): BattleUnit => ({
  uid: id, id, kind: "unit", unitId: fixtureUnitId(d.name), name: d.name, emoji: EMOJI[d.name] ?? "🧩",
  copies: 1, form: "sleeping", stats: { ...d.base }, recipe: formOf(d),
});

function team(defs: UnitDef[], side: Side, log: BattleEvent[]): BattleUnit[] {
  const start = log.find((e) => e.type === "BattleStart");
  const roster = start?.type === "BattleStart" ? start.teams[side] : [];
  return defs.map((d, i) => battleUnit(d, roster[i]?.id ?? `${side}${i + 1}:${d.name}`));
}

export const FIXTURE_SCENE_NAMES = Object.keys(FIXTURES);

/** The fixture battle `name` (as fixtures.test.ts fights it) as a stored record. */
export function fixtureBattle(name: string, player: PlayerRef, contentVersion: string, at: string): BattleRecord {
  const fx = FIXTURES[name];
  if (!fx) throw new Error(`no fixture ${name}`);
  const log = battle(fx.input);
  return {
    battleId: `fixture-${name}`,
    runId: null,
    player,
    seed: fx.input.seed,
    contentVersion,
    kind: "round",
    round: 1,
    at,
    teamA: team(fx.input.teamA, "A", log),
    teamB: team(fx.input.teamB, "B", log),
    opponent: { id: "bot", name: "bot-Fixture", bot: true },
    winner: winnerOf(log),
    log,
  };
}

/** Every unit a fixture battle fields, as a sheet the client can open. */
export function fixtureUnits(): UnitContent[] {
  const seen = new Map<string, UnitContent>();
  for (const { input } of Object.values(FIXTURES)) {
    for (const d of [...input.teamA, ...input.teamB]) {
      if (seen.has(d.name)) continue;
      const form = formOf(d);
      seen.set(d.name, { id: fixtureUnitId(d.name), name: d.name, emoji: EMOJI[d.name] ?? "🧩", archetype: `Rules v2 fixture (#1043 §4–5): ${d.name}`, tier: 1, base: { ...d.base }, forms: { sleeping: form, awoken: form } });
    }
  }
  return [...seen.values()];
}

/** The served content plus the fixtures' words; the pool's own entries win. */
export function withSceneFixtures(content: MvpContent): MvpContent {
  const have = new Set([...content.units, ...(content.left ?? [])].map((u) => u.id));
  return {
    ...content,
    left: [...(content.left ?? []), ...fixtureUnits().filter((u) => !have.has(u.id))],
    abilities: { ...FIXTURE_ABILITIES, ...content.abilities },
    statuses: { ...FIXTURE_STATUSES, ...content.statuses },
  };
}
