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
    const a = fakeCanvas(false);
    mountSigil(a.cv, { spec: FALLBACK_SPEC, S: 44 });
    tick();
    expect(a.draws.length).toBe(0);
    expect(mountedCount()).toBe(0);
    expect(cbs.size).toBe(0);
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
