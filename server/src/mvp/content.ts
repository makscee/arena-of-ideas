// MVP content (mission #574). Slice 1 maps the v5 pool (DEFAULT_RUN_POOL plus
// the approved registry) onto the two-form shape, with the awoken form a copy
// of the sleeping one; slice 7 replaces this with ~80 authored units.
import { createHash } from "node:crypto";
import type { MvpContent, Tier, UnitContent } from "../../../src/mvp/contract.js";
import type { UnitDef } from "../../../src/index.js";
import { defaultArenaContent } from "../content.js";

const EMOJI: Record<string, string> = {
  Venomancer: "🐍", Summoner: "🔮", Silencer: "🤫", Necromancer: "💀", Brawler: "🥊", Bulwark: "🛡️", Squire: "🗡️",
  Berserker: "🪓", "War Drummer": "🥁", Duelist: "🤺", Emberling: "🔥", Icebinder: "🧊", Leech: "🩸", Medic: "⛑️",
  Miasma: "☁️", Phoenix: "🐦‍🔥", "Plague Rat": "🐀", Sniper: "🎯", Stoneskin: "🪨", "Field Surgeon": "🩺", "Bog Witch": "🧙",
};

function slug(name: string): string {
  return name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
}

function toUnitContent(def: UnitDef, i: number): UnitContent {
  const form = {
    when: def.triggers ?? [],
    who: def.selectors ?? [],
    does: def.abilities ?? (def.ability ? [def.ability] : []),
    ...(def.condition ? { condition: def.condition } : {}),
  };
  return {
    id: slug(def.name),
    name: def.name,
    emoji: EMOJI[def.name] ?? "❔",
    tier: ((i % 4) + 1) as Tier,
    base: { ...def.base },
    forms: { sleeping: form, awoken: structuredClone(form) },
  };
}

export function mvpContent(): MvpContent {
  const base = defaultArenaContent();
  const units = base.pool.map(toUnitContent);
  const body = { units, abilities: base.abilities, statuses: base.statuses };
  const version = "mvp-" + createHash("sha256").update(JSON.stringify(body)).digest("hex").slice(0, 10);
  return { version, ...body };
}
