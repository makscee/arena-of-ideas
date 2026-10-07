// The beat clock (round 4, note 5; docs/round4/music.md): where the music's
// beat is now, for the dance (ui/dance.ts) and the battle on the beat. It
// reads the music's position (`currentTime − offset`) and smooths it with
// performance.now(), since a media element's clock moves in coarse steps;
// with no music playing (off, loading, a hidden tab) it runs free at the
// track's BPM. Pure: the sources are passed in, so tests drive it.

/** What the music says now. `time` is the track's playback position in
 * seconds, or null when nothing plays (the clock then runs free). */
export interface BeatSource {
  bpm: number;
  /** Seconds to the track's first beat. */
  offset: number;
  time: number | null;
}

export interface Beat {
  bpm: number;
  beatMs: number;
  /** Beats so far, whole (it counts the music's beats, or free ones). */
  index: number;
  /** 0..1 through the current beat: 0 is on the beat. */
  phase: number;
  /** performance.now() of beat 0: what the dance's CSS keys its phase on. */
  origin: number;
  /** ms to the next beat, read when called. */
  nextBeatIn(): number;
}

/** Past this, a reading is a jump (a seek, a loop back, a stall), not
 * jitter: the clock takes it as it is. */
export const JUMP_S = 0.15;
/** A reading that hasn't moved for this long is a stall: the clock stops
 * with the music instead of running on alone. */
export const STALL_MS = 500;
/** How much of the gap to a new reading the smoothed position takes. */
export const SMOOTH = 0.1;

/** A clock over `source`, read with performance.now() (`now`, in ms). */
export function makeBeatClock(source: () => BeatSource, now: () => number = () => performance.now()): () => Beat {
  // The smoothed music position: `pos` seconds at performance.now() `at`;
  // `seen` is the last raw reading, so a reading that hasn't moved yet (the
  // element steps every ~250 ms) adds nothing and the clock just runs on.
  let anchor: { at: number; pos: number; bpm: number; seen: number; seenAt: number } | null = null;
  return () => {
    const s = source();
    const t = now();
    const bpm = s.bpm > 0 ? s.bpm : 100;
    const beatMs = 60000 / bpm;
    let beats: number;
    if (s.time === null) {
      anchor = null;
      beats = t / beatMs;
    } else {
      const a = anchor && anchor.bpm === bpm ? anchor : null;
      const predicted = a ? a.pos + (t - a.at) / 1000 : 0;
      let pos: number;
      if (a && s.time === a.seen) pos = t - a.seenAt > STALL_MS ? s.time : predicted;
      else pos = !a || Math.abs(s.time - predicted) > JUMP_S ? s.time : predicted + (s.time - predicted) * SMOOTH;
      anchor = { at: t, pos, bpm, seen: s.time, seenAt: a && s.time === a.seen ? a.seenAt : t };
      beats = ((pos - s.offset) * bpm) / 60;
    }
    const index = Math.floor(beats);
    const phase = beats - index;
    const origin = t - beats * beatMs;
    return {
      bpm,
      beatMs,
      index,
      phase,
      origin,
      nextBeatIn: () => {
        const ahead = (now() - origin) / beatMs;
        return (Math.floor(ahead) + 1 - ahead) * beatMs;
      },
    };
  };
}

/** Where the dance's two-beat cycle stands at `t` (ms into it), for a slot:
 * odd slots run a beat behind, so neighbours sway in opposite phase. */
export function bobPhase(b: Pick<Beat, "beatMs" | "origin">, t: number, odd: boolean): number {
  const cycle = 2 * b.beatMs;
  const at = t - b.origin + (odd ? b.beatMs : 0);
  return ((at % cycle) + cycle) % cycle;
}
