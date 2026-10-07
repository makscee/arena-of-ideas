# Battle viewer notes (notes 5 visual, 6, 7, 9)

Scout, against f9243387 (code only: the live build is invite-only).

## How a fight plays
- `beatPlayOf` (`src/mvp/trace.ts:662`) splits the fight into beats (a strike or turn end plus what it sets off, in waves); each has `turn`, waves, and its log range (`trace.ts:394-405`). Timing `trace.ts:404-524`: waves 220 ms apart, a beat 1.3–2.2 s, 0.9 s when quiet; kills, big hits, summons, first fatigue and last blow add time; a quiet beat repeating the turn before plays at 0.7.
- Player: `mobile/screens/battle.ts:288-315`, timer divided by `speed` (1/2/4×, line 65); each wave calls `render()`; animations keep their place with a negative delay. Sounds: one per wave from `beatCues` (`ui/sound-map.ts`) in `schedule()`. Beams: `drawBeams` ~1161-1240.
- On each card: cause badge above (`triggerBadge` 763-799); floating merged number above (`motion` 949-969, `mergedFloat` 996-1022, fades 0.9 s, `style.css:327-348`); **change chip on the emoji** (`changeBadge` 629-641, added 614 and 867; `.bv-changes { top:14px; height:44px }` `style.css:369`, desktop 843); status chips at the foot.
- **The duplicate Maks saw** is the change chip: the same "−5" as the floating number. It is also the tap target that opens the trace.
- Dead units: `deadCard` (852-873) keeps a greyed ✝ card until its beat ends; next beat it goes and the line slides.

## 6. Turn-end summary
- Pure `turnSummaryOf(log, beats)` in `trace.ts`: per unit per turn, damage taken, healing, PWR/HP change, shield blocked, net status stacks, died. Changes carry `unit`, `kind`, `eventId`: no server change.
- After a turn's last beat, `schedule()` holds ~1.2 s at 1× (divided by speed; skipped at 4×). Floating numbers, badges and beams hide; each card shows one label with the turn's totals ("−7 · +2 · 🛡×3"); the dead show greyed with theirs. Then the dead drop out (existing slide) and labels clear.
- Pause keeps the summary; ←/→ treats it as a step after a turn's last beat; End, Replay, key moments skip it; `controls.dataset.planMs` (line 273) includes it (tests pin it). Reduced motion: the caption lists the totals.
- Stop drawing the change chip during playback; keep it as a tap target (or a dot) when paused or stepping.

## 7. Gift menu
- `mobile/main.ts:958-1000` (`openGift`), a `dismissable` modal (`ui/dom.ts:224`); tapping a card stacks `unitSheet` (`ui/card.ts:142`).
- Desktop shop reads on hover (`main.ts:585-588`, `renderInspector` 648-690), but the gift modal covers the inspector. Give the modal its own read pane beside the three cards: hover fills it with `unitSheet(mine ?? u)`, first card by default. Phone keeps tap.

## 9. Tier text
- `ui/card.ts:48`, style `style.css:110` (9px serif, `color-mix(--text 75%, --dim)`), overrides at 62 (bench) and 600 (desktop). Dark theme only. `--gold` is price/buff: not for tiers.
- Tokens `--tier-1..4`: I `#a8b3c7`, II `#3fdc8f`, III `#5aa7ff`, IV `#c08cff`, class `.tier.t1..t4`, 10–11px. Same tokens on Codex tier chips (`screens/codex.ts:261`).

## 5. Cheap dance
- Animate `.card .emoji` only (the card's `transform` is used by lunge, shake, pop: `style.css:295-308`). `--bpm`, `--beat: calc(60s / var(--bpm))`; keyframes `translateY(-2px) scale(1.04, .96)`; opposite phase for alternate slots (the 2023 game flipped on even slots).
- `render()` rebuilds cards every wave, restarting animations: set `--bob-phase = -(clock % beat)` on the screen root on every render. Clock: `performance.now()` until music, then `ctx.currentTime - musicStart` (`ui/sound.ts:41`).
- Transform-only, compositor: cheap on phones. Off under reduced motion and on a hidden tab; no `will-change` on every card.
