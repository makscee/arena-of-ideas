# Arena rating: a fair formula (Maks's point 3)

**Recommendation: rate every fight Elo-style against that ghost's rating, add the fights up, and show the total once at the end of the run.** K is 32 for a player's first 5 runs, 16 for the next 10, then 10. An abandoned run counts every heart left as a lost fight. Drop the slay bonus: the Crown is one more fight, against the champion's rating.

With this formula a 50% player stops drifting. New players reach their level in about 5 runs instead of 30+, and quitting a run never pays. A player can check any change by hand: against an equal opponent, a win is +5 and a loss is −5 (K=10). Beating someone stronger earns more, and losing to someone weaker costs more.

Simulation: `rating-sim.mjs` next to this file (`node rating-sim.mjs`, about 1.5 s).

---

## 1. What the code does now

`src/mvp/run.ts` `ratingChange`, with rules `ratingK: 32` and `ratingStart: 1000` in `contract.ts`:

```
actual   = clamp(round wins / round fights (draw = 0.5) + 0.25 if slay, 0, 1)
expected = 1 / (1 + 10^((1000 - before) / 400))      // vs the field, not the opponents
after    = round(before + 32 * (actual - expected))
```

Opponents come from `pickGhost`: a random team from the newest 200 saved at that round, at any rating. Ghosts don't store a rating. Bots never move.

Five problems:

