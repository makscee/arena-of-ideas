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

## The 3rd copy and a fusion (slice F, arena#40)

- **`aw`:** a piece passes `aw: 1` to its sigil when the unit is awoken (`PieceUnit.awoken`), so a spec's `aw` terms grow what its designer said (a halo, more arcs, a higher count) on every piece the game draws, with the frame's gold stand and crown. A fused unit is drawn with `aw: 1` too: both its parts were awoken to fuse.
- **`fuse(a, b)`** (`fuse.ts`, the duel's `Sigil.fuse`): `{body: a.body, mark: b.mark}`. A part with no valid spec counts as the generic sigil; if the pair together draws more than the shape budget it keeps `a`'s whole portrait (none of the 60 × 60 live pairs does: `fuse.test.ts` checks them all). `lineToPiece`, the card's piece (`pieceCard`) and the gallery's fused state use it, replacing the two-emoji piece.
- **Visual only:** a fusion's rule, name and stats come from the fusion; nothing here reads or changes them. The designer's own `FusionDesign.portrait` (slice C) is not drawn by the client.
