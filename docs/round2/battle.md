# Battle view: readability, fun, replay, cause chain

Scout of the MVP at build d8650aa3 (branch mission-574-mvp). I ran it locally (port 8902, throwaway db), watched 4 battles step by step at 1440x900 and 360x640, and measured pacing over the 957 battles that the local bot jobs played into that db.

- Frames: `v2/battle-shots/` (`desk-r1-*`, `desk-r4-*`, `phone-r2-*`, `phone-r6-*`: `rtNN` = real-time at 1x, `stepNNN` = one frame per step, `end`, `result`)
- Mockup: `v2/mockups/battle.html` (one responsive file; `?state=chain` opens the phone sheet, `?state=end` shows the summary). Screenshots: `battle-desktop.png`, `battle-desktop-end.png`, `battle-phone.png`, `battle-phone-chain.png`, `battle-phone-end.png`
- Scripts: `v2/scripts/watch.mjs` (plays a run and captures frames), `stats.mts` and `beats.mts` (pacing numbers)

## What makes it hard to read today

1. **Too long, all at one pace.** Playback is one event per 650 ms. A median battle is **82 steps, or 53 s at 1x**. The 90th percentile is 172 steps (1 min 52 s), and the longest was 546 steps (6 min). A "−1" trade takes as long as a kill or a fusion combo. Round 1 in my run was two Distractors trading −1 for 12 turns.
2. **Half the steps are status bookkeeping.** Across all steps: StatusApplied 37%, StatusRemoved 12%, ability hits 17%, strike hits 15%, deaths 7%, heals 6%. "Shield fades from Distractor (3 left)" gets its own beat, separated from the hit that caused it. One Coach firing at "all allies" plays as 4 separate steps.
3. **Nothing moves.** Each step re-renders the board. Apart from a gold ring on the actor, nothing shows who hit whom. The attacker doesn't move, there's no line to the target and no hit flash. The target only gets a small chip over its emoji, and the eye has to read the caption to learn what happened.
4. **The eye has no single place to look.** The meaning is in the caption between the two lines, the effect is a chip on a card, and the last 3 steps sit dimmed below. On desktop all of it is squeezed into a 456 px column, and two thirds of the screen is empty.
5. **Names, not sides.** "THEM Distractor strikes Distractor → −0 (1 absorbed)" is how a mirror match reads, and "−0 (1 absorbed)" means nothing to a new player. Shield, Vitality, Strength and Freeze are bare words in 10 px grey text ("Shield 3 · Freeze 1"), with no icon and no explanation.
6. **Noise on every card.** Win and pick rates (`W46% P11%`) sit on every battle card, so the battle shows meta stats at the moment they matter least. HP has no bar, so "1 / 4" doesn't read as "almost dead".
7. **Dead units vanish.** A unit fades for one step and is gone, so the line shifts left and you lose track of where things were.
8. **Fronts don't face each other.** Both fronts sit at the left edge of two stacked rows. That works on a phone, but on desktop the clash could happen in the middle.
9. **Replay is half there.** The result screen has a Replay button, but the battle's own end card only offers Continue. Pressing ▶ after the last step silently restarts from step 0. History battles can be watched, but there's no "replay from start" or "jump to the moment X died".
10. **The chain shows units, not triggers.** "Strength ×1 ← Victim ← Medic" plus "ability" / "strike, set off the one above" names who acted, but not which **trigger** fired ("after this unit is hurt"), and not what that ability does.

## Does each event already carry its cause?

**Yes.** Every kernel event has `causedBy` (the parent event id; null only for kernel roots) and `source` (`"kernel"` or `{unit, status?, ability}`, i.e. which unit's or status's ability made it). `src/mvp/trace.ts` already walks this: it jumps from a status tick to where the status was applied, and it stops at turn structure. So the cause chain needs **no kernel change** to work.

What's missing, and how to add it:

