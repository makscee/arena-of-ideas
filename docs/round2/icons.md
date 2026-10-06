# Keywords and triggers: icons, highlighting, tooltips

Scout: build d8650aa3 (branch mission-574-mvp), read-only. The terms come from the code: `src/mvp/units.ts` (81 units, both forms), `src/content/stress.ts` (the 7 statuses) and `src/describe.ts`. Every icon was rendered and checked at 16, 20 and 48px, on dark and light backgrounds.

Files made:
- `icons/`: 33 cleaned SVGs (black square removed, `fill="currentColor"`, so CSS colours them) and `CREDITS.txt`.
- `glossary.json`: the term table below as data (id, group, label, tone, tip, icon, author, license, url).
- `mockups/icons.html`, `mockups/icons.png` (desktop, dark and light side by side) and `mockups/icons-narrow.png` (420px phone width).
- `build_icons.py`: rebuilds all of the above from one table. `gi-repo/` is a shallow clone of github.com/game-icons/icons.

## The short version

- **The highlighting data already exists, and the client throws it away.** `describeAbilitySegments()` already returns `{text, statusRef?, partRef?}` for every trigger, target, effect and status. But `mobile/ui/card.ts formText()` calls plain `describeAbility()` and gets one string. The battle screen writes statuses as plain text (`"Shield 2 · Poison 1"`, battle.ts:114). `abilityChips()` (the short ⚑/⚔ glyph line) is never used by the MVP client.
- **39 terms in 7 groups:** stats (2), statuses (7 + "stacks"), triggers (11 + "would"), targets (7), effects (6), unit states (3), battle (2). 33 icons cover them. A few terms deliberately share an icon, and 4 need none (see "Terms without an icon of their own").
- **How text looks:** a trigger is an amber pill with the event's icon. A status is its icon plus its word in its own colour, dotted underline. A target is an icon plus its words, teal for allies and pink for enemies (the game's existing you/ghost colours). An amount is the effect icon plus a bold number. Hover shows a tooltip on desktop; click or tap opens a sheet with "Open in Codex".
- **Two rule facts the tooltips must say** (checked in the kernel; the first by running a battle):
  1. A hit that Shield blocks completely still counts as "is hurt", so Rose behind Shield strikes back for free.
  2. Healing a unit at full HP is no heal at all, so "after an ally is healed" doesn't fire.

## Every term, its tooltip and its icon

Icon pages are `https://game-icons.net/1x1/<author>/<icon>.html`. All icons are CC BY 3.0 except heart-plus, which is CC0.

### Stats
| Term | Seen in text as | Tooltip | Icon (author) |
|---|---|---|---|
| PWR | `2 / 5`, "this unit's pwr", "(1 PWR / 2 HP)" | Power: the damage this unit deals with each strike. | broadsword (Lorc) |
| HP | same, "at 2 hp" | Health. When it reaches 0 the unit dies. | hearts (Skoll) |

### Statuses (keywords)
| Term | Tooltip | Icon (author) | Used by |
|---|---|---|---|
| Shield | Blocks damage: each point blocked uses up 1 Shield. | shield (Sbed) | 19 units |
| Vitality | +1 HP for each Vitality, for the rest of the battle. | heart-plus (Zeromancer, CC0) | |
| Strength | +1 PWR for each Strength, for the rest of the battle. | biceps (Delapouite) | |
| Curse | -1 PWR for each Curse, for the rest of the battle (PWR stops at 0). | cursed-star (Lorc) | |
| Poison | At the end of each turn: takes damage equal to its Poison, then Poison drops by 1. | drop (Lorc) | |
| Freeze | Skips its next strike; each skipped strike uses up 1 Freeze. | snowflake-2 (Lorc) | |
| Blessing | The next time it would die, it lives on with HP equal to its Blessing instead. Then the Blessing is gone. | angel-wings (Lorc) | |
| Stacks | The number next to a status. More stacks, stronger effect; some statuses use stacks up. | none: the number is the badge | |

