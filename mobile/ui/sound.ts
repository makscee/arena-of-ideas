// The sound engine (round 3, note 16; docs/round3/sounds.md): Web Audio,
// unlocked by the first tap or key, the 26 files preloaded then, one master
// gain. What plays when is ui/sound-map.ts. No music.
import { SOUND_KEYS, cue, type Cue, type SoundKey } from "./sound-map";

export interface SoundSettings {
  on: boolean;
  /** 0..1; the master gain is its square, so the slider feels even. */
  volume: number;
}

export const SOUND_STORE = "arena.sound";
export const SOUND_DEFAULTS: SoundSettings = { on: true, volume: 0.6 };
/** The same sound never starts twice within this many ms. */
export const THROTTLE_MS = 70;
const LOG_MAX = 50;

/** The saved settings, or the defaults when storage is missing, throws, or
 * holds something else. `store` is injectable for tests. */
export function loadSettings(store: () => Pick<Storage, "getItem"> = () => localStorage): SoundSettings {
  try {
    const raw = store().getItem(SOUND_STORE);
    if (!raw) return { ...SOUND_DEFAULTS };
    const s = JSON.parse(raw) as Partial<SoundSettings>;
    const volume = typeof s.volume === "number" && s.volume >= 0 && s.volume <= 1 ? s.volume : SOUND_DEFAULTS.volume;
    return { on: typeof s.on === "boolean" ? s.on : SOUND_DEFAULTS.on, volume };
  } catch {
    return { ...SOUND_DEFAULTS };
  }
}

export function saveSettings(s: SoundSettings, store: () => Pick<Storage, "setItem"> = () => localStorage): void {
  try {
    store().setItem(SOUND_STORE, JSON.stringify(s));
  } catch {
    // a private window: the choice lasts this visit
  }
}

let settings: SoundSettings = SOUND_DEFAULTS;
let ctx: AudioContext | null = null;
let master: GainNode | null = null;
const buffers = new Map<SoundKey, AudioBuffer>();
const lastStart = new Map<SoundKey, number>();
const listeners = new Set<(s: SoundSettings) => void>();

declare global {
  interface Window {
    /** The last 50 sounds asked for, "key" or "key (muted)": e2e reads it. */
    __sfx?: string[];
  }
}

const masterGain = () => (settings.on ? settings.volume * settings.volume : 0);

function unlock(): void {
  const AC = window.AudioContext ?? (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!AC) return;
  if (!ctx) {
    try {
      ctx = new AC();
    } catch {
      return;
    }
    master = ctx.createGain();
    master.gain.value = masterGain();
    master.connect(ctx.destination);
    const base = `${import.meta.env.BASE_URL}sfx/`;
    const audio = ctx;
    for (const key of SOUND_KEYS) {
      void fetch(`${base}${key}.mp3`)
        .then((r) => (r.ok ? r.arrayBuffer() : Promise.reject(new Error(String(r.status)))))
        .then((b) => audio.decodeAudioData(b))
        .then((buf) => void buffers.set(key, buf))
        .catch(() => {
          // a missing or broken file: that sound stays silent
        });
    }
  }
  if (settings.on && !document.hidden) void ctx.resume().catch(() => {});
}

let started = false;
/** Reads the settings and arms the first-gesture unlock; call once at start. */
export function initSound(): void {
  if (started) return;
  started = true;
  settings = loadSettings();
  window.__sfx = window.__sfx ?? [];
  const first = () => {
    removeEventListener("pointerdown", first, true);
    removeEventListener("keydown", first, true);
    unlock();
  };
  addEventListener("pointerdown", first, true);
  addEventListener("keydown", first, true);
  document.addEventListener("visibilitychange", () => {
    if (!ctx) return;
    if (document.hidden) void ctx.suspend().catch(() => {});
    else if (settings.on) void ctx.resume().catch(() => {});
  });
}

export const soundSettings = (): SoundSettings => ({ ...settings });

/** Changes the settings, saves them, and tells every open Sound row. */
export function setSound(next: Partial<SoundSettings>): void {
  settings = { ...settings, ...next };
  saveSettings(settings);
  if (master && ctx) master.gain.setValueAtTime(masterGain(), ctx.currentTime);
  if (ctx && settings.on && !document.hidden) void ctx.resume().catch(() => {});
  for (const fn of listeners) fn(soundSettings());
}

export const toggleSound = (): void => setSound({ on: !settings.on });

/** Calls fn on every settings change until the returned function is called. */
export function onSoundChange(fn: (s: SoundSettings) => void): () => void {
  listeners.add(fn);
  return () => void listeners.delete(fn);
}

/** Plays a cue now (or after its delay). Never throws and never queues: a
 * sound not loaded yet, muted or in a hidden tab just doesn't play. */
export function play(c: Cue | SoundKey | null | undefined): void {
  if (!c) return;
  const q = typeof c === "string" ? cue(c) : c;
  if (q.delay) return void setTimeout(() => play({ ...q, delay: 0 }), q.delay);
  const log = (window.__sfx ??= []);
  log.push(settings.on ? q.key : `${q.key} (muted)`);
  if (log.length > LOG_MAX) log.splice(0, log.length - LOG_MAX);
  if (!settings.on || document.hidden || !ctx || !master) return;
  const buf = buffers.get(q.key);
  if (!buf) return;
  const now = performance.now();
  if (now - (lastStart.get(q.key) ?? -Infinity) < THROTTLE_MS) return;
  lastStart.set(q.key, now);
  try {
    const src = ctx.createBufferSource();
    src.buffer = buf;
    src.playbackRate.value = q.rate;
    const g = ctx.createGain();
    g.gain.value = q.gain;
    src.connect(g).connect(master);
    src.start();
  } catch {
    // a context closed under us: stay silent
  }
}
