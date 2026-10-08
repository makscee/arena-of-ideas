# Round 4: plan and Maks's answers

Maks's 10 notes on build f9243387 (2026-10-07) are on void-board#574. Plan page: https://m1.twin-pogona.ts.net/r/arena-r4-plan.html.
**Approved 2026-10-07: "go, all recommended"** (Maks, via Eva). This file wins over the notes beside it (`engine.md`, `viewer.md`, `music.md`, `units.md`, written by scouts against f9243387, so line numbers may have moved).

## Answers (all the recommended options)
1. **Stalls: sudden death.** Fatigue stays linear for turns 10–19. From turn 20 it doubles every turn (20, 40, 80…), pierces Shield and Blessing, and Summon and Revive do nothing. `turnCap: 30` goes away; the kernel's 200 stays as an internal safety net that never shows. The knobs (e.g. `suddenDeathAt`) live in `MvpRules`, so stored runs keep their rules and golden fixtures don't move.
2. **Awoken builds on the sleeping form.** Engine change: a form can carry an extra "and" action with its own Who on the same When. `Row.awoken` becomes additive (bumps, added Does, an `also` clause), always applied on top of the sleeping form. A test checks no number goes down and nothing is dropped. The round-3 rule R3 ("Awoken must do something new") is relaxed: a numbers-only Awoken is allowed. Henchman Awoken: ally dies → +2 Strength to me, and Silence the front enemy.
3. **Unique archetypes, with the cuts.** Cut Wither, Director, Doctor, Rat, Rot, Spike, Redirector, Pediatrician (81 → 73). Change Commander (strikes → 1 Shield to all allies), Pathologist (enemy gets Poison → 1 Freeze to it), Syren (turn start → 1 Freeze to a random enemy), Icebinder (enemy gets Curse → 1 Freeze to it), Wire (ally gains PWR → 1 Freeze to the front enemy). Archetype sentences: `units.md`.
4. **Music on by default at 40%**, with a Music on/off and volume. The 2022 `music_loop` on home; 4 of the 2024 themes in runs (muffled in the shop, open in battle). Credit: Maks's own earlier games (Arena of Ideas, Duamo); composer: makscee (Maks's answer), named in `CREDITS.txt`.
5. **Battle line of 8**, team stays 5. Meta check and retune after.

## Calls (Maks can overturn any)
- New rules ride in the run's `MvpRules`: runs already going keep theirs.
- Battle steps fit the music's beat (half-beats at 2×); the turn-end hold is skipped at 4× and by End, Replay and key moments.
- Fodder/Custodian and Spore/Morbid stay (own death vs any ally's death).
- Content changes end runs in progress at that deploy with "content changed", as in rounds 2–3. World and ratings kept.

## Slices (R4-n), in order
| # | Slice | Note | After |
|---|---|---|---|
| 1 | Sudden death | 1 | – |
| 2 | "No room" event | 10 | – |
| 3 | Log grouping rules 1–4 | 4 | – |
| 4 | Tier colours | 9 | – |
| 5 | Gift hover inspect | 7 | – |
| 6 | Per-turn totals, no duplicate chip | 6 | – |
| 7 | Additive Awoken engine | 8 | – |
| 8 | Archetype field | 3 | – |
| 9 | Music | 5 | – |
| 10 | Battle line of 8 | 10 | 2 |
| 11 | Compact battle cards >5 | 10 | 10 |
| 12 | Turn-end summary hold | 6 | 6 |
| 13 | Log turn folding | 4 | 3 |
| 14 | Fix the 14 Awoken forms | 8 | 7 |
| 15 | Archetypes, cuts and changes | 3 | 8, 14 |
| 16 | Beat clock and dance | 5 | 9 |
| 17 | Battle steps on the beat | 5 | 16, 12 |
| 18 | Meta check and retune | 1, 3, 8, 10 | 1, 10, 14, 15 |
| 19 | Playtest, fixes, report (orchestrator) | all | all |
