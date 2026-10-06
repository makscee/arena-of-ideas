// Every MVP unit's text, both forms, and every status's text: the wording
// players read, in one place. `npm run -s mvp:texts` prints it, and
// unit-texts.golden.txt pins it.

import { describeStatus } from "../describe.js";
import { formText } from "./form-text.js";
import type { MvpPool } from "./units.js";

export function unitTexts(pool: MvpPool): string {
  const out: string[] = [];
  for (const u of pool.units) {
    out.push(`${u.emoji} ${u.name}`);
    out.push(`  sleeping: ${formText(u.forms.sleeping, pool.abilities)}`);
    out.push(`  awoken:   ${formText(u.forms.awoken, pool.abilities)}`);
  }
  out.push("", "Statuses");
  for (const [name, def] of Object.entries(pool.statuses)) out.push(`  ${name}: ${describeStatus(def)}`);
  return out.join("\n") + "\n";
}
