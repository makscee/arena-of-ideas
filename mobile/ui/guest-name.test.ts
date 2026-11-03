import { describe, expect, it } from "vitest";
import { guestName } from "./guest-name";

describe("guestName", () => {
  it("is word, word and a number the server's name rule takes", () => {
    for (let i = 0; i < 200; i++) expect(guestName()).toMatch(/^[\p{L}\p{N}_\- ]{1,24}$/u);
    expect(guestName(() => 0)).toBe("Amber Badger 1");
  });
  it("takes more digits when asked", () => {
    expect(guestName(() => 99998, 5)).toMatch(/^\w+ \w+ 99999$/);
  });
});
