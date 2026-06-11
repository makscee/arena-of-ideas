// Layout-stability invariants for the board markup (audit LS-1): the grave
// row is part of a side's shape from event 0 — present before any death and
// after a full wipe — so deaths never change the board's height mid-replay.

import { describe, expect, test } from "vitest";
import { battle, boardAt, type UnitDef } from "../src/index.js";
import { boardHtml } from "./board-render.js";

const dummy = (name: string, hp = 10, pwr = 3): UnitDef => ({ name, base: { hp, pwr } });
const id = (s: string): string => s;

describe("boardHtml", () => {
  // 1v1 with a lopsided matchup: side A's Frail dies, so the last event's
  // board has a populated grave and a wiped line.
  const log = battle({ teamA: [dummy("Frail")], teamB: [dummy("Bruiser", 30, 9)], seed: 0 });

  test("grave rows render on both sides before any death", () => {
    const html = boardHtml(boardAt(log, 0), id, new Set(), {});
    expect(html.match(/class="grave"/g)).toHaveLength(2);
    expect(html).not.toContain("no one standing");
  });

  test("a wiped side keeps both its line and grave rows", () => {
    const html = boardHtml(boardAt(log, log.length - 1), id, new Set(), {});
    expect(html).toContain("no one standing");
    expect(html.match(/class="grave"/g)).toHaveLength(2);
  });
});
