// A unit card's icon line (round 3, R3-4, docs/round3/words.md (8)): what the
// form does, in icons, in its text's order: the When, then the Who, then each
// Does. Display-only, built from formSegments' glossary terms, so fusions and
// summons get it free. Pure data; the client draws it (mobile/ui/card.ts).

import { termDef, termIcon, type IconId, type TermId } from "../glossary.js";
import type { AbilityRegistry, UnitFilter } from "../types.js";
import type { UnitForm } from "./contract.js";
import { formSegments } from "./form-text.js";

/** Whose event a trigger is: an ally's (teal pip), an enemy's (pink), either
 * side's (dim). Unset: the unit's own, or a turn or battle trigger. */
export type Pip = "ally" | "enemy" | "any";

export interface CardIcon {
  icon: IconId;
  /** "when", "who" or "does": which part of the sentence it stands for. */
  role: "when" | "who" | "does";
  /** The tone class suffix (`tone-<tone>`): amber When, a Who by side, a Does in its own colour. */
  tone: string;
  /** What it names in a tooltip: "Battle start", "Front enemy", "Freeze". */
  label: string;
  pip?: Pip;
}

/** A trigger scope's pip. The holder's own event has none. */
export function scopePip(scope: UnitFilter | undefined): Pip | undefined {
  if (scope === "ally" || scope === "otherAlly") return "ally";
  if (scope === "enemy") return "enemy";
  if (scope === "any") return "any";
  return undefined;
}

const SCOPE_WORD: Record<Pip, string> = { ally: "Ally", enemy: "Enemy", any: "Any unit" };

/** A trigger's label said of its scope: "Dies", "Ally dies", "Enemy is hit". */
export function scopedLabel(label: string, pip: Pip | undefined): string {
  return pip ? `${SCOPE_WORD[pip]} ${label.charAt(0).toLowerCase()}${label.slice(1)}` : label;
}

/** The form's icons, When first, then Who, then each Does (each part in its
 * text's order; the text says "deal 2 damage to the front enemy", the line
 * reads When · Who · Does). One per idea: a repeated icon is dropped. An
 * authored text line is ignored: the icons come from the form's parts. */
export function cardIcons(form: UnitForm, abilities: AbilityRegistry): CardIcon[] {
  const { text: _authored, ...parts } = form;
  const segs = formSegments(parts, abilities);
  const out: CardIcon[] = [];
  const seen = new Set<IconId>();
  const add = (c: CardIcon | null) => {
    if (!c || seen.has(c.icon)) return;
    seen.add(c.icon);
    out.push(c);
  };
  for (let i = 0; i < segs.length; i++) {
    const seg = segs[i]!;
    if (seg.clause === "when") {
      // One icon per trigger clause: the status it names, else the trigger's own.
      const clause = [seg];
      while (segs[i + 1]?.clause === "when") clause.push(segs[++i]!);
      const trig = clause.find((s) => s.term?.startsWith("trigger:"));
      if (!trig?.term) continue;
      const status = clause.map((s) => (s.term?.startsWith("status:") ? s.term.slice("status:".length) : undefined)).find(Boolean);
      const ic = termIcon(trig.term, status);
      const pip = scopePip(trig.scope);
      // A status trigger reads as the status: "Gets Shield", "Ally loses Poison".
      const base = status ? `${trig.term === "trigger:StatusRemoved" ? "Loses" : "Gets"} ${status}` : (termDef(trig.term)?.label ?? trig.term);
      if (ic) add({ icon: ic, role: "when", tone: "when", label: scopedLabel(base, pip), ...(pip ? { pip } : {}) });
      continue;
    }
    const term = seg.term;
    if (!term) continue;
    add(termCardIcon(term, seg));
  }
  const rank = { when: 0, who: 1, does: 2 } as const;
  return out.sort((a, b) => rank[a.role] - rank[b.role]);
}

/** A Who or Does run's icon: a target in its side's colour, an effect or a
 * status in its own. Stats, conditions and plain words get none. */
function termCardIcon(term: TermId, seg: { text: string; side?: string }): CardIcon | null {
  const def = termDef(term);
  const ic = termIcon(term);
  if (!def || !ic) return null;
  if (term.startsWith("target:")) {
    const label = term === "target:eventUnit" ? seg.text.charAt(0).toUpperCase() + seg.text.slice(1) : def.label;
    return { icon: ic, role: "who", tone: seg.side ?? def.tone, label };
  }
  if (term.startsWith("effect:") || term.startsWith("status:")) return { icon: ic, role: "does", tone: def.tone, label: def.label };
  return null;
}