### Triggers (When)
Kernel event to text: `BattleStart` "When the battle begins", `TurnStart`/`TurnEnd` "At the start/end of each turn", `Strike` "After this unit strikes", `Hurt` "After this/another ally is hurt", `Heal` "After an ally is healed", `Death` "After this unit / another ally / an enemy dies", `Summon` "After another ally is summoned", `StatusApplied` "After Shield/Poison/Curse lands on …", `StatChanged` "After an ally gains pwr". `StatusRemoved` exists in the kernel but no MVP unit uses it.

| Term | Tooltip | Icon (author) |
|---|---|---|
| Battle start | Once, when the battle begins, before anyone strikes. | flying-flag (Lorc) |
| Turn start | At the start of every turn. | sunrise (Lorc) |
| Turn end | At the end of every turn, after the front units have struck. | moon (Lorc) |
| Strikes | When it makes its normal attack. Each turn the two front units strike each other. | crossed-swords (Lorc) |
| Is hurt | When it is hit, even if Shield blocks all of the damage. | broken-heart (Lorc) |
| Is healed | When it gets HP back. A unit at full HP can't be healed, so this doesn't fire. | health-normal (Sbed), shared with Heal |
| Dies | When it dies. Its own death ability still fires as it leaves the line. | death-skull (Sbed) |
| Is summoned | When a new unit joins the line: summoned, or revived. | magic-portal (Lorc), shared with Summon |
| <Status> lands | When that status is put on it. | the status's own icon |
| Gains PWR | When its PWR goes up, for example from Strength. | upgrade (Delapouite) |
| "would …" | "When X would …" happens just before X, and can change or stop it (how Shield, Freeze and Blessing work). | none: only appears in status text |

### Targets (Who)
| Term (text today) | Tooltip | Icon (author) |
|---|---|---|
| This unit | The unit that has this ability. | person (Delapouite) |
| That unit (text says "the event's unit") | The unit the trigger was about: the ally who got hurt, the enemy who got poisoned. | pointing (Lorc) |
| Front enemy | The first enemy in line, the one fighting right now. | targeted (Sbed) |
| Random enemy | One living enemy, picked at random. | perspective-dice-six-faces-random (Delapouite) |
| Every enemy | All living enemies. | minions (Lorc) |
| Every ally | All living allies, this unit included. | three-friends (Delapouite) |
| Fallen ally (text says "the most recently dead ally") | The ally who died most recently and is still dead. | tombstone (Sbed) |

### Effects (Does)
| Term | Ability names | Tooltip | Icon (author) |
|---|---|---|---|
| Damage | Hit N | Takes away HP. Shield blocks it first. | spiky-explosion (Lorc) |
| Heal | Heal N, Mend (= PWR) | Gives back lost HP, never above the unit's max. | health-normal (Sbed) |
| Apply status | Shield/Poison/… N | Puts stacks of a status on the target. | the status's own icon |
| Summon | Call Imp/Wolf/Golem/Wraith/Treant | Adds a new unit at the back of the line, if the line has room (5 max). | magic-portal (Lorc) |
| Revive | Revive N | Brings a fallen ally back at the back of the line with that much HP, if there's room. | raise-zombie (Skoll) |
| Silence | Silence | Removes all its statuses and turns off its abilities for the rest of the battle. | silence (Lorc) |

### Unit states and battle
| Term | Tooltip | Icon (author) |
|---|---|---|
| Sleeping | A unit's first form. Owning 3 copies awakens it. | sleepy (Lorc) |
| Awoken | The stronger form, unlocked by the 3rd copy. | third-eye (Lorc) |
| Fused | Two awoken units made into one: When of the 1st, Who of the 2nd, Does of both. | linked-rings (Lorc) |
| Fatigue | From turn 10, every unit takes damage at each turn's end: 1, then 2, 3 … so battles always end. | hourglass (Lorc) |
| Chain stopped | A chain of reactions ran 64 steps and was cut off, so a battle can't loop forever. | breaking-chain (Skoll) |

## How the set holds together

