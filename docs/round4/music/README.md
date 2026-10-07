# Music tracks (R4-9, void-board#692)

How `mobile/public/music/*.m4a` and `mobile/ui/tracks.json` were made, so the
next track goes in the same way. Sources (Maks's own games, see the CREDITS.txt
beside the files):

    git clone --filter=blob:none --no-checkout https://github.com/makscee/arena-of-ideas.git hist
    git -C hist show b256823d:static/sounds/music_loop.ogg > music_loop.ogg
    git -C hist show "878131b7:assets/audio/bg/game theme 1.ogg" > theme1.ogg   # 2, 4, 9 alike

Measure (numpy + ffmpeg; an onset comb over 70-170 BPM, prints bpm and the
first beat):

    python3 -I bpm.py music_loop.ogg theme*.ogg

| Source | BPM | first beat | notes |
|---|---|---|---|
| music_loop | 90.02 | 0.441 s | 85.333 s = 128 beats, shipped whole |
| game theme 1 | 105.02 | 0.395 s | |
| game theme 2 | 105.02 | 0.534 s | |
| game theme 3 | 100.00 | 0.023 s | scout guessed 134 |
| game theme 4 | 105.02 | 0.534 s | |
| game theme 5 | 90.00 | 0.627 s | scout guessed 123 |
| game theme 6 | 80.00 | 0.093 s | |
| game theme 7 | 102.30 | 0.372 s | |
| game theme 8 | 80.00 | 0.046 s | |
| game theme 9 | 100.00 | 0.139 s | |

Shipped: themes 1, 2, 4 and 9, the ones whose tempo agreed with the scout's
estimate. Each is cut to 192 beats (48 bars) from its first beat, with a 0.5 s
crossfade baked into the seam so the file loops whole (~110 s, 1.3 MB):

    python3 -I loop.py theme1.ogg theme1.m4a 105.02 0.395 192
    ffmpeg -i music_loop.ogg -c:a aac -b:a 96k -movflags +faststart home.m4a

`tracks.json`'s `offset` is the first beat measured again on the shipped file
(0.557 s on the 105 BPM cuts is the beat just after 0).
