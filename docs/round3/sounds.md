# Sounds (round 3, note 16)

> "add sounds, take it from past iterations of the game"

This scout covers build 15a93ba9 (`/Users/admin/Work/arena-574/scout-r3`) and the history of the arena-of-ideas repo, plus makscee/duamo. Nothing was run against the live instance.

- Candidates: `v3/sounds/` (originals in `aoi-2022/`, `duamo-2024/`, `music/`; list, sources and licence in `v3/sounds/README.md`)
- Ready to ship: `v3/sounds/web/*.mp3`, 26 files, 320 KB, named by game event. Built by `web-map.txt`.
- To listen: open `v3/sounds/index.html` in a browser.

## Today

- The client has no audio at all: no audio files, no `Audio` or `AudioContext` under `mobile/`, and no `mobile/public/`.
- Shop actions all go through `decide()` (`mobile/main.ts:421`), which sends a decision and redraws from the `RunView` it gets back. The reply carries no events, so "a copy merged" and "it awoke" can only be read by comparing the line before and after. Fuse has its own path (`main.ts:634`), and so does the "You discovered" reveal (`main.ts:641`). The reroll key is at `main.ts:876`, sell at `:889`, buy by number at `:871`.
- Battle playback is `schedule()` in `mobile/screens/battle.ts:235`. Each timer tick lands one **wave**: `beats[at].waves[wave]`, a `Step` with `changes[]` (`src/mvp/trace.ts:300`). Waves are 150 ms apart and a beat lasts 0.7 to 1.4 s at 1× (`trace.ts:398-425`). Seeking, stepping and scrubbing go through `go()`, `back()` and `forward()` and never call `schedule()`. Speed is 1× or 2×, and is saved in localStorage the way `storedSpeed()` does it (`battle.ts:68`). The end card comes from `finish()` (`battle.ts:341`). `fight.kind === "crown"` with `outcome === "win"` is a slay (`src/mvp/contract.ts:248`).
- Menus: the title menu is `home-actions` (`main.ts:241`) and the in-run ☰ is `runMenu()` (`main.ts:314`). The battle's ☰ reuses `runMenu` through `outro.menu` (`battle.ts:180`). Keys go through `onKeys()` (`mobile/ui/dom.ts:101`). The letter M is free.

## Past iterations had a full set

There are three eras, all in `/Users/admin/Work/arena-574/repo` history (full table in `v3/sounds/README.md`):

1. **2022 geng build** (`b256823d:static/sounds/`, 15 files). It has a dedicated sound for nearly every event we have now: buy/sell `coin`, copy stacked `merge`, level-up `level_up`, melee `face_hit`, ranged `remote_attack`, Shield absorbs `absorb`, Shield attached `shield`, `buff`, `debuff`, summon `spawn`, `start_game`, `win_game`, `lose_game` and `click`. The 2023 rewrite kept the same files.
2. **2024 Bevy build** (`ac67520e^:assets/audio/fx/`). It used the same files, plus Duamo's puzzle sounds (`insert_main` = strike, `insert_wrong` = pain, `insert_start` = status added, `undo` = status removed, `field_complete` = victory) and `debuff` for death. Its plugin was a queue of fire-and-forget players with separate music and fx volumes (`c10fb395^:src/plugins/audio.rs`).
3. **Music**: a 2022 85 s loop (1.4 MB), 2023 shop tracks (14 MB) and nine Duamo themes (3.7 to 7 MB each). None of it is proposed for this round (see the decision below).

**Licence:** there is no credits or licence file anywhere, in any branch, in the duamo repo or on its itch page. Everything comes from Maks's own games. The Arena repo carried the sounds under its Apache-2.0 LICENSE from 2023-12. Ship them with `mobile/public/sfx/CREDITS.txt` naming the source: "Arena of Ideas 2022 (makscee/arena-of-ideas static/sounds) and Duamo (makscee/duamo)". If Maks knows who made them, add the name there.

## The change

### Files

- Copy `v3/sounds/web/*.mp3` to `mobile/public/sfx/` and add `CREDITS.txt` there. Vite serves them at `/arena/sfx/<key>.mp3`; use `import.meta.env.BASE_URL + "sfx/"`.
- They are MP3 because Safari before 18.4 and older iPhones won't play Ogg. They are mono, trimmed and peak-normalised. The 320 KB loads only after the first click or tap, so first paint costs nothing.

### Event → sound map

