// The icon sprite holds one symbol per SVG in mobile/icons/ (R2-6).
import { expect, test } from "vitest";
import { ICON_IDS } from "../../src/glossary";
import { spriteMarkup } from "./icon";

test("the sprite has a symbol for every glossary icon, paths kept", () => {
  const sprite = spriteMarkup();
  const ids = [...sprite.matchAll(/<symbol id="i-([^"]+)" viewBox="0 0 512 512"><path fill="currentColor" d="/g)].map((m) => m[1]);
  expect(ids.sort()).toEqual([...ICON_IDS].sort());
  expect(sprite).not.toMatch(/<symbol[^>]*><svg/);
});
