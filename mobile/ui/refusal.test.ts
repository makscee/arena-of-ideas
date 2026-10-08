// The server's refusals read as plain words (R2-17).
import { expect, test } from "vitest";
import { buttonRefusal, plainRefusal } from "./refusal";

test("each refusal the run engine gives reads as plain words", () => {
  expect(plainRefusal("invalid buy: the line is full")).toBe("Line full: sell or fuse first");
  expect(plainRefusal("invalid buy: costs 3, have 2")).toBe("Not enough gold: it costs 3g, you have 2g");
  expect(plainRefusal("invalid buy: only the Crown fight is left")).toMatch(/^The Crown: your line is final/);
  expect(plainRefusal("invalid buy: the run is over (out-of-hearts)")).toBe("This run is over");
  expect(plainRefusal("invalid buy: the run's content is no longer live: start a new run")).toMatch(/^The game changed/);
  expect(plainRefusal("invalid buy: no offer in slot 4")).toBe("That offer is gone: the shop changed");
  expect(plainRefusal("invalid fuse: both units must be Awoken")).toBe("Both units must be Awoken");
  // The gift's own words (R3-16): "make room", never "sell or fuse" (fuse is refused while it waits).
  expect(plainRefusal("invalid gift: your line and bench are full: sell a unit to make room, or skip the gift")).toBe("Line and bench full: sell a unit to make room, or skip the gift");
  expect(plainRefusal("invalid fuse: pick your awakening gift first, or skip it")).toBe("Pick your gift first, or skip it");
});

test("a disabled Buy says a full line in one short line", () => {
  expect(buttonRefusal(plainRefusal("invalid buy: the line is full"))).toBe("Full: sell or fuse");
  expect(buttonRefusal("Needs 3g")).toBe("Needs 3g");
  expect(buttonRefusal(plainRefusal("invalid gift: your line and bench are full: sell a unit to make room, or skip the gift"))).toBe("Full: make room");
});