| Event | Sound (web/) | Rate | Gain | Origin |
| --- | --- | --- | --- | --- |
| **Shop** | | | | |
| buy a new unit | coin | 1 | 1.24 | 2022 buy |
| buy a copy that merges in (not the awakening one) | merge | 1 | 0.70 | 2022 "+Stack" |
| buy the copy that awakens it (form sleeping → awoken) | level-up | 1 | 1.17 | 2022 "Level Up!" |
| sell | sell | 1 | 1.08 | 2022 shop.ogg (first buy/sell sound) |
| reroll | reroll | 1 | 0.21 | Duamo rotate |
| freeze / unfreeze a slot (note 12) | freeze / unfreeze | 1 | 0.37 / 0.39 | Duamo shape sides |
| move a unit (reorder) | move-left / move-right | 1 | 0.27 / 0.26 | Duamo move |
| select or pick up a unit | click | 1 | 1.06 | 2022 click |
| fuse confirmed | fuse | 1 | 0.84 | Duamo insert_main (pieces lock) |
| "You discovered" reveal | discover | 1 | 1.01 | Duamo field_complete (2024 victory) |
| refused (a disabled buy/reroll by key, a 409) | wrong | 1 | 0.27 | Duamo insert_wrong |
| **Battle** (one sound per wave, see the cap below) | | | | |
| line-up shown (the battle starts) | start | 1 | 0.71 | 2022 battle start |
| strike hit (a Hurt caused by a Strike, amount > 0) | hit | wave rate | 0.79 | 2022 melee |
| ability or Fatigue damage (other Hurt, amount > 0) | zap | wave rate | 1.10 | 2022 ranged |
| hit fully blocked by Shield (amount 0, absorbed > 0) | block | wave rate | 0.86 | 2022 ShieldAbsorb |
| Shield applied | shield | wave rate | 0.72 | 2022 Shield attached |
| other status applied | status | wave rate | 0.18 | 2024 status add |
| status removed, alone in its wave | status-off | wave rate | 0.18 | 2024 status remove |
| heal | buff | 1.2 × wave rate | 1.16 | 2022 stats gain, pitched up |
| +stat | buff | wave rate | 1.16 | 2022 stats gain |
| −stat, silenced | debuff | wave rate | 0.55 | 2022 stats lose |
| death | debuff | 0.7 | 0.8 | 2024 death, pitched down so it differs from −stat |
| summon / returns | spawn | 1 / 1.15 | 0.93 | 2022 spawn |
| ChainCapped (cascade cut at the cap) | wrong | 1 | 0.4 | Duamo insert_wrong |
| end: your win / loss / draw | win / lose / draw | 1 | 1.02 / 0.83 / 1.27 | 2022 win and lose; Duamo field_close for draw |
| end: a Crown win (slay) | win, then discover 0.5 s later | 1 | as above | a bigger win |
| end of a battle watched without a side (playoff, champion) | win (or draw) | 1 | | |

Gains are linear, picked to even out loudness: UI ticks around −24 dB mean, battle around −19, jingles around −16. They are a starting point to tune by ear.

**Which change picks a wave's sound:** death > summon > damage (block when every hit in the wave was absorbed) > heal > +stat > −stat or silence > Shield applied > status applied > status removed > ChainCapped.

**Wave rate (the chain sound):** in a beat, wave *i* (0-based) plays at `1 + 0.05 × min(i, 6)`, rising to 1.3×. A chain is heard as a rising run without needing another file. Death and summon keep their own rate.

### Cap: no noise from chains

- Only waves landed by `schedule()` while playing make sound. `go()`, `back()`, `forward()`, the timeline drag, Log or Why clicks and Replay's jump make none.
- One sound per wave, and at most **5 per beat**. Deaths, summons and the end sound don't count toward the cap and always play.
- The same key is never started twice within **70 ms** of real time; 2× speed lands waves 75 ms apart.
- A ChainCapped beat at the new cap of 32 (note 19) therefore makes at most 5 to 6 sounds plus its deaths.
- The end sound plays once, when the end card first shows (`finish()`, by playing through or by Skip/⏭). It doesn't play again when the card is reopened, and Replay doesn't count.

### Engine: `mobile/ui/sound.ts` (Web Audio)

