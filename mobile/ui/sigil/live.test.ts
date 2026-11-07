import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { FALLBACK_SPEC } from "./fallback";
import { mountSigil, mountedCount, resetSigils } from "./live";

function fakeCanvas(connected = true) {
  const draws: number[] = [];
  const ctx = new Proxy({}, { get: (_t, k: string) => (k === "createLinearGradient" || k === "createRadialGradient" ? () => ({ addColorStop() {} }) : k === "clearRect" ? () => { draws.push(1); } : () => {}), set: () => true });
  return { cv: { width: 0, height: 0, style: {} as Record<string, string>, isConnected: connected, getContext: () => ctx } as unknown as HTMLCanvasElement, draws };
}

describe("the live loop", () => {
  let cbs: Map<number, () => void>;
  let id: number;
  beforeEach(() => {
    cbs = new Map(); id = 0;
    vi.stubGlobal("requestAnimationFrame", (f: () => void) => { cbs.set(++id, f); return id; });
    vi.stubGlobal("cancelAnimationFrame", (n: number) => { cbs.delete(n); });
    vi.stubGlobal("devicePixelRatio", 3);
  });
  afterEach(() => { resetSigils(); vi.unstubAllGlobals(); });
  const tick = () => { const [[k, f]] = [...cbs] as [[number, () => void]]; cbs.delete(k); f(); };

  test("one requestAnimationFrame serves every mounted canvas", () => {
    const a = fakeCanvas(), b = fakeCanvas(), c = fakeCanvas();
    mountSigil(a.cv, { spec: FALLBACK_SPEC, S: 44 });
    mountSigil(b.cv, { spec: FALLBACK_SPEC, S: 44 });
    mountSigil(c.cv, { spec: FALLBACK_SPEC, S: 44 });
    expect(cbs.size).toBe(1);
    tick();
    expect([a.draws.length, b.draws.length, c.draws.length]).toEqual([1, 1, 1]);
    expect(cbs.size).toBe(1); // the next frame is queued once
  });
  test("a canvas is sized for the device, capped at 2x", () => {
    const a = fakeCanvas();
    mountSigil(a.cv, { spec: FALLBACK_SPEC, S: 44 });
    expect(a.cv.width).toBe(88);
    expect(a.cv.style.width).toBe("44px");
  });
  test("the loop stops with the last canvas and starts again", () => {
    const a = fakeCanvas(), b = fakeCanvas();
    const ha = mountSigil(a.cv, { spec: FALLBACK_SPEC, S: 44 });
    const hb = mountSigil(b.cv, { spec: FALLBACK_SPEC, S: 44 });
    ha.unmount();
    tick();
    expect(a.draws.length).toBe(0);
    expect(b.draws.length).toBe(1);
    hb.unmount();
    tick();
    expect(cbs.size).toBe(0);
    expect(mountedCount()).toBe(0);
    mountSigil(a.cv, { spec: FALLBACK_SPEC, S: 44 });
    expect(cbs.size).toBe(1);
  });
  test("a canvas that left the document is dropped", () => {
    const a = fakeCanvas();
    mountSigil(a.cv, { spec: FALLBACK_SPEC, S: 44 });
    tick();
    expect(a.draws.length).toBe(1);
    (a.cv as unknown as { isConnected: boolean }).isConnected = false;
    tick();
    expect(a.draws.length).toBe(1);
    expect(mountedCount()).toBe(0);
    expect(cbs.size).toBe(0);
  });
  test("a canvas not attached yet waits, is drawn once attached, and is dropped if it never is", () => {
    const a = fakeCanvas(false), b = fakeCanvas(false);
    let now = 1000;
    vi.spyOn(performance, "now").mockImplementation(() => now);
    mountSigil(a.cv, { spec: FALLBACK_SPEC, S: 44 });
    mountSigil(b.cv, { spec: FALLBACK_SPEC, S: 44 });
    tick();
    expect(mountedCount()).toBe(2);
    expect(a.draws.length).toBe(0);
    (a.cv as unknown as { isConnected: boolean }).isConnected = true;
    now += 1000;
    tick();
    expect(a.draws.length).toBe(1);
    now += 1500;
    tick();
    expect(mountedCount()).toBe(1); // b never came
    vi.restoreAllMocks();
  });
  test("a pulse falls (hit) or grows (die) over its time, and a running one is picked up by its age", () => {
    const flash: number[] = [], alpha: number[] = [];
    const ctx = new Proxy({}, {
      get: (_t, k: string) => (k === "createLinearGradient" || k === "createRadialGradient" ? () => ({ addColorStop() {} }) : () => {}),
      set: (_t, k: string, v: unknown) => {
        const m = k === "fillStyle" && typeof v === "string" ? /^rgba\(255,255,255,([\d.]+)\)$/.exec(v) : null;
        if (m) flash.push(Number(m[1]) / 0.55);
        if (k === "globalAlpha" && typeof v === "number" && Number.isFinite(v)) alpha.push(v);
        return true;
      },
    });
    const cv = { width: 0, height: 0, style: {}, isConnected: true, getContext: () => ctx } as unknown as HTMLCanvasElement;
    let now = 5000;
    vi.spyOn(performance, "now").mockImplementation(() => now);
    const h = mountSigil(cv, { spec: FALLBACK_SPEC, S: 44 });
    h.pulse("hit", 400, 100); // began 100 ms ago: 0.75 left
    tick();
    expect(flash.at(-1)).toBeCloseTo(0.75, 2);
    now += 200;
    tick();
    expect(flash.at(-1)).toBeCloseTo(0.25, 2);
    now += 300;
    flash.length = 0;
    tick();
    expect(flash).toEqual([]); // over: no flash drawn
    h.pulse("die", 400, -200); // starts in 200 ms
    alpha.length = 0;
    tick();
    expect(alpha).toEqual([]); // not started: the portrait is whole
    now += 290; // 90 ms into the die: halfway to the peak at 180 ms
    alpha.length = 0;
    tick();
    expect(Math.min(...alpha)).toBeCloseTo(0.5, 2);
    now += 90; // the peak: fully shattered
    alpha.length = 0;
    tick();
    expect(Math.min(...alpha)).toBeCloseTo(0, 2);
    now += 1000; // over: the portrait is back whole, for the ghost the card greys
    alpha.length = 0;
    tick();
    expect(alpha).toEqual([]);
    vi.restoreAllMocks();
  });
  test("update changes the state and resizes", () => {
    const a = fakeCanvas();
    const h = mountSigil(a.cv, { spec: FALLBACK_SPEC, S: 44 });
    h.update({ S: 60, hit: 1 });
    expect(a.cv.style.width).toBe("60px");
    expect(a.cv.width).toBe(120);
    tick();
    expect(a.draws.length).toBe(1);
  });
});
