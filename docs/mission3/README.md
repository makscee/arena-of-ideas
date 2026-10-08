# Mission 3: units evolve

Card: makscee/void-board#800. Plan page: https://m1.twin-pogona.ts.net/r/arena-m3-plan.html. Concept §8: https://m1.twin-pogona.ts.net/r/arena-concept.html. Branch `mission-800-evolve`, cut from `mission-735-ideas` at 65a7aeee (mission 2, PR #691, not yet on main). This file wins over the plan page where they differ.

## Maks's answers (2026-10-08, "agree, go")
1. **Versions compete on the existing vote cards.** Each version that passed the overnight check, and an "unchanged" candidate, is compared with a typical live unit exactly as ideas are (M2-8, `server/src/mvp/votes.ts`). The best-scoring qualified version of an archetype is its entry; "unchanged" enters only by beating every proposal of its archetype.
2. **An unfilled slot goes to the other queue.** Of the daily entrants (`rotationEntrants`, 3), about ⅔ are ideas and ⅓ evolutions/returns (`rotationEvolveShare`, default 1 of 3). When one queue has nothing qualified, its slot goes to the other that day. The pool never shrinks: an entrant still needs a leaver.
3. **Go for slices 3 and 7** (prod data), with the live DB copied first.

## Calls (Maks can overturn any)
- Anyone can propose a version, the original author included; it costs one idea (`spent` in `mvp_idea_counts`).
- Only a `library` unit can be proposed for. Proposals for one archetype collect while it's in the Library.
- A version keeps the name, emoji and archetype line. When → Who → Does may all change; the line is what stays true. The Codex shows "v2", "v3".
- A version is a new unit (a new permanent id) with `parentId` = the version it was proposed from (`StoredUnit.parentId`, `server/src/mvp/store.ts:172`, already there) and a `rootId` = the archetype's first version, kept in the unit's JSON. A unit without `rootId` is its own root, so the 81 existing units need no write.
- "Unchanged" is a candidate that points at the Library version itself (no new row, no new numbers). It exists only while the archetype has at least one proposal in `voting`.
- A unit returns only through a contest it wins: a Library unit with no proposal stays there.
- A failed proposal goes back to its author with the reason, refunded, as ideas do.
- When a version enters, its archetype's other proposals are closed (`library`, not refunded: they competed and lost) and "unchanged" is dropped.
- Credits: "idea by @first-author, evolved by @version-author". Days live count over every version of the archetype (`liveDays` over the root's lineage). Creator number: the first author counts the whole archetype; an evolver counts their own versions' days.
- The minimum stay applies to an entering version like any entrant.
- Out of scope: weekly balance fixes, logins, Russian, Discord, website stats, money.

## The flow
1. **Propose:** Codex → Library → a unit → "Propose a new version" (needs an idea). Text box, the same limits and privacy as an idea. The idea is stored with `kind: "evolve"` and `target` = the Library unit id.
2. **Read:** skips the archetype stage: the archetype is the target's name, emoji and line. The reader writes 3 readings (When → Who → Does with awoken forms) for "a new version of <unit>, its line <line>, its current rule <rule>, and what the player wants changed". Each must pass the existing checks (`idea-checks.ts`) plus: not the same shape as the current version, and a **faithfulness check**: a second reader call answers whether the reading is true to the line (yes/no + reason). A "no" is dropped like a failed check.
3. **Pick:** the author picks 1 of 3 (the M2-6 reading screen, titled "a new version of 🦔 Quillback").
4. **Simulate:** the same overnight check and bar as ideas.
5. **Vote:** the same cards. The card says "new version of Quillback" for a version and "Quillback, unchanged" for the return.
6. **Enter:** the rotation (04:00) fills slots by the split above. An evolution/return entrant gets stint reason `evolution` or `return`, origin `evolution`/`return`; the old version stays `library`.

## Slices
| # | Slice | Waits for | Prod data |
|---|---|---|---|
| 1 | The "attacker" word + near-miss logging | – | |
| 2 | Idea screens: reading card text, disabled New idea | – | |
| 3 | Lineage: rootId, idea kind/target | – | yes |
| 4 | Propose and read (server) | 1, 3 | |
| 5 | Propose screens (client) | 2, 4 | |
| 6 | Versions on the vote cards, "unchanged" | 4 | |
| 7 | Slots at 04:00: split, evolution/return entrants | 6 | yes |
| 8 | Credits and history | 3 | |
| 9 | Bots, e2e, playtest, report (orchestrator) | all | |

## Rules for every slice
- PR into `mission-800-evolve`, never main. Fresh clone: `git clone --depth 1 --single-branch --branch mission-800-evolve https://github.com/makscee/arena-of-ideas.git`.
- Migrations only add (new files in `server/src/mvp/sql/`, named `m3-NN-*.sql`); update the expected migrations list in `pool.test.ts`.
- Never run a bot, e2e or anything that registers, plays or writes against the live instance (arena.makscee.ru, m1 /arena). Local server only.
- The orchestrator (m1/void-board) checks the mission branch and redeploys live after merges, with a DB copy first.
