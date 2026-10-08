# Arena v2: gameplay and flow changes, scouted in code

Scope: items 1–7 from Maks's playtest notes, read against `mission-574-mvp` @ d8650aa3
(a checkout of mission-574-mvp at d8650aa3). No servers were run. I ran two unit test files (both pass on
the base) and one scratch in-memory bot simulation (`scratchpad/v2/sim/shop-curve.mts`) to compare shop curves.

Sizes: S = under half a day for one attempt, M = about one attempt, L = more than one.

---

## 1. Beating your own champion team counts as a slay

**Today.** Three guards stop it, plus the copy that explains it:

| Where | What it does |
|---|---|
| `server/src/mvp/runs.ts:102` (decide) | writes the Slay row only if `step.fight.opponent.player.id !== run.player.id` |
| `src/mvp/run.ts:295` `slewChampion()` | false when the Crown opponent is the run's own player → no slay bonus in `ratingChange`, no `slays + 1` in `finish()` (runs.ts:177) |
| `src/mvp/day.ts:110` `playoffSlays()` | drops slays by `champion.player.id` → no playoff entry, and Home's `slayers` count skips them (`dayView`) |
| `mobile/main.ts` 281, 347, 456, 529–530, 580–583, plus Home hint ~241 | "(your own team)", "doesn't count as a slay", "the others try to beat it" |

**Checked for the same player as both champion and slayer, all fine:**
- Playoff (`playoffEntrants`, `playRoundRobin`): entrants are keyed by player id. The reigning team never defends in the playoff (only slayers play), so the champion player enters once with their best slaying team, like anyone else. If they win, `endDay` crowns their new team and `daysAsChampion` goes up again. If they are the only slayer, their new team takes the crown without a game.
- Ghost hiding (`pickGhost`, runs.ts:136): hides runs that slew today's champion and runs waiting at the Crown, by runId. Nothing there is keyed to the champion's player id. A player never meets their own ghosts anyway (`excludePlayerId`).
- `hiddenSlay` (day.ts:150) hides the battle from everyone except the slayer, and works the same here.
- The Crown opponent switch after a rollover (`currentCrown`) does no player check.
- Bots: each bot run gets a new player id, so this case only comes up for humans.
- The battle viewer labels sides as A and B, so both sides read "@maks". The copy could say "your champion team" instead (cosmetic).

**Spec.**
- `slewChampion(run)` becomes `run.endedBy === "crown-won"`. Keep the optional `player` parameter so callers still compile, or drop it.
- Remove the player check at runs.ts:102 and the champion filter in `playoffSlays` (keep the content-version filter).
- Copy: Home hint for the champion: "This is your team. Others try to beat it today, and so can you, with another team." Crown hint: "Beat your own champion team to be a slayer again." Result and run-over screens use the normal slayer text.
- Contract comments that say "the champion's own … is no slay": `contract.ts`, `src/mvp/day.ts:80–84`, `run.ts:290–294`.

**Risks.** Low. A champion could farm the slay bonus against their own team, but each slay costs a full 12-round run plus a Crown win. A mirror win (same line) also counts as a slay; it is harmless because the same team just keeps the crown.

**Tests to flip.** `server/src/mvp/day.test.ts:204` (now: a Slay row, the bonus, a playoff entry, crowned again with the new line), `src/mvp/day.test.ts:48` ("skips … the champion's own"), `e2e/mvp-phone.mjs:322–348` (the own-crown section expects "own champion team" and no "slayer today"), `e2e/mvp-own-crown.ts` (setup script; assert a slay now).

**Size: S.** Touches `src/mvp/run.ts`, `src/mvp/day.ts`, `server/src/mvp/runs.ts`, `mobile/main.ts` (copy only), tests.

---

## 2. Shop size grows with rounds (Hearthstone Battlegrounds style)

