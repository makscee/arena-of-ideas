// R4-2 (#685), as Maks meets it: Sexton + Necromancer fused ("ally dies →
// Call Ghoul, Revive + Call Imp") on a full line. One slot frees, the Ghoul
// fills it, and the revive and the Imp find no room. Both now show as
// "No room" in the Log, the caption and the Why trace, with their cause.
import { describe, expect, it } from "vitest";
import type { BattleEvent } from "../types.js";
import { MVP_RULES, type LineUnit, type MvpContent, type PlayerRef, type UnitContent } from "./contract.js";
import { fightLines } from "./fight.js";
import { fuseUnits, lineUnitOf } from "./forms.js";
import { beatPlayOf, captionOf, chainOf, NO_ROOM, stepsOf } from "./trace.js";
import { mvpPool } from "./units.js";
import { displayNames } from "../trace.js";
import { termDef } from "../glossary.js";

const pool = mvpPool();
const content: MvpContent = { version: "no-room", ...pool };
const p: PlayerRef = { id: "p", name: "p", bot: false };
const unit = (id: string): UnitContent => {
  const u = pool.units.find((x) => x.id === id);
  if (!u) throw new Error(`no unit ${id} in the pool`);
  return u;
};

function fight(): BattleEvent[] {
  const fused = fuseUnits(lineUnitOf(unit("sexton"), "a0", 3), lineUnitOf(unit("necromancer"), "a9", 3), { name: "Mortwrought", discoveredBy: null }, content, MVP_RULES);
  const fillers = ["squire", "squire", "squire", "squire"].map((id, i) => lineUnitOf(unit(id), `f${i}`, 1));
  // Mortwrought at the back, so the front fillers die and it answers.
  const a: LineUnit[] = [...fillers, fused];
  const b: LineUnit[] = [lineUnitOf(unit("squire"), "b0", 3)].map((u) => ({ ...u, stats: { pwr: 6, hp: 80 } }));
  return fightLines({ player: p, line: a }, { player: p, line: b }, { battleId: "x", seed: 1, kind: "round", round: 9, runId: null, at: "2026-10-07T00:00:00.000Z", content, rules: MVP_RULES }).log;
}

describe("No room in the viewer (R4-2)", () => {
  const log = fight();
  const name = displayNames(log);
  const noRoom = log.filter((e): e is Extract<BattleEvent, { type: "NoRoom" }> => e.type === "NoRoom");

  it("the fused Sexton + Necromancer finds no room for the Imp and the revive", () => {
    expect(noRoom.some((e) => e.name === "Imp" && e.revive === undefined)).toBe(true);
    expect(noRoom.some((e) => e.revive !== undefined)).toBe(true);
  });

  it("captions say No room, who tried and for whom", () => {
    const imp = noRoom.find((e) => e.name === "Imp")!;
    expect(captionOf(log, imp.id, name)).toBe(`Mortwrought → ${NO_ROOM} for Imp`);
    const rev = noRoom.find((e) => e.revive !== undefined)!;
    expect(captionOf(log, rev.id, name)).toBe(`Mortwrought → ${NO_ROOM} to revive ${name(rev.revive!)}`);
  });

  it("the Log has a row for each, with a change to tap for its trace", () => {
    const rows = beatPlayOf(log, stepsOf(log)).flatMap((b) => b.waves);
    const rowsOf = rows.filter((w) => w.caption.includes(NO_ROOM));
    expect(rowsOf.length).toBeGreaterThanOrEqual(noRoom.length > 1 ? 2 : 1);
    for (const w of rowsOf) expect(w.changes[0]).toMatchObject({ kind: "noRoom", label: "no room" });
  });

  it("the Why trace starts at No room and walks back to the death it answered", () => {
    const imp = noRoom.find((e) => e.name === "Imp")!;
    const chain = chainOf(log, imp.id);
    expect(chain.change).toMatchObject({ kind: "noRoom", unit: imp.unit });
    expect(chain.nodes[0]).toMatchObject({ kind: "change", event: "NoRoom" });
    expect(chain.nodes.some((n) => n.kind === "firing" && n.unit === imp.unit && n.trigger === "trigger:Death")).toBe(true);
    expect(chain.nodes.some((n) => log[n.eventId]?.type === "Death")).toBe(true);
  });

  it("the glossary explains it", () => {
    expect(termDef("battle:noRoom")).toMatchObject({ label: NO_ROOM, icon: "magic-portal" });
  });
});
