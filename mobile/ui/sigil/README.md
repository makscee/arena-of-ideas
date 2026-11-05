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

## The 3rd copy and a fusion (slice F, arena#40)

- **`aw`:** a piece passes `aw: 1` to its sigil when the unit is awoken (`PieceUnit.awoken`), so a spec's `aw` terms grow what its designer said (a halo, more arcs, a higher count) on every piece the game draws, with the frame's gold stand and crown. A fused unit is drawn with `aw: 1` too: both its parts were awoken to fuse.
- **`fuse(a, b)`** (`fuse.ts`, the duel's `Sigil.fuse`): `{body: a.body, mark: b.mark}`. A part with no valid spec counts as the generic sigil; if the pair together draws more than the shape budget it keeps `a`'s whole portrait (none of the 60 × 60 live pairs does: `fuse.test.ts` checks them all). `lineToPiece`, the card's piece (`pieceCard`) and the gallery's fused state use it, replacing the two-emoji piece.
- **Visual only:** a fusion's rule, name and stats come from the fusion; nothing here reads or changes them. The designer's own `FusionDesign.portrait` (slice C) is not drawn by the client.

## Release gates (arena#42)

- `npm run mvp:sigil-sheet` builds one self-contained page, `e2e/.shots/sheet/contact-sheet.html` (and a PNG): every portrait at 44 px, sleeping and awoken, both sides, live. A person reviews it; the script only checks that every canvas drew.
- `npm run mvp:sigil-timing` draws 10 units for 5 s each at 390×844 with a 4× CPU throttle (`--rate`, `--seconds`, `--units A,B` to change) and prints frame time per unit beside its shape count, then the 10 at once. The 10 are the costliest per draw. The frame time is the main-thread cost of the live loop's draw callback; canvas rasterising runs off the main thread and is not throttled, so a clean result is no proof of a low-end GPU (the gap between frames shows a frame that came late).
=======
## Words and glyphs (slice G, arena#41)

No emoji is left in a piece's chrome.

- **The When tab is its word alone** (`face.when.label`, 11 px and up, at every size; the old `word` option and the docked panel's "hide the words" rule are gone). Its colour still follows the glyph family.
- **Glyphs** (`sigil/glyphs.ts`): 29 small drawn marks, each a sigil spec (bold flat shapes, the same thick ink outline, the closed palette, no state expressions), drawn by `glyphIcon(name, px)` in `ui/glyph.ts` as one still frame. The names are the closed set in `src/mvp/glyph-names.ts` (`GLYPH_NAMES`). The headline badge, the status chips (`statusGlyph`: Shield, Strength, Vitality, Blessing, Charge, Poison, Curse, Freeze, Silence, anything else the spark), the sheet's "now" chips and the offer's lock use them.
- **Faces hold names.** `Face.when.icon` and `Face.headline.icon` are glyph names; `V2_FACE` and the trigger fallbacks were converted. A face saved with an emoji draws through `LEGACY_GLYPH` (`glyphName`), and `normalizeFace` maps one when a face is checked; an icon that is neither a name nor a known emoji fails `faceProblems`.
- **The night designer** gets the names in its `face` prompt and as an `enum` in its JSON schema; `draftFace` maps a known emoji, else falls back to the trigger's face.
- `npm run mvp:sigil-glyphs` validates all of them in a browser, checks each fills its 16 px box with ink outline and that no two look alike, and writes a contact sheet (`e2e/.shots/glyphs/`). `npm run mvp:sigil-table` also fails on an emoji in a tab, badge, chip or lock.
- **Kept:** the ★ / ✦ marks (text symbols, not emoji) and the card's 💡 "idea by" mark, which is a card line, not the piece's chrome.
>>>>>>> 85a30401df710210981ea8b0815927ec67b56929