**HSBG facts** ([wiki.gg Tavern Tier](https://hearthstone.wiki.gg/wiki/Battlegrounds/Tavern_Tier), [Gold](https://hearthstone.wiki.gg/wiki/Battlegrounds/Gold)): offers are 3 at tier 1, 4 at tiers 2–3, 5 at tiers 4–5 and 6 at tier 6 (+1 at tiers 2, 4 and 6). Every minion costs 3. The tavern upgrade starts at 5 gold, each tier costs more, and the price drops by 1 each turn. Gold grows by 1 each turn (3 → 10). Players usually reach tier 2 on turn 2–3, tier 4 around turn 6–7 and tier 6 around turn 10 or later.

**This game today** (`src/mvp/contract.ts:47` MVP_RULES): `offers: 5` fixed, 10 gold every round (no carry-over), unit 3g, reroll 1g. Tiers open by themselves at rounds 1/3/6/9 (`tierOpensAt`), and the open pool grows 22 → 49 → 68 → 81 units. Offers are drawn uniformly from open units with replacement (`rollOffers`, `src/mvp/run.ts:68`). The old desktop kernel already had a curve, `shopSizeForRound` in `src/tunables.ts:278` (3 → 6), so this has been done before.

**Gold.** Gold, not offers, is the limit: 10g buys at most 3 units plus 1 reroll. More offers give more choice and better odds of finding a copy, not more spending. Bots end a shop round with about 2.9g left in every variant. No gold change is needed.

**Simulation** (scratch, in memory, 600 bot runs per curve, bots playing their own ghosts, `botDecision` unchanged):

| curve (offers by round) | awoken by R3 | by R6 | ever | runs that fused |
|---|---|---|---|---|
| today: 5 always | 27% | 90% | 97% | 77% |
| strict HSBG pace: 3/4/5/6 at R1/R3/R6/R9 (on tier openings) | 17% | 77% | 89% | 68% |
| **fast 3→6: 3 at R1, 4 at R2–3, 5 at R4–6, 6 at R7–12** | 21% | 88% | 94% | 76% |
| 4→7: 4/5/6/7 at R1/R3/R6/R9 | 23% | 86% | 95% | 76% |

The strict HSBG pace slows awakening and cuts fusions noticeably (12 rounds is short). **Recommended: the fast 3→6 curve.** It starts small, grows like HSBG, ends at 6, and keeps today's awakening and fusion pace within noise.

**Spec.**
- Contract: keep `offers` as the round-1 count (3) and add `offersGrowAt: number[]` (`[2, 4, 7]`, +1 at each listed round). This mirrors `tierOpensAt`. Add a pure `offersAt(rules, round)` in run.ts and use it in `rollOffers`.
- Runs store their own `rules` (`MvpRunState.rules`), so a run started before the deploy keeps its fixed 5. Read `rules.offersGrowAt ?? []`.
- Rules sheet text (`mobile/main.ts:76`): "3 offers in round 1, growing to 6 by round 7".
- Layout: `.slots` is a 5-column grid (`style.css:39`), so 6 offers would wrap. This belongs with the compact-cards / desktop layout work.

**Bots / meta.** `botDecision` (bots.ts:107) only reads `run.offers`, so it needs no change. `synthGhost` doesn't use offers. `scripts/meta-health.ts` builds teams directly at 3 copies and never runs a shop, so its verdict doesn't change. It doesn't measure reachability at all; the bot-run numbers above do. Ghost pools mix teams from before and after the deploy for a while, which is minor.

**Tests.** `src/mvp/run.test.ts:18` and `server/src/mvp/app.test.ts:34` expect 5 offers. Add an `offersAt` table test. The e2e phone run just buys `offer-0`, so it is fine.

**Size: S.** Touches `src/mvp/contract.ts`, `src/mvp/run.ts`, `mobile/main.ts` (one line), `mobile/style.css` (with layout).

---

## 3. Leave to the title menu, come back later, or give up

**Today.** Resume already works on the server. `GET /home` returns `activeRunId` and Home shows **Continue** (`main.ts:157`). `POST /runs` hands back the active run (`startRun`), and a run waiting at the Crown re-targets to the current champion after a rollover (`currentRun`). What's missing is a way out: the shop has no Menu or Home button, and you can only leave by closing the tab. There is also no way to give up a run.

**Spec: pause (main ask).** Add a **Menu** button in the shop HUD and on the result screen that goes to Home; **Continue** comes back. The run stays open indefinitely. A content change ends it on the next touch ("content-changed"), as today. No server change is needed.
- Nice-to-have: leaving mid-battle currently skips the result on return, because the fight is already decided server-side. On Continue, if the last fight's battleId isn't marked seen (localStorage), show its result/replay first.
- Deep links (`#run/<id>`, `#codex/...`, as the old desktop codex did with `#codex/unit/Necromancer`) would let a reload land back in the game. That fits the desktop-shell work.

**Spec: give up (abandon).** Add a "Give up run" option behind Menu and next to Continue on Home, with a confirm.
- Server: `RunEndReason` gains `"abandoned"`. `endRun(state, "abandoned")` (run.ts:252 already ends runs for "no-champion" and "content-changed"). New `abandonRun(deps, run)` in runs.ts calls `finish()` and the `onRunEnd` hooks. Route: `POST /runs/:runId/abandon` (the owner only, like decisions). A route keeps `Decision`, `botDecision` and preview untouched.
- **Hearts and rating: give up = lose all remaining hearts now.** For rating, count each remaining heart as a lost round fight. Today's formula is round-win share; without this, giving up at 3/3 wins would lock a perfect score. Giving up is then never better than playing on. State the rule this way so it fits whatever rating formula the rating agent picks. Giving up at the Crown scores like a lost Crown.
- Ghosts: teams saved at fights already fought stay in the pool; they were real fights. Nothing is saved for the unfought round. A run given up at the Crown stops being hidden, which is fine because there is no slay.
- Stats: `onRunEnd` counts it as a finished run (pick rate). Keep that; it matches out-of-hearts runs.
- Bots never give up.

**Tests.** runs.test (abandon ends, rating counts remaining hearts, 401 for another player, 409 on an over run), app.test route, run-over copy for "abandoned", phone e2e: Menu → Home → Continue.

**Size: M** (server S + client S). Touches `src/mvp/contract.ts`, `src/mvp/run.ts` (endRun, ratingChange), `server/src/mvp/runs.ts`, `server/src/mvp/app.ts`, `mobile/api.ts`, `mobile/main.ts`.

---

## 4. Codex of all units

**What exists.**
- `GET /content`: all 81 units with both forms, abilities and statuses. The client already caches it (`getContent`).
- `GET /fusions`: every discovered pair (name, parts, `discoveredBy`, `discoveredAt`, `nameSource`), unbounded.
- `GET /stats`: rates for units that have tallies.
- The Stats screen (`mobile/screens/stats.ts`) already has Units (rates), Champions and Fusions (newest 60) tabs.
- The fused recipe isn't stored, but it is pure: the client can build it with `lineUnitOf(u, uid, 3)` + `fuseUnits` from `src/mvp/forms.ts` (the client already imports kernel code).

So **no server change is required.**

**Missing:** keyword text. `StatusDef` (`src/types.ts:153`) has no description, and trigger kinds (When) have no glossary entries either. This needs a small shared module (`src/mvp/keywords.ts` or `mobile/ui/keywords.ts`): id, label, game-icons.net SVG, one-line meaning. That module is the same thing the keyword-highlight and icons work needs, so the Codex should read it, not author its own. game-icons.net is CC BY 3.0, so the Codex needs a credits line naming the icon authors.

**Spec.** Home → **Codex** with three tabs:
- **Units**: all 81 by tier. Each shows the active form (sleeping), with an Awoken toggle per Maks's "only show the active state" note. Rates are a subtle line in the detail view only.
- **Fusions**: every discovered pair with name, the recipe (When of 1st, Who of 2nd, Does of both), "discovered by @x / you / bots, unclaimed", a "mine" filter, and a count ("N of 6,480 pairs found"; 81×80 ordered pairs). Undiscovered pairs stay hidden.
- **Keywords**: statuses and triggers with icon and meaning, plus the icon credits.

Recommended: the Codex takes over Stats' Units and Fusions tabs. Stats keeps your records and the champion history. This fits "rates only as a subtle hint".

**Size: M** (L if it waits on nothing and has to build the keyword module itself). Touches a new `mobile/screens/codex.ts`, `mobile/main.ts` (Home button), `mobile/screens/stats.ts`, the keywords module.

---

## 5. Swap button in the fusion preview

**Today.** `fusePreview(first, second)` (`main.ts:362`) asks `POST /runs/:id/preview {kind:"fuse", first, second}` and shows the card plus `unitSheet`. Order is tap order. `fuseUnits` takes When (and condition) from first, Who from second, and Does from first then second (`forms.ts:88`). Each order is its own pair, with its own name and credit.

**Spec.** Add a ⇄ **Swap** button between Cancel and Fuse that re-runs the preview with the indices swapped. Both previews can be fetched up front so the toggle is instant (a preview writes nothing). Under the card, show a recipe line: "When · Taser → Who · Victim → Does · both".

**Catch, in the kernel.** The fused unit takes `line[first]`, then `splice(second)` (run.ts:190), so swapping also moves the result. With [A,B,C,D], fusing D+B gives [A,C,DB] but B+D gives [A,BD,C]. Recommended fix: the result takes the **front-most** of the two slots and keeps first's uid (`notify` finds it by uid, so hooks are unaffected). Then a swap changes only the recipe. Existing tests use first:1/second:0, where the result is already at 0, so they still pass. Update the contract/run.ts comment.

**Size: S.** Touches `src/mvp/run.ts` (fuse case), `mobile/main.ts` (fusePreview), a run.test case.

---

## 6. An undiscovered fusion shows no name until the fuse is confirmed

**Today.** `preview()` (runs.ts:183) names the fuse with `peekFusionName` (fusions.ts:578): the stored name, else the model name prepared ahead, else the portmanteau stand-in. The preview card and sheet show it. The tests assert preview equals decision (`app.test.ts:164` `done.json.run toEqual pv.json.run`, `fusions.test.ts:218`).

**Spec.**
- Server: in `preview()`, a pair with no stored `FusionDiscovery` gets `{ name: "", discoveredBy: by }`. Enforce it on the server so the name isn't in the network response either. `decide()` is unchanged: the fuse still takes the prepared or portmanteau name and records it. Contract note: "in a preview, a pair nobody has fused has name '' (named when fused)".
- Client: when `fused.fusion.name === ""`, show "??? · New fusion, named when you fuse" (card and sheet title). After the fuse is confirmed, show a reveal: "✨ You discovered <Name>" with the card. The client knows it's a first discovery from the empty preview name plus `discoveredBy` being you.
- Pairs that only bots have fused already have a public name (Stats/Fusions shows "made by bots, unclaimed"). Recommended: show the name and add "fuse it to claim the discovery".
- Link to the bigger-model idea: once the name is hidden until confirm, the reveal can cover a short wait. The fuse route could await the model for about 3 s for an unprepared pair before running the synchronous `decide()`. That belongs to the bigger-model work (it changes fusions.ts/app.ts); just don't block it.

**Tests.** Flip the two preview==decision assertions to "equal except the fusion name". Add: an unstored pair previews '', a stored one shows its name.

**Size: S.** Touches `server/src/mvp/runs.ts` (preview), contract comment, `mobile/main.ts` (fusePreview + reveal), tests.

---

## 7. Single-word fusion names

**Stand-in (portmanteau, fusions.ts:52).** Already one word: across all 6,480 ordered pairs, none has a space. The numeral fallback ("Sileat II") is a rare last resort. 80 pairs get inner capitals from two-word unit names ("WarDrhter", from War Drummer). Fix: title-case the result ("Wardrhter"), or take the head from the last word of a two-word name.

**Model names.** These are where the multi-word names come from:
- The prompt (`NAMER_SYSTEM`, fusions.ts:291) asks for "one or two words".
- The few-shot examples are 3 of 4 multi-word ("Fang Paladin", "Mender of Storms", "Alms Cutter").
- `cleanModelName` accepts 1–3 words.
- The screenshot shows "Ironclad Trophy".

Spec:
- Prompt: "a single word; a compound like Gravewing is welcome".
- Examples all single words, none a unit's name (a test checks this; e.g. Howlguard, Stormmender, Gravewing, Almsthief).
- Prefer, don't require: in `drainFusionNames`, a multi-word answer counts as "ask again" on tries 1–2 (MODEL_TRIES = 3), a two-word answer is accepted only on the last try, and three words never.
- Names already stored stay. The contract says a pair's name never changes, and renaming would be a prod-data change.
- Checking it against the real model needs the namer. Don't use m1:8792 from a worker. A stub covers the unit tests; Maks's session can rerun `mvp:fusion-names` on m1.

**Overlap.** The "bigger model for names" work edits the same file (`httpModelNamer`, prompt, `max_tokens`). Do both in one slice, or this one first.

**Size: S.** Touches `server/src/mvp/fusions.ts`, `fusions.test.ts`.

---

## Where these collide (order to avoid conflicts)

| File | Items (plus other v2 work) |
|---|---|
| `src/mvp/run.ts` | 1 slewChampion, 2 rollOffers, 3 endRun + ratingChange, 5 fuse slot; **the rating-formula work also rewrites ratingChange** |
| `src/mvp/contract.ts` | 1 comments, 2 MvpRules, 3 RunEndReason, 6 preview note |
| `server/src/mvp/runs.ts` | 1 decide guard, 3 abandonRun, 6 preview |
| `server/src/mvp/fusions.ts` | 7, bigger model |
| `mobile/main.ts` | 1 copy, 2 rules text, 3 Menu/Give up, 4 Codex button, 5 + 6 fusePreview; **the desktop version, compact cards, active-form-only sheet and keyword highlighting all rewrite it** |
| `mobile/style.css` | 2 (6 offers vs the 5-column `.slots`), desktop/compact cards |
| `e2e/mvp-phone.mjs` | 1, 3, 5/6; the desktop version will need its own e2e |

Order:
1. Server/kernel parts first, in parallel: 1 → rating formula → 3-server (give up scores through the new formula); 2; 5-kernel + 6-server; 7 (+ bigger model).
2. Client parts land after the desktop shell splits `main.ts` into screens (home/shop/result), so the copy for 1, the Menu for 3, and Swap/hidden name/reveal for 5–6 go into the new files instead of fighting the rewrite.
3. The Codex goes after the keyword module (icons + glossary) and compact cards exist.

## Slices

| # | Slice | Size | After |
|---|---|---|---|
| A | Own-champion slay counts | S | none |
| B | Growing shop (3→6 by R7) | S | none (the 6-card row fits with the desktop layout) |
| C | Menu, resume, give up | M | A; rating formula (scoring); desktop shell (client) |
| D | Fusion preview: swap, hidden name, reveal | S | desktop shell (client) |
| E | Single-word fusion names | S | none; pair with bigger-model work |
| F | Codex (units, fusions, keywords) | M | keyword module, compact cards, desktop shell |
