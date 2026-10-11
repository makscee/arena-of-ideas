# Sound, one family (void-board#922)

Maks's playtest (10-11): overlapping sounds, loops far quieter than the
effects, a home loop unlike the rest, and two sfx families that don't sit
together. What changed and how it was measured. Levels are LUFS by BS.1770
([lufs.py](lufs.py)): an effect's loudest 400 ms (M-max), a loop's
integrated level (I), both before the master (Sound 0.6 is −8.9 dB on both).

## Listen

A session rendered offline the way each engine mixes it: home 6 s, the shop
(taps, buys, a merge, an awakening, reroll, lock, reorder, sell, a fuse that
discovers), then a real seeded battle with a 5-wave chain and deaths.
Default sliders.

| | 1× | 4× |
|---|---|---|
| before | [before-1x.mp3](before-1x.mp3) | [before-4x.mp3](before-4x.mp3) |
| after | [after-1x.mp3](after-1x.mp3) | [after-4x.mp3](after-4x.mp3) |

| render | sounds at once (max) | 3+ at once |
|---|---|---|
| before 1× | 6 | 7.2% of the time |
| after 1× | 4 (3 plus a cut voice's 40 ms fade) | 1.7% |
| before 4× | 8 | 28.5% |
| after 4× | 4 | 5.9% |

Home music on its own: −46 LUFS before, −35 after (as heard, master in).

Rebuild (numpy, scipy, ffmpeg):

    node --import tsx/esm docs/mission5/sound/timeline.ts mobile/ui/sound-map.ts t.json 1
    python3 -I docs/mission5/sound/render.py after t.json mobile/public/sfx mobile/public/music after-1x.mp3

`before`: the same with the old sound-map.ts and the old sfx and music files
from git (mission-5-toybox @7a630c1), in `before` mode.

## 1. Music: one family

Home played the 2022 Arena loop (90 BPM, other instruments). It now plays
theme4, the calmest of the four Duamo themes the runs use (−20.5 LUFS, peaks
−6 dBFS); home.m4a is gone (−1 MB). Home into a run on theme4 keeps the same
file playing, no restart. No new assets.

## 2. Loudness

Each track has a gain in `mobile/ui/tracks.json` that levels it to −18 LUFS
([level.py](level.py)): theme1 1.02, theme2 1.0, theme4 1.33, theme9 0.98.

The Music slider is now linear instead of squared. At the default 40 % it
was −16 dB, so the bed sat at −34 under effects anywhere from −34 to −12.5; now it is
−8 dB, a bed at −26 under battle cues at −17. Both sliders work as before;
a saved setting keeps its position and simply sounds louder.

Each cue's gain in `mobile/ui/sounds.json` brings it, as played (its file at
its rate and cut), to its tier: UI ticks −22, shop −18, battle −17, jingles
−14, never past −1 dBFS at full volume (freeze and unfreeze are spiky ticks
that the cap keeps at −26). Before, the effects spread over 21 dB.

| cue | before: file → lands at | after: file × rate → lands at |
|---|---|---|
| click | click.mp3 → -29.9 | move-right × 1.3 → -22.0 |
| move-left | move-left.mp3 → -32.9 | move-left × 1 → -22.6 |
| move-right | move-right.mp3 → -32.4 | move-right × 1 → -22.0 |
| freeze | freeze.mp3 → -33.7 | freeze × 1 → -26.0 |
| unfreeze | unfreeze.mp3 → -32.9 | unfreeze × 1 → -25.8 |
| wrong | wrong.mp3 → -26.7 | wrong × 0.85 → -22.1 |
| coin | coin.mp3 → -15.4 | status × 1 → -18.0 |
| sell | sell.mp3 → -15.4 | status-off × 0.9 → -17.9 |
| reroll | reroll.mp3 → -31.5 | reroll × 1 → -18.9 |
| merge | merge.mp3 → -17.8 | fuse × 1.2 cut 0.7 s → -18.0 |
| fuse | fuse.mp3 → -14.0 | fuse × 1 → -18.0 |
| level-up | level-up.mp3 → -12.5 | discover × 1.12 cut 1.3 s → -14.1 |
| discover | discover.mp3 → -13.1 | discover × 1 → -14.1 |
| start | start.mp3 → -16.3 | start × 1 cut 1.8 s → -16.4 |
| hit | hit.mp3 → -20.7 | fuse × 1.35 cut 0.35 s → -17.0 |
| zap | zap.mp3 → -18.7 | wrong × 1.15 → -17.0 |
| block | block.mp3 → -19.6 | status-off × 0.8 → -17.0 |
| shield | shield.mp3 → -19.5 | status × 0.85 → -16.9 |
| buff | buff.mp3 → -13.8 | turn × 1.1 → -19.3 |
| debuff | debuff.mp3 → -18.2 | reroll × 0.85 → -19.2 |
| status | status.mp3 → -27.1 | status × 1.1 → -17.0 |
| status-off | status-off.mp3 → -27.5 | status-off × 1 → -17.1 |
| spawn | spawn.mp3 → -18.8 | start × 1.5 cut 0.6 s → -17.0 |
| death | debuff.mp3 × 0.7 (a death) | draw × 1.35 cut 0.7 s → -17.5 |
| win | win.mp3 → -13.1 | discover × 1 → -14.1 |
| lose | lose.mp3 → -15.7 | draw × 0.85 → -18.4 |
| draw | draw.mp3 → -14.8 | draw × 1 → -17.9 |

## 3. Overlaps

`admit()` in `mobile/ui/sound-map.ts` (pure, tested) decides each cue
against the voices still sounding:

1. the same cue within 120 ms: dropped (was 70 ms);
2. any cue within 90 ms of another is one moment and the stronger plays: a
   merge that awakens plays level-up, a fuse that discovers plays discover, a
   death beats its hit; a tie keeps the first;
3. at most 3 voices at once: the weakest, then the oldest, fades out in 40 ms.

A beat's cap drops from 5 sounds to 3 (deaths and summons still always play),
and long files make short battle cues (hit 0.35 s, spawn 0.6 s, death 0.7 s),
so a fast chain thins out instead of stacking tails.

## 4. One sfx family: Duamo

Kept: Duamo's 13 sounds (the 11 shipped before, plus field_open and
rotate_right from makscee/duamo). Dropped: the 14 Arena 2022 files.

Why Duamo: it is the game the run themes come from, made alongside them, and
it measures like them: spectral centroid 190–800 Hz (the themes 350–660 Hz),
soft synth tones. The 2022 set is bright library-style sound at 1.4–10 kHz
(coin 10.5 kHz, buff 8.3 kHz, spawn 5.5 kHz). The 2024 Arena build already
played Duamo sounds in battle (Strike, a status, Pain, undo, start,
victory), and the mapping follows it where it can. Every cue still has a
sound; a cue that had a 2022 file now plays a Duamo file at its own rate
(table above).

Picked by measurement, not by ear (the attempt had no speakers): judge it
with the renders above. Duamo's sounds are low; on a phone speaker the
lowest (fuse, status) may sound thin.
