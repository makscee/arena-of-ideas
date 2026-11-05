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
