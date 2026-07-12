# ADR 0001 — Canonical Trigger → Selector → Ability grammar

Status: accepted (2026-07-11) · Supersedes the pre-v2 Part/axis recipe model

## Decision

The player-facing behavior grammar is `Trigger(s) → Selector(s) → Ability/Abilities`:

- **Trigger** is when the recipe fires (including interceptor timing).
- **Selector** is who or what receives it.
- **Ability** is what happens: a named, referenceable action containing ordered low-level effects.

A base Unit owns one Trigger set and one Selector set and references exactly one Ability. Ability family supplies visual identity; it is not a compositional axis. Unit, Ability, Status, and Summon are the four card-bearing entities. Trigger, Selector, Condition, low-level Effect, and battle event remain inline grammar/runtime structures.

Fusion is a later implementation, but its semantic contract is fixed: first-named parent supplies Triggers, second-named parent supplies Selectors, and the fused Unit carries both inherited Abilities in parent/name order. It creates no Ability and has no permutation chooser.

## Persistence consequence

Content grammar v2 stores Trigger and Selector context on Unit/Status recipes and stores only what happens on Ability. The explicit v1 compatibility reader migrates `unit.ability` plus contextual `AbilityDef.whens/selectors/condition` to v2, preserves effects, and stamps migration provenance. Mixed old/new payloads are rejected rather than guessed.
