// Every MVP unit's text, both forms, and every status's text: the wording
// players read, in one place. `npm run -s mvp:texts` prints it, and
// unit-texts.golden.txt pins it.

import { describeStatus, type Lang } from "../describe.js";
import { ruStatusName } from "../describe-ru.js";
import { formText } from "./form-text.js";
import type { MvpPool } from "./units.js";

/** `lang` "ru" writes the rules in Russian (M4-3, unit-texts.ru.golden.txt);
 * names and archetype lines stay as written until M4-4. */
export function unitTexts(pool: MvpPool, lang?: Lang): string {
  const out: string[] = [];
  for (const u of pool.units) {
    out.push(`${u.emoji} ${u.name}`);
    out.push(`  archetype: ${u.archetype}`);
    out.push(`  sleeping: ${formText(u.forms.sleeping, pool.abilities, lang)}`);
    out.push(`  awoken:   ${formText(u.forms.awoken, pool.abilities, lang)}`);
  }
  out.push("", "Statuses");
  for (const [name, def] of Object.entries(pool.statuses)) out.push(`  ${lang === "ru" ? ruStatusName(name) : name}: ${describeStatus(def, lang)}`);
  return out.join("\n") + "\n";
}
