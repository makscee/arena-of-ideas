// The sound engine (round 3, note 16; docs/round3/sounds.md): Web Audio,
// unlocked by the first tap or key, the 26 files preloaded then, one master
// gain. What plays when is ui/sound-map.ts. Music (round 4, note 5;
// docs/round4/music.md) streams through the same context: home plays the
// loop, a run one theme, muffled in the shop and open in battle.
import { SOUND_KEYS, cue, type Cue, type SoundKey } from "./sound-map";
import TRACKS from "./tracks.json";

export interface SoundSettings {
  on: boolean;
  /** 0..1; the master gain is its square, so the slider feels even. */
  volume: number;
  /** The music channel, under the master: on by default at 40%. */
  music: boolean;
  /** 0..1, squared like the master. */
  musicVolume: number;
}

export const SOUND_STORE = "arena.sound";
export const SOUND_DEFAULTS: SoundSettings = { on: true, volume: 0.6, music: true, musicVolume: 0.4 };
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
    const unit = (v: unknown, d: number) => (typeof v === "number" && v >= 0 && v <= 1 ? v : d);
    const flag = (v: unknown, d: boolean) => (typeof v === "boolean" ? v : d);
    return {
      on: flag(s.on, SOUND_DEFAULTS.on),
      volume: unit(s.volume, SOUND_DEFAULTS.volume),
      music: flag(s.music, SOUND_DEFAULTS.music),
      musicVolume: unit(s.musicVolume, SOUND_DEFAULTS.musicVolume),
    };
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
    /** What the music does now: e2e reads it. */
    __music?: () => { track: string | null; muffled: boolean; playing: boolean; time: number; filterHz: number };
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
    musicGain = ctx.createGain();
    musicGain.gain.value = musicLevel();
    musicFilter = ctx.createBiquadFilter();
    musicFilter.type = "lowpass";
    musicFilter.Q.value = 0.7;
    musicFilter.frequency.value = OPEN_HZ;
    musicFilter.connect(musicGain).connect(master);
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
  applyMusic();
}

let started = false;
/** Reads the settings and arms the first-gesture unlock; call once at start. */
export function initSound(): void {
  if (started) return;
  started = true;
  settings = loadSettings();
  window.__sfx = window.__sfx ?? [];
  window.__music = () => {
    const el = playing ? elements.get(playing.file) : undefined;
    return { track: playing?.file ?? null, muffled: want?.muffled ?? false, playing: !!el && !el.paused, time: el?.currentTime ?? 0, filterHz: musicFilter?.frequency.value ?? 0 };
  };
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
    // The <audio> runs on its own clock: pause it too, so a hidden tab
    // neither plays on nor jumps ahead.
    if (document.hidden) for (const el of elements.values()) el.pause();
    else applyMusic();
  });
}

export const soundSettings = (): SoundSettings => ({ ...settings });

/** Changes the settings, saves them, and tells every open Sound row. */
export function setSound(next: Partial<SoundSettings>): void {
  settings = { ...settings, ...next };
  saveSettings(settings);
  if (master && ctx) master.gain.setValueAtTime(masterGain(), ctx.currentTime);
  if (musicGain && ctx) musicGain.gain.setTargetAtTime(musicLevel(), ctx.currentTime, 0.05);
  if (ctx && settings.on && !document.hidden) void ctx.resume().catch(() => {});
  applyMusic();
  for (const fn of listeners) fn(soundSettings());
}

export const toggleSound = (): void => setSound({ on: !settings.on });
export const toggleMusic = (): void => setSound({ music: !settings.music });

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

// ---------- music (round 4, note 5) ----------

export interface Track {
  file: string;
  bpm: number;
  /** Seconds to the first beat. */
  offset: number;
  loopStart: number;
  loopEnd: number;
}
export const HOME_TRACK: Track = TRACKS.home;
export const THEMES: readonly Track[] = TRACKS.themes;

/** Where the music is: home's loop, or a run's theme in the shop (muffled)
 * or in battle (open). */
export type MusicScene = "home" | "shop" | "battle";

