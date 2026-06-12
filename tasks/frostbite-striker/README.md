# Creation task: frostbite-striker

You are pointed at this directory to **create one new unit** for Arena of Ideas.
Read only this directory and the files it points at — no other repo context is
needed. The contract is the checks below, not any prompt.

## The idea

See [`idea.txt`](./idea.txt) — a frost-themed front-line striker, plain language.

## What you produce

A single file at **`tasks/frostbite-striker/out/candidate.json`** (create the
`out/` directory if it does not exist), a team file of the form
`{ "units": UnitDef[] }` (1..5 units). It expresses the idea as DSL data — the
`UnitDef` schema is the contract; you never write engine code, only data the
existing pipeline validates.

(The committed `fixtures/` directory holds hand-proven reference candidates —
leave it alone; your output goes in `out/`, which is git-ignored.)

## The DSL

- **Schema (normative):** `src/types.ts` — `UnitDef`, `Ability`, `When`,
  `Selector`, `Effect`, `Amount`, `StatusDef`. Every field a unit may carry is
  there. The available statuses (e.g. `Curse`, `Freeze`, `Poison`, `Strength`,
  `Vitality`, `Shield`) are the keys of the registry in
  `src/content/stress.ts`.
- **Examples:** `examples/team-alpha.json`, `examples/team-beta.json` — valid
  team files built from the same vocabulary. Copy their shape.
- **Validate entry point:** `assertValidContent(units, registry, label)` in
  `src/validate.ts`. A typo'd effect kind, a wrong-context part, or a dangling
  status reference fails loudly here — it never reaches the battle kernel.
- **Describe entry points:** `describeAbility`, `describeStatus` in
  `src/describe.ts` render a unit's rules to English, to read back what you
  built.

## Setup (once, on a clean checkout)

The gauntlet runs under Node + `tsx`. On a fresh clone, install dependencies
first, from the repo root:

```
npm ci
```

## Self-test (the gauntlet)

Run, from the repo root:

```
npm run check-candidate -- tasks/frostbite-striker out/candidate.json
```

It runs both checks and prints a human transcript plus one machine-readable JSON
line. Exit codes: **0** = passed both checks, **1** = validator failed,
**2** = sim gate bounced. The JSON line carries the win-rate numbers — on a
bounce, those are what you adjust against and re-run:

- **underpowered** (pooled win-rate below the band): raise magnitudes.
- **overtuned** (pooled win-rate above the band): lower magnitudes.
- **counter-folded** (pooled win-rate in-band, but one matchup fell below the
  per-matchup floor): the JSON's `foldedTo` names the opponents you fold to —
  shore the unit up against *those* shapes without inflating the matchups you
  already win.

## DONE condition

Emit `out/candidate.json` that:

1. **passes the validator** (content-valid DSL — `assertValidContent` throws
   nothing), **and**
2. **lands inside the win-rate band** vs the reference meta: its overall
   win-rate, swept across the configured seeds against every reference team,
   falls within `[bandMin, bandMax]` (inclusive), **and**
3. **clears the per-matchup floor**: its win-rate in *every* matchup is at least
   `floor`. A candidate that hard-counters one reference team and folds to
   another (high pooled rate, one matchup near 0%) is bounced as
   `counter-folded` — broad viability is the bar, not a lucky average.

The band, floor, and sweep depth are config, not prose — see
[`gate.json`](./gate.json) in this directory (it overrides the `GATE_*` defaults
in `src/tunables.ts`). The reference meta is the fixed set of teams in
`src/content/reference-meta.ts`.

When `check-candidate` exits **0**, the task is done.