- `AudioContext` with one master `GainNode`. Each play is `createBufferSource()` → its own `GainNode` (the key's gain × an optional extra) → master, with `playbackRate` set.
- **Unlock and preload:** on the first `pointerdown` or `keydown` (one listener, `{ once: true }`), create or resume the context, then fetch and `decodeAudioData` all 26 files in parallel. Until a buffer is ready, `play()` of that key does nothing. It never queues and never throws, and a failed fetch is ignored.
- **Settings:** `{ on: boolean, volume: 0..1 }` in localStorage key `arena.sound`. Every read and write is wrapped in try/catch, the way `storedSpeed()` does it; when storage fails, use the defaults (on, 0.6). Master gain = `on ? volume² : 0`. Squared, so the slider feels even.
- **Tab hidden:** on `visibilitychange`, call `ctx.suspend()` when hidden and `ctx.resume()` when visible and on. `play()` also returns early while `document.hidden`.
- **Test hook:** every `play()` call appends `key` (or `key (muted)`) to `window.__sfx`, the last 50 entries. e2e asserts against it; no audio hardware needed.
- **Pure, node-tested mapping:** `mobile/ui/sound-map.ts`, like `ui/diff.ts`:
  - `shopSound(decision, before: RunView, after: RunView): Cue | null`
  - `waveSound(step: Step, log: BattleEvent[], waveIndex: number): Cue | null`
  - `beatCues(beat, log): (Cue | null)[]` (applies the cap of 5)
  - `endSound(outcome, kind, you?)`

  `Cue = { key, rate, gain }`.

### Controls

- A **Sound** row in the title menu (`home-actions`, `main.ts:241`) and in the in-run ☰ (`runMenu`, `main.ts:314`), so the battle's ☰ gets it too. The row is a toggle button "Sound on / off" (testid `sound-toggle`) and a range input 0 to 100 (`sound-volume`). Moving the slider plays `click` at the new volume so you can hear the level.
- Desktop: **M** toggles sound anywhere, in `dom.ts`'s keydown listener before the screen's `keys`. It is skipped in inputs, like the other keys. Its hint goes in the legend or rules sheet: "M: sound on/off".
- No music.

## Slices

1. **Sound engine, shop sounds, menu controls** (M, client only)
   - `mobile/public/sfx/` (the 26 mp3 files and CREDITS.txt), `ui/sound.ts`, and `ui/sound-map.ts` with tests.
   - Hooks in `decide()`: compare `run` with `res.run` for buy, merge and awaken; sell, reroll, reorder. Also fuse and discover (`main.ts:634-647`), `wrong` on a refusal, and `click` on select.
   - The Sound row in both menus, and the M key.
   - Freeze and unfreeze keys exist in the map; whoever builds note 12 calls them.
2. **Battle sounds** (S–M, client only, after 1)
   - In `schedule()`'s tick, after `render()`, play `beatCues(beats[at], log)[wave]`. At the line-up, play `start`. In `finish()`, play the end sound once, and the slay pair when `a.fight?.kind === "crown" && outcome === "win"`.
   - Nothing on seek.
   - Sound plays as each wave lands, so it stays in step with whatever new timing note 14 sets.

## Tests

- **Unit** (`vitest --maxWorkers=4`):
  - `shopSound`: a new unit → coin; copy 2 → merge; copy 3 (form changes) → level-up; sell, reroll, reorder left and right.
  - `waveSound`:
    - a Strike's Hurt → hit, an ability Hurt → zap, `amount 0 absorbed 2` → block
    - Death beats Hurt in priority
    - Shield applied → shield; Poison applied → status
    - heal → buff at rate 1.2
  - `beatCues`:
    - a 12-wave beat gives at most 5 cues plus deaths
    - wave rates 1, 1.05, … capped at 1.3
  - Settings: localStorage that throws → defaults on and 0.6, and no crash.
- **e2e** (`mvp:desktop`), reading `window.__sfx`:
  - buy by key 1 → `coin`; the third copy → `level-up`; R → `reroll`; S → `sell`
  - Fight → `start`, then at least one of hit/zap/block, then `win`, `lose` or `draw` once at the end
  - ← during the battle adds nothing
  - M → the next sound is logged `(muted)`
  - On the phone run: the menu's Sound toggle works at 360×640.

## How to try

Open `/arena/` and click Play. A buy clinks, and the third copy plays the level-up chime. Reroll, sell and moving a unit each have their own tick. Fight: the start sting, then a hit, zap or Shield-block sound with each wave, rising in pitch through a chain, a low thud on a death, and the win, lose or draw jingle on the end card. Open ☰ to find the Sound toggle and volume, or press M on desktop. Switch tabs mid-battle: it goes quiet.

## Choices for Maks

- **Music?** Recommended: none this round (sfx only). The others: the 2022 85 s loop in the shop only, quiet, off by default (1.4 MB); or the Duamo themes, re-encoded to about 1 MB a minute (3 to 5 MB each).
- **Default:** recommended: sound on at 60%, since he asked for sounds. The other: off until turned on in ☰.
- **Gaps:** recommended: fill them (draw, death, reroll, freeze, fuse) from the old Duamo sounds, with no downloads. The other: Kenney's CC0 packs (kenney.nl/assets: Music Jingles, Impact Sounds, Interface Sounds, RPG Audio) if the puzzle sounds feel out of place.
