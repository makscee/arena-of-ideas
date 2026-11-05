# Sigil engine (mission arena#34, slice A)

A unit's portrait is a JSON spec `{body: Layer[], mark?: Layer[]}` drawn on a canvas.
Nothing in the client mounts one yet; later slices do.

- `grammar.ts` the closed expression grammar (known names and functions, compiled once, no eval).
- `spec.ts` types and `validate(spec) -> {ok, problems[]}` (schema, palette, expressions, shape budget).
- `draw.ts` `drawSigil(ctx, spec, state)`: the shared frame, the spec, the caps (16 copies, 48 shapes).
- `live.ts` `mountSigil(canvas, opts)`: one `requestAnimationFrame` loop for every mounted canvas.
- `fallback.ts` the generic spec; `index.ts` `specOrFallback(spec)`.
- `pixels.ts` the pixel checks (inside its circle, fills 44 px, alive): browser only, run by `npm run mvp:sigil`.
- `duel-specs.json` the 16 specs of the UI duel, the validator's real-world fixture.

## Who writes a spec (slice C, arena#37)

New units get theirs from their LLM: the idea reader's archetypes (`server/src/mvp/idea-reader.ts`, kept on `IdeaArchetype.portrait` and then `StoredUnit.portrait`) and the night designer's fusions (`night-designer.ts`, kept on `FusionDesign.portrait`). `server/src/mvp/portrait.ts` holds the brief, the JSON schema and `resolvePortrait`: `validate`, one more ask with the problem list, else `FALLBACK_SPEC`. The spec rides in the existing JSON: no migration.
