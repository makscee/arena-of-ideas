# Scout notes: idea → unit pipeline (against 9ef07ac)

## What exists
- **Earn:** nothing. The store (`server/src/mvp/store.ts:20-87`) has runs, ratings and per-unit tallies.
  - `src/ideas.ts` is the old pre-phone ideas table (free text, up/down votes). The phone game never imports it.
- **Archetype:** a unit row has name, emoji, tier and a one-sentence archetype (`src/mvp/units.ts:130-160`, entries at `:166-237`).
  - `archetypeProblems` (`:354`) enforces one sentence, ≤15 words, unique.
- **Grammar:** a closed vocabulary.
  - When has 15 options in `WHEN` (`:27`). Who has 7 in `WHO` (`:48`).
  - Does is one regex of words like "Hit 3" or "Poison 2" (`:113`), plus Smite, Mend, Silence and summons (`:66`).
  - Awoken is `AwokenRow` (`:147`).
- **Checks:**
  - `awokenKeeps` (`:485`) and `awokenNewPart` (`:451`).
  - `units.test.ts`: one unit per shape (R1/R2, `:163-167`), no copied heroes (`:360-428`), no listen→emit loops (R4, `:247`).
  - `formText` (`src/mvp/form-text.ts`) renders a form as the player reads it.
- **Simulation:** `npm run mvp:meta` (`scripts/meta-health.ts`).
  - `breaker()` (`:111`) finds the best team of 5 that contains each unit: "the unit at its best".
  - Equilibrium and health checks are at `:133`, `:170+`.
  - A full run takes about 907 s: about 275k battles at about 3.3 ms each (`docs/mvp/meta-health.md:13`).
  - It judges only Awoken forms at 3 copies, and numbers are tuned by hand (R4 retune: `meta-health.md:60+`).
- **Old creation loop:** `npm run create` (`src/create/cli.ts`, `worker.ts`, `check-candidate.ts`, `gate.ts`).
  - It drives `claude -p` (`src/create/claude-code.ts:125-149`) or an OpenAI-compatible endpoint until a candidate passes a sim gate.
  - It targets the old team-file DSL, so reuse its `claude -p` wrapper, not the rest.
- **Local model on m1:** Qwen3-4B on `mlx_lm.server` :8792 (`scripts/mvp-redeploy.sh:72-76, 178`), reached through `ARENA_NAMER_URL` (`server/src/mvp/fusions.ts:308-331`).

## Hard parts
- **Any content change today ends every run and restarts the ghost pool and stats** (see `pool.md`). M2-2 first.
- **Compiling free text:** ask the model only for enum choices plus name, emoji and archetype line.
  - The public output is then safe by construction; check name and line with `isCrudeName` and `hasCrudeStem`.
  - The input risk is prompt injection steering the result; the checks bound it.
- **Tuning cost:** about 1.9k battles (about 6 s) per breaker try for one unit.
  - A grid of about 20 points for both forms takes about 2–4 min, and the full meta check with the unit takes about 15 min.
  - That suits overnight batches.
- **Shape collisions:** with 73 units, many When·Who·Does shapes are taken, so readings must avoid them.

## Approach per stage
| Stage | Approach | Size |
|---|---|---|
| Earn | finished runs ÷ 3 − spent, cap 3 | S |
| Archetype + readings | one `claude -p` call per stage with a JSON schema; validated with the existing checks; retried or dropped | M |
| Sim + tune | `scripts/tune-unit.ts`: grid or hill-climb with `breaker()`; band + Awoken margin; then full `mvp:meta` with the unit, HEALTHY required | M–L |
| Votes | pairs against a typical live unit chosen by pick rate; Elo + novelty bonus | M |
| Enter | units live in the DB (M2-1); rotation in `endDay` (M2-10); `isNew` for the badge | M |
