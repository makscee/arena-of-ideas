import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync(new URL("./style.css", import.meta.url), "utf8");

describe("style.css", () => {
  // A merge once dropped a phone @media's closing brace: every rule after it
  // (the piece cards, the XL detail) then applied only on a portrait phone
  // (mission #1286, void-board#1295).
  it("closes every block it opens", () => {
    const code = css.replace(/\/\*[\s\S]*?\*\//g, (c) => c.replace(/[^\n]/g, ""));
    const open: number[] = [];
    code.split("\n").forEach((line, i) => {
      for (const ch of line) {
        if (ch === "{") open.push(i + 1);
        else if (ch === "}") expect(open.pop(), `a "}" with nothing open, line ${i + 1}`).toBeDefined();
      }
    });
    expect(open, "blocks left open, by line").toEqual([]);
  });
});
