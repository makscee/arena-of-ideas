# Round 3: words, keywords, tiers, card icons, summon cards, Esc

Scout: build 15a93ba9 (`/Users/admin/Work/arena-574/scout-r3`), read-only. Notes 3, 4, 6, 7, 8 and 9, plus the Codex's wording in general.
All 162 unit texts as they read today: `npm run -s mvp:texts` (scripts/unit-texts.ts). Run it again after the change to compare.

What players read comes from three places:
- `src/describe.ts`: every unit sentence (When, Who, Does), built from the content. No unit text is written by hand.
- `src/glossary.ts`: each term's label (pill, tooltip, Codex row, battle badge) and its one-line rule (`tip`, `more`, `SCOPED_EVENT`).
- The client (`mobile/ui/card.ts`, `term.ts`, `screens/codex.ts`, `main.ts` legend and rules) adds labels around them.

---

## (6) Shorter unit text

**Problem.** Sentences are full of filler: "At the end of each turn: heal every ally for 1." Maks wants card-speak: "Turn end", "Ally summoned".

**Today.** Trigger phrases come from `describe.ts:169-215` (`describeWhenSegments`), targets from `describeSelector` (`:289`), effects from `describeEffectSegments` (`:315-372`), and the joining from `describeAbilitySegments`.

**The change.** It is wording only, with no rule change. Keep `DescribeSegment` and its tags as they are: joining the runs must still give the plain sentence, as the tests check.

### Triggers (When). The subject is left out when it is the unit itself.

| Kernel pattern | Today | New |
|---|---|---|
| BattleStart | When the battle begins | **Battle start** |
| TurnStart | At the start of each turn | **Turn start** |
| TurnEnd | At the end of each turn | **Turn end** |
| Strike, striker holder | After this unit strikes | **Strikes** |
| Strike, ally / enemy | After an ally / an enemy strikes | **Ally strikes** / **Enemy strikes** |
| Hurt, holder | After this unit is hit | **Hit** |
| Hurt, otherAlly | After another ally is hit | **Ally hit** |
| Hurt, enemy / any | After an enemy / any unit is hit | **Enemy hit** / **Any unit hit** |
| Heal, holder | After this unit is healed | **Healed** |
| Heal, ally | After an ally is healed | **Ally healed** (see decision 1) |
| Death, holder | After this unit dies | **Dies** |
| Death, otherAlly | After another ally dies | **Ally dies** |
| Death, enemy | After an enemy dies | **Enemy dies** |
| Summon, holder | After this unit is summoned | **Summoned** |
| Summon, otherAlly | After another ally is summoned | **Ally summoned** |
| StatusApplied, status S | After S lands on an ally / an enemy / this unit | **Ally gets S** / **Enemy gets S** / **Gets S** |
| StatusApplied, no status | After a status lands on … | **Ally gets a status** |
| StatusRemoved | After S leaves … | **Ally loses S** / **Loses S** |
| StatChanged gain / loss / any | After an ally gains / loses / changes PWR | **Ally gains PWR** / **Ally loses PWR** / **Ally's PWR changes** |
| Interceptor (status text only) | When the holder would be hit / strike / die / be healed | **Would be hit** / **Would strike** / **Would die** / **Would be healed** (keep "would" as its own run with `term:would`) |
| Several whens | ", or " | unchanged: "Hit, or turn end" |
| Condition holderHpAtMost | while this unit is at 5 HP or less | **at 5 HP or less** |

The R2 "is hit" rule stays. The Hurt event still reads as "hit", never "hurt" or "damaged", because a hit that Shield blocks completely still counts. Only the "is" goes.

Subjects: holder = no subject, otherAlly = "Ally", ally = "Ally" (decision 1), enemy = "Enemy", any = "Any unit".

### Targets (Who)

