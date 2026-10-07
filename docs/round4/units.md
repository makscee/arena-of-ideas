# Units notes (notes 3, 8)

Scout, against f9243387. Approved: all recommended (see README).

## Where things are
- Units: `ROWS` in `src/mvp/units.ts` (~140–245): 81 units (22/27/19/13 by tier), fields name, emoji, tier, pwr, hp, when, who, does, `awoken: {who?, does?}`. The Awoken `who`/`does` **replace** the sleeping ones (`unitOf` ~252): the root of note 8. Summons: `SUMMONS` in the same file.
- No unit archetype field (the "archetype" hits in `src/content/reference-meta.ts` are strategy archetypes). Types: `UnitContent`, `UnitForm` in `src/mvp/contract.ts:146-167`; `UnitForm.text` is unused.
- Text is generated: `src/mvp/form-text.ts`; all texts pinned in `src/mvp/unit-texts.golden.txt` (`unit-texts.test.ts`).
- Shown: card icons (`mobile/ui/card.ts`, `src/mvp/card-icons.ts`); unit sheet ~140–225 with "See Awoken"; Codex (`mobile/screens/codex.ts`, search line 217).
- Rules in `src/mvp/units.test.ts`: R1/R2 unique sleeping/Awoken signatures, R3 Awoken adds something new (**relaxed this round**: numbers-only allowed), R4 no listen→emit loops.
- Pediatrician sits in the tier-3 block but is coded tier 2 (it is cut anyway).

## Archetypes (one sentence each; unique, case-insensitive)
| Unit | Tier | Archetype | Status |
|---|---|---|---|
| Fighter | I | A plain brawler whose every swing adds a jab. |  |
| Fodder | I | A sacrificial body that shields the team when it falls. |  |
| Squire | I | A starter who powers itself up before the fight. |  |
| Gnat | I | A pest that pings a random enemy every turn. |  |
| Spore | I | Its own death weakens the whole enemy line. |  |
| Rat | I | Its own death poisons whoever stands in front. | cut |
| Planter | I | Opens by planting a small body, and grows if there's no room. |  |
| Nurse | I | A medic who patches each ally right after they're hit. |  |
| Prepper | I | Hands the team a little armour before the fight. |  |
| Coach | I | A pep talk: team-wide Strength at the start. |  |
| Bat | I | A flier whose strikes land on a random enemy. |  |
| Taser | I | Opens by freezing the front enemy. |  |
| Wire | I | A wire that numbs the front enemy whenever an ally powers up. | changed |
| Rose | I | Thorns: hits back at the front when struck. |  |
| Victim | I | A taunting tank whose pain powers the team. |  |
| Saboteur | I | Opens by weakening the front enemy. |  |
| Spike | I | Every Shield on an ally makes it lash out at the front. | cut |
| Wither | I | Each enemy kill powers the whole team. | cut |
| Distractor | I | A big decoy that swells each time it's hit. |  |
| Henchman | I | Gets stronger each time a comrade falls (revenge). |  |
| Sniper | I | One big opening shot at a random enemy. |  |
| Medic | I | Steady team heal at every turn end. |  |
| Guardian | II | Puts a Shield on each ally the moment it's hit. |  |
| Almsgiver | II | Turns Shields into heals for the shielded ally. |  |
| Sanctifier | II | Each heal also strengthens the healed ally. |  |
| Enhancer | II | Each Shield also strengthens the shielded ally. |  |
| Battery | II | Charges itself on every ally heal. |  |
| Injector | II | Poisons the front enemy when hit. |  |
| Venomancer | II | Strikes that stack Poison on the front enemy. |  |
| Plague Rat | II | Its own death poisons the whole enemy line. |  |
| Duelist | II | Lunges at the front enemy with full power at the start. |  |
| Berserker | II | Gets stronger with every wound. |  |
| Emberling | II | Explodes on death, hurting every enemy. |  |
| Icebinder | II | Frostbite: every cursed enemy also freezes. | changed |
| Summoner | II | Leaves a wolf behind when it dies. |  |
| Gardener | II | Toughens every newly summoned ally. |  |
| Fruiter | II | Drops a team-wide heal when it dies. |  |
| Leech | II | Heals itself by its damage on every strike. |  |
| Bloodthinner | II | Makes each poisoned enemy bleed at once. |  |
| Hag | II | Each Poison it sees also curses the target. |  |
| Trickster | II | Each Curse sets off damage. |  |
| Custodian | II | Shields the team every time an ally falls. |  |
| Silencer | II | Opens by shutting off the front enemy's ability. |  |
| Scavenger | II | Heals itself off fallen allies. |  |
| Syren | II | A lullaby that freezes a random enemy each turn. | changed |
| Rot | II | Drips Poison on the front enemy every turn. | cut |
| Bulwark | II | A wall that starts heavily shielded. |  |
| Stoneskin | II | Hardens with every hit it takes. |  |
| Pediatrician | II | Strengthens every new summon. | cut |
| Commander | III | A captain whose every strike shields the line. | changed |
| War Drummer | III | A team Strength ramp every turn. |  |
| Physician | III | Saps the front enemy every turn. |  |
| Crusader | III | A heavy hit at the front every turn. |  |
| Lightning | III | Ally power-ups call down random strikes. |  |
| Battle Mage | III | Ally Shields fire random bolts. |  |
| Pathologist | III | A numbing toxin: every poisoned enemy also freezes. | changed |
| Plague Doctor | III | Poisons the whole enemy line at the start. |  |
| Virus | III | Each kill spreads Poison to all enemies. |  |
| Necromancer | III | Raises fallen allies. |  |
| Sexton | III | Digs a fresh body for every ally that falls. |  |
| Fungoid | III | Sprouts an Imp every turn. |  |
| Mesmerist | III | Opens by locking down the front enemy. |  |
| Redirector | III | Reflects hits at a random enemy. | cut |
| Keeper | III | Shields the team every turn. |  |
| Wane | III | Each ally heal saps the front enemy. |  |
| Harvest | III | Each kill feeds the team a heal. |  |
| Robber | III | Steals HP whenever an enemy is cursed. |  |
| Ritualist | III | Each ally death becomes damage to all enemies. |  |
| King | IV | Grants max HP to the whole team at the start. |  |
| Priest | IV | Avenges the fallen by silencing an enemy. |  |
| Divinity | IV | Each ally death blesses the team against dying. |  |
| Phoenix | IV | Comes back from death, many times over. |  |
| Lilith | IV | Grows on every kill. |  |
| Famin | IV | Slowly poisons random enemies. |  |
| Mentalist | IV | Each ally death freezes all enemies. |  |
| Equalizer | IV | Ally power-ups sap the front enemy. |  |
| Director | IV | Each ally death powers the team. | cut |
| Doctor | IV | Heals the team each time it's hit. | cut |
| Ruin | IV | Opens with damage to the whole enemy line. |  |
| Fertilizer | IV | Arms and armours every new summon. |  |
| Morbid | IV | Each ally death weakens all enemies. |  |

