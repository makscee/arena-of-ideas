// What each game event sounds like (round 3, note 16).
import { expect, test } from "vitest";
import type { BattleEvent } from "../../src/types";
import type { LineUnit, RunView } from "../../src/mvp/contract";
import type { PlayBeat, Step } from "../../src/mvp/trace";
import { BEAT_CAP, GAIN, SOUND_KEYS, beatCues, endSound, shopSound, waveSound } from "./sound-map";
import { readdirSync } from "node:fs";

const unit = (uid: string, copies: number, form: "sleeping" | "awoken" = "sleeping"): LineUnit =>
  ({ uid, kind: "unit", unitId: uid, name: uid, emoji: "?", copies, form, stats: { pwr: 1, hp: 1 } }) as unknown as LineUnit;
const runOf = (...line: LineUnit[]): RunView => ({ line }) as unknown as RunView;

test("every key has a file and a gain, and every file a key", () => {
  const files = readdirSync(new URL("../public/sfx/", import.meta.url)).filter((f) => f.endsWith(".mp3")).map((f) => f.replace(/\.mp3$/, ""));
  expect(files.sort()).toEqual([...SOUND_KEYS].sort());
  for (const k of SOUND_KEYS) expect(GAIN[k]).toBeGreaterThan(0);
});

test("a buy clinks, a copy merges, the third copy levels up", () => {
  const buy = { kind: "buy", slot: 0 } as const;
  expect(shopSound(buy, runOf(), runOf(unit("a", 1)))?.key).toBe("coin");
  expect(shopSound(buy, runOf(unit("a", 1)), runOf(unit("a", 2)))?.key).toBe("merge");
  expect(shopSound(buy, runOf(unit("b", 1), unit("a", 2)), runOf(unit("b", 1), unit("a", 3, "awoken")))?.key).toBe("level-up");
  // A 4th copy into an Awoken unit is a merge again.
  expect(shopSound(buy, runOf(unit("a", 3, "awoken")), runOf(unit("a", 4, "awoken")))?.key).toBe("merge");
});

test("sell, reroll, reorder left and right, fuse", () => {
  const r = runOf(unit("a", 1), unit("b", 1));
  expect(shopSound({ kind: "sell", index: 0 }, r, runOf(unit("b", 1)))?.key).toBe("sell");
  expect(shopSound({ kind: "reroll" }, r, r)?.key).toBe("reroll");
  expect(shopSound({ kind: "reorder", from: 1, to: 0 }, r, r)?.key).toBe("move-left");
  expect(shopSound({ kind: "reorder", from: 0, to: 1 }, r, r)?.key).toBe("move-right");
  expect(shopSound({ kind: "fuse", first: 0, second: 1 }, r, r)?.key).toBe("fuse");
  expect(shopSound({ kind: "fight" }, r, r)).toBeNull();
});

// A hand-made log: ids are indices.
function logOf(...bodies: Partial<BattleEvent>[]): BattleEvent[] {
  return bodies.map((b, id) => ({ id, turn: 1, causedBy: null, source: "kernel", ...b }) as BattleEvent);
}
const stepOf = (...eventIds: number[]): Step => ({ eventIds, turn: 1, actor: null, actorSide: null, subject: null, subjectSide: null, caption: "", changes: [] });
const ability = { unit: "a1", ability: "X" } as unknown as BattleEvent["source"];

test("a Strike's Hurt hits, an ability's zaps, a fully blocked one blocks", () => {
  const log = logOf(
    { type: "Strike", striker: "a1", defender: "b1" },
    { type: "Hurt", unit: "b1", amount: 3, causedBy: 0 },
    { type: "Hurt", unit: "b1", amount: 2, causedBy: 1, source: ability },
    { type: "Hurt", unit: "b1", amount: 0, absorbed: 2, causedBy: 0 },
    { type: "Death", unit: "b1", causedBy: 1 },
  );
  expect(waveSound(stepOf(1), log, 0)?.key).toBe("hit");
  expect(waveSound(stepOf(2), log, 0)?.key).toBe("zap");
  expect(waveSound(stepOf(3), log, 0)?.key).toBe("block");
  // Death beats Hurt, and is the low thud.
  expect(waveSound(stepOf(1, 4), log, 0)).toMatchObject({ key: "debuff", rate: 0.7 });
});