| Need | Today | Add |
|---|---|---|
| Which trigger fired | Derivable: the firing's first event has `source = ref`, its `causedBy` is the event that matched one of the holder's `triggers` (When). | **Small kernel stamp (recommended):** `AbilityRef.when?: number`, the index of the matched When, set in the reactor when it fires. That gives an exact answer, with no re-matching of patterns in the client. It costs ~5 lines in `battle.ts` plus a golden-log refresh. |
| What the ability does (Does text) | Unit defs are in `BattleRecord.teamA/teamB` (fused units included); summons only have name/hp/pwr in the log. | Client: look up the def by unit id; for a summon, by name in `MvpContent`. Text from `describe.ts` (`describeWhenSegments` already marks the trigger with `partRef {family:"trigger", kind:"Hurt"}`, and status names with a ref), which is the hook for icons and highlights. |
| Chain nodes for the UI | `traceOf` collapses to unit links + a text line. | New pure `chainOf(log, eventId)` in `src/mvp/trace.ts` that returns nodes: `change` (the clicked event) → `firing` (unit, side, When index, Does) → `cause event` (strike / hurt / death / status landed) → … → `root` (Turn N began / battle start / fatigue). Each node has an `eventId`, so clicking it **moves the playhead there**. |
| "What did this cause" (optional) | Not shown. | The reverse index `childrenOf(eventId)`; the same data, the other direction. Good for a "this Freeze stopped 3 strikes" line. Later. |

How it behaves: clicking anything in the battle (a floating number, a trigger badge on a unit, a status icon, the caption, a log row) pauses playback and opens **Why** (a right panel on desktop, a bottom sheet on phone). It shows a vertical chain, nearest cause first. Every highlighted word (Strength, Hurt) opens its glossary line, and every node jumps the board to that moment. See `battle-desktop.png` and `battle-phone-chain.png`.

## Improvements ranked by fun per effort

Effort: S ≈ a day or less, M ≈ 2–3 days, L ≈ a week. They are client-only unless marked.

