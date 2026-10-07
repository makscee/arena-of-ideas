// The beat clock follows the music, smooths its coarse steps and runs free
// without it (round 4, note 5).
import { expect, test } from "vitest";
import { JUMP_S, STALL_MS, bobPhase, makeBeatClock, type BeatSource } from "./beat";

/** A fake world: a performance.now() and a music source tests move. */
function world(src: BeatSource) {
  const w = { t: 0, src };
  const clock = makeBeatClock(() => w.src, () => w.t);
  return { w, clock };
}

test("free-running at the track's BPM when no music plays", () => {
  const { w, clock } = world({ bpm: 100, offset: 0.3, time: null });
  w.t = 0;
  expect(clock()).toMatchObject({ bpm: 100, beatMs: 600, index: 0, phase: 0 });
  w.t = 1500; // 2.5 beats at 100 BPM
  const b = clock();
  expect(b.index).toBe(2);
  expect(b.phase).toBeCloseTo(0.5);
  expect(b.nextBeatIn()).toBeCloseTo(300);
  expect(b.origin).toBeCloseTo(0);
  w.src = { bpm: 90, offset: 0, time: null };
  expect(clock().beatMs).toBeCloseTo(666.667, 2);
});

test("with music, the beat is the music's position minus the offset", () => {
  const { w, clock } = world({ bpm: 120, offset: 0.25, time: 1.5 }); // (1.5 − 0.25)·2 = 2.5 beats
  w.t = 10_000;
  const b = clock();
  expect(b.index).toBe(2);
  expect(b.phase).toBeCloseTo(0.5);
  expect(b.nextBeatIn()).toBeCloseTo(250);
  // beat 0 landed 2.5 beats (1250 ms) before now
  expect(b.origin).toBeCloseTo(10_000 - 1250);
});

test("coarse media steps are smoothed with performance.now()", () => {
  const { w, clock } = world({ bpm: 120, offset: 0, time: 0 });
  clock();
  // 100 ms later the element still reports 0 (it steps every ~250 ms):
  // the clock keeps moving with real time instead of standing still.
  w.t = 100;
  expect(clock().phase).toBeCloseTo(0.2, 6);
  w.t = 150;
  expect(clock().phase).toBeCloseTo(0.3, 6);
  // a new reading off by a little is pulled in by 10%, not taken whole
  w.t = 200;
  w.src = { ...w.src, time: 0.24 };
  expect(clock().phase).toBeCloseTo((0.2 + (0.24 - 0.2) * 0.1) * 2, 6);
  // the element stalls (buffering): past STALL_MS the clock stops with it
  w.t = 200 + STALL_MS + 1;
  expect(clock().phase).toBeCloseTo(0.48, 6);
});

test("a steady track keeps the origin still; a seek or loop jump moves it at once", () => {
  const { w, clock } = world({ bpm: 100, offset: 0, time: 5 });
  w.t = 1000;
  const o1 = clock().origin;
  for (let i = 1; i <= 10; i++) {
    w.t = 1000 + i * 250;
    w.src = { ...w.src, time: 5 + i * 0.25 };
    expect(clock().origin).toBeCloseTo(o1, 6);
  }
  // the element jumps back (a loop, a seek) by more than JUMP_S
  w.t += 16;
  w.src = { ...w.src, time: 1 };
  const b = clock();
  expect(JUMP_S).toBeLessThan(1.5);
  expect(b.index).toBe(Math.floor((1 * 100) / 60));
  expect(b.origin).toBeCloseTo(w.t - 1000, 6);
});

test("music stops: the clock runs free; a track change re-anchors", () => {
  const { w, clock } = world({ bpm: 100, offset: 0, time: 3 });
  w.t = 5000;
  clock();
  w.src = { bpm: 100, offset: 0, time: null };
  expect(clock().origin).toBeCloseTo(0);
  w.src = { bpm: 90, offset: 0.441, time: 0.441 };
  w.t = 7000;
  const b = clock();
  expect(b).toMatchObject({ bpm: 90, index: 0 });
  expect(b.phase).toBeCloseTo(0);
  expect(b.nextBeatIn()).toBeCloseTo(60000 / 90);
});

test("bobPhase: a two-beat cycle, odd slots a beat behind, never negative", () => {
  const b = { beatMs: 600, origin: 1000 };
  expect(bobPhase(b, 1000, false)).toBe(0);
  expect(bobPhase(b, 1000, true)).toBe(600);
  expect(bobPhase(b, 1900, false)).toBe(900);
  expect(bobPhase(b, 2300, false)).toBe(100);
  expect(bobPhase(b, 400, false)).toBe(600);
});