/** A run's theme: the same run always gets the same one. */
export function themeFor(runId: string): Track {
  let h = 2166136261;
  for (let i = 0; i < runId.length; i++) h = Math.imul(h ^ runId.charCodeAt(i), 16777619);
  return THEMES[(h >>> 0) % THEMES.length]!;
}

/** What a scene plays and through how open a filter. */
export function sceneTrack(scene: MusicScene, runId = ""): { track: Track; muffled: boolean } {
  return scene === "home" ? { track: HOME_TRACK, muffled: false } : { track: themeFor(runId), muffled: scene === "shop" };
}

const OPEN_HZ = 20000;
/** The shop's low-pass: the theme heard through a wall. */
export const MUFFLED_HZ = 700;
export const CROSSFADE_S = 1;

let musicGain: GainNode | null = null;
let musicFilter: BiquadFilterNode | null = null;
let want: { track: Track; muffled: boolean } | null = null;
let playing: Track | null = null;
const elements = new Map<string, HTMLAudioElement>();
const fades = new Map<string, GainNode>();
const stopTimers = new Map<string, ReturnType<typeof setTimeout>>();

const musicLevel = () => (settings.music ? settings.musicVolume * settings.musicVolume : 0);

function log(entry: string): void {
  const l = (window.__sfx ??= []);
  l.push(entry);
  if (l.length > LOG_MAX) l.splice(0, l.length - LOG_MAX);
}

/** Sets where the music is. A new track crossfades in over 1 s; shop ↔
 * battle on the same theme only opens or closes the filter, so the beat
 * never breaks. Before the first tap it is only remembered. */
export function music(scene: MusicScene, runId = ""): void {
  const next = sceneTrack(scene, runId);
  if (want?.track.file !== next.track.file) log(`music:${next.track.file.replace(/\.m4a$/, "")}${settings.on && settings.music ? "" : " (muted)"}`);
  want = next;
  applyMusic();
}

/** The <audio> of a track, wired into the music bus once: streamed, never
 * decoded whole. */
function element(t: Track): HTMLAudioElement | null {
  const had = elements.get(t.file);
  if (had) return had;
  if (!ctx || !musicFilter) return null;
  const el = new Audio(`${import.meta.env.BASE_URL}music/${t.file}`);
  el.preload = "auto";
  // The files are cut to whole loops, so the element loops them; a track
  // with an inner loop jumps back at its end.
  el.loop = t.loopStart === 0;
  el.addEventListener("timeupdate", () => {
    if (!el.loop && el.currentTime >= t.loopEnd) el.currentTime = t.loopStart;
  });
  try {
    const fade = ctx.createGain();
    fade.gain.value = 0;
    ctx.createMediaElementSource(el).connect(fade).connect(musicFilter);
    fades.set(t.file, fade);
  } catch {
    return null;
  }
  elements.set(t.file, el);
  return el;
}

function fadeTo(file: string, level: number): void {
  const g = fades.get(file);
  if (!g || !ctx) return;
  const now = ctx.currentTime;
  g.gain.cancelScheduledValues(now);
  g.gain.setValueAtTime(g.gain.value, now);
  g.gain.linearRampToValueAtTime(level, now + CROSSFADE_S);
}

/** Brings what plays in line with `want` and the settings. */
function applyMusic(): void {
  if (!ctx || !musicFilter) return;
  const on = settings.music && !document.hidden && want !== null;
  const target = on ? want!.track : null;
  if (playing && playing !== target) {
    const old = playing;
    fadeTo(old.file, 0);
    stopTimers.set(old.file, setTimeout(() => elements.get(old.file)?.pause(), CROSSFADE_S * 1000 + 50));
    playing = null;
  }
  if (!target) return;
  const el = element(target);
  if (!el) return;
  clearTimeout(stopTimers.get(target.file));
  if (playing !== target) fadeTo(target.file, 1);
  playing = target;
  if (el.paused) void el.play().catch(() => {});
  const hz = want!.muffled ? MUFFLED_HZ : OPEN_HZ;
  musicFilter.frequency.cancelScheduledValues(ctx.currentTime);
  musicFilter.frequency.setTargetAtTime(hz, ctx.currentTime, CROSSFADE_S / 3);
}
