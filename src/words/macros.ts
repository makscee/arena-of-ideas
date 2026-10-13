// Composed words (#1043 §7): macros over existing primitives. "echo" is
// remember + delay + perform. The compiler (expandAbilities, run by the
// engine on its input and by the validator) replaces `{kind: "macro", name}`
// with the macro's effects, so the kernel only ever sees primitives. A macro
// has its own sentence in both languages; its trace is its primitives' lines.
// Coining a new composed word is one entry here; a forged word (a new
// primitive) is a file in this folder instead.

import type { Ability, AbilityRegistry, Effect, StatusRegistry } from "../types.js";

export interface Macro {
  name: string;
  effects: Effect[];
  /** The clause in a card sentence; `{target}` is the rule's Who. */
  en: string;
  ru: string;
}

export const MACROS: Record<string, Macro> = {
  echo: {
    name: "echo",
    effects: [{ kind: "remember" }, { kind: "delay", turns: 1, effect: { kind: "performRemembered", mode: "once" } }],
    en: "echoes {target} a turn later",
    ru: "повторяет {target} ходом позже",
  },
};

export const macro = (name: string): Macro | undefined => MACROS[name];

/** The effects with every macro replaced by its primitives, nested macros included. */
export function expandEffects(effects: Effect[], seen: string[] = []): Effect[] {
  if (!effects.some((e) => e.kind === "macro")) return effects;
  return effects.flatMap((e) => {
    if (e.kind !== "macro") return [e];
    const m = MACROS[e.name];
    if (!m) throw new Error(`unknown macro "${e.name}"`);
    if (seen.includes(e.name)) throw new Error(`macro "${e.name}" expands to itself`);
    return expandEffects(m.effects, [...seen, e.name]);
  });
}

const expandAbility = <A extends Ability>(a: A): A => {
  const effects = expandEffects(a.effects);
  return effects === a.effects ? a : { ...a, effects };
};

/** A registry with its macros expanded; untouched (the same object) when it has none. */
export function expandAbilities(abilities: AbilityRegistry): AbilityRegistry {
  let out: AbilityRegistry | undefined;
  for (const [id, a] of Object.entries(abilities)) {
    const x = expandAbility(a);
    if (x !== a) (out ??= { ...abilities })[id] = x;
  }
  return out ?? abilities;
}

export function expandStatuses(statuses: StatusRegistry): StatusRegistry {
  let out: StatusRegistry | undefined;
  for (const [id, s] of Object.entries(statuses)) {
    const abilities = s.abilities.map(expandAbility);
    if (abilities.some((a, i) => a !== s.abilities[i])) (out ??= { ...statuses })[id] = { ...s, abilities };
  }
  return out ?? statuses;
}
