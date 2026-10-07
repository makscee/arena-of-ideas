# Music notes (note 5)

Scout, against f9243387. Round 3's sfx: `docs/round3/sounds.md`, `mobile/ui/sound.ts` (Web Audio; first tap creates the `AudioContext` and decodes all sfx; one master gain, volume², saved as `arena.sound`; suspended on hidden tab; `window.__sfx` logs the last 50).

## Tracks (Ogg Vorbis, stereo, in makscee/arena-of-ideas history)
| Track | Commit:path | Size | Length | BPM |
|---|---|---|---|---|
| `music_loop.ogg` (2022) | `b256823d:static/sounds/` | 1.4 MB | 85.3 s | **90** (exactly 128 beats) |
| `background 1.ogg` (2023 shop) | `22f79fd1:assets/audio/` | 13.5 MB | 268.8 s | **100** (hardcoded) |
| `background 1 filtered.ogg` | same | 5.0 MB | – | low-pass copy |
| `game theme 1–9.ogg` (2024, from Duamo) | `878131b7:assets/audio/bg/` | 3.7–7.0 MB | 303–417 s | ≈103, 104, 134, 104, 123, 107/161, 100, 107/161, 99 (estimates: confirm) |

Fetch without a full clone: `git clone --filter=blob:none --no-checkout`, then `git show <commit>:<path>`. Duamo (`makscee/duamo:Assets/Sounds/`, MP3) holds themes 5–9 plus ending/trailer themes. No licence file anywhere: all from Maks's own games; composer to be asked.

## The 2023 beat sync (Bevy, `src/plugins/audio.rs`, `Expression::Beat`)
BPM 100, beat 0.6 s, `beat_index`/`to_next_beat` from the music's playback position. `Beat` goes ±1 → 0 with QuartOut over half a beat, sign flips each beat and on even slots; units used `1 + 0.1·Beat` for size/rotation (52 of 98 heroes). Shop played the filtered version, battle the full one from the same position. Replay started at `-to_next_beat`.

## Design (chosen: on by default, 40%)
- Home: `music_loop`. A run gets one theme (4 shipped): muffled in the shop with a `BiquadFilterNode` low-pass, opened in battle, so the beat never breaks.
- AAC-LC `.m4a` 96 kbps stereo (~0.7 MB/min; Ogg fails on older iPhones). Optionally trim themes to 90–120 s loops.
- Stream through `<audio>` → `createMediaElementSource → musicGain → master`; never decode whole themes (~100 MB on a phone). Lazy after first tap, current track only, preload next during the shop, 1 s crossfade, pause on hidden tab.
- Music on/off + volume (default 0.4) in `arena.sound` as `music`/`musicVolume`; M stays master mute; log `music:<track>` to `__sfx`.
- `tracks.json`: `{file, bpm, offset, loopStart, loopEnd}`. `beatClock()` → `{bpm, beatMs, phase, nextBeatIn()}` from `audio.currentTime − offset`, smoothed with `performance.now()`; free-running at the same BPM when music is off or loading.
- Battle on the beat: `schedule()` starts each beat at `nextBeatIn()`, rounds beat lengths to whole music beats (at 100 BPM: 900 → 1200, 1300 → 1200, 2200 → 2400 ms), waves on 8th notes; 2× on the half-beat grid. Stepping and scrubbing stay instant.