test("Shield applied, another status, heal at 1.2, stats, removed, capped", () => {
  const log = logOf(
    { type: "StatusApplied", unit: "a1", status: "Shield", stacks: 2, total: 2 },
    { type: "StatusApplied", unit: "a1", status: "Poison", stacks: 1, total: 1 },
    { type: "Heal", unit: "a1", amount: 2 },
    { type: "StatChanged", unit: "a1", stat: "pwr", delta: 1, now: 3 },
    { type: "StatChanged", unit: "a1", stat: "pwr", delta: -1, now: 2 },
    { type: "StatusRemoved", unit: "a1", status: "Poison", stacks: 1, remaining: 0 },
    { type: "ChainCapped", root: 0, steps: 32 },
    { type: "Summon", unit: "a9", name: "Imp", side: "A", hp: 1, pwr: 1, resurrected: true },
  );
  expect(waveSound(stepOf(0), log, 0)?.key).toBe("shield");
  expect(waveSound(stepOf(1), log, 0)?.key).toBe("status");
  expect(waveSound(stepOf(2), log, 0)).toMatchObject({ key: "buff", rate: 1.2 });
  expect(waveSound(stepOf(3), log, 0)).toMatchObject({ key: "buff", rate: 1 });
  expect(waveSound(stepOf(4), log, 0)?.key).toBe("debuff");
  expect(waveSound(stepOf(5), log, 0)?.key).toBe("status-off");
  expect(waveSound(stepOf(1, 5), log, 0)?.key).toBe("status");
  expect(waveSound(stepOf(6), log, 0)?.key).toBe("wrong");
  expect(waveSound(stepOf(7), log, 3)).toMatchObject({ key: "spawn", rate: 1.15 });
});

test("a chain rises in pitch, up to 1.3", () => {
  const log = logOf({ type: "Heal", unit: "a1", amount: 1 }, { type: "StatChanged", unit: "a1", stat: "hp", delta: 1, now: 3 });
  expect([0, 1, 2, 6, 9].map((i) => waveSound(stepOf(1), log, i)?.rate)).toEqual([1, 1.05, 1.1, 1.3, 1.3]);
});

test("a 12-wave beat makes at most 5 sounds, plus every death", () => {
  const bodies: Partial<BattleEvent>[] = [];
  for (let i = 0; i < 12; i++) bodies.push(i % 4 === 3 ? { type: "Death", unit: `b${i}` } : { type: "StatChanged", unit: "a1", stat: "pwr", delta: 1, now: 2 + i });
  const log = logOf(...bodies);
  const beat: PlayBeat = { index: 0, turn: 1, waves: log.map((e) => stepOf(e.id)), end: 11, start: 0 };
  const cues = beatCues(beat, log);
  expect(cues).toHaveLength(12);
  const played = cues.filter((c) => c !== null);
  const deaths = played.filter((c) => c!.rate === 0.7);
  expect(deaths).toHaveLength(3);
  expect(played.length - deaths.length).toBe(BEAT_CAP);
});

test("the end: win, lose, draw; a slay adds the chime half a second later", () => {
  expect(endSound("win", "round").map((c) => c.key)).toEqual(["win"]);
  expect(endSound("loss", "round").map((c) => c.key)).toEqual(["lose"]);
  expect(endSound("draw", "crown").map((c) => c.key)).toEqual(["draw"]);
  expect(endSound("win", "crown")).toMatchObject([{ key: "win" }, { key: "discover", delay: 500 }]);
  expect(endSound("loss", "playoff", false).map((c) => c.key)).toEqual(["win"]);
});
