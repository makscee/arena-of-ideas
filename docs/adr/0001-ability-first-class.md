# ADR 0001 — Ability is a first-class entity; the mechanical entities are {Ability, Summon, Status}

Status: accepted (2026-06-27) · Decider: Maks · Builds: PRD #081 · Supersedes #074's entity model

Promoted into the repo from the vault ADR (`adr-ability-first-class`) as PRD #081's slice 1 first touched `SPEC.md`.

## Context

#074 pinned entities = {Unit, Status, Part} (Part = Trigger / Interceptor / Condition / Selector / Effect) and **explicitly killed "Ability"** — the reusable atom was the Part (e.g. `Deal Damage`), never a frozen combo; the trigger→selector→effect grouping survived only as internal data rendered to the player as a sentence. Its rationale: per-unit behavior variety, no frozen combos, card-size as the complexity budget.

## Decision

Bring **Ability** back as a first-class, named, referenceable entity. The mechanical entities that each pack a mechanic and can be referenced are **{Ability, Summon, Status}**. A **Unit references exactly one Ability**, and that Ability **defines the unit's color**. Effects are never stray — every effect is packaged inside a named Ability. **Fusion: only units with *different* Abilities may fuse** (you combine colors/mechanics; same-ability stacking is disallowed).

## Why (the trade-off #074 didn't weigh)

1. **Legibility** — a named Ability with no loose effects reads cleaner than a freeform part-bag.
2. **Color = identity for free** — ability→color hands the generative-graphics pillar an identity axis at zero extra art cost. The 7 ability families and their pinned hexes (the palette lives in `tunables.ts`): Poison `#a06bff`, Strike `#ff7a4d`, Shield `#4d9bff`, Summon `#25e6d4`, Arcane `#e056fd`, Control `#6b8cff`, Heal `#33d98a`.
3. **Fusion** — "different-Ability-only" is a clean recombination rule that makes fusion about combining distinct colors.

## Consequences

The kernel content ontology in `SPEC.md §1` changes: Ability/Summon/Status are the referenceable types; a Unit is stats (hp/pwr) + one Ability ref. The migration is **behavior-preserving** — `battle()` produces byte-identical event logs for the migrated seed content (pinned by `src/golden-migration.test.ts`).

**Fusion reconciliation (the ADR's open nuance, pinned at build):** the mockup's *"one ability, slots stack"* and this ADR's *"different abilities fuse"* are two views of one rule — a fused unit presents exactly one Ability (one color); fusing units of *different* Abilities is what's allowed, and absorbed parents ride as stacked slots under the presented Ability. v1 ships the **gate + single-color presentation**; the slot-stacking budget stays deferred (`SPEC §8`).