| Selector | Today | New |
|---|---|---|
| holder | this unit | **self** |
| eventUnit | that ally / that enemy / that unit | **it** (the run keeps its side tint and pointing icon; when the trigger's unit is the holder, **self**) |
| frontEnemy | the front enemy | **front enemy** |
| randomEnemy | a random enemy | **random enemy** |
| allEnemies | every enemy | **all enemies** |
| allAllies | every ally | **all allies** |
| lastDeadAlly | the last fallen ally | **fallen ally** |

### Effects (Does). Damage and statuses lose their verb and read as "amount, word, to whom".

| Effect | Today | New |
|---|---|---|
| damage, const | deal 2 damage to T | **2 damage to T** |
| damage, stat | deal damage equal to this unit's PWR to T | **PWR damage to T** |
| heal | heal T for 2 / for this unit's PWR | **heal T for 2** / **heal T for PWR** |
| applyStatus, const | apply 2 Poison to T | **2 Poison to T** |
| applyStatus, derived | apply Poison equal to its stacks to T | **Poison equal to stacks to T** |
| consumeStacks | consume 2 stacks of Shield | **spend 2 Shield** |
| summon, holder | summon an Imp (1/2) | **summon Imp (1/2)** |
| summon, all allies | summon an Imp (1/2) for every ally, this unit included, while the line has room | **summon Imp (1/2) for each ally, self included, if there's room** |
| summon, other | … on the side of T | **summon Imp (1/2) for T** |
| resurrect | revive the last fallen ally at 2 HP | **revive fallen ally at 2 HP** |
| silence | silence the front enemy | **silence front enemy** |
| cancel | cancel it, consuming 1 stack | **cancel it, spend 1 stack** |
| absorbHurt | absorb the damage up to its stacks, consuming what it absorbs | **block damage up to stacks, spending them** |
| preventDeathHeal | cancel the death and heal the holder to HP equal to its stacks, spending this status | **cancel death, set holder to HP equal to stacks, spend this status** |
| holder in status text | the holder | **holder** |
| stat amount | this unit's PWR | **PWR** (an Amount stat is always the holder's: `types.ts:135`) |
| stacks amount | its stacks | **stacks** |

**Merge rule** (new, in `describeAbilitySegments`): consecutive effects that read "amount, word, to T" (const damage, const applyStatus) and share the same targets are joined with " and " and name the target once: "1 Freeze and 2 damage to front enemy". Any other pair keeps ", then ". The order stays left to right.

### Before and after (real units)

| Unit | Today | New |
|---|---|---|
| ⛑️ Medic | At the end of each turn: heal every ally for 1. | Turn end: heal all allies for 1. |
| 🪴 Gardener, awoken | After another ally is summoned: apply 2 Vitality to that ally, then apply 1 Strength to that ally. | Ally summoned: 2 Vitality and 1 Strength to it. |
| ⚡ Taser, awoken | When the battle begins: apply 1 Freeze to the front enemy, then deal 2 damage to the front enemy. | Battle start: 1 Freeze and 2 damage to front enemy. |
| 🌵 Spike | After Shield lands on an ally: deal 1 damage to the front enemy. | Ally gets Shield: 1 damage to front enemy. |
| 💀 Necromancer | After another ally dies: revive the last fallen ally at 2 HP. | Ally dies: revive fallen ally at 2 HP. |
| 🩸 Leech, awoken | After this unit strikes: heal this unit for this unit's PWR, then apply 1 Strength to this unit. | Strikes: heal self for PWR, then 1 Strength to self. |
| 🌹 Rose | After this unit is hit: deal 2 damage to the front enemy. | Hit: 2 damage to front enemy. |
| 🌱 Planter, awoken | When the battle begins: summon a Treant (1/8), then apply 3 Vitality to this unit. | Battle start: summon Treant (1/8), then 3 Vitality to self. |

The new texts run roughly 40% shorter.