- **All solid silhouettes** from 5 authors (mostly Lorc and Delapouite), all coloured by CSS. They read well at 20px. 16px is the floor, and only in the compact card line.
- **Groups rhyme:**
  - Life: heart = HP, heart+ = Vitality, broken heart = is hurt, cross = heal.
  - Power: sword = PWR, biceps = Strength, chevrons = gains PWR, cursed star = Curse.
  - Ally targets are people (person, three friends, tombstone). Enemy targets are aim and chance (crosshair, dice, horde).
- **A trigger shares its event's icon on purpose.** "Is healed" uses the Heal cross. "Is summoned" uses the Summon portal. "Shield lands" uses the Shield. So in a chain, the cause and the reaction show the same picture: Fodder's [shield] lands, then Spike's [shield] trigger fires.
- **Tones** (dark / light) are in `build_icons.py` `TONES`. They pass on both backgrounds in the screenshot. Strength is orange (it feeds PWR), Curse is pink, Poison is the Poison family purple, Freeze is ice blue, Blessing and Revive are gold, triggers are amber.

**Weakest at 16px (flagged):**
- broadsword and flying-flag are thin. Fallback: render the 20px size in card lines.
- minions vs three-friends are both "a group" and differ mainly by colour, which colour-blind players can't rely on. The word and side still separate them. Fallback: crosshair-group (delapouite/multiple-targets) for enemies, but it is too thin below 20px.
- sleepy is a blob at 16px. That's fine, because Sleeping is the default and per your note should rarely be shown.
- lorc/ankh (classic for Revive) looked like an X next to crossed swords, so I picked raise-zombie instead.

**Terms without an icon of their own:**
- Stacks: the number is the badge.
- "would" (interceptor): wording only.
- Status lands and Apply status: they use the status's icon.

## Wording fixes I'd make in describe.ts at the same time

These came up while mapping the text:
- "heal **the event's unit** for 1" → "heal **that ally** for 1" (or "that enemy" / "it"). The trigger's filter already says which.
- Lowercase "pwr", "hp" ("After an ally gains pwr", "at 2 hp") → PWR and HP as stat terms with icons.
- "silence the front enemy **— strip its statuses and disable its abilities for the battle**" → "silence the front enemy". The explanation moves to the tooltip, which is the point of tooltips.
- "return the most recently dead ally to the back of the line at 2 hp" → "revive the last fallen ally at 2 HP".
- "summon Imp (1 PWR / 2 HP) at the back of this unit's side" → "summon an Imp (1/2)". The Imp is a tappable term showing its card.
- Damage and heal amounts become their own segments, "deal [💥 2] to …", so the number carries the icon. Today "deal 2 damage to " is one segment.

## Token format: what describe.ts emits

Keep `DescribeSegment` and add one field. The joined `text` must still equal the plain sentence, which the existing tests check.

```ts
// src/glossary.ts (pure data, shared by client, codex and server)
export type TermId =
  | `stat:${"pwr" | "hp"}`
  | `status:${string}`                 // registry name: Shield, Poison, ...
  | `trigger:${EventPattern["on"]}`
  | `target:${Selector["kind"]}`
  | `effect:${Effect["kind"]}`
  | `state:${"sleeping" | "awoken" | "fused"}`
  | `battle:${"fatigue" | "chainCapped"}`
  | `term:${"stacks" | "would"}`;

export interface TermDef { label: string; icon: string; tone: Tone; tip: string; more?: string }
export const GLOSSARY: Record<FixedTermId, TermDef>;   // Record over the unions, like parts.ts: a new kind can't ship without an entry
export function termDef(id: TermId, statuses: StatusRegistry): TermDef; // unknown status → tip = describeStatus(def)

// src/describe.ts
export interface DescribeSegment {
  text: string;
  term?: TermId;            // set on every highlighted run; glue text ("to", ", then", ":") has none
  side?: "ally" | "enemy";  // targets, for the teal/pink tint
  clause?: "when";          // all runs of one trigger clause, so "After [Shield] lands on an ally" draws as ONE pill
  statusRef?: string;       // kept (existing)
  partRef?: PartRef;        // kept (existing, codex deep link)
}
```

