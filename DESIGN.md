# Arena visual contract

## Status and reading rule

This file backfills the human-accepted visual direction from prototype passes AOI-74 through AOI-77. It is a contract for future mockup passes, not authorization to change production UI. “Accepted” below means carry the decision forward at the level the source records. AOI-77 is only directionally accepted at the high-level layout level; its detail and final aesthetics remain open.

Do not infer unlisted tokens, dimensions, components, behavior, or polish from prototype CSS. Later passes may add decisions only through their own human gate.

**Sources:** [`AOI-S22`](file:///Users/admin/void-vault/workshop/specs/AOI-S22-complete-visual-mockup-system.md), [`visual-retouch roadmap`](file:///Users/admin/void-vault/workshop/artifacts/2026-07-26-arena-visual-retouch-roadmap.md), [`AOI-74`](file:///Users/admin/void-vault/workshop/tickets/AOI-74-shop-board-hierarchy-directions.md), [`AOI-75`](file:///Users/admin/void-vault/workshop/tickets/AOI-75-choose-visual-foundations.md), [`AOI-76`](file:///Users/admin/void-vault/workshop/tickets/AOI-76-settle-shell-run-state-actions.md), [`AOI-77`](file:///Users/admin/void-vault/workshop/tickets/AOI-77-prototype-extensible-unit-card-chassis.md)

## Accepted contract

### Pass 0 — shop and team hierarchy

The run/shop surface reads in this order:

1. run stakes;
2. a horizontal shop supporting three through six offers;
3. a compact selected-Unit decision stage;
4. the persistent five-Unit team as the bottom anchor.

Unit offers are equal square cards. Trigger → Selector → Effect is a vertical grammar. The selected decision and its Buy action remain directly associated with the shop/team comparison, without a memory-heavy gap. Buy and Fight occupy separate decision zones: Buy commits commerce; Fight advances the run.

This is the structural baseline, not finished shop styling. Battle remains only a stable causal-board consequence at this stage.

**Sources:** [`AOI-74 final direction decision`](file:///Users/admin/void-vault/workshop/tickets/AOI-74-shop-board-hierarchy-directions.md#final-direction-decision), [`AOI-74 accepted gallery`](file:///Users/admin/void-vault/workshop/artifacts/aoi-74-shop-board-directions/hybrid-gallery.html), [`AOI-74 A8 evidence`](file:///Users/admin/void-vault/workshop/artifacts/aoi-74-shop-board-directions/hybrid-evidence.md)

### Pass 1 — Quiet Constellation foundation

Carry forward the dark Quiet Constellation direction:

- dark, organic surfaces with guided contrast rather than generic dashboard chrome or indiscriminate neon;
- combat stats and the current action remain prominent;
- Trigger, Selector, and Effect use distinct semantic color roles;
- Units have distinguishable portrait silhouettes or motifs, while portrait ornament never outranks PWR, HP, or the next action;
- semantic card frames communicate role;
- fusion uses an equal, hard two-parent color frame rather than an interpolated single color;
- progression explicitly says `Awakened` at `3/3`.

The original foundation evidence used cyan PWR and coral HP. AOI-77’s later, human-requested final evidence supersedes only that PWR color: **PWR is amber/yellow** (the existing Trigger/progress amber; `#ffc76a` when enabled and muted amber `#e2c980` when disabled), while **HP remains red/coral** (`#ff7d72`). This is semantic color, not permission to derive a wider palette.

**Sources:** [`AOI-75 selection`](file:///Users/admin/void-vault/workshop/artifacts/aoi-75-visual-foundations/selection.md), [`AOI-75 A7 evidence`](file:///Users/admin/void-vault/workshop/artifacts/aoi-75-visual-foundations/quiet-dark-evidence.md), [`AOI-77 final PWR color tweak`](file:///Users/admin/void-vault/workshop/artifacts/aoi-77-portrait-ledger-v2/evidence.md#final-pwr-color-tweak), [`AOI-77 focused check`](file:///Users/admin/void-vault/workshop/artifacts/aoi-77-portrait-ledger-v2/stat-mirror-check-output.txt)

### Pass 2 — Run Spine shell and action grammar

On desktop, use the selected **B — Run spine** shell: a persistent left spine holds navigation, run stakes, opponent, and Fight beside one uninterrupted shop → selected decision → bottom-team column. Stakes and one truthful next action should read before supporting chrome.

Action meaning remains stable across states:

- **Buy** — purchase primary inside the selected decision context;
- **Reroll** — utility; when disabled, keep its reason adjacent;
- **Fight** — run progression, spatially separate from Buy;
- **Retry** — error recovery next to the failure;
- **Continue** — progression after a result;
- **Challenge** — terminal commit with the upside named;
- **Cash out** — destructive/irreversible, with the consequence that it ends the run.

Do not present adjacent competing primaries. Post-battle and terminal actions belong to their states and must not masquerade as shop actions.

**Sources:** [`AOI-76 final direction decision`](file:///Users/admin/void-vault/workshop/tickets/AOI-76-settle-shell-run-state-actions.md#final-direction-decision), [`AOI-76 brief`](file:///Users/admin/void-vault/workshop/artifacts/aoi-76-shell-actions/brief.md), [`AOI-76 state matrix`](file:///Users/admin/void-vault/workshop/artifacts/aoi-76-shell-actions/state-matrix.md), [`AOI-76 action evidence`](file:///Users/admin/void-vault/workshop/artifacts/aoi-76-shell-actions/evidence.md#action-manifest)

### Pass 3 — Portrait Ledger entity chassis (directional, high-level only)

Carry **Portrait Ledger v2** forward for later detail work. Its hierarchy is:

1. portrait and name establish identity;
2. causal behavior follows;
3. PWR and HP share a fixed bottom baseline.

Use full and compact forms. The compact information budget is portrait/name, one truthful behavior cue, PWR/HP, three-step progression, and one fused/status/Awakened recognition line. Full Trigger → Selector → Effect detail hands off to adjacent selected detail or an inspector rather than being compressed into the compact card.

The chassis has bounded optional module slots between recipe and state lanes. Proven module kinds are Creator, Ability II, and Summon; they may be absent or reordered without moving identity, recipe, state lanes, or stats. Long recipes use an elastic recipe field and explicit disclosure to the full recipe.

Fusion preserves equal parent identity by splitting only the portrait 50/50 and naming both parents. The shared name, ordered recipe, modules, progression, runtime statuses, and stats remain continuous; do not bisect the recipe. Progression and runtime statuses occupy separate lanes.

PWR and HP numbers mirror at equal outer corners in full and compact cards, with labels facing inward and a common baseline/type treatment. PWR uses the accepted amber/yellow semantics and HP the accepted red semantics recorded above.

These are accepted high-level allocation and semantics only. The card system is not finally or aesthetically accepted.

**Sources:** [`AOI-77 final bounded human feedback`](file:///Users/admin/void-vault/workshop/tickets/AOI-77-prototype-extensible-unit-card-chassis.md#final-bounded-human-feedback-for-this-pass), [`Portrait Ledger v2 contract`](file:///Users/admin/void-vault/workshop/artifacts/aoi-77-portrait-ledger-v2/evidence.md#v2-system-contract), [`stat-mirror repair`](file:///Users/admin/void-vault/workshop/artifacts/aoi-77-portrait-ledger-v2/evidence.md#final-bounded-human-gate-and-stat-mirror-repair), [`screenshot critique`](file:///Users/admin/void-vault/workshop/artifacts/aoi-77-portrait-ledger-v2/screenshot-critique.md)

### Pass 4 — Causal Atlas with Split Brief comparison header

Carry forward **Causal Atlas hierarchy with Split Brief’s stable comparison header**. Selected detail uses the Causal Atlas composition: identity and state lead into a stable Trigger → Selector → Effect causal sequence, followed by references and deeper proof. The comparison header from Split Brief keeps the selected Unit and owned-team comparison stable while the deeper detail changes. On phone, the inspector becomes a bottom sheet while alternatives remain visible; close/focus return and comparison context remain part of the interaction contract.

The inspector must cover Unit, Ability, Status, and Summon references; base, Awakened, and fused states; equal A+B provenance; and long, empty, disabled, and error states without obscuring alternatives or their consequences. This is an inspector/reference-language decision only, not authorization for a production component or for Pass 5 composition.

**Sources:** [`AOI-79 result and human gate`](file:///root/void-vault/workshop/tickets/AOI-79-prototype-inspector-reference-language.md#human-gate), [`Pass 4 evidence`](file:///root/work/arena-of-ideas/design/aoi-s22-pass-4/evidence.md), [`Pass 4 gallery`](file:///root/work/arena-of-ideas/design/aoi-s22-pass-4/gallery.html), commit `7d70826bacbd5462810f4f074c2f333d5e3bacb4`

### Responsive behavior accepted so far

- At 390px, the shell transforms rather than merely scaling. Run Spine becomes a compact preface, followed by the same shop → selected decision → team reading order.
- Context precedes its action; stakes, one obvious current action, and comparison context remain available without a memory-heavy separation.
- The accepted hierarchy retains square offer comparison and a bottom team anchor. In the latest Portrait Ledger v2 evidence, offers and team become local horizontal scrollers with a partial-next-card continuation cue; the page itself does not horizontally overflow.
- Compact cards use progressive disclosure: their truthful cue selects or hands off to adjacent detail/inspector for the complete causal grammar.

No general breakpoint system, grid measurements, or production card dimensions are settled here.

**Sources:** [`AOI-74 A8 evidence`](file:///Users/admin/void-vault/workshop/artifacts/aoi-74-shop-board-directions/hybrid-evidence.md), [`AOI-76 final direction`](file:///Users/admin/void-vault/workshop/tickets/AOI-76-settle-shell-run-state-actions.md#final-direction-decision), [`AOI-76 brief`](file:///Users/admin/void-vault/workshop/artifacts/aoi-76-shell-actions/brief.md), [`Portrait Ledger v2 evidence`](file:///Users/admin/void-vault/workshop/artifacts/aoi-77-portrait-ledger-v2/evidence.md), [`Portrait Ledger critique`](file:///Users/admin/void-vault/workshop/artifacts/aoi-77-portrait-ledger-v2/screenshot-critique.md)

### States, accessibility, and motion accepted so far

- Selected treatment must not erase a card’s semantic role frame.
- Disabled actions retain an adjacent truthful reason; disabled cards remain recognizable without pretending to be active.
- Error recovery stays adjacent to the error. Irreversible actions state their consequence.
- Long content uses an explicit disclosure/handoff rather than silent clipping. Keyboard focus enters and returns from the proven disclosure interaction.
- Enabled controls have visible keyboard focus. Reduced-motion treatment resolves state changes immediately rather than depending on animation.
- Selection, focus, semantic role, progression, runtime status, and disabled state are separate meanings and should not be collapsed into one decorative treatment.

Prototype checks proved legibility, focus, target, contrast-intent, clipping, and overflow constraints at their tested viewports. They do **not** freeze universal production text sizes, target dimensions, focus thickness, contrast tokens, animation durations, or empty/loading visuals.

**Sources:** [`AOI-75 A7 evidence`](file:///Users/admin/void-vault/workshop/artifacts/aoi-75-visual-foundations/quiet-dark-evidence.md), [`AOI-76 brief`](file:///Users/admin/void-vault/workshop/artifacts/aoi-76-shell-actions/brief.md), [`AOI-76 state matrix`](file:///Users/admin/void-vault/workshop/artifacts/aoi-76-shell-actions/state-matrix.md), [`AOI-77 v2 checks`](file:///Users/admin/void-vault/workshop/artifacts/aoi-77-portrait-ledger-v2/check-output.txt), [`AOI-77 verifier`](file:///Users/admin/void-vault/workshop/artifacts/aoi-77-portrait-ledger-v2/verifier-verdict.md)

## Rejected, superseded, or deferred choices

### Not selected / superseded

- AOI-74’s original A, B, and C wireframe directions are comparison evidence, not the contract; the accepted A8 hybrid supersedes them.
- AOI-75’s Field Ledger and Cutline Forge were not selected. Quiet Constellation is the carried foundation.
- Cyan PWR in AOI-75 and early AOI-76/AOI-77 evidence is superseded by AOI-77’s final amber/yellow PWR tweak. HP remains red/coral.
- AOI-76’s A — Horizon rail and C — Journey brackets were not selected. B — Run spine is the contract.
- AOI-77’s B — Stat Crest and C — Recipe Fold were not selected. Portrait Ledger v1 is refinement history, not the current chassis; v2 is the directional carry-forward.
- AOI-79’s Field Lens remains a rejected browse-first alternative. Standalone Split Brief was not selected as the full composition; only its stable comparison header is carried into the accepted Causal Atlas hybrid.

**Sources:** [`AOI-74`](file:///Users/admin/void-vault/workshop/tickets/AOI-74-shop-board-hierarchy-directions.md), [`AOI-75`](file:///Users/admin/void-vault/workshop/tickets/AOI-75-choose-visual-foundations.md), [`AOI-76`](file:///Users/admin/void-vault/workshop/tickets/AOI-76-settle-shell-run-state-actions.md), [`AOI-77`](file:///Users/admin/void-vault/workshop/tickets/AOI-77-prototype-extensible-unit-card-chassis.md), [`AOI-77 v2 evidence`](file:///Users/admin/void-vault/workshop/artifacts/aoi-77-portrait-ledger-v2/evidence.md)

### Explicitly deferred

- Quiet Constellation’s small visual corrections wait for Pass 12.
- Portrait Ledger motif quality, icon design, final density, detail polish, and final aesthetic acceptance remain open. The AOI-77 card-system program is not closed and production implementation is not authorized.
- Prototype CSS values and specimen dimensions are evidence mechanics, not a production token or component specification.
- Loading and empty-state visual language was not settled by AOI-74 through AOI-77. AOI-76 explicitly used static fixtures and no loading behavior.
- Pass 4 does not settle the high-fidelity shop/team composition, production inspector implementation, production tokens, or later-surface reference layouts. Those remain gated by their own passes and the final Pass 12 acceptance.

**Sources:** [`AOI-75 selection deferred section`](file:///Users/admin/void-vault/workshop/artifacts/aoi-75-visual-foundations/selection.md#deferred), [`AOI-77 ticket`](file:///Users/admin/void-vault/workshop/tickets/AOI-77-prototype-extensible-unit-card-chassis.md), [`AOI-77 screenshot critique`](file:///Users/admin/void-vault/workshop/artifacts/aoi-77-portrait-ledger-v2/screenshot-critique.md), [`AOI-76 state matrix`](file:///Users/admin/void-vault/workshop/artifacts/aoi-76-shell-actions/state-matrix.md)

## Unresolved program: Passes 5–12

None of the following is settled by this document:

- **Pass 5:** high-fidelity shop/team composition;
- **Pass 6:** fusion, Awakening, and run decisions;
- **Pass 7:** battle board and causal replay;
- **Pass 8:** hub, navigation, and account chrome;
- **Pass 9:** Ideas governance and season transition;
- **Pass 10:** tower, leaderboard, and history;
- **Pass 11:** codex and reference flow;
- **Pass 12:** whole-product coherence, deferred polish, and freeze.

Mentions of inspectors, battle/reference reuse, fusion, post-battle, or terminal states above are only constraints or prototype proofs established in Passes 0–3. They do not claim those later surfaces are composed or accepted. Production implementation remains gated on the final Pass 12 gallery and Maks’s acceptance under AOI-S22.

**Sources:** [`visual-retouch roadmap — ordered passes`](file:///Users/admin/void-vault/workshop/artifacts/2026-07-26-arena-visual-retouch-roadmap.md#ordered-passes), [`AOI-S22 acceptance and implementation gate`](file:///Users/admin/void-vault/workshop/specs/AOI-S22-complete-visual-mockup-system.md#acceptance)

## Provenance

| Pass | Ticket | Human outcome | Carried evidence |
|---|---|---|---|
| 0 | AOI-74 | A8 hybrid accepted by Maks | `workshop/artifacts/aoi-74-shop-board-directions/hybrid-gallery.html` |
| 1 | AOI-75 | dark Quiet Constellation directionally accepted; small corrections deferred | `workshop/artifacts/aoi-75-visual-foundations/selection.md` and `quiet-dark-gallery.html` |
| 2 | AOI-76 | B — Run spine selected by Maks | `workshop/artifacts/aoi-76-shell-actions/gallery.html` and ticket decision |
| 3 | AOI-77 | Portrait Ledger v2 high-level layout directionally accepted; footer/color correction accepted; detail deferred | `workshop/artifacts/aoi-77-portrait-ledger-v2/portrait-ledger-v2.html`, `evidence.md`, and focused stat captures/check |
| 4 | AOI-79 | Maks selected Causal Atlas hierarchy hybridized with Split Brief’s stable comparison header | `design/aoi-s22-pass-4/gallery.html`, `evidence.md`, and commit `7d70826bacbd5462810f4f074c2f333d5e3bacb4` |

Passes 0–3 artifact paths in this table are under `/Users/admin/void-vault/`; the Pass 4 artifact paths are repository-relative on the recorded prototype branch. This contract does not elevate neutral galleries, machine checks, or reviewer verdicts into taste authority; they establish provenance and objective conformance only. Maks remains the human visual gate.

**Sources:** [`AOI-S22 authority`](file:///Users/admin/void-vault/workshop/specs/AOI-S22-complete-visual-mockup-system.md), [`AOI-74`](file:///Users/admin/void-vault/workshop/tickets/AOI-74-shop-board-hierarchy-directions.md), [`AOI-75 selection`](file:///Users/admin/void-vault/workshop/artifacts/aoi-75-visual-foundations/selection.md), [`AOI-76 decision`](file:///Users/admin/void-vault/workshop/tickets/AOI-76-settle-shell-run-state-actions.md#final-direction-decision), [`AOI-77 human gate`](file:///Users/admin/void-vault/workshop/tickets/AOI-77-prototype-extensible-unit-card-chassis.md#final-bounded-human-feedback-for-this-pass)
