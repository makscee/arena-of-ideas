# Creation task: frostbite-striker

You are pointed at this directory to **create one new unit** for Arena of Ideas.
Read only this directory and the files it points at — no other repo context is
needed. The contract is the checks below, not any prompt.

## The idea

See [`idea.txt`](./idea.txt) — a frost-themed front-line striker, plain language.

## What you produce

A single file in this directory named **`candidate.json`**, a team file of the
form `{ "units": UnitDef[] }` (1..5 units). It expresses the idea as DSL data —
the `UnitDef` schema is the contract; the AI never writes engine code, only data
the existing pipeline validates.

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

## Self-test (the gauntlet)

Run, from the repo root:

```
node --import=tsx/esm src/check-candidate.ts tasks/frostbite-striker candidate.json
```

It runs both checks and prints a human transcript plus one machine-readable JSON
line. Exit codes: **0** = passed both checks, **1** = validator failed,
**2** = sim gate bounced. The JSON line carries the win-rate numbers — on a
bounce, those are what you adjust against (raise magnitudes if underpowered,
lower them if overtuned) and re-run.

## DONE condition

Emit `candidate.json` in this directory that:

1. **passes the validator** (content-valid DSL — `assertValidContent` throws
   nothing), **and**
2. **lands inside the win-rate band** vs the reference meta: its overall
   win-rate, swept across the configured seeds against every reference team,
   falls within `[bandMin, bandMax]` (inclusive).

The band and sweep depth are config, not prose — see [`gate.json`](./gate.json)
in this directory (it overrides the `GATE_*` defaults in `src/tunables.ts`). The
reference meta is the fixed set of teams in `src/content/reference-meta.ts`.

When `check-candidate` exits **0**, the task is done.
