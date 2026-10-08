// The fuse preview's plain warning (R3-26). A fused unit sends both parts'
// Does to the second part's Who, so a harmful effect can land on your own
// units, or a helpful one on the enemy. Such fusions stay (Maks, round 2:
// "Fusions that hurt their own team stay"); the preview only says so.
import type { MvpContent, UnitForm } from "../../src/mvp/contract";
import type { Effect, Selector, StatusRegistry, When } from "../../src/types";

type Side = "ally" | "enemy" | null;

/** Whose units a Who reaches: yours, theirs, or either (null). The event's
 * unit is whoever the When names (an ally's hit: an ally). */
export function sideOf(sel: Selector, when: When[]): Side {
  switch (sel.kind) {
    case "holder":
    case "allAllies":
    case "lastDeadAlly":
      return "ally";
    case "attacker":
    case "frontEnemy":
    case "allEnemies":
    case "randomEnemy":
      return "enemy";
    case "eventUnit": {
      const sides = new Set(
        when.map((w) => {
          const on = w.on as { striker?: string; unit?: string };
          const f = on.striker ?? on.unit;
          return f === "holder" || f === "ally" || f === "otherAlly" ? "ally" : f === "enemy" ? "enemy" : null;
        }),
      );
      return sides.size === 1 ? [...sides][0]! : null;
    }
  }
}

/** +1 helps whoever gets it, −1 harms them, 0 neither or unclear. A status
 * reads by what it does: stats up or down, Shield and Blessing save, Poison
 * hurts its holder, Freeze cancels its holder's strike. */
export function effectSign(e: Effect, statuses: StatusRegistry): 1 | -1 | 0 {
  switch (e.kind) {
    case "damage":
    case "silence":
      return -1;
    case "heal":
    case "resurrect":
    case "summon":
      return 1;
    case "applyStatus": {
      const st = statuses[e.status];
      if (!st) return 0;
      const mods = Object.values(st.statMods ?? {}).reduce((a, b) => a + (b ?? 0), 0);
      if (mods !== 0) return mods > 0 ? 1 : -1;
      const kinds = st.abilities.flatMap((a) => a.effects.map((x) => x.kind));
      if (kinds.includes("absorbHurt") || kinds.includes("preventDeathHeal") || kinds.includes("heal")) return 1;
      if (kinds.includes("damage") || kinds.includes("cancel")) return -1;
      return 0;
    }
    default:
      return 0;
  }
}

/** What an effect is called in the warning. */
function effectName(e: Effect): string {
  if (e.kind === "applyStatus") return e.status;
  if (e.kind === "resurrect") return "revive";
  if (e.kind === "summon") return "a summon";
  if (e.kind === "silence") return "Silence";
  return e.kind;
}

type Misfire = { hurts: string[]; helps: string[] };

/** The effects a form sends the wrong way: harmful ones to your side, helpful
 * ones to theirs. */
export function misfires(form: UnitForm, content: Pick<MvpContent, "abilities" | "statuses">): Misfire {
  const out: Misfire = { hurts: [], helps: [] };
  // The form's own Does on its Who, then each "and" clause on its own Who.
  for (const clause of [{ who: form.who, does: form.does }, ...(form.also ?? [])]) {
    const sides = new Set(clause.who.map((s) => sideOf(s, form.when)));
    for (const id of clause.does) {
      for (const e of content.abilities[id]?.effects ?? []) {
        const sign = effectSign(e, content.statuses);
        const name = effectName(e);
        if (sign < 0 && sides.has("ally") && !out.hurts.includes(name)) out.hurts.push(name);
        if (sign > 0 && sides.has("enemy") && !out.helps.includes(name)) out.helps.push(name);
      }
    }
  }
  return out;
}

const list = (xs: string[]) => (xs.length < 2 ? xs.join("") : `${xs.slice(0, -1).join(", ")} and ${xs[xs.length - 1]}`);

/** The fuse preview's warning, or null: only what the fusion adds, not what
 * a part already did on its own (a unit built to hurt itself). */
export function fuseWarning(fused: UnitForm, parts: UnitForm[], content: Pick<MvpContent, "abilities" | "statuses">): string | null {
  const own = parts.map((p) => misfires(p, content));
  const m = misfires(fused, content);
  const hurts = m.hurts.filter((n) => !own.some((o) => o.hurts.includes(n)));
  const helps = m.helps.filter((n) => !own.some((o) => o.helps.includes(n)));
  const says: string[] = [];
  if (hurts.length) says.push(`hurts your own units (its ${list(hurts)} ${hurts.length === 1 ? "lands" : "land"} on them)`);
  if (helps.length) says.push(`helps the enemy (its ${list(helps)} ${helps.length === 1 ? "goes" : "go"} to them)`);
  return says.length ? `⚠ This fusion ${says.join(" and ")}.` : null;
}
