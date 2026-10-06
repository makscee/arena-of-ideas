# Arena v3: the shop (notes 12, 13, 17)

Scouted against build 15a93ba9 (`/Users/admin/Work/arena-574/scout-r3`). Nothing was run against the live
instance. Two scratch sims, both in-memory and under 2 s of CPU: `sim/late-shop.mts` (today's bots, 150 runs) and
`sim/bench-econ.mts` (a shop-only economy, 2000 runs per variant).

## The shop today

| What | Where | Value |
|---|---|---|
| Rules | `src/mvp/contract.ts:56` `MVP_RULES` | 12 rounds, 5 hearts, 10g a round with no carry-over, unit 3g, reroll 1g, **sell 1g flat**, line 5, copy 3 awakens, +1 PWR/+2 HP a copy, tiers open at R1/3/6/9 |
| Offers | `contract.ts:80` `offersAt`, `src/mvp/run.ts:73` `rollOffers` | 3 → 4 (R2) → 5 (R4) → 6 (R7). Drawn uniformly from open units, with replacement, so there is no shared pool and nothing "returns" to it |
| Buy | `run.ts:155` | A copy merges anywhere on the line (`forms.ts:55` `mergeTarget`), even when the line is full. A new unit needs a free slot, else 409 "the line is full" |
| Sell | `run.ts:176` | Removes the unit and every copy in it; `gold += rules.sellRefund` |
| Reroll | `run.ts:170` | Redraws every offer |
| Fuse | `run.ts:189`, `forms.ts:88` | Two Awoken → one final fused unit (frees a slot). Copies of either part still merge into it |
| Runs are stored | `server/src/mvp/sql/04-runs.sql:4` | As JSON with their own `rules`, so new fields need no migration. Old runs read `?? default` |
| Client shop | `mobile/main.ts:410` `shopScreen` | Offers 712–735, Reroll 737, Sell 506, buy refusal `buyBlock` 650, keys 850–905 (1–7, R, Space, ← →, F, S, Esc), rules text 91, legend 121 |
| Bots | `server/src/mvp/bots.ts:107` `botDecision` | Fuse, buy copies, fill with the best tier, sell the weakest single copy for a higher tier, reroll at most twice, toughest unit in front |

**What a late shop looks like** (today's bots, 150 runs, `sim/late-shop.mts`). The line is full in 95% of shop
rounds from round 3 on. Then 93–95% of the offers can't be bought without selling something: only copies can. Late
rounds average about 1.2 copy buys, 0.3 new units and 1.6 rerolls, and bots leave about 4g a round unspent (they stop
at 2 rerolls). That is Maks's "you're just looking for copies".

**Name clash.** "Freeze" is already a live battle status (`src/glossary.ts:77`, snowflake icon, "Skips its next
strike"). Taser, Icebinder, Mesmerist and Mentalist use it, and the Codex lists it as a keyword. A shop button called
Freeze with a snowflake would read as that status. See decision 1.

---

## 12. Keep an offer for later ("freeze")

**How others do it.**
- Super Auto Pets: freeze is per item, pets and food alike, and free. A frozen item stays in the shop through rolls
  and turns until you buy or unfreeze it. Frozen items move to the left-most slots and take up shop slots, so freezing
  leaves fewer fresh offers. ([SAP wiki, The Basics](https://superautopets.wiki.gg/wiki/The_Basics))
- Hearthstone Battlegrounds: one Freeze button freezes the whole tavern for the next turn, for free. Bought slots and
  the extra slots from a tier-up are filled with new minions next turn.
  ([wiki.gg](https://hearthstone.wiki.gg/wiki/Battlegrounds/Freeze), [Icy Veins](https://www.icy-veins.com/hearthstone/hearthstone-battlegrounds-mechanics-guide))
- TFT: "Lock" holds the whole shop for one turn.

Maks asked for one slot ("save a shop slot"), so this follows the per-offer SAP model.

**The rule.**
- Any offer can be locked or unlocked, at no cost, any number of times, in the shop phase.
- A reroll keeps locked offers, moved to the left in their order, and redraws only the others. If every offer is
  locked, the reroll is refused (409 "every offer is locked"), so gold can't be spent for nothing. The client disables
  Reroll in that case.
- A locked offer stays locked until you buy it or unlock it, across rounds, like SAP. At the next round the shop is
  the locked offers plus fresh draws up to `offersAt(round)`. `offersAt` never shrinks, so locked offers always fit.
  The cost of locking is that each locked offer is one fewer fresh offer. That is the decision.
- A locked offer keeps its unit and its price (always `unitCost`). Buying it works like any buy.
- The Crown and an ended run clear the offers, as today (`run.ts:242`, `endIn`).

**Kernel and server.**
- `Offer.frozen?: boolean` in `contract.ts`. Keep the field name even if the UI says Lock; the wording is a UI
  choice. Old stored offers have no field and read as not locked.
- `Decision` gains `{ kind: "freeze"; slot: number }`, which toggles. Add it to `DECISION_KINDS` (run.ts:58).
  409 for no offer in that slot. It is allowed only in the shop phase, which the existing crown guard already covers.
- `rollOffers(s, content)` keeps `s.offers.filter(o => o.frozen)`, then draws `offersAt − kept` new offers and
  renumbers `slot`. The reroll and the round change both go through it. Today the round change calls `rollOffers` at
  run.ts:246, after `s.round += 1`, and the offers still hold the last round's, so this works as long as nothing
  clears them first.
- The buy's slot renumbering (run.ts:167) keeps `frozen`.
- No server or route change. `/preview` accepts it like any non-fight decision. No rules field is needed.
- Bots: in `botDecision`, when a copy is on offer but `gold < cost`, lock it. One line. Without it, humans build
  about 12% stronger lines than the bots whose ghosts they fight (below).

**Client.**
- Phone: the offer sheet gets a **Lock / Unlock** button beside Buy.
- Desktop: the inspector's offer view gets "Lock · L" ("Freeze · Z" if Maks keeps the word), plus right-click on an
  offer card to toggle it.
- A locked card gets a padlock badge on its cost line ("🔒 3g") and a cool outline (`.card.locked`). Locked cards
  sit left.
- Hint, when a copy is on offer but gold is short: "Not enough gold: lock it to keep it for next round."
- Rules text (`main.ts:91`): "Lock an offer to keep it: it stays until you buy it, through rerolls and rounds."
- Add the key to the keys line (`main.ts:794`).

**What it does to balance** (`sim/bench-econ.mts`, a greedy player who rerolls spare gold and locks any copy found
with under 3g). Awakenings per run go 4.4 → 5.2, fusions 2.0 → 2.4, and final line power +12%. That is a real buff to
building, and it is good for fusions. Ghosts and the champion were built without it, so the first days after the
deploy are a bit easier. Teaching the bots to lock closes most of that gap.

**Tests** (`src/mvp/run.test.ts`):
- Lock toggles.
- A reroll keeps a locked offer at slot 0 and redraws only the others.
- A reroll with every offer locked → 409, gold unchanged.
- A locked offer survives the fight into the next round and stays locked, and the shop holds `offersAt(round)` offers.
- Buying a locked offer removes it.
- An old run whose offers have no `frozen` field still rerolls.
- Server `app.test`: POST decision `{kind:"freeze"}` 200, and a bad slot gives 409.
- Phone e2e: lock offer-0, reroll, offer-0 is the same unit.

**How to try.** In round 1, lock an offer and reroll: it stays at the left. Fight. It is still there, locked, in
round 2, with fresh offers beside it.

**Size: S–M** (kernel S, client S). Files: `contract.ts`, `run.ts`, `bots.ts`, `mobile/main.ts`, `style.css`, tests.

---

## 13. More to do once the line is full: storage and decisions

**How others do it.**
- **TFT:** a bench of 9 next to a board that grows with your level. You buy units ahead (to make a 2★ or 3★ later)
  and swap them in per fight. Gold also earns interest and buys levels, which is the economy decision.
- **Hearthstone BG:** a hand of up to 10 as storage, a board of 7. You pay to raise the tavern tier, which is the
  main economy decision. A triple gives a Discover: pick 1 of 3 from the next tier.
- **Super Auto Pets:** no bench, a team of 5. Late game it uses food (buy a buff and choose which pet gets it),
  freeze, and selling for level.
- **Backpack Battles:** a storage box for items that don't fight. The decisions are placement and combining.
- **The Bazaar:** a stash for items off the board, sizes on a 10-slot board, plus events and merchants between fights.

What they share: storage on its own makes buying ahead possible, but the decision comes from **what you do with what
you store**, in three ways. You build toward a combine (TFT/HSBG triples, here awakening and fusion). You swap per
fight (TFT, with the opponent's board in view). Or you get choices handed out (HSBG Discover, SAP food).

**For this game, three options** (rules are exact; numbers are tunables):

### A. Bench of 3 (storage)
- `rules.benchSize: 3`, `RunView.bench: LineUnit[]`, packed with no holes. Bench units don't fight. They are not
  part of ghosts, the Slay, the champion or stats pick rate (those read `line` only).
- **Board slots:** decisions keep their numbers. 0..lineSize−1 is the line, and lineSize..lineSize+benchSize−1 is the
  bench (5, 6, 7). Old clients and bots that only use 0–4 keep working.
- **buy:** a copy merges into its unit wherever it is (line, then bench). It can awaken on the bench. A new unit goes
  to the line if there is room, else the bench, else 409 "your line and bench are full".
- **sell / fuse:** take any board slot. A fuse result keeps first's uid and stands in the front-most line slot of the
  two if either part is on the line, else in the lower bench slot.
- **reorder `{from, to}`:** within one zone it works as today (move and shift). Across zones, if `to` holds a unit the
  two swap. If `to` is an empty slot of that zone, the unit moves there (409 if that zone is full).
- **Crown:** the line is final and the bench doesn't matter. Old runs read `rules.benchSize ?? 0` and `bench ?? []`,
  so runs started before the deploy have no bench.
- **Server:** `fusions.ts` onDecision should queue fusion names for Awoken units on the line *and* the bench, so a
  fuse with a bench unit has its model name ready.
- **Bots must use it,** or the ghost pool falls behind humans. When the line is full and the best offer is a higher
  tier than the weakest single copy, buy it onto the bench (when the bench has room) instead of selling. Before the
  fight, swap the best-scoring bench unit in for the weakest line unit if it scores higher. The fuse search runs over
  every board slot.
- **Client:**
  - A "Bench · 3" row under the line, with smaller cards and dashed empty slots.
  - Desktop: drag between the line and the bench, plus key **B** to send the selected unit to the bench, or to the
    line (into an empty slot, else a swap with the last line unit).
  - Phone: a selected unit gets "To bench" or "To line". When the line is full, the next tap on a line unit swaps.
  - The buy preview says "Goes to your bench".
  - Hint, when the line is empty and the bench isn't: "Move a unit to your line: only the line fights."
  - Rules text: "3 bench slots hold units that don't fight; copies still merge into them."
- **What it changes** (`sim/bench-econ.mts`): with a greedy policy it is power-neutral (line power −1 to −3%). Under
  that policy the share of late offers you can act on goes from 21% to 30%; the 21–30% figures come from this one
  greedy policy, so read them as a direction, not a forecast. The value is options, not power: you can start a second
  fusion without weakening your line, and you can keep a unit for later.
- **Risk:** low for balance. The rules and UI changes are moderate: board slot numbers touch buy, sell, fuse and
  reorder, the bots, and the shop layout on two screen sizes.

### B. A plus "see who you fight" (the bench becomes a sideboard)
- The round's opponent is already fixed at round start (`setOpponent`, runs.ts:67/134), and the fight uses exactly
  that ghost. Show its line in the shop: `RunView.nextOpponent` gains `line` (the ghost's line, from
  `run.opponent.line`).
- The client already draws the foe's line for the Crown (`main.ts:749` `crown-foe`). Round fights reuse it, folded
  under "Next: @x" on the phone and in a side panel on desktop.
- With a bench, every round has a real choice: which 5 of 8 fight *this* team, and in what order. That is the TFT and
  HSBG "look at the next board" decision.
- **Risk:** shop turns get longer, and fights become more about countering. Reload can't re-pick the ghost (it is
  stored), so there is no exploit. Ghosts are other players' saved teams, and you see them in the fight anyway.
- **Size:** S on top of A (one contract field and a view).

### C. A plus a reward for awakening (HSBG Discover)
- The copy that awakens a unit also offers a free pick of 1 of 3 units from the highest open tier. It goes to the
  line or the bench, and is refused if both are full: then you pick nothing, or sell first.
- It needs a pending-choice state (`RunView.discover?: UnitId[]` and a `{kind:"discover", pick}` decision; other
  decisions are refused while it waits), a preview, a bot rule and a chooser UI.
- More decisions, but also a lot more power (one free 3g+ unit per awakening, about 4–5 a run), and it pushes even
  harder toward "find copies", which is the thing Maks finds dull.
- **Size: M.**

**Considered and dropped:**
- Gold carry-over or interest: it needs something to save for.
- Paying to open tiers (HSBG tavern tier): a big rebalance, and it isn't about the late game.
- Buying a 6th line slot: battle balance and card width.
- SAP-style food items: new content and icons, too big for this round.

**Recommendation: A + B.** A is what Maks asked for. B is what turns stored units into a decision every round, for
little extra code. Lock (12) adds a third late decision (lock vs reroll) and works with both. C waits until we see
whether A + B + lock is enough.

**What to measure in the playtest:** late rounds with a non-copy buy, bench use per run, fusions per run (today about
2 per bot run), and how long a shop round takes.

**Tests** (A):
- buy goes to the bench when the line is full.
- A copy merges into a bench unit, and the 3rd copy awakens it there.
- sell from bench slot 5.
- reorder line→bench swap, move into an empty bench slot, and into a full bench → 409.
- fuse line+bench: the result goes to the line slot; bench+bench: the result goes to the bench.
- Fight: the ghost line excludes the bench, and the bench persists across rounds.
- An old run with no `benchSize` has no bench and buy says "the line is full".
- `botDecision` uses the bench.
- Sim check: bot fusions per run don't fall.

Tests (B): the RunView in the shop carries `nextOpponent.line` equal to `run.opponent.line`; at the Crown, the
champion's line.

**How to try:** fill your line, buy a non-copy (it lands on the bench), buy its copies until it awakens on the bench,
fuse it with a line unit. With B: read the next foe's line, swap a bench unit in, fight.

---

## 17. Awoken units sell for 2

**Today.** Selling is a flat `sellRefund: 1` (`contract.ts:63`), added at `run.ts:179`. The client shows it in the
rules text (`main.ts:91`) and on the Sell button (`main.ts:506`, "Sell +1g · S"). Bots sell only single sleeping
copies (`bots.ts:132`), so they aren't affected.

**The change.**
- `MvpRules.sellRefundAwoken?: number` = 2. Add a pure `sellValue(rules, unit)` next to `offersAt` in `contract.ts`:
  `unit.form === "awoken" ? (rules.sellRefundAwoken ?? rules.sellRefund) : rules.sellRefund`.
- A fused unit is `form: "awoken"`, so it sells for 2 too (decision 4).
- run.ts:179 uses `sellValue`.
- A run started before the deploy keeps its stored rules with no field, so it still sells for 1, like the
  `offersGrowAt` precedent.
- Client: the Sell button reads `Sell +${sellValue(rules, u)}g`, using the live rules the client already holds. Only
  a run already going at the deploy can show +2g and get 1; that is harmless. Rules text: "selling gives back 1g, 2g
  for an Awoken or fused unit". With a bench, the same applies to bench units.

**Interactions.**
- Selling an Awoken unit removes it and all its copies. Buying it again starts at 1 copy, sleeping. Offers are drawn
  with replacement, so no pool gets copies back.
- No exploit: an Awoken unit costs at least 9g and returns 2.
- Stats count the line at the run's end only, so they don't change.

**Tests** (`run.test.ts`): sleeping +1, Awoken +2, fused +2, an old-rules run (no field) Awoken +1. Client: the Sell
label shows +2g on an Awoken unit.

**How to try:** awaken a unit and select it. Sell reads "+2g", and gold goes up by 2.

**Size: S.**

---

## Where these collide

- `src/mvp/run.ts`: lock (`rollOffers`, new decision), bench (buy, sell, reorder, fuse) and sell value all touch the
  `applyMvpDecision` switch. Order: 17 → 12 → 13A. 17 and 12 are each about 10 lines and can also land together.
- `mobile/main.ts` `shopScreen`: every slice here edits it, and so do other round-3 notes (Esc, icons on cards,
  roman tiers). Land the kernel parts in parallel, and the client parts one after another.
- `server/src/mvp/bots.ts`: lock (one rule) and bench (bigger). Same file, do them in order.

## Maks's calls (recommendation first)

1. **Name of the shop freeze.** Battle Freeze already exists. **Lock** (padlock, as in TFT, key L), or Freeze
   (snowflake, his word, but it clashes with the status), or Hold.
2. **How long a lock lasts.** **Until bought or unlocked** (SAP), or one round only (HSBG/TFT).
3. **Full-line decisions (13).** **A + B** (bench of 3, and the shop shows the next foe's line), or A alone, or A + C
   (awakening Discover).
4. **A fused unit's sell value.** **2** (it is Awoken), or 3 (two Awoken units went into it).