1. **Drift down (the −1.3/run Maks saw).** A run stops at the 5th lost heart, so "wins / fights" underestimates a player's true win rate. Runs with an early bad streak end early with a low share, and lucky runs don't get extra weight for being longer. This is the known bias of inverse ("stop at the r-th failure") sampling. The unbiased estimate is (r−1)/(N−1), not r/N ([Haldane 1945, via Mendo & Hernando](https://oa.upm.es/21857/1/sr5_bis.pdf); [Springer: Inverse Sampling](https://link.springer.com/rwe/10.1007/978-3-642-04898-2_313)). The sim shows **−1.2 per run** for a 50% player.
2. **Too slow, and squeezed.** One run moves the rating at most ±16. A player of skill 700 is still at 947 after 10 runs and 906 after 30. Skill 700 and skill 1300 settle only 230 points apart (873 vs 1103).
3. **Opponents are ignored.** Expected is measured against "1000", whoever you actually fought. A run against strong late-round survivors counts the same as a run against bots.
4. **Abandon would be an exploit.** "Share of fights won" can be locked in early. Quitting whenever you're ahead (≥70% after 2+ fights) ends **+76** above an honest player of the same skill.
5. **The slay bonus is free rating** with no matching cost, so it pushes the scale up over time.

## 2. What existing systems do, and what fits

| System | How it rates | Fit for Arena |
|---|---|---|
| **Elo** ([Wikipedia: Elo](https://en.wikipedia.org/wiki/Elo_rating_system); [FIDE K = 40 / 20 / 10](https://en.wikipedia.org/wiki/Chess_rating_system)) | Per game: ΔR = K·(S − E), E = 1/(1+10^((Ropp−R)/400)). FIDE uses a big K for new players and a small K for veterans. | **Best fit.** Each round fight is one 1v1 game against a ghost with a known rating. Summed over a run, the stop rule can't bias it (see 3). |
| **Glicko-2** ([Glickman, glicko2.pdf](https://www.glicko.net/glicko/glicko2.pdf)) | Rating + uncertainty (RD) + volatility, updated once per "rating period". Works best with about 10–15 games per period. | Fits on paper: a run is a period of 5–13 fights. In the sim it converges fastest, but a new player's first run swings −294 to +353, it drifted −2.8/run, and it's hard to explain. Not worth it yet. |
| **TrueSkill / OpenSkill (Weng–Lin)** ([JMLR 2011](https://jmlr.csail.mit.edu/papers/volume12/weng11a/weng11a.pdf); [OpenSkill](https://arxiv.org/pdf/2401.05451)) | Bayesian μ, σ, built for teams and many-player matches. Leaderboards often show μ − 3σ. | Our fights are 1v1, where it comes down to Glicko-like updates. Showing μ − 3σ makes the number climb just by playing, which Maks doesn't want. Skip. |
| **Hearthstone Battlegrounds** ([Blizzard dev insights](https://hearthstone.blizzard.com/en-us/blog/23239989/); [Hearthstone wiki](https://hearthstone.fandom.com/wiki/Battlegrounds)) | Places you against all 7 others (56 pairwise win chances), with a "variance" that shrinks with games played. Blizzard also adds a small bonus per game below 6500, a deliberate upward drift. | Same idea as per-fight Elo with a falling K: pairwise expectations and fast-then-slow movement. We don't want their designed drift. |
| **TFT LP vs MMR** ([guide](https://boosteria.org/guides/tft-lp-mmr-explained-tft-ranked-really-works)) | Hidden MMR does the real rating; the visible LP is nudged toward it. | Two numbers mean more to explain. Only needed if we later want a seasonal ladder on top. |
| **Backpack Battles** ([Steam: ranking details](https://steamcommunity.com/app/2427700/discussions/0/4290313152637001687/), [matchmaking](https://steamcommunity.com/app/2427700/discussions/0/4626978969929966711/)) | Asynchronous snapshot ghosts matched by rank and current win/loss. Rating moves once per run by wins, and the same wins give fewer points the higher you are. No official formula. | Our model: one change per run, ghosts. Per-fight Elo gives the same "fewer points when higher" feel, with an exact formula. |
| **The Bazaar** ([Screen Rant](https://screenrant.com/bazaar-how-to-play-ranked-ccg/)) | Flat rank points per run: <4 wins −1, 4–6 wins 0, 7–9 wins +1, 10 wins +2. | Easy to read but a grind ladder, not skill: any above-average player climbs just by playing more. Rejected. |
| **Super Auto Pets** ([wiki](https://superautopets.wiki.gg/wiki/The_Basics)) | Arena (async, lives) is unrated. Versus ranked starts at 1500 and moves per win or loss. | Confirms: async lives runs are usually unrated, and the rated mode is per game. |
| Slay-the-Spire-style daily climbs | A score leaderboard per day, with no skill rating. | Our daily champion and slays already play this role. The rating is a separate, long-term number. |

**Why per-fight sums have no drift:** each fight adds K·(S − E), which averages to zero for a correctly rated player. A sum of zero-average steps still averages to zero however the run is cut short: 5th heart, round 12, abandon or content change. (This is the optional stopping theorem.) The current "share" divides by a run length that depends on the results, and that is where the bias comes from.

## 3. The formula

When a run ends, using the player's rating R at run start (the rating stays frozen during the run):

```
for each rated fight i (each round fight, plus the Crown if fought):
    Opp_i = rating stamped on that ghost (owner's rating at the start of their run;
            bots: BOT_RATING; Crown: the champion's stamped rating)
    E_i   = 1 / (1 + 10^((Opp_i - R) / 400))
    S_i   = 1 win, 0.5 draw, 0 loss
abandoned run: + one extra loss (S=0) per heart left, against this round's
               already-picked opponent
K  = 32 if runs < 5, 16 if runs < 15, else 10      (runs = Rating.runs before this run)
ΔR = round(K * Σ (S_i - E_i))
```

Constants: `ratingStart = 1000`, `ratingKSteps = [[5, 32], [15, 16], [∞, 10]]`, `BOT_RATING = 1000` (see bots below), `DRAW_SCORE = 0.5`. **No SLAY_BONUS in the rating.** Slays still count in records and send you to the playoff.

The run-end screen can show "Expected 6.1 wins, got 9 → +29": Σ E_i and Σ S_i are exactly the `expected` and `actual` fields `RatingChange` already has.

**Champion (Maks: beating your own champion is a slay).** The Crown is just a fight against the champion's stamped rating. Fighting your own champion means facing your own past rating: about ±5 at K=10, so there's nothing to farm. Beating a 1250 champion as a 1000 player gives +8, and losing to it costs −2.

**Abandon.** "Every heart left becomes a lost fight" equals the worst case of playing on, because playing on can only add wins on top of those losses. Quitting can therefore never beat staying. If "go to the title menu and come back" only pauses the run, nothing is rated until the run really ends.

**Bots (Maks: keep them).** A bot's fixed rating anchors the scale. Since gains shrink as you climb, farming bots runs dry: at 1300 a win over a 1000 bot is +1.5 and a loss is −8.5. The one risk is a bot label that doesn't match how strong bots really are. If bots are really 850 but labelled 1000 and fill half the pool, every human reads about +120 too high. Everyone shifts equally, so the ranking stays fair. Fix: once there is real play, set `BOT_RATING` to bots' measured strength, `mean(opponent human rating) + 400·log10(bot wins / bot losses)` over bot-vs-human round fights. That's one query over battles.

## 4. Simulation results

Model: 400 humans with true skill ~N(1000, 150) and 30% bot ghosts (true 1000, rated 1000). Each run's team is skill ± luck (SD 80). There are 12 rounds against random ghosts from the round's newest 200 (like `pickGhost`), 5 hearts, 5% draws, and a Crown against a champion of strength 1350 rated 1250. Cohorts of 40 start at 1000.

**Drift** (mean change per run for a player rated exactly at the pool's 50% level; noise ±0.2):

| current | current + stop-rule fix | per-fight K=16 | per-fight K=10 | **per-fight 32/16/10** | Glicko-2 |
|---|---|---|---|---|---|
| **−1.2** | 0.0 | +0.4 | +0.5 | **+0.2** | −2.8 |

**Convergence** (cohort average after 1 / 5 / 10 / 30 runs, then where it settles; noise = how much a settled rating swings run to run):

| method | skill 700 | skill 1300 | settled noise |
|---|---|---|---|
| current | 990 / 963 / 947 / 906 → 873 | 1013 / 1039 / 1050 / 1081 → 1103 | ±12–16 |
| current + stop-rule fix | 991 / 968 / 957 / 927 → 903 | 1013 / 1041 / 1054 / 1092 → 1121 | ±12–15 |
| per-fight K=10 | 982 / 933 / 907 / 837 → 759 | 1026 / 1076 / 1107 / 1190 → 1249 | ±25–27 |
| **per-fight 32/16/10** | **942 / 847 / 837 / 798 → 753** | **1081 / 1151 / 1170 / 1219 → 1251** | **±25–27** |
| per-fight K=16 | 971 / 903 / 875 / 806 → 748 | 1040 / 1104 / 1138 / 1218 → 1255 | ±35–38 |
| Glicko-2 (run = period) | 795 / 762 / 767 / 755 → 730 | 1160 / 1183 / 1191 / 1221 → 1245 | ±26–28 |

(Ratings settle about 50 points inside the true skills because shop luck flattens real win chances. That's correct: the rating predicts actual win rates.)

**Abandon** (settled rating of skill-1000 cohorts; "bad team" quits at round 4 when the team rolled badly, "ahead" quits once ≥70% won after 2+ fights):

| method | honest | quit bad team | quit while ahead |
|---|---|---|---|
| current, abandon not penalised | 979 | 991 (+12) | **1055 (+76)** |
| per-fight, abandon not penalised | 1009 | 1028 (+19) | 997 (−12) |
| current, hearts left = losses | 979 | 942 (−37) | 889 (−90) |
| **per-fight, hearts left = losses** | **1009** | **981 (−28)** | **905 (−104)** |

**What a run is worth** (a 1000 player against 1000 ghosts, champion 1250):

| run | current | **veteran (K=10)** | **first 5 runs (K=32)** |
|---|---|---|---|
| 0-5 | −16 | **−25** | −80 |
| 3-5 | −4 | **−10** | −32 |
| 6-5 | +1 | **+5** | +16 |
| 7-5, then the Crown lost | +3 | **+10** | +32 |
| 9-3, Crown won | +16 | **+38** | +122 |
| 12-0, Crown lost | +16 | **+58** | +186 |
| 12-0, Crown won | +16 | **+68** | +218 |

Long runs carry more fights, so they carry more weight. That's fair: a 12-fight run is more evidence than a 5-fight run, and over many runs a 50% player's total stays at zero (drift table).

## 5. What changes in the code

- `src/mvp/contract.ts`:
  - `MvpRules`: replace `ratingK` with `ratingKSteps: [runsBelow, K][]` (`[[5,32],[15,16],[Infinity,10]]`) and add `botRating: 1000`.
  - `Ghost`: add `rating: number`.
  - `FightResult.opponent`: also pick `rating`.
  - `RunEndReason`: add `"abandoned"`.
  - `RatingChange`: keep `{before, after, expected, actual}`, now meaning Σ E_i and Σ S_i (expected wins vs actual), and add `k`.
  - Update the doc comment.
- `src/mvp/run.ts`:
  - `ratingChange(before, runsPlayed, run, rules)` sums over `run.fights` (round + crown) with `f.opponent.rating`, plus forfeits for `"abandoned"` (`hearts` left × loss against `run.opponent.rating`).
  - Remove `SLAY_BONUS` from the rating (keep `slewChampion` for records).
  - `synthGhost` and `championGhost` stamp `rating`.
  - Add an `abandonRun(state)` helper that ends the run with `"abandoned"`.
- `server/src/mvp/runs.ts`:
  - `ghostOf` stamps the owner's rating at run start: store it on `MvpRunState` at `startRun` (`ratingAtStart`), or `rules.botRating` for bots.
  - `finish()` passes `before.runs` for K.
  - Old ghost rows without `rating` read as `ratingStart`.
  - Champion: stamp the slayer's run-start rating on the `Slay`/`Champion` so the Crown has an opponent rating; default `ratingStart`.
- `server/src/mvp/sqlite-store.ts`: no schema change (ghost JSON carries the field).
- `mobile/main.ts` `runOverScreen`: "Rating 1000 → 1029 (+29) · expected 6.1 wins, got 9" (the second part small or behind a tap, per Maks's "subtle info" rule).
- Tests (`server/src/mvp/runs.test.ts` "MVP run-end rating", `day.test.ts` lines using `ratingChange`):
  - Rewrite the table to the numbers above.
  - Add: a seeded property test that a correctly rated player's mean change is about 0 over many runs.
  - Add: abandoning is never better than losing every remaining heart.
  - Add: beating your own champion is about ±K/2.
- Existing ratings: keep the numbers as they are (all close to 1000); they re-settle within a few runs.

## 6. Left open (for Maks)

- The Crown counts as one fight (recommended). Counting it double would make the Crown matter more but adds noise.
- An abandoned run counts every heart left as a loss (recommended). Counting only one heart would let quitters dodge (+19 in the sim).
- `BOT_RATING` stays 1000 until a day of real play, then gets measured (recommended).
- Not in scope, noted only: rating-based ghost matchmaking (Backpack Battles style) isn't needed for this formula, since Elo handles random opponents.
