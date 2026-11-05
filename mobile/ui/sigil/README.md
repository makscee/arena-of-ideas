# Sigil engine (mission arena#34, slice A)

A unit's portrait is a JSON spec `{body: Layer[], mark?: Layer[]}` drawn on a canvas.
The table's pieces mount them (slice D, below); later slices do the other screens.

- `grammar.ts` the closed expression grammar (known names and functions, compiled once, no eval).
- `spec.ts` types and `validate(spec) -> {ok, problems[]}` (schema, palette, expressions, shape budget).
- `draw.ts` `drawSigil(ctx, spec, state)`: the shared frame, the spec, the caps (16 copies, 48 shapes).
- `live.ts` `mountSigil(canvas, opts)`: one `requestAnimationFrame` loop for every mounted canvas.
- `fallback.ts` the generic spec; `index.ts` `specOrFallback(spec)`.
- `pixels.ts` the pixel checks (inside its circle, fills 44 px, alive): browser only, run by `npm run mvp:sigil`.
- `duel-specs.json` the 16 specs of the UI duel, the validator's real-world fixture.

## Who has a spec

Every live unit has one (slice B, arena#36): `src/mvp/portraits.ts` keys 73 specs by name (the 60 pool units, the 11 summoned bodies, and the Chrysalis's Butterfly and Monarch), and each pool unit and summon in the content carries it as `portrait`. `src/mvp/portraits.test.ts` validates all of them; `npm run mvp:portraits` draws them, runs the pixel checks and fails when two share a silhouette at 44 px (overlap above 0.8), and writes a contact sheet to `e2e/.shots/portraits/`.

## Who writes a spec (slice C, arena#37)

New units get theirs from their LLM: the idea reader's archetypes (`server/src/mvp/idea-reader.ts`, kept on `IdeaArchetype.portrait` and then `StoredUnit.portrait`) and the night designer's fusions (`night-designer.ts`, kept on `FusionDesign.portrait`). `server/src/mvp/portrait.ts` holds the brief, the JSON schema and `resolvePortrait`: `validate`, one more ask with the problem list, else `FALLBACK_SPEC`. The spec rides in the existing JSON: no migration.

## The table's pieces (slice D, arena#38)

`mobile/ui/piece.ts` draws a sigil in place of the emoji on every piece (shop offers, the line, the bench, the battle viewer, and every other screen that builds a `piece`): a `canvas.pc-sg` in the face box, no plate behind it (the stand under it carries the side's colour). The piece mounts it in the shared loop when it is built, and the loop drops it by itself once it leaves the document (a redraw of the table leaves none behind; a canvas not yet attached is waited for 2 s). A unit with no valid spec draws `FALLBACK_SPEC`.

- **Bound state:** `hp` is the fraction left (a battle's live hp over its max; whole on the shop's line), `pwr` the piece's PWR, `charge` and `shield` the stacks of those statuses (a battle passes them from the full list, whatever the shelf has room for).
- **Pulses:** the battle viewer's `motion()` drives them from the wave's events through `sigilOf(piece).pulse(...)`: `hit` (a damage change), `fire` (the unit's rule acts) and `die` (a death change, after the killing blow's shake and freeze), each at the card's own timing and speed. `die` shatters it and the portrait comes back whole for the ghost the card greys.
- **Copies:** a `cloneNode` copy is not mounted; `copySigils(from, to)` paints a still frame onto it (the drag ghost, a fading ghost card).
- `npm run mvp:sigil-table` checks it in a browser: every piece has a canvas, no emoji, the loop draws what is on the page after redraws, a hit and a rule firing reach a sigil.

## Every other screen (slice E, arena#39)

Where a screen used to print a unit's emoji it draws the sigil instead: `mobile/ui/glyph.ts` gives `sigilIcon(spec, px, {live?, fluid?, team?, aw?})` (a canvas: one still frame, or `live` in the shared loop for anything big), `unitSigil(unit, px)` (its own spec, else its pool unit's by id, a fusion's first part's, else the generic one) and `pairSigils(a, b, px)` (the Codex's pair rows). `setPortraits` (called by `setCardFaces`) keeps the specs by unit id.

- **Cards** (`ui/card.ts`): the piece cards were already sigils (slice D); a summon's card now draws its own spec; the candidate card (`unset`, the reading step) mounts a live one; `CardUnit.portrait` carries one. **Sheets:** `sheet-portrait` is the sigil with no plate; a part's label, the summon's name line and the "summoned by" chips lead with a small one.
- **Screens:** Codex (unit chips, pair rows, library cards), forge and claim (the picker, the unit and design cards, partner rows; a design draws its own spec, else its first part's), ideas (`ideaNamed` returns nodes), pick (archetype rows and title, reading cards), vote (the tag), the champion pin and the Champions tab, the battle's damage list, unit sheet head and Why chain, and a unit named in a sentence (`ui/term.ts`'s unit ref, from the summon's spec).
- **Strings:** the `{emoji}` placeholder is gone from every catalog line (`vote.*`, `pick.newVersionTitle`, `ideas.*`, `codex.pairsOf`); the screen puts the sigil beside the name as an element. Text-only strings (a toast, a `title`, a `<select>` option, the Codex's "A + B" sentences) say the name alone.
- **Kept:** server texts that print emoji for players (`daily-post.ts`, the Telegram post) and the share-card PNGs (`server/src/mvp/share.ts`, Twemoji on the server: no canvas there). The dev gallery's `fused: {emoji}` field is not drawn.
- `npm run mvp:sigil-screens` opens 20 scenes in a browser: sigils on the page, none of the live units' emoji left in its text, small ones inked, no page error.