## Cuts and changes (approved)
- Cut: Wither, Director, Doctor, Rat, Rot, Spike, Redirector, Pediatrician → 73 units. Check summons, fusion names, bots and docs that reference them.
- Commander: strikes → 1 Shield to all allies. Pathologist: enemy gets Poison → 1 Freeze to it. Syren: turn start → 1 Freeze to a random enemy. Icebinder: enemy gets Curse → 1 Freeze to it. Wire: ally gains PWR → 1 Freeze to the front enemy (power→Shield would loop via Enhancer). Each passes R4 (Freeze emits nothing).
- Kept on purpose: Fodder/Custodian, Spore/Morbid (own death vs any ally's death).

## Awoken builds on the sleeping form
Widening the target (front/random → all enemies, self → all allies) and bigger summoned bodies count as enhancing: Fighter, Gnat, Saboteur, Syren, Stoneskin, Wolf→Warg, Wraith→Ghoul, Imp→Puffball, Imp→Treant are fine.

Today one form has one Who for all its Does (`types.ts unitActionsOf`; `battle.ts:348` applies every selector to every effect), so the fix needs an **"and" clause**: a second Does with its own Who on the same When. Proposed Awoken for the 14 that drop part of the sleeping effect:

| Unit | Today (sleeping → Awoken) | Awoken that keeps sleeping |
|---|---|---|
| Spore | Curse all → Freeze+Curse front | Curse 1 all, and Freeze 1 front |
| Distractor | Vitality self → Curse front | Vitality 1 self, and Curse 1 front |
| **Henchman** | Strength 2 self → Silence front | Strength 2 self, and Silence front |
| Guardian | Shield it → Poison front | Shield 1 it, and Poison 1 front |
| Almsgiver | Heal it → Poison random | Heal 1 it, and Poison 1 random |
| Sanctifier | Strength it → Smite front | Strength 1 it, and Smite front |
| Battery | Strength self → Freeze random | Strength 1 self, and Freeze 1 random |
| Gardener | Vitality it → Poison random | Vitality 2 it, and Poison 2 random |
| Fruiter | Heal 3 → Heal 2 + Bless | Heal 3 + Bless 1 |
| Bloodthinner | Hit it → Curse random | Hit 1 it, and Curse 1 random |
| Trickster | Hit 2 random → Hit 2 it | Hit 2 random, and Hit 1 it |
| Silencer | Silence front → Silence+Hit random | Silence + Hit 2 front |
| Commander | (redesigned above) | its new form plus a bump or "and" |
| Robber | Vitality self → Hit front | Vitality 1 self, and Hit 1 front |

Traps: an Awoken that adds Shield from a Curse or Power trigger creates an R4 loop. Weak "+1 rider" Awokens today (Fodder, Victim, Enhancer, Berserker, War Drummer, Pathologist, Keeper, Custodian, Medic, Nurse, Doctor, Coach, Wither, Harvest) may stay or get a bigger bump; numbers-only is allowed now.

## Tests
- Archetype: required `archetype: string` on `Row`/`UnitContent`; non-empty, one sentence ending in ".", ≤ ~15 words, unique case-insensitive, printed in the golden file. Optional warning on a matching role key (when-family · who-side · effect family), with K/O as known exceptions.
- Awoken: additive shape `{ more?, add?, also?: {who, does} }` applied on top of sleeping; a test that no number goes down and every sleeping part is still there (same word, ≥ number, same or wider Who; summoned body ≥ pwr/hp).