### Client strings that follow
- `ui/term.ts termInfo`: the `/^this unit$/i` special case becomes `/^self$/i`.
- `ui/term.ts clauseInfo`, `screens/codex.ts triggerOf`: "Shield lands" / "leaves" become "Gets Shield" / "Loses Shield".
- `describe.ts TRIGGER_CHIP` / `SELECTOR_CHIP` (`abilityChips`, which the MVP doesn't use): align them with the table, or delete them.

### Tests to update
- `src/describe.test.ts`: the inline strings at :34, :60, :65, :71-77, :112, :245, :295.
- `src/glossary.test.ts`: the KEYWORDS list at :43-46 gets "start", "end", "Strikes", "Hit", "Healed", "Dies", "Summoned", "gets", "gains". Also :124, :133, :145.
- `src/codex.test.ts` (:58, :98), `src/mvp/forms.test.ts`, `mobile/ui/diff.test.ts`, `e2e/probe-refs.mjs` (:26, :36).

New tests:
- (a) A snapshot of all 162 texts (`mvp:texts` output) committed as a golden file.
- (b) No text contains " the ", "this unit", "every " or "each turn".
- (c) Every merged "and" joins effects that share one target.

**How to try.** Run `npm run -s mvp:texts`. Then, in the shop, read Taser, Gardener and Spike's sheets with See Awoken; the underline still marks what changes.

---

## (4) Keywords that stand alone

**Problem.** Some rules name another keyword (Damage says "Shield blocks it first"). If that keyword is removed, the rule lies.

**Rule for the slice.** A status, trigger, effect, target, state or battle rule may name only core words: PWR, HP, stack(s), damage, heal, strike, turn, ally, enemy, line, battle. It never names another status, trigger, effect or battle rule. A rule that spans two keywords lives in the keyword that changes the other's behaviour.

| Entry (glossary.ts) | Today | New |
|---|---|---|
| Shield `tip` / `more` (:69-72) | Blocks damage: each point blocked uses up 1 Shield. / It also blocks Poison and Fatigue damage. A fully blocked hit still counts as being hit. | tip: **Blocks damage of any kind: each point blocked uses up 1 stack.** more: drop it (the "still counts" rule moves to Hit) |
| Poison (:76) | At the end of each turn: takes damage equal to its Poison, then Poison drops by 1. | **Turn end: takes damage equal to its stacks, then loses 1 stack.** |
| Freeze (:77) | … uses up 1 Freeze. | **Skips its next strike; each skipped strike uses up 1 stack.** |
| Blessing (:78-81) | … HP equal to its Blessing … Then the Blessing is gone. | **The next time it would die, it lives on with HP equal to its stacks instead, and loses them all.** |
| Vitality / Strength / Curse | +1 HP for each Vitality … | **+1 HP per stack, for the rest of the battle.** (likewise PWR, −1 PWR) |
| Hit trigger (:95-98) | When damage comes at it: a strike, an ability, Poison or Fatigue. It counts even if Shield blocks all of it. | **When any damage comes at it, from anything. It counts even if all of it is blocked.** |
| Summoned trigger (:101) | When a new unit joins the line: summoned, or revived. | **When a unit joins the line mid-battle, new or brought back.** |
| Gains PWR (:104) | When its PWR goes up, for example from Strength. | **When its PWR goes up, from anything.** |
| would (:105-108) | … (how Shield, Freeze and Blessing work). | **"Would …" happens just before the thing, and can change or stop it.** |
| That unit / it (:115-118) | … the enemy who got poisoned. | **The unit the trigger was about.** |
| Damage (:126) | Takes away HP. Shield blocks it first. | **Takes away that much HP.** |
| Fused (state) | Two awoken units made into one … | keep: fusion is defined over Awoken (core progression) |
| Chain stopped (:150) | … ran 64 steps … | read the number from `MVP_RULES.chainStepCap` (note 19 makes it 32), never a literal |
| `SCOPED_EVENT` (:186-195) | Hurt: "…Poison or Fatigue… Shield…"; Summon: "summoned, or revived"; StatChanged: "…from Strength." | the same rewrites, said of another unit |

**Test.** In `glossary.test.ts`, every `tip` and `more` contains no other keyword's label: every status name, plus Fatigue, Summon, Revive, Silence, Blessing, Awoken (Fused may say Awoken). Self-mentions are allowed. The test builds the list from `STATUS_TERMS` and `GLOSSARY`, so a new keyword is covered automatically.

### The Codex's wording in general (Keywords tab, `screens/codex.ts:395-461`)
- **The label is the words on the card.** Trigger labels become Battle start, Turn start, Turn end, Strikes, Hit, Healed, Dies, Summoned, Gets status, Loses status, Gains PWR. A scoped line (`kw-scope`) gets its own bold label, "Ally dies", above its rule, so a player can find the exact words from a card.
- **Hide rows no unit or shipped status uses.** That is Consume stacks, Cancel, Absorb, Cheat death, Low HP, Status leaves and "would". Today they show as internal jargon with no "Used by". Rows for statuses, stats, unit states and battle rules always show. They return automatically if content starts using them.
- Group titles: "Triggers (When)" becomes **When**, "Targets (Who)" becomes **Who**, "Effects (Does)" becomes **Does**. These match the fusion recipe line "When · Who · Does".
- Summon's tip follows note 2 (front, not back) in whichever slice moves the summon.

**How to try.** Codex → Keywords. Read Shield, Damage, Hit and Poison: none names another keyword. Then pick a card term, Open in Codex: the row it lands on has the card's words as its label.

---

## (7) Tiers in Roman numerals

Every place a tier shows today:

| Where | Today | New |
|---|---|---|
| Card corner, offers and Codex (`ui/card.ts:43`, style `.card .tier` style.css:56, :468) | ●●● dots, 5px font on a phone | **III** in the corner: 8px phone / 11px desktop, letter-spaced, dim; `aria-label` "tier 3" |
| Unit sheet head (`ui/card.ts:172`, an offer you don't own, a Codex unit) | Sleeping · tier 3 | **Sleeping · Tier III** |
| Codex tier filter (`screens/codex.ts:232`) | chips 1 2 3 4 | **I II III IV** (keep testids `codex-tier-3`) |
| Legend "?" (`main.ts:121`) | The dots top right are its tier. | **The numeral top right (I–IV) is its tier.** |
| Rules (`main.ts:91`) | stronger tiers open as rounds pass | **tier II opens in round 3, III in 6, IV in 9** (from `rules.tierOpensAt`) |

Stats, battle and the end card show no tier today; nothing to change there.

**Code.** One helper, `roman(n)` (1-10), in `mobile/ui/card.ts`, used everywhere above.

**Tests.**
- A vitest for `roman`.
- `e2e/mvp-phone.mjs:166` pins "●●●" today; it becomes "III".

**How to try.** Open the shop and the Codex Units tab: corners read I–IV, and the filter reads I II III IV.

---

## (8) Icons for everything in the card's text

**Today.** The card shows one icon: the first When's, top left (`ui/card.ts:84-92` `triggerMark`). The Codex trigger filter (`codex.ts:165`) and the legend use the same mark. Who and Does are only in the sheet.

**Icons already cover the whole MVP pool.**
- Triggers: flying-flag, sunrise, moon, crossed-swords, broken-heart, health-normal, death-skull, magic-portal, upgrade, and the status's own icon for "gets S".
- All 7 targets: person, pointing, targeted, perspective-dice, minions, three-friends, tombstone.
- Effects: spiky-explosion, health-normal, the status icons, magic-portal, raise-zombie, silence.
- Without an icon: Stacks, "would", Consume stacks and Cancel. No MVP unit text uses them, so the card row never needs them.

**The change.** An icon line in the card's top row, in text order: **When, then Who, then each Does**.
- Each icon is in its tone: amber When, a teal or pink Who by side, each Does in its own colour. One idea gets one icon, and repeated icons are dropped (Sniper awoken shows flag, dice, explosion).
- The tier numeral stays top right.
- Phone (62px card): 11px icons, at most 3 (When, Who, first Does), then a small "+" when more.
- Desktop (120px card): 16px icons, at most 5, then "+n".
- Hovering the card on desktop shows a tooltip naming the icons: "Battle start · Front enemy · Freeze · Damage".
- Battle cards show the same line; it replaces the trigger mark there too.
- The row is built from `formSegments(form)` terms: each term's `termIcon`, with a status icon for `status:` runs. So fusions and summons get it free.

**One gap: whose event a trigger is.** "Dies" (Spore) and "Ally dies" (Necromancer) share death-skull. The fix is a 4px pip on the When icon's corner: teal = an ally's event, pink = an enemy's, none = its own. Draw the pip on the sentence's pill too, and in the Codex trigger filter (which then separates "Dies" from "Ally dies"). No new icon is needed.

**New icons.** None for this note. Round-3 features from other notes may want these from game-icons.net (CC BY 3.0, credit Lorc / Delapouite in CREDITS.txt and the Codex foot); check each at 16px first:
- the shop freeze: https://game-icons.net/1x1/lorc/padlock.html
- storing units: https://game-icons.net/1x1/delapouite/locked-chest.html
- sound on/off: https://game-icons.net/1x1/delapouite/speaker.html and https://game-icons.net/1x1/delapouite/speaker-off.html

**Legend.** "Top left: what wakes it" becomes "**Top: what it does in icons: when, who, what.** Its sheet says it in words."

**Tests.**
- A unit test: given a form, `cardIcons(form)` returns the icon ids in order with no repeats. Fighter → crossed-swords, targeted, spiky-explosion. Taser awoken → flying-flag, targeted, snowflake-2, spiky-explosion.
- e2e: on the phone (390px) and the desktop (1280px) shops, the icon line never overflows the card (`scrollWidth <= clientWidth`) for all 81 units, both forms.

**How to try.** Open the shop and the Codex grid: Necromancer shows skull with a teal pip, tombstone, raise-zombie. Spore shows skull with no pip, minions, drop.

---

## (3) A summoned unit's card

**Today.**
- A summon's unit is an inline `UnitDef` inside its ability (`src/mvp/units.ts:66-72` SUMMONS: Imp 1/2, Wolf 2/3, Golem 2/6, Wraith 3/3, Treant 1/8). It has no emoji and only the inert "Strike" ability, so it has no text of its own.
- `describe.ts:350` writes "summon an Imp (1/2)" as one run tagged `effect:summon`. Tapping it shows the Summon rule, never the Imp.
- The Codex lists only `content.units` (`codex.ts:197`).
- In battle, a summon's card falls back to ✨ (`battle.ts:122` `emojiOf`), and a tap opens its change's trace instead of a sheet (`battle.ts:707-714` `openUnit`).
- Nothing tells the player what a summon does. That matters once a summon has an ability, which note 1's redesign may add.

**The change.**
1. **Content.**
   - `MvpContent.summons: SummonContent[]`, built in `mvpPool()` from SUMMONS: `{ id, name, emoji, base, form: UnitForm | null }`.
   - The form is `{ when: def.triggers, who: def.selectors, does: def.abilities }`, or null when the abilities are only "Strike".
   - Emojis: Imp 👺, Wolf 🐺, Golem 🗿, Wraith 👻, Treant 🌳.
   - This changes the content version; let it land with the round's other content changes (notes 1 and 5).
2. **Text.**
   - `describeEffectSegments` splits the summon run: "summon " (`effect:summon`), then "Imp (1/2)" as its own run with a new field `unitRef: "imp"` (and `side`).
   - `richText` draws a `unitRef` run as a button with the summon's emoji. A click opens the summon's sheet.
3. **The summoner's sheet** (`unitSheet`).
   - Under the text, a "Summons" block. It has the summoned unit's compact card and one line: its form's text, or "**No ability: it fights with its PWR / HP.**"
   - It follows See Awoken: Planter shows the Imp, and awoken shows the Treant.
   - It is inline, so no second click is needed on desktop or phone.
4. **The summon's own sheet.** Name, emoji, PWR / HP, its text (or the no-ability line), and "**Summoned by**" chips (the units whose forms summon it). A chip opens that unit.
5. **Codex Units tab.**
   - A "**Summoned**" group after tier IV, with the 5 bodies' cards. The corner tag reads "S" instead of a numeral.
   - The tier filter adds a "Summoned" chip.
   - The Keywords "Summon" row lists the summoners (it does today).
6. **Battle.**
   - A summoned unit's card uses the summon's emoji, and its icon line if it has a form.
   - A tap opens its summon sheet, with live statuses (the same sheet note 15 adds for every battle unit).
   - Map a battle unit to its summon by the `Summon` event's `name` (`types.ts:198`, `resurrected` false). A revived unit keeps its own id and sheet, as today.

**Tests.**
- `mvpPool().summons` has 5 entries. Every summon effect's unit name resolves to one. Each has an emoji.
- describe gives Planter "Battle start: summon Imp (1/2), …" with a `unitRef` run, and the joined text is unchanged.
- e2e: a Codex unit sheet for Planter shows a Summons block with "Imp". The Codex "Summoned" chip shows 5 cards. In a battle with Summoner, clicking the Wolf card opens a sheet titled "🐺 Wolf".

**How to try.** Codex → Planter: the Imp's card and line sit in the sheet. Codex → Summoned → Wolf: "Summoned by Summoner, Fungoid".

---

## (9) Esc everywhere

**Today.** Each screen sets one `onKeys` handler (`ui/dom.ts:113-119`), and only some handle Esc:

| Screen / thing | Esc today | Esc should |
|---|---|---|
| Term popover (desktop) | closes it (`ui/term.ts:291-301`, capture) ✓ | same |
| Term tooltip | any key hides it ✓ | same |
| Any overlay, the **top-most** one: unit sheet, rules, legend, offer sheet, fusion reveal, champion team, live statuses, why panel, term sheet | shop (desktop only): removes the **first** `.overlay` in the DOM, which is the bottom one when two are stacked (`main.ts:854-855`). Battle: closes it ✓. Codex: **ignored** (`codex.ts:155`). Home, stats, run over: **nothing** | close the top-most overlay, through its own close (so the run menu resumes the battle and the offer sheet stays unbought) |
| Abandon confirm | removed (shop desktop only) | = **Cancel** |
| Run menu ☰ | removed (shop), resume (battle) | = **Resume** |
| Shop, nothing open | desktop: fuse preview → pick → selection → run menu ✓. **Phone width: no keys at all** (`onKeys` is set after `if (!desk) return`, `main.ts:839`) | same steps at every width (a narrow desktop window, a tablet with a keyboard) |
| Battle of a run | trace → run menu ✓ | same |
| Battle watched from Home (playoff game, `main.ts:372`) | nothing | trace, then **back to Home** (onDone) |
| Codex | Back ✓ (but not with a sheet open) | phone: the overlay first. Desktop: an inspected unit first (clear the inspector), then **Back** |
| Codex search field | the browser clears it; onKeys ignores inputs | first Esc clears (native), the next blurs, the next acts as above |
| Stats | nothing | champion sheet, then the unit sheet over it, top first; else **Back** (Home) |
| Run over | Enter only | **Home** |
| Home | nothing | close the overlay; else nothing (Home is the title menu) |
| Name screen | nothing | nothing |

**The change** (in `mobile/ui/dom.ts`).
1. `overlay()` keeps a stack. Each entry is `{ el, dismiss }`, where `dismiss` is what a tap outside does (default: close).
   - The run menu passes `resume`.
   - The abandon sheet passes Cancel.
   - `closable` passes close.
2. One global keydown listener, in capture, after the term popover's:
   - Esc with an overlay open calls the top entry's `dismiss` and stops the event.
   - Else Esc in a text field blurs it.
   - Else the screen's `onKeys` gets the key, at **every** width.
3. Screens.
   - The shop moves its Esc branch out of the desktop-only part.
   - Battle adds "no outro: onDone".
   - Codex adds "inspected: clear".
   - Stats and run over add an `onKeys` for Esc.
   - The battle's and the shop's own "overlay open" checks go (the stack does it).
4. Make the order testable: put it in a pure `escStep(state)` that returns "popover" | "overlay" | "blur" | "screen", and unit-test it.

**Tests.**
- vitest for `escStep`.
- Extend `e2e/mvp-desktop.mjs` and `e2e/mvp-phone.mjs` with a phone-width keyboard pass:
  - Home → Rules → Esc closes.
  - Stats → Esc → Home.
  - Codex (390px) → unit sheet → a term's sheet → Esc closes only the term sheet → Esc closes the unit sheet → Esc → Back.
  - Shop (390px) → offer sheet → Esc closes, unbought.
  - Battle → ☰ → Esc resumes play.
  - A playoff game → Esc → Home.
  - Run over → Esc → Home.

**How to try.** On desktop, open any sheet anywhere and press Esc: it closes. Press Esc again: you go back one screen (in a run, the ☰ menu opens). Narrow the window under 1024px and repeat.

---

## Decisions for Maks

1. **Does "Ally" include the unit itself?** The kernel has two filters. 16 units' triggers mean another ally (Nurse, Guardian, Physician: "ally hit"; 10 units: "ally dies"; Gardener, Pediatrician, Fertilizer: "ally summoned"). 11 units' triggers include the unit itself: Spike, Almsgiver, Enhancer, Battle Mage (ally gets Shield); Sanctifier, Battery, Wane (ally healed); Wire, Crusader, Lightning, Equalizer (ally gains PWR).
   - (a) **One meaning:** switch those 11 to "another ally", so "Ally" always means another ally. This is a small content change; rerun `npm run mvp:meta`.
   - (b) Keep the rules; the text says "Ally" or "Any ally" (self counts).
   - (c) Keep the rules; both say "Ally" and only the tooltip tells.
   - Recommend (a): one word, one rule, like "friend" in Super Auto Pets.
2. **Where the card's icon line goes.**
   - (a) **The top row:** When, Who, Does, left to right, with the tier numeral top right (phone: 3 icons + "+").
   - (b) A row under the name (needs a taller phone card: 84 → 96px).
   - (c) Desktop only; the phone keeps today's one trigger icon.
   - Recommend (a).

Calls made without asking (R4): "self" and "it" for the targets; "Hit" (not "Is hit") for the holder; "and" merges effects on a shared target; hide unused jargon rows in the Codex; the scope pip.
