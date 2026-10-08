# Engine notes (notes 1, 4, 10)

Scout, against f9243387. Experiments were env-flag hacks on a copy, 6,000 late-game fights (meta-health breaker teams + random fused lines, 3–5 copies).

## 1. How battles end today
- `TURN_CAP = 200` in `src/battle.ts:34`, but MVP rules set `turnCap: 30` (`src/mvp/contract.ts:94`, passed in `src/mvp/fight.ts:74`): a draw with `BattleEnd.timeUp` (`battle.ts:173`).
- Fatigue from turn 10, +1 a turn (`battle.ts:30-43`); every living unit takes a `Hurt` at turn end (`:162`, `:734-741`). It goes through interceptors: Shield absorbs, Blessing cancels death and heals, heal-on-hurt units fire on it.
- Chain cap 32 (`battle.ts:38`, `:318-331`) stops one cascade; ~3% of battles hit it.
- With the cap lifted to 200: median 12 turns, p90 15, p99 19; 0.1% never end even at 190 fatigue.
- Stall engines: Blessing re-armed every turn (War Drummer+Divinity fusion); two Necromancer-types reviving each other; Sexton+X re-summoning a Ghoul on every ally death.

| Variant (cap 200) | Never end | Max turn | Draws | Winner changed |
|---|---|---|---|---|
| baseline | 6 | 200 | 213 | – |
| doubling from 10 | 6 | 200 | 508 | 10.7% |
| doubling from 10, pierces Shield/Blessing | 1 | 200 | 800 | 17% |
| linear, pierces, no summon/revive | 0 | 22 | 510 | 13.5% |
| **linear to 19, sudden death from 20** | **0** | **21** | **239** | **0.5%** |

Sudden death (chosen): doubling 20, 40, 80… from turn 20, pierces Shield and Blessing, Summon and Revive do nothing. Codex/glossary text: `src/codex.ts:242-257`.

## 10. Battle size and Mortwrought
- `TEAM_SIZE = 5` (`battle.ts:29`) limits both the team (`:185`) and room for Summon (`:524`) and Revive (`:549`). Full line → the summon/revive does nothing and logs nothing. Run side: `rules.lineSize` (`src/mvp/run.ts:137`).
- Mortwrought is most likely Sexton+Necromancer. Fusion = When of the first, Who of the second, Does of both in order (`src/mvp/forms.ts:99-105`): "ally dies → [Call Ghoul, Revive 2 + Call Imp]". One slot frees, the Ghoul fills it, the rest finds no room. 300 fights: ~5,400 Ghouls vs 419 Imps; with 8 slots 1,534 Imps and 960 revives. Its own Ghouls' deaths re-trigger it (~18 summons a fight).
- Change: a battle-only limit (`BattleInput.lineCap`, `MvpRules.battleSize: 8`) used at `:524`/`:549`; team stays 5. Log a "No room" event when a summon or revive fails.
- UI: phone row widens (`mobile/screens/battle.ts:1096`) but 8 cards at 360px ≈ 37px each (now ~62px): compact card or second row. Desktop both lines share one row (`style.css:817-821`): ~318px a side at 1024px → smaller `--cw` or collapse the panel. Check `.slots repeat(5)` (`style.css:42`).

## 4. Event log
- Log tab = desktop side panel, one row per wave (`mobile/screens/battle.ts:435-460`). Waves from `beatPlayOf` (`src/mvp/trace.ts:662-722`), which already merges one firing's fan-out.
- Longest battles: ~5,400 events, ~1,300 rows, ~68 rows a turn in loops. Worst: "NurseDivinity → Shield ×1 on [5 names]" 496× in one battle; Poison ticking on each unit with a "fades" row each; Blessing saves as 3 rows; five names written out each time.
- Rules: R1 same actor + same effect within a beat merges (sum, ×N): 1,321 → 492 and 1,287 → 548 rows. R2 one row per status tick across units ("Poison ticks 5 units, −4 each, 4 blocked"), fade folded in. R3 a Blessing save is one row. R4 "all allies"/"all enemies" instead of the names. R5 a turn whose rows repeat the previous turn folds into a summary (damage, heals, deaths) that expands on tap.
