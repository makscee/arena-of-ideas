# Round 3: units, Awoken forms, fusion stats (notes 1, 5, 17's Awoken sell)

Scouted against the live build 15a93ba9 (`/Users/admin/Work/arena-574/scout-r3`). I edited nothing there. The scratch
scripts are in `scratchpad/v3/sim/`: `dupes.mts` is the duplicate and Awoken sweep, `fusion.mts` is the fusion sim
(400 in-memory bot runs plus 2,000 fights, about 4 s of CPU), and `revive.mts` checks that a status lands on a revived ally.
Run them from the checkout with `node --import tsx/esm <file>`.

Where things live:
- All 81 units are rows in `src/mvp/units.ts:126-217` (`ROWS`). Each row is `r(name, emoji, tier, pwr, hp, when, who, does, awoken)`.
- An Awoken form is **hand-written per row**: `awoken: { who?, does? }`. `unitOf` (units.ts:224) copies the When and
  falls back to the sleeping Who and Does.
- The only shape rule today is `formProblems` (`src/mvp/forms.ts:131-149`). It requires the same When, and it requires
  that the Who or the Does differ in some way. "Revive 2 → Revive 3" passes it.
- Copies and stats: each copy adds +1 PWR / +2 HP (`MVP_RULES.copyGrowth`, `src/mvp/contract.ts:68`), and copy 3
  awakens the unit. So an Awoken unit is base +2/+4.

---

## 1a. Necromancer and Divinity are the same hero

**Today** (units.ts:192 and :206):

| | When | Who | Does | Awoken | Stats |
|---|---|---|---|---|---|
| 💀 Necromancer T3 | an ally dies | the last fallen ally | Revive 2 | Revive 3 | 2/7 |
| 😇 Divinity T4 | an ally dies | the last fallen ally | Revive 3 | Revive 3 + Bless 2 | 2/8 |

Divinity is a strictly better Necromancer with one more HP of revive. Necromancer's Awoken form is "+1 HP on the
revived ally", which is the "+1 hp" Maks saw.

**The change.** Each gets its own job.
- **💀 Necromancer stays the reviver.** Sleeping is unchanged: an ally dies → the last fallen ally → Revive 2.
  Awoken: **Revive 1 + Strength 3**. The dead come back as undead: 1 HP and +3 PWR, glass cannons. `revive.mts`
  confirms that a status after Revive lands on the revived unit: the log shows Summon (resurrected), then
  StatusApplied on the same id.
- **😇 Divinity becomes the protector, built on Blessing (death save).** Sleeping: **an ally dies → all allies →
  Bless 1**. Each survivor will live through its next lethal hit at 1 HP. Awoken: **all allies → Bless 1 + Shield 2**.
  A Shield rider counts as a real change under the rule in (1c), but if Maks wants more, Bless 2 + Shield 2 is the
  bolder version. Chains are safe: Death is a root event, nothing listens to Blessing, and Shield is emitted
  after the root.
- Phoenix (`start/me/Bless 8`) stays the self-blessing unit, and Divinity is the team-blessing one.

## 1b. Other near-duplicates across the roster

From `dupes.mts`. "Signature" means When · Who · the kinds of effect it Does (numbers ignored).

**Same sleeping signature (13 pairs, 27 units):**

| Signature | Units |
|---|---|
| strike · front · Hit | Fighter T1, Duelist T2 |
| start · me · Shield | Squire T1, Distractor T1, Bulwark T2 |
| start · random · Hit | Gnat T1, Sniper T1 |
| die · enemies · Poison | Spore T1, Plague Rat T2 |
| allyHurt · it · Heal | Nurse T1, Physician T3 |
| start · allies · Strength | Coach T1, Commander T3 |
| start · front · Freeze | Taser T1, Mesmerist T3 |
| allyPower · front · Hit | Wire T1, Crusader T3 |
| turnEnd · allies · Heal | Medic T1, Priest T4 |
| strike · front · Poison | Injector T2, Venomancer T2 |
| start · enemies · Hit | Emberling T2, Ruin T4 |
| allySummoned · it · Strength | Pediatrician T2, Fertilizer T4 |
| allyDies · fallen · Revive | Necromancer T3, Divinity T4 |

**Same Awoken signature:** Rot and Famin both end on `turnEnd · enemies · Poison 1`.

**Same job, different When** (worth knowing, not a rule break): six "me · Strength" self-growers (Henchman, Battery,
Berserker, Pathologist, Robber, Lilith), six "random · Hit" pingers, and five "allies · Heal" healers.

