# Arena sound candidates (round 3, note 16)

All of these come from Maks's own past games. No credits or licence file sits next to any of them, in any branch or repo. The arena-of-ideas repo carried them under its Apache-2.0 LICENSE (added 2023-12) until the v5 clean slate (ac67520e, 2026-06-10). Duamo (makscee/duamo, public, Unity, 2020) has no licence file and its itch.io page credits only makscee.

Get one back from git: `git -C /Users/admin/Work/arena-574/repo show <commit>:<path> > file`.

## aoi-2022/: the first Arena of Ideas sounds (geng/Rust build, Nov 2022)

Source: `b256823d:static/sounds/*.ogg` (the 2022-11-23 "Fix sound volumes" set). The 2023 rewrite (33079762) kept the same files, renaming face_hit to hit_melee, remote_attack to hit_range, win_game to victory and lose_game to defeat. The 2024 Bevy build re-added most of them under `assets/audio/fx/` (878131b7). shop.ogg is from 03dbeb68.

| file | length | what the old code played it for |
| --- | --- | --- |
| click.ogg | 0.11 s | a unit picked up from the shop or the team (2022 shop/mod.rs:283,289); every button click in 2023 and 2024 |
| coin.ogg | 0.43 s | a buy and a sell (2022 sound_controller buy/sell); credits notification (2024) |
| shop.ogg | 0.80 s | the first buy/sell sound, swapped for coin.ogg 4 days later (03dbeb68) |
| merge.ogg | 0.92 s | a copy stacked onto a unit you own: "+Stack" (2022 shop/mod.rs:330) |
| level_up.ogg | 1.27 s | a stack that levels the unit up: "Level Up!" (2022 shop/mod.rs:323); a Medics heal (2023) |
| face_hit.ogg | 0.41 s | a melee hit (2022 clan_effects/common.json; 2023 HitMelee) |
| remote_attack.ogg | 0.42 s | a ranged hit or laser (2022 vfx_laser; 2023 HitRange) |
| absorb.ogg | 0.64 s | Shield absorbs a hit (2022 statuses/Shield.json, ShieldAbsorb) |
| shield.ogg | 0.59 s | Shield attached to a unit (2022 statuses/Shield.json) |
| buff.ogg | 0.81 s | stats gained (2022 vfx_stats_gain; 2023 "+" status) |
| debuff.ogg | 0.80 s | stats lost (2022 vfx_stats_lose); a unit's death (2024 build) |
| spawn.ogg | 0.80 s | a unit spawned or summoned (2022 vfx_spawn) |
| start_game.ogg | 2.11 s | battle start (2022, 2023); shop entered (2024) |
| win_game.ogg | 2.38 s | victory |
| lose_game.ogg | 2.63 s | defeat |

## duamo-2024/: Duamo's puzzle sounds, reused by the 2024 Arena build

Source: `ac67520e^:assets/audio/fx/*.ogg` (added 0f3d415c, 2024-09-29), converted from makscee/duamo `Assets/Sounds/*.wav`. The 2024 Arena build mapped them in `assets/ron/_dynamic.assets.ron`.

| file | length | 2024 Arena use (Duamo use, from its name) |
| --- | --- | --- |
| insert_main.ogg | 1.55 s | Strike (a piece locks in) |
| insert_start.ogg | 0.37 s | a status added |
| insert_wrong.ogg | 0.28 s | Pain, a unit hurt (a piece that doesn't fit) |
| undo.ogg | 0.34 s | a status removed |
| field_open.ogg | 2.56 s | game start |
| field_complete.ogg | 3.69 s | victory (the puzzle solved) |
| field_close.ogg | 1.95 s | unused (the field closing) |
| move_attached_left_1 / right_1.ogg | 0.17 / 0.15 s | unused (a piece moved left / right) |
| move_attached_rotate_left_1 / right_1.ogg | 0.28 / 0.27 s | unused (a piece rotated) |
| shape_sides_close / open.ogg | 0.14 / 0.17 s | unused (a shape's sides closing / opening) |

## music/ (not proposed for this round)

| file | length | size | source |
| --- | --- | --- | --- |
| aoi-2022-music_loop.ogg | 85 s | 1.4 MB | `b256823d:static/sounds/music_loop.ogg`, looped in the 2022 shop and battle |
| duamo-game-theme-1.ogg | 303 s | 3.7 MB | `ac67520e^:assets/audio/bg/game theme 1.ogg`, one of 9 Duamo themes (3.7 to 7 MB each) the 2024 build shuffled as background music |

Left out for size: `13711559:assets/audio/background 1.ogg` (269 s, 14 MB, the 2023 shop music) and `22f79fd1:assets/audio/background 1 filtered.ogg` (13.5 MB).

## web/: ready to ship

26 mp3 files (320 KB total), named by what they do in the game. Each one is mono, 44.1 kHz, LAME VBR q4, leading silence trimmed, peak normalised to −3 dBFS. discover.mp3 is cut to 2.6 s with a 0.4 s fade. Built from `web-map.txt` (key, source, optional max seconds). The relative gains in `notes/sounds.md` even out how loud they sound. MP3 because Safari before 18.4 doesn't play Ogg.

Open `index.html` here in a browser to hear every one of them.
