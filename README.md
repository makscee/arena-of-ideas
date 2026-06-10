# Arena of Ideas

A game that makes itself — players create the content, the community curates it, the ladder selects what survives.

An async-PvP auto-battler where heroes and abilities are **data**, composed from atomic parts (triggers, interceptors, selectors, effects). Players create new parts through an LLM interface, a simulation gate checks balance, player votes judge fun, and fusion recombines everything at the table.

**This is v5** — a clean rebuild. The previous version (~3,800 commits of Rust/Bevy/SpacetimeDB) is preserved at tag [`v4-final`](../../tree/v4-final).

## Status

Building the headless battle kernel first: a pure, deterministic `battle(teamA, teamB, seed) → event log` with a causal trace — no client yet. See [SPEC.md](SPEC.md).

```sh
npm install
npm test
npm run typecheck
```
