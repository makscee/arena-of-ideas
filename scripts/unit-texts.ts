// Prints every MVP unit's text, both forms, and every status's text: the
// wording players read, in one place (`npm run -s mvp:texts`).

import { describeStatus } from "../src/describe.js";
import { formText } from "../src/mvp/form-text.js";
import { mvpPool } from "../src/mvp/units.js";

const pool = mvpPool();
for (const u of pool.units) {
  console.log(`${u.emoji} ${u.name}`);
  console.log(`  sleeping: ${formText(u.forms.sleeping, pool.abilities)}`);
  console.log(`  awoken:   ${formText(u.forms.awoken, pool.abilities)}`);
}
console.log("\nStatuses");
for (const [name, def] of Object.entries(pool.statuses)) console.log(`  ${name}: ${describeStatus(def)}`);
