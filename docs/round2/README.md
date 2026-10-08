# Arena MVP, round 2: the plan

Maks played the MVP (build d8650aa3) on 2026-10-06 and asked for a round 2. Six scouts studied his notes; he approved
this plan the same day. The slices are sub-issues of makscee/void-board#574 and merge into `mission-574-mvp`.

**Where this README and the notes disagree, this README wins:** it holds Maks's final answers.

## Maks's answers

- **Beating your own champion is a slay**, and a champion can be crowned again with another team.
- Fusions that hurt their own team stay. Bots keep slaying and can be crowned.
- **Rating: per-fight Elo, shown once per run** ([rating.md](rating.md)). Each rated fight (rounds and the Crown)
  scores K·(S − E) against the rating stamped on that ghost; K is 32 for a player's first 5 runs, 16 for the next 10,
  then 10. No slay bonus; the Crown is one fight. Giving up counts every heart left as a lost fight. BOT_RATING stays
  1000 until a day of real play.
- **Namer: Qwen3-4B-Instruct-2507 (4-bit) with a one-word prompt** ([namer.md](namer.md)). Existing names stay.
- **Shop grows 3 → 6:** 3 offers in round 1, 4 from round 2, 5 from round 4, 6 from round 7 (`sim/shop-curve.mts`).
- **The Crown's difficulty stays** for now; the final playtest measures it again.
- **A hit fully blocked by Shield still counts** as the trigger today called "is hurt". Keep the rule, but change the
  wording so it doesn't promise lost HP: "is hit" or similar. The glossary slice picks the best wording for every unit
  text and says why.
- Win and pick rates appear only as a subtle hint: one dim line in a unit's sheet, plus Stats and the Codex.

## Calls made for him (R4; he can overturn them)

- Desktop is the same client, switching layout at 1024px and wider. The phone layout keeps working.
- The title menu only pauses a run (no time limit); Abandon ends it.
- The fused unit takes the front-most of the two slots, so Swap changes only the recipe.
- Only a pair nobody has fused hides its name in the preview; a pair bots made shows its name and who found it.
- New fusion names are one word; two words only as the last try, never three.
- The kernel stamps which trigger fired (`AbilityRef.when`) so Why chains are exact.
- Battles default to 2× from round 4; the viewer's speed is remembered.
- The Codex takes over Stats' Units and Fusions tabs; Stats keeps records and champions.
- Icons: game-icons.net, CC BY 3.0, credited in the Codex ([icons/CREDITS.txt](icons/CREDITS.txt)).

## The slices

| # | Slice | Waits for | Notes |
|---|---|---|---|
| 1 | Your own champion is a slay | – | mechanics.md (1) |
| 2 | Per-fight rating, and giving up | – | rating.md, mechanics.md (3) |
| 3 | The shop grows | – | mechanics.md (2) |
| 4 | One-word names on Qwen3-4B | – | namer.md, mechanics.md (7) |
| 5 | Fusion preview: Swap, and no name until you fuse | 7 | mechanics.md (5, 6) |
| 6 | Glossary and icons | – | icons.md |
| 7 | Compact cards, the active form only, rates as a hint | 6 | ui.md, mockups/b-* |
| 8 | Highlighted keywords and triggers | 7 | icons.md |
| 9 | Desktop layout | 7 | ui.md, mockups/a-desktop-shop |
| 10 | Title menu and in-run menu | 2, 9 | ui.md, mockups/e-navigation |
| 11 | Codex | 8, 10 | ui.md, icons.md, mechanics.md (4) |
| 12 | One beat at a time, with motion | – | battle.md |
| 13 | Readable battle cards | 6, 12 | battle.md |
| 14 | Controls, replay, end card | 13 | battle.md |
| 15 | Why: what triggered it | 14 | battle.md, icons.md |
| 16 | Desktop battle | 9, 15 | battle.md, mockups/battle-* |
| 17 | Playtest, fixes, report (the orchestrator) | all | – |

The mockups are static HTML (`mockups/*.html`, open in a browser) with screenshots next to them. They show
illustrative units and numbers: match their layout and feel, not their data.
