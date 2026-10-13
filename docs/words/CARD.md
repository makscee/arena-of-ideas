Forge the word **{{word}}** (arena rules v2, design §7). A word is a new primitive of the language: a designer asked for it instead of bending a design. Low-risk work: a worker builds it, runs the meta, merges its own PR.

**Asked for by**
{{requests}}

**What to build.** Read the parts above and decide the smallest word that lets those designs say what they meant. Name its kind in camelCase. If the asks need two different things, forge the one that fits most and say the other in Next.

**Checklist** (the scaffold does steps 1–2: `npm run word:new -- <selector|condition|amount|effect> <kind> "<what it means>" --word "{{word}}" --card <this card>`)
1. `src/words/<kind>.ts`: how it applies, its English and Russian templates, its trace line, an example battle. Registered in `src/words/index.ts`; its kind added to the union in `src/types.ts`.
2. `registry/forged-words.json`: `{ "word": "{{word}}", "kind": "<kind>", "card": <this card> }` (the Dictionary reads it; a pair waiting on the word un-seals once it is listed).
3. If it has parameters: a case in `src/validate.ts`. If it is an effect that lands in the kernel: its case in `src/battle.ts`, and a line in `SPEC.md` §11.
4. Test and golden: `UPDATE_WORD_GOLDENS=1 npx vitest run src/words/words.test.ts -t <kind>` writes `src/words/goldens/<kind>.jsonl`. Read the log: it must show the word doing what its sentence says. The registry test (`npx vitest run src/words`) runs the example, the EN and RU sentences and the trace line.
5. Meta rerun: `npm run mvp:meta` on the base commit and on yours (m4, ~11 min). A new word changes no existing unit, so the two should match; say so in the PR, or say what moved.
6. `npm run mvp:check`, then the PR into the mission branch (or main). A PR that is live once merged says `Closes`.

**After the merge** the next night batch redesigns the pairs that were waiting, now allowed to use the word. The word is *found* the first time a claimed design uses it; the Dictionary credits the claimer.
