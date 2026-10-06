// The "See Awoken" underline over the tagged pieces (R2-8).
import { expect, test } from "vitest";
import { mvpPool } from "../../src/mvp/units";
import { formSegments } from "../../src/mvp/form-text";
import { changedTokens } from "./diff";

const pool = mvpPool();
const abilities = pool.abilities;

test("each token stays in its piece, and the pieces still read as the sentence", () => {
  for (const u of pool.units) {
    const was = formSegments(u.forms.sleeping, abilities);
    const next = formSegments(u.forms.awoken, abilities);
    const toks = changedTokens(was, next);
    const byPiece = next.map((_, i) => toks.filter((t) => t.piece === i).map((t) => t.tok).join(""));
    expect(byPiece, u.name).toEqual(next.map((s) => s.text));
  }
});

test("only the words the awoken form adds are marked", () => {
  const seg = (text: string, term?: "effect:damage") => (term ? { text, term, amount: true as const } : { text });
  const was = [seg("deal "), seg("2", "effect:damage"), seg(" damage to the front enemy.")];
  const next = [seg("deal "), seg("4", "effect:damage"), seg(" damage to every enemy.")];
  const marked = changedTokens(was, next).filter((t) => t.changed).map((t) => [t.tok, t.piece]);
  expect(marked).toEqual([["4", 1], ["every", 2]]);
});