| # | Change | Fun / clarity | Effort |
|---|---|---|---|
| 1 | **Play by beat, not by event.** Use `beatsOf()` (it's already in `src/beats.ts`): one strike or turn-end plus its whole cascade is one beat. The events inside a beat play as fast waves (~150 ms apart; same-wave hits land together; a cap of ~1.5 s per beat). A battle drops from a median of 82 steps to **22 beats ≈ 26 s at 1x** (p90 37 beats ≈ 44 s). Status removals and the stat changes they cause merge into their hit ("−2, Shield absorbs 1"). | Very high: removes most of the boredom | M |
| 2 | **Attacker lunge, target hit flash and shake, floating numbers** (−n red, +n green, +1 PWR gold) that rise and fade above the card. Deaths: a skull pop, then the card greys out and **stays in place** (a "graveyard slot") until the beat ends. | Very high: the board tells the story, the text confirms it | S |
| 3 | **HP bars on cards; drop W%/P% from battle cards** (keep them in the codex or unit sheet). Big PWR / HP numbers with icons. | High: you read "almost dead" at a glance | S |
| 4 | **Status icons with stack counts on the card** (game-icons.net: Shield, Vitality = heart-plus, Strength = biceps, Poison = poison-bottle, Freeze = snowflake…). Every keyword in captions and sheets gets its icon and colour and opens a one-line glossary. This shares a component with the shop and codex work. | High; it's what Maks asked for | S–M |
| 5 | **Trigger badge pops above the unit when its ability fires**: `[icon Hurt] → [icon Strength]`, in a trigger colour (purple). Clicking it opens Why. Trigger kinds get icons too (Strike = crossed swords, Hurt = broken heart, Death = skull, Turn end = hourglass, Battle start = flag). | High: makes combos visible and satisfying | S (+ the `when` stamp) |
| 6 | **Why panel / sheet with the full chain** (above): `chainOf()` + UI; each node jumps the playhead. | High: answers Maks's ask directly | M |
| 7 | **Controls: ▶/❚❚, beat back/forward, 1× 2× 4×, End (jump to result), Replay from start.** Desktop keys: Space, ←/→, R. Remember the speed per viewer (localStorage). Default to 2x after round 3 if the player always speeds up. | Medium–high | S |
| 8 | **Timeline under the board**: one block per turn, with marks for deaths (side-coloured skulls), big hits and fatigue start. Click or drag to scrub (`boardAt(log, id)` already makes scrubbing pure). | Medium–high: shows the shape of the fight and lets you jump to "where I lost" | M |
| 9 | **End-of-battle card**: VICTORY/DEFEAT, damage-by-unit bars for both sides, 2–3 **key moments** (biggest kill, longest combo, fatigue) that you click to watch, plus *Replay from start* / *Why I lost* (or *won*) / *Continue*. "Why I lost" folds in here instead of being a separate panel. | Medium–high: the payoff moment, and replay becomes obvious | M |
| 10 | **Desktop layout: the two lines face each other horizontally**, fronts in the middle with a clash mark, a log/why side panel on the right, controls at the bottom. Phone keeps the stacked rows (enemy on top, front at the left) with the same pieces, smaller. | High on desktop | M (shares the desktop shell with the other v2 work) |
| 11 | **Caption = one line per beat**, with side-coloured names (no YOU/THEM tags needed once names are coloured), keyword and trigger icons, and "why? ›". Mirror names get a slot number only when both sides have the same name ("Distractor²"). | Medium | S |
| 12 | **Ability beams**: a short line or arc from the firing unit to each target of an ability (an all-allies buff fans out). | Medium: very readable for AoE | S–M |
| 13 | **Reduced motion**: with `prefers-reduced-motion`, no lunge or shake. The target gets a ring, numbers appear in place, and beats hold a little longer. | Required | S |
| 14 | **Big-moment beats slow down** (a kill, a resurrection, a chain of 4+ firings) and trivial ones speed up (a −1 trade): pacing by weight. | Medium | S (after #1) |
| 15 | Sound (hit, kill, trigger chime), off by default. | Medium | M (assets) |

**Suggested first slice (about a week):** 1 + 2 + 3 + 4 + 5 + 7 + 13. That covers "hard to read" and "make it fun to watch". The second slice is 6 + 8 + 9 (why chain, timeline, end card with replay). 10 rides along with the desktop shell work. 11, 12 and 14 can follow.

## Replay

- The data is already there: each battle is stored whole (`mvp_battles`, `GET /battles/:id`), and the viewer is a pure function of `(log, playhead)`. So replay is purely a UI matter.
- Add: a **Replay from start** button on the end card and in the controls (↻); **End** jumps to the result instead of the current "Skip" that leaves the screen; history and playoff battles open the same viewer. Optional: a link per battle (`#/battle/<id>`) so a slay or a playoff final can be shared.
- Fix: ▶ after the last step should not silently restart (`play()` sets `at = -1`); it should show the end card.

## Mockup notes

`v2/mockups/battle.html` is static (no build) with CSS animations for lunge, hit, rise and pop, and a reduced-motion fallback. The units are illustrative, though the shown chain (Medic hits Victim → Victim's Hurt trigger → Strength) is the real chain from the MVP report. Icons are real game-icons.net SVGs (CC BY 3.0: Lorc, Delapouite, sbed, Zeromancer), inlined as a sprite; the game will need an attribution line in its credits. On the phone frame the "−1" and "+1 PWR" floats overlap because the static screenshot freezes both; in motion they are staggered.

## Questions (each with my recommendation)

1. **Pace:** play by beat, ~1.2 s per beat at 1x, 2x remembered per player? I recommend yes; it's the single biggest win.
2. **The `when` stamp in the kernel log** (`AbilityRef.when`), or derive the trigger in the client? I recommend the stamp: it's exact and cheap, but it touches golden test logs.
3. **Default speed in later rounds:** start at 2x from round 4? I recommend yes, with the choice remembered.
4. **Win/pick rates:** none on battle cards at all? I recommend removing them there (they belong in the codex or unit sheet as the "subtle hint" Maks asked for).
