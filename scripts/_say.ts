import { contentOf } from "../server/src/mvp/content.js";
import { sentenceText } from "../src/mvp/form-text.js";
import { ROWS_V2 } from "../src/mvp/units-v2.js";
import { slug } from "../src/mvp/units.js";
const c = contentOf(ROWS_V2);
for (const n of process.argv.slice(2)) {
  const u = c.units.find((x) => x.id === slug(n))!;
  console.log(`${n} T${u.tier}: ${sentenceText(u.forms.sleeping, c.abilities, "en", c.statuses)} || AW: ${sentenceText(u.forms.awoken, c.abilities, "en", c.statuses)}`);
}