Example (Spike): `[{text:"After ",clause:"when",term:"trigger:StatusApplied"}, {text:"Shield",clause:"when",term:"status:Shield"}, {text:" lands on an ally",clause:"when",term:"trigger:StatusApplied"}, {text:": deal "}, {text:"1",term:"effect:damage"}, {text:" to "}, {text:"the front enemy",term:"target:frontEnemy",side:"enemy"}, {text:"."}]`

The same `TermId` is used outside sentences: status badges on battle cards, change pills, trace links and Codex rows. So one glossary drives every place a term appears.

## How the client renders it

- `mobile/ui/term.ts`: `richText(segs)` → spans. A run with `term` becomes `<button class="t t-status tone-shield" data-term="status:Shield"><svg><use href="#i-shield"/></svg>Shield</button>`. Runs sharing `clause:"when"` are wrapped in one amber pill. The pill's icon is the status icon when the clause names one, else the trigger icon.
- **Icons:** one inline SVG sprite (`<symbol id="i-…">`, about 40 KB unminified for all 33) injected once at boot. Use them with `<use>` and `fill: currentColor`. There's no icon font and no network fetch.
- **Desktop:** hover or focus shows a small tooltip after about 250ms (icon, name, tip). Click pins a popover with "Used by N units · Open in Codex".
- **Phone:** tap opens the existing `overlay()` bottom sheet with the same content.
- Terms are buttons: Tab reaches them, Enter opens the sheet, Esc closes it.
- **Battle screen:**
  - `bv-status` text becomes icon + stack badges ([shield]3 [drop]2 instead of "Shield 3 · Poison 2").
  - Change pills use the effect or status icon.
  - Each trace link shows the icon of the trigger that fired: "−2 Dummy ← 💔 Rose (is hurt) ← ⚔ Fighter (strike)". Answering "what triggered this" is then reading the icons. `TraceLink.via` plus the source unit's `when` gives the TermId, so no new kernel data is needed.
- **Compact cards** (your "see them all in the shop"): one icon line, `[trigger] › [target] [effect]N`, e.g. Rose 💔 › ⌖ 💥2. The full sentence is in the hover tooltip and the unit sheet. It's in the mockup; the compact-card scout owns the decision.

## Codex glossary entry

The Codex gets a "Keywords" tab with the groups above (Statuses, Triggers, Targets, Effects, Stats, Unit states, Battle). Each row has:
- the 48px icon, name and tooltip line;
- a longer "more" line where useful (Shield: "It also blocks Poison and Fatigue damage. A fully blocked hit still counts as 'is hurt'.");
- the units that use it, as chips that open their cards. The list is computed by scanning every unit's segments for the TermId, so player-made and fused units show up by themselves;
- "how it shows in battle" (the badge or pill).

Every highlighted term in the game deep-links to its row (`#codex/term/status:Shield`). The existing `partRef` codex Part cards can stay as the creator view.

## Attribution the game must show

CC BY 3.0 needs: the author's name, a link to the license, and a note that the icons were changed. The game-icons README asks for "Icons made by {author}". Put this text in the Codex footer and an About/credits screen:

> Icons made by Delapouite, Lorc, Sbed and Skoll from game-icons.net, licensed CC BY 3.0 (https://creativecommons.org/licenses/by/3.0/). Heart-plus by Zeromancer (CC0). Icons were recoloured and their background removed.

`icons/CREDITS.txt` lists each icon with its author and URL. Ship it with the assets and update it whenever an icon is swapped.

## Suggested slices

1. **Glossary + term segments (S):** `src/glossary.ts`, `term`/`side`/`clause` on segments, the wording fixes above, and a test that every TermId in all 81 units' text has a glossary entry.
2. **Rich text + tooltips (M):** `mobile/ui/term.ts`, the sprite, hover tooltip and tap sheet. Card sheet and shop text use `richText`. Credits go in the footer.
3. **Battle icons (M):** status badges, change pills and trace links with trigger icons.
4. **Codex Keywords tab (S):** glossary rows with "used by" lists and deep links. Comes after the Codex itself exists.
