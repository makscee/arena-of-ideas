// Sound settings survive a storage that throws (round 3, note 16).
import { expect, test } from "vitest";
import { SOUND_DEFAULTS, SOUND_STORE, loadSettings, saveSettings } from "./sound";

const broken = () => {
  throw new Error("SecurityError");
};

test("no storage, or one that throws: on at 60%, no crash", () => {
  expect(loadSettings(broken)).toEqual({ on: true, volume: 0.6 });
  expect(SOUND_DEFAULTS).toEqual({ on: true, volume: 0.6 });
  expect(() => saveSettings({ on: false, volume: 0.2 }, broken)).not.toThrow();
  expect(loadSettings(() => ({ getItem: () => "{not json" }))).toEqual(SOUND_DEFAULTS);
  expect(loadSettings(() => ({ getItem: () => null }))).toEqual(SOUND_DEFAULTS);
});

test("saved settings come back; a bad volume falls back", () => {
  const box = new Map<string, string>();
  const store = () => ({ getItem: (k: string) => box.get(k) ?? null, setItem: (k: string, v: string) => void box.set(k, v) });
  saveSettings({ on: false, volume: 0.25 }, store);
  expect(box.has(SOUND_STORE)).toBe(true);
  expect(loadSettings(store)).toEqual({ on: false, volume: 0.25 });
  box.set(SOUND_STORE, JSON.stringify({ on: true, volume: 7 }));
  expect(loadSettings(store)).toEqual({ on: true, volume: 0.6 });
});
