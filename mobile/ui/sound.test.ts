// Sound settings survive a storage that throws (round 3, note 16).
import { expect, test } from "vitest";
import { existsSync } from "node:fs";
import { HOME_TRACK, SOUND_DEFAULTS, SOUND_STORE, THEMES, loadSettings, onAdFinished, onAdStarted, play, saveSettings, sceneTrack, setPortalMute, themeFor } from "./sound";

const broken = () => {
  throw new Error("SecurityError");
};

test("no storage, or one that throws: on at 60%, music on at 40%, no crash", () => {
  expect(loadSettings(broken)).toEqual({ on: true, volume: 0.6, music: true, musicVolume: 0.4 });
  expect(SOUND_DEFAULTS).toEqual({ on: true, volume: 0.6, music: true, musicVolume: 0.4 });
  expect(() => saveSettings({ on: false, volume: 0.2, music: false, musicVolume: 0.1 }, broken)).not.toThrow();
  expect(loadSettings(() => ({ getItem: () => "{not json" }))).toEqual(SOUND_DEFAULTS);
  expect(loadSettings(() => ({ getItem: () => null }))).toEqual(SOUND_DEFAULTS);
});

test("saved settings come back; a bad volume falls back", () => {
  const box = new Map<string, string>();
  const store = () => ({ getItem: (k: string) => box.get(k) ?? null, setItem: (k: string, v: string) => void box.set(k, v) });
  saveSettings({ on: false, volume: 0.25, music: false, musicVolume: 0.7 }, store);
  expect(box.has(SOUND_STORE)).toBe(true);
  expect(loadSettings(store)).toEqual({ on: false, volume: 0.25, music: false, musicVolume: 0.7 });
  box.set(SOUND_STORE, JSON.stringify({ on: true, volume: 7, musicVolume: -1 }));
  expect(loadSettings(store)).toEqual({ on: true, volume: 0.6, music: true, musicVolume: 0.4 });
});

test("round 3's saved settings gain music on at 40%", () => {
  const store = () => ({ getItem: () => JSON.stringify({ on: true, volume: 0.3 }) });
  expect(loadSettings(store)).toEqual({ on: true, volume: 0.3, music: true, musicVolume: 0.4 });
});

test("home plays a run theme (one music family, void-board#922); a run keeps one theme, muffled in the shop and open in battle", () => {
  expect(sceneTrack("home")).toEqual({ track: HOME_TRACK, muffled: false });
  expect(THEMES.map((t) => t.file)).toContain(HOME_TRACK.file);
  const shop = sceneTrack("shop", "run-1");
  const battle = sceneTrack("battle", "run-1");
  expect(shop.track).toBe(battle.track);
  expect(THEMES).toContain(shop.track);
  expect([shop.muffled, battle.muffled]).toEqual([true, false]);
  // The runs spread over all four themes.
  const used = new Set(Array.from({ length: 200 }, (_, i) => themeFor(`r${i}`).file));
  expect(used.size).toBe(THEMES.length);
});

test("tracks.json: four themes, each with a measured beat and a loop inside its file", () => {
  expect(THEMES).toHaveLength(4);
  for (const t of [HOME_TRACK, ...THEMES]) {
    expect(t.file).toMatch(/\.m4a$/);
    expect(t.bpm).toBeGreaterThan(60);
    expect(t.bpm).toBeLessThan(180);
    expect(t.offset).toBeGreaterThanOrEqual(0);
    expect(t.offset).toBeLessThan(60 / t.bpm);
    expect(t.loopEnd).toBeGreaterThan(t.loopStart);
    // Levelled (docs/mission5/sound/level.py): within ±3 dB of the others.
    expect(t.gain).toBeGreaterThan(0.7);
    expect(t.gain).toBeLessThan(1.42);
    expect(existsSync(new URL(`../public/music/${t.file}`, import.meta.url))).toBe(true);
  }
});

test("the portal's muteAudio and an ad break silence the game, and give it back (#1219)", () => {
  const g = globalThis as unknown as { window?: { __sfx?: string[] } | undefined; document?: { hidden: boolean } | undefined };
  const had = { window: g.window, document: g.document };
  g.window = {};
  g.document = { hidden: false };
  try {
    const heard = () => (g.window!.__sfx ?? []).splice(0);
    play("click");
    expect(heard()).toEqual(["click"]);
    setPortalMute(true);
    play("click");
    expect(heard()).toEqual(["click (muted)"]);
    // An ad ending can't unmute what the portal still mutes.
    onAdStarted();
    onAdFinished();
    play("click");
    expect(heard()).toEqual(["click (muted)"]);
    setPortalMute(false);
    play("click");
    expect(heard()).toEqual(["click"]);
    onAdStarted();
    play("click");
    expect(heard()).toEqual(["click (muted)"]);
    onAdFinished();
    play("click");
    expect(heard()).toEqual(["click"]);
  } finally {
    g.window = had.window;
    g.document = had.document;
  }
});
