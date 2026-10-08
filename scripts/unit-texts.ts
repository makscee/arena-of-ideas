// Prints every MVP unit's text, both forms, and every status's text: the
// wording players read, in one place (`npm run -s mvp:texts`).

import { unitTexts } from "../src/mvp/unit-texts.js";
import { mvpPool } from "../src/mvp/units.js";

process.stdout.write(unitTexts(mvpPool()));
