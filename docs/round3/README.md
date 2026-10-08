# Arena MVP, round 3: the plan

Maks played round 2 (build 15a93ba9) on 2026-10-06 and sent 19 notes. Five scouts studied them against the code, and he
approved this plan the same day (plan page: https://m1.twin-pogona.ts.net/r/arena-r3-plan.html). The slices are
sub-issues of makscee/void-board#574 and merge into `mission-574-mvp`. Slice 13 (#588, invite links) runs alongside.

**Where this README and the notes disagree, this README wins:** it holds Maks's final answers. The notes were written
against 15a93ba9; their file:line pointers may have moved.

## Maks's notes (numbered)

1. Necromancer and Divinity are the same hero; that shouldn't be allowed. Necromancer's Awoken is just +1 HP: Awoken
   should change the unit more drastically.
2. A summon goes to the front, so it acts at once.
3. A summoned unit's card can be seen.
4. Keywords stand alone: Damage mustn't mention Shield (keywords may be removed later).
5. Fusions have too much HP.
6. Shorter keyword text: "at the end of each turn" → "turn end", "after another ally summoned" → "ally summoned".
7. Tiers in Roman numerals.
8. The card shows icons for everything in its text, not only the trigger.
9. Esc closes things and goes back.
10. Trigger icons always show during playback, so the cause of every action is clear.
11. See instantly who targets whom (a line or similar the eye catches).
12. Freeze, to keep a shop slot.
13. Storing units; more decisions once the team is full.
14. Slower battle animation, with emphasis on some actions.
15. Clicking a unit in battle shows its statuses.
16. Sounds, taken from past iterations of the game.
17. Awoken units sell for 2.
18. The battle opens on Log; Why is active only when there is something to show.
19. Chain cap 32.

## Maks's answers

- **Go, plus slice 13** (#588, invite links and sign-in), which runs alongside. It is strict (auth): its PR into the
  mission branch waits for Maks's word.
- **A full team gets a bench of 3 and an awakening gift** (shop.md, options A and C). The next foe's line is NOT shown.
  - **Bench:** shop.md (A). It holds 3 units that don't fight. Copies merge and awaken there, units fuse from it, and
    units move between line and bench freely.
  - **Gift** (shop.md, option C; Hearthstone's Discover): the copy that awakens a unit also gives a free pick of 1
    of 3 units from the highest tier open that round (`tierOpensAt`). The pick goes to the line, else the bench. While
    a gift is waiting, the player may sell and reorder but not buy, reroll, lock, fuse or fight. They pick one or skip
    it. Picking with line and bench both full is refused ("make room"). A picked copy merges like a buy, and if it
    awakens a unit, that gives another gift. Bots take the best pick by their usual score, or skip when they have no
    room. In the game it is called a **gift**, never "Discover": fusions already use "discovered" for first-found names.
  - The final playtest measures the gift's power: win rates of lines with and without gifts, gifts per run, how often
    a run reaches the Crown.
- **Awoken must do something new** (units.md 1c, stricter than its recommendation). Every Awoken form must change its
  Who kind or gain an effect kind its sleeping form lacks, and **Strength or Vitality riders don't count** as the new
  part. A bigger number alone never counts. About 68 forms change, in two content slices (tiers I–II, then III–IV).
  units.md's table covers the 32 numbers-only forms; the slices write the rest in the same spirit (a new job, not a
  bigger number).
- **The Crown keeps 5 hearts.**

## Calls made for him (R4; he can overturn them)

- "Same hero" is strict at every tier: no two units share a sleeping (or Awoken) When + Who kind + effect-kind
  signature, and the listen→emit graph has no loops (units.md R1, R2, R4).
- Necromancer stays the reviver (Awoken: Revive 1 + Strength 3, undead glass cannons). Divinity becomes the team
  protector (ally dies → all allies → Bless 1; Awoken adds Shield 2). See units.md 1a.
- Fusion stats: the stronger part's PWR and HP, plus one copy's growth (+1 PWR, +2 HP), instead of the sum. Later
  copies of either part still add +1/+2.
- Awoken and fused units sell for 2. Sleeping units still sell for 1.
- The shop's freeze is called **Lock** (padlock icon). Freeze is a battle status. A lock is free and lasts until the
  offer is bought or unlocked; locked offers survive rerolls and new rounds.
- Only summons go to the front. Revived units still return at the back. Old replays keep drawing summons at the back,
  because the Summon event says where the unit went.
- "Ally" in unit text always means another ally. The 11 units that count themselves change to match (words.md).
- The card's When · Who · Does icon line sits in the top row; the phone fits 3 icons plus "+".
- Who targets whom: curved beams in the effect's colour (battle.md, mockups/battle-targets.html).
- Pacing: about 1.5× today's beat times, with extra time on kills, big hits, summons, the first fatigue and the last
  beat. 2× stays the default from round 4.
- Sounds: on at 60% volume, no music this round. The 2022 Arena set is used first, and Duamo's sounds fill the gaps
  (sounds/README.md: all from Maks's own games).
- The chain cap is 32 for new runs; runs already going keep 64.
- Content changes end runs in progress at deploy with "content changed", as in round 2.

## The slices

| # | Slice | After | Notes |
|---|---|---|---|
| 1 | Short unit text | – | words.md (6) |
| 2 | Keywords stand alone, Codex wording | 1 | words.md (4) |
| 3 | Roman tiers | – | words.md (7) |
| 4 | Card icon line | 3 | words.md (8) |
| 5 | Summoned units have cards | 1, 18 | words.md (3) |
| 6 | Esc everywhere | 17 | words.md (9) |
| 7 | One hero per shape | 1 | units.md 1a, 1b |
| 8 | Awoken does something new: tiers I–II | 7 | units.md 1c |
| 9 | Awoken does something new: tiers III–IV | 8 | units.md 1c |
| 10 | Fusion stats: the stronger part, +1/+2 | – | units.md (5) |
| 11 | Awoken and fused sell for 2 | – | shop.md (17) |
| 12 | Lock shop offers | 11 | shop.md (12) |
| 13 | Bench: rules, server, bots | 12 | shop.md (13 A) |
| 14 | Bench in the shop | 13 | shop.md (13 A) |
| 15 | Awakening gift: rules, server, bots | 13 | this README, shop.md (13 C) |
| 16 | Awakening gift: the chooser | 14, 15 | this README |
| 17 | Log first, Why on demand | – | battle.md (18) |
| 18 | Inspect a unit in battle | 17 | battle.md (15) |
| 19 | A cause icon on every action | 18 | battle.md (10) |
| 20 | Slower battles, with emphasis | 19 | battle.md (14) |
| 21 | Target beams | 20 | battle.md (11), mockups/ |
| 22 | Summons at the front | – | battle.md (2) |
| 23 | Chain cap 32 | 22 | battle.md (19) |
| 24 | Sound: engine, shop, menu | – | sounds.md |
| 25 | Battle sounds | 20, 24 | sounds.md |
| 26 | Playtest, fixes, report (the orchestrator) | all | – |

`sim/` holds the scouts' measuring scripts (run with `npx tsx`). `sounds/web/` holds the 26 sound files ready for
`mobile/public/sfx/` (`sounds/index.html` plays them).