**The rule a test enforces** (new test in `src/mvp/units.test.ts`, with the helper exported from `units.ts`):
- `sig(form) = when-key + who-kind + sorted set of effect kinds`. An effect kind is the ability word: Hit, Smite,
  Poison, Shield, Heal, Mend, Strength, Vitality, Curse, Freeze, Bless, Revive, Silence, Call.
- (R1) No two units share a sleeping signature.
- (R2) No two units share an Awoken signature.
- Cross-form collisions (A's Awoken equals B's sleeping, as with Gnat's Awoken and Ruin's sleeping) are printed as a
  warning only.

**Fixes for the sleeping collisions.** These are first-pass proposals: keep R1 and R2 green, then tune with
`npm run mvp:meta -- --quick`.

| Unit | Today | Proposed sleeping |
|---|---|---|
| Duelist T2 | strike/front/Hit 2 | strike/front/**Smite** (strikes again for its PWR, so it grows with copies and Strength) |
| Squire T1 | start/me/Shield 2 | start/me/**Strength 2** |
| Distractor T1 | start/me/Shield 4 | **hurt/me/Vitality 1** (grows each time it is hit) |
| Gnat T1 | start/random/Hit 2 | **turnEnd**/random/Hit 1 |
| Spore T1 | die/enemies/Poison 1 | die/enemies/**Curse 1** |
| Physician T3 | allyHurt/it/Heal 2 | allyHurt/it/**Mend** (heals by its PWR) |
| Commander T3 | start/allies/Strength 1 | **strike**/allies/Strength 1 (rallies on every strike) |
| Mesmerist T3 | start/front/Freeze 2 | start/front/**Freeze 1 + Curse 1** |
| Crusader T3 | allyPower/front/Hit 1 | **turnStart**/front/Hit 2 |
| Priest T4 | turnEnd/allies/Heal 2 | turnEnd/allies/**Mend** |
| Injector T2 | strike/front/Poison 1 | **hurt**/front/Poison 1 (needles when hit) |
| Emberling T2 | start/enemies/Hit 1 | **die**/enemies/Hit 2 (explodes) |
| Fertilizer T4 | allySummoned/it/Strength 2 | allySummoned/it/**Strength 2 + Shield 2** |
| Divinity T4 | see 1a | see 1a |

## 1c. Awoken forms must change the unit, not its numbers

**Today's classes** (81 units):
- **Numbers only, 32 units:** Fighter, Fodder, Spore, Nurse, Wire, Rose, Spike, Medic, Guardian, Enhancer, Battery,
  Injector, Plague Rat, Duelist, Berserker, Emberling, Bloodthinner, Trickster, Stoneskin, Pediatrician, Crusader,
  Lightning, Battle Mage, Pathologist, Virus, Necromancer, Redirector, Keeper, Robber, Ritualist, Doctor, Ruin.
- **Same Does plus a small rider, 36 units:** mostly a +Strength 1, +Vitality 1 or Shield 1 rider, for example
  Squire's Shield 2 → Shield 2 + Strength 1.
- **New Who, 8 units:** one target becomes all enemies (Gnat, Rat, Saboteur, Icebinder, Syren, Rot, Famin), plus
  Silencer, which goes front → random and adds Hit 2.
- **Bigger summon, 4 units:** Planter, Summoner, Sexton, Fungoid (Imp → Wolf, and so on). These are numbers in
  disguise, but the body changes.
- Bat: Heal 1 → Heal 2 + Strength 1.

**The rule (R3, in the same test):** `sig(awoken) ≠ sig(sleeping)`. The Awoken form must either change the kind of
Who (one → all, front → random, me → allies) or add an effect *kind* the sleeping form lacks. A bigger number of the
same thing fails. Today 32 units fail R3, and the 4 summoners pass only because the Call word changes. Write the test
on effect kinds so that "Call Imp → Call Wolf" counts as numbers too.

**Rewrites for the 32 numbers-only units, plus the 4 summoners.** Every row keeps the chain discipline in units.ts:9-16
(listeners emit only events further down the order, and one-unit events act on one unit). Every row also keeps the
friend-or-foe test green. Each one was checked against the R2 collisions by hand; the test is the final word.

| Unit | Awoken today | Proposed Awoken | What changes |
|---|---|---|---|
| Fighter | Hit 3 front | **enemies** · Hit 1 | cleave: every strike hits all enemies |
| Fodder | Shield 2 | Shield 1 + **Bless 1** | its death saves every ally once |
| Spore (new sleeping: Curse 1) | – | Curse 1 + **Poison 1** | |
| Nurse | Heal 2 | Heal 1 + **Vitality 1** | hurt allies grow |
| Wire | Hit 2 front | **Smite** | hits for its PWR, so it grows with pumps |
| Rose | Hit 3 | Hit 2 + **Curse 1** | thorns weaken the attacker |
| Spike | Hit 2 | Hit 1 + **Poison 1** | |
| Medic | Heal 2 | Heal 1 + **Vitality 1** | |
| Guardian | Shield 2 | Shield 1 + **Strength 1** | |
| Enhancer | Strength 2 | Strength 1 + **Vitality 1** | |
| Battery | Strength 2 | Strength 1 + **Vitality 1** | |
| Injector (new sleeping) | – | **enemies** · Poison 1 | |
| Plague Rat | Poison 3 | Poison 2 + **Freeze 1** | its death stuns every enemy |
| Duelist (new sleeping: Smite) | – | Smite + **Curse 1** | |
| Berserker | Strength 2 | **Heal 1** + Strength 1 | |
| Emberling (new sleeping) | – | Hit 2 + **Freeze 1** | |
| Bloodthinner | Hit 2 | **Smite** | scales with PWR |
| Trickster | Hit 3 random | **it** · Smite | punishes the cursed unit |
| Stoneskin | Shield 2 me | **allies** · Shield 1 | its pain shields the team |
| Pediatrician | Strength 2 | Strength 1 + **Bless 1** | newborns get a death save |
| Crusader (new sleeping) | – | **enemies** · Hit 1 | |
| Lightning | Hit 3 | **Smite** (random) | |
| Battle Mage | Hit 3 | Hit 2 + **Poison 1** | |
| Pathologist | Strength 2 | Strength 1 + **Vitality 1** | |
| Virus | Poison 3 | Poison 2 + **Curse 1** | |
| Necromancer | Revive 3 | Revive 1 + **Strength 3** | undead glass cannon (1a) |
| Redirector | Hit 3 | Hit 2 + **Curse 1** | |
| Keeper | Shield 2 | Shield 1 + **Heal 1** | |
| Robber | Strength 2 | Strength 1 + **Vitality 1** | |
| Ritualist | Hit 3 | Hit 2 + **Poison 1** | |
| Doctor | Heal 2 | Heal 1 + **Shield 1** | |
| Ruin | Hit 3 | Hit 2 + **Curse 1** | |
| Famin | Poison 1 enemies (equals Rot's) | random · **Poison 1 + Curse 1** | fixes the R2 collision |

**Loops.** A unit that listens to an ally gaining power must emit only damage (Hit or Smite). A status it emits
loops through Pathologist (enemy poisoned → Strength) or Robber (enemy cursed → Strength). This loop already exists
today: Equalizer (ally powered → Curse) feeds Robber (enemy cursed → Strength), which powers an ally, which fires
Equalizer again, until the chain cap stops it. Add **R4, an acyclic emit graph**: build edges from each form's
listened event to its emitted events and fail on any cycle. Then fix Equalizer and Robber, for example Robber gains
Vitality instead of Strength.

Riders I avoided on purpose: Freeze or Curse on listeners to *frequent* events (an ally shielded or powered, an enemy
cursed). With Keeper or War Drummer they fire about five times a turn and would freeze-lock the front.

**Choice for Maks:** whether a +Strength or +Vitality rider counts as "drastic" (see decisions). If it doesn't, the 36
rider units and the Strength/Vitality rows above need a second pass. Another lever: let the Awoken form *add a second
When* ("also at turn start"). That needs `formProblems` to accept an Awoken When that starts with the sleeping one,
describe and the cards to show two triggers, and the rules text at `mobile/main.ts:95` to change. Fusion already
copies the first part's whole When list.

**Tests:**
- R1, R2 and R3 in units.test.ts.
- The existing pool tests (chain discipline at units.test.ts:31, targeting at :43) stay green.
- `contentFormProblems` returns `[]`.
- `npm run mvp:meta -- --quick` has no unit at 0% and no single team above 40% of the meta.
- The Planter test (units.test.ts:68) still passes.
- A new test: Necromancer's Awoken form revives an ally with 1 HP and Strength 3.

**Content version:** any row change bumps `mvp-<hash>`, so live runs end with "content-changed" on deploy. This is
expected, and the same as round 2.

**Cross-note:** if note 2 moves summons to the front, decide whether Revive (also a `Summon` event, battle.ts:537)
goes to the front too. My recommendation is yes, for one rule.

---

## 5. Fusions have too much HP

**Today:** fusion sums both Awoken units' stats (`fuseUnits`, `src/mvp/forms.ts:105`). Every later copy of either part
still adds +1/+2 (`mergeTarget` and `addCopy`, forms.ts:55-74). Only the front unit strikes (battle.ts:131-149), so
summed PWR and HP piled into one front body is worth far more than the same stats spread over two units.

**Measured** (`fusion.mts`, 200 bot runs that fuse and 200 that never fuse, live rules):

| | PWR / HP |
|---|---|
| Awoken unit at 3 copies, average by tier | T1 3.5/9.1 · T2 3.9/10.3 · T3 4.3/11.4 · T4 4.9/13.2 |
| Awoken unit still in the line at round 12 | 4.5/12.1 |
| **Fused unit at round 12** (82% of bot runs fuse) | **10.0/24.5**, 8.2 copies; HP p50 24, p90 29, max 35 |

The fused unit stands in front in 100% of these lines (bots put the toughest unit first, and so do humans).

**Formulas compared.** A fused unit is rebuilt from its parts' Awoken stats, assuming 3+3 copies at the fuse, plus +1/+2 for each
later copy. Its round-9+ line then fights round-9+ lines from bots that never fuse (500 fights per formula, seeded).

| Formula | Fused-line win rate | Fused HP p50 / p90 | Average turns |
|---|---|---|---|
| sum (today) | **80%** | 23 / 28 | 8.6 |
| **max of each stat +1 PWR / +2 HP** | **62%** | 16 / 21 | 9.4 |
| max + half of the smaller | 67% | 18 / 23 | 9.3 |
| sum PWR, max +2 HP | 69% | 16 / 21 | 8.7 |

**The change (recommended): `fused stats = max(PWR) + 1, max(HP) + 2`.** That is the stronger body plus one copy's
growth (`rules.copyGrowth`, so it follows the tunable). Later copies keep adding +1/+2. A fusion is still a clear
upgrade (62%): two abilities in one slot and a free slot. It just stops being a 24-HP wall.

Implementation:
- In `fuseUnits` (forms.ts:105), compute `{ pwr: max(a.pwr, b.pwr) + rules.copyGrowth.pwr, hp: max(a.hp, b.hp) + rules.copyGrowth.hp }`.
  `fuseUnits` needs `rules`, passed from `run.ts:201`.
- Optional: add a `fusionStats` field to MvpRules so the formula is data. Stored runs carry their own `rules`, so old
  runs would keep summing; a missing field means the new formula, because content changes end old runs anyway.
- Copy changes: rules text `mobile/main.ts:96` ("stats summed" → "the stronger stats, +1 PWR / +2 HP"), the comments
  at contract.ts:122-123 and forms.ts:9-11, and the Codex or rules glossary if it repeats this.
- The fusion preview is served by the server (`/preview`), so the client shows the new numbers on its own.

**Tests that pin today's numbers:** `src/mvp/forms.test.ts:129` (3+4 / 10+8), `:149` (7/18), `:172` (9/22 after
copies) and `:207` (BattleStart hp 18 / pwr 7). These are the only pins. `scripts/meta-health.ts` never fuses
(it builds lines with 3 copies, line 68), so it can't see this change. Keep `fusion.mts` as
`docs/round3/sim/fusion.mts`; with 200 runs it takes about 4 s of CPU, fine on m4.

**Try it:** fuse two Awoken units in a run. The preview shows, for example, 5/13 + 4/11 → 6/15, where today it shows 9/24.

---

## 17. Awoken units sell for 2 (only the part that touches Awoken)

**Today:** every sale gives back `rules.sellRefund = 1` (`src/mvp/run.ts:179`, contract.ts:63). The Sell button reads
the same field (`mobile/main.ts:506`), and so does the rules sheet (main.ts:91).

**The change:**
- Add `sellRefundAwoken: 2` to MvpRules, optional so stored runs fall back to `sellRefund`.
- `sell` gives `u.form === "awoken" ? rules.sellRefundAwoken ?? rules.sellRefund : rules.sellRefund`. A fused unit is
  `form: "awoken"`, so it also sells for 2 (recommended; 3 is the alternative).
- Add a pure `sellValue(rules, unit)` in run.ts that both the server and the client use:
  - the Sell button shows "Sell +2g" per unit;
  - the rules text reads "selling gives back 1, or 2 for an Awoken unit".
- Bots only sell single sleeping copies (bots.ts:130), so they are unaffected.

**Tests:**
- `run.test.ts`: selling a sleeping unit gives +1, an Awoken one +2, a fused one +2.
- An old run without the field still gives +1.
