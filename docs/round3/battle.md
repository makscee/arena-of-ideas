# Round 3, battle notes: 2, 10, 11, 14, 15, 18, 19

I scouted the live build (15a93ba9) in `/Users/admin/Work/arena-574/scout-r3`. Paths below are relative to it.

Supporting files are in `scratchpad/v3/`:

- `sim/r3-battle.mts` plays 4000 random battles: 5 pool units a side, 1 to 3 copies each, so some are awoken. Run it with `node_modules/.bin/tsx sim/r3-battle.mts 4000`; it takes about 10 s.
- `sim/battle-front*.ts` are copies of `src/battle.ts` with summons (and revives) put at the front. The sim uses them; they are not meant to ship.
- `mockups/battle-targets.html` is the target-beam mockup (note 11). Add `?f=hit` or `?f=chain` to pick a frame. It uses a copy of `style.css` and the real icons. The screenshots are `targets-{hit,chain}-{desktop,phone}.png` (1440x900 and 390x844), and `build.mjs` rebuilds the page and the screenshots.

---

## 2. Summon at the front, so it acts at once

**Problem.** A summoned unit joins at the back of its line. It fights only after everyone in front of it has died, so it often does nothing.

**Today**

- Kernel `src/battle.ts:665` (revive) and `:668` (summon) add the unit to the end of the line with `lines[side].push(id)`. A full line of 5 skips the summon (`:513`).
- `src/board.ts:93-101` replays this from the log and also pushes to the back. The log does not record where the unit went: both sides simply assume the back.
- The text says "back" in several places: `src/glossary.ts:132` (effect:summon tip) and `:134` (Revive tip), `src/replay.ts:161`, `src/describe.ts:352` (comment), `src/types.ts:145` (comment).
- Tests pin the back: `src/board.test.ts:116` checks "a summon enters at the back". The golden log `src/__fixtures__/golden-battles.jsonl` has 24 Summon events.
- Units that summon: Planter (battle start: Imp / Treant), Summoner (dies: Wolf / Wraith), Sexton and Morbid (ally dies), Fungoid (turn end). Units that revive: Necromancer and Divinity.

**The change**

- A summon goes to the front of its line, at index 0, in front of every living ally. If several units are summoned in one cascade, each one goes in front of the last, so the newest is at the front. A full line still skips the summon.
- The two units that strike each turn are chosen when the turn starts (`battle.ts:131-149`), so a unit summoned mid-turn does not strike in that turn. It is the front at the next turn start. Abilities that target "front" hit it straight away.
- The Summon event gets a new field `front: true` (`types.ts:198`). The kernel sets it, and `board.ts` puts the unit at the front only when the field is there. Battles already stored in `mvp_battles` have no field, so they still replay with summons at the back. Without this rule, every old replay would draw its lines wrongly.
- Revive stays at the back unless Maks picks otherwise (decision below).
- Texts: "Adds a new unit at the front of the line, if the line has room (5 max)." Fix `replay.ts:161` and the comments in `types.ts:145` and `describe.ts:352` to match.
- Client: the board redraws from `boardAt`, so the unit simply appears in the front slot. Give it the summon emphasis from note 14: it slides in from the clash side and the line shifts back one slot.

