// The only colours a sigil spec may name. "owner" is the side's colour (yours
// teal, theirs rose), so one spec reads for either side. Lookups go through
// Maps: a name like "constructor" or "__proto__" must never find anything.
export type Team = "you" | "them";

const NAMED: ReadonlyMap<string, string> = new Map(
  Object.entries({
    ink: "#120e1c", night: "#2b2340", paper: "#fbf7f0", bone: "#eadcbf", gold: "#f0c24b",
    ember: "#ef8a3c", blood: "#d0566a", frost: "#a6e3f5", venom: "#9bd35a", arcane: "#b9a6e0",
  }),
);

const OWNER: Readonly<Record<Team, ReadonlyMap<string, string>>> = {
  you: new Map(Object.entries({ owner: "#3fb9a0", "owner.dark": "#1f6f63", "owner.light": "#9fe6d4" })),
  them: new Map(Object.entries({ owner: "#e07a86", "owner.dark": "#8e3448", "owner.light": "#f6c1c8" })),
};

export const INK = NAMED.get("ink") as string;
export const GOLD = NAMED.get("gold") as string;
export const PAPER = NAMED.get("paper") as string;

/** Every colour name a spec may use. */
export const PALETTE_NAMES: readonly string[] = [...NAMED.keys(), ...OWNER.you.keys()];

export function isPaletteName(c: unknown): boolean {
  return typeof c === "string" && (NAMED.has(c) || OWNER.you.has(c));
}

/** The CSS colour for a palette name on a side, or null when it is not in the palette. */
export function color(c: unknown, team: Team): string | null {
  if (typeof c !== "string") return null;
  return OWNER[team].get(c) ?? NAMED.get(c) ?? null;
}

export function ownerColor(team: Team): string {
  return OWNER[team].get("owner") as string;
}
