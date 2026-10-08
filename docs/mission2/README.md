# Mission 2: players' ideas become units

Card: makscee/void-board#735. Plan page: https://m1.twin-pogona.ts.net/r/arena-m2-plan.html. Concept §7–8: https://m1.twin-pogona.ts.net/r/arena-concept.html.
**Approved 2026-10-08: "agree, go"** (Maks), with all three recommendations and the go for the prod-data slices (M2-1, M2-2, M2-10).
This file wins over the scout notes beside it (`pool.md`, `pipeline.md`, written against 9ef07ac, so line numbers may move).

## Answers
1. **Claude reads ideas**, through Claude Code on m1 (`claude -p`, as `src/create/claude-code.ts` already does), in small batches every few minutes. Player text goes to Anthropic; it is never shown publicly. Behind one interface, so an API key (Haiku) or the local Qwen can replace it later.
2. **A unit leaves only when an entrant takes its place.** The pool never shrinks. Bots never submit ideas on the live game.
3. **Evolution is the next mission.** This one leaves the library it needs.

## Calls (Maks can overturn any)
- The idea rate is 1 per 3 finished runs, hold at most 3. Minimum stay is 2 weeks. 1–3 units enter per day. All tunable (`MvpRules` or a tunables file).
- Your vote never counts for your own idea. One vote per player per pair.
- An idea that fails the simulation goes back to its author with the reason, and the idea is refunded.
- Unit ids are permanent and never reused. Two ideas with the same name get distinct ids.
- The 73 live units are the seed. The 8 cut in round 4 go to the library.
- **Out of scope:** evolution, logins, Russian, Discord, website stats, money.

## The flow
1. **Earn:** 1 idea per 3 finished runs, hold 3. Home shows "💡 2 ideas".
2. **Write:** a text box, stored privately, with a length limit.
3. **Archetype:** the model proposes 3: name, emoji, and a line of ≤15 words. Each must pass `archetypeProblems` and the crude checks.
4. **Reading:** 3 readings as When → Who → Does, built only from the grammar in `src/mvp/units.ts` (`WHEN`, `WHO`, the Does words), each with its awoken form. Each must pass `formProblems`, `awokenKeeps`, R1/R2/R4 and the shape-uniqueness check. "Can't express" goes to a new-words log.
5. **Simulation (overnight batch, plus a dev button):** a tuner sets the numbers for both forms with `breaker()`. A unit passes when:
   - its field score at its best is in a band;
   - Awoken is clearly better than sleeping;
   - the full meta check with the unit added stays HEALTHY.
6. **Votes:** either/or cards with a coolness score and a novelty bonus. A candidate must be preferred over a typical live unit.
7. **Enter at 04:00, after the playoff:** entrants replace the least played units that have stayed at least 2 weeks. Leavers go to the library. Nothing is deleted.

## Slices (M2-n)
| # | Slice | Size | After | Touches prod data |
|---|---|---|---|---|
| 1 | Units as data: tables for units, pool snapshots, stints and per-day tallies, seeded from `ROWS`; no visible change | M | – | yes |
| 2 | A pool change ends nothing: runs pin their pool; ghosts, Crown, playoff and champion work across pools; bots stop replacing a human champion; the client refetches; stats per day | M | 1 | yes |
| 3 | Earn ideas | S | – | |
| 4 | Write an idea | S | 3 | |
| 5 | Read the idea (the model call, checks, new-words log) | M | 4 | |
| 6 | Pick screens | M | 5 | |
| 7 | Numbers by simulation (tuner + overnight batch) | L | 5 | |
| 8 | Votes | M | 7 | |
| 9 | Credits, NEW badge, Library tab, creator number, counted fusion total | S | 1 | |
| 10 | Rotation at 04:00, starts switched off, dry-run on a DB copy | M | 2, 8, 9 | yes |
| 11 | Bots in the loop (API bot and phone e2e walk the idea path) | M | 6, 8 | |
| 12 | Playtest, fixes, report (held by the orchestrator) | M | all | |

**Prod-data rule (M2-1, M2-2, M2-10):**
- Migrations only add tables or columns, never drop or rewrite.
- A migration test runs against a copy of a pre-migration DB.
- The orchestrator copies the live DB file before each deploy.
- M2-2 must merge before anything changes the pool's content version, or the deploy ends every run in progress.
- M2-10's switch stays off until a dry run on a copy of the live DB is clean.

## How to try (the whole mission)
1. Finish 3 runs, or use the dev menu's "+1 idea". Write an idea on Home.
2. A few minutes later, pick an archetype, then a reading.
3. In the dev menu, tap "Run the overnight check now", then vote on a few cards.
4. In the dev menu, tap "End the day now". Your unit is in the shop, marked NEW, with "idea by @you".