**What it does to balance** (`sim/r3-battle.mts`, 4000 random battles, units' score base → front)

| | base | summons at the front | summons and revives at the front |
|---|---|---|---|
| Sexton | 46% | 58% | 58% |
| Morbid | 55% | 69% | 65% |
| Fungoid | 48% | 53% | 53% |
| Planter | 44% | 45% | 46% |
| Summoner | 41% | 44% | 43% |
| Necromancer | 54% | 53% | 67% |
| Divinity | 60% | 59% | 74% |

- With summons at the front, 7.8% of battles change winner; with revives at the front too, 12.3%.
- Sexton and Morbid gain the most: a Golem (2/6) in front shields the whole line. After this ships, rerun `npm run mvp:meta` and tone Sexton and Morbid down (for example Morbid's Golem from 2/6 to 2/4) if they take over the meta.
- Fungoid becomes a wall: it puts an Imp in front every turn end.

**Tests**

- Kernel: Planter's Imp is the front in turn 1's PairFaced. A Summoner killed by a strike leaves its Wolf as the next turn's front. Two summons in one cascade put the newest at the front. A full line still skips the summon.
- Board: invert `board.test.ts:116`. Add a test that an old log without `front` still replays with the summon at the back.
- Refresh the golden logs (`golden-battles.jsonl`, `golden-chains.jsonl`) with `vitest -u` and read the diff: only the Summon events and what follows them should change.
- Rerun meta-health and commit the report.

**How to try.** Put a Planter in your line. At battle start, the Imp or Treant appears at your front and strikes in turn 1.

---

## 19. Chain cap 32

**Today.** The cap is 64 in two places: `DEFAULT_CHAIN_STEP_CAP` (`src/battle.ts:35`) and `MVP_RULES.chainStepCap` (`src/mvp/contract.ts:70`). `fightLines` passes the rules' value (`src/mvp/fight.ts:73`).

- The cap is enforced in `Engine.settle()` (`battle.ts`, about line 296). Each settle drains one cascade, which is everything a strike, turn start, turn end or fatigue sets off. After `chainStepCap` trigger firings it drops the rest of the queue and logs a ChainCapped event.
- The viewer shows it as one step: "Chain capped after 64 steps" (`src/mvp/trace.ts:644`). Replay text is at `src/replay.ts:209`.

**What hits it.** Group reactions to group events. For example, an "ally is hurt" healer next to an area hit fans out n² (see the comment at `src/mvp/units.ts:16`).

| | cap 64 | cap 32 | cap 16 |
|---|---|---|---|
| Random lines (my sim) | 0% | 0.13% | 2.75% |
| Winner changes from 64 | | 0 of 4000 | |
| Meta teams (`docs/mvp/meta-health.md`) | 0.13% | | |

At 32, expect a few tenths of a percent of meta battles to be capped.

**The change**

- Set `chainStepCap` to 32 in `MVP_RULES`, and `DEFAULT_CHAIN_STEP_CAP` to 32 so there is one number.
- Each run keeps the rules it started with (`run.rules`, `server/src/mvp/runs.ts:66`). Runs already in progress keep 64 until they end. New runs, and anything built from `deps.rules`, get 32.
- Optional and small: give the capped step the breaking-chain icon in its caption and a timeline mark, so a cut-off cascade is visible. The caption wording stays.

**Tests**

- `src/chains.test.ts:273` ("exploding cascade stops at the default cap") reads the constant, so it follows the change.
- Add a test that `MVP_RULES.chainStepCap === 32`.
- `src/mvp/fight.test.ts:31` still passes.
- Rerun meta-health. Its line "x% of them hit the chain cap" now means a cap of 32.

**How to try.** `npm run mvp:meta` reports the capped share. In a game it shows only in rare chain-heavy fights.

---

## 10. A cause icon on every action

**Problem.** When something happens on the board, the player can't always see why.

**Today.** `triggerBadge()` (`mobile/screens/battle.ts:668-706`) draws a pill above the unit: [When icon] → [Does icon]. It pops when its wave lands, stays for the rest of the beat, and opens Why when clicked.

The badge comes from `firingOf()` (`src/mvp/trace.ts:449`). That returns null for anything caused by the kernel or by a status, so only about 40% of waves get a badge. Over 309k waves in my sim:

| Wave | Share | Badge today |
|---|---|---|
| Unit ability | 39.7% | yes |
| Strike hit | 22.4% | no |
| Poison tick | 15.5% | no |
| Death | 10.7% | no |
| Fatigue | 8.6% | no |
| Freeze / Blessing | 1.7% | no |

On a short phone screen the pill shrinks to 14 px (`style.css:804-807`).

**The change:** every wave shows its cause.

1. Add a pure `causeOf(log, step)` in `src/mvp/trace.ts`, next to `firingOf`. It returns `{ at: unitId | "clash", cause: TermId (+status), effect: TermId }` for every wave:

   | Wave | Badge sits on | Badge shows |
   |---|---|---|
   | Unit ability | the unit | as today |
   | Strike | the striker | [crossed swords (trigger:Strike)] → [effect icon] |
   | Status ability: Poison tick, Freeze stopping a strike, Blessing saving a unit | the holder | [the status's icon] → [effect icon], e.g. Poison: [poison drop] → [spiky explosion] |
   | Fatigue | the clash mark on desktop, the caption on phone | hourglass |
   | Death | the dying card (its ✝ chip) | the killing wave's badge stays; a death never needs its own cause |
   | BattleEnd | | none |

2. `triggerBadge()` uses `causeOf`. The newest wave's badge is bright; earlier badges in the same beat stay at 55% opacity. They stay until the beat ends, as today.
3. The caption starts with the same cause icon, so the line of text names the cause too.
4. The card's own trigger mark (`.trig`, top left, `ui/card.ts:84`) flashes gold for 0.4 s when that unit's trigger fires. This ties the card's permanent icon to the moment it fires.

**Tests**

- trace.test: across 300 random battles, every wave except BattleEnd has a `causeOf`. A strike wave's cause is trigger:Strike on the striker. A Poison tick's cause is the Poison icon on its holder.
- e2e (mvp-phone): a strike beat shows a trigger-badge on the striker.

**How to try.** Play a fight and pause on any beat. Every card that did something shows a pill saying why.

---

## 11. Who targets whom: beams

**Research**

- Hearthstone: the attacker physically flies into its target, and targeted effects draw a projectile or beam from source to target. Its arrow is for the player's own targeting.
- Super Auto Pets: ability effects fly as small projectiles or particles from pet to target, and buffs land on the target.
- Marvel Snap: power changes fly as glowing particles onto the card.
- Slay the Spire: intent icons above enemies announce the next action. That is a different need: prediction rather than explanation.
- TFT: a projectile for every unit; it is busy and hard to read.

What the readable games share: the eye follows something that **travels from source to target**, the target **reacts at the moment it arrives**, and the colour says what kind of effect it is.

**Proposal: one beam per target, drawn on one SVG layer over the board.** See `mockups/targets-*.png`.

- **What is drawn.** For each wave, a curved line from the source card to each target card, in the effect's colour: damage `#ff6b4d`, heal `#3fdc8f`, Strength/buff `#ffa53d`, Shield `#5aa7ff`, Poison/Curse `#b48cff`, summon `#2ee6d4`. Other statuses use their glossary tone.
- **Shape of a beam.** A dot marks the source end. The beam is 4 px wide on desktop and 3 px on phone, with a soft glow. At the target there is an arrowhead and a small disc carrying the effect icon.
- **Motion.** The beam draws from source to target in 220 ms at 1x (stroke-dashoffset; scale it with `--bv-sp` like the other motion). The target's existing flash, shake and float start when the head arrives, so the effects shift +220 ms.
- **Same line** (an ally buff): the beam bows away from the board, above the line on desktop and on the phone's enemy row, below your row on phone, so it never crosses the cards. **Across the clash:** a gentle bow over the gap. **Self-target** (Who is "me", a Poison tick): no line, just a ring that pulses on the card.
- **Several targets:** a fan from one source, all drawn in the same 220 ms.
- **Chains:** each wave's beam starts at the unit that fired, which is usually the previous wave's target. A chain therefore reads as a path: Archer → Victim (red), then Victim → every ally (gold).
- **Beats.** Earlier waves of the beat stay as thin dashed lines at 32% opacity. Only the newest wave is bright. All beams clear when the next beat starts.
- **Paused or stepping** (←/→): the beams of the waves landed so far show without motion. A screenshot or a paused board therefore always shows who hit whom.
- **No source unit** (fatigue): the beam starts at the clash mark.
- **Strikes** get a short beam too (front to front), on top of the existing lunge.
- **Reduced motion:** static lines and no travel, as in the paused case.
- **Data:** the source is `step.actor` (or `causeOf().at` from note 10), the targets are the unique `step.changes[].unit`, and the kind comes from the change. The kernel needs no change.
- **Drawing:** after `render()`, read each card's `getBoundingClientRect()` (cards carry `data-unit`) and draw into one fixed `<svg class="bv-fx">` with `pointer-events: none`, under sheets and the end card. Redraw on resize. On phone the beams cross the caption band; that's fine because they fade within the beat.

**Tests**

- Unit test a pure `beamsOf(step, sides)` that returns `[from, to, kind]`: a fan for an all-allies buff, none for a self-target, the clash for fatigue.
- e2e desktop and phone: during an ability beat, `.bv-fx path` exists and its end points sit inside the target cards' boxes. Under `prefers-reduced-motion`, no `.bv-fx` animation runs.

**How to try.** Watch any fight. Every hit or buff draws a line from who did it to who got it, and pausing keeps the lines.

---

## 14. Pacing: a little slower, with weight on the big moments

**Today**

- `src/mvp/trace.ts:399-406`: waves 150 ms apart, squeezed so the last lands within 700 ms. A beat lasts 1000 ms, at most 1400 ms. A quiet beat (one wave of plain hits or statuses) lasts 700 ms. The last wave holds 700 ms.
- `battle.ts`: the line-up shows for `LINEUP_MS` 400; motion runs for `MOTION_MS` 1000.
- Battles default to 2x from round 4 (`FAST_FROM_ROUND`, `battle.ts:59`), and the speed choice is remembered (`arena.battleSpeed`).
- CSS motion (`style.css:215-248`): lunge 0.32 s, shake 0.34 s, float 0.7 s, dying pop 0.5 s, badge 0.3 s.
- Measured in my sim: a median battle is 27 beats, **25.4 s at 1x** (p90 39.7 s). At 2x, the default from round 4, that is **12.7 s**, with a quiet hit taking just 350 ms.

**The change**

| Constant | Today | New |
|---|---|---|
| `WAVE_MS` | 150 | 220 |
| Wave span (BEAT_MAX − HOLD) | 700 | 1400 |
| `BEAT_HOLD_MS` | 700 | 800 |
| `BEAT_MS` | 1000 | 1300 |
| `BEAT_MAX_MS` | 1400 | 2200 |
| `QUIET_BEAT_MS` | 700 | 900 |
| `LINEUP_MS` | 400 | 900 |

- **Emphasis** (added to the beat at 1x):
  - **Kill** (24% of beats): +350 ms. The card freezes for 120 ms when the killing blow lands, then the pop is bigger (scale 1.25), a skull bursts, and the dead card dims.
  - **Big hit** (≥4 damage, the timeline's `BIG_HIT_MIN`; 28% of beats): +200 ms, the float 1.4x larger, the shake 6 px instead of 4.
  - **Summon or revive** (4%): +250 ms, and the card slides in.
  - **First fatigue beat:** +500 ms and a "Fatigue" banner at the clash.
  - **Last beat** (the deciding blow): +600 ms before the end card.
  - **Crown fight:** a 1.2 s "Crown fight" title over the line-up.
  - Awakening happens in the shop, so its emphasis belongs to the shop notes.
- **Motion:** lunge 0.42 s, shake 0.40 s, float 0.9 s, badge 0.35 s, plus the beam's 0.22 s from note 11. Raise `MOTION_MS` to 1300 and `SHAKE_END_MS` to match.
- **Result** (sim): median **38.5 s at 1x** (p90 57.3 s), **19.3 s at 2x**. That is about 1.5x today's length, with the extra time spent on kills and big hits rather than −1 trades.
- `timingOf(beat)` takes the weight: kill, big, summon, fatigue-first, last.

**Tests**

- trace.test: `beatTiming` and `timingOf` numbers. A kill beat is longer than the same beat without its death. A quiet beat stays at 900 ms.
- The e2e plan check `controls.dataset.planMs` (beat-frames.mjs) keeps passing: it reads the plan, not constants.
- beat-frames: no float is cut short.

**How to try.** Play rounds 1 to 4. A −1 trade is quick, a kill lands with a beat of stillness, and the board can be followed at 2x.

---

## 15. Clicking a unit in battle: its statuses now

**Today.** A card's click (`battle.ts:565` and `:735` → `openUnit`, `:708-714`) pauses and opens `unitSheet`: the unit as it **entered** the battle, with base stats and its form text.

- Current statuses show only through the "+n" chip, which appears when a card holds more than its two rows of chips. That opens `liveStatuses(u)` (`:648-666`): the icon, stacks and tip for each status.
- A summoned unit has no `BattleUnit`, so clicking it opens the trace of its last change, or nothing at all.

**The change:** one "Now" sheet for every card in battle (`closable`, so Esc closes it, see note 9).

- **Header:** emoji, name, side tag ("You" or "Them"), and "Turn 5 · this beat".
- **Stats line:** `PWR 3 (base 2) · HP 4 / 6`.
- **Statuses:** reuse `liveStatuses`: each status's icon, stacks and tip. Add "Silenced: its ability is off" when the unit is silenced. When it has none: "No statuses."
- **Ability:** the form text (`formRich`, with its trigger icon). For a summon, look its body up by name in the content, which is the same lookup note 3 needs for the summon card.
- **Entered as:** a dim last line with the base stats.
- Clicking a status chip on the card opens the same sheet. The "+n" chip keeps opening it too.
- On desktop the sheet is the same overlay as today. A live "Unit" tab in the side panel that follows ←/→ would be nicer; it goes in Next, not this slice.

**Tests**

- e2e desktop and phone: pause on a beat where a unit has Shield, click it, and the sheet lists Shield with the stacks its chip shows.
- Clicking a summoned unit opens the sheet, not a trace.
- Esc closes the sheet and playback stays paused.

**How to try.** Pause on any beat and click a unit. Its live PWR/HP and every status it has at that moment are listed, each with what it does.

---

## 18. The Why panel: Log first, Why only when there is something to show

**Today.** On desktop the side panel opens on Why: `let tab = "why"` (`battle.ts:161`).

- With nothing traced, Why shows only a hint (`whyHint()`, `:376-378`: "Click any number, trigger badge, the caption or a Log row…"). That is why Maks sees it empty and selected from the start.
- The Log fills only when you switch to it (`setTab`, `:370`; `drawLog`, `:415-422`).
- The phone has no tabs: Why is a sheet that opens only with a trace. The phone is unaffected.

**The change**

- `tab` starts on "log". The Log follows playback; it already scrolls the current row into view (`:421`).
- The Why tab is disabled (`disabled`, dim, title "Click a number, badge or log row to see why") whenever nothing is traced (`trace === null`).
- Opening a trace (a chip, a badge, the caption, a log row, Why I lost) enables Why and switches to it, as `openTrace` already does.
- The Why sheet's ✕, Esc, or playing on (▶ clears `trace`) disables Why again and returns to Log.
- Delete `whyHint()`.

**Tests.** `e2e/mvp-desktop.mjs:416-427` needs a small update:

- The battle opens with the Log tab on.
- Why has `disabled`.
- A log row's click turns Why on.
- ✕ (or Esc) returns to Log, with Why disabled again.

**How to try.** Open any fight on desktop. The Log runs alongside the battle, and Why lights up only after you click something.

---

## Slices (about 1 hour each), in this order to avoid conflicts in `battle.ts`

1. **Summon at the front** (kernel): `battle.ts`, `types.ts`, `board.ts`, texts, tests, golden refresh, meta-health rerun. Independent.
2. **Chain cap 32** (kernel/content): `contract.ts`, `battle.ts`, tests, meta-health. Run it after slice 1, since both refresh the meta report.
3. **Log first, Why on demand** (client): `battle.ts` and the desktop e2e. Size S.
4. **Unit inspect sheet** (client): the "Now" sheet. Its summon lookup is shared with note 3's slice: build it once, in whichever slice runs first.
5. **Cause on every wave** (trace and client): `causeOf`, the badges, the caption icon, the trigger-mark flash.
6. **Slower pacing with emphasis** (trace, client and CSS): timings, weight, motion durations.
7. **Target beams** (client): the SVG layer. It needs `causeOf` from slice 5 and the timings from slice 6.

## Decisions for Maks

- **Revive (Necromancer, Divinity):** back of the line as today (recommended: revives at the front push Divinity from 60% to 74% and Necromancer from 54% to 67%, and note 1 is redesigning Necromancer anyway), the front like summons, or the slot it died in.
- **Speed default after the slowdown:** keep 2x from round 4 (recommended; about 19 s per battle), always 1x unless chosen, or drop the speed control's 4x and add 1.5x.
- **Beam style:** a persistent beam with a travelling head (recommended: it reads when paused and in chains), a flying projectile icon only (Super Auto Pets style; it vanishes when paused), or lines only while paused.
