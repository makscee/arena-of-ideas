// The sound engine (round 3, note 16; docs/round3/sounds.md): Web Audio,
// unlocked by the first tap or key, the sfx files preloaded then, one master
// gain. What plays when is ui/sound-map.ts, and its admit() keeps cues from
// piling up (void-board#922). Music (round 4, note 5; docs/round4/music.md)
// streams through the same context: home plays one theme, a run one theme,
// muffled in the shop and open in battle.
import { SOUND_FILES, admit, cue, type Cue, type SoundKey, type Voice } from "./sound-map";
import TRACKS from "./tracks.json";

export interface SoundSettings {
  on: boolean;
  /** 0..1; the master gain is its square, so the slider feels even. */
  volume: number;
  /** The music channel, under the master: on by default at 40%. */
  music: boolean;
  /** 0..1, linear: 40% sets the loops (levelled to −18 LUFS) about 8 dB
   * under the battle sounds, a bed under them, not far below. */
  musicVolume: number;
}

export const SOUND_STORE = "arena.sound";
export const SOUND_DEFAULTS: SoundSettings = { on: true, volume: 0.6, music: true, musicVolume: 0.4 };
/** How long a cut-short voice takes to fade out, in seconds. */
const CUT_FADE_S = 0.04;
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
const buffers = new Map<string, AudioBuffer>();
/** The sfx voices still sounding, for admit(), with how to cut each. */
const voices = new Map<number, Voice & { stop: (fade: number) => void }>();
let nextVoice = 1;
const listeners = new Set<(s: SoundSettings) => void>();

declare global {
  interface Window {
    /** The last 50 sounds asked for, "key" or "key (muted)": e2e reads it. */
    __sfx?: string[];
    /** What the music does now: e2e reads it. */
    __music?: () => { track: string | null; muffled: boolean; playing: boolean; time: number; filterHz: number; master: number };
  }
}

/** The portal's muteAudio setting and an ad break (the cg build, #1219): each
 * keeps everything silent without touching the player's saved settings. */
let portalMuted = false;
let adPlaying = false;
const audible = () => settings.on && !portalMuted && !adPlaying;
const masterGain = () => (audible() ? settings.volume * settings.volume : 0);

function applyMaster(): void {
  if (master && ctx) master.gain.setValueAtTime(masterGain(), ctx.currentTime);
}

/** CrazyGames' muteAudio: on keeps the game silent, off gives the player's own settings back. */
export function setPortalMute(on: boolean): void {
  portalMuted = on;
  applyMaster();
}

/** An ad break starts: mute. */
export function onAdStarted(): void {
  adPlaying = true;
  applyMaster();
}

/** The ad break ended (or failed): back to what it was. */
export function onAdFinished(): void {
  adPlaying = false;
  applyMaster();
}

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
    for (const key of SOUND_FILES) {
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
    return { track: playing?.file ?? null, muffled: want?.muffled ?? false, playing: !!el && !el.paused, time: el?.currentTime ?? 0, filterHz: musicFilter?.frequency.value ?? 0, master: master?.gain.value ?? 0 };
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
  applyMaster();
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
  log.push(audible() ? q.key : `${q.key} (muted)`);
  if (log.length > LOG_MAX) log.splice(0, log.length - LOG_MAX);
  if (!audible() || document.hidden || !ctx || !master) return;
  const buf = buffers.get(q.file);
  if (!buf) return;
  const now = performance.now();
  // A voice whose end never came (a suspended tab) stops counting after 4 s.
  for (const [id, v] of voices) if (now - v.at > 4000) voices.delete(id);
  const verdict = admit([...voices.values()], q, now);
  if (!verdict.play) return;
  for (const id of verdict.cut) voices.get(id)?.stop(CUT_FADE_S);
  try {
    const audio = ctx;
    const src = audio.createBufferSource();
    src.buffer = buf;
    src.playbackRate.value = q.rate;
    const g = audio.createGain();
    g.gain.value = q.gain;
    src.connect(g).connect(master);
    const id = nextVoice++;
    let stopped = false;
    const stop = (fade: number) => {
      if (stopped) return;
      stopped = true;
      voices.delete(id);
      const at = audio.currentTime;
      try {
        g.gain.cancelScheduledValues(at);
        g.gain.setValueAtTime(g.gain.value, at);
        g.gain.linearRampToValueAtTime(0, at + fade);
        src.stop(at + fade + 0.01);
      } catch {
        // an older Safari refuses a second stop(): the cut's own stop ends it
      }
    };
    src.onended = () => void voices.delete(id);
    voices.set(id, { id, key: q.key, prio: q.prio, at: now, stop });
    src.start();
    // A long file made a short cue: it fades out at its cut.
    if (q.cut && q.cut < buf.duration / q.rate) {
      const fade = 0.06;
      g.gain.setValueAtTime(q.gain, audio.currentTime + q.cut - fade);
      g.gain.linearRampToValueAtTime(0, audio.currentTime + q.cut);
      src.stop(audio.currentTime + q.cut + 0.01);
    }
  } catch {
    // a context closed under us: stay silent
  }
}

// ---------- music (round 4, note 5) ----------

export interface Track {
  file: string;
  /** Linear gain that levels the track to −18 LUFS (docs/mission5/sound/level.py). */
  gain: number;
  bpm: number;
  /** Seconds to the first beat. */
  offset: number;
  loopStart: number;
  loopEnd: number;
}
export const HOME_TRACK: Track = TRACKS.home;
export const THEMES: readonly Track[] = TRACKS.themes;

/** Where the music is: home's theme, or a run's theme in the shop (muffled)
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

const musicLevel = () => (settings.music ? settings.musicVolume : 0);

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
  if (want?.track.file !== next.track.file) log(`music:${next.track.file.replace(/\.m4a$/, "")}${audible() && settings.music ? "" : " (muted)"}`);
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
  // Home's theme is a run theme too: the same file plays on, never restarts.
  if (playing && playing.file !== target?.file) {
    const old = playing;
    fadeTo(old.file, 0);
    stopTimers.set(old.file, setTimeout(() => elements.get(old.file)?.pause(), CROSSFADE_S * 1000 + 50));
    playing = null;
  }
  if (!target) return;
  const el = element(target);
  if (!el) return;
  clearTimeout(stopTimers.get(target.file));
  if (playing?.file !== target.file) fadeTo(target.file, target.gain);
  playing = target;
  if (el.paused) void el.play().catch(() => {});
  const hz = want!.muffled ? MUFFLED_HZ : OPEN_HZ;
  musicFilter.frequency.cancelScheduledValues(ctx.currentTime);
  musicFilter.frequency.setTargetAtTime(hz, ctx.currentTime, CROSSFADE_S / 3);
}
